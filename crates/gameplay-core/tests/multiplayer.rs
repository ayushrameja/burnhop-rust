use burnhop_gameplay_core::*;
fn encounter() -> MatchState {
    MatchState {
        tick: 0,
        actors: ActorId::ALL.map(|id| (id.index() < 2).then(|| Actor::new(id, 1, &PRACTICE_ARENA))),
    }
}
fn commands(state: &MatchState) -> [InputCommand; 2] {
    [InputCommand {
        tick: state.tick,
        ..Default::default()
    }; 2]
}
#[test]
fn both_human_actors_use_approved_movement_and_cannot_reset_the_match() {
    let mut state = encounter();
    step_match(&mut state, [InputCommand::default(); 2], &PRACTICE_ARENA).unwrap();
    let mut worlds = [state.actors[0], state.actors[1]].map(|a| World {
        tick: 1,
        player: a.unwrap().movement,
    });
    for _ in 0..90 {
        let inputs = [
            InputCommand {
                tick: state.tick,
                move_x: MoveAxis::Right,
                reset: true,
                ..Default::default()
            },
            InputCommand {
                tick: state.tick,
                move_x: MoveAxis::Left,
                ..Default::default()
            },
        ];
        step_match(&mut state, inputs, &PRACTICE_ARENA).unwrap();
        for i in 0..2 {
            step(
                &mut worlds[i],
                InputCommand {
                    reset: false,
                    ..inputs[i]
                },
                &PRACTICE_ARENA,
            )
            .unwrap();
            assert_eq!(state.actors[i].unwrap().movement, worlds[i].player);
        }
    }
}
#[test]
fn simultaneous_human_shots_kill_both_then_respawn_with_full_state() {
    let mut state = encounter();
    for actor in state.actors.iter_mut().flatten() {
        actor.require_neutral = false;
        actor.combat.health = 1;
    }
    let mut inputs = commands(&state);
    for id in [ActorId::One, ActorId::Two] {
        inputs[id.index()].fire_held = true;
        inputs[id.index()].aim_at = Some(body_center(
            state.actors[ActorId::ALL[id.index() ^ 1].index()]
                .unwrap()
                .movement
                .body,
        ));
    }
    let events = step_match(&mut state, inputs, &PRACTICE_ARENA).unwrap();
    assert_eq!(events.died[..2], [true; 2]);
    assert!(events.shots[..2].iter().all(Option::is_some));
    for _ in 1..RESPAWN_TICKS {
        let inputs = commands(&state);
        step_match(&mut state, inputs, &PRACTICE_ARENA).unwrap();
        assert!(state.actors.iter().flatten().all(|a| !a.combat.alive()));
    }
    let inputs = commands(&state);
    let events = step_match(&mut state, inputs, &PRACTICE_ARENA).unwrap();
    assert_eq!(events.respawned[..2], [true; 2]);
    for a in state.actors.iter().flatten() {
        assert_eq!(a.combat.health, 100);
        assert_eq!(a.combat.weapons[0].ammo, 12);
        assert_eq!(a.movement.fuel, MAX_FUEL);
        assert_eq!((a.kills, a.deaths), (1, 1));
    }
}
#[test]
fn absent_actor_never_fires_or_absorbs_hits_and_wrong_ticks_are_atomic() {
    let mut state = encounter();
    state.actors[1] = None;
    let before = state;
    let mut inputs = commands(&state);
    inputs[0].tick = 12;
    assert!(step_match(&mut state, inputs, &PRACTICE_ARENA).is_err());
    assert_eq!(state, before);
    state.actors[0].as_mut().unwrap().require_neutral = false;
    inputs[0] = InputCommand {
        aim_at: Some(BOT_SPAWN),
        fire_held: true,
        ..Default::default()
    };
    let events = step_match(&mut state, inputs, &PRACTICE_ARENA).unwrap();
    assert!(events.shots[1].is_none());
    assert!(!matches!(events.shots[0].unwrap().impact, Impact::Body(_)));
}
#[test]
fn neutral_missing_input_cancels_human_fire_jet_without_pausing_other_actor() {
    let mut state = encounter();
    for a in state.actors.iter_mut().flatten() {
        a.require_neutral = false;
    }
    let inputs = [
        InputCommand {
            jet_pressed: true,
            jet_held: true,
            fire_held: true,
            aim_at: Some(BOT_SPAWN),
            ..Default::default()
        },
        InputCommand {
            move_x: MoveAxis::Left,
            ..Default::default()
        },
    ];
    step_match(&mut state, inputs, &PRACTICE_ARENA).unwrap();
    let before = state.actors[1].unwrap().movement.body.x;
    let events = step_match(
        &mut state,
        [
            InputCommand {
                tick: 1,
                release_input: true,
                ..Default::default()
            },
            InputCommand {
                tick: 1,
                move_x: MoveAxis::Left,
                ..Default::default()
            },
        ],
        &PRACTICE_ARENA,
    )
    .unwrap();
    assert!(!state.actors[0].unwrap().movement.thrust_latched);
    assert!(events.shots[0].is_none());
    assert!(state.actors[1].unwrap().movement.body.x < before);
}

// Preserve the original two-human scenarios inside the expanded shared match.
fn step_match(
    state: &mut MatchState,
    pair: [InputCommand; 2],
    arena: &Arena,
) -> Result<MatchEvents, TickMismatch> {
    let mut inputs = [InputCommand {
        tick: state.tick,
        ..Default::default()
    }; MAX_PLAYERS];
    inputs[..2].copy_from_slice(&pair);
    burnhop_gameplay_core::step_match(state, inputs, arena)
}
