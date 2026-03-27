use particelle_core::audio_block::AudioBlock;
use particelle_core::engine::{Engine, EngineConfig, GranularEngine};
use particelle_core::grain::Cloud;
use particelle_core::layout::AudioLayout;
use particelle_core::pool::GrainPool;
use particelle_core::spatializer::AmplitudePanner;
use particelle_params::signal::{OscShape, ParamSignal};
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

fn make_cloud(id: &str, pool: GrainPool, phase: f64, base_density: f64, base_rate: f64) -> Cloud {
    let mut cloud = Cloud::new(id.to_string(), pool);
    cloud.density = ParamSignal::ScaleOffset {
        input: Box::new(ParamSignal::Oscillator {
            shape: OscShape::Sine,
            freq: Box::new(ParamSignal::Const(0.19)),
            phase,
        }),
        scale: 35.0,
        offset: base_density,
    };
    cloud.duration = ParamSignal::ScaleOffset {
        input: Box::new(ParamSignal::Oscillator {
            shape: OscShape::Triangle,
            freq: Box::new(ParamSignal::Const(0.11)),
            phase: phase * 0.5,
        }),
        scale: 0.06,
        offset: 0.03,
    };
    cloud.amplitude = ParamSignal::ScaleOffset {
        input: Box::new(ParamSignal::Oscillator {
            shape: OscShape::Sine,
            freq: Box::new(ParamSignal::Const(0.07)),
            phase: phase * 0.25,
        }),
        scale: 0.35,
        offset: 0.15,
    };
    cloud.width = ParamSignal::ScaleOffset {
        input: Box::new(ParamSignal::Oscillator {
            shape: OscShape::Triangle,
            freq: Box::new(ParamSignal::Const(0.05)),
            phase: phase * 0.75,
        }),
        scale: 0.8,
        offset: 0.1,
    };
    cloud.playback_rate = ParamSignal::ScaleOffset {
        input: Box::new(ParamSignal::Oscillator {
            shape: OscShape::Sine,
            freq: Box::new(ParamSignal::Const(0.13)),
            phase: phase * 0.33,
        }),
        scale: 0.5,
        offset: base_rate,
    };
    cloud
}

fn main() -> Result<(), Box<dyn Error>> {
    let sample_rate = env_f64("BENCH_SAMPLE_RATE", 48_000.0);
    let block_size = env_usize("BENCH_BLOCK_SIZE", 256);
    let warmup_blocks = env_usize("BENCH_WARMUP_BLOCKS", 512);
    let soak_blocks = env_usize("BENCH_SOAK_BLOCKS", 12_000);
    let max_avg_ratio = env_f64("MAX_SOAK_AVG_BLOCK_RATIO", 0.55);
    let max_p99_ratio = env_f64("MAX_SOAK_P99_BLOCK_RATIO", 0.98);
    let max_xrun_ratio = env_f64("MAX_SOAK_XRUN_RATIO", 0.005);
    let max_consecutive_xruns = env_usize("MAX_SOAK_CONSECUTIVE_XRUNS", 2);

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
        let t = i as f64 / sample_rate;
        let sample = (t * 220.0 * std::f64::consts::TAU).sin() * 0.45
            + (t * 331.0 * std::f64::consts::TAU).sin() * 0.35
            + (t * 441.0 * std::f64::consts::TAU).sin() * 0.20;
        src.push(sample);
    }
    let source = Arc::new(vec![src]);
    let window = Arc::from(vec![1.0; 1024]);

    let cloud_a = make_cloud(
        "soak_a",
        GrainPool::new(4096, Arc::clone(&source), Arc::clone(&window), 2),
        0.13,
        55.0,
        0.85,
    );
    let cloud_b = make_cloud(
        "soak_b",
        GrainPool::new(4096, Arc::clone(&source), Arc::clone(&window), 2),
        0.47,
        62.0,
        1.15,
    );
    let cloud_c = make_cloud(
        "soak_c",
        GrainPool::new(4096, Arc::clone(&source), Arc::clone(&window), 2),
        0.79,
        48.0,
        0.65,
    );
    engine.add_cloud(cloud_a);
    engine.add_cloud(cloud_b);
    engine.add_cloud(cloud_c);

    let mut block = AudioBlock::new(2, block_size);

    for _ in 0..warmup_blocks {
        engine.process(&mut block)?;
    }

    let realtime_budget_us = (block_size as f64 / sample_rate) * 1_000_000.0;
    let mut durations_us = Vec::with_capacity(soak_blocks);
    let mut xrun_blocks = 0usize;
    let mut current_xrun_streak = 0usize;
    let mut max_xrun_streak = 0usize;

    for _ in 0..soak_blocks {
        let start = Instant::now();
        engine.process(&mut block)?;
        let elapsed_us = start.elapsed().as_secs_f64() * 1_000_000.0;
        if elapsed_us > realtime_budget_us {
            xrun_blocks += 1;
            current_xrun_streak += 1;
            max_xrun_streak = max_xrun_streak.max(current_xrun_streak);
        } else {
            current_xrun_streak = 0;
        }
        durations_us.push(elapsed_us);
    }

    let mut sorted = durations_us.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let avg_us = durations_us.iter().sum::<f64>() / durations_us.len() as f64;
    let p95_us = percentile(&sorted, 95.0);
    let p99_us = percentile(&sorted, 99.0);
    let max_us = *sorted.last().unwrap_or(&0.0);
    let avg_ratio = avg_us / realtime_budget_us;
    let p95_ratio = p95_us / realtime_budget_us;
    let p99_ratio = p99_us / realtime_budget_us;
    let max_ratio = max_us / realtime_budget_us;
    let xrun_ratio = xrun_blocks as f64 / soak_blocks as f64;
    let avg_realtime_factor = realtime_budget_us / avg_us;

    println!(
        "sample_rate={} block_size={} warmup_blocks={} soak_blocks={}",
        sample_rate, block_size, warmup_blocks, soak_blocks
    );
    println!(
        "avg_block_us={:.3} p95_block_us={:.3} p99_block_us={:.3} max_block_us={:.3} realtime_budget_us={:.3}",
        avg_us, p95_us, p99_us, max_us, realtime_budget_us
    );
    println!(
        "avg_block_ratio={:.6} p95_block_ratio={:.6} p99_block_ratio={:.6} max_block_ratio={:.6}",
        avg_ratio, p95_ratio, p99_ratio, max_ratio
    );
    println!(
        "xrun_blocks={} xrun_ratio={:.6} max_consecutive_xruns={} avg_realtime_factor={:.3}",
        xrun_blocks, xrun_ratio, max_xrun_streak, avg_realtime_factor
    );

    if avg_ratio > max_avg_ratio {
        eprintln!(
            "Average soak block latency ratio {:.6} exceeded max {:.6}",
            avg_ratio, max_avg_ratio
        );
        std::process::exit(1);
    }
    if p99_ratio > max_p99_ratio {
        eprintln!(
            "P99 soak block latency ratio {:.6} exceeded max {:.6}",
            p99_ratio, max_p99_ratio
        );
        std::process::exit(1);
    }
    if xrun_ratio > max_xrun_ratio {
        eprintln!(
            "Soak XRUN ratio {:.6} exceeded max {:.6}",
            xrun_ratio, max_xrun_ratio
        );
        std::process::exit(1);
    }
    if max_xrun_streak > max_consecutive_xruns {
        eprintln!(
            "Soak max consecutive XRUN streak {} exceeded max {}",
            max_xrun_streak, max_consecutive_xruns
        );
        std::process::exit(1);
    }

    Ok(())
}
