use super::*;
use std::collections::VecDeque;

#[derive(Debug)]
pub struct Prediction {
    pub welcome: Welcome,
    pub local: Option<Actor>,
    pub latest: Option<Snapshot>,
    pub next_sequence: u64,
    pub corrections: super::diagnostics::Samples,
    pub history_peak: usize,
    pub target_lead: u64,
    pub rescheduled_slots: u64,
    snapshot_age: f64,
    lower_lead_age: f64,
    network_timing: bool,
    history: VecDeque<NetInput>,
    snapshots: VecDeque<Snapshot>,
    shown_shots: [Option<(u64, u64)>; MAX_PLAYERS],
}
#[derive(Default, Debug)]
pub struct ReconcileResult {
    pub accepted: bool,
    pub life_changed: bool,
    pub shots: Vec<Shot>,
    pub changed: [bool; MAX_PLAYERS],
}
impl Prediction {
    pub fn new(welcome: Welcome) -> Self {
        Self {
            welcome,
            local: None,
            latest: None,
            next_sequence: 1,
            corrections: Default::default(),
            history_peak: 0,
            target_lead: INPUT_LEAD,
            rescheduled_slots: 0,
            snapshot_age: 0.,
            lower_lead_age: 0.,
            network_timing: false,
            history: VecDeque::new(),
            snapshots: VecDeque::new(),
            shown_shots: [None; MAX_PLAYERS],
        }
    }
    /// RTT includes both directions. The received snapshot clock is already one
    /// direction old; reserve RTT plus four ticks for jitter/send cadence. No
    /// wall-clock synchronization is required. Grow promptly, reduce one tick per
    /// ten stable seconds. Sequence gaps are neutral, never simulated in a burst.
    pub fn observe_timing(&mut self, rtt: f64, elapsed: f64) {
        if !rtt.is_finite() || !elapsed.is_finite() || rtt < 0. || elapsed < 0. {
            return;
        }
        self.network_timing = true;
        self.snapshot_age = (self.snapshot_age + elapsed).min(1.);
        let desired = ((rtt * 60.).ceil() as u64)
            .saturating_add(4)
            .clamp(INPUT_LEAD, 24);
        if desired >= self.target_lead {
            self.target_lead = desired;
            self.lower_lead_age = 0.;
        } else {
            self.lower_lead_age += elapsed;
            if self.lower_lead_age >= 10. {
                self.target_lead -= 1;
                self.lower_lead_age = 0.;
            }
        }
    }
    pub fn history_len(&self) -> usize {
        self.history.len()
    }
    pub fn snapshot_len(&self) -> usize {
        self.snapshots.len()
    }
    pub fn clear_intent(&mut self) {
        self.history.clear();
        if let Some(actor) = &mut self.local {
            actor.require_neutral = true;
        }
    }
    pub fn command(&mut self, mut command: InputCommand) -> Option<[Option<NetInput>; 3]> {
        let actor = self.local.as_mut()?;
        if self.history.len() >= HISTORY_LIMIT {
            return None;
        }
        command.tick = self.next_sequence;
        command.reset = false;
        let input = NetInput {
            actor: self.welcome.actor,
            sequence: self.next_sequence,
            command,
        };
        if !valid_input(&input) {
            return None;
        }
        if self.network_timing
            && let Some(latest) = self.latest
        {
            let ceiling =
                latest.ack + self.target_lead + (self.snapshot_age * 60.).floor() as u64 + 2;
            if self.next_sequence > ceiling {
                return None;
            }
        }
        // Avoid predicting beyond the server's documented acceptance window.
        let tick = self
            .welcome
            .start_tick
            .checked_add(self.next_sequence - 1)?;
        if let Some(latest) = self.latest
            && tick >= latest.state.tick.saturating_add(INPUT_WINDOW - 1)
        {
            return None;
        }
        predict_movement(actor, command, &PRACTICE_ARENA);
        self.history.push_back(input);
        self.history_peak = self.history_peak.max(self.history.len());
        self.next_sequence += 1;
        let mut bundle = [None; 3];
        for (slot, input) in bundle.iter_mut().zip(self.history.iter().rev()) {
            *slot = Some(*input);
        }
        Some(bundle)
    }
    pub fn reconcile(&mut self, snapshot: Snapshot) -> ReconcileResult {
        let mut result = ReconcileResult::default();
        let expected_ack = snapshot.state.tick.saturating_sub(self.welcome.start_tick);
        if snapshot.ack != expected_ack
            || snapshot.last_applied > snapshot.ack
            || self
                .latest
                .is_some_and(|old| snapshot.state.tick <= old.state.tick || snapshot.ack < old.ack)
        {
            return result;
        }
        let Some(mut actor) = snapshot.state.actors[self.welcome.actor.index()] else {
            return result;
        };
        if actor.generation != self.welcome.generation {
            return result;
        }
        result.life_changed = self.latest.is_some_and(|old| {
            old.state.actors[actor.id.index()].is_some_and(|prev| {
                prev.deaths != actor.deaths || prev.combat.alive() != actor.combat.alive()
            })
        });
        if result.life_changed {
            self.history.clear();
        }
        self.history.retain(|input| input.sequence > snapshot.ack);
        self.snapshot_age = 0.;
        let reserve = if self.network_timing {
            self.target_lead
        } else {
            INPUT_LEAD
        };
        let behind = if self.network_timing {
            self.next_sequence + 2 < snapshot.ack + reserve
        } else {
            snapshot.ack > 0 && self.next_sequence <= snapshot.ack + 1
        };
        if behind {
            let next = snapshot.ack + reserve;
            self.rescheduled_slots += next.saturating_sub(self.next_sequence);
            self.next_sequence = next;
        }
        let end = self
            .history
            .back()
            .map_or(snapshot.ack, |input| input.sequence);
        if end.saturating_sub(snapshot.ack) > INPUT_WINDOW {
            self.history.clear();
        } else {
            for sequence in snapshot.ack + 1..=end {
                let command = self.history.iter().find(|i| i.sequence == sequence).map_or(
                    InputCommand {
                        tick: sequence,
                        release_input: true,
                        ..Default::default()
                    },
                    |i| i.command,
                );
                predict_movement(&mut actor, command, &PRACTICE_ARENA);
            }
        }
        for (i, confirmed) in snapshot.shots.iter().enumerate() {
            let generation = snapshot.state.actors[i].map(|a| a.generation);
            if self.latest.is_some_and(|old| {
                old.state.actors[i].map(|a| (a.generation, a.deaths, a.combat.alive()))
                    != snapshot.state.actors[i].map(|a| (a.generation, a.deaths, a.combat.alive()))
            }) {
                result.changed[i] = true;
            }
            if let Some(confirmed) = confirmed
                && let Some(generation) = generation
                && self.shown_shots[i]
                    .is_none_or(|(g, old)| g != generation || confirmed.tick > old)
            {
                self.shown_shots[i] = Some((generation, confirmed.tick));
                if self.latest.is_some() && snapshot.state.tick.saturating_sub(confirmed.tick) <= 12
                {
                    result.shots.push(confirmed.shot);
                }
            }
        }
        if let Some(previous) = self.local
            && !result.life_changed
        {
            self.corrections.add(
                (previous.movement.body.x - actor.movement.body.x)
                    .hypot(previous.movement.body.y - actor.movement.body.y),
            );
        }
        self.local = Some(actor);
        self.latest = Some(snapshot);
        self.snapshots.push_back(snapshot);
        while self.snapshots.len() > SNAPSHOT_LIMIT {
            self.snapshots.pop_front();
        }
        result.accepted = true;
        result
    }
    /// Delayed remote presentation, no extrapolation through loss or lifecycle.
    /// Call with estimated server tick minus six ticks (100 ms).
    pub fn remote_at(&self, id: ActorId, tick: f64) -> Option<Actor> {
        if !tick.is_finite() {
            return None;
        }
        let i = id.index();
        if id == self.welcome.actor {
            return None;
        }
        // Departures and new joins take effect immediately; never resurrect an old slot.
        let latest = self.latest?.state.actors[i]?;
        let first = self
            .snapshots
            .iter()
            .find(|s| s.state.actors[i].is_some_and(|a| a.generation == latest.generation))?;
        let mut before = first.state.actors[i]?;
        let mut before_tick = first.state.tick as f64;
        for sample in &self.snapshots {
            let Some(after) = sample.state.actors[i] else {
                continue;
            };
            if after.generation != latest.generation {
                continue;
            }
            let after_tick = sample.state.tick as f64;
            if after_tick >= tick {
                if before.generation != after.generation
                    || before.deaths != after.deaths
                    || before.combat.alive() != after.combat.alive()
                {
                    return Some(after);
                }
                let alpha = if after_tick > before_tick {
                    ((tick - before_tick) / (after_tick - before_tick)).clamp(0., 1.)
                } else {
                    1.
                };
                let mut actor = after;
                actor.movement.body.x = before.movement.body.x
                    + (after.movement.body.x - before.movement.body.x) * alpha;
                actor.movement.body.y = before.movement.body.y
                    + (after.movement.body.y - before.movement.body.y) * alpha;
                return Some(actor);
            }
            before = after;
            before_tick = after_tick;
        }
        Some(latest)
    }
}
