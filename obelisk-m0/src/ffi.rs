//! Small C ABI for embedding the fixed M0 instrument in a native host.
//!
//! Event dispatch and rendering must be externally serialized. Send MIDI at a
//! block boundary before calling `obelisk_m0_process_stereo`; this keeps the
//! realtime path allocation-free and avoids hidden cross-thread locks.

use crate::{default_twelve_tet_tuning, M0Config, M0Engine};

/// Opaque C handle. Create and destroy it only with the functions below.
pub struct ObeliskM0Handle {
    engine: M0Engine,
}

/// Create a stereo M0 engine. Returns null for invalid arguments or allocation
/// failure. `sample_rate` must be finite and positive; `max_voices` is at least one.
#[unsafe(no_mangle)]
pub extern "C" fn obelisk_m0_create(sample_rate: f64, max_voices: usize) -> *mut ObeliskM0Handle {
    let config = M0Config {
        sample_rate,
        max_voices,
        ..M0Config::default()
    };
    match M0Engine::new(config, default_twelve_tet_tuning()) {
        Ok(engine) => Box::into_raw(Box::new(ObeliskM0Handle { engine })),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Destroy a handle returned by [`obelisk_m0_create`]. Passing null is allowed.
///
/// # Safety
/// `handle` must be null or a live pointer returned by `obelisk_m0_create`, and
/// it must not be used again after this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn obelisk_m0_destroy(handle: *mut ObeliskM0Handle) {
    if !handle.is_null() {
        // SAFETY: the caller contract guarantees exclusive ownership of this allocation.
        unsafe { drop(Box::from_raw(handle)) };
    }
}

/// Dispatch one complete two- or three-byte MIDI message before rendering.
///
/// Returns false for null handles or malformed messages.
///
/// # Safety
/// `handle` must be a live handle and externally synchronized with rendering.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn obelisk_m0_send_midi3(
    handle: *mut ObeliskM0Handle,
    status: u8,
    data1: u8,
    data2: u8,
) -> bool {
    if handle.is_null() || status < 0x80 {
        return false;
    }
    // SAFETY: the caller contract guarantees a live, exclusively accessed handle.
    let handle = unsafe { &mut *handle };
    handle.engine.handle_midi_bytes(&[status, data1, data2]);
    true
}

/// Render `frames` stereo frames to an interleaved `f32` output buffer.
///
/// Returns false for null pointers. The engine writes exactly `frames * 2`
/// samples and performs no heap allocation on this path.
///
/// # Safety
/// `handle` must be live and synchronized with MIDI dispatch. `output` must
/// point to writable storage for at least `frames * 2` contiguous `f32` values.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn obelisk_m0_process_stereo(
    handle: *mut ObeliskM0Handle,
    output: *mut f32,
    frames: usize,
) -> bool {
    if handle.is_null() || (output.is_null() && frames != 0) {
        return false;
    }
    if frames == 0 {
        return true;
    }
    // SAFETY: the caller guarantees a valid handle and an interleaved stereo buffer.
    let handle = unsafe { &mut *handle };
    // SAFETY: `[f32; 2]` is contiguous two-element `f32` storage and the caller
    // guarantees exactly this many samples are writable.
    let output = unsafe { std::slice::from_raw_parts_mut(output.cast::<[f32; 2]>(), frames) };
    handle.engine.process_block(&[], output);
    true
}

/// Submit a complete 128-note host tuning table at a block boundary.
///
/// This function is the ABI bridge for an MTS-ESP-aware VST3/AU wrapper. The
/// core does not discover or link to MTS-ESP itself, keeping the engine usable
/// in standalone and sandboxed hosts.
///
/// # Safety
/// `handle` must be live and externally synchronized. `frequencies_hz` must
/// point to 128 readable `double` values.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn obelisk_m0_set_tuning_table(
    handle: *mut ObeliskM0Handle,
    frequencies_hz: *const f64,
) -> bool {
    if handle.is_null() || frequencies_hz.is_null() {
        return false;
    }
    // SAFETY: the caller contract guarantees 128 readable contiguous values.
    let values = unsafe { std::slice::from_raw_parts(frequencies_hz, 128) };
    let mut table = [0.0; 128];
    table.copy_from_slice(values);
    // SAFETY: the caller contract guarantees exclusive access to the handle.
    unsafe { &mut *handle }
        .engine
        .set_external_tuning_table(table)
        .is_ok()
}

/// Clear a tuning table previously supplied with `obelisk_m0_set_tuning_table`.
///
/// # Safety
/// `handle` must be live and externally synchronized with rendering.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn obelisk_m0_clear_tuning_table(handle: *mut ObeliskM0Handle) {
    if !handle.is_null() {
        // SAFETY: the caller contract guarantees exclusive access to the handle.
        unsafe { &mut *handle }.engine.clear_external_tuning();
    }
}

/// Release every active note without deallocating the engine.
///
/// # Safety
/// `handle` must be live and externally synchronized with rendering.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn obelisk_m0_all_notes_off(handle: *mut ObeliskM0Handle) {
    if !handle.is_null() {
        // SAFETY: the caller contract guarantees a live, exclusively accessed handle.
        unsafe { &mut *handle }.engine.all_notes_off();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c_abi_renders_midi_note_without_silence() {
        let handle = obelisk_m0_create(48_000.0, 16);
        assert!(!handle.is_null());
        // SAFETY: the test owns the created handle and output allocation.
        unsafe {
            assert!(obelisk_m0_send_midi3(handle, 0x91, 60, 100));
            let mut output = vec![0.0_f32; 256 * 2];
            assert!(obelisk_m0_process_stereo(handle, output.as_mut_ptr(), 256));
            assert!(output.iter().any(|sample| *sample != 0.0));
            obelisk_m0_destroy(handle);
        }
    }

    #[test]
    fn c_abi_accepts_complete_external_tuning_table() {
        let handle = obelisk_m0_create(48_000.0, 16);
        let table: [f64; 128] =
            core::array::from_fn(|note| 440.0 * 2.0_f64.powf((note as f64 - 69.0) / 12.0));
        // SAFETY: the test owns the handle and table has exactly 128 values.
        unsafe {
            assert!(obelisk_m0_set_tuning_table(handle, table.as_ptr()));
            obelisk_m0_clear_tuning_table(handle);
            obelisk_m0_destroy(handle);
        }
    }
}
