use anyhow::anyhow;
use pulse::{context::introspect::SinkInputInfo, volume};

const FULL_VOLUME: u32 = 1 << 16;

pub fn volume_to_percentage(volume: volume::ChannelVolumes) -> u8 {
    let average = volume.avg().0;

    total_volume_to_percentage(average)
}

pub fn total_volume_to_percentage(volume: u32) -> u8 {
    ((volume as f32 / FULL_VOLUME as f32) * 100.0).round() as u8
}

pub fn percentage_to_total_volume(percentage: u8) -> u32 {
    ((FULL_VOLUME as f32 / 100.0) * percentage as f32).round() as u32
}

pub fn get_sink_input_name(sink_input: &SinkInputInfo) -> anyhow::Result<String> {
    let Some(name_bytes) = sink_input.proplist.get("application.name") else {
        return Err(anyhow!("Invalid sink input name"));
    };

    Ok(String::from_utf8(
        name_bytes[..name_bytes.len() - 1].to_vec(),
    )?)
}
