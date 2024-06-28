mod instructions;
pub mod mixer;
pub mod pulseaudio;
pub mod utils;
pub mod playerctl;

use mixer::Mixer;
use pulseaudio::PulseInstruction;
use std::sync::mpsc::channel;

fn main() {
    let mainloop = pulse::mainloop::standard::Mainloop::new().expect("Error getting PulseAudio main loop");

    let (pulse_ix_tx, pulse_ix_rx) = channel::<PulseInstruction>();

    let mut mixer = Mixer::new(mainloop, pulse_ix_tx);

    mixer.run(pulse_ix_rx);
}
