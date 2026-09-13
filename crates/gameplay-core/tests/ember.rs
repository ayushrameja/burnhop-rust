use burnhop_gameplay_core::{
    ember::{self, EMBER_RELAY as MAP},
    offline::*,
    *,
};
fn new() -> OfflinePracticeState {
    OfflinePracticeState::new(MapId::EmberRelay).unwrap()
}
fn tick(s: &mut OfflinePracticeState, mut c: OfflineCommand) -> OfflineEvents {
    c.input.tick = s.world.tick;
    let e = s.step(c).unwrap();
    assert!(
        !mixed::overlaps(s.world.player.body, &MAP),
        "overlap {:?}",
        s.world.player
    );
    e
}
fn at(x: f64, feet: f64) -> OfflinePracticeState {
    let mut s = new();
    s.world.player.body = Rect::new(x, feet - 68., 36., 68.);
    s.world.player.grounded = mixed::supported(s.world.player.body, &MAP);
    s.world.player.coyote_ticks = if s.world.player.grounded { 8 } else { 0 };
    s
}
fn input(i: InputCommand) -> OfflineCommand {
    OfflineCommand {
        input: i,
        ..Default::default()
    }
}
#[test]
fn geometry_and_all_authored_spawns_are_valid_and_stable() {
    assert_eq!(MAP.solids.len(), 29);
    assert_eq!(MAP.quads.len(), 9);
    assert_eq!(ember::SPAWNS.len(), 9);
    assert_eq!((MAP.width, MAP.height, MAP.floor_y), (3200., 1900., 1640.));
    ember::validate(&MAP, ember::SPAWNS).unwrap();
    for &(id, p) in ember::SPAWNS {
        let mut w = World::new(&Arena { spawn: p, ..MAP });
        let body = w.player.body;
        for _ in 0..60 {
            let t = w.tick;
            step(
                &mut w,
                InputCommand {
                    tick: t,
                    ..Default::default()
                },
                &MAP,
            )
            .unwrap();
            assert_eq!(w.player.body, body, "{id}");
            assert!(w.player.grounded);
        }
    }
    assert!(
        ember::validate(&MAP, &[("bad", Vec2 { x: 1490., y: 1112. })])
            .unwrap_err()
            .contains("bad")
    );
    assert!(
        ember::validate(
            &Arena {
                quads: &[mixed::Quad([Vec2 { x: 0., y: 0. }; 4])],
                ..MAP
            },
            &[]
        )
        .is_err()
    );
}
#[test]
fn all_nine_slopes_walk_both_directions_and_idle_without_drift() {
    for q in MAP.quads {
        let a = q.0[0];
        let b = q.0[1];
        for dir in [-1., 1.] {
            let start = if dir > 0. { a.x + 10. } else { b.x - 46. };
            let high_x = if b.y < a.y { start + 36. } else { start };
            let y = a.y + (b.y - a.y) * (high_x - a.x) / (b.x - a.x);
            let mut body = Rect::new(start, y - 68., 36., 68.);
            for _ in 0..10 {
                let old = body;
                let c = mixed::move_body(
                    &mut body,
                    Vec2 {
                        x: 0.,
                        y: 0.4166666667,
                    },
                    &MAP,
                    true,
                );
                assert!(c.contacts.grounded);
                assert_eq!(body.x, old.x);
                assert!((body.y - old.y).abs() < 1e-8);
            }
            for _ in 0..20 {
                let c = mixed::move_body(
                    &mut body,
                    Vec2 {
                        x: dir * 4.,
                        y: 0.4166666667,
                    },
                    &MAP,
                    true,
                );
                assert!(!c.cap_hit);
                assert!(!mixed::overlaps(body, &MAP));
                assert!(c.contacts.grounded, "slope {q:?} {body:?}");
                assert!(!c.contacts.hit_x);
            }
        }
    }
}
#[test]
fn seams_preserve_horizontal_speed_and_grounded_descent() {
    for (x, feet, dir, n) in [
        (440., 1186., 1., 80),
        (680., 1540., 1., 60),
        (960., 1640., -1., 60),
        (2160., 1560., 1., 70),
    ] {
        let mut body = Rect::new(x, feet - 68., 36., 68.);
        // Place exactly on the highest terrain under the body using a downward probe.
        body.y -= 50.;
        mixed::move_body(&mut body, Vec2 { x: 0., y: 100. }, &MAP, false);
        for _ in 0..n {
            let old = body.x;
            let c = mixed::move_body(
                &mut body,
                Vec2 {
                    x: dir * 3.,
                    y: 0.42,
                },
                &MAP,
                true,
            );
            assert!(!c.cap_hit);
            assert!(!mixed::overlaps(body, &MAP));
            assert!((body.x - old - dir * 3.).abs() < 1e-7);
            assert!(c.contacts.grounded);
        }
    }
}
#[test]
fn extreme_sweeps_block_floats_roofs_sides_and_ceiling_without_tunneling() {
    for (mut b, d) in [
        (Rect::new(330., 500., 36., 68.), Vec2 { x: 0., y: 1400. }),
        (
            Rect::new(1120., 1581., 36., 54.2),
            Vec2 { x: 0., y: -2000. },
        ),
        (
            Rect::new(3100., 100., 36., 68.),
            Vec2 { x: 5000., y: -500. },
        ),
        (
            Rect::new(140., 100., 36., 68.),
            Vec2 {
                x: -5000.,
                y: -500.,
            },
        ),
        (Rect::new(500., 100., 36., 68.), Vec2 { x: 20., y: -5000. }),
    ] {
        let c = mixed::move_body(&mut b, d, &MAP, false);
        assert!(!mixed::overlaps(b, &MAP));
        assert!(!c.cap_hit);
        assert!(c.contacts.hit_x || c.contacts.hit_y);
        assert!(b.x >= -1e-7 && b.x + 36. <= 3200. + 1e-7 && b.y >= -1e-7);
    }
}
#[test]
fn crouch_partial_expansion_and_ceiling_jet_in_both_sleeves() {
    for x in [1030., 2400.] {
        let mut s = at(x, 1640.);
        for _ in 0..11 {
            tick(
                &mut s,
                OfflineCommand {
                    crouch_held: true,
                    ..Default::default()
                },
            );
        }
        assert!((s.world.player.body.height - CROUCH_HEIGHT).abs() < 1e-10);
        for _ in 0..30 {
            tick(
                &mut s,
                OfflineCommand {
                    crouch_held: true,
                    ..input(InputCommand {
                        move_x: MoveAxis::Right,
                        ..Default::default()
                    })
                },
            );
        }
        for _ in 0..20 {
            tick(&mut s, OfflineCommand::default());
        }
        assert!(s.world.player.body.height > 58. && s.world.player.body.height < 60.);
        let short = s.world.player.body.height;
        tick(
            &mut s,
            input(InputCommand {
                jump_pressed: true,
                jet_pressed: true,
                jet_held: true,
                ..Default::default()
            }),
        );
        for _ in 0..10 {
            tick(
                &mut s,
                input(InputCommand {
                    jet_held: true,
                    ..Default::default()
                }),
            );
        }
        assert!(s.world.player.fuel < 99.);
        assert_eq!(s.world.player.body.height, short);
        for _ in 0..85 {
            tick(
                &mut s,
                input(InputCommand {
                    move_x: MoveAxis::Right,
                    ..Default::default()
                }),
            );
        }
        assert_eq!(s.world.player.body.height, 68.);
    }
}
#[test]
fn held_space_and_jets_prevent_recrouch_and_airborne_hold_cannot_shrink() {
    for c in [
        OfflineCommand {
            crouch_held: true,
            jump_held: true,
            ..Default::default()
        },
        OfflineCommand {
            crouch_held: true,
            ..input(InputCommand {
                jet_held: true,
                ..Default::default()
            })
        },
    ] {
        let mut s = new();
        for _ in 0..20 {
            tick(&mut s, c);
        }
        assert_eq!(s.stance.amount, 0.);
    }
    let mut s = at(1810., 1100.);
    for _ in 0..10 {
        tick(
            &mut s,
            OfflineCommand {
                crouch_held: true,
                ..Default::default()
            },
        );
    }
    assert_eq!(s.stance.amount, 0.);
}
#[test]
fn both_voids_are_full_height_and_recover_without_death() {
    for x in [1490., 1810.] {
        let mut s = at(x, 1168.);
        let mut recovered = false;
        for _ in 0..150 {
            let e = tick(&mut s, OfflineCommand::default());
            if e.recovered {
                recovered = true;
                break;
            }
            assert!(!s.world.player.grounded);
        }
        assert!(recovered);
        assert_eq!(s.world.player.body, World::new(&MAP).player.body);
        assert_eq!(s.combat.deaths, 0);
        assert_eq!(s.combat.bot_attack_ticks, 180);
    }
}
#[test]
fn recovery_preserves_resources_advances_reload_once_and_requires_release() {
    let mut s = at(1490., 2000.);
    s.world.player.fuel = 31.;
    s.world.player.fuel_delay_ticks = 17;
    s.combat.player.health = 37;
    s.combat.kills = 5;
    s.combat.deaths = 2;
    s.combat.player.selected = WeaponId::M416;
    s.combat.player.equip_ticks = 9;
    s.combat.player.weapons[1].ammo = 5;
    s.combat.player.weapons[1].reserve = Reserve::Rounds(20);
    s.combat.player.weapons[1].reload_ticks = 1;
    s.combat.player.weapons[0].cooldown_ticks = 7;
    let held = OfflineCommand {
        crouch_held: true,
        jump_held: true,
        ..input(InputCommand {
            move_x: MoveAxis::Right,
            jet_pressed: true,
            jet_held: true,
            fire_held: true,
            reload_pressed: true,
            select_weapon: Some(WeaponId::Pistol),
            aim_at: Some(Vec2 { x: 1200., y: 1100. }),
            ..Default::default()
        })
    };
    let e = tick(&mut s, held);
    assert!(e.recovered);
    assert_eq!(e.combat.shots, [None, None]);
    assert_eq!(s.world.player.fuel, 31.);
    assert_eq!(s.world.player.fuel_delay_ticks, 17);
    assert_eq!(s.combat.player.health, 37);
    assert_eq!((s.combat.kills, s.combat.deaths), (5, 2));
    assert_eq!(s.combat.player.selected, WeaponId::M416);
    assert_eq!(s.combat.player.weapon().ammo, 25);
    assert_eq!(s.combat.player.weapon().reserve, Reserve::Rounds(0));
    assert_eq!(s.combat.player.equip_ticks, 8);
    assert_eq!(s.combat.player.weapons[0].cooldown_ticks, 6);
    for _ in 0..30 {
        tick(&mut s, held);
        assert_eq!(s.world.player.body.x, 200.);
        assert!(s.stance.require_release);
    }
    assert!(s.world.player.fuel > 31.);
    tick(&mut s, OfflineCommand::default());
    assert!(!s.stance.require_release);
    tick(
        &mut s,
        input(InputCommand {
            move_x: MoveAxis::Right,
            ..Default::default()
        }),
    );
    assert!(s.world.player.body.x > 200.);
}
#[test]
fn threshold_pre_post_movement_and_dead_precedence() {
    let mut s = at(1490., 1968.);
    s.world.player.fuel = 30.;
    s.world.player.fuel_delay_ticks = 10;
    assert!(tick(&mut s, OfflineCommand::default()).recovered);
    assert_eq!(s.world.player.fuel_delay_ticks, 9); // equality moves first
    let mut s = at(1490., 1968.0001);
    s.world.player.fuel_delay_ticks = 10;
    assert!(tick(&mut s, OfflineCommand::default()).recovered);
    assert_eq!(s.world.player.fuel_delay_ticks, 10);
    let mut s = at(1490., 2000.);
    s.combat.player.health = 0;
    s.combat.player.life = LifeState::Dead {
        remaining_ticks: 180,
    };
    for _ in 0..179 {
        let e = s
            .step(OfflineCommand {
                input: InputCommand {
                    tick: s.world.tick,
                    ..Default::default()
                },
                ..Default::default()
            })
            .unwrap();
        assert!(!e.recovered);
        assert!(!e.combat.player_respawned);
    }
    let e = tick(&mut s, OfflineCommand::default());
    assert!(e.combat.player_respawned);
    assert!(!e.recovered);
    assert_eq!(s.combat.player.health, 100);
    assert_eq!(s.world.player.body, World::new(&MAP).player.body);
}
#[test]
fn grace_counts_only_eligible_ticks_and_first_attempt_is_180() {
    let mut s = at(1490., 2000.);
    tick(&mut s, OfflineCommand::default());
    s.combat.player.life = LifeState::Dead {
        remaining_ticks: 10,
    };
    for _ in 0..9 {
        tick(&mut s, OfflineCommand::default());
        assert_eq!(s.combat.bot_attack_ticks, 180);
    }
    tick(&mut s, OfflineCommand::default());
    // Exposed supported position beside bot; ordinary neutral commands, no teleport during countdown.
    s.world.player.body = Rect::new(1105., 1032., 36., 68.);
    for n in 1..180 {
        let e = tick(&mut s, OfflineCommand::default());
        assert!(e.combat.shots[1].is_none());
        assert_eq!(s.combat.bot_attack_ticks, 180 - n);
    }
    assert!(tick(&mut s, OfflineCommand::default()).combat.shots[1].is_some());
    assert_eq!(s.combat.bot_attack_ticks, 60);
}
#[test]
fn f5_resets_active_map_and_cloned_partial_stance_replays_exactly() {
    let mut s = at(1030., 1640.);
    for _ in 0..5 {
        tick(
            &mut s,
            OfflineCommand {
                crouch_held: true,
                ..Default::default()
            },
        );
    }
    let mut copy = s;
    for _ in 0..100 {
        let c = OfflineCommand {
            crouch_held: true,
            ..input(InputCommand {
                move_x: MoveAxis::Right,
                ..Default::default()
            })
        };
        assert_eq!(tick(&mut s, c), tick(&mut copy, c));
        assert_eq!(s, copy);
    }
    s.combat.kills = 9;
    s.world.player.fuel = 5.;
    let old = s.world.tick;
    let e = tick(
        &mut s,
        input(InputCommand {
            reset: true,
            ..Default::default()
        }),
    );
    assert!(e.combat.movement.reset);
    assert_eq!(s.world.tick, old + 1);
    assert_eq!(s.world.player, World::new(&MAP).player);
    assert_eq!(s.combat.kills, 0);
    assert_eq!(s.combat.bot_body, Rect::new(1170., 1032., 36., 68.));
}
#[test]
fn map_rays_share_slopes_roofs_occlusion_and_open_sightlines() {
    let from = body_center(World::new(&MAP).player.body);
    let to = body_center(new().combat.bot_body);
    let aim = direction(from, to).unwrap();
    assert_eq!(
        mixed::nearest_hit(
            from,
            aim,
            1000.,
            ActorId::One,
            &MAP,
            &[(ActorId::Two, new().combat.bot_body)]
        )
        .1,
        Impact::Terrain
    );
    for (x, y, len) in [(1810., 500., 1300.), (1300., 600., 400.)] {
        assert_eq!(
            mixed::cover(Vec2 { x, y }, Vec2 { x: 0., y: 1. }, len, &MAP),
            None
        );
    }
    for q in MAP.quads {
        let center = q.0.iter().fold(Vec2::default(), |a, b| Vec2 {
            x: a.x + b.x / 4.,
            y: a.y + b.y / 4.,
        });
        assert_eq!(
            mixed::ray_quad(center, Vec2 { x: 1., y: 0. }, *q, 1000.),
            Some(0.)
        );
    }
    for x in [1150., 2500.] {
        assert_eq!(
            mixed::cover(Vec2 { x, y: 1600. }, Vec2 { x: 0., y: -1. }, 100., &MAP),
            Some(20.)
        );
    }
}
#[test]
fn range_wrapper_ignores_stance_and_keeps_exact_body_and_defaults() {
    let mut s = OfflinePracticeState::new(MapId::Range).unwrap();
    let mut w = s.world;
    let mut c = s.combat;
    for t in 0..120 {
        let cmd = InputCommand {
            tick: t,
            move_x: MoveAxis::Right,
            jump_pressed: t == 10,
            ..Default::default()
        };
        let expected = step_practice(&mut w, &mut c, cmd, &PRACTICE_ARENA).unwrap();
        let actual = s
            .step(OfflineCommand {
                input: cmd,
                crouch_held: true,
                jump_held: true,
            })
            .unwrap();
        assert_eq!(actual.combat, expected);
        assert_eq!((s.world, s.combat), (w, c));
    }
}

