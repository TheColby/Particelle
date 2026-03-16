#[path = "../src/osc_control.rs"]
mod osc_control;

use particelle_midi::routing::{MidiRouter, RoutingRule};
use particelle_midi::{MidiEvent, MidiEventKind};
use rosc::{OscMessage, OscPacket, OscType};
use std::collections::HashMap;

fn osc_message(addr: &str, args: Vec<OscType>) -> OscPacket {
    OscPacket::Message(OscMessage {
        addr: addr.to_string(),
        args,
    })
}

#[test]
fn osc_field_message_emits_ack_and_enqueues_update() {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut replies = Vec::new();

    osc_control::handle_osc_packet(
        osc_message("/field/density", vec![OscType::Float(12.5)]),
        &tx,
        &mut replies,
    );

    let queued = rx.try_recv().expect("expected queued update");
    assert_eq!(queued.0, "density");
    assert!((queued.1 - 12.5).abs() < 1e-9);

    assert_eq!(replies.len(), 1);
    match &replies[0] {
        OscPacket::Message(reply) => {
            assert_eq!(reply.addr, "/ack");
            assert_eq!(
                reply.args,
                vec![
                    OscType::String("density".to_string()),
                    OscType::Double(12.5)
                ]
            );
        }
        _ => panic!("expected OSC message reply"),
    }
}

#[test]
fn osc_ping_emits_pong() {
    let (tx, _rx) = std::sync::mpsc::channel();
    let mut replies = Vec::new();

    osc_control::handle_osc_packet(osc_message("/ping", vec![]), &tx, &mut replies);

    assert_eq!(replies.len(), 1);
    match &replies[0] {
        OscPacket::Message(reply) => assert_eq!(reply.addr, "/pong"),
        _ => panic!("expected OSC message reply"),
    }
}

#[test]
fn unsupported_osc_address_emits_error() {
    let (tx, _rx) = std::sync::mpsc::channel();
    let mut replies = Vec::new();

    osc_control::handle_osc_packet(osc_message("/unsupported", vec![]), &tx, &mut replies);

    assert_eq!(replies.len(), 1);
    match &replies[0] {
        OscPacket::Message(reply) => assert_eq!(reply.addr, "/error"),
        _ => panic!("expected OSC message reply"),
    }
}

#[test]
fn osc_bundle_order_is_deterministic() {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut replies = Vec::new();
    let bundle = OscPacket::Bundle(rosc::OscBundle {
        timetag: (0, 1).into(),
        content: vec![
            osc_message("/field/density", vec![OscType::Double(0.25)]),
            osc_message("/field/density", vec![OscType::Double(0.75)]),
        ],
    });

    osc_control::handle_osc_packet(bundle, &tx, &mut replies);
    let mut fields = HashMap::new();
    let applied = osc_control::drain_field_updates(&rx, &mut fields);

    assert_eq!(applied, 2);
    assert_eq!(fields.get("density").copied(), Some(0.75));
    assert_eq!(replies.len(), 2);
}

#[test]
fn osc_overrides_midi_when_targeting_same_field_in_same_block() {
    let router = MidiRouter::new(vec![RoutingRule::direct("midi.cc.1", "density_mod")]);
    let events = vec![MidiEvent {
        frame_offset: 0,
        kind: MidiEventKind::ControlChange {
            channel: 1,
            cc: 1,
            value: 0.25,
        },
    }];

    let mut fields = router.process(&events);
    assert_eq!(fields.get("density_mod").copied(), Some(0.25));

    let (tx, rx) = std::sync::mpsc::channel();
    let mut replies = Vec::new();
    osc_control::handle_osc_packet(
        osc_message("/field/density_mod", vec![OscType::Double(0.75)]),
        &tx,
        &mut replies,
    );
    let applied = osc_control::drain_field_updates(&rx, &mut fields);

    assert_eq!(applied, 1);
    assert_eq!(fields.get("density_mod").copied(), Some(0.75));
    assert_eq!(replies.len(), 1);
}

#[test]
fn osc_updates_become_visible_on_next_drain_step() {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut replies = Vec::new();
    let mut fields = HashMap::new();

    osc_control::handle_osc_packet(
        osc_message("/field/position_mod", vec![OscType::Double(0.1)]),
        &tx,
        &mut replies,
    );
    assert!(!fields.contains_key("position_mod"));

    let applied_block_1 = osc_control::drain_field_updates(&rx, &mut fields);
    assert_eq!(applied_block_1, 1);
    assert_eq!(fields.get("position_mod").copied(), Some(0.1));

    osc_control::handle_osc_packet(
        osc_message("/field/position_mod", vec![OscType::Double(0.9)]),
        &tx,
        &mut replies,
    );
    assert_eq!(fields.get("position_mod").copied(), Some(0.1));

    let applied_block_2 = osc_control::drain_field_updates(&rx, &mut fields);
    assert_eq!(applied_block_2, 1);
    assert_eq!(fields.get("position_mod").copied(), Some(0.9));
}
