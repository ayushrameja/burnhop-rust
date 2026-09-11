use burnhop_gameplay_core::*;

fn tick(world: &mut World, input: InputCommand, arena: &Arena) -> StepEvents {
    step(
        world,
        InputCommand {
            tick: world.tick,
            ..input
        },
        arena,
    )
    .unwrap()
}
fn advance(world: &mut World, count: usize, input: InputCommand) {
    for _ in 0..count {
        tick(world, input, &PRACTICE_ARENA);
    }
}
fn right() -> InputCommand {
    InputCommand {
        move_x: MoveAxis::Right,
        ..Default::default()
    }
}
fn jet(pressed: bool) -> InputCommand {
    InputCommand {
        jet_pressed: pressed,
        jet_held: true,
        ..Default::default()
    }
}
fn jump() -> InputCommand {
    InputCommand {
        jump_pressed: true,
        ..Default::default()
    }
}
fn near(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-7, "{a} != {b}");
}

#[test]
fn practice_geometry_spawn_and_idle_support() {
    let mut w = World::new(&PRACTICE_ARENA);
    assert_eq!(PRACTICE_ARENA.solids.len(), 8);
    assert_eq!(w.player.body, Rect::new(390.0, 1152.0, 36.0, 68.0));
    advance(&mut w, 600, InputCommand::default());
    assert_eq!(w.player, World::new(&PRACTICE_ARENA).player);
}

#[test]
fn acceleration_braking_and_speed_caps() {
    let mut w = World::new(&PRACTICE_ARENA);
    tick(&mut w, right(), &PRACTICE_ARENA);
    near(w.player.velocity.x, 3800.0 * DT);
    advance(&mut w, 29, right());
    near(w.player.velocity.x, 320.0);
    assert!(w.player.body.x > 520.0);
    advance(&mut w, 5, InputCommand::default());
    near(w.player.velocity.x, 0.0);
    w.player.body.y = 400.0;
    w.player.grounded = false;
    tick(&mut w, right(), &PRACTICE_ARENA);
    near(w.player.velocity.x, 2300.0 * DT);
    tick(&mut w, InputCommand::default(), &PRACTICE_ARENA);
    near(w.player.velocity.x, (2300.0 - 320.0) * DT);
    advance(&mut w, 200, InputCommand::default());
    assert!(w.player.velocity.y <= MAX_FALL_SPEED);
}

#[test]
fn jump_trajectory_matches_reference_and_does_not_autojet_or_repeat() {
    let mut w = World::new(&PRACTICE_ARENA);
    assert!(tick(&mut w, jump(), &PRACTICE_ARENA).jumped);
    near(w.player.velocity.y, -495.0);
    let mut apex = w.player.body.y;
    let mut landings = 0;
    for _ in 0..75 {
        let events = tick(&mut w, InputCommand::default(), &PRACTICE_ARENA);
        landings += usize::from(events.landed);
        assert!(!events.jumped && !w.player.thrusting);
        apex = apex.min(w.player.body.y);
    }
    near(1152.0 - apex, 85.83333333333333);
    assert_eq!(landings, 1);
    assert!(w.player.grounded);
    near(w.player.fuel, 100.0);
}

#[test]
fn arena_walls_floor_and_ceiling_stop_velocity() {
    let mut w = World::new(&PRACTICE_ARENA);
    advance(
        &mut w,
        600,
        InputCommand {
            move_x: MoveAxis::Left,
            ..Default::default()
        },
    );
    near(w.player.body.x, 0.0);
    near(w.player.velocity.x, 0.0);
    advance(&mut w, 600, right());
    near(w.player.body.x, 2400.0 - BODY_WIDTH);
    w.player.body.y = 1.0;
    w.player.velocity.y = -480.0;
    w.player.grounded = false;
    tick(&mut w, InputCommand::default(), &PRACTICE_ARENA);
    near(w.player.body.y, 0.0);
    near(w.player.velocity.y, 0.0);
    assert!(!w.player.grounded);
    advance(&mut w, 180, InputCommand::default());
    near(w.player.body.y + BODY_HEIGHT, PRACTICE_ARENA.floor_y);
    assert!(w.player.grounded);
}

