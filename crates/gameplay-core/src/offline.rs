//! Complete offline-only stance/recovery state; never encoded in network snapshots.
use crate::*;
pub const CROUCH_HEIGHT: f64 = 54.20060507330696;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MapId {
    #[default]
    Range,
    EmberRelay,
}
impl MapId {
    pub fn arena(self) -> Arena {
        match self {
            Self::Range => PRACTICE_ARENA,
            Self::EmberRelay => ember::EMBER_RELAY,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct OfflineCommand {
    pub input: InputCommand,
    pub crouch_held: bool,
    pub jump_held: bool,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Stance {
    pub amount: f64,
    pub require_release: bool,
    pub(crate) crouch_held: bool,
    pub(crate) jump_held: bool,
    pub(crate) recovered: bool,
}
impl Stance {
    pub(crate) fn update(&mut self, p: &mut Player, input: InputCommand, arena: &Arena) {
        let crouch = p.grounded
            && self.crouch_held
            && !self.jump_held
            && !input.jump_pressed
            && !input.jet_held
            && p.jump_buffer_ticks == 0
            && !input.release_input;
        let amount = crate::approach(self.amount, if crouch { 1. } else { 0. }, DT / 0.18);
        let height = BODY_HEIGHT + (CROUCH_HEIGHT - BODY_HEIGHT) * amount;
        let b = Rect {
            y: p.body.y + p.body.height - height,
            height,
            ..p.body
        };
        if !mixed::overlaps(b, arena) {
            p.body = b;
            self.amount = amount;
        }
    }
    pub(crate) fn recover(&mut self, p: &mut Player, arena: &Arena) {
        let fuel = p.fuel;
        let delay = p.fuel_delay_ticks;
        *p = Player::spawn(arena);
        p.fuel = fuel;
        p.fuel_delay_ticks = delay;
        self.amount = 0.;
        self.require_release = true;
        self.recovered = true;
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OfflinePracticeState {
    pub map: MapId,
    pub world: World,
    pub combat: CombatState,
    pub stance: Stance,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct OfflineEvents {
    pub combat: CombatEvents,
    pub recovered: bool,
}
impl OfflinePracticeState {
    pub fn new(map: MapId) -> Result<Self, String> {
        let arena = map.arena();
        if map == MapId::EmberRelay {
            ember::validate(&arena, ember::SPAWNS)?;
        }
        Ok(Self {
            map,
            world: World::new(&arena),
            combat: CombatState {
                bot_body: Rect::new(
                    arena.bot_spawn.x,
                    arena.bot_spawn.y,
                    BODY_WIDTH,
                    BODY_HEIGHT,
                ),
                ..Default::default()
            },
            stance: Stance::default(),
        })
    }
    pub fn step(&mut self, command: OfflineCommand) -> Result<OfflineEvents, TickMismatch> {
        if command.input.tick != self.world.tick {
            return Err(TickMismatch {
                expected: self.world.tick,
                received: command.input.tick,
            });
        }
        if self.map == MapId::Range {
            return step_practice(
                &mut self.world,
                &mut self.combat,
                command.input,
                &PRACTICE_ARENA,
            )
            .map(|combat| OfflineEvents {
                combat,
                recovered: false,
            });
        }
        let arena = self.map.arena();
        let mut input = command.input;
        self.stance.recovered = false;
        let neutral = input.move_x == MoveAxis::Idle
            && !input.jump_pressed
            && !command.jump_held
            && !command.crouch_held
            && !input.jet_pressed
            && !input.jet_held
            && !input.fire_held
            && !input.reload_pressed
            && input.select_weapon.is_none()
            && !input.reset;
        if self.stance.require_release && !input.reset {
            if neutral {
                self.stance.require_release = false;
            }
            input = InputCommand {
                tick: input.tick,
                release_input: true,
                ..Default::default()
            };
        }
        self.stance.crouch_held = command.crouch_held && !input.release_input;
        self.stance.jump_held = command.jump_held;
        if input.reset {
            self.stance = Stance {
                require_release: true,
                ..Default::default()
            };
        }
        let combat = crate::combat::step_practice_internal(
            &mut self.world,
            &mut self.combat,
            input,
            &arena,
            Some(&mut self.stance),
        )?;
        if combat.player_died || combat.player_respawned {
            self.stance.require_release = true;
        }
        Ok(OfflineEvents {
            combat,
            recovered: self.stance.recovered,
        })
    }
}
