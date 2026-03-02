use crate::Tuning;

#[derive(Debug, Clone)]
pub struct ScalaTuning {}

impl Tuning for ScalaTuning {
    fn frequency_for_degree(&self, _degree: i32) -> f64 { 440.0 }
}
