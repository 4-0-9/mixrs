use anyhow::Result;

use std::{
    borrow::{Borrow, BorrowMut},
    collections::HashMap,
    fs,
    io::Read,
    os::unix::net::UnixListener,
    path::Path,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
    time::Duration,
    usize,
};

use pulse::{
    callbacks::ListResult,
    context::{
        subscribe::{Facility, InterestMaskSet, Operation},
        FlagSet,
    },
    mainloop::standard::{IterateResult, Mainloop},
    volume::ChannelVolumes,
};

use crate::{
    instructions::MixerInstruction,
    playerctl::{playerctl_next, playerctl_play_pause, playerctl_previous},
    pulseaudio::{PulseInstruction, SinkInputMixerData},
    utils::{get_sink_input_name, percentage_to_total_volume, send_notification, total_volume_to_percentage, volume_to_percentage},
};

pub struct Mixer {
    sink_inputs: HashMap<u32, SinkInputMixerData>,
    selected_index: Arc<Mutex<Option<usize>>>,
    mainloop: Mainloop,
    context: pulse::context::Context,
}

impl Mixer {
    pub fn new(mut mainloop: Mainloop, pulse_ix_tx: Sender<PulseInstruction>) -> Self {
        let mut context =
            pulse::context::Context::new(&mainloop, "Mixrs").expect("Error creating pulse context");

        context
            .borrow_mut()
            .connect(None, FlagSet::NOFLAGS, None)
            .expect("Error connecting pulse context");

        loop {
            match mainloop.borrow_mut().iterate(false) {
                IterateResult::Quit(_) | IterateResult::Err(_) => {
                    panic!("Iterate state was not success, quitting...");
                }
                IterateResult::Success(_) => {}
            }
            match context.borrow().get_state() {
                pulse::context::State::Ready => {
                    break;
                }
                pulse::context::State::Failed | pulse::context::State::Terminated => {
                    panic!("Context state failed/terminated, quitting...");
                }
                _ => {}
            }
        }

        let sink_inputs: HashMap<u32, SinkInputMixerData> = HashMap::new();

        let selected_index: Arc<Mutex<Option<usize>>> = Arc::new(Mutex::new(None));

        context.subscribe(InterestMaskSet::SINK_INPUT, |_| {});

        context.set_subscribe_callback(Some(Box::new(move |facility, operation, index| {
            let Some(facility) = facility else {
                return;
            };

            let Some(operation) = operation else {
                return;
            };

            let Facility::SinkInput = facility else {
                return;
            };

            pulse_ix_tx
                .send(match operation {
                    Operation::New => PulseInstruction::AddSinkInput(index),
                    Operation::Changed => PulseInstruction::UpdateSinkInput(index),
                    Operation::Removed => PulseInstruction::RemoveSinkInput(index),
                })
                .unwrap();
        })));

        Self {
            sink_inputs,
            selected_index,
            mainloop,
            context,
        }
    }

    pub fn create_socket_listener(&self) -> Result<UnixListener> {
        let socket_path = Path::new("/tmp/mixrs");

        if socket_path.exists() {
            fs::remove_file(socket_path)?;
        }

        let listener = UnixListener::bind(socket_path)?;

        Ok(listener)
    }

