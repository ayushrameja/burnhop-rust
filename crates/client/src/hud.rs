//! Native, responsive combat HUD. Diagnostic output belongs in opt-in logs.
use crate::{
    Playground,
    artwork::{CREAM, CYAN, OCHRE, color},
};
use bevy::{prelude::*, window::PrimaryWindow};
use burnhop_gameplay_core::{LifeState, Reserve};
#[derive(Component, Clone, Copy)]
pub enum Field {
    Title,
    Mode,
    Connection,
    Health,
    Fuel,
    Weapon,
    Ammo,
    WeaponState,
    Help,
    Overlay,
}
#[derive(Component)]
pub enum Panel {
    Identity,
    Connection,
    Vitals,
    Weapon,
    Help,
    Overlay,
}
#[derive(Component)]
pub struct MeterTrack;
type MeterFilter = (Without<Panel>, Without<Field>, Without<MeterTrack>);
type TrackFilter = (With<MeterTrack>, Without<Panel>, Without<Field>);

#[derive(Default)]
pub struct ControlsHelp {
    expanded: bool,
}
impl ControlsHelp {
    fn update(&mut self, compact: bool, focused: bool, toggle: bool) {
        if !compact || !focused {
            self.expanded = false;
        } else if toggle {
            self.expanded = !self.expanded;
        }
    }
}

fn compact_layout(width: f32, height: f32) -> bool {
    width < 1050. || height < 500.
}

