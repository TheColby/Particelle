//! Single-threaded offline M0 browser ABI. No device or JavaScript DSP backend.
use obelisk_m0::{default_twelve_tet_tuning, render_interleaved_stereo, M0Config, M0Engine};
use std::cell::RefCell;

thread_local! {
    static OUTPUT: RefCell<Vec<f32>> = const { RefCell::new(Vec::new()) };
}

#[no_mangle]
pub extern "C" fn obelisk_abi_version() -> u32 {
    1
}

/// Renders a four-note phrase followed by a two-second release/effects tail.
/// Returns 0 on success, 1 for invalid arguments, 2 for invalid M0 controls.
/// Output is interleaved stereo f32, valid until the next render/clear call.
#[no_mangle]
pub extern "C" fn obelisk_render(
    sample_rate: u32,
    root: u32,
    seconds: f32,
    morph: f32,
    modulation_hz: f32,
    delay_mix: f32,
    reverb_mix: f32,
) -> u32 {
    OUTPUT.with(|output| output.borrow_mut().clear());
    if ![44100, 48000].contains(&sample_rate)
        || root > 115
        || !seconds.is_finite()
        || !(0.25..=8.0).contains(&seconds)
    {
        return 1;
    }
    let config = M0Config {
        sample_rate: sample_rate as f64,
        base_morph: morph,
        modulation_hz,
        delay_mix,
        reverb_mix,
        ..M0Config::default()
    };
    let mut engine = match M0Engine::new(config, default_twelve_tet_tuning()) {
        Ok(engine) => engine,
        Err(_) => return 2,
    };
    let phrase_frames = (seconds * sample_rate as f32).round() as usize;
    let total_frames = phrase_frames + 2 * sample_rate as usize;
    let mut result = Vec::with_capacity(total_frames * 2);
    let intervals = [0, 7, 12, 4];
    for (index, interval) in intervals.into_iter().enumerate() {
        let start = phrase_frames * index / 4;
        let end = phrase_frames * (index + 1) / 4;
        let held = (end - start) * 3 / 4;
        let note = root as u8 + interval;
        // MIDI channel 2 is the first member of M0's default lower MPE zone.
        engine.handle_midi_bytes(&[0x91, note, 100]);
        append_frames(&mut engine, &mut result, held);
        engine.handle_midi_bytes(&[0x81, note, 0]);
        append_frames(&mut engine, &mut result, end - start - held);
    }
    append_frames(&mut engine, &mut result, 2 * sample_rate as usize);
    OUTPUT.with(|output| *output.borrow_mut() = result);
    0
}

fn append_frames(engine: &mut M0Engine, result: &mut Vec<f32>, mut frames: usize) {
    while frames > 0 {
        let block = frames.min(128);
        result.extend(render_interleaved_stereo(engine, &[], block));
        frames -= block;
    }
}

#[no_mangle]
pub extern "C" fn obelisk_output_ptr() -> *const f32 {
    OUTPUT.with(|output| output.borrow().as_ptr())
}

#[no_mangle]
pub extern "C" fn obelisk_output_len() -> usize {
    OUTPUT.with(|output| output.borrow().len())
}

#[no_mangle]
pub extern "C" fn obelisk_clear() {
    OUTPUT.with(|output| *output.borrow_mut() = Vec::new());
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn m0_render_is_bounded_finite_audible_and_deterministic() {
        assert_eq!(obelisk_render(48000, 60, 0.25, 0.7, 0.2, 0.1, 0.2), 0);
        let first = OUTPUT.with(|out| out.borrow().clone());
        assert_eq!(first.len(), 108000 * 2);
        assert!(first.iter().all(|v| v.is_finite()));
        assert!(first.iter().any(|v| v.abs() > 0.001));
        assert_eq!(obelisk_render(48000, 60, 0.25, 0.7, 0.2, 0.1, 0.2), 0);
        OUTPUT.with(|out| assert_eq!(first, *out.borrow()));
    }
    #[test]
    fn invalid_input_clears_stale_output() {
        for (rate, note, seconds) in [
            (0, 60, 1.0),
            (48000, 116, 1.0),
            (48000, 60, f32::NAN),
            (48000, 60, 9.0),
        ] {
            assert_eq!(obelisk_render(rate, note, seconds, 0.5, 0.0, 0.0, 0.0), 1);
            assert_eq!(obelisk_output_len(), 0);
        }
        assert_eq!(obelisk_render(48000, 60, 1.0, f32::NAN, 0.0, 0.0, 0.0), 2);
    }
}
