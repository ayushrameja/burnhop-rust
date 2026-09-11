//! Presentation timing and device-edge buffering. No Bevy required by these
//! helpers, but they belong in the client: a server has its own time policy.
use burnhop_gameplay_core::{DT, InputCommand, MoveAxis, Vec2, WeaponId};
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Left,
    Right,
    Jump,
    JetLeft,
    JetRight,
    Reset,
    Fire,
    Reload,
    Pistol,
    Rifle,
}
impl Key {
    fn index(self) -> usize {
        self as usize
    }
}

#[derive(Default)]
pub struct InputBuffer {
    observed: [bool; 10],
    held: [bool; 10],
    pending: VecDeque<(Key, bool)>,
    release: bool,
    pub aim_at: Option<Vec2>,
}
impl InputBuffer {
    pub fn push(&mut self, key: Key, down: bool) {
        if self.observed[key.index()] == down {
            return;
        }
        if self.pending.len() >= 128 {
            self.clear();
            return;
        }
        self.observed[key.index()] = down;
        self.pending.push_back((key, down));
    }
    pub fn clear_fire(&mut self) {
        self.observed[Key::Fire.index()] = false;
        self.held[Key::Fire.index()] = false;
        self.pending.retain(|(key, _)| *key != Key::Fire);
        self.aim_at = None;
    }
    pub fn clear(&mut self) {
        self.aim_at = None;
        self.observed = [false; 10];
        self.held = [false; 10];
        self.pending.clear();
        self.release = true;
    }
    pub fn command(&mut self, tick: u64) -> InputCommand {
        let mut command = InputCommand {
            tick,
            ..Default::default()
        };
        if std::mem::take(&mut self.release) {
            command.release_input = true;
            return command;
        }
        let mut pressed = [false; 10];
        while let Some(&(key, down)) = self.pending.front() {
            let index = key.index();
            // Quantize a tap to at least one simulation tick. Keep its release
            // ordered for the next tick; never repeat an edge during catch-up.
            if !down && pressed[index] {
                break;
            }
            self.pending.pop_front();
            let was_jet = self.held[Key::JetLeft.index()] || self.held[Key::JetRight.index()];
            self.held[index] = down;
            pressed[index] |= down;
            command.reload_pressed |= key == Key::Reload && down;
            if down {
                match key {
                    Key::Pistol => command.select_weapon = Some(WeaponId::Pistol),
                    Key::Rifle => command.select_weapon = Some(WeaponId::M416),
                    _ => {}
                }
            }
            command.jump_pressed |= key == Key::Jump && down;
            command.jet_pressed |= matches!(key, Key::JetLeft | Key::JetRight) && down && !was_jet;
            if key == Key::Reset && down {
                self.clear();
                command.reset = true;
                return command;
            }
        }
        command.move_x = match (self.held[0], self.held[1]) {
            (true, false) => MoveAxis::Left,
            (false, true) => MoveAxis::Right,
            _ => MoveAxis::Idle,
        };
        command.jet_held = self.held[3] || self.held[4];
        command.fire_held = self.held[Key::Fire.index()];
        command.aim_at = self.aim_at;
        command
    }
}

#[derive(Default)]
pub struct FrameClock {
    accumulator: f64,
}
impl FrameClock {
    /// At most 100 ms admitted, five ticks executed, excess whole ticks dropped.
    /// This is a local playground policy, not a future server overload policy.
    pub fn advance(&mut self, elapsed: f64) -> (usize, f64) {
        let elapsed = if elapsed.is_finite() {
            elapsed.clamp(0.0, 0.1)
        } else {
            0.0
        };
        self.accumulator += elapsed;
        let mut ticks = 0;
        while self.accumulator + 1e-10 >= DT && ticks < 5 {
            self.accumulator = (self.accumulator - DT).max(0.0);
            ticks += 1;
        }
        if self.accumulator + 1e-10 >= DT {
            self.accumulator %= DT;
        }
        (ticks, (self.accumulator / DT).clamp(0.0, 1.0))
    }
    pub fn reset(&mut self) {
        self.accumulator = 0.0;
    }
}

/// Keeps the view inside the arena even on extreme aspect ratios; ordinary
/// 16:9 is 1280×720 world units. Resizing never changes simulation geometry.
pub fn view_size(width: f64, height: f64, arena_width: f64, arena_height: f64) -> (f64, f64) {
    let aspect = width.max(1.0) / height.max(1.0);
    let view_height = 720.0_f64.min(arena_height).min(arena_width / aspect);
    (view_height * aspect, view_height)
}

pub fn camera_axis(
    current: f64,
    target: f64,
    half_view: f64,
    extent: f64,
    rate: f64,
    lag: f64,
    dt: f64,
) -> f64 {
    let followed = current + (target - current) * (1.0 - (-rate * dt.clamp(0.0, 0.06)).exp());
    followed
        .clamp(target - lag, target + lag)
        .clamp(half_view, (extent - half_view).max(half_view))
}
