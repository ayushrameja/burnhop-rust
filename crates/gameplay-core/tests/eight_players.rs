use burnhop_gameplay_core::*;
fn commands(tick: u64) -> [InputCommand; MAX_PLAYERS] {
    [InputCommand {
        tick,
        ..Default::default()
    }; MAX_PLAYERS]
}
#[test]
fn eight_candidates_have_full_body_clearance_and_support_and_joins_separate() {
    let mut state = MatchState::default();
    for point in SPAWN_CANDIDATES {
        assert!(valid_spawn(point, &PRACTICE_ARENA), "{point:?}");
    }
    for point in [
        Vec2 { x: -1., y: 1152. },
        Vec2 { x: 2350., y: 1151. },
        Vec2 { x: 210., y: 920. },
        Vec2 { x: 210., y: 890. },
    ] {
        assert!(!valid_spawn(point, &PRACTICE_ARENA));
    }
    for id in ActorId::ALL {
        let spawn = select_spawn(&state, id, &PRACTICE_ARENA);
        assert!(
            !state
                .actors
                .iter()
                .flatten()
                .any(|a| a.movement.body.x == spawn.x && a.movement.body.y == spawn.y)
        );
        let mut actor = Actor::new(id, 1, &PRACTICE_ARENA);
        actor.movement = World::new(&Arena {
            spawn,
            ..PRACTICE_ARENA
        })
        .player;
        state.actors[id.index()] = Some(actor);
    }
    assert_eq!(state.actors.iter().flatten().count(), 8);
    assert!(ActorId::from_index(8).is_none());
    assert!(ActorId::from_index(usize::MAX).is_none());
    // The scoring policy is deterministic even if a custom test body contests all candidates.
    state.actors[0].as_mut().unwrap().movement.body = Rect::new(0., 0., 2400., 1350.);
    let spawn = select_spawn(&state, ActorId::Eight, &PRACTICE_ARENA);
    assert!(valid_spawn(spawn, &PRACTICE_ARENA));
    assert_eq!(spawn, select_spawn(&state, ActorId::Eight, &PRACTICE_ARENA));
}
#[test]
fn eight_simultaneous_shooters_each_die_once_and_respawn_with_fresh_inventory() {
    let mut state = MatchState {
        actors: ActorId::ALL.map(|id| {
            let mut actor = Actor::new(id, id.index() as u64 + 1, &PRACTICE_ARENA);
            actor.movement.body = Rect::new(
                300. + id.index() as f64 * 180.,
                1152.,
                BODY_WIDTH,
                BODY_HEIGHT,
            );
            actor.require_neutral = false;
            actor.combat.health = 1;
            Some(actor)
        }),
        ..Default::default()
    };
    let mut inputs = commands(0);
    for id in ActorId::ALL {
        inputs[id.index()].fire_held = true;
        inputs[id.index()].aim_at = Some(body_center(
            state.actors[id.index() ^ 1].unwrap().movement.body,
        ));
    }
    let events = step_match(&mut state, inputs, &PRACTICE_ARENA).unwrap();
    assert_eq!(events.died, [true; MAX_PLAYERS]);
    assert!(events.shots.iter().all(Option::is_some));
    for _ in 1..RESPAWN_TICKS {
        let inputs = commands(state.tick);
        step_match(&mut state, inputs, &PRACTICE_ARENA).unwrap();
        assert!(
            state
                .actors
                .iter()
                .flatten()
                .all(|a| !a.combat.alive() && a.kills == 1 && a.deaths == 1)
        );
    }
    let inputs = commands(state.tick);
    let events = step_match(&mut state, inputs, &PRACTICE_ARENA).unwrap();
    assert_eq!(events.respawned, [true; MAX_PLAYERS]);
    for a in state.actors.iter().flatten() {
        assert_eq!(a.combat, Combatant::new(false));
        assert!(valid_spawn(
            Vec2 {
                x: a.movement.body.x,
                y: a.movement.body.y
            },
            &PRACTICE_ARENA
        ));
        assert_eq!(a.movement.fuel, MAX_FUEL);
        assert_eq!((a.kills, a.deaths), (1, 1));
    }
}
#[test]
fn multiple_attackers_share_tick_bodies_but_cannot_duplicate_kills() {
    let mut state = MatchState::default();
    for id in [ActorId::One, ActorId::Four, ActorId::Eight] {
        let mut actor = Actor::new(id, 1, &PRACTICE_ARENA);
        actor.movement.body = Rect::new(
            if id == ActorId::One {
                300.
            } else if id == ActorId::Four {
                500.
            } else {
                700.
            },
            1152.,
            BODY_WIDTH,
            BODY_HEIGHT,
        );
        actor.require_neutral = false;
        state.actors[id.index()] = Some(actor);
    }
    state.actors[3].as_mut().unwrap().combat.health = 1;
    let mut inputs = commands(0);
    for i in [0, 7] {
        inputs[i].fire_held = true;
        inputs[i].aim_at = Some(body_center(state.actors[3].unwrap().movement.body));
    }
    let events = step_match(&mut state, inputs, &PRACTICE_ARENA).unwrap();
    assert_eq!(events.shots[0].unwrap().impact, Impact::Body(ActorId::Four));
    assert_eq!(events.shots[7].unwrap().impact, Impact::Body(ActorId::Four));
    assert_eq!(events.shots[0].unwrap().damage, 1);
    assert_eq!(events.shots[7].unwrap().damage, 0);
    assert_eq!(state.actors[3].unwrap().deaths, 1);
    assert_eq!(state.actors[0].unwrap().kills, 1);
    assert_eq!(state.actors[7].unwrap().kills, 0);
    let inputs = commands(state.tick);
    step_match(&mut state, inputs, &PRACTICE_ARENA).unwrap();
    assert_eq!(state.actors[3].unwrap().deaths, 1);
}
