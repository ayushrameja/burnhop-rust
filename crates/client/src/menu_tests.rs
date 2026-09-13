use super::*;
use crate::adapter::Key;
use burnhop_gameplay_core::{DT, MoveAxis};
use std::{
    net::UdpSocket,
    thread,
    time::{Duration, Instant},
};
fn event(code: KeyCode, text: Option<&str>) -> KeyboardInput {
    KeyboardInput {
        key_code: code,
        logical_key: bevy::input::keyboard::Key::Unidentified(
            bevy::input::keyboard::NativeKey::Unidentified,
        ),
        state: ButtonState::Pressed,
        text: text.map(Into::into),
        repeat: false,
        window: Entity::PLACEHOLDER,
    }
}
#[test]
fn numeric_address_validation_and_editor_selection_caret_deletion() {
    for text in ["127.0.0.1:5000", "[::1]:5000", "[fe80::1%3]:5000"] {
        assert!(address(text).is_ok(), "{text}");
    }
    for text in [
        "localhost:5000",
        "127.0.0.1:0",
        "0.0.0.0:5000",
        "[::]:5000",
        "224.0.0.1:5000",
        "1.2.3.4:65536",
    ] {
        assert!(address(text).is_err(), "{text}");
    }
    let mut editor = Editor::new("127.0.0.1:5000");
    editor.key(&event(KeyCode::Backspace, None), false);
    editor.key(&event(KeyCode::Digit1, Some("1")), false);
    assert_eq!(editor.text, "127.0.0.1:5001");
    editor.key(&event(KeyCode::Home, None), false);
    editor.key(&event(KeyCode::Delete, None), false);
    assert_eq!(editor.text, "27.0.0.1:5001");
    editor.key(&event(KeyCode::KeyA, None), true);
    editor.key(&event(KeyCode::BracketLeft, Some("[::1]:5000")), false);
    assert_eq!(editor.text, "[::1]:5000");
    editor.key(&event(KeyCode::KeyZ, Some("🦀")), false);
    assert_eq!(editor.text, "[::1]:5000");
    for _ in 0..100 {
        editor.key(&event(KeyCode::Digit1, Some("1")), false);
    }
    assert_eq!(editor.text.len(), 64);
    assert!(editor.display(true).len() < 50);
}
#[test]
fn repeated_practice_sessions_reset_scores_input_clock_effect_epoch() {
    let mut game = Playground::default();
    for _ in 0..16 {
        act(&mut game, Action::Practice);
        assert_eq!(game.menu.screen, Screen::Playing);
        assert_eq!(game.world.tick, 0);
        assert_eq!(game.combat.kills, 0);
        game.combat.kills = 8;
        game.world.tick = 600;
        game.input.push(Key::Fire, true);
        game.input.push(Key::Right, true);
        game.input.command(0);
        let epoch = game.feedback.epoch;
        change(&mut game, Screen::Paused);
        assert!(game.input.command(1).release_input);
        assert_eq!(game.input.command(2).move_x, MoveAxis::Idle);
        act(&mut game, Action::Leave);
        assert_eq!(game.menu.screen, Screen::Main);
        assert!(game.online.is_none() && game.menu.owned.is_none());
        assert!(game.feedback.epoch > epoch);
    }
}
#[test]
fn offline_pause_uses_real_simulation_system_without_advancing_or_catching_up() {
    let mut app = App::new();
    app.insert_resource(Playground::default())
        .insert_resource(Time::<Real>::default())
        .add_systems(Update, crate::simulate);
    app.world_mut()
        .resource_mut::<Time<Real>>()
        .advance_by(Duration::from_secs_f64(DT));
    app.update();
    assert_eq!(app.world().resource::<Playground>().world.tick, 1);
    change(
        &mut app.world_mut().resource_mut::<Playground>(),
        Screen::Paused,
    );
    app.world_mut()
        .resource_mut::<Time<Real>>()
        .advance_by(Duration::from_secs(30));
    app.update();
    assert_eq!(app.world().resource::<Playground>().world.tick, 1);
    act(
        &mut app.world_mut().resource_mut::<Playground>(),
        Action::Resume,
    );
    app.world_mut()
        .resource_mut::<Time<Real>>()
        .advance_by(Duration::from_secs_f64(DT));
    app.update();
    assert_eq!(app.world().resource::<Playground>().world.tick, 2);
}
#[test]
fn failed_bind_retry_and_cancel_clean_partial_hosting() {
    let occupied = UdpSocket::bind("127.0.0.1:0").unwrap();
    let addr = occupied.local_addr().unwrap();
    let mut game = Playground::default();
    game.menu.editor = Editor::new(&addr.to_string());
    act(&mut game, Action::StartHost);
    assert_eq!(game.menu.screen, Screen::Error);
    assert!(game.menu.notice.contains("another port"));
    assert!(game.menu.owned.is_none() && game.online.is_none());
    drop(occupied);
    act(&mut game, Action::StartHost);
    assert_eq!(game.menu.screen, Screen::Connecting);
    act(&mut game, Action::Cancel);
    assert!(game.menu.owned.is_none() && game.online.is_none());
    drop(UdpSocket::bind(addr).unwrap());
    act(&mut game, Action::Connect);
    act(&mut game, Action::Cancel);
    act(&mut game, Action::StartHost);
    assert_eq!(game.menu.screen, Screen::Connecting);
    // Simulate an established/partially connected client failing; owned port must release.
    game.online.as_mut().unwrap().network.close("test failure");
    poll(&mut game);
    assert_eq!(game.menu.screen, Screen::Error);
    assert!(game.menu.owned.is_none());
    drop(UdpSocket::bind(addr).unwrap());
}
fn drive(games: &mut [&mut Playground], seconds: f64) {
    let end = Instant::now() + Duration::from_secs_f64(seconds);
    let mut last = Instant::now();
    while Instant::now() < end {
        let now = Instant::now();
        let dt = now.duration_since(last).as_secs_f64();
        last = now;
        for game in games.iter_mut() {
            game.input_blocked = game.menu.screen != Screen::Playing;
            if game.online.is_some() {
                crate::online::simulate(game, dt);
            }
            poll(game);
        }
        thread::sleep(Duration::from_millis(2));
    }
}
#[test]
fn live_owned_host_guest_leave_neutral_pause_stop_disconnect_and_same_port_rehost() {
    let reservation = UdpSocket::bind("127.0.0.1:0").unwrap();
    let addr = reservation.local_addr().unwrap();
    drop(reservation);
    let mut host = Playground::default();
    let mut guest = Playground::default();
    host.menu.editor = Editor::new(&addr.to_string());
    guest.menu.editor = Editor::new(&addr.to_string());
    act(&mut host, Action::StartHost);
    act(&mut guest, Action::Connect);
    drive(&mut [&mut host, &mut guest], 0.8);
    assert_eq!(host.menu.screen, Screen::Playing);
    assert_eq!(guest.menu.screen, Screen::Playing);
    assert_eq!(
        host.online
            .as_ref()
            .unwrap()
            .actors
            .iter()
            .flatten()
            .count(),
        2
    );
    guest.input.push(Key::Left, true);
    guest.input.push(Key::Fire, true);
    drive(&mut [&mut host, &mut guest], 0.3);
    let tick = guest.world.tick;
    change(&mut guest, Screen::Paused);
    drive(&mut [&mut host, &mut guest], 0.5);
    assert!(guest.world.tick > tick + 15);
    assert_eq!(guest.world.player.velocity.x, 0.);
    let ammo = guest.combat.player.weapon().ammo;
    drive(&mut [&mut host, &mut guest], 0.2);
    assert_eq!(guest.combat.player.weapon().ammo, ammo);
    act(&mut guest, Action::Leave);
    drive(&mut [&mut host], 0.3);
    assert_eq!(
        host.online
            .as_ref()
            .unwrap()
            .actors
            .iter()
            .flatten()
            .count(),
        1
    );
    assert!(host.menu.owned.is_some());
    act(&mut guest, Action::Connect);
    drive(&mut [&mut host, &mut guest], 0.6);
    assert_eq!(guest.menu.screen, Screen::Playing);
    assert_eq!(guest.combat.kills, 0);
    act(&mut host, Action::Leave);
    assert_eq!(host.menu.screen, Screen::ConfirmStop);
    assert!(host.menu.owned.is_some());
    act(&mut host, Action::Resume);
    assert!(host.menu.owned.is_some());
    act(&mut host, Action::Leave);
    act(&mut host, Action::ConfirmStop);
    drive(&mut [&mut guest], 0.5);
    assert_eq!(guest.menu.screen, Screen::Error);
    assert!(guest.online.is_none());
    act(&mut host, Action::StartHost);
    drive(&mut [&mut host], 0.6);
    assert_eq!(host.menu.screen, Screen::Playing);
    assert_eq!(
        host.online
            .as_ref()
            .unwrap()
            .actors
            .iter()
            .flatten()
            .count(),
        1
    );
    drop(host);
    drop(UdpSocket::bind(addr).unwrap());
}
#[test]
fn rejection_copy_is_actionable_and_cancel_allows_a_new_attempt() {
    assert!(
        connection_message(&ConnectionState::CompatibilityError).contains("same Burnhop build")
    );
    assert!(
        connection_message(&ConnectionState::Disconnected(
            "server rejected: Full".into()
        ))
        .contains("8 players")
    );
    assert!(
        connection_message(&ConnectionState::Disconnected(
            "no authoritative snapshot for five seconds".into()
        ))
        .contains("timed out")
    );
    let mut game = Playground::default();
    for _ in 0..10 {
        act(&mut game, Action::Connect);
        assert!(game.online.is_some());
        act(&mut game, Action::Cancel);
        assert!(game.online.is_none());
    }
}
#[test]
fn native_menu_navigation_disabled_controls_and_gameplay_event_isolation() {
    use bevy::{input::mouse::MouseButtonInput, window::WindowFocused};
    let mut app = App::new();
    let mut game = Playground::default();
    game.menu.screen = Screen::Main;
    app.insert_resource(game)
        .init_resource::<ButtonInput<KeyCode>>()
        .add_message::<KeyboardInput>()
        .add_message::<MouseButtonInput>()
        .add_message::<WindowFocused>()
        .add_message::<AppExit>()
        .add_systems(Startup, setup)
        .add_systems(Update, (input, crate::capture_input).chain());
    let window = app
        .world_mut()
        .spawn((
            Window {
                focused: true,
                ..default()
            },
            PrimaryWindow,
        ))
        .id();
    let send = |app: &mut App, code, text| {
        let mut key = event(code, text);
        key.window = window;
        app.world_mut().write_message(key);
        app.update();
    };
    app.update();
    let hover = app
        .world_mut()
        .query::<(Entity, &Control)>()
        .iter(app.world())
        .find(|(_, c)| c.0 == 3)
        .unwrap()
        .0;
    *app.world_mut().get_mut::<Interaction>(hover).unwrap() = Interaction::Hovered;
    app.update();
    assert_eq!(
        app.world().resource::<Playground>().menu.selected,
        0,
        "stationary hover after reflow must not steal keyboard focus"
    );
    send(&mut app, KeyCode::Tab, None);
    send(&mut app, KeyCode::Enter, None);
    assert_eq!(
        app.world().resource::<Playground>().menu.screen,
        Screen::Host
    );
    send(&mut app, KeyCode::Backspace, None);
    assert!(
        app.world()
            .resource::<Playground>()
            .menu
            .editor
            .text
            .is_empty()
    );
    send(&mut app, KeyCode::Tab, None);
    assert_eq!(
        app.world().resource::<Playground>().menu.selected,
        2,
        "disabled Start Hosting must be skipped"
    );
    send(&mut app, KeyCode::Escape, None);
    send(&mut app, KeyCode::Space, Some(" "));
    assert_eq!(
        app.world().resource::<Playground>().menu.screen,
        Screen::Playing
    );
    let cmd = app
        .world_mut()
        .resource_mut::<Playground>()
        .input
        .command(0);
    assert!(
        !cmd.jump_pressed && cmd.release_input,
        "menu Space must not jump"
    );
    send(&mut app, KeyCode::KeyD, Some("d"));
    assert_eq!(
        app.world_mut()
            .resource_mut::<Playground>()
            .input
            .command(1)
            .move_x,
        MoveAxis::Right
    );
    send(&mut app, KeyCode::Escape, None);
    let button = app
        .world_mut()
        .query::<(Entity, &Control)>()
        .iter(app.world())
        .find(|(_, c)| c.0 == 0)
        .unwrap()
        .0;
    *app.world_mut().get_mut::<Interaction>(button).unwrap() = Interaction::Pressed;
    app.world_mut().write_message(MouseButtonInput {
        button: MouseButton::Left,
        state: ButtonState::Pressed,
        window,
    });
    app.update();
    let mut game = app.world_mut().resource_mut::<Playground>();
    assert_eq!(game.menu.screen, Screen::Playing);
    assert!(game.input.command(2).release_input);
    let cmd = game.input.command(3);
    assert!(
        !cmd.fire_held && cmd.move_x == MoveAxis::Idle,
        "Resume click must not fire or resume held D"
    );
}
#[test]
fn menu_entity_pool_is_constant_through_repeated_screen_and_session_transitions() {
    let mut app = App::new();
    app.insert_resource(Playground::default())
        .add_systems(Startup, setup)
        .add_systems(Update, present);
    app.world_mut().spawn((
        Window {
            resolution: (480, 320).into(),
            ..default()
        },
        PrimaryWindow,
    ));
    app.update();
    let count = app.world().entities().len();
    for _ in 0..12 {
        for action in [
            Action::Practice,
            Action::Leave,
            Action::Host,
            Action::Back,
            Action::Join,
            Action::Back,
        ] {
            act(&mut app.world_mut().resource_mut::<Playground>(), action);
            app.update();
            assert_eq!(app.world().entities().len(), count);
        }
    }
}
