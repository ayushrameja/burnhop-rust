//! Shared actor combat rules and an offline practice adapter. Instant rays are authoritative; tracers are cosmetic.
use crate::{Arena, InputCommand, Player, Rect, StepEvents, TickMismatch, Vec2, World, step};

pub const MAX_HEALTH: u16 = 100;
pub const RESPAWN_TICKS: u16 = 180;
pub const EQUIP_TICKS: u16 = 18;
pub const BOT_GRACE_TICKS: u16 = 180;
pub const BOT_INTERVAL_TICKS: u16 = 60;
pub const BOT_SPAWN: Vec2 = Vec2 {
    x: 910.0,
    y: 1152.0,
};
const EPS: f64 = 1e-8;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WeaponId {
    #[default]
    Pistol,
    M416,
}
impl WeaponId {
    pub const fn index(self) -> usize {
        self as usize
    }
    pub const fn tuning(self) -> WeaponTuning {
        match self {
            Self::Pistol => WeaponTuning {
                name: "Pistol",
                magazine: 12,
                cooldown: 12,
                reload: 72,
                damage: 18,
                range: 1000.0,
                falloff_start: 300.0,
                falloff_end: 800.0,
                minimum_factor: 0.4,
                barrel: 17.0 * 68.0 / 85.94,
            },
            Self::M416 => WeaponTuning {
                name: "M416",
                magazine: 30,
                cooldown: 7,
                reload: 114,
                damage: 23,
                range: 1900.0,
                falloff_start: 800.0,
                falloff_end: 1600.0,
                minimum_factor: 0.8,
                barrel: 34.0 * 68.0 / 85.94,
            },
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WeaponTuning {
    pub name: &'static str,
    pub magazine: u16,
    pub cooldown: u16,
    pub reload: u16,
    pub damage: u16,
    pub range: f64,
    pub falloff_start: f64,
    pub falloff_end: f64,
    pub minimum_factor: f64,
    pub barrel: f64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reserve {
    Rounds(u16),
    Unlimited,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WeaponState {
    pub id: WeaponId,
    pub ammo: u16,
    pub reserve: Reserve,
    pub cooldown_ticks: u16,
    pub reload_ticks: u16,
}
impl WeaponState {
    fn new(id: WeaponId, reserve: Reserve) -> Self {
        Self {
            id,
            ammo: id.tuning().magazine,
            reserve,
            cooldown_ticks: 0,
            reload_ticks: 0,
        }
    }
    fn advance(&mut self) {
        self.cooldown_ticks = self.cooldown_ticks.saturating_sub(1);
        if self.reload_ticks > 0 {
            self.reload_ticks -= 1;
            if self.reload_ticks == 0 {
                let missing = self.id.tuning().magazine - self.ammo;
                let amount = match self.reserve {
                    Reserve::Unlimited => missing,
                    Reserve::Rounds(n) => n.min(missing),
                };
                self.ammo += amount;
                if let Reserve::Rounds(n) = &mut self.reserve {
                    *n -= amount;
                }
            }
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LifeState {
    Alive,
    Dead { remaining_ticks: u16 },
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Combatant {
    pub health: u16,
    pub life: LifeState,
    pub selected: WeaponId,
    pub weapons: [WeaponState; 2],
    pub equip_ticks: u16,
    /// Last valid normalized direction in X-right / Y-down coordinates.
    pub aim: Vec2,
    pub reload_was_pressed: bool,
}
impl Combatant {
    pub fn new(bot: bool) -> Self {
        Self {
            health: MAX_HEALTH,
            life: LifeState::Alive,
            selected: WeaponId::Pistol,
            weapons: [
                WeaponState::new(
                    WeaponId::Pistol,
                    if bot {
                        Reserve::Unlimited
                    } else {
                        Reserve::Rounds(48)
                    },
                ),
                WeaponState::new(WeaponId::M416, Reserve::Rounds(120)),
            ],
            equip_ticks: 0,
            aim: Vec2 {
                x: if bot { -1.0 } else { 1.0 },
                y: 0.0,
            },
            reload_was_pressed: false,
        }
    }
    pub fn alive(&self) -> bool {
        self.life == LifeState::Alive
    }
    pub fn weapon(&self) -> &WeaponState {
        &self.weapons[self.selected.index()]
    }
    fn advance_life(&mut self, bot: bool) -> bool {
        if let LifeState::Dead { remaining_ticks } = &mut self.life {
            *remaining_ticks = remaining_ticks.saturating_sub(1);
            if *remaining_ticks == 0 {
                *self = Self::new(bot);
                return true;
            }
        }
        false
    }
    fn advance_weapons(&mut self) {
        self.equip_ticks = self.equip_ticks.saturating_sub(1);
        for weapon in &mut self.weapons {
            weapon.advance();
        }
    }
    fn prepare(&mut self, input: InputCommand, body: Rect) -> bool {
        self.advance_weapons();
        if let Some(selected) = input.select_weapon
            && selected != self.selected
        {
            self.weapons[self.selected.index()].reload_ticks = 0;
            self.selected = selected;
            self.equip_ticks = self.equip_ticks.max(EQUIP_TICKS);
        }
        let valid_aim = input
            .aim_at
            .and_then(|point| direction(body_center(body), point));
        if let Some(aim) = valid_aim {
            self.aim = aim;
        }
        let reload_edge = input.reload_pressed && !self.reload_was_pressed;
        self.reload_was_pressed = input.reload_pressed;
        let weapon = &mut self.weapons[self.selected.index()];
        if reload_edge
            && self.equip_ticks == 0
            && weapon.reload_ticks == 0
            && weapon.ammo < weapon.id.tuning().magazine
            && weapon.reserve != Reserve::Rounds(0)
        {
            weapon.reload_ticks = weapon.id.tuning().reload;
        }
        input.fire_held
            && valid_aim.is_some()
            && self.equip_ticks == 0
            && weapon.reload_ticks == 0
            && weapon.cooldown_ticks == 0
            && weapon.ammo > 0
    }
    fn spend_shot(&mut self) {
        let weapon = &mut self.weapons[self.selected.index()];
        weapon.ammo -= 1;
        weapon.cooldown_ticks = weapon.id.tuning().cooldown;
    }
    fn damage(&mut self, amount: u16) {
        self.health = self.health.saturating_sub(amount);
        if self.health == 0 {
            self.life = LifeState::Dead {
                remaining_ticks: RESPAWN_TICKS,
            };
            self.reload_was_pressed = false;
            for weapon in &mut self.weapons {
                weapon.reload_ticks = 0;
            }
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CombatState {
    pub player: Combatant,
    pub bot: Combatant,
    pub bot_body: Rect,
    pub bot_attack_ticks: u16,
    /// Held commands are ignored after death/respawn/reset until a neutral tick.
    pub require_neutral: bool,
    pub kills: u32,
    pub deaths: u32,
}
impl Default for CombatState {
    fn default() -> Self {
        Self {
            player: Combatant::new(false),
            bot: Combatant::new(true),
            bot_body: Rect::new(BOT_SPAWN.x, BOT_SPAWN.y, 36.0, 68.0),
            bot_attack_ticks: BOT_GRACE_TICKS,
            require_neutral: false,
            kills: 0,
            deaths: 0,
        }
    }
}
use crate::{BODY_HEIGHT, BODY_WIDTH, PRACTICE_ARENA};
pub const MAX_PLAYERS: usize = 8;
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ActorId {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
}
impl ActorId {
    pub const ALL: [Self; MAX_PLAYERS] = [
        Self::One,
        Self::Two,
        Self::Three,
        Self::Four,
        Self::Five,
        Self::Six,
        Self::Seven,
        Self::Eight,
    ];
    pub const fn index(self) -> usize {
        self as usize
    }
    pub fn from_index(index: usize) -> Option<Self> {
        Self::ALL.get(index).copied()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Impact {
    Range,
    Terrain,
    Body(ActorId),
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shot {
    pub shooter: ActorId,
    pub weapon: WeaponId,
    pub origin: Vec2,
    pub end: Vec2,
    pub impact: Impact,
    pub damage: u16,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CombatEvents {
    pub movement: StepEvents,
    pub shots: [Option<Shot>; 2],
    pub player_respawned: bool,
    pub bot_respawned: bool,
    pub player_died: bool,
    pub bot_died: bool,
}
pub fn body_center(body: Rect) -> Vec2 {
    Vec2 {
        x: body.x + body.width / 2.0,
        y: body.y + body.height / 2.0,
    }
}
pub fn direction(from: Vec2, to: Vec2) -> Option<Vec2> {
    let x = to.x - from.x;
    let y = to.y - from.y;
    let length = x.hypot(y);
    (length.is_finite() && length > EPS).then(|| Vec2 {
        x: x / length,
        y: y / length,
    })
}
/// Whole-segment slab intersection. No projectile stepping, even over huge ranges.
/// Initial overlap/contact blocks at zero; terrain wins exact/near distance ties.
pub fn ray_rect(origin: Vec2, direction: Vec2, rect: Rect, range: f64) -> Option<f64> {
    let mut enter = 0.0_f64;
    let mut exit = range;
    for (o, d, low, high) in [
        (origin.x, direction.x, rect.x, rect.x + rect.width),
        (origin.y, direction.y, rect.y, rect.y + rect.height),
    ] {
        if d.abs() < EPS {
            if o < low || o > high {
                return None;
            }
        } else {
            let a = (low - o) / d;
            let b = (high - o) / d;
            enter = enter.max(a.min(b));
            exit = exit.min(a.max(b));
            if enter > exit {
                return None;
            }
        }
    }
    (exit >= 0.0 && enter <= range).then_some(enter)
}
/// Excludes the shooter. Equal body contacts use stable ActorId ordering.
pub fn nearest_hit(
    origin: Vec2,
    aim: Vec2,
    range: f64,
    shooter: ActorId,
    solids: &[Rect],
    bodies: &[(ActorId, Rect)],
) -> (f64, Impact) {
    let mut distance = range;
    let mut impact = Impact::Range;
    for &solid in solids {
        if let Some(hit) = ray_rect(origin, aim, solid, range)
            && hit <= distance
        {
            distance = hit;
            impact = Impact::Terrain;
        }
    }
    for &(id, body) in bodies {
        if id == shooter {
            continue;
        }
        if let Some(hit) = ray_rect(origin, aim, body, range) {
            let closer = hit < distance - EPS;
            let body_tie = matches!(impact, Impact::Body(other) if id < other)
                && (hit - distance).abs() <= EPS;
            if closer || body_tie {
                distance = hit;
                impact = Impact::Body(id);
            }
        }
    }
    (distance, impact)
}
pub fn body_damage(id: WeaponId, distance: f64) -> u16 {
    let t = id.tuning();
    if !distance.is_finite() || !(0.0..=t.range).contains(&distance) {
        return 0;
    }
    let falloff =
        ((distance - t.falloff_start) / (t.falloff_end - t.falloff_start)).clamp(0.0, 1.0);
    (f64::from(t.damage) * (1.0 - falloff * (1.0 - t.minimum_factor))).round() as u16
}
fn shot(
    actor: &mut Combatant,
    body: Rect,
    id: ActorId,
    targets: &[(ActorId, Rect)],
    arena: &Arena,
) -> Shot {
    actor.spend_shot();
    let origin = body_center(body);
    let (distance, impact) = crate::mixed::nearest_hit(
        origin,
        actor.aim,
        actor.selected.tuning().range,
        id,
        arena,
        targets,
    );
    Shot {
        shooter: id,
        weapon: actor.selected,
        origin,
        end: Vec2 {
            x: origin.x + actor.aim.x * distance,
            y: origin.y + actor.aim.y * distance,
        },
        impact,
        damage: if matches!(impact, Impact::Body(_)) {
            body_damage(actor.selected, distance)
        } else {
            0
        },
    }
}
fn neutral(input: InputCommand) -> bool {
    input.release_input
        || (input.move_x == crate::MoveAxis::Idle
            && !input.jump_pressed
            && !input.jet_pressed
            && !input.jet_held
            && !input.fire_held
            && !input.reload_pressed
            && input.select_weapon.is_none())
}
fn clear_motion(player: &mut Player) {
    player.velocity = Vec2::default();
    player.thrust_latched = false;
    player.thrusting = false;
    player.jump_buffer_ticks = 0;
}
/// Complete actor state. `generation` distinguishes a fresh connection in a reused slot.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Actor {
    pub id: ActorId,
    pub generation: u64,
    pub movement: Player,
    pub combat: Combatant,
    pub require_neutral: bool,
    pub kills: u32,
    pub deaths: u32,
}
impl Actor {
    pub fn new(id: ActorId, generation: u64, arena: &Arena) -> Self {
        Self {
            id,
            generation,
            movement: Player::spawn(&spawn_arena(id, arena)),
            combat: Combatant::new(false),
            require_neutral: true,
            kills: 0,
            deaths: 0,
        }
    }
}
/// Existing floor and platform tops; terrain is unchanged. Top-left body coordinates.
pub const SPAWN_CANDIDATES: [Vec2; MAX_PLAYERS] = [
    PRACTICE_ARENA.spawn,
    BOT_SPAWN,
    Vec2 { x: 90., y: 1152. },
    Vec2 { x: 2240., y: 1152. },
    Vec2 { x: 210., y: 892. },
    Vec2 { x: 700., y: 687. },
    Vec2 { x: 1300., y: 887. },
    Vec2 { x: 1900., y: 602. },
];
fn spawn_arena(id: ActorId, arena: &Arena) -> Arena {
    Arena {
        spawn: if id.index() == 0 {
            arena.spawn
        } else {
            if id == ActorId::Two {
                arena.bot_spawn
            } else {
                SPAWN_CANDIDATES[id.index()]
            }
        },
        ..*arena
    }
}
fn overlaps(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.width - EPS
        && a.x + a.width > b.x + EPS
        && a.y < b.y + b.height - EPS
        && a.y + a.height > b.y + EPS
}
pub fn valid_spawn(point: Vec2, arena: &Arena) -> bool {
    let body = Rect::new(point.x, point.y, BODY_WIDTH, BODY_HEIGHT);
    point.x.is_finite()
        && point.y.is_finite()
        && point.x >= 0.
        && point.y >= 0.
        && point.x + BODY_WIDTH <= arena.width
        && point.y + BODY_HEIGHT <= arena.height
        && !arena.solids.iter().any(|&solid| overlaps(body, solid))
        && crate::collision::supported(&body, arena.solids)
}
/// Prefer clear candidates, then maximize nearest living distance. Stable candidate
/// order breaks ties. All-contested fallback can overlap: there is no immunity.
pub fn select_spawn(state: &MatchState, id: ActorId, arena: &Arena) -> Vec2 {
    let mut best: Option<(Vec2, bool, f64)> = None;
    for point in SPAWN_CANDIDATES {
        if !valid_spawn(point, arena) {
            continue;
        }
        let body = Rect::new(point.x, point.y, BODY_WIDTH, BODY_HEIGHT);
        let mut clear = true;
        let mut distance = f64::INFINITY;
        for actor in state
            .actors
            .iter()
            .flatten()
            .filter(|a| a.id != id && a.combat.alive())
        {
            clear &= !overlaps(body, actor.movement.body);
            distance = distance
                .min((point.x - actor.movement.body.x).hypot(point.y - actor.movement.body.y));
        }
        if best.is_none_or(|(_, old_clear, old_distance)| {
            (clear && !old_clear) || (clear == old_clear && distance > old_distance)
        }) {
            best = Some((point, clear, distance));
        }
    }
    best.map_or(arena.spawn, |(point, _, _)| point)
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MatchState {
    pub tick: u64,
    pub actors: [Option<Actor>; MAX_PLAYERS],
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MatchEvents {
    pub movement: [StepEvents; MAX_PLAYERS],
    pub shots: [Option<Shot>; MAX_PLAYERS],
    pub respawned: [bool; MAX_PLAYERS],
    pub died: [bool; MAX_PLAYERS],
}
/// One 60 Hz tick: life transitions, all movement, all shot decisions, then damage.
/// Return fire is simultaneous; shooter-ID order breaks lethal-credit ties.
/// Online reset flags are ignored.
pub fn step_match(
    state: &mut MatchState,
    inputs: [InputCommand; MAX_PLAYERS],
    arena: &Arena,
) -> Result<MatchEvents, TickMismatch> {
    step_actors(state, inputs, arena, None, None)
}
/// Predict movement only. Life, weapon state, damage and respawns stay authoritative.
/// Complete movement state (including grace/buffer/fuel/latches) is replayed.
pub fn predict_movement(actor: &mut Actor, mut input: InputCommand, arena: &Arena) {
    input.reset = false;
    if actor.require_neutral || input.release_input {
        if neutral(input) {
            actor.require_neutral = false;
        }
        input = InputCommand {
            tick: input.tick,
            release_input: true,
            ..Default::default()
        };
    }
    if actor.combat.alive() {
        let mut world = World {
            tick: input.tick,
            player: actor.movement,
        };
        step(&mut world, input, arena).expect("prediction supplies its own tick");
        actor.movement = world.player;
    } else {
        clear_motion(&mut actor.movement);
    }
}
fn step_actors(
    state: &mut MatchState,
    mut inputs: [InputCommand; MAX_PLAYERS],
    arena: &Arena,
    mut bot_timer: Option<&mut u16>,
    mut stance: Option<&mut crate::offline::Stance>,
) -> Result<MatchEvents, TickMismatch> {
    for id in ActorId::ALL {
        if state.actors[id.index()].is_some() && inputs[id.index()].tick != state.tick {
            return Err(TickMismatch {
                expected: state.tick,
                received: inputs[id.index()].tick,
            });
        }
    }
    let mut events = MatchEvents::default();
    for id in ActorId::ALL {
        let i = id.index();
        let spawn = if bot_timer.is_some() {
            spawn_arena(id, arena).spawn
        } else {
            select_spawn(state, id, arena)
        };
        if let Some(actor) = &mut state.actors[i] {
            let bot = i == 1 && bot_timer.is_some();
            events.respawned[i] = actor.combat.advance_life(bot);
            if events.respawned[i] {
                actor.movement = Player::spawn(&Arena { spawn, ..*arena });
                if i == 0
                    && let Some(s) = stance.as_deref_mut()
                {
                    s.amount = 0.;
                }
                actor.require_neutral = true;
            }
            inputs[i].reset = false;
            if !bot {
                if actor.require_neutral || inputs[i].release_input {
                    if neutral(inputs[i]) {
                        actor.require_neutral = false;
                    }
                    inputs[i] = InputCommand {
                        tick: state.tick,
                        release_input: true,
                        ..Default::default()
                    };
                }
                if actor.combat.alive() && !events.respawned[i] {
                    let mut world = World {
                        tick: state.tick,
                        player: actor.movement,
                    };
                    if let Some(s) = stance.as_deref_mut() {
                        if world.player.body.y > arena.height {
                            s.recover(&mut world.player, arena);
                        } else {
                            s.update(&mut world.player, inputs[i], arena);
                            events.movement[i] = crate::step_scaled(
                                &mut world,
                                inputs[i],
                                arena,
                                1. - 0.5 * s.amount,
                            )?;
                            if world.player.body.y > arena.height {
                                s.recover(&mut world.player, arena);
                            }
                        }
                        if s.recovered {
                            inputs[i] = InputCommand {
                                tick: state.tick,
                                release_input: true,
                                ..Default::default()
                            };
                            actor.require_neutral = true;
                            events.movement[i] = StepEvents::default();
                        }
                    } else {
                        events.movement[i] = step(&mut world, inputs[i], arena)?;
                    }
                    actor.movement = world.player;
                } else {
                    clear_motion(&mut actor.movement);
                }
            }
        }
    }
    if events.respawned.iter().any(|value| *value)
        && let Some(timer) = bot_timer.as_deref_mut()
    {
        *timer = BOT_GRACE_TICKS;
    }
    let mut shoots = [false; MAX_PLAYERS];
    for id in ActorId::ALL {
        let i = id.index();
        let target = state.actors[0]; // Only the offline bot consumes this target.
        let Some(actor) = &mut state.actors[i] else {
            continue;
        };
        if !actor.combat.alive() {
            continue;
        }
        if i == 1
            && let Some(timer) = bot_timer.as_deref_mut()
        {
            if events.respawned.iter().any(|value| *value) {
                *timer = BOT_GRACE_TICKS;
            }
            if let Some(target) = target
                && target.combat.alive()
                && !events.respawned[0]
                && !events.respawned[1]
                && !stance.as_ref().is_some_and(|s| s.recovered)
            {
                *timer = timer.saturating_sub(1);
                let point = body_center(target.movement.body);
                let visible =
                    direction(body_center(actor.movement.body), point).is_some_and(|aim| {
                        matches!(
                            crate::mixed::nearest_hit(
                                body_center(actor.movement.body),
                                aim,
                                WeaponId::Pistol.tuning().range,
                                id,
                                arena,
                                &[(target.id, target.movement.body)]
                            )
                            .1,
                            Impact::Body(_)
                        )
                    });
                shoots[i] = actor.combat.prepare(
                    InputCommand {
                        aim_at: Some(point),
                        fire_held: *timer == 0 && visible,
                        reload_pressed: actor.combat.weapon().ammo == 0,
                        ..Default::default()
                    },
                    actor.movement.body,
                );
                if shoots[i] {
                    *timer = BOT_INTERVAL_TICKS;
                }
            } else {
                actor.combat.advance_weapons();
            }
        } else if !events.respawned[i] {
            shoots[i] = actor.combat.prepare(inputs[i], actor.movement.body);
        }
    }
    // Snapshot target bodies before damage; dead/absent targets cannot absorb rays.
    let mut bodies = [(ActorId::One, Rect::new(0., 0., 0., 0.)); MAX_PLAYERS];
    let mut count = 0;
    for actor in state.actors.iter().flatten().filter(|a| a.combat.alive()) {
        bodies[count] = (actor.id, actor.movement.body);
        count += 1;
    }
    for id in ActorId::ALL {
        let i = id.index();
        if !shoots[i] {
            continue;
        }
        let actor = state.actors[i].as_mut().expect("shooter exists");
        events.shots[i] = Some(shot(
            &mut actor.combat,
            actor.movement.body,
            id,
            &bodies[..count],
            arena,
        ));
    }
    for shot in events.shots.iter_mut().flatten() {
        if let Impact::Body(id) = shot.impact {
            let victim = state.actors[id.index()]
                .as_mut()
                .expect("ray target exists");
            shot.damage = shot.damage.min(victim.combat.health);
            victim.combat.damage(shot.damage);
            if !victim.combat.alive() && !events.died[id.index()] {
                events.died[id.index()] = true;
                victim.deaths += 1;
                victim.require_neutral = true;
                clear_motion(&mut victim.movement);
                state.actors[shot.shooter.index()]
                    .as_mut()
                    .expect("shooter exists")
                    .kills += 1;
            }
        }
    }
    if stance.as_ref().is_some_and(|s| s.recovered)
        && let Some(timer) = bot_timer
    {
        *timer = BOT_GRACE_TICKS;
    }
    state.tick += 1;
    Ok(events)
}
/// Offline compatibility adapter. Bot policy supplies commands to the same actor
/// rules as multiplayer; the approved stationary bot and reset behavior remain.
pub fn step_practice(
    world: &mut World,
    combat: &mut CombatState,
    input: InputCommand,
    arena: &Arena,
) -> Result<CombatEvents, TickMismatch> {
    step_practice_internal(world, combat, input, arena, None)
}
pub(crate) fn step_practice_internal(
    world: &mut World,
    combat: &mut CombatState,
    input: InputCommand,
    arena: &Arena,
    stance: Option<&mut crate::offline::Stance>,
) -> Result<CombatEvents, TickMismatch> {
    if input.tick != world.tick {
        return Err(TickMismatch {
            expected: world.tick,
            received: input.tick,
        });
    }
    if input.reset {
        let movement = step(world, input, arena)?;
        *combat = CombatState {
            bot_body: Rect::new(
                arena.bot_spawn.x,
                arena.bot_spawn.y,
                BODY_WIDTH,
                BODY_HEIGHT,
            ),
            require_neutral: true,
            ..Default::default()
        };
        return Ok(CombatEvents {
            movement,
            ..Default::default()
        });
    }
    let mut bot = Actor::new(ActorId::Two, 0, arena);
    bot.movement.body = combat.bot_body;
    bot.combat = combat.bot;
    bot.require_neutral = false;
    let mut state = MatchState {
        tick: world.tick,
        actors: [
            Some(Actor {
                id: ActorId::One,
                generation: 0,
                movement: world.player,
                combat: combat.player,
                require_neutral: combat.require_neutral,
                kills: combat.kills,
                deaths: combat.deaths,
            }),
            Some(bot),
            None,
            None,
            None,
            None,
            None,
            None,
        ],
    };
    let result = step_actors(
        &mut state,
        std::array::from_fn(|i| {
            if i == 0 {
                input
            } else {
                InputCommand {
                    tick: world.tick,
                    ..Default::default()
                }
            }
        }),
        arena,
        Some(&mut combat.bot_attack_ticks),
        stance,
    )?;
    let player = state.actors[0].unwrap();
    world.tick = state.tick;
    world.player = player.movement;
    combat.player = player.combat;
    combat.bot = state.actors[1].unwrap().combat;
    combat.bot_body = state.actors[1].unwrap().movement.body;
    combat.require_neutral = player.require_neutral;
    combat.kills = player.kills;
    combat.deaths = player.deaths;
    Ok(CombatEvents {
        movement: result.movement[0],
        shots: [result.shots[0], result.shots[1]],
        player_respawned: result.respawned[0],
        bot_respawned: result.respawned[1],
        player_died: result.died[0],
        bot_died: result.died[1],
    })
}
