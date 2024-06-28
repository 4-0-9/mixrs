#[repr(u8)]
pub enum MixerInstruction {
    SelectNext,
    SelectPrevious,
    ToggleMuteCurrent,
    IncreaseCurrent,
    DecreaseCurrent,
    GetCurrent,
    PlayPauseCurrent,
}

impl MixerInstruction {
    pub fn from_u8(byte: u8) -> Self {
        match byte {
            0 => MixerInstruction::SelectNext,
            1 => MixerInstruction::SelectPrevious,
            2 => MixerInstruction::ToggleMuteCurrent,
            3 => MixerInstruction::IncreaseCurrent,
            4 => MixerInstruction::DecreaseCurrent,
            5 => MixerInstruction::GetCurrent,
            6 => MixerInstruction::PlayPauseCurrent,
            _ => panic!("Could not parse '{byte}' to MixerInstruction"),
        }
    }
}
