//! Presentation only: the core's confirmed events drive every tracer and flash.
use crate::{Playground, position};
use bevy::{prelude::*, window::PrimaryWindow};
use burnhop_gameplay_core::{
    self as core, ActorId, CombatEvents, Impact, LifeState, Reserve, Shot,
};

#[derive(Default)]
pub struct Feedback {
    pub player_hit: f32,
    bot_hit: f32,
    traces: Vec<(Shot, f32)>,
    pending: Vec<Shot>,
}
impl Feedback {
    pub fn record(&mut self, events: &CombatEvents) {
        if events.movement.reset {
            *self = Self::default();
        }
        if events.player_respawned {
            self.player_hit = 0.0;
        }
        if events.bot_respawned {
            self.bot_hit = 0.0;
        }
        for &shot in events.shots.iter().flatten() {
            match shot.impact {
                Impact::Body(ActorId::One) => self.player_hit = 0.16,
                Impact::Body(ActorId::Two) => self.bot_hit = 0.16,
                _ => {}
            }
            self.pending.push(shot);
        }
    }
}
#[derive(Component)]
pub(super) enum CombatVisual {
    Bot,
    Barrel(ActorId),
    BotHealth,
}
#[derive(Component)]
pub(super) struct Trace;
#[derive(Component)]
pub(super) struct Reticle;

