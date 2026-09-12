//! Bounded wire format, input scheduling and client prediction. No renderer.
mod codec;
pub mod prediction;
pub mod transport;
use burnhop_gameplay_core::*;
pub use codec::{decode, encode};
use std::collections::BTreeMap;

pub const PROTOCOL_VERSION: u16 = 2;
/// Bump whenever arena, tuning, ordering or required snapshot state changes.
pub const GAMEPLAY_VERSION: u64 = 0x4255_524e_0008_0001;
/// Stable envelope ID allows application compatibility errors to be displayed.
pub const TRANSPORT_ID: u64 = 0x4255_524e_484f_5001;
pub const MAX_MESSAGE_BYTES: usize = 1200;
pub const MAX_SNAPSHOT_BYTES: usize = 1185;
pub const INPUT_LEAD: u64 = 6;
pub const INPUT_WINDOW: u64 = 32;
pub const HISTORY_LIMIT: usize = 128;
pub const SNAPSHOT_LIMIT: usize = 32;
pub mod diagnostics;
pub const CONTROL: u8 = 0;
pub const STATE: u8 = 1;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NetInput {
    pub actor: ActorId,
    /// Local command tick starts at 1. Its server tick is start_tick + sequence - 1.
    pub sequence: u64,
    pub command: InputCommand,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Welcome {
    pub actor: ActorId,
    pub generation: u64,
    pub start_tick: u64,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConfirmedShot {
    pub tick: u64,
    pub shot: Shot,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Snapshot {
    /// State immediately BEFORE state.tick (all earlier ticks completed).
    pub state: MatchState,
    /// All slots through ack are retired, including missing commands.
    pub ack: u64,
    pub last_applied: u64,
    pub shots: [Option<ConfirmedShot>; MAX_PLAYERS],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rejection {
    Compatibility,
    Full,
    InvalidInput,
    RateLimit,
    Timeout,
}
#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(clippy::large_enum_variant)] // Fixed <2 KiB stack value avoids peer-driven allocations.
pub enum Message {
    Hello {
        protocol: u16,
        gameplay: u64,
    },
    Welcome(Welcome),
    Reject(Rejection),
    /// Latest command plus up to two previous commands for unreliable redundancy.
    Inputs([Option<NetInput>; 3]),
    /// Focus/death clearing: discard queued commands through this local sequence.
    Release {
        through: u64,
    },
    Snapshot(Snapshot),
}
impl Message {
    pub fn hello() -> Self {
        Self::Hello {
            protocol: PROTOCOL_VERSION,
            gameplay: GAMEPLAY_VERSION,
        }
    }
}
pub fn compatible(protocol: u16, gameplay: u64) -> bool {
    protocol == PROTOCOL_VERSION && gameplay == GAMEPLAY_VERSION
}
pub fn valid_input(input: &NetInput) -> bool {
    let c = input.command;
    input.sequence > 0
        && c.tick == input.sequence
        && !c.reset
        && (!c.jet_pressed || c.jet_held)
        && c.aim_at.is_none_or(|p| {
            p.x.is_finite() && p.y.is_finite() && p.x.abs() <= 100_000.0 && p.y.abs() <= 100_000.0
        })
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Admission {
    Accepted,
    DuplicateOrStale,
    WrongActor,
    Invalid,
    TooFarAhead,
    RateLimited,
}
/// One bounded queue per connection. Its actor identity comes from the server.
#[derive(Debug)]
pub struct InputQueue {
    pub actor: ActorId,
    pub start_tick: u64,
    pub ack: u64,
    pub last_applied: u64,
    pub stats: InputStats,
    retired_by_release: u64,
    commands: BTreeMap<u64, InputCommand>,
    tokens: f64,
}
impl InputQueue {
    pub fn new(actor: ActorId, start_tick: u64) -> Self {
        Self {
            actor,
            start_tick,
            ack: 0,
            last_applied: 0,
            stats: InputStats::default(),
            retired_by_release: 0,
            commands: BTreeMap::new(),
            tokens: 12.0,
        }
    }
    pub fn len(&self) -> usize {
        self.commands.len()
    }
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
    /// Wall-time allowance, capped burst; stepping catch-up ticks does not refill it.
    pub fn refill(&mut self, seconds: f64) {
        if seconds.is_finite() {
            self.tokens = (self.tokens + seconds.clamp(0.0, 1.0) * 90.0).min(12.0);
        }
    }
    pub fn admit(&mut self, input: NetInput, server_tick: u64) -> Admission {
        if input.actor != self.actor {
            return Admission::WrongActor;
        }
        if !valid_input(&input) {
            return Admission::Invalid;
        }
        let retired = server_tick.saturating_sub(self.start_tick);
        if input.sequence <= self.ack.max(retired).max(self.retired_by_release) {
            self.stats.late += 1;
            return Admission::DuplicateOrStale;
        }
        if self.commands.contains_key(&input.sequence) {
            self.stats.duplicates += 1;
            return Admission::DuplicateOrStale;
        }
        let Some(tick) = self.start_tick.checked_add(input.sequence - 1) else {
            return Admission::TooFarAhead;
        };
        if tick >= server_tick.saturating_add(INPUT_WINDOW) {
            return Admission::TooFarAhead;
        }
        if self.tokens < 1.0 {
            return Admission::RateLimited;
        }
        self.tokens -= 1.0;
        self.commands.insert(input.sequence, input.command);
        self.stats.accepted += 1;
        self.stats.peak = self.stats.peak.max(self.commands.len());
        Admission::Accepted
    }
    pub fn release(&mut self, through: u64, server_tick: u64) -> bool {
        if through
            > server_tick
                .saturating_sub(self.start_tick)
                .saturating_add(INPUT_WINDOW)
        {
            return false;
        }
        self.retired_by_release = self.retired_by_release.max(through);
        self.commands.retain(|seq, _| *seq > through);
        true
    }
    /// A missing slot is a neutral release immediately. Never repeat held fire/jet.
    /// The next slot advances even if this one was absent; late arrival is dropped.
    pub fn take_tick(&mut self, tick: u64) -> InputCommand {
        let mut command = InputCommand {
            tick,
            release_input: true,
            ..Default::default()
        };
        if tick >= self.start_tick {
            self.ack = tick - self.start_tick + 1;
            if let Some(input) = self.commands.remove(&self.ack) {
                command = InputCommand { tick, ..input };
                self.last_applied = self.ack;
                self.stats.applied += 1;
            } else {
                self.stats.missing += 1;
            }
            self.commands.retain(|seq, _| *seq > self.ack);
        }
        command
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct InputStats {
    pub accepted: u64,
    pub applied: u64,
    pub missing: u64,
    /// Includes stale redundant copies; not a count of lost unique actions.
    pub late: u64,
    pub duplicates: u64,
    pub peak: usize,
}
