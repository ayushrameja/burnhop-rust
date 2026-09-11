mod adapter;
mod combat_view;
mod online;
mod playtest;
#[cfg(test)]
mod tests;

use adapter::{FrameClock, InputBuffer, Key, camera_axis, view_size};
use bevy::{
    camera::ScalingMode,
    input::{ButtonState, keyboard::KeyboardInput, mouse::MouseButtonInput},
    prelude::*,
    window::{PresentMode, PrimaryWindow, WindowFocused},
};
use burnhop_gameplay_core::{
    CombatState, PRACTICE_ARENA, Player, Rect, World as GameWorld, step_practice,
};

#[derive(Resource)]
struct Playground {
    online: Option<online::Online>,
    world: GameWorld,
    combat: CombatState,
    feedback: combat_view::Feedback,
    diagnostics: bool,
    previous: Player,
    input: InputBuffer,
    clock: FrameClock,
    alpha: f64,
    focused: bool,
    snap_camera: bool,
    route: Option<playtest::Route>,
    combat_route: Option<playtest::CombatRoute>,
}
impl Default for Playground {
    fn default() -> Self {
        let world = GameWorld::new(&PRACTICE_ARENA);
        Self {
            online: None,
            previous: world.player,
            world,
            combat: CombatState::default(),
            feedback: combat_view::Feedback::default(),
            diagnostics: std::env::args().any(|arg| arg == "--combat-diagnostics"),
            input: InputBuffer::default(),
            clock: FrameClock::default(),
            alpha: 0.0,
            focused: true,
            snap_camera: true,
            combat_route: std::env::args()
                .any(|arg| arg == "--combat-playtest")
                .then(playtest::CombatRoute::default),
            route: std::env::args()
                .any(|arg| arg == "--movement-playtest")
                .then(playtest::Route::default),
        }
    }
}
#[derive(Component)]
struct PlayerVisual;
#[derive(Component)]
struct JetVisual;
#[derive(Component)]
struct HudText;
#[derive(Component)]
struct FuelFill;

fn main() {
    let mut game = Playground::default();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut address = None;
    let mut offline = false;
    let mut scripted = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--connect" => {
                i += 1;
                address = args
                    .get(i)
                    .and_then(|a| a.parse::<std::net::SocketAddr>().ok());
                if address.is_none() {
                    eprintln!("--connect requires an explicit IP:port, e.g. 127.0.0.1:5000");
                    return;
                }
            }
            "--offline" => offline = true,
            "--online-playtest" => scripted = true,
            "--combat-playtest" | "--combat-diagnostics" | "--movement-playtest" => {}
            _ => {
                eprintln!(
                    "Usage: burnhop-client [--offline | --connect IP:port] [--online-playtest]"
                );
                return;
            }
        }
        i += 1;
    }
    if (offline && address.is_some())
        || (scripted && address.is_none())
        || (address.is_some() && (game.route.is_some() || game.combat_route.is_some()))
    {
        eprintln!("Choose offline practice or --connect IP:port; online route requires --connect.");
        return;
    }
    if let Some(address) = address {
        match online::Online::new(address, scripted) {
            Ok(online) => game.online = Some(online),
            Err(error) => {
                eprintln!("Cannot start multiplayer: {error}");
                return;
            }
        }
    }
    let title = if game.online.is_some() {
        "Burnhop — Direct Connect"
    } else {
        "Burnhop — Combat Practice"
    };
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.055, 0.075, 0.10)))
        .insert_resource(game)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: title.into(),
                resolution: (1280, 720).into(),
                resize_constraints: bevy::window::WindowResizeConstraints {
                    min_width: 480.0,
                    min_height: 320.0,
                    ..default()
                },
                present_mode: PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, (setup, combat_view::setup))
        .add_systems(
            Update,
            (
                capture_input,
                combat_view::capture_aim,
                simulate,
                present,
                combat_view::present,
            )
                .chain(),
        )
        .run();
}

