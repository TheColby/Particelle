use criterion::{black_box, criterion_group, criterion_main, Criterion};
use particelle_midi::{MidiRouter, RoutingRule, MidiEvent, MidiEventKind};

fn criterion_benchmark(c: &mut Criterion) {
    let rules = vec![
        RoutingRule::direct("midi.cc.1", "density_mod"),
        RoutingRule::with_transform("midi.cc.74", "filter_cutoff", 20000.0, 20.0),
    ];
    let router = MidiRouter::new(rules);

    let events = vec![
        MidiEvent {
            frame_offset: 0,
            kind: MidiEventKind::ControlChange { channel: 1, cc: 1, value: 0.5 },
        },
        MidiEvent {
            frame_offset: 10,
            kind: MidiEventKind::ControlChange { channel: 1, cc: 74, value: 1.0 },
        },
        MidiEvent {
            frame_offset: 20,
            kind: MidiEventKind::PitchBend { channel: 1, value: 0.5 },
        },
    ];

    c.bench_function("router_process", |b| b.iter(|| router.process(black_box(&events))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
