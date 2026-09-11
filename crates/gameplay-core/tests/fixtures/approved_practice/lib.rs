//! Shared, dependency-free 60 Hz movement and practice combat simulation.
//!
//! World units are browser pixels: X right, Y down; rectangles/player positions
//! are top-left. Velocity is pixels/second, acceleration pixels/second². All
//! timers count simulation ticks. f64 retains the browser's number precision;
//! same-machine replay tests are NOT proof of cross-platform determinism.
//!
//! Callers supply one command for the current tick, with edges consumed once.
//! No clocks, rendering, files or device input belong here. Arena geometry and
//! initial states must be finite, positive-sized and free of interior overlap.

mod collision;
mod combat;
pub use collision::{Contacts, move_body, supported};
pub use combat::*;

pub const SIMULATION_HZ: u32 = 60;
pub const DT: f64 = 1.0 / SIMULATION_HZ as f64;
pub const BODY_WIDTH: f64 = 36.0;
pub const BODY_HEIGHT: f64 = 68.0;
pub const MOVE_SPEED: f64 = 320.0;
pub const GRAVITY: f64 = 1500.0;
pub const JUMP_SPEED: f64 = 520.0;
pub const MAX_RISE_SPEED: f64 = 480.0;
pub const MAX_FALL_SPEED: f64 = 740.0;
pub const COYOTE_TICKS: u8 = 8;
pub const BUFFER_TICKS: u8 = 9;
pub const MAX_FUEL: f64 = 100.0;
pub const FUEL_DRAIN: f64 = 40.0 / 1.4;
pub const FUEL_REGEN: f64 = 30.0;
pub const FUEL_DELAY_TICKS: u8 = 24;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
impl Rect {
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

/// Solids include the boundaries. Render these same rectangles in the client.
#[derive(Clone, Copy, Debug)]
pub struct Arena {
    pub width: f64,
    pub height: f64,
    pub floor_y: f64,
    pub spawn: Vec2,
    pub solids: &'static [Rect],
}

pub const PRACTICE_ARENA: Arena = Arena {
    width: 2400.0,
    height: 1350.0,
    floor_y: 1220.0,
    spawn: Vec2 {
        x: 390.0,
        y: 1152.0,
    },
    // Four authored platforms, then floor, left/right walls and ceiling, exactly
    // as public/assets/arena.json + compileArena in the browser reference.
    solids: &[
        Rect::new(110.0, 960.0, 300.0, 42.0),
        Rect::new(570.0, 755.0, 350.0, 44.0),
        Rect::new(1170.0, 955.0, 330.0, 44.0),
        Rect::new(1730.0, 670.0, 370.0, 46.0),
        Rect::new(-2400.0, 1220.0, 7200.0, 2700.0),
        Rect::new(-2400.0, -1350.0, 2400.0, 4050.0),
        Rect::new(2400.0, -1350.0, 2400.0, 4050.0),
        Rect::new(-2400.0, -1350.0, 7200.0, 1350.0),
    ],
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MoveAxis {
    Left,
    #[default]
    Idle,
    Right,
}
impl MoveAxis {
    fn value(self) -> f64 {
        match self {
            Self::Left => -1.0,
            Self::Idle => 0.0,
            Self::Right => 1.0,
        }
    }
}

/// Separate jump/jet bindings: jump is an edge, jet has an edge and a hold.
/// A sub-tick jet tap must arrive as one pressed+held tick, then a release.
/// `release_input` cancels buffered intent (e.g. focus loss) before this tick.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct InputCommand {
    pub tick: u64,
    pub move_x: MoveAxis,
    pub jump_pressed: bool,
    pub jet_pressed: bool,
    pub jet_held: bool,
    pub reset: bool,
    pub release_input: bool,
    /// World-space pointer target; None disables firing and preserves facing.
    pub aim_at: Option<Vec2>,
    /// Both reference weapons repeat while held, at their tick cadence.
    pub fire_held: bool,
    pub reload_pressed: bool,
    pub select_weapon: Option<WeaponId>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Player {
    pub body: Rect,
    pub velocity: Vec2,
    pub grounded: bool,
    pub coyote_ticks: u8,
    pub jump_buffer_ticks: u8,
    pub fuel: f64,
    pub fuel_delay_ticks: u8,
    pub thrust_latched: bool,
    pub thrusting: bool,
}
impl Player {
    fn spawn(arena: &Arena) -> Self {
        let body = Rect::new(arena.spawn.x, arena.spawn.y, BODY_WIDTH, BODY_HEIGHT);
        let grounded = supported(&body, arena.solids);
        Self {
            body,
            velocity: Vec2::default(),
            grounded,
            coyote_ticks: if grounded { COYOTE_TICKS } else { 0 },
            jump_buffer_ticks: 0,
            fuel: MAX_FUEL,
            fuel_delay_ticks: 0,
            thrust_latched: false,
            thrusting: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct World {
    pub tick: u64,
    pub player: Player,
}
impl World {
    pub fn new(arena: &Arena) -> Self {
        Self {
            tick: 0,
            player: Player::spawn(arena),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StepEvents {
    pub jumped: bool,
    pub landed: bool,
    pub reset: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TickMismatch {
    pub expected: u64,
    pub received: u64,
}

fn approach(value: f64, target: f64, amount: f64) -> f64 {
    if value < target {
        (value + amount).min(target)
    } else {
        (value - amount).max(target)
    }
}
fn horizontal(player: &mut Player, axis: MoveAxis) {
    let acceleration = match (axis == MoveAxis::Idle, player.grounded) {
        (false, true) => 3800.0,
        (false, false) => 2300.0,
        (true, true) => 4200.0,
        (true, false) => 320.0,
    };
    player.velocity.x = approach(
        player.velocity.x,
        axis.value() * MOVE_SPEED,
        acceleration * DT,
    );
}
fn jump(p: &mut Player, events: &mut StepEvents, preserve_thrust: bool) {
    p.velocity.y = -JUMP_SPEED;
    p.grounded = false;
    p.coyote_ticks = 0;
    p.jump_buffer_ticks = 0;
    if !preserve_thrust {
        p.thrust_latched = false;
        p.thrusting = false;
    }
    events.jumped = true;
}

/// Predict the actual swept horizontal/vertical path, rather than proximity.
fn landing_within_buffer(p: &Player, axis: MoveAxis, arena: &Arena) -> bool {
    if p.velocity.y < 0.0 {
        return false;
    }
    let mut probe = *p;
    probe.grounded = false;
    for _ in 0..BUFFER_TICKS {
        horizontal(&mut probe, axis);
        probe.velocity.y = (probe.velocity.y + GRAVITY * DT).min(MAX_FALL_SPEED);
        let movement = Vec2 {
            x: probe.velocity.x * DT,
            y: probe.velocity.y * DT,
        };
        let contacts = move_body(&mut probe.body, movement, arena.solids);
        if contacts.grounded {
            return true;
        }
        if contacts.hit_x {
            probe.velocity.x = 0.0;
        }
        if contacts.hit_y {
            probe.velocity.y = 0.0;
        }
    }
    false
}

pub fn jet_acceleration(fuel: f64) -> f64 {
    3600.0 * (0.5 + 0.5 * (fuel / MAX_FUEL / 0.9).clamp(0.0, 1.0))
}

/// Advances exactly one tick. Rejected commands leave the world untouched.
/// Reset consumes its tick, restores spawn/fuel/timers, and ignores other input.
pub fn step(
    world: &mut World,
    input: InputCommand,
    arena: &Arena,
) -> Result<StepEvents, TickMismatch> {
    if input.tick != world.tick {
        return Err(TickMismatch {
            expected: world.tick,
            received: input.tick,
        });
    }
    let mut events = StepEvents::default();
    world.tick += 1;
    if input.reset {
        world.player = Player::spawn(arena);
        events.reset = true;
        return Ok(events);
    }
    let p = &mut world.player;
    let input = if input.release_input {
        p.thrust_latched = false;
        p.thrusting = false;
        p.jump_buffer_ticks = 0;
        InputCommand::default()
    } else {
        input
    };
    let was_grounded = p.grounded;
    p.coyote_ticks = if was_grounded {
        COYOTE_TICKS
    } else {
        p.coyote_ticks.saturating_sub(1)
    };
    p.fuel_delay_ticks = p.fuel_delay_ticks.saturating_sub(1);
    if !input.jet_held {
        p.thrust_latched = false;
    }
    if input.jump_pressed {
        if was_grounded || p.coyote_ticks > 0 {
            jump(p, &mut events, true);
        } else if landing_within_buffer(p, input.move_x, arena) {
            p.jump_buffer_ticks = BUFFER_TICKS;
        }
    }
    if input.jet_pressed && input.jet_held && p.fuel > 0.0 {
        p.thrust_latched = true;
    }
    horizontal(p, input.move_x);
    p.thrusting = p.thrust_latched && input.jet_held && p.fuel > 0.0;
    let mut thrust = 0.0;
    if p.thrusting {
        if p.grounded {
            p.coyote_ticks = 0;
        }
        let used = p.fuel.min(FUEL_DRAIN * DT);
        thrust = jet_acceleration(p.fuel - used / 2.0) * used / (FUEL_DRAIN * DT);
        p.fuel = (p.fuel - used).max(0.0);
        p.fuel_delay_ticks = FUEL_DELAY_TICKS;
        if p.fuel < 1e-8 {
            p.fuel = 0.0;
            p.thrust_latched = false;
            p.thrusting = false;
        }
    } else if p.fuel_delay_ticks == 0 {
        p.fuel = (p.fuel + FUEL_REGEN * DT).min(MAX_FUEL);
    }
    p.velocity.y = (p.velocity.y + (GRAVITY - thrust) * DT).min(MAX_FALL_SPEED);
    if thrust > 0.0 && !events.jumped {
        p.velocity.y = p.velocity.y.max(-MAX_RISE_SPEED);
    }
    let movement = Vec2 {
        x: p.velocity.x * DT,
        y: p.velocity.y * DT,
    };
    let contacts = move_body(&mut p.body, movement, arena.solids);
    if contacts.hit_x {
        p.velocity.x = 0.0;
    }
    if contacts.hit_y {
        p.velocity.y = 0.0;
    }
    p.grounded = contacts.grounded;
    if p.grounded {
        p.thrust_latched = false;
        p.thrusting = false;
        p.coyote_ticks = COYOTE_TICKS;
        events.landed = !was_grounded;
        if p.jump_buffer_ticks > 0 {
            jump(p, &mut events, false);
        }
    }
    p.jump_buffer_ticks = p.jump_buffer_ticks.saturating_sub(1);
    Ok(events)
}
