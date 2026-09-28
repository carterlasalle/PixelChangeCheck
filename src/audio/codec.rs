use anyhow::{anyhow, Context, Result};

/// Opus operates internally at this rate, so using it avoids resampling at the
/// codec boundary and is portable across every Opus implementation.
pub const SAMPLE_RATE: u32 = 48_000;
/// The wire audio is always interleaved stereo.
pub const CHANNELS: usize = 2;
/// Twenty milliseconds at 48 kHz. This is the conventional Opus real-time
/// compromise: enough audio for efficient packets, without adding the 40–60 ms
/// packetization delay that starts to feel laggy in interactive sharing.
pub const SAMPLES_PER_CHANNEL: usize = 960;
/// Total interleaved `f32` values in one 20 ms stereo frame.
pub const SAMPLES_PER_FRAME: usize = SAMPLES_PER_CHANNEL * CHANNELS;
const MAX_PACKET_BYTES: usize = 1_275;

fn require_frame_len(samples: usize) -> Result<()> {
    if samples != SAMPLES_PER_FRAME {
        return Err(anyhow!(
            "Opus frame must be {SAMPLES_PER_FRAME} interleaved samples ({SAMPLES_PER_CHANNEL} per channel), got {samples}"
        ));
    }
    Ok(())
}

/// A 48 kHz stereo Opus encoder with fixed 20 ms frames.
pub struct OpusEncoder {
    inner: opus::Encoder,
}

impl OpusEncoder {
    pub fn new() -> Result<Self> {
        let inner = opus::Encoder::new(
            SAMPLE_RATE,
            opus::Channels::Stereo,
            opus::Application::Audio,
        )
        .context("creating 48 kHz stereo Opus encoder")?;
        Ok(Self { inner })
    }

    /// Encodes exactly one interleaved 20 ms stereo frame.
    pub fn encode(&mut self, pcm: &[f32]) -> Result<Vec<u8>> {
        require_frame_len(pcm.len())?;
        let mut packet = vec![0; MAX_PACKET_BYTES];
        let bytes = self
            .inner
            .encode_float(pcm, &mut packet)
            .context("encoding Opus frame")?;
        packet.truncate(bytes);
        Ok(packet)
    }
}

/// A 48 kHz stereo Opus decoder with fixed 20 ms output buffers.
///
/// Opus has roughly 6.5 ms of algorithmic delay. A decoder must therefore not
/// expect immediately audible output from its first packet; the playout buffer
/// owns that startup delay rather than pretending the first packet is late.
pub struct OpusDecoder {
    inner: opus::Decoder,
}

impl OpusDecoder {
    pub fn new() -> Result<Self> {
        let inner = opus::Decoder::new(SAMPLE_RATE, opus::Channels::Stereo)
            .context("creating 48 kHz stereo Opus decoder")?;
        Ok(Self { inner })
    }

    /// Decodes one packet into exactly one interleaved 20 ms stereo frame.
    /// Returns the number of samples decoded per channel, matching libopus.
    pub fn decode(&mut self, packet: &[u8], pcm: &mut [f32]) -> Result<usize> {
        require_frame_len(pcm.len())?;
        self.inner
            .decode_float(packet, pcm, false)
            .context("decoding Opus packet")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encoder_rejects_a_non_20_ms_stereo_frame() {
        let mut encoder = OpusEncoder::new().expect("creates encoder");
        let error = encoder
            .encode(&vec![0.0; SAMPLES_PER_FRAME / 2])
            .expect_err("rejects short frame");
        let message = error.to_string();
        assert!(message.contains(&SAMPLES_PER_FRAME.to_string()));
        assert!(message.contains(&(SAMPLES_PER_FRAME / 2).to_string()));
    }
}