#[test]
fn thin_platform_sweeps_from_all_four_sides_at_extreme_speed() {
    let solid = Rect::new(100.0, 500.0, 200.0, 1.0);
    let mut b = Rect::new(120.0, 100.0, 36.0, 68.0);
    let c = move_body(&mut b, Vec2 { x: 0.0, y: 10000.0 }, &[solid]);
    near(b.y, 432.0);
    assert!(c.grounded && c.hit_y);
    b.y = 550.0;
    let c = move_body(
        &mut b,
        Vec2 {
            x: 0.0,
            y: -10000.0,
        },
        &[solid],
    );
    near(b.y, 501.0);
    assert!(c.hit_y && !c.grounded);
    b.x = 0.0;
    b.y = 470.0;
    assert!(move_body(&mut b, Vec2 { x: 10000.0, y: 0.0 }, &[solid]).hit_x);
    near(b.x, 64.0);
    b.x = 400.0;
    assert!(
        move_body(
            &mut b,
            Vec2 {
                x: -10000.0,
                y: 0.0
            },
            &[solid]
        )
        .hit_x
    );
    near(b.x, 300.0);
}

#[test]
fn wall_slide_corner_contacts_and_geometry_order() {
    let wall = Rect::new(150.0, 100.0, 20.0, 500.0);
    let mut b = Rect::new(0.0, 0.0, 20.0, 20.0);
    let c = move_body(&mut b, Vec2 { x: 500.0, y: 500.0 }, &[wall]);
    near(b.x, 130.0);
    near(b.y, 500.0);
    assert!(c.hit_x);
    let floor = Rect::new(-100.0, 200.0, 1000.0, 20.0);
    let wall = Rect::new(200.0, -100.0, 20.0, 1000.0);
    let start = Rect::new(0.0, 0.0, 20.0, 20.0);
    let mut a = start;
    let mut b = start;
    let delta = Vec2 { x: 500.0, y: 500.0 };
    let ca = move_body(&mut a, delta, &[floor, wall]);
    let cb = move_body(&mut b, delta, &[wall, floor]);
    assert_eq!(a, b);
    assert_eq!(ca, cb);
    near(a.x, 180.0);
    near(a.y, 180.0);
    assert!(ca.hit_x && ca.hit_y && ca.grounded);
}

#[test]
fn platform_top_is_walkable_and_final_support_is_lost_at_edge() {
    let mut w = World::new(&PRACTICE_ARENA);
    w.player.body.x = 130.0;
    w.player.body.y = 892.0;
    advance(&mut w, 15, right());
    assert!(w.player.body.x > 185.0 && w.player.grounded);
    near(w.player.body.y, 892.0);
    w.player.body.x = 409.0;
    w.player.velocity.x = 320.0;
    tick(&mut w, right(), &PRACTICE_ARENA);
    assert!(!w.player.grounded);
    // A corner merely grazed while leaving must not behave like an invisible wall.
    let mut b = Rect::new(80.0, 80.0, 20.0, 20.0);
    let c = move_body(
        &mut b,
        Vec2 { x: 20.0, y: -20.0 },
        &[Rect::new(100.0, 100.0, 20.0, 20.0)],
    );
    assert!(!c.hit_x && !c.hit_y);
}

#[test]
fn ledge_grace_is_available_late_but_consumed_and_expires() {
    let mut w = World::new(&PRACTICE_ARENA);
    w.player.body.x = 409.0;
    w.player.body.y = 892.0;
    w.player.velocity.x = 320.0;
    tick(&mut w, right(), &PRACTICE_ARENA);
    let mut expired = w;
    advance(&mut w, 6, right());
    assert!(tick(&mut w, jump(), &PRACTICE_ARENA).jumped);
    assert_eq!(w.player.coyote_ticks, 0);
    assert!(!tick(&mut w, jump(), &PRACTICE_ARENA).jumped);
    advance(&mut expired, 8, right());
    assert!(!tick(&mut expired, jump(), &PRACTICE_ARENA).jumped);
    assert!(!expired.player.thrusting); // separate Space never falls back to jet
}

#[test]
fn buffered_landing_tap_survives_release_and_preserves_horizontal_speed() {
    for distance in [1.0, 30.0, 72.0] {
        let mut w = World::new(&PRACTICE_ARENA);
        w.player.body.y -= distance;
        w.player.velocity = Vec2 { x: 320.0, y: 360.0 };
        w.player.grounded = false;
        w.player.coyote_ticks = 0;
        let mut event = tick(
            &mut w,
            InputCommand {
                jump_pressed: true,
                ..right()
            },
            &PRACTICE_ARENA,
        );
        for _ in 1..BUFFER_TICKS {
            if event.jumped {
                break;
            }
            event = tick(&mut w, right(), &PRACTICE_ARENA);
        }
        assert!(event.landed && event.jumped, "distance {distance}");
        near(w.player.velocity.y, -520.0);
        near(w.player.velocity.x, 320.0);
        assert_eq!(w.player.jump_buffer_ticks, 0);
    }
}