    pub fn run(&mut self, pulse_ix_rx: Receiver<PulseInstruction>) -> ! {
        let listener = self
            .create_socket_listener()
            .expect("Error creating unix socket listener");

        let (mixer_tx, mixer_rx) = channel::<MixerInstruction>();

        thread::spawn(move || {
            for client in listener.incoming() {
                match client {
                    Ok(mut stream) => {
                        let mut buf: Vec<u8> = Vec::with_capacity(1);
                        stream.read_to_end(&mut buf).expect("Error reading stream");

                        match MixerInstruction::from_u8(buf[0]) {
                            Some(ix) => mixer_tx.send(ix).unwrap(),
                            None => println!("Invalid instruction: {}", buf[0]),
                        }
                    }
                    Err(_) => println!("Stream error"),
                }
            }
        });

        let initial_sink_inputs: Arc<Mutex<HashMap<u32, SinkInputMixerData>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let callback_initial_sink_inputs = initial_sink_inputs.clone();

        let initial_sink_inputs_operation = self
            .context
            .borrow_mut()
            .introspect()
            .borrow_mut()
            .get_sink_input_info_list(move |r| {
                let ListResult::Item(sink_input) = r else {
                    return;
                };

                callback_initial_sink_inputs.lock().unwrap().insert(
                    sink_input.index,
                    SinkInputMixerData {
                        name: get_sink_input_name(&sink_input).unwrap(),
                        volume: sink_input.volume.avg().0,
                        channels: sink_input.volume.len(),
                        muted: sink_input.mute,
                    },
                );
            });

        while initial_sink_inputs_operation.get_state() == pulse::operation::State::Running {
            iterate_mainloop(&mut self.mainloop);
        }

        self.sink_inputs = initial_sink_inputs.lock().unwrap().clone();

        *self.selected_index.lock().unwrap() = match self.sink_inputs.keys().nth(0) {
            Some(_) => Some(0),
            None => None,
        };

        loop {
            match mixer_rx.try_recv() {
                Ok(ix) => match ix {
                    MixerInstruction::SelectNext => self.select_next(),
                    MixerInstruction::SelectPrevious => self.select_previous(),
                    MixerInstruction::ToggleMuteCurrent => self.toggle_mute_current(),
                    MixerInstruction::IncreaseCurrent => self.increase_volume_current(),
                    MixerInstruction::DecreaseCurrent => self.decrease_volume_current(),
                    MixerInstruction::GetCurrent => self.get_current(),
                    MixerInstruction::PlayPauseCurrent => self.play_pause_current(),
                    MixerInstruction::PlayNext => self.play_next_current(),
                    MixerInstruction::PlayPrevious => self.play_previous_current(),
                },
                Err(_) => (),
            }

            if let Some(ix) = pulse_ix_rx.try_recv().ok() {
                match ix {
                    PulseInstruction::AddSinkInput(sink_index) => {
                        let result: Arc<Mutex<Option<SinkInputMixerData>>> =
                            Arc::new(Mutex::new(None));
                        let operation_result = result.clone();

                        let operation = self
                            .context
                            .borrow_mut()
                            .introspect()
                            .borrow_mut()
                            .get_sink_input_info(sink_index, move |r| {
                                if let ListResult::Item(sink_input) = r {
                                    *operation_result.lock().unwrap() = Some(SinkInputMixerData {
                                        name: get_sink_input_name(sink_input).unwrap(),
                                        volume: sink_input.volume.avg().0,
                                        channels: sink_input.volume.len(),
                                        muted: sink_input.mute,
                                    });
                                }
                            });

                        while operation.get_state() == pulse::operation::State::Running {
                            iterate_mainloop(&mut self.mainloop);
                        }

                        let sink_input = result.lock().unwrap().take();
                        if let Some(sink_input) = sink_input {
                            self.sink_inputs.insert(sink_index, sink_input);
                        }
                    }
                    PulseInstruction::RemoveSinkInput(sink_index) => {
                        let selected_index_lock = self.selected_index.lock().unwrap();

                        match *selected_index_lock {
                            Some(current_index) => {
                                drop(selected_index_lock);

                                let removed_sink_input_index = self
                                    .sink_inputs
                                    .keys()
                                    .position(|k| *k == sink_index)
                                    .unwrap();

                                let current_key =
                                    *self.sink_inputs.keys().nth(current_index).unwrap();

                                if self.sink_inputs.remove(&sink_index).is_some() {
                                    if sink_index == current_key
                                        || removed_sink_input_index > current_index
                                    {
                                        self.select_previous();
                                    }
                                }
                            }

                            None => (),
                        }
                    }
                    PulseInstruction::UpdateSinkInput(sink_index) => {
                        match self.sink_inputs.get_mut(&sink_index) {
                            Some(sink_input_mixer_data) => {
                                let new_sink_input: Arc<Mutex<Option<SinkInputMixerData>>> =
                                    Arc::new(Mutex::new(None));
                                let callback_new_sink_input = new_sink_input.clone();

                                let operation = self
                                    .context
                                    .borrow_mut()
                                    .introspect()
                                    .borrow_mut()
                                    .get_sink_input_info(sink_index, move |r| {
                                        let ListResult::Item(sink_input) = r else {
                                            return;
                                        };

                                        *callback_new_sink_input.lock().unwrap() =
                                            Some(SinkInputMixerData {
                                                name: get_sink_input_name(&sink_input).unwrap(),
                                                volume: sink_input.volume.avg().0,
                                                channels: sink_input.volume.len(),
                                                muted: sink_input.mute,
                                            });
                                    });

                                while operation.get_state() == pulse::operation::State::Running {
                                    iterate_mainloop(&mut self.mainloop);
                                }

                                let mut sink_input_lock = new_sink_input.lock().unwrap();
                                if let Some(new_sink_input) = sink_input_lock.take() {
                                    *sink_input_mixer_data = new_sink_input;
                                }
                            }
                            None => (),
                        }
                    }
                }
            }

            iterate_mainloop(&mut self.mainloop);
        }
    }

    pub fn select_next(&mut self) {
        let mut index_lock = self.selected_index.lock().unwrap();

        match *index_lock {
            Some(current_index) => {
                let new_index: usize =
                    (current_index.overflowing_add(1).0 % self.sink_inputs.len()).max(0);

                if current_index != new_index {
                    *index_lock = Some(new_index);
                }

                drop(index_lock);
                self.get_current();
            }
            None => {
                *index_lock = if self.sink_inputs.len() > 0 {
                    Some(0)
                } else {
                    None
                };
            }
        }
    }

