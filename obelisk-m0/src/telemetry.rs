//! Fixed-size callback timing telemetry suitable for realtime use.

use std::time::Duration;

const BUCKETS: usize = 64;
const MAX_BUDGET_MULTIPLE: f64 = 2.0;

#[derive(Debug, Clone)]
pub struct CallbackHistogram {
    buckets: [u64; BUCKETS],
    samples: u64,
    max_ratio: f64,
}

impl CallbackHistogram {
    pub const fn new() -> Self {
        Self {
            buckets: [0; BUCKETS],
            samples: 0,
            max_ratio: 0.0,
        }
    }

    pub fn observe(&mut self, elapsed: Duration, callback_budget: Duration) {
        let budget = callback_budget.as_secs_f64();
        if budget <= 0.0 {
            return;
        }
        let ratio = elapsed.as_secs_f64() / budget;
        let normalized = (ratio / MAX_BUDGET_MULTIPLE).clamp(0.0, 1.0);
        let index = ((normalized * (BUCKETS - 1) as f64).floor() as usize).min(BUCKETS - 1);
        self.buckets[index] += 1;
        self.samples += 1;
        self.max_ratio = self.max_ratio.max(ratio);
    }

    pub fn percentile_ratio(&self, percentile: f64) -> f64 {
        if self.samples == 0 {
            return 0.0;
        }
        let target = (self.samples as f64 * percentile.clamp(0.0, 1.0)).ceil() as u64;
        let mut cumulative = 0_u64;
        for (index, count) in self.buckets.iter().enumerate() {
            cumulative += count;
            if cumulative >= target {
                return (index + 1) as f64 / BUCKETS as f64 * MAX_BUDGET_MULTIPLE;
            }
        }
        MAX_BUDGET_MULTIPLE
    }

    pub fn max_ratio(&self) -> f64 {
        self.max_ratio
    }

    pub fn samples(&self) -> u64 {
        self.samples
    }
}

impl Default for CallbackHistogram {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn histogram_reports_callback_budget_ratios() {
        let mut histogram = CallbackHistogram::new();
        let budget = Duration::from_millis(10);
        histogram.observe(Duration::from_millis(5), budget);
        histogram.observe(Duration::from_millis(7), budget);
        assert_eq!(histogram.samples(), 2);
        assert!(histogram.percentile_ratio(0.99) >= 0.68);
        assert!((histogram.max_ratio() - 0.7).abs() < 1e-9);
    }
}
