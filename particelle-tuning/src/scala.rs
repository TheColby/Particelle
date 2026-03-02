use crate::Tuning;

/// A single scale interval parsed from a .scl file.
/// Can be either cents or a ratio.
#[derive(Debug, Clone)]
pub enum SclInterval {
    /// Interval in cents (100 cents = 1 semitone in 12-TET).
    Cents(f64),
    /// Interval as a rational ratio (e.g. 3/2).
    Ratio { num: u64, den: u64 },
}

impl SclInterval {
    /// Convert the interval to a frequency ratio relative to unison.
    pub fn to_ratio(&self) -> f64 {
        match self {
            Self::Cents(c) => 2.0_f64.powf(*c / 1200.0),
            Self::Ratio { num, den } => *num as f64 / *den as f64,
        }
    }
}

/// Parsed Scala .scl file.
#[derive(Debug, Clone)]
pub struct SclScale {
    pub description: String,
    /// Intervals, NOT including the implicit 1/1 unison at degree 0.
    /// The last interval is the interval of equivalence (typically the octave).
    pub intervals: Vec<SclInterval>,
}

/// Parsed Scala .kbm keyboard mapping file.
#[derive(Debug, Clone)]
pub struct KbmMapping {
    /// Size of the keyboard mapping (number of entries in the map).
    pub map_size: usize,
    /// First MIDI note number to retune.
    pub first_note: u8,
    /// Last MIDI note number to retune.
    pub last_note: u8,
    /// MIDI note number for which the reference frequency is given.
    pub reference_note: u8,
    /// Reference frequency tied to `reference_note`.
    pub reference_frequency: f64,
    /// Scale degree corresponding to `reference_note`.
    pub reference_degree: i32,
    /// Formal octave interval index in the .scl intervals list (1-based).
    pub formal_octave: usize,
    /// Key map: for each keyboard position, which scale degree it maps to.
    /// `None` means the key is unmapped (silent).
    pub key_map: Vec<Option<u32>>,
}

/// Parse a .scl string into an SclScale.
pub fn parse_scl(input: &str) -> Result<SclScale, String> {
    let mut lines = input.lines()
        .map(|l| l.trim())
        .filter(|l| !l.starts_with('!') && !l.is_empty());

    let description = lines.next()
        .ok_or("Missing description line")?
        .to_string();

    let count_str = lines.next()
        .ok_or("Missing note count line")?;
    let count: usize = count_str.parse()
        .map_err(|_| format!("Invalid note count: '{}'", count_str))?;

    let mut intervals = Vec::with_capacity(count);
    for _ in 0..count {
        let line = lines.next()
            .ok_or("Not enough interval lines")?;
        let token = line.split_whitespace().next()
            .ok_or("Empty interval line")?;

        if token.contains('.') {
            // Cents format
            let cents: f64 = token.parse()
                .map_err(|_| format!("Invalid cents value: '{}'", token))?;
            intervals.push(SclInterval::Cents(cents));
        } else if token.contains('/') {
            // Ratio format
            let parts: Vec<&str> = token.split('/').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid ratio: '{}'", token));
            }
            let num: u64 = parts[0].parse()
                .map_err(|_| format!("Invalid ratio numerator: '{}'", parts[0]))?;
            let den: u64 = parts[1].parse()
                .map_err(|_| format!("Invalid ratio denominator: '{}'", parts[1]))?;
            if den == 0 {
                return Err(format!("Zero denominator in ratio: '{}'", token));
            }
            intervals.push(SclInterval::Ratio { num, den });
        } else {
            // Integer — treat as ratio N/1
            let num: u64 = token.parse()
                .map_err(|_| format!("Invalid interval value: '{}'", token))?;
            intervals.push(SclInterval::Ratio { num, den: 1 });
        }
    }

    Ok(SclScale { description, intervals })
}

/// Parse a .kbm string into a KbmMapping.
pub fn parse_kbm(input: &str) -> Result<KbmMapping, String> {
    let mut lines = input.lines()
        .map(|l| l.trim())
        .filter(|l| !l.starts_with('!') && !l.is_empty());

    let parse_int = |lines: &mut dyn Iterator<Item = &str>, label: &str| -> Result<i64, String> {
        let s = lines.next().ok_or(format!("Missing {}", label))?;
        s.parse().map_err(|_| format!("Invalid {}: '{}'", label, s))
    };

    let map_size = parse_int(&mut lines, "map size")? as usize;
    let first_note = parse_int(&mut lines, "first note")? as u8;
    let last_note = parse_int(&mut lines, "last note")? as u8;
    let _middle_note = parse_int(&mut lines, "middle note")?;
    let reference_note = parse_int(&mut lines, "reference note")? as u8;
    let reference_frequency: f64 = lines.next()
        .ok_or("Missing reference frequency")?
        .parse()
        .map_err(|_| "Invalid reference frequency".to_string())?;
    let formal_octave = parse_int(&mut lines, "formal octave")? as usize;

    let mut key_map = Vec::with_capacity(map_size);
    for _ in 0..map_size {
        let s = lines.next().ok_or("Not enough key map entries")?;
        if s == "x" || s == "X" {
            key_map.push(None);
        } else {
            let d: u32 = s.parse()
                .map_err(|_| format!("Invalid key map entry: '{}'", s))?;
            key_map.push(Some(d));
        }
    }

    Ok(KbmMapping {
        map_size,
        first_note,
        last_note,
        reference_note,
        reference_frequency,
        reference_degree: 0,
        formal_octave,
        key_map,
    })
}

