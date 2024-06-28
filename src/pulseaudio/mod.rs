use std::u32;

use crate::utils::total_volume_to_percentage;

pub enum PulseInstruction {
    AddSinkInput(u32),
    RemoveSinkInput(u32),
    UpdateSinkInput(u32),
}

pub enum PulseResponse {
    Ok,
    Error,
    SinkInput(Option<SinkInputMixerData>),
    SinkInputs(Vec<SinkInputMixerData>),
}

#[derive(Clone, Debug)]
pub struct SinkInputMixerData {
    /// The input sink's `application.name`
    pub name: String,
    /// The input sink's volume
    pub volume: u32,
    pub muted: bool,
    pub channels: u8,
}

impl SinkInputMixerData {
    pub fn get_volume_percent(&self) -> u8 {
        total_volume_to_percentage(self.volume)
    }
}
