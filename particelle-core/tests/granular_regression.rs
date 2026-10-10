use particelle_core::audio_block::AudioBlock;
use particelle_core::engine::{Engine, EngineConfig, GranularEngine};
use particelle_core::grain::{Cloud, Grain};
use particelle_core::layout::AudioLayout;
use particelle_core::pool::GrainPool;
use particelle_core::spatializer::AmbisonicEncoder;
use particelle_params::signal::ParamSignal;
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::Arc;

thread_local! {
    static COUNT_ALLOCATIONS: Cell<bool> = const { Cell::new(false) };
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
}

struct CountingAllocator;

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        COUNT_ALLOCATIONS.with(|enabled| {
            if enabled.get() {
                ALLOCATIONS.with(|count| count.set(count.get() + 1));
            }
        });
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        COUNT_ALLOCATIONS.with(|enabled| {
            if enabled.get() {
                ALLOCATIONS.with(|count| count.set(count.get() + 1));
            }
        });
        System.realloc(ptr, layout, size)
    }
}

fn engine(density: f64, duration: f64, capacity: usize) -> GranularEngine {
    let layout = AudioLayout::mono();
    let mut engine = GranularEngine::new(
        EngineConfig::new(10.0, 4).unwrap(),
        layout.clone(),
        Box::new(AmbisonicEncoder::new(layout)),
        Box::new(particelle_params::context::NullFields),
    )
    .unwrap();
    let source = Arc::new(vec![vec![1.0; 16]]);
    let window = Arc::from(vec![1.0]);
    let pool = GrainPool::new(capacity, source, window, 1);
    let mut cloud = Cloud::new("timing".into(), pool);
    cloud.density = ParamSignal::Const(density);
    cloud.duration = ParamSignal::Const(duration);
    cloud.amplitude = ParamSignal::Const(1.0);
    engine.add_cloud(cloud);
    engine
}

#[test]
fn fractional_onsets_cross_block_boundaries() {
    let mut engine = engine(4.0, 0.1, 1);
    let mut block = AudioBlock::new(1, 4);
    let mut samples = Vec::new();
    for _ in 0..3 {
        engine.process(&mut block).unwrap();
        samples.extend_from_slice(&block.channels[0]);
    }
    assert_eq!(
        samples,
        [1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0]
    );
    assert_eq!(engine.state.frame, 12);
}

#[test]
fn slot_reuse_does_not_erase_earlier_samples() {
    let mut engine = engine(5.0, 0.2, 1);
    let mut block = AudioBlock::new(1, 6);
    engine.process(&mut block).unwrap();
    assert_eq!(block.channels[0], [1.0; 6]);
}

#[test]
fn nonlooping_grain_stops_at_edge_without_cross_edge_interpolation() {
    let source = Arc::new(vec![vec![1.0, 3.0]]);
    let window = Arc::from(vec![1.0]);
    let mut grain = Grain::new(source, window, 1);
    grain.loop_source = false;
    grain.activate(1.5, 4.0, 1.0, &[1.0]);
    let mut block = AudioBlock::new(1, 4);
    assert!(!grain.process(&mut block));
    assert_eq!(block.channels[0], [3.0, 0.0, 0.0, 0.0]);
    grain.activate(-0.5, 4.0, 1.0, &[1.0]);
    block.silence();
    assert!(!grain.process(&mut block));
    assert_eq!(block.channels[0], [0.0; 4]);
}

#[test]
fn looping_grain_interpolates_across_edge_and_wraps_reverse() {
    let source = Arc::new(vec![vec![1.0, 3.0]]);
    let window = Arc::from(vec![1.0]);
    let mut grain = Grain::new(source, window, 1);
    grain.activate(1.5, 3.0, -1.0, &[1.0]);
    let mut block = AudioBlock::new(1, 3);
    assert!(!grain.process(&mut block));
    assert_eq!(block.channels[0], [2.0, 2.0, 2.0]);
}

#[test]
fn cloud_loop_flag_applies_to_new_grains() {
    let mut engine = engine(2.5, 0.4, 1);
    engine.clouds[0].loop_source = false;
    engine.clouds[0].position = ParamSignal::Const(1.5);
    let mut block = AudioBlock::new(1, 4);
    engine.process(&mut block).unwrap();
    assert_eq!(block.channels[0], [1.0, 0.0, 0.0, 0.0]);
    assert!(!engine.clouds[0].pool.grains[0].loop_source);
}

#[test]
fn position_uses_source_rate_with_engine_rate_fallback() {
    let mut engine = engine(2.5, 0.1, 1);
    let source = Arc::new(vec![(0..32).map(f64::from).collect()]);
    engine.clouds[0].pool.grains[0].source = source;
    engine.clouds[0].position = ParamSignal::Const(1.0);
    engine.clouds[0].loop_source = false;
    let mut block = AudioBlock::new(1, 4);

    engine.process(&mut block).unwrap();
    assert_eq!(block.channels[0], [10.0, 0.0, 0.0, 0.0]);

    engine.clouds[0].source_sample_rate = Some(20.0);
    engine.process(&mut block).unwrap();
    assert_eq!(block.channels[0], [20.0, 0.0, 0.0, 0.0]);
}

#[test]
fn processing_does_not_allocate_after_setup() {
    let mut engine = engine(5.0, 0.3, 2);
    let mut block = AudioBlock::new(1, 4);
    engine.process(&mut block).unwrap();
    ALLOCATIONS.with(|count| count.set(0));
    COUNT_ALLOCATIONS.with(|enabled| enabled.set(true));
    for _ in 0..8 {
        engine.process(&mut block).unwrap();
    }
    COUNT_ALLOCATIONS.with(|enabled| enabled.set(false));
    let allocations = ALLOCATIONS.with(Cell::get);
    assert_eq!(allocations, 0);
}
