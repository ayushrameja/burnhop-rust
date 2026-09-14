//! Fixed UI preview pool; only changed appearance values replace atlas data.
use crate::{
    Playground,
    appearance::Appearance,
    artwork::{self, Artwork, Tile},
    pilot::{self, Part},
};
use bevy::{prelude::*, window::PrimaryWindow};
#[derive(Resource)]
pub struct CustomArt {
    pub saved: Artwork,
    draft: Artwork,
    saved_value: Appearance,
    draft_value: Appearance,
}
#[derive(Component)]
pub struct PreviewRoot;
#[derive(Component)]
pub struct PreviewPart(Part);
#[derive(Component)]
pub struct PreviewCaption;
pub const POSES: [&str; 6] = ["Standing", "Crouching", "Jet", "Aim up", "Reload", "Death"];
pub fn uses_saved(game: &Playground, actor: usize) -> bool {
    game.online.is_none() && actor == 0
}
pub fn setup(
    mut commands: Commands,
    art: Res<Artwork>,
    game: Res<Playground>,
    mut images: ResMut<Assets<Image>>,
) {
    let custom = CustomArt {
        saved: Artwork {
            image: images.add(artwork::create_image(game.appearance.saved)),
            layout: art.layout.clone(),
        },
        draft: Artwork {
            image: images.add(artwork::create_image(game.appearance.draft)),
            layout: art.layout.clone(),
        },
        saved_value: game.appearance.saved,
        draft_value: game.appearance.draft,
    };
    commands
        .spawn((
            PreviewRoot,
            GlobalZIndex(110),
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                display: Display::None,
                ..default()
            },
        ))
        .with_children(|root| {
            for part in pilot::PARTS {
                root.spawn((
                    PreviewPart(part),
                    Node {
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                    ImageNode::from_atlas_image(
                        custom.draft.image.clone(),
                        TextureAtlas {
                            layout: art.layout.clone(),
                            index: 0,
                        },
                    ),
                    UiTransform::default(),
                    ZIndex(0),
                ));
            }
            root.spawn((
                PreviewCaption,
                Text::new("LIVE PREVIEW"),
                TextFont {
                    font_size: 14.0.into(),
                    ..default()
                },
                TextColor(artwork::color(artwork::CREAM)),
                Node {
                    position_type: PositionType::Absolute,
                    ..default()
                },
            ));
        });
    commands.insert_resource(custom);
}
pub fn refresh(
    game: Res<Playground>,
    mut custom: ResMut<CustomArt>,
    mut images: ResMut<Assets<Image>>,
) {
    if custom.saved_value != game.appearance.saved {
        *images.get_mut(&custom.saved.image).expect("saved atlas") =
            artwork::create_image(game.appearance.saved);
        custom.saved_value = game.appearance.saved;
    }
    if custom.draft_value != game.appearance.draft {
        *images.get_mut(&custom.draft.image).expect("draft atlas") =
            artwork::create_image(game.appearance.draft);
        custom.draft_value = game.appearance.draft;
    }
}
#[allow(clippy::type_complexity)]
pub fn present(
    game: Res<Playground>,
    time: Res<Time<Real>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut root: Single<
        &mut Node,
        (
            With<PreviewRoot>,
            Without<PreviewPart>,
            Without<PreviewCaption>,
        ),
    >,
    mut parts: Query<
        (
            &PreviewPart,
            &mut Node,
            &mut ImageNode,
            &mut UiTransform,
            &mut ZIndex,
        ),
        (Without<PreviewRoot>, Without<PreviewCaption>),
    >,
    mut caption: Single<
        (&mut Node, &mut Text, &mut TextFont),
        (
            With<PreviewCaption>,
            Without<PreviewRoot>,
            Without<PreviewPart>,
        ),
    >,
) {
    let open = game.menu.screen == crate::menu::Screen::Character;
    root.display = if open { Display::Flex } else { Display::None };
    if !open {
        return;
    }
    let compact = window.width() < 850. || window.height() < 540.;
    let width = if compact {
        (window.width() - 288.).max(160.)
    } else {
        window.width() * 0.48
    };
    let scale = if compact {
        2.25
    } else {
        (window.height() / 120.).min(5.)
    };
    let base = Vec2::new(
        width * 0.5,
        if compact {
            232.
        } else {
            window.height() * 0.70
        },
    );
    let mut movement = game.world.player;
    movement.body = burnhop_gameplay_core::Rect {
        x: 500.,
        y: 500.,
        width: 36.,
        height: 68.,
    };
    movement.grounded = true;
    movement.thrusting = false;
    movement.velocity = Default::default();
    let mut combat = burnhop_gameplay_core::CombatState::default().player;
    let direction = if game.appearance.left { -1. } else { 1. };
    combat.aim = burnhop_gameplay_core::Vec2 {
        x: direction,
        y: 0.,
    };
    combat.selected = burnhop_gameplay_core::WeaponId::M416;
    let age = time.elapsed_secs();
    match game.appearance.pose {
        1 => movement.body.height = 54.20060507330696,
        2 => {
            movement.grounded = false;
            movement.thrusting = true;
        }
        3 => {
            combat.aim = burnhop_gameplay_core::Vec2 {
                x: direction * 0.8,
                y: -0.6,
            }
        }
        4 => {
            let ticks = combat.selected.tuning().reload;
            combat.weapons[1].reload_ticks =
                (ticks as f32 * (1. - (age * 0.35).fract())).max(1.) as u16;
        }
        5 => combat.health = 0,
        _ => {}
    }
    // A preview fixture never enters simulation or changes the actual player.
    if game.appearance.pose == 5 {
        combat.life = burnhop_gameplay_core::LifeState::Dead {
            remaining_ticks: 180,
        };
    }
    let view = pilot::ActorView {
        movement,
        combat,
        present: true,
        generation: 0,
        deaths: 0,
    };
    let anim = pilot::Animator::preview(age, game.appearance.pose == 5);
    let arena = burnhop_gameplay_core::PRACTICE_ARENA;
    for (part, mut node, mut image, mut transform, mut z) in &mut parts {
        let d = pilot::pose_in(view, anim, part.0, &arena);
        let shown = d.visible
            && !matches!(
                part.0,
                Part::Health | Part::HealthBack | Part::Marker | Part::Bolt
            );
        node.display = if shown { Display::Flex } else { Display::None };
        node.width = px(64. * scale);
        node.height = px(64. * scale);
        node.left = px(base.x + (d.at.x - 32.) * scale);
        node.top = px(base.y - (d.at.y + 32.) * scale);
        transform.scale = d.scale;
        transform.rotation = Rot2::radians(-d.angle);
        image.texture_atlas.as_mut().unwrap().index = if matches!(
            part.0,
            Part::FarUpper | Part::FarFore | Part::NearUpper | Part::NearFore
        ) {
            Tile::Sleeve as usize
        } else {
            d.tile as usize
        };
        image.color = if matches!(part.0, Part::Badge) {
            artwork::color(artwork::CYAN)
        } else {
            d.tint
        };
        *z = ZIndex((d.z * 100.) as i32);
    }
    caption.0.left = px(12.);
    caption.0.top = px(if compact {
        260.
    } else {
        window.height() * 0.82
    });
    caption.0.width = px(width - 24.);
    caption.1.0 = format!(
        "{} / {}\n{}",
        if game.appearance.left {
            "LEFT"
        } else {
            "RIGHT"
        },
        POSES[game.appearance.pose],
        if game.appearance.draft == game.appearance.saved {
            "Saved appearance"
        } else {
            "Unsaved preview"
        }
    );
    caption.2.font_size = if compact { 11. } else { 16. }.into();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        adapter::Key,
        menu::{self, Action, Screen},
    };
    use burnhop_gameplay_core as core;
    #[test]
    fn cosmetics_survive_actual_simulation_lifecycle_without_changing_any_state() {
        let build = |custom: bool| {
            let mut game = Playground::default();
            if custom {
                game.appearance.saved = Appearance::preset(3);
                game.appearance.draft = Appearance::preset(2);
            }
            menu::act(&mut game, Action::Ember);
            let mut app = App::new();
            app.insert_resource(game)
                .init_resource::<Time<Real>>()
                .add_systems(Update, crate::simulate);
            app
        };
        let mut plain = build(false);
        let mut custom = build(true);
        let mut recovered = false;
        for frame in 0..600 {
            for app in [&mut plain, &mut custom] {
                let mut g = app.world_mut().resource_mut::<Playground>();
                match frame {
                    5 => {
                        g.input.push(Key::Right, true);
                    }
                    30 => {
                        g.input.push(Key::Right, false);
                        g.input.push(Key::CrouchC, true);
                    }
                    50 => {
                        g.input.push(Key::CrouchC, false);
                        g.input.push(Key::JetLeft, true);
                    }
                    70 => g.input.push(Key::JetLeft, false),
                    90 => {
                        g.world.player.body.x = 1490.;
                        g.world.player.body.y = 1901.;
                        g.world.player.grounded = false;
                        g.previous = g.world.player;
                    }
                    110 => {
                        g.combat.player.health = 0;
                        g.combat.player.life = core::LifeState::Dead { remaining_ticks: 5 };
                    }
                    150 => {
                        g.menu.screen = Screen::Paused;
                        menu::release(&mut g);
                    }
                    160 => {
                        menu::act(&mut g, Action::Resume);
                    }
                    170 => {
                        g.input.push(Key::Reset, true);
                    }
                    171 => g.input.push(Key::Reset, false),
                    200 => {
                        menu::act(&mut g, Action::Leave);
                    }
                    210 => {
                        menu::act(&mut g, Action::Practice);
                    }
                    250 => g.input.push(Key::Fire, true),
                    270 => g.input.push(Key::Fire, false),
                    280 => g.input.push(Key::Reload, true),
                    281 => g.input.push(Key::Reload, false),
                    _ => {}
                }
                app.world_mut()
                    .resource_mut::<Time<Real>>()
                    .advance_by(std::time::Duration::from_secs_f64(core::DT));
                app.update();
            }
            let a = plain.world().resource::<Playground>();
            let b = custom.world().resource::<Playground>();
            assert_eq!(a.world, b.world, "movement at {frame}");
            assert_eq!(a.combat, b.combat, "combat at {frame}");
            assert_eq!(a.stance, b.stance);
            assert_eq!(a.recovery_tick, b.recovery_tick);
            recovered |= b.recovery_tick.is_some();
            assert_eq!(b.appearance.saved, Appearance::preset(3));
            assert_eq!(b.appearance.draft, Appearance::preset(2));
        }
        assert!(recovered, "fixture exercised fall recovery");
        assert_eq!(
            custom.world().resource::<Playground>().map,
            core::offline::MapId::Range
        );
    }
    #[test]
    fn customization_is_excluded_from_all_online_slots_and_bots() {
        let socket = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        let mut g = Playground::default();
        g.appearance.saved = Appearance::preset(3);
        assert!(uses_saved(&g, 0));
        for actor in 1..8 {
            assert!(!uses_saved(&g, actor));
        }
        g.online = Some(crate::online::Online::new(socket.local_addr().unwrap(), false).unwrap());
        for actor in 0..8 {
            assert!(!uses_saved(&g, actor));
        }
        menu::act(&mut g, Action::Cancel);
        assert_eq!(g.appearance.saved, Appearance::preset(3));
        assert!(uses_saved(&g, 0));
    }
    #[test]
    fn atlas_and_preview_pools_remain_bounded_and_preview_cannot_mutate_gameplay() {
        let mut app = App::new();
        app.insert_resource(Playground::default())
            .init_resource::<Assets<Image>>()
            .init_resource::<Assets<TextureAtlasLayout>>();
        app.add_systems(Startup, (pilot::setup, setup).chain())
            .add_systems(Update, refresh);
        app.update();
        let count = app.world().entities().len();
        let initial = app.world().resource::<Playground>().world;
        for i in 0..24 {
            let mut g = app.world_mut().resource_mut::<Playground>();
            menu::act(&mut g, Action::Character);
            let field = i % 8;
            g.appearance.draft.cycle(field, false);
            g.appearance.saved = Appearance::preset(i % 4);
            app.update();
            assert_eq!(app.world().resource::<Assets<Image>>().len(), 3);
            assert_eq!(
                app.world().resource::<Assets<TextureAtlasLayout>>().len(),
                1
            );
            assert_eq!(app.world().entities().len(), count);
            assert_eq!(app.world().resource::<Playground>().world, initial);
        }
    }
}