#[test]
fn buffer_checks_the_path_and_never_turns_separate_jump_into_thrust() {
    const SOLIDS: &[Rect] = &[Rect::new(100.0, 500.0, 60.0, 20.0)];
    let map = Arena {
        solids: SOLIDS,
        ..PRACTICE_ARENA
    };
    let mut w = World::new(&map);
    w.player.body.x = 155.0;
    w.player.body.y = 380.0;
    w.player.velocity.y = 360.0;
    w.player.grounded = false;
    w.player.coyote_ticks = 0;
    let mut passing = w;
    passing.player.velocity.x = 320.0;
    tick(
        &mut passing,
        InputCommand {
            jump_pressed: true,
            ..right()
        },
        &map,
    );
    assert_eq!(passing.player.jump_buffer_ticks, 0);
    tick(&mut w, jump(), &map);
    assert!(w.player.jump_buffer_ticks > 0);
    for _ in 0..BUFFER_TICKS {
        assert!(!tick(&mut w, right(), &map).jumped);
    }
    assert_eq!(w.player.jump_buffer_ticks, 0);
    assert!(!w.player.thrusting);
    let mut early = World::new(&PRACTICE_ARENA);
    early.player.body.y -= 80.0;
    early.player.velocity.y = 360.0;
    early.player.grounded = false;
    early.player.coyote_ticks = 0;
    tick(&mut early, jump(), &PRACTICE_ARENA);
    assert_eq!(early.player.jump_buffer_ticks, 0);
}

#[test]
fn direct_jet_takes_off_consumes_grace_and_stops_on_release() {
    let mut w = World::new(&PRACTICE_ARENA);
    tick(&mut w, jet(true), &PRACTICE_ARENA);
    assert!(w.player.thrusting && !w.player.grounded);
    assert_eq!(w.player.coyote_ticks, 0);
    near(w.player.velocity.y, -35.0);
    assert!(
        !tick(
            &mut w,
            InputCommand {
                jump_pressed: true,
                ..jet(false)
            },
            &PRACTICE_ARENA
        )
        .jumped
    );
    let fuel = w.player.fuel;
    tick(&mut w, InputCommand::default(), &PRACTICE_ARENA);
    assert!(!w.player.thrusting && !w.player.thrust_latched);
    near(w.player.fuel, fuel);
    tick(&mut w, jet(false), &PRACTICE_ARENA);
    assert!(!w.player.thrusting);
    tick(&mut w, jet(true), &PRACTICE_ARENA);
    assert!(w.player.thrusting);
}

#[test]
fn jet_tapers_caps_rise_and_spends_only_the_final_partial_tick() {
    near(jet_acceleration(100.0), 3600.0);
    near(jet_acceleration(45.0), 2700.0);
    near(jet_acceleration(0.0), 1800.0);
    let mut w = World::new(&PRACTICE_ARENA);
    w.player.body.x = 1000.0;
    tick(&mut w, jet(true), &PRACTICE_ARENA);
    advance(&mut w, 40, jet(false));
    near(w.player.velocity.y, -480.0);
    w.player.body.y = 400.0;
    w.player.velocity.y = 0.0;
    w.player.fuel = FUEL_DRAIN * DT * 0.25;
    let expected = (GRAVITY - jet_acceleration(w.player.fuel / 2.0) * 0.25) * DT;
    tick(&mut w, jet(false), &PRACTICE_ARENA);
    near(w.player.velocity.y, expected);
    near(w.player.fuel, 0.0);
    assert!(!w.player.thrusting && !w.player.thrust_latched);
}

#[test]
fn exhaustion_and_landing_require_fresh_jet_press() {
    let mut w = World::new(&PRACTICE_ARENA);
    w.player.body.x = 1000.0;
    tick(&mut w, jet(true), &PRACTICE_ARENA);
    advance(&mut w, 209, jet(false));
    near(w.player.fuel, 0.0);
    assert!(!w.player.thrust_latched);
    advance(&mut w, 50, jet(false));
    assert!(w.player.fuel > 0.0 && !w.player.thrusting);
    tick(&mut w, InputCommand::default(), &PRACTICE_ARENA);
    tick(&mut w, jet(true), &PRACTICE_ARENA);
    assert!(w.player.thrusting);
    w.player.body.y = 1151.0;
    w.player.velocity.y = 500.0;
    let events = tick(&mut w, jet(false), &PRACTICE_ARENA);
    assert!(events.landed && !w.player.thrust_latched);
    advance(&mut w, 5, jet(false));
    assert!(w.player.grounded && !w.player.thrusting);
}

