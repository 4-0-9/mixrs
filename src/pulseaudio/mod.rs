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

    /// Formats the sink input data to a string separating fields by new lines
    pub fn get_output_data(
        &self,
        selection_index: usize,
        sink_count: usize,
        sink_index: u32,
    ) -> String {
        format!(
            "selection: {}/{sink_count}\nid: {sink_index}\nname: {}\nvolume: {}\nvolume_percentage: {}\nmuted: {}\n",
            selection_index + 1, self.name, self.volume, self.get_volume_percent(), self.muted
        )
    }
}
