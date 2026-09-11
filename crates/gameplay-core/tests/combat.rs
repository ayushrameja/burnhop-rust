use burnhop_gameplay_core::*;

struct Practice {
    world: World,
    combat: CombatState,
}
impl Practice {
    fn new() -> Self {
        Self {
            world: World::new(&PRACTICE_ARENA),
            combat: CombatState {
                bot_attack_ticks: u16::MAX,
                ..Default::default()
            },
        }
    }
    fn tick(&mut self, input: InputCommand) -> CombatEvents {
        let tick = self.world.tick;
        step_practice(
            &mut self.world,
            &mut self.combat,
            InputCommand { tick, ..input },
            &PRACTICE_ARENA,
        )
        .unwrap()
    }
    fn idle(&mut self, ticks: u16) {
        for _ in 0..ticks {
            self.tick(InputCommand::default());
        }
    }
}
fn fire() -> InputCommand {
    InputCommand {
        aim_at: Some(Vec2 {
            x: 1900.0,
            y: 1186.0,
        }),
        fire_held: true,
        ..Default::default()
    }
}

#[test]
fn both_reference_weapons_repeat_on_hold_at_exact_cadence() {
    for id in [WeaponId::Pistol, WeaponId::M416] {
        let mut p = Practice::new();
        p.combat.player.selected = id;
        let cooldown = u64::from(id.tuning().cooldown);
        let mut fired = Vec::new();
        for tick in 0..40 {
            if p.tick(fire()).shots[0].is_some() {
                fired.push(tick);
            }
        }
        assert_eq!(
            fired,
            (0..40).step_by(cooldown as usize).collect::<Vec<_>>()
        );
        assert_eq!(
            p.combat.player.weapon().ammo,
            id.tuning().magazine - fired.len() as u16
        );
        assert_eq!(p.tick(InputCommand::default()).shots[0], None);
    }
}
#[test]
fn taps_release_and_rapid_clicks_never_bypass_cooldown() {
    let mut p = Practice::new();
    assert!(p.tick(fire()).shots[0].is_some());
    for t in 1..12 {
        assert!(
            p.tick(if t % 2 == 0 {
                fire()
            } else {
                InputCommand::default()
            })
            .shots[0]
                .is_none()
        );
    }
    assert!(p.tick(fire()).shots[0].is_some());
}
#[test]
fn empty_trigger_never_auto_reloads_or_spends_reserve() {
    let mut p = Practice::new();
    p.combat.player.weapons[0].ammo = 1;
    assert!(p.tick(fire()).shots[0].is_some());
    for _ in 0..300 {
        assert!(p.tick(fire()).shots[0].is_none());
    }
    assert_eq!(p.combat.player.weapon().ammo, 0);
    assert_eq!(p.combat.player.weapon().reserve, Reserve::Rounds(48));
    assert_eq!(p.combat.player.weapon().reload_ticks, 0);
}
#[test]
fn reload_consumes_exact_ticks_and_cannot_fire_during_reload() {
    for id in [WeaponId::Pistol, WeaponId::M416] {
        let mut p = Practice::new();
        p.combat.player.selected = id;
        p.combat.player.weapons[id.index()].ammo = 2;
        let start = p.tick(InputCommand {
            reload_pressed: true,
            ..fire()
        });
        assert!(start.shots[0].is_none());
        let duration = id.tuning().reload;
        assert_eq!(p.combat.player.weapon().reload_ticks, duration);
        for remaining in (1..duration).rev() {
            assert!(p.tick(fire()).shots[0].is_none());
            assert_eq!(p.combat.player.weapon().reload_ticks, remaining);
            assert_eq!(p.combat.player.weapon().ammo, 2);
        }
        assert!(p.tick(fire()).shots[0].is_some());
        assert_eq!(p.combat.player.weapon().reload_ticks, 0);
        assert_eq!(p.combat.player.weapon().ammo, id.tuning().magazine - 1);
    }
}
#[test]
fn partial_reserve_reload_and_full_or_empty_reserve_requests() {
    let mut p = Practice::new();
    p.tick(InputCommand {
        reload_pressed: true,
        ..Default::default()
    });
    assert_eq!(p.combat.player.weapon().reload_ticks, 0);
    p.tick(InputCommand::default());
    p.combat.player.weapons[0].ammo = 3;
    p.combat.player.weapons[0].reserve = Reserve::Rounds(2);
    p.tick(InputCommand {
        reload_pressed: true,
        ..Default::default()
    });
    p.idle(72);
    assert_eq!(p.combat.player.weapon().ammo, 5);
    assert_eq!(p.combat.player.weapon().reserve, Reserve::Rounds(0));
    p.tick(InputCommand {
        reload_pressed: true,
        ..Default::default()
    });
    assert_eq!(p.combat.player.weapon().reload_ticks, 0);
}
#[test]
fn held_reload_edge_does_not_restart_or_queue_another_reload() {
    let mut p = Practice::new();
    p.combat.player.weapons[0].ammo = 5;
    for _ in 0..80 {
        p.tick(InputCommand {
            reload_pressed: true,
            ..fire()
        });
    }
    assert_eq!(p.combat.player.weapon().reload_ticks, 0);
    assert_eq!(p.combat.player.weapon().ammo, 11);
}
#[test]
fn switching_cancels_reload_preserves_inventory_and_enforces_equip_delay() {
    let mut p = Practice::new();
    p.tick(fire());
    p.tick(InputCommand {
        reload_pressed: true,
        ..Default::default()
    });
    assert!(p.combat.player.weapon().reload_ticks > 0);
    p.tick(InputCommand {
        select_weapon: Some(WeaponId::M416),
        ..fire()
    });
    assert_eq!(p.combat.player.weapons[0].reload_ticks, 0);
    assert_eq!(p.combat.player.weapons[0].ammo, 11);
    assert_eq!(p.combat.player.weapons[0].cooldown_ticks, 10);
    assert_eq!(p.combat.player.equip_ticks, 18);
    for _ in 0..17 {
        assert!(
            p.tick(InputCommand {
                select_weapon: Some(WeaponId::M416),
                ..fire()
            })
            .shots[0]
                .is_none()
        );
    }
    assert!(p.tick(fire()).shots[0].is_some());
    p.tick(InputCommand {
        select_weapon: Some(WeaponId::Pistol),
        ..fire()
    });
    assert_eq!(p.combat.player.weapons[1].cooldown_ticks, 6);
    assert_eq!(p.combat.player.weapon().ammo, 11);
    assert_eq!(p.combat.player.weapon().reserve, Reserve::Rounds(48));
}
#[test]
fn repeated_switches_and_releases_do_not_bypass_longer_weapon_cooldown() {
    let mut p = Practice::new();
    p.combat.player.weapons[0].cooldown_ticks = 80;
    for tick in 0..20 {
        assert!(
            p.tick(InputCommand {
                select_weapon: Some(if tick % 2 == 0 {
                    WeaponId::M416
                } else {
                    WeaponId::Pistol
                }),
                ..fire()
            })
            .shots[0]
                .is_none()
        );
    }
    assert_eq!(p.combat.player.weapons[0].cooldown_ticks, 60);
    for _ in 0..59 {
        assert!(p.tick(fire()).shots[0].is_none());
    }
    assert!(p.tick(fire()).shots[0].is_some());
}
#[test]
fn invalid_missing_or_zero_length_aim_cannot_fire_or_corrupt_direction() {
    let mut p = Practice::new();
    for aim_at in [
        None,
        Some(Vec2 {
            x: f64::NAN,
            y: 0.0,
        }),
        Some(Vec2 {
            x: f64::INFINITY,
            y: 0.0,
        }),
        Some(body_center(p.world.player.body)),
    ] {
        assert!(p.tick(InputCommand { aim_at, ..fire() }).shots[0].is_none());
        assert_eq!(p.combat.player.aim, Vec2 { x: 1.0, y: 0.0 });
    }
    assert!(p.tick(fire()).shots[0].is_some());
}
#[test]
fn whole_range_ray_stops_at_a_thin_wall_before_any_body_even_at_extreme_distance() {
    let origin = Vec2 { x: 0.0, y: 5.0 };
    let aim = Vec2 { x: 1.0, y: 0.0 };
    let wall = Rect::new(100.0, 0.0, 0.01, 10.0);
    let bodies = [
        (ActorId::One, Rect::new(-1.0, 0.0, 2.0, 10.0)),
        (ActorId::Two, Rect::new(200.0, 0.0, 10.0, 10.0)),
    ];
    assert_eq!(
        nearest_hit(origin, aim, 1_000_000.0, ActorId::One, &[wall], &bodies),
        (100.0, Impact::Terrain)
    );
    assert_eq!(
        nearest_hit(origin, aim, 1_000_000.0, ActorId::One, &[], &bodies),
        (200.0, Impact::Body(ActorId::Two))
    );
    // Moving the wall behind the body selects the body; geometry ordering is irrelevant.
    let behind = Rect::new(500.0, 0.0, 1.0, 10.0);
    for solids in [[wall, behind], [behind, wall]] {
        assert_eq!(
            nearest_hit(origin, aim, 1000.0, ActorId::One, &solids, &bodies).0,
            100.0
        );
    }
    assert_eq!(
        nearest_hit(origin, aim, 1000.0, ActorId::One, &[behind], &bodies).1,
        Impact::Body(ActorId::Two)
    );
}
#[test]
fn cover_wins_ties_and_barrel_protrusion_cannot_skip_near_cover() {
    let wall = Rect::new(427.0, 1100.0, 1.0, 120.0);
    let mut p = Practice::new();
    p.combat.player.selected = WeaponId::M416;
    const ARENA: Arena = Arena {
        solids: &[Rect::new(427.0, 1100.0, 1.0, 120.0)],
        ..PRACTICE_ARENA
    };
    let origin = body_center(p.world.player.body);
    assert_eq!(
        nearest_hit(
            origin,
            Vec2 { x: 1.0, y: 0.0 },
            1000.0,
            ActorId::One,
            &[wall],
            &[(ActorId::Two, wall)]
        ),
        (19.0, Impact::Terrain)
    );
    let events = step_practice(&mut p.world, &mut p.combat, fire(), &ARENA).unwrap();
    assert_eq!(events.shots[0].unwrap().impact, Impact::Terrain);
    assert_eq!(p.combat.bot.health, 100);
}
#[test]
fn ray_handles_vertical_reverse_start_overlap_and_range_boundaries() {
    let wall = Rect::new(10.0, 10.0, 1.0, 1.0);
    for (origin, aim, expected) in [
        (
            Vec2 { x: 10.5, y: 0.0 },
            Vec2 { x: 0.0, y: 1.0 },
            Some(10.0),
        ),
        (
            Vec2 { x: 20.0, y: 10.5 },
            Vec2 { x: -1.0, y: 0.0 },
            Some(9.0),
        ),
        (
            Vec2 { x: 10.5, y: 10.5 },
            Vec2 { x: 1.0, y: 0.0 },
            Some(0.0),
        ),
        (Vec2 { x: 9.0, y: 0.0 }, Vec2 { x: 0.0, y: 1.0 }, None),
    ] {
        assert_eq!(ray_rect(origin, aim, wall, 100.0), expected);
    }
    assert_eq!(
        ray_rect(Vec2 { x: 0.0, y: 10.5 }, Vec2 { x: 1.0, y: 0.0 }, wall, 9.0),
        None
    );
}
#[test]
fn damage_retains_reference_falloff_and_never_exceeds_remaining_health() {
    assert_eq!(body_damage(WeaponId::Pistol, 300.0), 18);
    assert_eq!(body_damage(WeaponId::Pistol, 550.0), 13);
    assert_eq!(body_damage(WeaponId::Pistol, 800.0), 7);
    assert_eq!(body_damage(WeaponId::M416, 800.0), 23);
    assert_eq!(body_damage(WeaponId::M416, 1600.0), 18);
    for d in [-1.0, f64::NAN, 2000.0] {
        assert_eq!(body_damage(WeaponId::Pistol, d), 0);
    }
    let mut p = Practice::new();
    p.combat.bot.health = 1;
    let events = p.tick(fire());
    assert_eq!(events.shots[0].unwrap().damage, 1);
    assert!(events.bot_died);
    assert_eq!(p.combat.kills, 1);
    p.idle(12);
    assert_eq!(p.tick(fire()).shots[0].unwrap().damage, 0);
    assert_eq!(p.combat.kills, 1);
}
#[test]
fn bot_dies_and_respawns_after_exactly_180_ticks_with_full_state() {
    let mut p = Practice::new();
    p.combat.bot.health = 1;
    assert!(p.tick(fire()).bot_died);
    p.idle(179);
    assert!(!p.combat.bot.alive());
    assert!(p.tick(InputCommand::default()).bot_respawned);
    assert_eq!(p.combat.bot, Combatant::new(true));
    assert_eq!(p.combat.bot_attack_ticks, BOT_GRACE_TICKS);
}
#[test]
fn player_death_blocks_all_actions_and_respawn_requires_fresh_intent() {
    let mut p = Practice::new();
    p.combat.player.health = 1;
    p.combat.bot_attack_ticks = 1;
    assert!(p.tick(InputCommand::default()).player_died);
    let body = p.world.player.body;
    let stale = InputCommand {
        move_x: MoveAxis::Right,
        jump_pressed: true,
        jet_pressed: true,
        jet_held: true,
        reload_pressed: true,
        select_weapon: Some(WeaponId::M416),
        ..fire()
    };
    for _ in 0..179 {
        let events = p.tick(stale);
        assert_eq!(p.world.player.body, body);
        assert!(events.shots.iter().all(Option::is_none));
    }
    let events = p.tick(stale);
    assert!(events.player_respawned);
    assert_eq!(p.world.player, World::new(&PRACTICE_ARENA).player);
    assert_eq!(p.combat.player, Combatant::new(false));
    assert!(p.tick(stale).shots[0].is_none());
    assert_eq!(p.world.player.body, body);
    p.tick(InputCommand::default());
    assert!(p.tick(fire()).shots[0].is_some());
    assert_eq!(p.combat.deaths, 1);
}
#[test]
fn bot_attack_has_grace_one_second_cadence_and_obeys_terrain() {
    let mut p = Practice::new();
    p.combat.bot_attack_ticks = BOT_GRACE_TICKS;
    p.idle(179);
    assert_eq!(p.combat.player.health, 100);
    assert!(p.tick(InputCommand::default()).shots[1].is_some());
    for _ in 0..59 {
        assert!(p.tick(InputCommand::default()).shots[1].is_none());
    }
    assert!(p.tick(InputCommand::default()).shots[1].is_some());
    const ARENA: Arena = Arena {
        solids: &[
            Rect::new(600.0, 1000.0, 1.0, 500.0),
            Rect::new(0.0, 1220.0, 2400.0, 200.0),
        ],
        ..PRACTICE_ARENA
    };
    p.combat.bot_attack_ticks = 0;
    for _ in 0..200 {
        let tick = p.world.tick;
        assert!(
            step_practice(
                &mut p.world,
                &mut p.combat,
                InputCommand {
                    tick,
                    ..Default::default()
                },
                &ARENA
            )
            .unwrap()
            .shots[1]
                .is_none()
        );
    }
}
#[test]
fn bot_reloads_and_resumes_its_limited_attack() {
    let mut p = Practice::new();
    p.combat.bot.weapons[0].ammo = 0;
    p.combat.bot_attack_ticks = 0;
    p.tick(InputCommand::default());
    assert_eq!(p.combat.bot.weapon().reload_ticks, 72);
    p.idle(71);
    assert_eq!(p.combat.bot.weapon().ammo, 0);
    assert!(p.tick(InputCommand::default()).shots[1].is_some());
    assert_eq!(p.combat.bot.weapon().ammo, 11);
    assert_eq!(p.combat.bot.weapon().reserve, Reserve::Unlimited);
}
#[test]
fn simultaneous_lethal_shots_are_resolved_before_both_deaths() {
    let mut p = Practice::new();
    p.combat.player.health = 1;
    p.combat.bot.health = 1;
    p.combat.bot_attack_ticks = 0;
    let events = p.tick(fire());
    assert!(events.player_died && events.bot_died);
    assert!(events.shots.iter().all(Option::is_some));
    assert_eq!((p.combat.kills, p.combat.deaths), (1, 1));
}
#[test]
fn reset_restores_both_lives_ammo_timers_counters_and_cancels_stale_intent() {
    let mut p = Practice::new();
    p.tick(fire());
    p.combat.player.health = 1;
    p.tick(InputCommand {
        reload_pressed: true,
        ..Default::default()
    });
    let tick = p.world.tick;
    let events = p.tick(InputCommand {
        reset: true,
        move_x: MoveAxis::Right,
        ..fire()
    });
    assert!(events.movement.reset);
    assert!(events.shots.iter().all(Option::is_none));
    assert_eq!(p.world.tick, tick + 1);
    assert_eq!(p.world.player, World::new(&PRACTICE_ARENA).player);
    assert_eq!(
        p.combat,
        CombatState {
            require_neutral: true,
            ..Default::default()
        }
    );
    assert!(p.tick(fire()).shots[0].is_none());
    p.tick(InputCommand {
        release_input: true,
        ..fire()
    });
    assert!(p.tick(fire()).shots[0].is_some());
}
#[test]
fn focus_release_cancels_fire_jet_and_jump_buffer_but_reload_time_continues() {
    let mut p = Practice::new();
    p.tick(fire());
    p.tick(InputCommand {
        reload_pressed: true,
        ..Default::default()
    });
    p.world.player.thrust_latched = true;
    p.world.player.jump_buffer_ticks = 9;
    let events = p.tick(InputCommand {
        release_input: true,
        jet_held: true,
        jet_pressed: true,
        jump_pressed: true,
        ..fire()
    });
    assert!(events.shots[0].is_none());
    assert!(!p.world.player.thrust_latched);
    assert_eq!(p.world.player.jump_buffer_ticks, 0);
    assert_eq!(p.combat.player.weapon().reload_ticks, 71);
}
#[test]
fn wrong_tick_is_atomic_and_identical_combat_commands_replay_on_this_machine() {
    let mut a = Practice::new();
    let mut b = Practice::new();
    assert!(
        step_practice(
            &mut a.world,
            &mut a.combat,
            InputCommand { tick: 3, ..fire() },
            &PRACTICE_ARENA
        )
        .is_err()
    );
    assert_eq!(a.world, b.world);
    assert_eq!(a.combat, b.combat);
    for tick in 0..10_000 {
        let command = InputCommand {
            move_x: match tick % 300 {
                0..100 => MoveAxis::Right,
                100..200 => MoveAxis::Left,
                _ => MoveAxis::Idle,
            },
            jet_pressed: tick % 400 == 0,
            jet_held: tick % 400 < 80,
            jump_pressed: tick % 91 == 0,
            fire_held: tick % 60 < 45,
            reload_pressed: tick % 190 == 0,
            select_weapon: if tick % 321 == 0 {
                Some(WeaponId::M416)
            } else if tick % 479 == 0 {
                Some(WeaponId::Pistol)
            } else {
                None
            },
            aim_at: Some(Vec2 {
                x: 928.0,
                y: 1186.0,
            }),
            reset: tick % 2100 == 2099,
            ..Default::default()
        };
        assert_eq!(a.tick(command), b.tick(command));
        assert_eq!(a.world, b.world);
        assert_eq!(a.combat, b.combat);
    }
}
#[test]
fn combat_wrapper_preserves_approved_movement_when_not_hit() {
    let mut p = Practice::new();
    let mut movement = World::new(&PRACTICE_ARENA);
    for tick in 0..1500 {
        let input = InputCommand {
            tick,
            move_x: if tick % 200 < 100 {
                MoveAxis::Right
            } else {
                MoveAxis::Left
            },
            jet_pressed: tick % 300 == 0,
            jet_held: tick % 300 < 90,
            jump_pressed: tick % 123 == 0,
            ..Default::default()
        };
        let event = step(&mut movement, input, &PRACTICE_ARENA).unwrap();
        assert_eq!(p.tick(input).movement, event);
        assert_eq!(p.world, movement);
    }
}
