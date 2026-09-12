//! Opt-in wall-clock frame interval probe; not GPU timing or a benchmark target.
use bevy::prelude::*;
#[derive(Default)]
pub struct Samples {
    frames: usize,
    values: Vec<f64>,
    done: bool,
}
pub fn record(time: Res<Time<Real>>, game: Res<crate::Playground>, mut samples: Local<Samples>) {
    if samples.done || std::env::var_os("BURNHOP_FRAME_PROFILE").is_none() {
        return;
    }
    samples.frames += 1;
    if samples.frames > 180 {
        samples.values.push(time.delta_secs_f64() * 1000.);
    }
    if samples.values.len() == 1800 {
        samples.values.sort_by(f64::total_cmp);
        let v = &samples.values;
        println!(
            "FRAME_PROFILE n={} mean_ms={:.3} median_ms={:.3} p95_ms={:.3} p99_ms={:.3} over_16_7ms={}",
            v.len(),
            v.iter().sum::<f64>() / v.len() as f64,
            v[v.len() / 2],
            v[v.len() * 95 / 100],
            v[v.len() * 99 / 100],
            v.iter().filter(|x| **x > 16.7).count()
        );
        if let Some(online) = &game.online
            && let Some(p) = &online.prediction
        {
            println!(
                "CLIENT_METRICS reconciliations={} corrections={} correction_px_p95={:.3} correction_px_max={:.3} history_peak={} snapshots={} lead={} skipped_slots={} rtt_ms={:.3}",
                p.corrections.count,
                p.corrections.nonzero,
                p.corrections.percentile(0.95),
                p.corrections.max,
                p.history_peak,
                p.snapshot_len(),
                p.target_lead,
                p.rescheduled_slots,
                online.network.measured_rtt() * 1000.
            );
        }
        samples.done = true;
    }
}
