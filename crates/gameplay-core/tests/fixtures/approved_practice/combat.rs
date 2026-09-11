//! Practice combat rules. Instant rays are authoritative; tracers are cosmetic.
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
    reload_was_pressed: bool,
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ActorId {
    Player,
    Bot,
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
    target: (ActorId, Rect),
    arena: &Arena,
) -> Shot {
    actor.spend_shot();
    let origin = body_center(body);
    let (distance, impact) = nearest_hit(
        origin,
        actor.aim,
        actor.selected.tuning().range,
        id,
        arena.solids,
        &[target],
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
/// One authoritative practice tick; movement-only `step` stays available unchanged.
/// Movement, timers, both shot decisions, then damage. Simultaneous kills are valid.
/// No corpse collision/damage. Respawn ignores commands for that tick.
pub fn step_practice(
    world: &mut World,
    combat: &mut CombatState,
    mut input: InputCommand,
    arena: &Arena,
) -> Result<CombatEvents, TickMismatch> {
    if input.tick != world.tick {
        return Err(TickMismatch {
            expected: world.tick,
            received: input.tick,
        });
    }
    let mut events = CombatEvents::default();
    if input.reset {
        events.movement = step(world, input, arena)?;
        *combat = CombatState {
            require_neutral: true,
            ..Default::default()
        };
        return Ok(events);
    }
    events.player_respawned = combat.player.advance_life(false);
    events.bot_respawned = combat.bot.advance_life(true);
    if events.bot_respawned {
        combat.bot_body = Rect::new(BOT_SPAWN.x, BOT_SPAWN.y, 36.0, 68.0);
    }
    if events.player_respawned {
        world.player = World::new(arena).player;
        combat.require_neutral = true;
    }
    if events.player_respawned || events.bot_respawned {
        combat.bot_attack_ticks = BOT_GRACE_TICKS;
    }
    if input.release_input || combat.require_neutral {
        if combat.require_neutral && neutral(input) {
            combat.require_neutral = false;
        }
        input = InputCommand {
            tick: world.tick,
            release_input: true,
            ..Default::default()
        };
    }
    if events.player_respawned {
        input = InputCommand {
            tick: world.tick,
            release_input: true,
            ..Default::default()
        };
    }
    if combat.player.alive() && !events.player_respawned {
        events.movement = step(world, input, arena)?;
    } else {
        world.tick += 1;
        clear_motion(&mut world.player);
    }
    let player_shoots = combat.player.alive()
        && !events.player_respawned
        && combat.player.prepare(input, world.player.body);
    let mut bot_shoots = false;
    if combat.bot.alive() {
        if combat.player.alive() && !events.player_respawned && !events.bot_respawned {
            combat.bot_attack_ticks = combat.bot_attack_ticks.saturating_sub(1);
            let origin = body_center(combat.bot_body);
            let target = body_center(world.player.body);
            let visible = direction(origin, target).is_some_and(|aim| {
                matches!(
                    nearest_hit(
                        origin,
                        aim,
                        WeaponId::Pistol.tuning().range,
                        ActorId::Bot,
                        arena.solids,
                        &[(ActorId::Player, world.player.body)]
                    )
                    .1,
                    Impact::Body(ActorId::Player)
                )
            });
            let fire = combat.bot_attack_ticks == 0 && visible;
            bot_shoots = combat.bot.prepare(
                InputCommand {
                    aim_at: Some(target),
                    fire_held: fire,
                    reload_pressed: combat.bot.weapon().ammo == 0,
                    ..Default::default()
                },
                combat.bot_body,
            );
            if bot_shoots {
                combat.bot_attack_ticks = BOT_INTERVAL_TICKS;
            }
        } else {
            combat.bot.advance_weapons();
        }
    }
    // Build both rays against the same living snapshot before applying damage.
    if player_shoots {
        let target = if combat.bot.alive() {
            (ActorId::Bot, combat.bot_body)
        } else {
            (ActorId::Player, world.player.body)
        };
        events.shots[0] = Some(shot(
            &mut combat.player,
            world.player.body,
            ActorId::Player,
            target,
            arena,
        ));
    }
    if bot_shoots {
        events.shots[1] = Some(shot(
            &mut combat.bot,
            combat.bot_body,
            ActorId::Bot,
            (ActorId::Player, world.player.body),
            arena,
        ));
    }
    for event in events.shots.iter_mut().flatten() {
        match event.impact {
            Impact::Body(ActorId::Bot) => {
                event.damage = event.damage.min(combat.bot.health);
                combat.bot.damage(event.damage);
            }
            Impact::Body(ActorId::Player) => {
                event.damage = event.damage.min(combat.player.health);
                combat.player.damage(event.damage);
            }
            _ => {}
        }
    }
    events.player_died = events.shots[1].is_some() && !combat.player.alive();
    events.bot_died = events.shots[0].is_some()
        && !combat.bot.alive()
        && matches!(events.shots[0].unwrap().impact, Impact::Body(ActorId::Bot));
    if events.player_died {
        combat.deaths += 1;
        combat.require_neutral = true;
        clear_motion(&mut world.player);
    }
    if events.bot_died {
        combat.kills += 1;
    }
    Ok(events)
}
