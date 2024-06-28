#[repr(u8)]
pub enum MixerInstruction {
    SelectNext,
    SelectPrevious,
    ToggleMuteCurrent,
    IncreaseCurrent,
    DecreaseCurrent,
    GetCurrent,
    PlayPauseCurrent,
    PlayNext,
    PlayPrevious,
}

impl MixerInstruction {
    pub fn from_u8(byte: u8) -> Option<Self> {
        match byte {
            0 => Some(MixerInstruction::SelectNext),
            1 => Some(MixerInstruction::SelectPrevious),
            2 => Some(MixerInstruction::ToggleMuteCurrent),
            3 => Some(MixerInstruction::IncreaseCurrent),
            4 => Some(MixerInstruction::DecreaseCurrent),
            5 => Some(MixerInstruction::GetCurrent),
            6 => Some(MixerInstruction::PlayPauseCurrent),
            7 => Some(MixerInstruction::PlayNext),
            8 => Some(MixerInstruction::PlayPrevious),
            _ => None,
        }
    }
}
