use super::adapter::*;
use burnhop_gameplay_core::{DT, InputCommand, MoveAxis, PRACTICE_ARENA, World, step};

#[test]
fn taps_survive_frames_without_ticks_and_edges_do_not_repeat_during_catchup() {
    for key in [Key::Jump, Key::JetLeft, Key::Right] {
        let mut input = InputBuffer::default();
        input.push(key, true);
        input.push(key, false);
        let mut clock = FrameClock::default();
        assert_eq!(clock.advance(DT / 4.0).0, 0);
        let first = input.command(0);
        match key {
            Key::Jump => assert!(first.jump_pressed),
            Key::JetLeft => assert!(first.jet_pressed && first.jet_held),
            _ => assert_eq!(first.move_x, MoveAxis::Right),
        }
        for tick in 1..5 {
            assert_eq!(
                input.command(tick),
                InputCommand {
                    tick,
                    ..Default::default()
                }
            );
        }
    }
}

#[test]
fn rapid_distinct_taps_stay_ordered_and_opposite_directions_cancel() {
    let mut input = InputBuffer::default();
    for _ in 0..2 {
        input.push(Key::Jump, true);
        input.push(Key::Jump, false);
    }
    assert!(input.command(0).jump_pressed);
    assert!(input.command(1).jump_pressed);
    assert!(!input.command(2).jump_pressed);
    input.push(Key::Left, true);
    input.push(Key::Right, true);
    assert_eq!(input.command(3).move_x, MoveAxis::Idle);
    input.push(Key::Left, false);
    assert_eq!(input.command(4).move_x, MoveAxis::Right);
}

#[test]
fn both_shift_keys_form_one_jet_hold_and_repeats_cannot_reactivate() {
    let mut input = InputBuffer::default();
    input.push(Key::JetLeft, true);
    assert!(input.command(0).jet_pressed);
    input.push(Key::JetRight, true);
    input.push(Key::JetLeft, false);
    let c = input.command(1);
    assert!(c.jet_held && !c.jet_pressed);
    input.push(Key::JetRight, true);
    assert!(!input.command(2).jet_pressed);
    input.push(Key::JetRight, false);
    assert!(!input.command(3).jet_held);
}

#[test]
fn focus_clear_cancels_held_pending_and_requires_new_press() {
    let mut input = InputBuffer::default();
    input.push(Key::Right, true);
    input.push(Key::JetLeft, true);
    input.push(Key::Jump, true);
    input.command(0);
    input.clear();
    let cleared = input.command(1);
    assert!(cleared.release_input && !cleared.jet_held && !cleared.jump_pressed);
    assert_eq!(cleared.move_x, MoveAxis::Idle);
    assert_eq!(
        input.command(2),
        InputCommand {
            tick: 2,
            ..Default::default()
        }
    );
    input.push(Key::JetLeft, true);
    assert!(input.command(3).jet_pressed);
}

#[test]
fn reset_discards_held_keys_and_pending_actions() {
    let mut input = InputBuffer::default();
    input.push(Key::Right, true);
    input.push(Key::Reset, true);
    input.push(Key::JetLeft, true);
    assert!(input.command(0).reset);
    assert!(input.command(1).release_input);
    assert_eq!(input.command(2).move_x, MoveAxis::Idle);
    assert!(!input.command(3).jet_held);
}

#[test]
fn long_gaps_are_bounded_and_do_not_leave_backlog() {
    let mut clock = FrameClock::default();
    let (ticks, alpha) = clock.advance(30.0);
    assert_eq!(ticks, 5);
    assert!((0.0..=1.0).contains(&alpha));
    assert_eq!(clock.advance(0.0).0, 0);
    assert_eq!(clock.advance(-1.0).0, 0);
    assert_eq!(clock.advance(f64::NAN).0, 0);
    clock.reset();
    assert_eq!(clock.advance(DT).0, 1);
}