/// Tuning using a Scala .scl scale and optional .kbm keyboard mapping.
#[derive(Debug, Clone)]
pub struct ScalaTuning {
    /// Pre-computed ratios for all degrees in one period, including the implicit 1/1 at index 0.
    ratios: Vec<f64>,
    /// The interval of equivalence (typically 2.0 for octave-repeating scales).
    equave_ratio: f64,
    /// Reference frequency for degree 0.
    base_frequency: f64,
}

impl ScalaTuning {
    /// Construct from a parsed SclScale. `base_frequency` is the Hz for degree 0 (the 1/1).
    pub fn from_scl(scl: &SclScale, base_frequency: f64) -> Self {
        let mut ratios = Vec::with_capacity(scl.intervals.len() + 1);
        ratios.push(1.0); // unison
        for interval in &scl.intervals {
            ratios.push(interval.to_ratio());
        }
        // The last ratio in .scl is the interval of equivalence (equave)
        let equave_ratio = ratios.last().copied().unwrap_or(2.0);
        // Remove the equave from the scale degrees — it belongs to the next period
        ratios.pop();
        Self {
            ratios,
            equave_ratio,
            base_frequency,
        }
    }

    /// Construct from raw .scl text and optional .kbm text.
    pub fn from_text(scl_text: &str, _kbm_text: Option<&str>, base_frequency: f64) -> Result<Self, String> {
        let scl = parse_scl(scl_text)?;
        Ok(Self::from_scl(&scl, base_frequency))
    }
}

impl Tuning for ScalaTuning {
    fn frequency_for_degree(&self, degree: i32) -> f64 {
        if self.ratios.is_empty() {
            return self.base_frequency;
        }
        let size = self.ratios.len() as i32;

        // Euclidean division
        let scale_degree = ((degree % size) + size) % size;
        let period = if degree >= 0 {
            degree / size
        } else {
            (degree - size + 1) / size
        };

        let ratio = self.ratios[scale_degree as usize];
        let equave_factor = self.equave_ratio.powi(period);

        self.base_frequency * ratio * equave_factor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PYTHAGOREAN_SCL: &str = "\
! pythagorean.scl
Pythagorean 7-note scale
7
9/8
5/4
4/3
3/2
5/3
15/8
2/1
";

    #[test]
    fn test_parse_scl() {
        let scl = parse_scl(PYTHAGOREAN_SCL).unwrap();
        assert_eq!(scl.description, "Pythagorean 7-note scale");
        assert_eq!(scl.intervals.len(), 7);
    }

    #[test]
    fn test_scala_unison() {
        let scl = parse_scl(PYTHAGOREAN_SCL).unwrap();
        let t = ScalaTuning::from_scl(&scl, 261.625);
        assert!((t.frequency_for_degree(0) - 261.625).abs() < 1e-6);
    }

    #[test]
    fn test_scala_perfect_fifth() {
        let scl = parse_scl(PYTHAGOREAN_SCL).unwrap();
        let t = ScalaTuning::from_scl(&scl, 261.625);
        let expected = 261.625 * 3.0 / 2.0;
        assert!((t.frequency_for_degree(4) - expected).abs() < 1e-6);
    }

    #[test]
    fn test_scala_octave_up() {
        let scl = parse_scl(PYTHAGOREAN_SCL).unwrap();
        let t = ScalaTuning::from_scl(&scl, 261.625);
        // Degree 7 wraps → period 1, degree 0 → base * equave(2.0)
        let expected = 261.625 * 2.0;
        assert!((t.frequency_for_degree(7) - expected).abs() < 1e-6);
    }

    #[test]
    fn test_scala_octave_down() {
        let scl = parse_scl(PYTHAGOREAN_SCL).unwrap();
        let t = ScalaTuning::from_scl(&scl, 261.625);
        // Degree -7 → period -1, degree 0 → base / equave
        let expected = 261.625 / 2.0;
        assert!((t.frequency_for_degree(-7) - expected).abs() < 1e-6);
    }

    #[test]
    fn test_cents_format() {
        let scl_text = "\
! 12tet.scl
12-TET
12
100.0
200.0
300.0
400.0
500.0
600.0
700.0
800.0
900.0
1000.0
1100.0
1200.0
";
        let scl = parse_scl(scl_text).unwrap();
        let t = ScalaTuning::from_scl(&scl, 440.0);
        // Degree 12 should be one octave up
        assert!((t.frequency_for_degree(12) - 880.0).abs() < 1e-3);
    }

    const SIMPLE_KBM: &str = "\
! Simple mapping
12
0
127
60
69
440.000000
12
0
1
2
3
4
5
6
7
8
9
10
11
";

    #[test]
    fn test_parse_kbm() {
        let kbm = parse_kbm(SIMPLE_KBM).unwrap();
        assert_eq!(kbm.map_size, 12);
        assert_eq!(kbm.reference_note, 69);
        assert!((kbm.reference_frequency - 440.0).abs() < 1e-6);
        assert_eq!(kbm.key_map.len(), 12);
    }
}
