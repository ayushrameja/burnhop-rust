use super::*;
use std::collections::VecDeque;

#[derive(Debug)]
pub struct Prediction {
    pub welcome: Welcome,
    pub local: Option<Actor>,
    pub latest: Option<Snapshot>,
    pub next_sequence: u64,
    history: VecDeque<NetInput>,
    snapshots: VecDeque<Snapshot>,
    shown_shots: [Option<u64>; 2],
}
#[derive(Default, Debug)]
pub struct ReconcileResult {
    pub accepted: bool,
    pub life_changed: bool,
    pub shots: Vec<Shot>,
}
impl Prediction {
    pub fn new(welcome: Welcome) -> Self {
        Self {
            welcome,
            local: None,
            latest: None,
            next_sequence: 1,
            history: VecDeque::new(),
            snapshots: VecDeque::new(),
            shown_shots: [None; 2],
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
        // After a stall/lost startup messages, reserve six future slots again.
        // Omitted sequence slots are neutral, never simulated faster to catch up.
        if snapshot.ack > 0 && self.next_sequence <= snapshot.ack + 1 {
            self.next_sequence = snapshot.ack + INPUT_LEAD;
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
            if let Some(confirmed) = confirmed
                && self.shown_shots[i].is_none_or(|old| confirmed.tick > old)
            {
                self.shown_shots[i] = Some(confirmed.tick);
                if self.latest.is_some() && snapshot.state.tick.saturating_sub(confirmed.tick) <= 12
                {
                    result.shots.push(confirmed.shot);
                }
            }
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
    pub fn remote_at(&self, tick: f64) -> Option<Actor> {
        if !tick.is_finite() {
            return None;
        }
        let i = self.welcome.actor.other().index();
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