#[test]
fn presentation_schedules_feed_equivalent_commands_for_the_same_event_stream() {
    // Edge times are placed at shared frame boundaries (whole seconds). Arbitrary
    // OS event timestamps quantize to the next frame/tick, not guaranteed parity.
    fn replay(schedule: &[f64]) -> (Vec<InputCommand>, World) {
        let mut clock = FrameClock::default();
        let mut input = InputBuffer::default();
        let mut world = World::new(&PRACTICE_ARENA);
        let mut commands = Vec::new();
        for second in 0..4 {
            match second {
                0 => input.push(Key::Right, true),
                1 => {
                    input.push(Key::Jump, true);
                    input.push(Key::Jump, false);
                }
                2 => input.push(Key::JetLeft, true),
                _ => {
                    input.push(Key::Right, false);
                    input.push(Key::JetLeft, false);
                }
            }
            for &elapsed in schedule {
                for _ in 0..clock.advance(elapsed).0 {
                    let command = input.command(world.tick);
                    step(&mut world, command, &PRACTICE_ARENA).unwrap();
                    commands.push(command);
                }
            }
        }
        (commands, world)
    }
    let expected = replay(&[1.0 / 60.0; 60]);
    assert_eq!(expected.0.len(), 240);
    for hz in [30, 144] {
        assert_eq!(expected, replay(&vec![1.0 / hz as f64; hz]));
    }
    let jitter: Vec<_> = (0..30).flat_map(|_| [1.0 / 120.0, 1.0 / 40.0]).collect();
    assert_eq!(expected, replay(&jitter));
}

#[test]
fn camera_bounds_and_resizing_cover_wide_tall_and_tiny_windows() {
    for (width, height) in [
        (1280.0, 720.0),
        (2560.0, 600.0),
        (480.0, 1000.0),
        (0.0, 0.0),
    ] {
        let (w, h) = view_size(width, height, 2400.0, 1315.0);
        assert!(w > 0.0 && w <= 2400.0 && h > 0.0 && h <= 1315.0);
        for target in [0.0, 400.0, 2400.0] {
            let x = camera_axis(1200.0, target, w / 2.0, 2400.0, 20.0, 24.0, 10.0);
            assert!(x >= w / 2.0 && x <= 2400.0 - w / 2.0);
        }
    }
}

#[test]
fn bevy_keyboard_messages_map_shift_and_focus_loss_clears_intent() {
    use super::{Playground, capture_input};
    use bevy::{
        input::{
            ButtonState,
            keyboard::{Key as LogicalKey, KeyboardInput},
            mouse::MouseButtonInput,
        },
        prelude::*,
        window::{PrimaryWindow, WindowFocused},
    };
    let mut app = App::new();
    app.init_resource::<Playground>()
        .add_message::<KeyboardInput>()
        .add_message::<WindowFocused>()
        .add_message::<MouseButtonInput>()
        .add_systems(Update, capture_input);
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
    let press = KeyboardInput {
        key_code: KeyCode::ShiftLeft,
        logical_key: LogicalKey::Shift,
        state: ButtonState::Pressed,
        text: None,
        repeat: false,
        window,
    };
    app.world_mut().write_message(press.clone());
    app.update();
    let command = app
        .world_mut()
        .resource_mut::<Playground>()
        .input
        .command(0);
    assert!(command.jet_pressed && command.jet_held);
    app.world_mut().write_message(KeyboardInput {
        repeat: true,
        ..press
    });
    app.update();
    assert!(
        !app.world_mut()
            .resource_mut::<Playground>()
            .input
            .command(1)
            .jet_pressed
    );
    app.world_mut().write_message(MouseButtonInput {
        button: MouseButton::Left,
        state: ButtonState::Pressed,
        window,
    });
    app.world_mut().write_message(KeyboardInput {
        key_code: KeyCode::KeyR,
        logical_key: LogicalKey::Character("r".into()),
        state: ButtonState::Pressed,
        text: None,
        repeat: false,
        window,
    });
    app.update();
    let command = app
        .world_mut()
        .resource_mut::<Playground>()
        .input
        .command(2);
    assert!(command.fire_held && command.reload_pressed);
    app.world_mut().get_mut::<Window>(window).unwrap().focused = false;
    app.world_mut().write_message(WindowFocused {
        window,
        focused: false,
    });
    app.update();
    let mut game = app.world_mut().resource_mut::<Playground>();
    assert!(!game.focused);
    let command = game.input.command(2);
    assert!(
        command.release_input
            && !command.jet_held
            && !command.jet_pressed
            && !command.fire_held
            && !command.reload_pressed
    );
    app.world_mut().get_mut::<Window>(window).unwrap().focused = true;
    for focused in [true, false, true] {
        app.world_mut()
            .write_message(WindowFocused { window, focused });
    }
    app.update();
    // Consume the intentional resume cancellation, then press in the next frame.
    app.world_mut()
        .resource_mut::<Playground>()
        .input
        .command(3);
    app.world_mut().write_message(KeyboardInput {
        key_code: KeyCode::ShiftLeft,
        logical_key: LogicalKey::Shift,
        state: ButtonState::Pressed,
        text: None,
        repeat: false,
        window,
    });
    app.update();
    let command = app
        .world_mut()
        .resource_mut::<Playground>()
        .input
        .command(4);
    assert!(command.jet_pressed && command.jet_held);
}