#[test]
fn regeneration_waits_exactly_24_ticks_then_caps_at_100() {
    let mut w = World::new(&PRACTICE_ARENA);
    tick(&mut w, jet(true), &PRACTICE_ARENA);
    let fuel = w.player.fuel;
    advance(&mut w, 23, InputCommand::default());
    near(w.player.fuel, fuel);
    tick(&mut w, InputCommand::default(), &PRACTICE_ARENA);
    near(w.player.fuel, (fuel + 0.5).min(100.0));
    advance(&mut w, 300, InputCommand::default());
    near(w.player.fuel, 100.0);
}

#[test]
fn release_cancels_buffer_and_reset_restores_all_player_fields() {
    let mut w = World::new(&PRACTICE_ARENA);
    w.player.body.y -= 50.0;
    w.player.velocity.y = 360.0;
    w.player.grounded = false;
    w.player.coyote_ticks = 0;
    tick(&mut w, jump(), &PRACTICE_ARENA);
    assert!(w.player.jump_buffer_ticks > 0);
    tick(
        &mut w,
        InputCommand {
            release_input: true,
            ..jet(true)
        },
        &PRACTICE_ARENA,
    );
    assert_eq!(w.player.jump_buffer_ticks, 0);
    assert!(!w.player.thrust_latched);
    let before_tick = w.tick;
    assert!(
        tick(
            &mut w,
            InputCommand {
                reset: true,
                ..jet(true)
            },
            &PRACTICE_ARENA
        )
        .reset
    );
    assert_eq!(w.player, World::new(&PRACTICE_ARENA).player);
    assert_eq!(w.tick, before_tick + 1);
}

#[test]
fn identical_commands_replay_exactly_on_this_machine_and_wrong_ticks_are_rejected() {
    let mut a = World::new(&PRACTICE_ARENA);
    let mut b = a;
    assert!(
        step(
            &mut a,
            InputCommand {
                tick: 1,
                ..Default::default()
            },
            &PRACTICE_ARENA
        )
        .is_err()
    );
    assert_eq!(a, b);
    let mut seed = 17_u64;
    for tick in 0..10000 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let input = InputCommand {
            tick,
            move_x: match seed % 3 {
                0 => MoveAxis::Left,
                1 => MoveAxis::Right,
                _ => MoveAxis::Idle,
            },
            jump_pressed: seed.is_multiple_of(31),
            jet_pressed: seed.is_multiple_of(47),
            jet_held: !seed.is_multiple_of(7),
            reset: tick % 971 == 970,
            release_input: false,
            ..Default::default()
        };
        assert_eq!(
            step(&mut a, input, &PRACTICE_ARENA),
            step(&mut b, input, &PRACTICE_ARENA)
        );
        assert_eq!(a, b);
        assert!(a.player.body.x >= -1e-7 && a.player.body.x + BODY_WIDTH <= 2400.0 + 1e-7);
        assert!(a.player.body.y >= -1e-7 && a.player.body.y + BODY_HEIGHT <= 1220.0 + 1e-7);
        for s in PRACTICE_ARENA.solids {
            let p = a.player.body;
            assert!(
                !(p.x + p.width > s.x + 1e-7
                    && p.x < s.x + s.width - 1e-7
                    && p.y + p.height > s.y + 1e-7
                    && p.y < s.y + s.height - 1e-7)
            );
        }
    }
}

#[test]
fn simultaneous_jump_and_direct_jet_preserve_the_initial_jump_impulse() {
    let mut w = World::new(&PRACTICE_ARENA);
    let event = tick(
        &mut w,
        InputCommand {
            jump_pressed: true,
            ..jet(true)
        },
        &PRACTICE_ARENA,
    );
    assert!(event.jumped && w.player.thrusting);
    near(w.player.velocity.y, -555.0);
    tick(&mut w, jet(false), &PRACTICE_ARENA);
    near(w.player.velocity.y, -MAX_RISE_SPEED);
}
