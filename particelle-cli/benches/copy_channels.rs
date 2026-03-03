use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_copy_channels(c: &mut Criterion) {
    let mut group = c.benchmark_group("copy_channels");

    // Mock data
    let n_channels = 2;
    let frames = 1024;
    let mut buffer = vec![0.0f32; n_channels * frames];
    let mut source_channels = vec![vec![1.0f32; frames]; n_channels];

    group.bench_function("nested_loops", |b| b.iter(|| {
        for f in 0..frames {
            for ch in 0..n_channels {
                buffer[f * n_channels + ch] = source_channels[ch][f];
            }
        }
        black_box(&buffer);
    }));

    group.bench_function("interleaved_zipped", |b| b.iter(|| {
        for (f, chunk) in buffer.chunks_exact_mut(n_channels).enumerate().take(frames) {
            for ch in 0..n_channels {
                chunk[ch] = source_channels[ch][f];
            }
        }
        black_box(&buffer);
    }));

    group.finish();
}

criterion_group!(benches, benchmark_copy_channels);
criterion_main!(benches);
