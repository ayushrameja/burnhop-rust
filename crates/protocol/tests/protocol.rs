use burnhop_gameplay_core::*;
use burnhop_protocol::{prediction::Prediction, *};

fn input(sequence: u64) -> NetInput {
    NetInput {
        actor: ActorId::One,
        sequence,
        command: InputCommand {
            tick: sequence,
            move_x: MoveAxis::Right,
            ..Default::default()
        },
    }
}
fn snapshot(tick: u64) -> Snapshot {
    Snapshot {
        state: MatchState {
            tick,
            actors: ActorId::ALL.map(|id| Some(Actor::new(id, 1, &PRACTICE_ARENA))),
        },
        ack: tick.saturating_sub(10),
        last_applied: 0,
        shots: [None; 2],
    }
}
fn prediction() -> Prediction {
    Prediction::new(Welcome {
        actor: ActorId::One,
        generation: 1,
        start_tick: 10,
    })
}
#[test]
fn complete_snapshot_and_inputs_round_trip_with_bounded_wire_size() {
    let mut s = snapshot(20);
    let a = s.state.actors[0].as_mut().unwrap();
    a.movement.velocity = Vec2 { x: 17., y: -40. };
    a.movement.jump_buffer_ticks = 3;
    a.movement.thrust_latched = true;
    a.movement.fuel_delay_ticks = 11;
    a.combat.reload_was_pressed = true;
    a.combat.weapons[0].ammo = 2;
    a.combat.weapons[0].reload_ticks = 51;
    a.combat.weapons[1].cooldown_ticks = 5;
    for m in [
        Message::Snapshot(s),
        Message::Inputs([Some(input(1)), Some(input(2)), Some(input(3))]),
        Message::hello(),
        Message::Welcome(Welcome {
            actor: ActorId::Two,
            generation: 89,
            start_tick: 321,
        }),
        Message::Reject(Rejection::Full),
        Message::Release { through: 17 },
    ] {
        let bytes = encode(&m);
        assert!(bytes.len() <= MAX_MESSAGE_BYTES);
        assert_eq!(decode(&bytes), Ok(m));
    }
}
#[test]
fn malformed_sizes_tags_booleans_floats_and_trailing_bytes_are_rejected() {
    assert!(decode(&vec![0; MAX_MESSAGE_BYTES + 1]).is_err());
    assert!(decode(&[255]).is_err());
    assert!(decode(&[3, 7]).is_err());
    let message = Message::Inputs([Some(input(1)), None, None]);
    let bytes = encode(&message);
    for n in 0..bytes.len() {
        assert!(decode(&bytes[..n]).is_err());
    }
    let mut extra = bytes;
    extra.push(0);
    assert!(decode(&extra).is_err());
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut bad = input(1);
        bad.command.aim_at = Some(Vec2 { x: value, y: 4. });
        assert!(decode(&encode(&Message::Inputs([Some(bad), None, None]))).is_err());
        assert!(!valid_input(&bad));
    }
    let mut state = snapshot(10);
    state.state.actors[0].as_mut().unwrap().combat.health = 500;
    assert!(decode(&encode(&Message::Snapshot(state))).is_err());
}
#[test]
fn arbitrary_byte_inputs_do_not_panic_or_allocate_from_peer_lengths() {
    let mut seed = 22_u64;
    for len in 0..=MAX_MESSAGE_BYTES {
        let bytes: Vec<u8> = (0..len)
            .map(|_| {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                (seed >> 32) as u8
            })
            .collect();
        let _ = decode(&bytes);
    }
}
#[test]
fn compatibility_requires_both_versions() {
    assert!(compatible(PROTOCOL_VERSION, GAMEPLAY_VERSION));
    assert!(!compatible(PROTOCOL_VERSION + 1, GAMEPLAY_VERSION));
    assert!(!compatible(PROTOCOL_VERSION, GAMEPLAY_VERSION + 1));
}
#[test]
fn ownership_reset_local_ticks_and_aim_limits_are_validated() {
    let mut queue = InputQueue::new(ActorId::Two, 10);
    assert_eq!(queue.admit(input(1), 10), Admission::WrongActor);
    for kind in 0..4 {
        let mut bad = input(1);
        bad.actor = ActorId::Two;
        match kind {
            0 => bad.command.reset = true,
            1 => bad.command.tick = 999,
            2 => bad.command.aim_at = Some(Vec2 { x: 100_001., y: 0. }),
            _ => bad.command.jet_pressed = true,
        }
        assert_eq!(queue.admit(bad, 10), Admission::Invalid);
    }
    assert!(queue.is_empty());
}
#[test]
fn duplicates_stale_reordered_and_far_future_commands_have_deliberate_outcomes() {
    let mut queue = InputQueue::new(ActorId::One, 10);
    assert_eq!(queue.admit(input(3), 10), Admission::Accepted);
    assert_eq!(queue.admit(input(1), 10), Admission::Accepted);
    assert_eq!(queue.admit(input(1), 10), Admission::DuplicateOrStale);
    assert_eq!(queue.admit(input(2), 10), Admission::Accepted);
    assert_eq!(queue.admit(input(1000), 10), Admission::TooFarAhead);
    assert_eq!(queue.admit(input(u64::MAX), 10), Admission::TooFarAhead);
    for tick in 10..13 {
        assert_eq!(queue.take_tick(tick).move_x, MoveAxis::Right);
    }
    assert_eq!(queue.ack, 3);
    assert_eq!(queue.last_applied, 3);
    assert_eq!(queue.admit(input(2), 13), Admission::DuplicateOrStale);
}
#[test]
fn bursts_cannot_accelerate_and_queue_and_rates_are_bounded() {
    let mut queue = InputQueue::new(ActorId::One, 0);
    for seq in 1..=12 {
        assert_eq!(queue.admit(input(seq), 0), Admission::Accepted);
    }
    assert_eq!(queue.admit(input(13), 0), Admission::RateLimited);
    assert_eq!(queue.len(), 12);
    queue.refill(1000.);
    for seq in 13..=24 {
        assert_eq!(queue.admit(input(seq), 0), Admission::Accepted);
    }
    queue.refill(1000.);
    for seq in 25..=32 {
        assert_eq!(queue.admit(input(seq), 0), Admission::Accepted);
    }
    assert_eq!(queue.len(), INPUT_WINDOW as usize);
    assert_eq!(queue.admit(input(33), 0), Admission::TooFarAhead);
    let mut world = World::new(&PRACTICE_ARENA);
    step(&mut world, queue.take_tick(0), &PRACTICE_ARENA).unwrap();
    assert!(world.player.body.x - PRACTICE_ARENA.spawn.x < 2.);
    assert_eq!(queue.len(), 31);
    assert_eq!(world.tick, 1);
}
#[test]
fn missing_input_immediately_cancels_fire_and_thrust_and_retires_ack() {
    let mut queue = InputQueue::new(ActorId::One, 0);
    let mut active = input(1);
    active.command.jet_pressed = true;
    active.command.jet_held = true;
    active.command.fire_held = true;
    queue.admit(active, 0);
    assert!(queue.take_tick(0).fire_held);
    let missing = queue.take_tick(1);
    assert!(missing.release_input);
    assert!(!missing.fire_held);
    assert!(!missing.jet_held);
    assert_eq!(queue.ack, 2);
    assert_eq!(queue.last_applied, 1);
    assert_eq!(queue.admit(input(2), 2), Admission::DuplicateOrStale);
}
#[test]
fn focus_release_is_a_barrier_to_queued_and_late_commands() {
    let mut queue = InputQueue::new(ActorId::One, 10);
    for seq in 1..5 {
        queue.admit(input(seq), 10);
    }
    assert!(queue.release(4, 10));
    assert!(queue.is_empty());
    assert_eq!(queue.admit(input(4), 10), Admission::DuplicateOrStale);
    assert!(!queue.release(1000, 10));
    assert!(queue.take_tick(10).release_input);
    assert_eq!(queue.admit(input(5), 11), Admission::Accepted);
}
#[test]
fn reconciliation_restores_complete_state_replays_only_unacknowledged_movement() {
    let mut p = prediction();
    p.reconcile(snapshot(10));
    // Initial server neutral gate is cleared by this first command.
    p.command(InputCommand::default()).unwrap();
    p.command(InputCommand {
        jet_pressed: true,
        jet_held: true,
        fire_held: true,
        aim_at: Some(BOT_SPAWN),
        ..Default::default()
    })
    .unwrap();
    let mut correction = snapshot(11);
    let a = correction.state.actors[0].as_mut().unwrap();
    a.require_neutral = false;
    a.movement.fuel = 27.;
    a.movement.fuel_delay_ticks = 13;
    a.movement.velocity.x = 31.;
    a.combat.weapons[0].ammo = 7;
    let mut expected = *a;
    predict_movement(
        &mut expected,
        InputCommand {
            tick: 2,
            jet_pressed: true,
            jet_held: true,
            fire_held: true,
            aim_at: Some(BOT_SPAWN),
            ..Default::default()
        },
        &PRACTICE_ARENA,
    );
    assert!(p.reconcile(correction).accepted);
    assert_eq!(p.local, Some(expected));
    assert_eq!(p.history_len(), 1);
    assert_eq!(p.local.unwrap().combat.weapons[0].ammo, 7);
}
#[test]
fn stale_invalid_ack_and_wrong_generation_snapshots_cannot_rewind_state() {
    let mut p = prediction();
    assert!(p.reconcile(snapshot(11)).accepted);
    for mut bad in [snapshot(10), snapshot(11), snapshot(12), snapshot(13)] {
        if bad.state.tick == 12 {
            bad.ack = 900;
        }
        if bad.state.tick == 13 {
            bad.state.actors[0].as_mut().unwrap().generation = 2;
        }
        assert!(!p.reconcile(bad).accepted);
    }
    assert_eq!(p.latest.unwrap().state.tick, 11);
}
#[test]
fn prediction_buffers_stop_growing_and_resume_after_a_stall() {
    let mut p = prediction();
    p.reconcile(snapshot(10));
    for _ in 0..500 {
        p.command(InputCommand::default());
    }
    assert!(p.history_len() <= INPUT_WINDOW as usize);
    for tick in 11..200 {
        assert!(p.reconcile(snapshot(tick)).accepted);
    }
    assert_eq!(p.snapshot_len(), SNAPSHOT_LIMIT);
    assert_eq!(p.history_len(), 0);
    assert!(p.command(InputCommand::default()).is_some());
}
#[test]
fn interpolation_handles_midpoints_gaps_join_departure_death_and_respawn() {
    let mut p = prediction();
    let a = snapshot(10);
    p.reconcile(a);
    let mut b = snapshot(14);
    b.state.actors[1].as_mut().unwrap().movement.body.x += 40.;
    p.reconcile(b);
    assert_eq!(p.remote_at(12.).unwrap().movement.body.x, BOT_SPAWN.x + 20.);
    assert_eq!(
        p.remote_at(999.).unwrap().movement.body.x,
        BOT_SPAWN.x + 40.
    );
    let mut dead = snapshot(16);
    let target = dead.state.actors[1].as_mut().unwrap();
    target.deaths = 1;
    target.combat.health = 0;
    target.combat.life = LifeState::Dead {
        remaining_ticks: 180,
    };
    p.reconcile(dead);
    assert!(!p.remote_at(15.).unwrap().combat.alive());
    let mut respawn = snapshot(196);
    respawn.state.actors[1].as_mut().unwrap().deaths = 1;
    p.reconcile(respawn);
    assert_eq!(p.remote_at(195.).unwrap().movement.body.x, BOT_SPAWN.x);
    let mut gone = snapshot(197);
    gone.state.actors[1] = None;
    p.reconcile(gone);
    assert!(p.remote_at(195.).is_none());
    let mut joined = snapshot(198);
    joined.state.actors[1].as_mut().unwrap().generation = 2;
    p.reconcile(joined);
    assert_eq!(p.remote_at(195.).unwrap().generation, 2);
}
#[test]
fn death_discards_prediction_and_confirmed_effects_are_never_duplicated() {
    let mut p = prediction();
    p.reconcile(snapshot(10));
    p.command(InputCommand::default());
    let mut dead = snapshot(11);
    let a = dead.state.actors[0].as_mut().unwrap();
    a.deaths = 1;
    a.combat.health = 0;
    a.combat.life = LifeState::Dead {
        remaining_ticks: 180,
    };
    let shot = Shot {
        shooter: ActorId::Two,
        weapon: WeaponId::Pistol,
        origin: BOT_SPAWN,
        end: PRACTICE_ARENA.spawn,
        impact: Impact::Body(ActorId::One),
        damage: 10,
    };
    dead.shots[1] = Some(ConfirmedShot { tick: 10, shot });
    let result = p.reconcile(dead);
    assert!(result.life_changed);
    assert_eq!(result.shots, vec![shot]);
    assert_eq!(p.history_len(), 0);
    dead.state.tick = 12;
    dead.ack = 2;
    assert!(p.reconcile(dead).shots.is_empty());
    let result = p.reconcile(snapshot(191));
    assert!(result.life_changed);
    assert!(p.local.unwrap().combat.alive());
}
