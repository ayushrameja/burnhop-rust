//! Opt-in rendered smoke route (`--movement-playtest`). It feeds the same input
//! buffer as the keyboard. This is scripted evidence, not a human feel test.
use crate::adapter::{InputBuffer, Key};
use burnhop_gameplay_core::*;

#[derive(Default)]
pub struct Route {
    pub saw_ceiling: bool,
    pub saw_empty: bool,
    pub complete: bool,
}
impl Route {
    pub fn feed(&self, tick: u64, input: &mut InputBuffer) {
        let events: &[(Key, bool)] = match tick {
            0 | 160 | 570 | 1060 | 1235 => &[(Key::Reset, true)],
            2 | 280 | 561 | 1085 => &[(Key::JetLeft, true)],
            90 | 560 | 1145 => &[(Key::JetLeft, false)],
            162 | 572 | 1062 => &[(Key::Right, true)],
            272 | 1050 | 1077 => &[(Key::Right, false)],
            1146 => &[(Key::Left, true)],
            1166 => &[(Key::Left, false)],
            _ => &[],
        };
        for &(key, down) in events {
            input.push(key, down);
        }
    }
    pub fn observe(&mut self, world: &World) {
        let p = world.player;
        if (280..=490).contains(&world.tick) {
            self.saw_ceiling |= p.body.y.abs() < 1e-7;
            self.saw_empty |= p.fuel == 0.0;
        }
        let passed = match world.tick {
            60 => Some((
                "platform underside",
                (p.body.y - 1002.0).abs() < 1e-7 && p.velocity.y == 0.0,
            )),
            150 => Some((
                "floor landing",
                p.grounded && (p.body.y - 1152.0).abs() < 1e-7,
            )),
            490 => Some((
                "arena ceiling and fuel exhaustion",
                self.saw_ceiling && self.saw_empty,
            )),
            550 => Some((
                "held jet stays off after regeneration",
                p.fuel > 0.0 && !p.thrusting,
            )),
            565 => Some(("fresh jet press reactivates", p.thrusting)),
            1000 => Some((
                "right arena wall",
                (p.body.x - 2364.0).abs() < 1e-7 && p.velocity.x == 0.0,
            )),
            1230 => Some((
                "landing on first platform",
                p.grounded && (p.body.y - 892.0).abs() < 1e-7,
            )),
            1236 => Some((
                "reset restores spawn and fuel",
                p == World::new(&PRACTICE_ARENA).player,
            )),
            _ => None,
        };
        if let Some((name, passed)) = passed {
            println!(
                "PLAYTEST tick={} {name}: {} (x={:.2}, y={:.2}, fuel={:.2})",
                world.tick,
                if passed { "PASS" } else { "FAIL" },
                p.body.x,
                p.body.y,
                p.fuel
            );
            assert!(passed, "rendered movement route failed: {name}");
        }
        if world.tick == 1238 {
            self.complete = true;
            println!("PLAYTEST COMPLETE: 8 checkpoints passed; keyboard control is now available.");
        }
    }
}

#[test]
fn rendered_route_has_valid_checkpoints() {
    let mut route = Route::default();
    let mut input = InputBuffer::default();
    let mut world = World::new(&PRACTICE_ARENA);
    while !route.complete {
        route.feed(world.tick, &mut input);
        let command = input.command(world.tick);
        step(&mut world, command, &PRACTICE_ARENA).unwrap();
        route.observe(&world);
    }
}

/// Combat input route: no state teleporting or fixture damage. Real ticks run in
/// the native renderer; aim/held buttons here are scripted, not device input.
#[derive(Default)]
pub struct CombatRoute {
    pub complete: bool,
    pub checkpoints: u8,
}
impl CombatRoute {
    pub fn feed(&self, tick: u64, input: &mut InputBuffer) {
        input.aim_at = Some(body_center(Rect::new(BOT_SPAWN.x, BOT_SPAWN.y, 36.0, 68.0)));
        let keys: &[(Key, bool)] = match tick {
            0 | 1300 => &[(Key::Reset, true)],
            4 | 300 => &[(Key::Fire, true)],
            104 | 335 => &[(Key::Fire, false)],
            110 | 340 => &[(Key::Reload, true)],
            111 | 341 => &[(Key::Reload, false)],
            280 => &[(Key::Rifle, true)],
            281 => &[(Key::Rifle, false)],
            305 => &[(Key::Right, true)],
            336 => &[(Key::Right, false)],
            _ => &[],
        };
        for &(key, down) in keys {
            input.push(key, down);
        }
    }
    pub fn observe(&mut self, world: &World, combat: &CombatState) {
        let check = match world.tick {
            100 => Some((
                "pistol hold killed bot",
                combat.kills == 1 && !combat.bot.alive(),
            )),
            183 => Some((
                "pistol reload completed",
                combat.player.weapons[0].ammo == 12
                    && combat.player.weapons[0].reserve == Reserve::Rounds(39),
            )),
            270 => Some((
                "bot respawn",
                combat.bot.alive() && combat.bot.health == 100,
            )),
            335 => Some((
                "M416 hold killed bot while moving",
                combat.kills == 2
                    && !combat.bot.alive()
                    && world.player.body.x > PRACTICE_ARENA.spawn.x,
            )),
            455 => Some((
                "M416 reload completed",
                combat.player.weapon().ammo == 30 && combat.player.weapon().reload_ticks == 0,
            )),
            // Closer distance after walking means stronger pistol damage; observe
            // the lifecycle by the cumulative counter instead of assuming a kill tick.
            1290 => Some((
                "bot killed player and player respawned",
                combat.deaths >= 1
                    && combat.player.alive()
                    && world.player.body == World::new(&PRACTICE_ARENA).player.body,
            )),
            1301 => Some((
                "F5 restores entire practice state",
                combat.kills == 0
                    && combat.deaths == 0
                    && combat.player == Combatant::new(false)
                    && combat.bot == Combatant::new(true)
                    && world.player == World::new(&PRACTICE_ARENA).player,
            )),
            _ => None,
        };
        if let Some((name, passed)) = check {
            println!(
                "COMBAT PLAYTEST tick={} {name}: {}",
                world.tick,
                if passed { "PASS" } else { "FAIL" }
            );
            assert!(passed, "rendered combat route failed: {name}");
            self.checkpoints += 1;
        }
        if world.tick == 1303 {
            self.complete = true;
            println!(
                "COMBAT PLAYTEST COMPLETE: {} checkpoints; device controls available.",
                self.checkpoints
            );
        }
    }
}
#[test]
fn rendered_combat_route_completes_through_the_real_input_buffer() {
    let mut route = CombatRoute::default();
    let mut input = InputBuffer::default();
    let mut world = World::new(&PRACTICE_ARENA);
    let mut combat = CombatState::default();
    while !route.complete {
        route.feed(world.tick, &mut input);
        let command = input.command(world.tick);
        let events = step_practice(&mut world, &mut combat, command, &PRACTICE_ARENA).unwrap();
        if events.player_died || events.player_respawned || events.movement.reset {
            input.clear();
        }
        route.observe(&world, &combat);
    }
    assert_eq!(route.checkpoints, 7);
}
