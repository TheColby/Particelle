//! Fixed-capacity stereo delay and decorrelated allpass diffusion.

pub struct GlobalEffects {
    delay: Vec<[f32; 2]>,
    diffusers: [Vec<[f32; 2]>; 3],
    cursor: usize,
}

impl GlobalEffects {
    pub fn new(sample_rate: f64) -> Self {
        let length = |seconds: f64| (sample_rate * seconds).round().max(1.0) as usize;
        Self {
            delay: vec![[0.0; 2]; length(0.29)],
            diffusers: [0.031, 0.043, 0.071].map(|s| vec![[0.0; 2]; length(s)]),
            cursor: 0,
        }
    }

    pub fn reset(&mut self) {
        self.delay.fill([0.0; 2]);
        for diffuser in &mut self.diffusers {
            diffuser.fill([0.0; 2]);
        }
        self.cursor = 0;
    }

    /// Returns dry stereo plus independently decorrelated wet signal.
    pub fn process(&mut self, input: [f32; 2], delay_mix: f32, reverb_mix: f32) -> [f32; 2] {
        let index = self.cursor % self.delay.len();
        let delayed = self.delay[index];
        self.delay[index] = [input[0] + delayed[1] * 0.43, input[1] + delayed[0] * 0.43];
        let mut diffuse = [input[0] + delayed[1] * 0.22, input[1] + delayed[0] * 0.22];
        for diffuser in &mut self.diffusers {
            let index = self.cursor % diffuser.len();
            for channel in 0..2 {
                let opposite = 1 - channel;
                let previous = diffuser[index][channel];
                let value = diffuse[opposite] + previous * 0.58;
                diffuse[channel] = previous - value * 0.58;
                diffuser[index][channel] = value;
            }
        }
        self.cursor = self.cursor.wrapping_add(1);
        [
            ((input[0] + delayed[0] * delay_mix + diffuse[0] * reverb_mix) * 0.72).tanh(),
            ((input[1] + delayed[1] * delay_mix + diffuse[1] * reverb_mix) * 0.72).tanh(),
        ]
    }
}