#[test]
fn mouse_taps_reload_and_selection_edges_are_consumed_once() {
    use burnhop_gameplay_core::{Vec2, WeaponId};
    let mut input = InputBuffer::default();
    input.aim_at = Some(Vec2 {
        x: 928.0,
        y: 1186.0,
    });
    for key in [Key::Fire, Key::Reload, Key::Rifle] {
        input.push(key, true);
        input.push(key, true);
        input.push(key, false);
    }
    assert!(input.command(0).fire_held);
    let second = input.command(1);
    assert!(!second.fire_held && second.reload_pressed);
    let third = input.command(2);
    assert!(!third.reload_pressed);
    assert_eq!(third.select_weapon, Some(WeaponId::M416));
    let fourth = input.command(3);
    assert!(!fourth.fire_held && !fourth.reload_pressed && fourth.select_weapon.is_none());
}
#[test]
fn cursor_exit_cancels_fire_without_canceling_movement_and_reentry_needs_new_press() {
    let mut input = InputBuffer::default();
    input.push(Key::Right, true);
    input.push(Key::Fire, true);
    assert!(input.command(0).fire_held);
    input.clear_fire();
    for tick in 1..5 {
        let c = input.command(tick);
        assert_eq!(c.move_x, MoveAxis::Right);
        assert!(!c.fire_held);
        assert!(c.aim_at.is_none());
    }
    input.push(Key::Fire, true);
    assert!(input.command(5).fire_held);
}
#[test]
fn focus_and_respawn_clear_all_combat_intent_and_pending_edges() {
    let mut input = InputBuffer::default();
    for key in [Key::Right, Key::Fire, Key::JetLeft, Key::Reload, Key::Rifle] {
        input.push(key, true);
    }
    input.clear();
    assert!(input.command(0).release_input);
    for tick in 1..5 {
        assert_eq!(
            input.command(tick),
            InputCommand {
                tick,
                ..Default::default()
            }
        );
    }
}
#[test]
fn actual_bevy_projection_converts_cursor_with_camera_translation_resize_and_retina_scale() {
    use super::combat_view::cursor_world;
    use bevy::{
        camera::{CameraProjection, ComputedCameraValues, RenderTargetInfo, ScalingMode},
        prelude::*,
    };
    for (width, height, scale) in [
        (1280., 720., 1.),
        (800., 560., 2.),
        (480., 320., 2.),
        (2000., 500., 1.),
    ] {
        let (w, h) = view_size(width, height, 2400., 1350.);
        let mut projection = OrthographicProjection {
            scaling_mode: ScalingMode::Fixed {
                width: w as f32,
                height: h as f32,
            },
            ..OrthographicProjection::default_2d()
        };
        projection.update(width as f32, height as f32);
        let camera = Camera {
            computed: ComputedCameraValues {
                clip_from_view: projection.get_clip_from_view(),
                target_info: Some(RenderTargetInfo {
                    physical_size: UVec2::new((width * scale) as u32, (height * scale) as u32),
                    scale_factor: scale as f32,
                }),
                ..default()
            },
            ..default()
        };
        for (x, y) in [(640., -955.), (1480., -600.)] {
            let transform = GlobalTransform::from_translation(Vec3::new(x, y, 0.));
            for (sx, sy) in [(0.5, 0.5), (0.25, 0.75), (0.8, 0.1)] {
                let cursor = Vec2::new(width as f32 * sx, height as f32 * sy);
                let point = cursor_world(
                    &camera,
                    &transform,
                    cursor,
                    Vec2::new(width as f32, height as f32),
                )
                .unwrap();
                assert!((point.x - (f64::from(x) + (f64::from(sx) - 0.5) * w)).abs() < 0.001);
                assert!((point.y - (-f64::from(y) + (f64::from(sy) - 0.5) * h)).abs() < 0.001);
                let round_trip = camera
                    .world_to_viewport(&transform, Vec3::new(point.x as f32, -point.y as f32, 0.))
                    .unwrap();
                assert!((round_trip - cursor).length() < 0.001);
            }
            assert!(
                cursor_world(
                    &camera,
                    &transform,
                    Vec2::new(-1., 0.),
                    Vec2::new(width as f32, height as f32)
                )
                .is_none()
            );
            assert!(
                cursor_world(
                    &camera,
                    &transform,
                    Vec2::ZERO,
                    Vec2::new(width as f32 + 100., height as f32)
                )
                .is_none()
            );
        }
    }
}

