//! Non-destructive clip trimming, fades and peak normalization. Exports PCM16 WAV.
use crate::{decode, Source};
use kvas_engine::assets::Assets;
use std::sync::Arc;

/// Processing operations on a selected time range.
#[derive(Clone, Copy, Debug)]
pub struct ClipEdit {
    /// Inclusive start in seconds.
    pub start: f32,
    /// Exclusive end in seconds; zero means the end of the clip.
    pub end: f32,
    /// Fade from silence over this many seconds.
    pub fade_in: f32,
    /// Fade to silence over this many seconds.
    pub fade_out: f32,
    /// Normalize the selected samples to a peak of 0.98 before fades.
    pub normalize: bool,
    /// Additional linear gain after normalization.
    pub gain: f32,
}

impl Default for ClipEdit {
    fn default() -> Self { Self { start: 0.0, end: 0.0, fade_in: 0.0, fade_out: 0.0, normalize: false, gain: 1.0 } }
}

/// Decodes and edits a project sound. Run on a worker for large files.
/// A 32-million-sample limit bounds memory use; oversized clips report an error.
pub fn edit_asset(assets: &Assets, path: &str, edit: ClipEdit) -> Result<Vec<u8>, String> {
    let bytes = Arc::<[u8]>::from(assets.read(path).map_err(|e| e.to_string())?);
    let decoder = decode(bytes)?;
    let channels = decoder.channels(); let rate = decoder.sample_rate();
    let samples: Vec<i16> = decoder.take(32_000_001).collect();
    if samples.len() > 32_000_000 { return Err("clip exceeds the 32-million-sample editing limit".into()); }
    edit_pcm(&samples, channels, rate, edit)
}

/// Processes interleaved samples without changing the source, returning a complete WAV file.
pub fn edit_pcm(samples: &[i16], channels: u16, rate: u32, edit: ClipEdit) -> Result<Vec<u8>, String> {
    if channels == 0 || channels > 32 || rate == 0 || rate > 384_000 { return Err("invalid channel count or sample rate".into()); }
    if [edit.start, edit.end, edit.fade_in, edit.fade_out, edit.gain].iter().any(|v| !v.is_finite() || *v < 0.0) { return Err("edit values must be finite and non-negative".into()); }
    if samples.len() % channels as usize != 0 { return Err("incomplete interleaved frame".into()); }
    let frames = samples.len() / channels as usize;
    let start = (edit.start as f64 * rate as f64).round().min(frames as f64) as usize;
    let end = if edit.end == 0.0 { frames } else { (edit.end as f64 * rate as f64).round().min(frames as f64) as usize };
    if start >= end { return Err("select a non-empty time range".into()); }
    let range = &samples[start * channels as usize..end * channels as usize];
    let peak = range.iter().map(|s| (*s as f32).abs()).fold(0.0, f32::max);
    let gain = edit.gain * if edit.normalize && peak > 0.0 { 32767.0 * 0.98 / peak } else { 1.0 };
    if !gain.is_finite() { return Err("gain is too large".into()); }
    let data_size = u32::try_from(range.len().checked_mul(2).ok_or("WAV size overflow")?).map_err(|_| "WAV size overflow")?;
    let riff_size = data_size.checked_add(36).ok_or("WAV size overflow")?;
    let mut out = Vec::with_capacity(data_size as usize + 44);
    out.extend(b"RIFF"); out.extend(riff_size.to_le_bytes()); out.extend(b"WAVEfmt ");
    out.extend(16u32.to_le_bytes()); out.extend(1u16.to_le_bytes()); out.extend(channels.to_le_bytes());
    out.extend(rate.to_le_bytes()); out.extend((rate * channels as u32 * 2).to_le_bytes());
    out.extend((channels * 2).to_le_bytes()); out.extend(16u16.to_le_bytes());
    out.extend(b"data"); out.extend(data_size.to_le_bytes());
    let fade_in = (edit.fade_in * rate as f32).min((end - start) as f32);
    let fade_out = (edit.fade_out * rate as f32).min((end - start) as f32);
    for (i, frame) in range.chunks_exact(channels as usize).enumerate() {
        let a = if fade_in > 0.0 { (i as f32 / fade_in).min(1.0) } else { 1.0 };
        let b = if fade_out > 0.0 { ((end - start - 1 - i) as f32 / fade_out).min(1.0) } else { 1.0 };
        for sample in frame { out.extend(((*sample as f32 * gain * a * b).round().clamp(-32768.0, 32767.0) as i16).to_le_bytes()); }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn trim_is_frame_aligned_and_wav_decodes_with_matching_metadata() {
        let source: Vec<i16> = (0..200).collect();
        let wav = edit_pcm(&source, 2, 100, ClipEdit { start: 0.2, end: 0.8, ..Default::default() }).unwrap();
        let decoder = decode(Arc::from(wav)).unwrap();
        assert_eq!(decoder.channels(), 2); assert_eq!(decoder.sample_rate(), 100);
        let result: Vec<_> = decoder.collect(); assert_eq!(result, source[40..160]);
    }
    #[test]
    fn fades_and_normalization_preserve_stereo_balance_and_source() {
        let samples = vec![1000, -500].repeat(100);
        let wav = edit_pcm(&samples, 2, 100, ClipEdit { fade_in: 0.1, fade_out: 0.1, normalize: true, ..Default::default() }).unwrap();
        let result: Vec<_> = decode(Arc::from(wav)).unwrap().collect();
        assert_eq!(&result[..2], &[0, 0]); assert_eq!(&result[198..], &[0, 0]);
        assert!(result[100] > 32000); assert!((result[100] as i32 + 2 * result[101] as i32).abs() <= 1);
        assert_eq!(samples[0], 1000);
        assert!(edit_pcm(&samples, 2, 100, ClipEdit { start: f32::NAN, ..Default::default() }).is_err());
    }
}
