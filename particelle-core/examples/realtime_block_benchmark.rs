use particelle_core::audio_block::AudioBlock;
use particelle_core::engine::{Engine, EngineConfig, GranularEngine};
use particelle_core::grain::Cloud;
use particelle_core::layout::AudioLayout;
use particelle_core::pool::GrainPool;
use particelle_core::spatializer::AmplitudePanner;
use particelle_params::signal::ParamSignal;
use std::error::Error;
use std::sync::Arc;
use std::time::Instant;

fn env_usize(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(default)
}

fn env_f64(name: &str, default: f64) -> f64 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(default)
}

fn percentile(sorted: &[f64], pct: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((pct / 100.0) * (sorted.len().saturating_sub(1) as f64)).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn main() -> Result<(), Box<dyn Error>> {
    let sample_rate = env_f64("BENCH_SAMPLE_RATE", 48_000.0);
    let block_size = env_usize("BENCH_BLOCK_SIZE", 256);
    let warmup_blocks = env_usize("BENCH_WARMUP_BLOCKS", 256);
    let measure_blocks = env_usize("BENCH_MEASURE_BLOCKS", 4_096);
    let max_avg_ratio = env_f64("MAX_AVG_BLOCK_RATIO", 0.45);
    let max_p95_ratio = env_f64("MAX_P95_BLOCK_RATIO", 0.90);

    let config = EngineConfig::new(sample_rate, block_size)?;
    let layout = AudioLayout::stereo();
    let panner = Box::new(AmplitudePanner::new(layout.clone()));
    let mut engine = GranularEngine::new(
        config,
        layout,
        panner,
        Box::new(particelle_params::context::NullFields),
    )?;

    let source_len = (sample_rate as usize).max(block_size * 8);
    let mut src = Vec::with_capacity(source_len);
    for i in 0..source_len {
        // Deterministic, non-constant source content to avoid degenerate fast paths.
        let t = i as f64 / sample_rate;
        src.push((t * 220.0 * std::f64::consts::TAU).sin() * 0.5);
    }
    let source = Arc::new(vec![src]);
    let window = Arc::from(vec![1.0; 1024]);
    let pool = GrainPool::new(4096, source, window, 2);
    let mut cloud = Cloud::new("bench_cloud".to_string(), pool);
    cloud.density = ParamSignal::Const(80.0);
    cloud.duration = ParamSignal::Const(0.08);
    cloud.amplitude = ParamSignal::Const(0.7);
    cloud.width = ParamSignal::Const(0.5);
    engine.add_cloud(cloud);

    let mut block = AudioBlock::new(2, block_size);

    for _ in 0..warmup_blocks {
        engine.process(&mut block)?;
    }

    let mut durations_us = Vec::with_capacity(measure_blocks);
    for _ in 0..measure_blocks {
        let start = Instant::now();
        engine.process(&mut block)?;
        durations_us.push(start.elapsed().as_secs_f64() * 1_000_000.0);
    }

    let mut sorted = durations_us.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let avg_us = durations_us.iter().sum::<f64>() / durations_us.len() as f64;
    let p95_us = percentile(&sorted, 95.0);
    let realtime_budget_us = (block_size as f64 / sample_rate) * 1_000_000.0;
    let avg_ratio = avg_us / realtime_budget_us;
    let p95_ratio = p95_us / realtime_budget_us;
    let avg_realtime_factor = realtime_budget_us / avg_us;

    println!(
        "sample_rate={} block_size={} warmup_blocks={} measure_blocks={}",
        sample_rate, block_size, warmup_blocks, measure_blocks
    );
    println!(
        "avg_block_us={:.3} p95_block_us={:.3} realtime_budget_us={:.3}",
        avg_us, p95_us, realtime_budget_us
    );
    println!(
        "avg_block_ratio={:.6} p95_block_ratio={:.6} avg_realtime_factor={:.3}",
        avg_ratio, p95_ratio, avg_realtime_factor
    );

    if avg_ratio > max_avg_ratio {
        eprintln!(
            "Average block latency ratio {:.6} exceeded max {:.6}",
            avg_ratio, max_avg_ratio
        );
        std::process::exit(1);
    }

    if p95_ratio > max_p95_ratio {
        eprintln!(
            "P95 block latency ratio {:.6} exceeded max {:.6}",
            p95_ratio, max_p95_ratio
        );
        std::process::exit(1);
    }

    Ok(())
}