    pub fn select_previous(&mut self) {
        let mut index_lock = self.selected_index.lock().unwrap();

        match *index_lock {
            Some(current_index) => {
                let new_index: usize = match current_index.overflowing_sub(1) {
                    (_, true) => self.sink_inputs.len() - 1,
                    (new_value, false) => new_value,
                };

                if current_index != new_index {
                    *index_lock = Some(new_index);
                }

                drop(index_lock);
                self.get_current();
            }
            None => {
                *index_lock = if self.sink_inputs.len() > 0 {
                    Some(0)
                } else {
                    None
                };
            }
        }
    }

    pub fn toggle_mute_current(&mut self) {
        let index_lock = self.selected_index.lock().unwrap();

        let Some(index) = *index_lock else {
            return;
        };

        drop(index_lock);

        let Some(sink_index) = self.sink_inputs.keys().nth(index) else {
            return;
        };

        self.context
            .borrow_mut()
            .introspect()
            .borrow_mut()
            .set_sink_input_mute(
                *sink_index,
                !self.sink_inputs.get(&sink_index).unwrap().muted,
                None,
            );
    }

    pub fn increase_volume_current(&mut self) {
        let index_lock = self.selected_index.lock().unwrap();

        let Some(index) = *index_lock else {
            return;
        };

        drop(index_lock);

        let Some(sink_index) = self.sink_inputs.keys().nth(index) else {
            return;
        };

        let sink_input = self.sink_inputs.get(&sink_index).unwrap();
        let sink_name = sink_input.name.clone();

        let mut volume = ChannelVolumes::default();
        volume.set(
            sink_input.channels,
            pulse::volume::Volume(sink_input.volume),
        );

        volume.increase(pulse::volume::Volume(percentage_to_total_volume(5)));

        self.context
            .borrow_mut()
            .introspect()
            .borrow_mut()
            .set_sink_input_volume(*sink_index, &volume, Some(Box::new(move |success| {
                if success {
                    let _ = send_notification(&format!("{sink_name}: {}%", volume_to_percentage(volume)));
                }
            })));
    }

    pub fn decrease_volume_current(&mut self) {
        let index_lock = self.selected_index.lock().unwrap();

        let Some(index) = *index_lock else {
            return;
        };

        drop(index_lock);

        let Some(sink_index) = self.sink_inputs.keys().nth(index) else {
            return;
        };

        let sink_input = self.sink_inputs.get(&sink_index).unwrap();
        let sink_name = sink_input.name.clone();

        let mut volume = ChannelVolumes::default();
        volume.set(
            sink_input.channels,
            pulse::volume::Volume(sink_input.volume),
        );

        volume.decrease(pulse::volume::Volume(percentage_to_total_volume(5)));

        self.context
            .borrow_mut()
            .introspect()
            .borrow_mut()
            .set_sink_input_volume(*sink_index, &volume, Some(Box::new(move |success| {
                if success {
                    let _ = send_notification(&format!("{sink_name}: {}%", volume_to_percentage(volume)));
                }
            })));
    }

    pub fn get_current(&self) {
        let index_lock = self.selected_index.lock().unwrap();

        let Some(index) = *index_lock else {
            return;
        };

        drop(index_lock);

        let Some(sink_index) = self.sink_inputs.keys().nth(index) else {
            return;
        };

        let current_name = &self.sink_inputs.get(&sink_index).unwrap().name;
        let _ = send_notification(&format!("{current_name}"));
    }

    pub fn play_pause_current(&self) {
        let index_lock = self.selected_index.lock().unwrap();

        let Some(index) = *index_lock else {
            return;
        };

        drop(index_lock);

        let Some(sink_index) = self.sink_inputs.keys().nth(index) else {
            return;
        };

        let current_name = &self.sink_inputs.get(&sink_index).unwrap().name;
        match playerctl_play_pause(current_name) {
            Ok(_) => (),
            Err(_) => (),
        };
    }

    pub fn play_next_current(&self) {
        let index_lock = self.selected_index.lock().unwrap();

        let Some(index) = *index_lock else {
            return;
        };

        drop(index_lock);

        let Some(sink_index) = self.sink_inputs.keys().nth(index) else {
            return;
        };

        let current_name = &self.sink_inputs.get(&sink_index).unwrap().name;
        match playerctl_next(current_name) {
            Ok(_) => (),
            Err(_) => (),
        };
    }

    pub fn play_previous_current(&self) {
        let index_lock = self.selected_index.lock().unwrap();

        let Some(index) = *index_lock else {
            return;
        };

        drop(index_lock);

        let Some(sink_index) = self.sink_inputs.keys().nth(index) else {
            return;
        };

        let current_name = &self.sink_inputs.get(&sink_index).unwrap().name;
        match playerctl_previous(current_name) {
            Ok(_) => (),
            Err(_) => (),
        };
    }
}

pub fn iterate_mainloop(mainloop: &mut pulse::mainloop::standard::Mainloop) {
    mainloop.borrow_mut().iterate(false);
    thread::sleep(Duration::from_millis(5));
}