pub fn setup(mut commands: Commands) {
    for visual in [
        CombatVisual::Bot,
        CombatVisual::Barrel(ActorId::One),
        CombatVisual::Barrel(ActorId::Two),
        CombatVisual::BotHealth,
    ] {
        commands.spawn((
            visual,
            Sprite::from_color(Color::WHITE, Vec2::ONE),
            Transform::default(),
        ));
    }
    commands
        .spawn((
            Reticle,
            Node {
                position_type: PositionType::Absolute,
                width: px(16),
                height: px(16),
                ..default()
            },
            Visibility::Hidden,
            GlobalZIndex(10),
        ))
        .with_children(|parent| {
            for (x, y, w, h) in [
                (0., 7., 5., 2.),
                (11., 7., 5., 2.),
                (7., 0., 2., 5.),
                (7., 11., 2., 5.),
            ] {
                parent.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(x),
                        top: px(y),
                        width: px(w),
                        height: px(h),
                        ..default()
                    },
                    BackgroundColor(Color::WHITE),
                ));
            }
        });
}
/// Invert the actual camera projection used for the last displayed frame.
/// Logical coordinates handle Retina scaling. Refuse stale viewport sizes during
/// resize; the next frame has Bevy's refreshed projection. No fixed 1280 shortcut.
pub fn cursor_world(
    camera: &Camera,
    transform: &GlobalTransform,
    cursor: Vec2,
    window_size: Vec2,
) -> Option<core::Vec2> {
    let size = camera.logical_viewport_size()?;
    if !cursor.is_finite()
        || cursor.x < 0.0
        || cursor.y < 0.0
        || cursor.x >= window_size.x
        || cursor.y >= window_size.y
        || (size - window_size).length() > 0.5
    {
        return None;
    }
    let point = camera.viewport_to_world_2d(transform, cursor).ok()?;
    point.is_finite().then_some(core::Vec2 {
        x: f64::from(point.x),
        y: -f64::from(point.y),
    })
}
pub fn capture_aim(
    mut game: ResMut<Playground>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
) {
    if game
        .combat_route
        .as_ref()
        .is_some_and(|route| !route.complete)
    {
        return;
    }
    let point = if game.focused {
        window
            .cursor_position()
            .and_then(|cursor| cursor_world(camera.0, camera.1, cursor, window.size()))
    } else {
        None
    };
    if point.is_none() {
        game.input.clear_fire();
    }
    game.input.aim_at = point;
}
pub fn hud(game: &Playground, movement: &str) -> String {
    let player = &game.combat.player;
    let w = player.weapon();
    let reserve = match w.reserve {
        Reserve::Rounds(n) => n.to_string(),
        Reserve::Unlimited => "unlimited".into(),
    };
    let weapon_status = if !player.alive() {
        "INACTIVE".into()
    } else if w.reload_ticks > 0 {
        format!("RELOADING {:.1}s", f32::from(w.reload_ticks) / 60.)
    } else if player.equip_ticks > 0 {
        "EQUIPPING".into()
    } else if w.ammo == 0 {
        if w.reserve == Reserve::Rounds(0) {
            if game.online.is_some() {
                "EMPTY - switch weapon".into()
            } else {
                "EMPTY - switch or F5".into()
            }
        } else {
            "EMPTY - press R".into()
        }
    } else {
        "READY".into()
    };
    let life = match player.life {
        LifeState::Alive => format!("HP {}/100", player.health),
        LifeState::Dead { remaining_ticks } => {
            format!("DOWN - respawn {:.1}s", f32::from(remaining_ticks) / 60.)
        }
    };
    let bot = if let Some(online) = &game.online {
        if online.remote_present {
            match game.combat.bot.life {
                LifeState::Alive => format!("Opponent HP {}", game.combat.bot.health),
                LifeState::Dead { remaining_ticks } => format!(
                    "Opponent down - respawn {:.1}s",
                    f32::from(remaining_ticks) / 60.
                ),
            }
        } else {
            "Waiting for opponent".into()
        }
    } else {
        match game.combat.bot.life {
            LifeState::Dead { remaining_ticks } => {
                format!("Bot respawn {:.1}s", f32::from(remaining_ticks) / 60.)
            }
            LifeState::Alive => {
                if game.combat.bot_attack_ticks > 60 {
                    format!(
                        "Bot HP {} - grace {:.1}s",
                        game.combat.bot.health,
                        f32::from(game.combat.bot_attack_ticks) / 60.
                    )
                } else {
                    format!("Bot HP {} - 1 shot/sec in sight", game.combat.bot.health)
                }
            }
        }
    };
    let connection = game
        .online
        .as_ref()
        .map_or(String::new(), |online| format!("{}\n", online.label()));
    format!(
        "{connection}{life}   |   {}  {} / {}   |   {weapon_status}\nFuel {:3.0}%   |   {movement}\n{bot}   |   Kills {} / Deaths {}",
        w.id.tuning().name,
        w.ammo,
        reserve,
        game.world.player.fuel,
        game.combat.kills,
        game.combat.deaths
    )
}
#[allow(clippy::too_many_arguments)]
pub fn present(
    mut commands: Commands,
    mut game: ResMut<Playground>,
    time: Res<Time<Real>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut visuals: Query<(&CombatVisual, &mut Transform, &mut Sprite)>,
    old: Query<Entity, With<Trace>>,
    mut reticle: Single<(&mut Node, &mut Visibility), With<Reticle>>,
) {
    let dt = if game.focused || game.online.is_some() {
        time.delta_secs().min(0.1)
    } else {
        0.0
    };
    let feedback = &mut game.feedback;
    feedback.player_hit = (feedback.player_hit - dt).max(0.0);
    feedback.bot_hit = (feedback.bot_hit - dt).max(0.0);
    for (_, ttl) in &mut feedback.traces {
        *ttl -= dt;
    }
    feedback.traces.retain(|(_, ttl)| *ttl > 0.0);
    feedback
        .traces
        .extend(feedback.pending.drain(..).map(|shot| (shot, 0.09)));
    for entity in &old {
        commands.entity(entity).despawn();
    }
    for (shot, _) in &feedback.traces {
        let from = Vec2::new(shot.origin.x as f32, -shot.origin.y as f32);
        let to = Vec2::new(shot.end.x as f32, -shot.end.y as f32);
        let ray = to - from;
        let color = if shot.shooter == ActorId::One {
            Color::srgb(1., 0.86, 0.35)
        } else {
            Color::srgb(1., 0.32, 0.36)
        };
        commands.spawn((
            Trace,
            Sprite::from_color(color, Vec2::new(ray.length().max(1.), 2.)),
            Transform::from_translation(((from + to) / 2.).extend(3.))
                .with_rotation(Quat::from_rotation_z(ray.y.atan2(ray.x))),
        ));
        if shot.impact != Impact::Range {
            commands.spawn((
                Trace,
                Sprite::from_color(
                    if shot.damage > 0 { Color::WHITE } else { color },
                    Vec2::splat(if shot.damage > 0 { 12. } else { 6. }),
                ),
                Transform::from_translation(to.extend(3.1)),
            ));
        }
    }
    let mut player_body = game.world.player.body;
    player_body.x = game.previous.body.x + (player_body.x - game.previous.body.x) * game.alpha;
    player_body.y = game.previous.body.y + (player_body.y - game.previous.body.y) * game.alpha;
    for (visual, mut transform, mut sprite) in &mut visuals {
        let remote_visual = !matches!(visual, CombatVisual::Barrel(ActorId::One));
        if remote_visual
            && game
                .online
                .as_ref()
                .is_some_and(|online| !online.remote_present)
        {
            sprite.color = Color::NONE;
            continue;
        }
        match visual {
            CombatVisual::Bot => {
                transform.translation = position(game.combat.bot_body, 1.);
                sprite.custom_size = Some(Vec2::new(36., 68.));
                sprite.color = if !game.combat.bot.alive() {
                    Color::srgba(0.35, 0.35, 0.38, 0.35)
                } else if game.feedback.bot_hit > 0. {
                    Color::WHITE
                } else {
                    Color::srgb(0.82, 0.25, 0.40)
                };
            }
            CombatVisual::Barrel(id) => {
                let (actor, body) = if *id == ActorId::One {
                    (&game.combat.player, player_body)
                } else {
                    (&game.combat.bot, game.combat.bot_body)
                };
                let aim = Vec2::new(actor.aim.x as f32, -actor.aim.y as f32);
                let length = actor.selected.tuning().barrel as f32 + 12.;
                transform.translation = position(body, 2.) + (aim * length / 2.).extend(0.);
                transform.rotation = Quat::from_rotation_z(aim.y.atan2(aim.x));
                sprite.custom_size = Some(Vec2::new(length, 6.));
                sprite.color = if actor.alive() {
                    Color::srgb(0.89, 0.91, 0.86)
                } else {
                    Color::NONE
                };
            }
            CombatVisual::BotHealth => {
                let body = game.combat.bot_body;
                let width = 48. * f32::from(game.combat.bot.health) / 100.;
                transform.translation = Vec3::new(
                    body.x as f32 + 18. - (48. - width) / 2.,
                    -body.y as f32 + 12.,
                    2.,
                );
                sprite.custom_size = Some(Vec2::new(width, 4.));
                sprite.color = Color::srgb(0.96, 0.45, 0.54);
            }
        }
    }
    if game.focused
        && game.input.aim_at.is_some()
        && game.combat.player.alive()
        && let Some(cursor) = window.cursor_position()
    {
        reticle.0.left = px(cursor.x - 8.);
        reticle.0.top = px(cursor.y - 8.);
        *reticle.1 = Visibility::Visible;
    } else {
        *reticle.1 = Visibility::Hidden;
    }
}
