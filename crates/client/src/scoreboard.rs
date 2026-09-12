//! Held Tab only. Actual stable slot/generation identity, authoritative scores.
use crate::{
    Playground,
    artwork::{CREAM, color},
};
use bevy::{prelude::*, window::PrimaryWindow};
#[derive(Component)]
pub struct Scoreboard;
pub fn setup(mut commands: Commands) {
    commands.spawn((
        Scoreboard,
        Text::new(""),
        TextFont {
            font_size: 14.0.into(),
            ..default()
        },
        TextColor(color(CREAM)),
        Node {
            position_type: PositionType::Absolute,
            left: percent(50),
            top: px(48),
            width: px(320),
            margin: UiRect {
                left: px(-160),
                ..default()
            },
            padding: UiRect::all(px(12)),
            display: Display::None,
            ..default()
        },
        BackgroundColor(Color::srgba(0.10, 0.15, 0.14, 0.96)),
        GlobalZIndex(30),
    ));
}
pub fn rows(
    state: &burnhop_gameplay_core::MatchState,
    local: burnhop_gameplay_core::ActorId,
) -> String {
    let mut text = String::from("PLAYER          KILLS  DEATHS\n");
    for actor in state.actors.iter().flatten() {
        text.push_str(&format!(
            "P{:<1} {:<5}        {:>4}   {:>4}\n",
            actor.id.index() + 1,
            if actor.id == local { "YOU" } else { "" },
            actor.kills,
            actor.deaths
        ));
    }
    text.push_str("\nTab: release to close");
    text
}
pub fn present(
    game: Res<Playground>,
    keys: Res<ButtonInput<KeyCode>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut board: Single<(&mut Text, &mut Node), With<Scoreboard>>,
) {
    board.1.display = Display::None;
    // Explicit renderer inspection only; ordinary Tab still releases on focus loss.
    let scripted_tab = std::env::var_os("BURNHOP_REVIEW_TAB").is_some();
    if (scripted_tab || (window.focused && keys.pressed(KeyCode::Tab)))
        && let Some(online) = &game.online
        && !online.network.status.terminal()
        && let Some(snapshot) = online.prediction.as_ref().and_then(|p| p.latest)
    {
        board.0.0 = rows(&snapshot.state, game.local_id());
        board.1.display = Display::Flex;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use burnhop_gameplay_core::*;
    #[test]
    fn native_tab_hold_release_focus_loss_and_disconnect_preserve_input() {
        use burnhop_protocol::{
            Snapshot, Welcome, prediction::Prediction, transport::ConnectionState,
        };
        let mut game = Playground::default();
        let welcome = Welcome {
            actor: ActorId::Eight,
            generation: 1,
            start_tick: 10,
        };
        let state = MatchState {
            tick: 10,
            actors: ActorId::ALL.map(|id| Some(Actor::new(id, 1, &PRACTICE_ARENA))),
        };
        let mut online = crate::online::Online::new("127.0.0.1:9".parse().unwrap(), false).unwrap();
        let mut prediction = Prediction::new(welcome);
        prediction.reconcile(Snapshot {
            state,
            ack: 0,
            last_applied: 0,
            shots: [None; MAX_PLAYERS],
        });
        online.prediction = Some(prediction);
        online.network.welcome = Some(welcome);
        online.network.status = ConnectionState::Connected;
        game.online = Some(online);
        game.input.push(crate::adapter::Key::Right, true);
        let mut app = App::new();
        app.insert_resource(game)
            .init_resource::<ButtonInput<KeyCode>>()
            .add_systems(Startup, setup)
            .add_systems(Update, present);
        let window = app
            .world_mut()
            .spawn((
                Window {
                    resolution: (480, 320).into(),
                    focused: true,
                    ..default()
                },
                PrimaryWindow,
            ))
            .id();
        app.update();
        let shown = |app: &mut App| {
            app.world_mut()
                .query_filtered::<&Node, With<Scoreboard>>()
                .single(app.world())
                .unwrap()
                .display
                == Display::Flex
        };
        assert!(!shown(&mut app));
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Tab);
        app.update();
        assert!(shown(&mut app));
        assert!(
            app.world_mut()
                .query_filtered::<&Text, With<Scoreboard>>()
                .single(app.world())
                .unwrap()
                .0
                .contains("P8 YOU")
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(KeyCode::Tab);
        app.update();
        assert!(!shown(&mut app));
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Tab);
        app.world_mut()
            .entity_mut(window)
            .get_mut::<Window>()
            .unwrap()
            .focused = false;
        app.update();
        assert!(!shown(&mut app));
        app.world_mut()
            .entity_mut(window)
            .get_mut::<Window>()
            .unwrap()
            .focused = true;
        app.world_mut()
            .resource_mut::<Playground>()
            .online
            .as_mut()
            .unwrap()
            .network
            .status = ConnectionState::Disconnected("test".into());
        app.update();
        assert!(!shown(&mut app));
        assert_eq!(
            app.world_mut()
                .resource_mut::<Playground>()
                .input
                .command(1)
                .move_x,
            MoveAxis::Right
        );
    }
    #[test]
    fn scoreboard_reports_all_slots_and_removes_departures_and_old_scores() {
        let mut state = MatchState {
            actors: ActorId::ALL.map(|id| Some(Actor::new(id, 1, &PRACTICE_ARENA))),
            ..Default::default()
        };
        state.actors[7].as_mut().unwrap().kills = 12;
        let text = rows(&state, ActorId::Eight);
        assert!(text.contains("P8 YOU") && text.contains("12"));
        assert_eq!(
            text.lines()
                .filter(|s| s.starts_with('P') && !s.starts_with("PLAYER"))
                .count(),
            8
        );
        state.actors[7] = None;
        assert!(!rows(&state, ActorId::One).contains("P8"));
        state.actors[7] = Some(Actor::new(ActorId::Eight, 2, &PRACTICE_ARENA));
        assert!(!rows(&state, ActorId::One).contains("12"));
    }
}