#[path = "../../client/src/ember_routes.rs"]
mod routes;
#[test]
fn measured_traversal_stages_at_sixty_and_full_fuel() {
    let mut failures = Vec::new();
    for fuel in [60., 100.] {
        for &stage in routes::STAGES {
            let mut s = routes::initial(stage, fuel);
            let mut done = false;
            let mut min_fuel = fuel;
            for t in 0..360 {
                let c = routes::command(stage, &s, t);
                let e = tick(&mut s, c);
                min_fuel = min_fuel.min(s.world.player.fuel);
                if e.recovered {
                    break;
                }
                if routes::landed(stage, &s, t) {
                    println!(
                        "ROUTE {} fuel={} ticks={} seconds={:.3} spent={:.3}",
                        stage.name,
                        fuel,
                        t + 1,
                        f64::from(t + 1) / 60.,
                        fuel - min_fuel
                    );
                    done = true;
                    break;
                }
            }
            if !done {
                failures.push(format!(
                    "{} fuel={} final={:?}",
                    stage.name, fuel, s.world.player
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn runtime_geometry_matches_every_authored_svg_solid() {
    let svg = include_str!("../../../docs/design/ember-relay-layout.svg");
    fn attr<'a>(tag: &'a str, name: &str) -> &'a str {
        tag.split_once(&format!("{name}=\""))
            .unwrap()
            .1
            .split('"')
            .next()
            .unwrap()
    }
    let mut nr = 0;
    let mut nq = 0;
    for tag in svg
        .split('<')
        .filter(|t| t.starts_with("rect id=\"") || t.starts_with("polygon id=\""))
    {
        let id = attr(tag, "id");
        if id.starts_with("actor-") {
            continue;
        }
        if tag.starts_with("rect") {
            let v: [f64; 4] = ["x", "y", "width", "height"].map(|k| attr(tag, k).parse().unwrap());
            assert_eq!(MAP.solids[nr], Rect::new(v[0], v[1], v[2], v[3]), "{id}");
            nr += 1;
        } else {
            let nums: Vec<f64> = attr(tag, "points")
                .split([' ', ','])
                .map(|n| n.parse().unwrap())
                .collect();
            let q = mixed::Quad(std::array::from_fn(|i| Vec2 {
                x: nums[i * 2],
                y: nums[i * 2 + 1],
            }));
            assert_eq!(MAP.quads[nq], q, "{id}");
            nq += 1;
        }
    }
    assert_eq!((nr, nq), (26, 9));
}
#[test]
fn invalid_roles_bounds_and_intersecting_interiors_report_errors() {
    const OUTSIDE: &[Rect] = &[Rect::new(-1., 800., 200., 100.)];
    const INTERSECT: &[Rect] = &[
        Rect::new(100., 100., 100., 100.),
        Rect::new(150., 150., 100., 100.),
    ];
    assert!(
        ember::validate(
            &Arena {
                spawn: Vec2 { x: 1490., y: 1112. },
                ..MAP
            },
            &[]
        )
        .unwrap_err()
        .contains("player/recovery")
    );
    assert!(
        ember::validate(
            &Arena {
                solids: OUTSIDE,
                ..MAP
            },
            &[]
        )
        .unwrap_err()
        .contains("bounds")
    );
    assert!(
        ember::validate(
            &Arena {
                solids: INTERSECT,
                ..MAP
            },
            &[]
        )
        .unwrap_err()
        .contains("intersect")
    );
}
#[test]
fn randomized_extreme_displacements_never_penetrate_or_exceed_contact_cap() {
    let mut seed = 12345_u64;
    let mut random = || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        (seed >> 32) as f64 / u32::MAX as f64
    };
    let mut checked = 0;
    for _ in 0..1500 {
        let mut b = Rect::new(random() * 3164., random() * 1832., 36., 68.);
        if mixed::overlaps(b, &MAP) {
            continue;
        }
        let delta = Vec2 {
            x: (random() - 0.5) * 10000.,
            y: (random() - 0.5) * 10000.,
        };
        let c = mixed::move_body(&mut b, delta, &MAP, false);
        assert!(!mixed::overlaps(b, &MAP), "{b:?} {delta:?} {c:?}");
        assert!(!c.cap_hit, "{c:?}");
        checked += 1;
    }
    assert!(checked > 1000);
}
#[test]
fn low_fuel_can_regenerate_in_air_and_stage_after_a_rest_without_retuning() {
    let mut s = at(1810., 1168.);
    s.world.player.fuel = 20.;
    for _ in 0..10 {
        assert!(!tick(&mut s, OfflineCommand::default()).recovered);
    }
    assert_eq!(s.world.player.fuel, 25.);
    assert!(!s.world.player.grounded);
    for t in 0..40 {
        let e = tick(
            &mut s,
            input(InputCommand {
                jet_pressed: t == 0,
                jet_held: true,
                ..Default::default()
            }),
        );
        assert!(!e.recovered);
    }
    assert!(
        s.world.player.body.y < 1250.,
        "airborne refill/repress should arrest this specific fall"
    );
    let stage = routes::STAGES[12];
    let mut s = routes::initial(stage, 20.);
    for _ in 0..80 {
        tick(&mut s, OfflineCommand::default());
    }
    assert_eq!(s.world.player.fuel, 60.);
    let mut landed = false;
    for t in 0..180 {
        let c = routes::command(stage, &s, t);
        tick(&mut s, c);
        if routes::landed(stage, &s, t) {
            landed = true;
            break;
        }
    }
    assert!(landed);
}
#[test]
fn slow_lower_launch_is_a_failed_attempt_and_recovery_is_not_a_hidden_floor() {
    let mut stage = routes::STAGES[8];
    stage.start.0 = 1320.;
    stage.start.1 = 1608.;
    stage.vx = 0.;
    let mut s = routes::initial(stage, 20.);
    let mut success = false;
    for t in 0..120 {
        let c = routes::command(stage, &s, t);
        let e = tick(&mut s, c);
        if routes::landed(stage, &s, t) {
            success = true;
            break;
        }
        if e.recovered {
            break;
        }
    }
    assert!(
        !success,
        "early slow takeoff is not equivalent to a full-speed lip launch"
    );
}
