//! Bounded rolling samples. Counters measure application work, not GPU or wire overhead.
use std::collections::VecDeque;
#[derive(Debug, Default)]
pub struct Samples {
    values: VecDeque<f64>,
    pub count: u64,
    pub nonzero: u64,
    pub total: f64,
    pub max: f64,
}
impl Samples {
    pub fn add(&mut self, value: f64) {
        if !value.is_finite() || value < 0. {
            return;
        }
        if self.values.len() == 4096 {
            self.values.pop_front();
        }
        self.values.push_back(value);
        self.count += 1;
        self.nonzero += u64::from(value > 1e-6);
        self.total += value;
        self.max = self.max.max(value);
    }
    pub fn percentile(&self, p: f64) -> f64 {
        let mut sorted: Vec<_> = self.values.iter().copied().collect();
        sorted.sort_by(f64::total_cmp);
        sorted
            .get(((sorted.len().saturating_sub(1)) as f64 * p.clamp(0., 1.)) as usize)
            .copied()
            .unwrap_or(0.)
    }
    pub fn len(&self) -> usize {
        self.values.len()
    }
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}
