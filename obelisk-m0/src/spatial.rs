//! Allocation-free circular speaker routing for planar buses.

pub fn route(sample: f32, position: f32, output: &mut [&mut [f32]], frame: usize) {
    let count = output.len();
    let speaker = position.rem_euclid(1.0) * count as f32;
    let left = speaker.floor() as usize % count;
    let right = (left + 1) % count;
    let angle = (speaker - speaker.floor()) * core::f32::consts::FRAC_PI_2;
    output[left][frame] += sample * angle.cos();
    output[right][frame] += sample * angle.sin();
}

pub fn voice_position(channel: u8, first: u8, last: u8, timbre: f64) -> f32 {
    let count = usize::from(last.saturating_sub(first)) + 1;
    let member = usize::from(channel.saturating_sub(first)) % count;
    ((member as f32 + (timbre as f32 - 0.5) * 0.8) / count as f32).rem_euclid(1.0)
}
