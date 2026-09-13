//! Explicit development route, never enabled by ordinary practice/CLI defaults.
use crate::{Playground, ember_routes as routes};
use burnhop_gameplay_core::{offline::*, *};
#[derive(Default)]
pub struct Review {
    pub index: usize,
    pub t: u16,
    pub settle: u16,
    pub complete: bool,
    pub capture_name: Option<&'static str>,
}
pub fn feed(game: &mut Playground) -> Option<OfflineCommand> {
    let mut r = game.ember_review.take()?;
    if r.complete {
        game.ember_review = Some(r);
        return None;
    }
    if r.t == 0 && r.settle == 0 {
        let mut s = if r.index < routes::STAGES.len() {
            routes::initial(routes::STAGES[r.index], 60.)
        } else {
            let mut s = OfflinePracticeState::new(MapId::EmberRelay).unwrap();
            s.world.player.body.x = if r.index == routes::STAGES.len() {
                1490.
            } else {
                1810.
            };
            s.world.player.body.y = 1100.;
            s.world.player.grounded = false;
            s.world.player.coyote_ticks = 0;
            s.world.player.fuel = 31.;
            s.combat.player.health = 57;
            s.combat.player.weapons[0].ammo = 7;
            s.combat.kills = 2;
            s
        };
        s.world.tick = game.world.tick;
        game.world = s.world;
        game.combat = s.combat;
        game.stance = s.stance;
        game.previous = s.world.player;
        game.snap_camera = true;
        game.release_gate = false;
        game.input.clear();
        let epoch = game.feedback.epoch.wrapping_add(1);
        game.feedback = Default::default();
        game.feedback.epoch = epoch;
        r.capture_name = None;
        println!(
            "EMBER REVIEW start index={} fixture={:?}",
            r.index, game.world.player
        );
    }
    let s = OfflinePracticeState {
        map: game.map,
        world: game.world,
        combat: game.combat,
        stance: game.stance,
    };
    let command = if r.settle > 0 || r.index >= routes::STAGES.len() {
        OfflineCommand {
            input: InputCommand {
                tick: game.world.tick,
                ..Default::default()
            },
            crouch_held: r.index < routes::STAGES.len() && routes::STAGES[r.index].crouch,
            ..Default::default()
        }
    } else {
        routes::command(routes::STAGES[r.index], &s, r.t)
    };
    game.ember_review = Some(r);
    Some(command)
}
pub fn observe(game: &mut Playground, recovered: bool) {
    let Some(mut r) = game.ember_review.take() else {
        return;
    };
    if !r.complete {
        let s = OfflinePracticeState {
            map: game.map,
            world: game.world,
            combat: game.combat,
            stance: game.stance,
        };
        assert!(
            !burnhop_gameplay_core::mixed::overlaps(game.world.player.body, &game.arena()),
            "native route penetrated terrain"
        );
        if r.settle > 0 {
            r.settle -= 1;
            if r.settle == 60 {
                r.capture_name = Some(if r.index < routes::STAGES.len() {
                    routes::STAGES[r.index].name
                } else if r.index == routes::STAGES.len() {
                    "void-one-recovered"
                } else {
                    "void-two-recovered"
                });
            }
            if r.settle == 0 {
                r.index += 1;
                r.t = 0;
                if r.index == routes::STAGES.len() + 2 {
                    r.complete = true;
                    println!(
                        "EMBER REVIEW complete: 26 traversals + 2 real falls; injected fixtures, not human feel approval"
                    );
                }
            }
        } else {
            if r.index < routes::STAGES.len() && routes::STAGES[r.index].crouch && r.t == 65 {
                r.capture_name = Some(if r.index == 22 {
                    "west-sleeve-inside"
                } else if r.index == 24 {
                    "east-sleeve-inside"
                } else {
                    "sleeve-reverse"
                });
            }
            let done = if r.index < routes::STAGES.len() {
                routes::landed(routes::STAGES[r.index], &s, r.t)
            } else {
                recovered
            };
            if done {
                println!(
                    "EMBER REVIEW pass index={} ticks={} fuel={:.3} body={:?}",
                    r.index,
                    r.t + 1,
                    game.world.player.fuel,
                    game.world.player.body
                );
                r.settle = 90;
            }
            r.t += 1;
            assert!(r.t < 360, "native injected route {} failed", r.index);
        }
    }
    game.ember_review = Some(r);
}
