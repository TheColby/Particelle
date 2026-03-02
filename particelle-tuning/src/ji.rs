use crate::Tuning;

#[derive(Debug, Clone)]
pub struct JiTuning {}

impl Tuning for JiTuning {
    fn frequency_for_degree(&self, _degree: i32) -> f64 { 440.0 }
}
