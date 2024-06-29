mod instructions;
pub mod mixer;
pub mod playerctl;
pub mod pulseaudio;
pub mod utils;

use mixer::Mixer;
use pulseaudio::PulseInstruction;
use std::{env, sync::mpsc::channel};

fn main() {
    let mainloop =
        pulse::mainloop::standard::Mainloop::new().expect("Error getting PulseAudio main loop");

    let args: Vec<String> = env::args().collect();
    let silent_mode = match args.iter().nth(1) {
        Some(arg) => arg == "--silent",
        None => false,
    };

    let (pulse_ix_tx, pulse_ix_rx) = channel::<PulseInstruction>();

    let mut mixer = Mixer::new(mainloop, pulse_ix_tx, silent_mode);

    mixer.run(pulse_ix_rx);
}
