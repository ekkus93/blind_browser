#[cfg(any(feature = "remote-openai", test))]
pub(crate) struct DecodedWav {
    pub(crate) sample_rate: u32,
    pub(crate) channels: u16,
    pub(crate) samples: Vec<f32>,
}

#[cfg(any(feature = "remote-openai", test))]
pub(crate) fn decode_wav_samples(bytes: &[u8]) -> Result<DecodedWav, String> {
    if bytes.len() < 12 {
        return Err(String::from("response was too short to be a WAV file"));
    }
    if &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(String::from("response was not a RIFF/WAVE file"));
    }

    let mut cursor = 12usize;
    let mut fmt_chunk = None;
    let mut data_chunk = None;
    while cursor + 8 <= bytes.len() {
        let chunk_id = &bytes[cursor..cursor + 4];
        let chunk_size = u32::from_le_bytes(
            bytes[cursor + 4..cursor + 8]
                .try_into()
                .map_err(|_| String::from("WAV chunk size bytes were truncated"))?,
        ) as usize;
        let chunk_start = cursor + 8;
        let chunk_end = chunk_start.saturating_add(chunk_size);
        if chunk_end > bytes.len() {
            return Err(String::from("WAV chunk length exceeded the response size"));
        }

        match chunk_id {
            b"fmt " => fmt_chunk = Some(&bytes[chunk_start..chunk_end]),
            b"data" => data_chunk = Some(&bytes[chunk_start..chunk_end]),
            _ => {}
        }

        cursor = chunk_end;
        if chunk_size % 2 == 1 {
            cursor = cursor.saturating_add(1);
        }
    }

    let fmt_chunk =
        fmt_chunk.ok_or_else(|| String::from("WAV response did not include a fmt chunk"))?;
    let data_chunk =
        data_chunk.ok_or_else(|| String::from("WAV response did not include a data chunk"))?;
    if fmt_chunk.len() < 16 {
        return Err(String::from("WAV fmt chunk was too short"));
    }

    let format_tag = u16::from_le_bytes(
        fmt_chunk[0..2]
            .try_into()
            .map_err(|_| String::from("WAV fmt chunk format tag was truncated"))?,
    );
    let channels = u16::from_le_bytes(
        fmt_chunk[2..4]
            .try_into()
            .map_err(|_| String::from("WAV fmt chunk channel count was truncated"))?,
    );
    let sample_rate = u32::from_le_bytes(
        fmt_chunk[4..8]
            .try_into()
            .map_err(|_| String::from("WAV fmt chunk sample rate was truncated"))?,
    );
    let bits_per_sample = u16::from_le_bytes(
        fmt_chunk[14..16]
            .try_into()
            .map_err(|_| String::from("WAV fmt chunk bit depth was truncated"))?,
    );

    if channels == 0 {
        return Err(String::from("WAV response reported zero channels"));
    }
    if sample_rate == 0 {
        return Err(String::from("WAV response reported zero sample rate"));
    }

    let samples = match (format_tag, bits_per_sample) {
        (1, 8) => data_chunk
            .iter()
            .map(|sample| (*sample as f32 - 128.0) / 128.0)
            .collect(),
        (1, 16) => data_chunk
            .chunks_exact(2)
            .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]) as f32 / i16::MAX as f32)
            .collect(),
        (1, 24) => data_chunk
            .chunks_exact(3)
            .map(|chunk| {
                let signed = i32::from_le_bytes([
                    chunk[0],
                    chunk[1],
                    chunk[2],
                    if chunk[2] & 0x80 != 0 { 0xFF } else { 0x00 },
                ]);
                signed as f32 / 8_388_607.0
            })
            .collect(),
        (1, 32) => data_chunk
            .chunks_exact(4)
            .map(|chunk| {
                i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]) as f32
                    / i32::MAX as f32
            })
            .collect(),
        (3, 32) => data_chunk
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect(),
        _ => {
            return Err(format!(
                "WAV response used unsupported encoding format_tag={format_tag}, bits_per_sample={bits_per_sample}"
            ))
        }
    };

    Ok(DecodedWav {
        sample_rate,
        channels,
        samples,
    })
}
