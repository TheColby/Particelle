use rosc::{OscMessage, OscPacket, OscType};
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};

fn osc_arg_to_f64(arg: &OscType) -> Option<f64> {
    match arg {
        OscType::Float(v) => Some(*v as f64),
        OscType::Double(v) => Some(*v),
        OscType::Int(v) => Some(*v as f64),
        _ => None,
    }
}

fn osc_msg(addr: &str, args: Vec<OscType>) -> OscPacket {
    OscPacket::Message(OscMessage {
        addr: addr.to_string(),
        args,
    })
}

/// Handle one OSC packet and collect control updates + reply messages.
///
/// Supported messages:
/// - `/field/<name> <number>` -> enqueue field update, emit `/ack`
/// - `/ping` -> emit `/pong "particelle"`
/// - unsupported address / malformed payload -> emit `/error`
pub fn handle_osc_packet(
    packet: OscPacket,
    tx: &Sender<(String, f64)>,
    replies: &mut Vec<OscPacket>,
) {
    match packet {
        OscPacket::Message(msg) => {
            if msg.addr == "/ping" {
                replies.push(osc_msg(
                    "/pong",
                    vec![OscType::String("particelle".to_string())],
                ));
                return;
            }

            if msg.addr.starts_with("/field/") {
                let field_name = msg.addr.trim_start_matches("/field/").to_string();
                if field_name.is_empty() {
                    replies.push(osc_msg(
                        "/error",
                        vec![OscType::String("Field name cannot be empty".to_string())],
                    ));
                    return;
                }

                if let Some(arg) = msg.args.first() {
                    if let Some(val) = osc_arg_to_f64(arg) {
                        let _ = tx.send((field_name.clone(), val));
                        replies.push(osc_msg(
                            "/ack",
                            vec![OscType::String(field_name), OscType::Double(val)],
                        ));
                    } else {
                        replies.push(osc_msg(
                            "/error",
                            vec![OscType::String(format!(
                                "Unsupported argument type for {}",
                                msg.addr
                            ))],
                        ));
                    }
                } else {
                    replies.push(osc_msg(
                        "/error",
                        vec![OscType::String(format!(
                            "Missing numeric argument for {}",
                            msg.addr
                        ))],
                    ));
                }
                return;
            }

            replies.push(osc_msg(
                "/error",
                vec![OscType::String(format!(
                    "Unsupported OSC address '{}'",
                    msg.addr
                ))],
            ));
        }
        OscPacket::Bundle(bundle) => {
            for packet in bundle.content {
                handle_osc_packet(packet, tx, replies);
            }
        }
    }
}

/// Drain all queued OSC field updates into the field map.
///
/// If multiple updates target the same field in one drain call, last write wins.
pub fn drain_field_updates(
    rx: &Receiver<(String, f64)>,
    fields: &mut HashMap<String, f64>,
) -> usize {
    let mut applied = 0usize;
    for (field_name, val) in rx.try_iter() {
        fields.insert(field_name, val);
        applied += 1;
    }
    applied
}
