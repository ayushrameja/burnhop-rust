//! Bounded, reused render slots driven exclusively by confirmed combat events.
use crate::{
    Playground,
    artwork::{CREAM, OCHRE, color},
};
use bevy::{prelude::*, window::PrimaryWindow};
use burnhop_gameplay_core::{self as core, ActorId, CombatEvents, Impact, Shot};
const MAX_SHOTS: usize = 24;
const EFFECT_LIFE: f32 = 0.18;
pub struct Feedback {
    hits: [f32; core::MAX_PLAYERS],
    traces: Vec<(Shot, f32)>,
    pending: Vec<Shot>,
    pub epoch: u64,
}
impl Default for Feedback {
    fn default() -> Self {
        Self {
            hits: [0.; core::MAX_PLAYERS],
            traces: Vec::with_capacity(MAX_SHOTS),
            pending: Vec::with_capacity(MAX_SHOTS),
            epoch: 0,
        }
    }
}
impl Feedback {
    pub fn has_terrain_impact(&self) -> bool {
        self.traces
            .iter()
            .any(|(s, ttl)| s.impact == Impact::Terrain && *ttl > 0.13)
    }
    pub fn has_shot(&self, actor: ActorId, weapon: core::WeaponId) -> bool {
        self.traces
            .iter()
            .any(|(s, ttl)| s.shooter == actor && s.weapon == weapon && *ttl > 0.13)
    }
    pub fn kick(&self, actor: usize) -> f32 {
        self.traces
            .iter()
            .filter(|(s, _)| s.shooter.index() == actor)
            .map(|(_, ttl)| ((ttl - 0.10) / 0.08).max(0.) * 1.5)
            .fold(0., f32::max)
    }
    pub fn hit(&self, actor: usize) -> f32 {
        self.hits[actor]
    }
    pub fn clear_actor(&mut self, actor: ActorId) {
        self.traces.retain(|(s, _)| {
            s.shooter != actor && !matches!(s.impact,Impact::Body(id) if id==actor)
        });
        self.pending
            .retain(|s| s.shooter != actor && !matches!(s.impact,Impact::Body(id) if id==actor));
        self.hits[actor.index()] = 0.;
    }
    pub fn record(&mut self, events: &CombatEvents) {
        if events.movement.reset {
            let epoch = self.epoch.wrapping_add(1);
            *self = Self { epoch, ..default() };
        }
        if events.player_respawned || events.player_died {
            self.clear_actor(ActorId::One);
        }
        if events.bot_respawned || events.bot_died {
            self.clear_actor(ActorId::Two);
        }
        self.record_shots(events.shots.iter().flatten().copied());
    }
    pub fn record_shots(&mut self, shots: impl Iterator<Item = Shot>) {
        for shot in shots {
            if let Impact::Body(id) = shot.impact {
                self.hits[id.index()] = 0.12;
            }
            if self.pending.len() < MAX_SHOTS {
                self.pending.push(shot);
            }
        }
    }
    fn advance(&mut self, dt: f32) {
        for hit in &mut self.hits {
            *hit = (*hit - dt).max(0.);
        }
        for (_, ttl) in &mut self.traces {
            *ttl -= dt;
        }
        self.traces.retain(|(_, ttl)| *ttl > 0.);
        for shot in self.pending.drain(..) {
            if self.traces.len() == MAX_SHOTS {
                self.traces.remove(0);
            }
            self.traces.push((shot, EFFECT_LIFE));
        }
    }
}
#[derive(Component)]
pub struct Effect {
    slot: usize,
    part: usize,
}
#[derive(Component)]
pub struct Reticle;
type ReticleFilter = (With<Reticle>, Without<Effect>);
pub fn setup(mut commands: Commands) {
    for slot in 0..MAX_SHOTS {
        for part in 0..5 {
            commands.spawn((
                Effect { slot, part },
                Sprite::from_color(Color::WHITE, Vec2::ONE),
                Transform::default(),
                Visibility::Hidden,
            ));
        }
    }
    commands
        .spawn((
            Reticle,
            Node {
                position_type: PositionType::Absolute,
                width: px(18),
                height: px(18),
                ..default()
            },
            Visibility::Hidden,
            GlobalZIndex(10),
        ))
        .with_children(|parent| {
            for (x, y, w, h) in [
                (0., 8., 5., 2.),
                (13., 8., 5., 2.),
                (8., 0., 2., 5.),
                (8., 13., 2., 5.),
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
                    BackgroundColor(color(CREAM)),
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
pub fn present(
    mut game: ResMut<Playground>,
    time: Res<Time<Real>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut effects: Query<(&Effect, &mut Sprite, &mut Transform, &mut Visibility)>,
    mut reticle: Single<(&mut Node, &mut Visibility), ReticleFilter>,
) {
    let dt = if game.focused || game.online.is_some() {
        time.delta_secs().min(0.1)
    } else {
        0.
    };
    game.feedback.advance(dt);
    for (effect, mut sprite, mut t, mut visible) in &mut effects {
        *visible = Visibility::Hidden;
        let Some((shot, ttl)) = game.feedback.traces.get(effect.slot) else {
            continue;
        };
        let age = EFFECT_LIFE - ttl;
        let from = Vec2::new(shot.origin.x as f32, -shot.origin.y as f32);
        let to = Vec2::new(shot.end.x as f32, -shot.end.y as f32);
        let ray = to - from;
        let unit = ray.normalize_or_zero();
        let hue = color(if shot.shooter == game.local_id() {
            CREAM
        } else {
            OCHRE
        });
        t.scale = Vec3::ONE;
        sprite.color = hue.with_alpha((ttl / EFFECT_LIFE).min(0.85));
        match effect.part {
            0 if age < 0.085 => {
                t.translation = ((from + to) * 0.5).extend(3.);
                t.rotation = Quat::from_rotation_z(ray.y.atan2(ray.x));
                sprite.custom_size = Some(Vec2::new(ray.length().max(0.1), 1.2));
            }
            // Origin flash fits strictly within the confirmed segment, even when
            // the origin is touching cover. It never jumps to a decorative muzzle.
            1 if age < 0.045 && ray.length() > 0.5 => {
                let len = ray.length().min(7.);
                t.translation = (from + unit * len * 0.5).extend(3.1);
                t.rotation = Quat::from_rotation_z(ray.y.atan2(ray.x));
                sprite.custom_size = Some(Vec2::new(len, 3.));
                sprite.color = color(CREAM);
            }
            2..=4 if shot.impact != Impact::Range => {
                let branch = effect.part as f32 - 3.;
                let normal = Vec2::new(-unit.y, unit.x);
                let offset = -unit * (age * 26. + 1.) + normal * branch * age * 30.;
                t.translation = (to + offset).extend(3.2);
                t.rotation = Quat::from_rotation_z(branch * 0.8 + ray.y.atan2(ray.x));
                sprite.custom_size = Some(Vec2::new(if shot.damage > 0 { 3. } else { 2. }, 1.));
            }
            _ => continue,
        }
        *visible = Visibility::Visible;
    }
    if game.focused
        && game.input.aim_at.is_some()
        && game.combat.player.alive()
        && let Some(cursor) = window.cursor_position()
    {
        reticle.0.left = px(cursor.x - 9.);
        reticle.0.top = px(cursor.y - 9.);
        *reticle.1 = Visibility::Visible;
    } else {
        *reticle.1 = Visibility::Hidden;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn shot() -> Shot {
        Shot {
            shooter: ActorId::One,
            weapon: core::WeaponId::Pistol,
            origin: core::Vec2 { x: 1., y: 1. },
            end: core::Vec2 { x: 20., y: 1. },
            impact: Impact::Body(ActorId::Two),
            damage: 18,
        }
    }
    #[test]
    fn sustained_batches_remain_bounded_and_expire() {
        let mut f = Feedback::default();
        let e = CombatEvents {
            shots: [Some(shot()); 2],
            ..default()
        };
        for _ in 0..1000 {
            f.record(&e);
        }
        assert_eq!(f.pending.len(), MAX_SHOTS);
        for _ in 0..100 {
            f.record(&e);
            f.advance(0.001);
            assert!(f.traces.len() <= MAX_SHOTS);
        }
        f.advance(EFFECT_LIFE + 0.1);
        assert!(f.traces.is_empty());
    }
    #[test]
    fn eight_shooters_keep_effects_bounded_and_cleanup_is_per_slot() {
        let mut f = Feedback::default();
        for _ in 0..6000 {
            f.record_shots(ActorId::ALL.into_iter().map(|id| Shot {
                shooter: id,
                impact: Impact::Body(ActorId::ALL[(id.index() + 1) % core::MAX_PLAYERS]),
                ..shot()
            }));
            f.advance(1. / 60.);
            assert!(f.traces.len() <= MAX_SHOTS && f.pending.len() <= MAX_SHOTS);
        }
        for id in ActorId::ALL {
            f.clear_actor(id);
            assert_eq!(f.hit(id.index()), 0.);
            assert!(
                f.traces
                    .iter()
                    .all(|(s, _)| s.shooter != id && s.impact != Impact::Body(id))
            );
        }
        assert!(f.traces.is_empty());
    }
    #[test]
    fn lifecycle_clears_pending_active_hits_and_reset_epoch() {
        let mut f = Feedback::default();
        f.record(&CombatEvents {
            shots: [Some(shot()), None],
            ..default()
        });
        f.advance(0.);
        f.record(&CombatEvents {
            shots: [Some(shot()), None],
            ..default()
        });
        f.clear_actor(ActorId::Two);
        assert!(f.pending.is_empty() && f.traces.is_empty());
        assert_eq!(f.hits[1], 0.);
        f.record(&CombatEvents {
            movement: core::StepEvents {
                reset: true,
                ..default()
            },
            ..default()
        });
        assert_eq!(f.epoch, 1);
    }
}