#[test]
fn online_bevy_f5_is_ignored_and_focus_clears_input_without_resetting_clock() {
    use super::{Playground, capture_input, online::Online};
    use bevy::{
        input::{
            ButtonState,
            keyboard::{Key as LogicalKey, KeyboardInput},
            mouse::MouseButtonInput,
        },
        prelude::*,
        window::{PrimaryWindow, WindowFocused},
    };
    let mut app = App::new();
    let mut game = Playground {
        online: Some(Online::new("127.0.0.1:9".parse().unwrap(), false).unwrap()),
        ..Default::default()
    };
    game.clock.advance(DT / 2.);
    game.input.push(Key::Right, true);
    app.insert_resource(game)
        .add_message::<KeyboardInput>()
        .add_message::<MouseButtonInput>()
        .add_message::<WindowFocused>()
        .add_systems(Update, capture_input);
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
    app.world_mut().write_message(KeyboardInput {
        key_code: KeyCode::F5,
        logical_key: LogicalKey::F5,
        state: ButtonState::Pressed,
        text: None,
        repeat: false,
        window,
    });
    app.update();
    let c = app
        .world_mut()
        .resource_mut::<Playground>()
        .input
        .command(0);
    assert!(!c.reset);
    assert_eq!(c.move_x, MoveAxis::Right);
    app.world_mut().get_mut::<Window>(window).unwrap().focused = false;
    app.world_mut().write_message(WindowFocused {
        window,
        focused: false,
    });
    app.update();
    let mut game = app.world_mut().resource_mut::<Playground>();
    let c = game.input.command(1);
    assert!(c.release_input);
    assert_eq!(c.move_x, MoveAxis::Idle);
    assert_eq!(
        game.clock.advance(DT / 2.).0,
        1,
        "online clock must keep its partial tick on focus loss"
    );
}
