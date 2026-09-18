//! MPE channel and voice lifecycle semantics for the M0 instrument.

use particelle_midi::{ExpressionKind, MidiEvent, MidiEventKind, NoteEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoicePhase {
    Free,
    Held,
    Released,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct VoiceExpression {
    pub pitch_bend_semitones: f64,
    pub pressure: f64,
    pub timbre: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct VoiceState {
    pub id: u64,
    pub phase: VoicePhase,
    pub channel: u8,
    pub note: u8,
    pub velocity: f64,
    pub release_velocity: f64,
    pub note_on_frame: u64,
    pub release_frame: u64,
    pub bound_to_channel: bool,
    pub expression: VoiceExpression,
    member_pitch_bend: f64,
    member_pressure: f64,
    member_timbre: f64,
}

impl VoiceState {
    pub const fn free() -> Self {
        Self {
            id: 0,
            phase: VoicePhase::Free,
            channel: 0,
            note: 0,
            velocity: 0.0,
            release_velocity: 0.0,
            note_on_frame: 0,
            release_frame: 0,
            bound_to_channel: false,
            expression: VoiceExpression {
                pitch_bend_semitones: 0.0,
                pressure: 0.0,
                timbre: 0.0,
            },
            member_pitch_bend: 0.0,
            member_pressure: 0.0,
            member_timbre: 0.5,
        }
    }

    pub fn is_active(&self) -> bool {
        self.phase != VoicePhase::Free
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ZoneConfig {
    pub master_channel: u8,
    pub member_first: u8,
    pub member_last: u8,
    pub default_member_pitch_range: f64,
    pub default_master_pitch_range: f64,
}

impl ZoneConfig {
    pub fn lower(member_channels: u8) -> Self {
        let count = member_channels.clamp(1, 15);
        Self {
            master_channel: 1,
            member_first: 2,
            member_last: 1 + count,
            default_member_pitch_range: 48.0,
            default_master_pitch_range: 2.0,
        }
    }

    pub fn upper(member_channels: u8) -> Self {
        let count = member_channels.clamp(1, 15);
        Self {
            master_channel: 16,
            member_first: 16 - count,
            member_last: 15,
            default_member_pitch_range: 48.0,
            default_master_pitch_range: 2.0,
        }
    }

    fn is_member(&self, channel: u8) -> bool {
        channel >= self.member_first && channel <= self.member_last
    }
}

#[derive(Debug, Clone, Copy)]
struct ChannelState {
    pitch_bend: f64,
    pressure: f64,
    timbre: f64,
    pitch_range_semitones: f64,
    rpn_msb: u8,
    rpn_lsb: u8,
}

impl ChannelState {
    const fn new(pitch_range_semitones: f64, timbre: f64) -> Self {
        Self {
            pitch_bend: 0.0,
            pressure: 0.0,
            timbre,
            pitch_range_semitones,
            rpn_msb: 127,
            rpn_lsb: 127,
        }
    }

    fn handle_cc(&mut self, cc: u8, value: f64) -> bool {
        let raw = (value.clamp(0.0, 1.0) * 127.0).round() as u8;
        match cc {
            101 => self.rpn_msb = raw,
            100 => self.rpn_lsb = raw,
            6 if self.rpn_msb == 0 && self.rpn_lsb == 0 => {
                self.pitch_range_semitones = f64::from(raw) + self.pitch_range_semitones.fract();
                return true;
            }
            38 if self.rpn_msb == 0 && self.rpn_lsb == 0 => {
                self.pitch_range_semitones =
                    self.pitch_range_semitones.floor() + f64::from(raw) / 100.0;
                return true;
            }
            74 => {
                self.timbre = value.clamp(0.0, 1.0);
                return true;
            }
            _ => {}
        }
        false
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct VoiceUpdate {
    pub started_slot: Option<usize>,
    pub released_slot: Option<usize>,
    pub stolen_slot: Option<usize>,
}

pub struct MpeVoiceManager {
    zone: ZoneConfig,
    channels: [ChannelState; 16],
    voices: Vec<VoiceState>,
    next_voice_id: u64,
}

impl MpeVoiceManager {
    pub fn new(zone: ZoneConfig, max_voices: usize) -> Self {
        let mut channels = [ChannelState::new(48.0, 0.5); 16];
        channels[usize::from(zone.master_channel - 1)] =
            ChannelState::new(zone.default_master_pitch_range, 0.0);
        for channel in zone.member_first..=zone.member_last {
            channels[usize::from(channel - 1)] =
                ChannelState::new(zone.default_member_pitch_range, 0.5);
        }
        Self {
            zone,
            channels,
            voices: vec![VoiceState::free(); max_voices.max(1)],
            next_voice_id: 1,
        }
    }

    pub fn voices(&self) -> &[VoiceState] {
        &self.voices
    }

    pub fn active_voice_count(&self) -> usize {
        self.voices.iter().filter(|voice| voice.is_active()).count()
    }

    pub fn handle_event(&mut self, event: &MidiEvent, absolute_frame: u64) -> VoiceUpdate {
        match &event.kind {
            MidiEventKind::Note(note) => self.handle_note(note.clone(), absolute_frame),
            MidiEventKind::PitchBend { channel, value } => {
                self.set_pitch_bend(*channel, *value);
                VoiceUpdate::default()
            }
            MidiEventKind::ChannelPressure { channel, value } => {
                self.set_pressure(*channel, *value);
                VoiceUpdate::default()
            }
            MidiEventKind::PolyPressure {
                channel,
                note,
                value,
            } => {
                self.set_poly_pressure(*channel, *note, *value);
                VoiceUpdate::default()
            }
            MidiEventKind::ControlChange { channel, cc, value } => {
                self.handle_cc(*channel, *cc, *value);
                VoiceUpdate::default()
            }
            MidiEventKind::Expression(expression) => {
                self.handle_direct_expression(
                    expression.channel,
                    expression.note,
                    expression.kind,
                    expression.value,
                );
                VoiceUpdate::default()
            }
            MidiEventKind::ProgramChange { .. } => VoiceUpdate::default(),
        }
    }

    pub fn free_slot(&mut self, slot: usize) {
        if let Some(voice) = self.voices.get_mut(slot) {
            *voice = VoiceState::free();
        }
    }

    pub fn all_notes_off(&mut self, absolute_frame: u64) {
        for voice in &mut self.voices {
            if voice.phase == VoicePhase::Held {
                voice.phase = VoicePhase::Released;
                voice.release_frame = absolute_frame;
            }
            voice.bound_to_channel = false;
        }
    }

    fn handle_note(&mut self, note: NoteEvent, absolute_frame: u64) -> VoiceUpdate {
        if !self.zone.is_member(note.channel) {
            return VoiceUpdate::default();
        }
        if note.is_on && note.velocity > 0.0 {
            self.note_on(note, absolute_frame)
        } else {
            self.note_off(note, absolute_frame)
        }
    }

    fn note_on(&mut self, note: NoteEvent, absolute_frame: u64) -> VoiceUpdate {
        let mut released_slot = None;
        for (slot, voice) in self.voices.iter_mut().enumerate() {
            if voice.is_active() && voice.bound_to_channel && voice.channel == note.channel {
                if voice.phase == VoicePhase::Held {
                    voice.phase = VoicePhase::Released;
                    voice.release_frame = absolute_frame;
                    released_slot = Some(slot);
                }
                voice.bound_to_channel = false;
            }
        }

        let free_slot = self
            .voices
            .iter()
            .position(|voice| voice.phase == VoicePhase::Free);
        let slot = free_slot.unwrap_or_else(|| self.voice_to_steal());
        let stolen_slot = self.voices[slot].is_active().then_some(slot);
        let channel = self.channels[usize::from(note.channel - 1)];

        self.voices[slot] = VoiceState {
            id: self.next_voice_id,
            phase: VoicePhase::Held,
            channel: note.channel,
            note: note.note,
            velocity: note.velocity.clamp(0.0, 1.0),
            release_velocity: note.velocity.clamp(0.0, 1.0),
            note_on_frame: absolute_frame,
            release_frame: 0,
            bound_to_channel: true,
            expression: VoiceExpression::default(),
            member_pitch_bend: channel.pitch_bend,
            member_pressure: channel.pressure,
            member_timbre: channel.timbre,
        };
        self.next_voice_id = self.next_voice_id.wrapping_add(1).max(1);
        self.refresh_voice(slot);

        VoiceUpdate {
            started_slot: Some(slot),
            released_slot,
            stolen_slot,
        }
    }

    fn note_off(&mut self, note: NoteEvent, absolute_frame: u64) -> VoiceUpdate {
        let slot = self.voices.iter().position(|voice| {
            voice.phase == VoicePhase::Held
                && voice.bound_to_channel
                && voice.channel == note.channel
                && voice.note == note.note
        });
        let Some(slot) = slot else {
            return VoiceUpdate::default();
        };
        let voice = &mut self.voices[slot];
        voice.phase = VoicePhase::Released;
        voice.release_velocity = note.velocity.clamp(0.0, 1.0);
        voice.release_frame = absolute_frame;
        VoiceUpdate {
            released_slot: Some(slot),
            ..VoiceUpdate::default()
        }
    }

    fn voice_to_steal(&self) -> usize {
        self.voices
            .iter()
            .enumerate()
            .filter(|(_, voice)| voice.phase == VoicePhase::Released)
            .min_by_key(|(_, voice)| voice.release_frame)
            .or_else(|| {
                self.voices
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, voice)| voice.note_on_frame)
            })
            .map(|(slot, _)| slot)
            .unwrap_or(0)
    }

    fn set_pitch_bend(&mut self, channel: u8, value: f64) {
        let Some(state) = self.channel_mut(channel) else {
            return;
        };
        state.pitch_bend = value.clamp(-1.0, 1.0);
        self.refresh_channel_or_zone(channel);
    }

    fn set_pressure(&mut self, channel: u8, value: f64) {
        let Some(state) = self.channel_mut(channel) else {
            return;
        };
        state.pressure = value.clamp(0.0, 1.0);
        self.refresh_channel_or_zone(channel);
    }

    fn set_poly_pressure(&mut self, channel: u8, note: u8, value: f64) {
        for voice in &mut self.voices {
            if voice.is_active() && voice.channel == channel && voice.note == note {
                voice.member_pressure = value.clamp(0.0, 1.0);
            }
        }
        self.refresh_all();
    }

    fn handle_cc(&mut self, channel: u8, cc: u8, value: f64) {
        let Some(state) = self.channel_mut(channel) else {
            return;
        };
        if state.handle_cc(cc, value) {
            self.refresh_channel_or_zone(channel);
        }
    }

    fn handle_direct_expression(
        &mut self,
        channel: u8,
        note: u8,
        kind: ExpressionKind,
        value: f64,
    ) {
        for voice in &mut self.voices {
            if !voice.is_active() || voice.channel != channel || voice.note != note {
                continue;
            }
            match kind {
                ExpressionKind::PitchBend => voice.member_pitch_bend = value.clamp(-1.0, 1.0),
                ExpressionKind::Pressure => voice.member_pressure = value.clamp(0.0, 1.0),
                ExpressionKind::Timbre => voice.member_timbre = value.clamp(0.0, 1.0),
            }
        }
        self.refresh_all();
    }

    fn channel_mut(&mut self, channel: u8) -> Option<&mut ChannelState> {
        if !(1..=16).contains(&channel) {
            return None;
        }
        self.channels.get_mut(usize::from(channel - 1))
    }

    fn refresh_channel_or_zone(&mut self, channel: u8) {
        if channel == self.zone.master_channel {
            self.refresh_all();
            return;
        }
        if !self.zone.is_member(channel) {
            return;
        }
        let state = self.channels[usize::from(channel - 1)];
        for voice in &mut self.voices {
            if voice.is_active() && voice.bound_to_channel && voice.channel == channel {
                voice.member_pitch_bend = state.pitch_bend;
                voice.member_pressure = state.pressure;
                voice.member_timbre = state.timbre;
            }
        }
        self.refresh_all();
    }

    fn refresh_all(&mut self) {
        for slot in 0..self.voices.len() {
            if self.voices[slot].is_active() {
                self.refresh_voice(slot);
            }
        }
    }

    fn refresh_voice(&mut self, slot: usize) {
        let master = self.channels[usize::from(self.zone.master_channel - 1)];
        let voice = &mut self.voices[slot];
        let member_range = self.channels[usize::from(voice.channel - 1)].pitch_range_semitones;
        voice.expression = VoiceExpression {
            pitch_bend_semitones: voice.member_pitch_bend * member_range
                + master.pitch_bend * master.pitch_range_semitones,
            pressure: (voice.member_pressure + master.pressure).clamp(0.0, 1.0),
            timbre: (voice.member_timbre + master.timbre).clamp(0.0, 1.0),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(kind: MidiEventKind) -> MidiEvent {
        MidiEvent {
            frame_offset: 0,
            kind,
        }
    }

    fn note(channel: u8, note: u8, velocity: f64, is_on: bool) -> MidiEvent {
        event(MidiEventKind::Note(NoteEvent {
            channel,
            note,
            velocity,
            is_on,
        }))
    }

    #[test]
    fn note_on_samples_expression_sent_before_the_note() {
        let mut manager = MpeVoiceManager::new(ZoneConfig::lower(15), 4);
        manager.handle_event(
            &event(MidiEventKind::PitchBend {
                channel: 2,
                value: 0.5,
            }),
            0,
        );
        manager.handle_event(
            &event(MidiEventKind::ChannelPressure {
                channel: 2,
                value: 0.75,
            }),
            0,
        );
        let update = manager.handle_event(&note(2, 60, 0.8, true), 10);
        let voice = manager.voices()[update.started_slot.unwrap()];
        assert_eq!(voice.expression.pitch_bend_semitones, 24.0);
        assert_eq!(voice.expression.pressure, 0.75);
    }

    #[test]
    fn released_voice_tracks_channel_until_it_is_reassigned() {
        let mut manager = MpeVoiceManager::new(ZoneConfig::lower(15), 4);
        let first = manager
            .handle_event(&note(2, 60, 1.0, true), 0)
            .started_slot
            .unwrap();
        manager.handle_event(&note(2, 60, 0.4, false), 100);
        manager.handle_event(
            &event(MidiEventKind::ChannelPressure {
                channel: 2,
                value: 0.6,
            }),
            110,
        );
        assert_eq!(manager.voices()[first].expression.pressure, 0.6);

        let second = manager
            .handle_event(&note(2, 64, 1.0, true), 120)
            .started_slot
            .unwrap();
        manager.handle_event(
            &event(MidiEventKind::ChannelPressure {
                channel: 2,
                value: 0.9,
            }),
            130,
        );
        assert!(!manager.voices()[first].bound_to_channel);
        assert_eq!(manager.voices()[first].expression.pressure, 0.6);
        assert_eq!(manager.voices()[second].expression.pressure, 0.9);
    }

    #[test]
    fn master_and_member_pitch_ranges_are_combined() {
        let mut manager = MpeVoiceManager::new(ZoneConfig::lower(15), 4);
        manager.handle_event(
            &event(MidiEventKind::PitchBend {
                channel: 2,
                value: 0.5,
            }),
            0,
        );
        manager.handle_event(
            &event(MidiEventKind::PitchBend {
                channel: 1,
                value: 0.5,
            }),
            0,
        );
        let slot = manager
            .handle_event(&note(2, 60, 1.0, true), 1)
            .started_slot
            .unwrap();
        assert_eq!(manager.voices()[slot].expression.pitch_bend_semitones, 25.0);
    }

    #[test]
    fn rpn_zero_updates_member_pitch_bend_range() {
        let mut manager = MpeVoiceManager::new(ZoneConfig::lower(15), 2);
        for (cc, raw) in [(101, 0), (100, 0), (6, 24)] {
            manager.handle_event(
                &event(MidiEventKind::ControlChange {
                    channel: 2,
                    cc,
                    value: f64::from(raw) / 127.0,
                }),
                0,
            );
        }
        manager.handle_event(
            &event(MidiEventKind::PitchBend {
                channel: 2,
                value: 1.0,
            }),
            0,
        );
        let slot = manager
            .handle_event(&note(2, 60, 1.0, true), 1)
            .started_slot
            .unwrap();
        assert_eq!(manager.voices()[slot].expression.pitch_bend_semitones, 24.0);
    }

    #[test]
    fn poly_pressure_targets_one_note() {
        let mut manager = MpeVoiceManager::new(ZoneConfig::lower(15), 4);
        let first = manager
            .handle_event(&note(2, 60, 1.0, true), 0)
            .started_slot
            .unwrap();
        let second = manager
            .handle_event(&note(3, 64, 1.0, true), 0)
            .started_slot
            .unwrap();
        manager.handle_event(
            &event(MidiEventKind::PolyPressure {
                channel: 2,
                note: 60,
                value: 0.7,
            }),
            1,
        );
        assert_eq!(manager.voices()[first].expression.pressure, 0.7);
        assert_eq!(manager.voices()[second].expression.pressure, 0.0);
    }
}
