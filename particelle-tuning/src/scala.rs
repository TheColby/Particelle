use thiserror::Error;

/// A Scala .scl scale definition: a list of intervals above the root.
#[derive(Debug, Clone)]
pub struct ScalaScale {
    pub description: String,
    /// Intervals in cents (relative to the root at 0 cents).
    /// The period (usually 1200 cents = octave) is the last entry.
    pub cents: Vec<f64>,
}

impl ScalaScale {
    pub fn steps(&self) -> usize {
        self.cents.len()
    }

    /// Parse a Scala .scl file from a string.
    pub fn parse(input: &str) -> Result<Self, ScalaError> {
        let mut lines = input.lines().filter(|l| !l.trim_start().starts_with('!'));
        let description = lines.next().unwrap_or("").trim().to_owned();
        let count_str = lines.next().ok_or(ScalaError::MissingCount)?.trim();
        let count: usize = count_str.parse().map_err(|_| ScalaError::InvalidCount(count_str.to_owned()))?;

        let mut cents = Vec::with_capacity(count);
        for line in lines.take(count) {
            let token = line.trim().split_whitespace().next().unwrap_or("");
            let ratio_cents = if token.contains('.') {
                token.parse::<f64>().map_err(|_| ScalaError::InvalidInterval(token.to_owned()))?
            } else if token.contains('/') {
                let mut parts = token.split('/');
                let num: f64 = parts.next().unwrap_or("1").parse()
                    .map_err(|_| ScalaError::InvalidInterval(token.to_owned()))?;
                let den: f64 = parts.next().unwrap_or("1").parse()
                    .map_err(|_| ScalaError::InvalidInterval(token.to_owned()))?;
                1200.0 * (num / den).log2()
            } else {
                let n: f64 = token.parse()
                    .map_err(|_| ScalaError::InvalidInterval(token.to_owned()))?;
                1200.0 * (n / 1.0).log2()
            };
            cents.push(ratio_cents);
        }

        if cents.len() != count {
            return Err(ScalaError::IntervalCountMismatch { expected: count, got: cents.len() });
        }

        Ok(Self { description, cents })
    }
}

/// A Scala .kbm keyboard mapping: maps MIDI note numbers to scale degrees.
#[derive(Debug, Clone)]
pub struct KbmMapping {
    pub map_size: usize,
    pub first_midi_note: u8,
    pub last_midi_note: u8,
    pub middle_note: u8,
    pub reference_note: u8,
    pub reference_freq: f64,
    pub octave_degree: usize,
    /// Mapping entries: scale degree (or -1 for unmapped).
    pub mapping: Vec<i32>,
}

impl KbmMapping {
    /// Default linear mapping: MIDI 60 = middle C = degree 0, reference A4=440.
    pub fn linear_a440() -> Self {
        Self {
            map_size: 0,
            first_midi_note: 0,
            last_midi_note: 127,
            middle_note: 60,
            reference_note: 69,
            reference_freq: 440.0,
            octave_degree: 12,
            mapping: vec![],
        }
    }

    /// Parse a Scala .kbm file from a string.
    pub fn parse(input: &str) -> Result<Self, ScalaError> {
        let mut lines = input.lines().filter(|l| !l.trim_start().starts_with('!'));

        macro_rules! next_val {
            ($t:ty) => {{
                let s = lines.next().ok_or(ScalaError::MissingCount)?.trim().to_owned();
                s.parse::<$t>().map_err(|_| ScalaError::InvalidCount(s))?
            }};
        }

        let map_size: usize     = next_val!(usize);
        let first_midi_note: u8 = next_val!(u8);
        let last_midi_note: u8  = next_val!(u8);
        let middle_note: u8     = next_val!(u8);
        let reference_note: u8  = next_val!(u8);
        let reference_freq: f64 = next_val!(f64);
        let octave_degree: usize = next_val!(usize);

        let mut mapping = Vec::with_capacity(map_size);
        for line in lines.take(map_size) {
            let token = line.trim();
            if token == "x" {
                mapping.push(-1);
            } else {
                let deg: i32 = token.parse().map_err(|_| ScalaError::InvalidInterval(token.to_owned()))?;
                mapping.push(deg);
            }
        }

        Ok(Self { map_size, first_midi_note, last_midi_note, middle_note, reference_note, reference_freq, octave_degree, mapping })
    }
}

#[derive(Debug, Error)]
pub enum ScalaError {
    #[error("Missing note count in .scl file")]
    MissingCount,
    #[error("Invalid note count: '{0}'")]
    InvalidCount(String),
    #[error("Invalid interval: '{0}'")]
    InvalidInterval(String),
    #[error("Interval count mismatch: expected {expected}, got {got}")]
    IntervalCountMismatch { expected: usize, got: usize },
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCL_12TET: &str = "! 12tet.scl\n12-tone equal temperament\n 12\n!\n 100.0\n 200.0\n 300.0\n 400.0\n 500.0\n 600.0\n 700.0\n 800.0\n 900.0\n 1000.0\n 1100.0\n 1200.0\n";

    #[test]
    fn parse_12tet_scl() {
        let scale = ScalaScale::parse(SCL_12TET).unwrap();
        assert_eq!(scale.steps(), 12);
        assert!((scale.cents[0] - 100.0).abs() < 1e-10);
        assert!((scale.cents[11] - 1200.0).abs() < 1e-10);
    }
}
