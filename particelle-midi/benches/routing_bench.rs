use criterion::{black_box, criterion_group, criterion_main, Criterion};
use particelle_midi::routing::{MidiRouter, RoutingRule};
use particelle_midi::events::{MidiEvent, MidiEventKind};

fn bench_routing(c: &mut Criterion) {
    let rules = vec![
        RoutingRule::direct("midi.cc.1", "density_mod"),
        RoutingRule::with_transform("midi.cc.74", "filter_cutoff", 20000.0, 20.0),
        RoutingRule::direct("midi.pitchbend", "pitch_mod"),
        RoutingRule::direct("midi.pressure", "volume_mod"),
    ];
    let router = MidiRouter::new(rules);

    let mut events = Vec::new();
    for i in 0..100 {
        events.push(MidiEvent {
            frame_offset: i,
            kind: MidiEventKind::ControlChange { channel: 1, cc: 1, value: 0.5 },
        });
        events.push(MidiEvent {
            frame_offset: i,
            kind: MidiEventKind::ControlChange { channel: 1, cc: 74, value: 0.8 },
        });
        events.push(MidiEvent {
            frame_offset: i,
            kind: MidiEventKind::PitchBend { channel: 1, value: 0.1 },
        });
        events.push(MidiEvent {
            frame_offset: i,
            kind: MidiEventKind::ChannelPressure { channel: 1, value: 0.3 },
        });
    }

    c.bench_function("midi_routing_process", |b| {
        b.iter(|| router.process(black_box(&events)))
    });
}

criterion_group!(benches, bench_routing);
criterion_main!(benches);