#[derive(Component)]
pub enum Meter {
    Health,
    Fuel,
    Reload,
}
fn font(size: f32) -> TextFont {
    TextFont {
        font_size: FontSize::Px(size),
        ..default()
    }
}
pub fn setup(mut commands: Commands) {
    let ink = BackgroundColor(color(0x172420).with_alpha(0.94));
    commands
        .spawn((
            Panel::Identity,
            Node {
                position_type: PositionType::Absolute,
                top: px(18),
                left: px(22),
                flex_direction: FlexDirection::Column,
                row_gap: px(4),
                ..default()
            },
        ))
        .with_children(|p| {
            p.spawn((
                Field::Title,
                Text::new("BURNHOP"),
                font(23.),
                TextColor(color(CREAM)),
            ));
            p.spawn((
                Field::Mode,
                Text::default(),
                font(11.),
                TextColor(color(0xa7b394)),
            ));
        });
    commands
        .spawn((
            Panel::Connection,
            Node {
                position_type: PositionType::Absolute,
                top: px(20),
                right: px(22),
                width: px(360),
                padding: UiRect::all(px(8)),
                ..default()
            },
            ink,
        ))
        .with_child((
            Field::Connection,
            Text::default(),
            font(12.),
            TextColor(color(CREAM)),
        ));
    commands
        .spawn((
            Panel::Vitals,
            Node {
                position_type: PositionType::Absolute,
                bottom: px(42),
                left: px(22),
                width: px(220),
                padding: UiRect::all(px(12)),
                flex_direction: FlexDirection::Column,
                row_gap: px(6),
                ..default()
            },
            ink,
        ))
        .with_children(|p| {
            p.spawn((
                Field::Health,
                Text::default(),
                font(16.),
                TextColor(color(CREAM)),
            ));
            p.spawn((
                MeterTrack,
                Node {
                    height: px(4),
                    width: percent(100),
                    ..default()
                },
                BackgroundColor(color(0x3d4b40)),
            ))
            .with_child((
                Meter::Health,
                Node {
                    height: percent(100),
                    width: percent(100),
                    ..default()
                },
                BackgroundColor(color(0xb4bc8b)),
            ));
            p.spawn((
                Field::Fuel,
                Text::default(),
                font(12.),
                TextColor(color(CYAN)),
            ));
            p.spawn((
                MeterTrack,
                Node {
                    height: px(4),
                    width: percent(100),
                    ..default()
                },
                BackgroundColor(color(0x3d4b40)),
            ))
            .with_child((
                Meter::Fuel,
                Node {
                    height: percent(100),
                    width: percent(100),
                    ..default()
                },
                BackgroundColor(color(CYAN)),
            ));
        });
    commands
        .spawn((
            Panel::Weapon,
            Node {
                position_type: PositionType::Absolute,
                bottom: px(42),
                right: px(22),
                width: px(240),
                padding: UiRect::all(px(12)),
                flex_direction: FlexDirection::Column,
                row_gap: px(2),
                ..default()
            },
            ink,
        ))
        .with_children(|p| {
            p.spawn((
                Field::Weapon,
                Text::default(),
                font(12.),
                TextColor(color(0xb4bc8b)),
            ));
            p.spawn((
                Field::Ammo,
                Text::default(),
                font(28.),
                TextColor(color(CREAM)),
            ));
            p.spawn((
                Field::WeaponState,
                Text::default(),
                font(11.),
                TextColor(color(CREAM)),
            ));
            p.spawn((
                MeterTrack,
                Node {
                    height: px(3),
                    width: percent(100),
                    ..default()
                },
                BackgroundColor(color(0x3d4b40)),
            ))
            .with_child((
                Meter::Reload,
                Node {
                    height: percent(100),
                    width: percent(0),
                    ..default()
                },
                BackgroundColor(color(OCHRE)),
            ));
        });
    commands
        .spawn((
            Panel::Help,
            BackgroundColor(Color::NONE),
            Node {
                position_type: PositionType::Absolute,
                bottom: px(10),
                left: px(22),
                right: px(22),
                ..default()
            },
        ))
        .with_child((
            Field::Help,
            Text::default(),
            font(11.),
            TextColor(color(0xa7b394)),
        ));
    commands
        .spawn((
            Panel::Overlay,
            Node {
                position_type: PositionType::Absolute,
                top: percent(33),
                left: percent(15),
                width: percent(70),
                padding: UiRect::all(px(16)),
                justify_content: JustifyContent::Center,
                ..default()
            },
            ink,
            Visibility::Hidden,
        ))
        .with_child((
            Field::Overlay,
            Text::default(),
            font(18.),
            TextColor(color(CREAM)),
        ));
}
fn overlay(game: &Playground) -> String {
    if let Some(o) = &game.online
        && o.network.status.terminal()
    {
        return format!("{}\nRestart the client to reconnect.", o.label());
    }
    if let LifeState::Dead { remaining_ticks } = game.combat.player.life {
        return format!(
            "PILOT DOWN\nReturning in {:.1}s",
            f32::from(remaining_ticks) / 60.
        );
    }
    if !game.focused && game.online.is_none() {
        return "PRACTICE PAUSED\nClick back, then press controls again.".into();
    }
    String::new()
}
pub fn present(
    game: Res<Playground>,
    window: Single<&Window, With<PrimaryWindow>>,
    (keys, mut help): (Res<ButtonInput<KeyCode>>, Local<ControlsHelp>),
    mut fields: Query<(&Field, &mut Text, &mut TextFont, &mut TextColor, &mut Node)>,
    mut panels: Query<
        (
            &Panel,
            &mut Node,
            &mut Visibility,
            Option<&mut BackgroundColor>,
        ),
        Without<Field>,
    >,
    mut meters: Query<(&Meter, &mut Node, &mut BackgroundColor), MeterFilter>,
    mut tracks: Query<&mut Node, TrackFilter>,
) {
    let compact = compact_layout(window.width(), window.height());
    help.update(compact, window.focused, keys.just_pressed(KeyCode::F1));
    let player = game.combat.player;
    let w = player.weapon();
    let message = overlay(&game);
    for (field, mut text, mut style, mut tint, mut node) in &mut fields {
        node.display = if compact && matches!(field, Field::Mode | Field::Weapon) {
            Display::None
        } else {
            Display::Flex
        };
        text.0 = match field {
            Field::Title => "BURNHOP".into(),
            Field::Mode => if game.online.is_some() {
                "DIRECT CONNECT"
            } else {
                "FIELD RANGE / PRACTICE"
            }
            .into(),
            Field::Connection => {
                if let Some(o) = &game.online {
                    let count = o.actors.iter().flatten().count();
                    let opponent = if count < 2 {
                        "Waiting for players / Tab".into()
                    } else {
                        format!("{count}/8 players / Tab scores")
                    };
                    if compact {
                        if o.network.status.terminal() {
                            "CONNECTION ERROR / see message".into()
                        } else if !game.focused {
                            "INPUT RELEASED / click back".into()
                        } else if let Some(welcome) = o.network.welcome {
                            format!("P{} / {}", welcome.actor.index() + 1, opponent)
                        } else {
                            o.label()
                        }
                    } else {
                        format!(
                            "{}\n{}{}",
                            o.label(),
                            opponent,
                            if !game.focused {
                                "\nUnfocused - input released"
                            } else {
                                ""
                            }
                        )
                    }
                } else {
                    let bot = if !game.combat.bot.alive() {
                        "Bot respawning".into()
                    } else if game.combat.bot_attack_ticks > 60 {
                        format!(
                            "Bot ready in {:.1}s",
                            f32::from(game.combat.bot_attack_ticks) / 60.
                        )
                    } else {
                        format!("Bot {} HP / LIVE FIRE", game.combat.bot.health)
                    };
                    if compact {
                        bot
                    } else {
                        format!(
                            "{bot}\n{} kills / {} deaths",
                            game.combat.kills, game.combat.deaths
                        )
                    }
                }
            }
            Field::Health if compact => format!(
                "HP {}{}",
                player.health,
                if player.health == 0 {
                    " DOWN"
                } else if player.health < 30 {
                    " LOW"
                } else {
                    ""
                }
            ),
            Field::Health => format!("HEALTH  {:3}", player.health),
            Field::Fuel => format!(
                "JET  {:3.0}%  {}",
                game.world.player.fuel,
                if game.world.player.fuel <= 0. {
                    "EMPTY"
                } else if game.world.player.fuel < 25. {
                    "LOW"
                } else if game.world.player.thrusting {
                    "THRUST"
                } else {
                    "FUEL"
                }
            ),
            Field::Weapon => format!(
                "[{}] {}",
                w.id.index() + 1,
                w.id.tuning().name.to_uppercase()
            ),
            Field::Ammo if compact => format!(
                "{} {}/{}",
                w.id.tuning().name,
                w.ammo,
                match w.reserve {
                    Reserve::Rounds(n) => n.to_string(),
                    Reserve::Unlimited => "INF".into(),
                }
            ),
            Field::Ammo => format!(
                "{:02} / {}",
                w.ammo,
                match w.reserve {
                    Reserve::Rounds(n) => n.to_string(),
                    Reserve::Unlimited => "INF".into(),
                }
            ),
            Field::WeaponState => {
                if !player.alive() {
                    "INACTIVE".into()
                } else if w.reload_ticks > 0 {
                    format!("RELOADING  {:.1}s", f32::from(w.reload_ticks) / 60.)
                } else if player.equip_ticks > 0 {
                    "EQUIPPING".into()
                } else if w.ammo == 0 {
                    if w.reserve == Reserve::Rounds(0) {
                        if compact {
                            "EMPTY / 1-2 SWITCH"
                        } else {
                            "EMPTY / SWITCH WEAPON"
                        }
                        .into()
                    } else {
                        if compact {
                            "EMPTY / R RELOAD"
                        } else {
                            "EMPTY / R TO RELOAD"
                        }
                        .into()
                    }
                } else if compact {
                    "MAG/RES  R Reload".into()
                } else {
                    "MAG / RESERVE   R RELOAD".into()
                }
            }
            Field::Help => {
                if compact && !help.expanded {
                    "F1 Controls".into()
                } else if compact {
                    format!(
                        "F1 Hide controls\nA/D Move  SPACE Jump  SHIFT Jet\nMouse Aim/Fire  1/2 Gun  R Reload{}",
                        if game.online.is_none() {
                            "  F5 Reset"
                        } else {
                            ""
                        }
                    )
                } else {
                    format!(
                        "A/D Move   SPACE Jump   SHIFT Jet   Mouse Aim/Fire   1/2 Weapon   R Reload{}",
                        if game.online.is_none() {
                            "   F5 Reset"
                        } else {
                            ""
                        }
                    )
                }
            }
            Field::Overlay => message.clone(),
        };
        style.font_size = FontSize::Px(match field {
            Field::Title => {
                if compact {
                    14.
                } else {
                    23.
                }
            }
            Field::Ammo => {
                if compact {
                    16.
                } else {
                    28.
                }
            }
            Field::Health => {
                if compact {
                    13.
                } else {
                    16.
                }
            }
            Field::Overlay => {
                if compact {
                    14.
                } else {
                    18.
                }
            }
            Field::Help | Field::Mode => 11.,
            Field::Connection if compact => 11.,
            _ => 12.,
        });
        if matches!(field, Field::Fuel) {
            tint.0 = color(if game.world.player.fuel < 25. {
                OCHRE
            } else {
                CYAN
            });
        }
        if matches!(field, Field::Health) {
            tint.0 = color(if player.health < 30 { OCHRE } else { CREAM });
        }
    }
    for (panel, mut node, mut visibility, background) in &mut panels {
        let edge = if compact { 6. } else { 22. };
        match panel {
            Panel::Identity => {
                node.left = px(edge);
                node.top = px(if compact { 6. } else { 18. });
            }
            Panel::Connection => {
                node.right = px(edge);
                node.top = if compact { Val::Auto } else { px(20.) };
                node.bottom = if compact { px(2.) } else { Val::Auto };
                node.width = if compact { Val::Auto } else { px(360.) };
                node.max_width = if compact { px(238.) } else { Val::Auto };
                node.padding = UiRect::all(px(if compact { 0. } else { 8. }));
                node.display = if compact && help.expanded && message.is_empty() {
                    Display::None
                } else {
                    Display::Flex
                };
            }
            Panel::Vitals | Panel::Weapon => {
                let vitals = matches!(panel, Panel::Vitals);
                node.width = px(if compact {
                    if vitals { 132. } else { 164. }
                } else if vitals {
                    220.
                } else {
                    240.
                });
                node.padding = UiRect::all(px(if compact { 4. } else { 6. }));
                node.row_gap = px(if compact {
                    2.
                } else if vitals {
                    6.
                } else {
                    2.
                });
                node.bottom = if compact { Val::Auto } else { px(10.) };
                node.top = if compact {
                    px(if vitals { 26. } else { 6. })
                } else {
                    Val::Auto
                };
                if vitals {
                    node.left = px(edge);
                } else {
                    node.right = px(edge);
                }
            }
            Panel::Help => {
                node.left = px(if compact { edge } else { 265. });
                node.right = if compact && !help.expanded {
                    Val::Auto
                } else {
                    px(if compact { edge } else { 410. })
                };
                node.bottom = if compact { px(2.) } else { Val::Auto };
                node.top = if compact { Val::Auto } else { px(22.) };
                node.padding = UiRect::all(px(if compact && help.expanded { 4. } else { 0. }));
                if let Some(mut bg) = background {
                    bg.0 = color(0x172420).with_alpha(if compact { 0.94 } else { 0. });
                }
                node.display = if compact && !message.is_empty() {
                    Display::None
                } else {
                    Display::Flex
                };
            }
            Panel::Overlay => {
                node.top = if compact { px(78.) } else { percent(33.) };
                node.left = percent(if compact { 5. } else { 15. });
                node.width = percent(if compact { 90. } else { 70. });
                node.padding = UiRect::all(px(if compact { 8. } else { 16. }));
                *visibility = if message.is_empty() {
                    Visibility::Hidden
                } else {
                    Visibility::Visible
                };
            }
        }
    }
    for mut track in &mut tracks {
        track.display = if compact {
            Display::None
        } else {
            Display::Flex
        };
    }
    for (meter, mut node, mut bg) in &mut meters {
        node.width = percent(match meter {
            Meter::Health => f32::from(player.health),
            Meter::Fuel => game.world.player.fuel as f32,
            Meter::Reload => {
                if w.reload_ticks > 0 {
                    100. * (1. - f32::from(w.reload_ticks) / f32::from(w.id.tuning().reload))
                } else {
                    0.
                }
            }
        });
        if matches!(meter, Meter::Fuel) {
            bg.0 = color(if game.world.player.fuel < 25. {
                OCHRE
            } else {
                CYAN
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn controls_disclosure_starts_closed_and_resets_on_focus_or_desktop() {
        let mut help = ControlsHelp::default();
        assert!(!help.expanded);
        help.update(true, true, true);
        assert!(help.expanded);
        help.update(true, true, false);
        assert!(help.expanded);
        help.update(true, true, true);
        assert!(!help.expanded);
        help.update(true, true, true);
        help.update(true, false, true);
        assert!(!help.expanded);
        help.update(true, true, true);
        help.update(false, true, false);
        help.update(true, true, false);
        assert!(!help.expanded);
        assert!(compact_layout(480., 320.));
        assert!(compact_layout(800., 524.));
        assert!(compact_layout(1280., 320.));
        assert!(!compact_layout(1280., 720.));
    }
    #[test]
    fn native_hud_disclosure_and_resize_preserve_gameplay_input() {
        use crate::adapter::Key;
        use bevy::input::{
            ButtonState,
            keyboard::{Key as LogicalKey, KeyboardInput},
            mouse::MouseButtonInput,
        };
        use bevy::window::WindowFocused;
        let mut app = App::new();
        let mut game = Playground::default();
        game.input.push(Key::Right, true);
        app.insert_resource(game)
            .init_resource::<ButtonInput<KeyCode>>()
            .add_message::<KeyboardInput>()
            .add_message::<MouseButtonInput>()
            .add_message::<WindowFocused>()
            .add_systems(Startup, setup)
            .add_systems(Update, (crate::capture_input, present).chain());
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
        fn help_text(app: &mut App) -> String {
            app.world_mut()
                .query::<(&Field, &Text)>()
                .iter(app.world())
                .find(|(f, _)| matches!(f, Field::Help))
                .unwrap()
                .1
                .0
                .clone()
        }
        assert_eq!(help_text(&mut app), "F1 Controls");
        app.world_mut().write_message(KeyboardInput {
            key_code: KeyCode::F1,
            logical_key: LogicalKey::F1,
            state: ButtonState::Pressed,
            text: None,
            repeat: false,
            window,
        });
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::F1);
        app.update();
        assert!(help_text(&mut app).contains("F1 Hide controls"));
        assert_eq!(
            app.world_mut()
                .resource_mut::<Playground>()
                .input
                .command(0)
                .move_x,
            burnhop_gameplay_core::MoveAxis::Right
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .resolution
            .set(1280., 720.);
        app.update();
        assert!(help_text(&mut app).contains("A/D Move"));
        for (field, node) in app.world_mut().query::<(&Field, &Node)>().iter(app.world()) {
            if matches!(field, Field::Weapon | Field::Mode) {
                assert_eq!(node.display, Display::Flex);
            }
        }
        for (panel, node) in app.world_mut().query::<(&Panel, &Node)>().iter(app.world()) {
            if matches!(panel, Panel::Vitals | Panel::Weapon) {
                assert_eq!(node.top, Val::Auto);
                assert_eq!(node.bottom, px(10.));
            }
        }
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .resolution
            .set(480., 320.);
        app.update();
        assert_eq!(help_text(&mut app), "F1 Controls");
        // Exhausted resources still provide an action without opening help.
        {
            let mut game = app.world_mut().resource_mut::<Playground>();
            game.combat.player.health = 15;
            game.world.player.fuel = 0.;
            game.combat.player.weapons[0].ammo = 0;
            game.combat.player.weapons[0].reserve = Reserve::Rounds(0);
        }
        app.update();
        for (field, text) in app.world_mut().query::<(&Field, &Text)>().iter(app.world()) {
            match field {
                Field::Health => assert!(text.0.contains("LOW")),
                Field::Fuel => assert!(text.0.contains("EMPTY")),
                Field::WeaponState => assert!(text.0.contains("1-2 SWITCH")),
                _ => {}
            }
        }
    }
}