fn position(rect: Rect, z: f32) -> Vec3 {
    Vec3::new(
        (rect.x + rect.width / 2.0) as f32,
        -(rect.y + rect.height / 2.0) as f32,
        z,
    )
}
fn setup(mut commands: Commands, game: Res<Playground>) {
    commands.spawn(Camera2d);
    for rect in PRACTICE_ARENA.solids {
        commands.spawn((
            Sprite::from_color(
                Color::srgb(0.25, 0.34, 0.38),
                Vec2::new(rect.width as f32, rect.height as f32),
            ),
            Transform::from_translation(position(*rect, 0.0)),
        ));
        // Thin highlight lies on the solid's top, using its actual authored edge.
        commands.spawn((
            Sprite::from_color(
                Color::srgb(0.48, 0.65, 0.65),
                Vec2::new(rect.width as f32, 3.0),
            ),
            Transform::from_xyz(
                (rect.x + rect.width / 2.0) as f32,
                -rect.y as f32 - 1.5,
                0.1,
            ),
        ));
    }
    let spawn = PRACTICE_ARENA.spawn;
    commands.spawn((
        Sprite::from_color(Color::srgb(0.20, 0.78, 0.64), Vec2::new(64.0, 5.0)),
        Transform::from_xyz(
            (spawn.x + 18.0) as f32,
            -PRACTICE_ARENA.floor_y as f32 + 2.5,
            0.2,
        ),
    ));
    commands.spawn((
        PlayerVisual,
        Sprite::from_color(Color::srgb(1.0, 0.39, 0.16), Vec2::new(36.0, 68.0)),
        Transform::from_translation(position(game.world.player.body, 1.0)),
    ));
    commands.spawn((
        JetVisual,
        Sprite::from_color(Color::srgb(0.25, 0.85, 1.0), Vec2::new(22.0, 22.0)),
        Transform::default(),
        Visibility::Hidden,
    ));
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(14),
                left: px(14),
                right: px(14),
                padding: UiRect::all(px(12)),
                flex_direction: FlexDirection::Column,
                row_gap: px(6),
                ..default()
            },
            BackgroundColor(Color::srgba(0.025, 0.04, 0.06, 0.92)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(if game.online.is_some() { "A/D Move   SPACE Jump   SHIFT Jet   Mouse Aim/Fire   R Reload   1/2 Weapon   Close to leave" } else { "A/D Move   SPACE Jump   SHIFT Jet   Mouse Aim/Fire   R Reload   1/2 Weapon   F5 Reset" }),
                TextFont {
                    font_size: FontSize::Px(18.0),
                    ..default()
                },
            ));
            parent.spawn((
                HudText,
                Text::default(),
                TextFont {
                    font_size: FontSize::Px(16.0),
                    ..default()
                },
                TextColor(Color::srgb(0.70, 0.88, 0.91)),
            ));
            parent
                .spawn((
                    Node {
                        width: px(220),
                        height: px(8),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.18, 0.25, 0.29)),
                ))
                .with_child((
                    FuelFill,
                    Node {
                        width: percent(100),
                        height: percent(100),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.25, 0.85, 1.0)),
                ));
        });
    if game.online.is_some() {
        info!("Direct connect: movement predicted locally; combat owned by the headless server.");
    } else {
        info!(
            "Combat practice: 60 Hz shared simulation, R reload / F5 reset. Bot fires once per second after 3 seconds grace."
        );
    }
}

fn capture_input(
    mut game: ResMut<Playground>,
    window: Single<(Entity, &Window), With<PrimaryWindow>>,
    mut keyboard: MessageReader<KeyboardInput>,
    mut focus: MessageReader<WindowFocused>,
    mut mouse: MessageReader<MouseButtonInput>,
) {
    // Drain the whole batch: leaving a second focus event unread would clear
    // fresh input again on the following frame.
    let changed_focus = focus
        .read()
        .filter(|event| event.window == window.0)
        .count()
        > 0;
    let focused = window.1.focused;
    if changed_focus || focused != game.focused || !focused {
        if (changed_focus || focused != game.focused) && game.route.is_some() {
            println!(
                "PLAYTEST focus={focused} tick={} x={:.2} y={:.2}",
                game.world.tick, game.world.player.body.x, game.world.player.body.y
            );
        }
        if !focused && game.world.tick > 0 {
            cancel_route(&mut game, "focus lost");
        }
        game.input.clear();
        if game.online.is_none() {
            game.clock.reset();
        }
        game.previous = game.world.player;
        game.alpha = 1.0;
        game.snap_camera = true;
        game.focused = focused;
        if game.diagnostics && (changed_focus || focused != game.focused) {
            println!(
                "COMBAT focus={focused} tick={} (input cleared)",
                game.world.tick
            );
        }
        keyboard.clear();
        mouse.clear();
        return;
    }
    for event in keyboard.read() {
        if event.window != window.0 || event.repeat {
            continue;
        }
        let key = match event.key_code {
            KeyCode::KeyA => Key::Left,
            KeyCode::KeyD => Key::Right,
            KeyCode::Space => Key::Jump,
            KeyCode::ShiftLeft => Key::JetLeft,
            KeyCode::ShiftRight => Key::JetRight,
            KeyCode::KeyR => Key::Reload,
            KeyCode::F5 if game.online.is_none() => Key::Reset,
            KeyCode::Digit1 => Key::Pistol,
            KeyCode::Digit2 => Key::Rifle,
            _ => continue,
        };
        if game.route.is_some() {
            println!(
                "PLAYTEST keyboard {key:?} {:?} tick={}",
                event.state, game.world.tick
            );
        }
        cancel_route(&mut game, "keyboard input");
        game.input.push(key, event.state == ButtonState::Pressed);
    }
    for event in mouse.read() {
        if event.window == window.0 && event.button == MouseButton::Left {
            cancel_route(&mut game, "mouse input");
            game.input
                .push(Key::Fire, event.state == ButtonState::Pressed);
        }
    }
}

