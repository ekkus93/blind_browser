#[cfg(any(feature = "remote-openai", test))]
pub(crate) fn encode_wav_pcm16(
    samples: &[f32],
    sample_rate: u32,
    channels: u16,
) -> Result<Vec<u8>, String> {
    if sample_rate == 0 {
        return Err(String::from("sample rate must be greater than zero"));
    }
    if channels == 0 {
        return Err(String::from("channel count must be greater than zero"));
    }

    let bytes_per_sample = 2u16;
    let block_align = channels
        .checked_mul(bytes_per_sample)
        .ok_or_else(|| String::from("wav block alignment overflowed"))?;
    let byte_rate = sample_rate
        .checked_mul(u32::from(block_align))
        .ok_or_else(|| String::from("wav byte rate overflowed"))?;

    let data_size = samples
        .len()
        .checked_mul(2)
        .and_then(|size| u32::try_from(size).ok())
        .ok_or_else(|| String::from("wav data chunk was too large"))?;
    let riff_size = 36u32
        .checked_add(data_size)
        .ok_or_else(|| String::from("wav riff chunk overflowed"))?;

    let mut bytes = Vec::with_capacity(44usize.saturating_add(data_size as usize));
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&riff_size.to_le_bytes());
    bytes.extend_from_slice(b"WAVE");
    bytes.extend_from_slice(b"fmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&channels.to_le_bytes());
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    bytes.extend_from_slice(&byte_rate.to_le_bytes());
    bytes.extend_from_slice(&block_align.to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_size.to_le_bytes());

    for sample in samples {
        let scaled = (sample.clamp(-1.0, 1.0) * i16::MAX as f32).round();
        bytes.extend_from_slice(&(scaled as i16).to_le_bytes());
    }

    Ok(bytes)
}