fn cancel_route(game: &mut Playground, reason: &str) {
    if reason != "focus lost"
        && let Some(online) = &mut game.online
        && online.scripted()
    {
        online.cancel_script();
        game.input.clear();
        println!("ONLINE PLAYTEST canceled ({reason}); device controls available.");
    }
    if let Some(route) = &mut game.combat_route
        && !route.complete
    {
        route.complete = true;
        game.input.clear();
        println!("COMBAT PLAYTEST canceled ({reason}); device controls available.");
    }
    if let Some(route) = &mut game.route
        && !route.complete
    {
        route.complete = true;
        game.input.clear();
        println!("PLAYTEST route canceled ({reason}); keyboard control is available.");
    }
}

fn simulate(mut game: ResMut<Playground>, time: Res<Time<Real>>) {
    if game.online.is_some() {
        online::simulate(&mut game, time.delta_secs_f64());
        return;
    }
    if !game.focused {
        return;
    }
    let (ticks, alpha) = game.clock.advance(time.delta_secs_f64());
    game.alpha = alpha;
    for _ in 0..ticks {
        let tick = game.world.tick;
        if let Some(route) = game.route.take() {
            if !route.complete {
                route.feed(tick, &mut game.input);
            }
            game.route = Some(route);
        }
        if let Some(route) = game.combat_route.take() {
            if !route.complete {
                route.feed(tick, &mut game.input);
            }
            game.combat_route = Some(route);
        }
        let command = game.input.command(tick);
        game.previous = game.world.player;
        let game_ref = &mut *game;
        let combat_events = if game_ref.route.as_ref().is_some_and(|route| !route.complete) {
            // Historical movement route deliberately has no attacking opponent.
            burnhop_gameplay_core::CombatEvents {
                movement: burnhop_gameplay_core::step(
                    &mut game_ref.world,
                    command,
                    &PRACTICE_ARENA,
                )
                .expect("next tick"),
                ..default()
            }
        } else {
            step_practice(
                &mut game_ref.world,
                &mut game_ref.combat,
                command,
                &PRACTICE_ARENA,
            )
            .expect("client owns the next tick")
        };
        game_ref.feedback.record(&combat_events);
        if game_ref.diagnostics
            && (command.fire_held
                || command.reload_pressed
                || command.select_weapon.is_some()
                || command.reset
                || command.release_input
                || combat_events.player_died
                || combat_events.bot_died
                || combat_events.player_respawned
                || combat_events.bot_respawned
                || combat_events.shots.iter().any(Option::is_some))
        {
            println!(
                "COMBAT tick={tick} command={command:?} events={combat_events:?} player_hp={} bot_hp={} weapon={:?} position={:?}",
                game_ref.combat.player.health,
                game_ref.combat.bot.health,
                game_ref.combat.player.weapon(),
                game_ref.world.player.body
            );
        }
        if combat_events.player_died
            || combat_events.player_respawned
            || combat_events.movement.reset
        {
            game_ref.input.clear();
        }
        if combat_events.player_respawned {
            game_ref.previous = game_ref.world.player;
            game_ref.snap_camera = true;
        }
        if let Some(mut route) = game_ref.combat_route.take() {
            if !route.complete {
                route.observe(&game_ref.world, &game_ref.combat);
            }
            game_ref.combat_route = Some(route);
        }
        let events = combat_events.movement;
        if game.route.as_ref().is_some_and(|route| route.complete)
            && (command.jump_pressed
                || command.jet_pressed
                || command.reset
                || command.release_input
                || events.landed)
        {
            println!(
                "PLAYTEST manual tick={tick} command={command:?} player={:?}",
                game.world.player
            );
        }
        if let Some(mut route) = game.route.take() {
            if !route.complete {
                route.observe(&game.world);
            }
            game.route = Some(route);
        }
        if events.reset {
            game.previous = game.world.player;
            game.snap_camera = true;
        }
    }
}

// Disjoint component filters make the transform borrows explicit to Bevy.
type PlayerFilter = (With<PlayerVisual>, Without<JetVisual>, Without<Camera2d>);
type JetFilter = (With<JetVisual>, Without<PlayerVisual>, Without<Camera2d>);
type CameraFilter = (With<Camera2d>, Without<PlayerVisual>, Without<JetVisual>);
#[allow(clippy::too_many_arguments)]
fn present(
    mut game: ResMut<Playground>,
    time: Res<Time<Real>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut player: Single<(&mut Transform, &mut Sprite), PlayerFilter>,
    mut jet: Single<(&mut Transform, &mut Visibility), JetFilter>,
    mut camera: Single<(&mut Transform, &mut Projection), CameraFilter>,
    mut text: Single<&mut Text, With<HudText>>,
    mut fuel: Single<&mut Node, With<FuelFill>>,
) {
    let p = game.world.player;
    let a = game.alpha;
    let mut rect = p.body;
    rect.x = game.previous.body.x + (p.body.x - game.previous.body.x) * a;
    rect.y = game.previous.body.y + (p.body.y - game.previous.body.y) * a;
    player.0.translation = position(rect, 1.0);
    player.1.color = if !game.combat.player.alive() {
        Color::srgba(0.35, 0.35, 0.38, 0.35)
    } else if game.feedback.player_hit > 0.0 {
        Color::WHITE
    } else {
        Color::srgb(1.0, 0.39, 0.16)
    };
    jet.0.translation = player.0.translation + Vec3::new(0.0, -45.0, -0.1);
    *jet.1 = if p.thrusting && game.focused && game.combat.player.alive() {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    let arena = PRACTICE_ARENA;
    let (width, height) = view_size(
        window.width() as f64,
        window.height() as f64,
        arena.width,
        arena.height,
    );
    if let Projection::Orthographic(projection) = &mut *camera.1 {
        projection.scaling_mode = ScalingMode::Fixed {
            width: width as f32,
            height: height as f32,
        };
    }
    let target_x = rect.x + rect.width / 2.0;
    let target_y = rect.y + rect.height / 2.0;
    let (old_x, old_y) = if game.snap_camera {
        (target_x, target_y)
    } else {
        (
            camera.0.translation.x as f64,
            -camera.0.translation.y as f64,
        )
    };
    let x = camera_axis(
        old_x,
        target_x,
        width / 2.0,
        arena.width,
        20.0,
        24.0,
        time.delta_secs_f64(),
    );
    let y = camera_axis(
        old_y,
        target_y,
        height / 2.0,
        arena.floor_y + 95.0,
        24.0,
        32.0,
        time.delta_secs_f64(),
    );
    camera.0.translation.x = x as f32;
    camera.0.translation.y = -y as f32;
    game.snap_camera = false;
    let status = if game.online.as_ref().is_some_and(|online| online.scripted()) {
        "SCRIPTED ONLINE INPUT - any key cancels"
    } else if game.route.as_ref().is_some_and(|route| !route.complete)
        || game
            .combat_route
            .as_ref()
            .is_some_and(|route| !route.complete)
    {
        "SCRIPT RUNNING - any key cancels"
    } else if !game.focused && game.online.is_some() {
        "UNFOCUSED - input cleared, match continues"
    } else if !game.focused {
        "PAUSED - click window, then press controls again"
    } else if p.thrusting {
        "THRUST"
    } else if p.grounded {
        "GROUNDED"
    } else {
        "AIRBORNE"
    };
    text.0 = combat_view::hud(&game, status);
    fuel.width = percent(p.fuel as f32);
}
