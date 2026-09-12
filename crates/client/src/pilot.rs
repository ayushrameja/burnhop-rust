//! Shared articulated pilot for practice, predicted local movement and remote snapshots.
use crate::{
    Playground,
    artwork::{Artwork, CYAN, OCHRE, Tile, color},
};
use bevy::prelude::*;
use burnhop_gameplay_core::{self as core, Combatant, Player, WeaponId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Motion {
    Idle,
    Run,
    Air,
    Jet,
    Dead,
}
pub fn motion(p: Player, alive: bool) -> Motion {
    if !alive {
        Motion::Dead
    } else if p.thrusting {
        Motion::Jet
    } else if !p.grounded {
        Motion::Air
    } else if p.velocity.x.abs() > 4. {
        Motion::Run
    } else {
        Motion::Idle
    }
}
#[derive(Clone, Copy)]
struct ActorView {
    movement: Player,
    combat: Combatant,
    present: bool,
    generation: u64,
}
fn actors(game: &Playground) -> [ActorView; 2] {
    let mut local = game.world.player;
    local.body.x = game.previous.body.x + (local.body.x - game.previous.body.x) * game.alpha;
    local.body.y = game.previous.body.y + (local.body.y - game.previous.body.y) * game.alpha;
    let mut remote = game.world.player;
    remote.body = game.combat.bot_body;
    remote.velocity = core::Vec2::default();
    remote.grounded = true;
    remote.thrusting = false;
    let mut generation = 0;
    if let Some(online) = &game.online {
        if let Some(p) = online.remote_movement {
            remote = p;
        }
        generation = online.remote_generation;
    }
    [
        ActorView {
            movement: local,
            combat: game.combat.player,
            present: game.online.as_ref().is_none_or(|o| {
                o.prediction.as_ref().is_some_and(|p| p.local.is_some())
                    && !o.network.status.terminal()
            }),
            generation: 0,
        },
        ActorView {
            movement: remote,
            combat: game.combat.bot,
            present: game.online.as_ref().is_none_or(|o| o.remote_present),
            generation,
        },
    ]
}
#[derive(Clone, Copy)]
enum Part {
    Pack,
    FarThigh,
    FarShin,
    FarBoot,
    Torso,
    NearThigh,
    NearShin,
    NearBoot,
    Head,
    FarUpper,
    FarFore,
    NearUpper,
    NearFore,
    Weapon,
    Magazine,
    Bolt,
    FarHand,
    NearHand,
    Badge,
    FarJet,
    NearJet,
    HealthBack,
    Health,
    Marker,
}
const PARTS: [Part; 24] = [
    Part::Pack,
    Part::FarThigh,
    Part::FarShin,
    Part::FarBoot,
    Part::Torso,
    Part::NearThigh,
    Part::NearShin,
    Part::NearBoot,
    Part::Head,
    Part::FarUpper,
    Part::FarFore,
    Part::NearUpper,
    Part::NearFore,
    Part::Weapon,
    Part::Magazine,
    Part::Bolt,
    Part::FarHand,
    Part::NearHand,
    Part::Badge,
    Part::FarJet,
    Part::NearJet,
    Part::HealthBack,
    Part::Health,
    Part::Marker,
];
#[derive(Component)]
pub struct PilotPart {
    actor: usize,
    part: Part,
}
#[derive(Component)]
pub struct PilotLabel(usize);
#[derive(Clone, Copy, Default)]
struct Animator {
    last: Option<Vec2>,
    phase: f32,
    age: f32,
    death: f32,
    alive: bool,
    present: bool,
    generation: u64,
    epoch: u64,
}
impl Animator {
    fn advance(&mut self, view: ActorView, epoch: u64, dt: f32) -> bool {
        let pos = Vec2::new(view.movement.body.x as f32, view.movement.body.y as f32);
        let changed = self.present != view.present
            || self.alive != view.combat.alive()
            || self.generation != view.generation
            || self.epoch != epoch
            || self.last.is_some_and(|last| last.distance(pos) > 80.);
        if changed {
            *self = Self {
                alive: view.combat.alive(),
                present: view.present,
                generation: view.generation,
                epoch,
                ..default()
            };
        }
        if motion(view.movement, view.combat.alive()) == Motion::Run
            && let Some(last) = self.last
        {
            self.phase = (self.phase + (pos.x - last.x).abs() / 28.).rem_euclid(1.);
        }
        self.last = Some(pos);
        self.age += dt;
        if !view.combat.alive() {
            self.death += dt;
        }
        changed
    }
}
#[derive(Resource, Default)]
pub struct Animation([Animator; 2]);
#[derive(Clone, Copy)]
struct Draw {
    at: Vec2,
    angle: f32,
    scale: Vec2,
    tile: Tile,
    visible: bool,
    tint: Color,
    z: f32,
}
impl Draw {
    fn new(at: Vec2, tile: Tile, z: f32) -> Self {
        Self {
            at,
            angle: 0.,
            scale: Vec2::ONE,
            tile,
            visible: true,
            tint: Color::WHITE,
            z,
        }
    }
}
fn joint(a: Vec2, b: Vec2, length: f32, bend: f32) -> Vec2 {
    let delta = b - a;
    let d = delta.length().max(0.001);
    let mid = (a + b) * 0.5;
    mid + Vec2::new(-delta.y, delta.x) / d * (length * length - d * d * 0.25).max(0.).sqrt() * bend
}
/// Half a stride is a planted foot: its world X remains fixed as the body travels.
fn foot(phase: f32, travel_sign: f32, rest: f32) -> Vec2 {
    let p = phase.rem_euclid(1.);
    if p < 0.5 {
        Vec2::new(rest + (7. - 28. * p) * travel_sign, 5.)
    } else {
        let t = (p - 0.5) * 2.;
        Vec2::new(
            rest + (-7. + 14. * t) * travel_sign,
            5. + (std::f32::consts::PI * t).sin() * 5.,
        )
    }
}
fn segment(a: Vec2, b: Vec2, z: f32) -> Draw {
    let v = b - a;
    let mut d = Draw::new((a + b) / 2., Tile::Limb, z);
    d.angle = v.y.atan2(v.x) - std::f32::consts::FRAC_PI_2;
    d.scale.y = v.length() / 16.;
    d
}
/// Read-only cover query. Artwork clips to the same center-origin terrain interval.
fn clearance(body: core::Rect, aim: core::Vec2, barrel: f32) -> f32 {
    let origin = core::body_center(body);
    core::PRACTICE_ARENA
        .solids
        .iter()
        .filter_map(|r| core::ray_rect(origin, aim, *r, f64::from(barrel)))
        .fold(barrel, |m, d| m.min(d as f32))
        .max(0.)
}
pub fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let art = crate::artwork::create(&mut images, &mut layouts);
    for actor in 0..2 {
        for part in PARTS {
            commands.spawn((
                PilotPart { actor, part },
                art.sprite(Tile::Torso),
                Transform::default(),
                Visibility::Hidden,
            ));
        }
        commands.spawn((
            PilotLabel(actor),
            Text2d::new(if actor == 0 { "YOU" } else { "BOT" }),
            TextFont {
                font_size: FontSize::Px(8.),
                ..default()
            },
            TextColor(color(if actor == 0 { CYAN } else { OCHRE })),
            Transform::default(),
            Visibility::Hidden,
        ));
    }
    commands.insert_resource(art);
    commands.init_resource::<Animation>();
}
fn pose(view: ActorView, anim: Animator, part: Part) -> Draw {
    let p = view.movement;
    let c = view.combat;
    let state = motion(p, c.alive());
    let facing = if c.aim.x < 0. { -1. } else { 1. };
    let aim = Vec2::new(c.aim.x as f32, -c.aim.y as f32);
    let angle = aim.y.atan2(aim.x);
    let origin = Vec2::new(0., 34.);
    let weapon_point = |x: f32, y: f32| origin + aim * x + Vec2::new(-aim.y, aim.x) * y * facing;
    let reload = if c.weapon().reload_ticks > 0 {
        1. - f32::from(c.weapon().reload_ticks) / f32::from(c.selected.tuning().reload)
    } else {
        0.
    };
    let mag_travel = (reload * std::f32::consts::PI).sin() * 7.;
    let breath = if state == Motion::Idle {
        (anim.age * 2.).sin() * 0.4
    } else {
        0.
    };
    let travel = if p.velocity.x < 0. { -1. } else { 1. };
    let legs = [(-5., 0.5), (5., 0.)].map(|(rest, offset)| {
        let hip = Vec2::new(rest, 26.);
        let ankle = match state {
            Motion::Run => foot(anim.phase + offset, travel, rest),
            Motion::Air => Vec2::new(rest - 3. * facing, if rest < 0. { 12. } else { 8. }),
            Motion::Jet => Vec2::new(rest * 1.2, 7.),
            Motion::Dead => Vec2::new(rest * 1.4, 5.),
            _ => Vec2::new(rest, 5.),
        };
        let knee = joint(hip, ankle, 13., facing);
        (hip, knee, ankle)
    });
    let shoulder = Vec2::new(-5. * facing, 42. + breath);
    let near_hand = weapon_point(if c.selected == WeaponId::M416 { 4. } else { 2. }, -5.);
    let far_hand = weapon_point(
        if c.selected == WeaponId::M416 {
            15.
        } else {
            5.
        },
        -3. - mag_travel,
    );
    let near_elbow = joint(shoulder, near_hand, 12., -facing);
    let far_shoulder = Vec2::new(3. * facing, 42. + breath);
    let far_elbow = joint(far_shoulder, far_hand, 14., -facing);
    let mut d = match part {
        Part::Pack => Draw::new(Vec2::new(-12. * facing, 36.), Tile::Pack, 1.0),
        Part::Torso => Draw::new(Vec2::new(0., 35.), Tile::Torso, 1.4),
        Part::Head => {
            let mut d = Draw::new(Vec2::new(0., 52. + breath), Tile::Head, 1.6);
            d.angle = (aim.y * 0.10) * facing;
            d
        }
        Part::FarThigh => segment(legs[0].0, legs[0].1, 1.1),
        Part::FarShin => segment(legs[0].1, legs[0].2, 1.1),
        Part::NearThigh => segment(legs[1].0, legs[1].1, 1.5),
        Part::NearShin => segment(legs[1].1, legs[1].2, 1.5),
        Part::FarBoot => Draw::new(legs[0].2, Tile::Boot, 1.2),
        Part::NearBoot => Draw::new(legs[1].2, Tile::Boot, 1.6),
        Part::FarUpper => segment(far_shoulder, far_elbow, 1.3),
        Part::FarFore => segment(far_elbow, far_hand, 1.3),
        Part::NearUpper => segment(shoulder, near_elbow, 1.7),
        Part::NearFore => segment(near_elbow, near_hand, 1.7),
        Part::Weapon => {
            let mut d = Draw::new(
                origin,
                if c.selected == WeaponId::Pistol {
                    Tile::Pistol
                } else {
                    Tile::Rifle
                },
                1.8,
            );
            d.angle = angle;
            d.scale.y = facing;
            d
        }
        Part::Bolt => {
            let mut d = Draw::new(weapon_point(7., 1.), Tile::Badge, 1.95);
            d.angle = angle;
            d.visible = c.alive();
            d
        }
        Part::Magazine => {
            let mut d = Draw::new(
                weapon_point(
                    if c.selected == WeaponId::Pistol {
                        2.
                    } else {
                        9.
                    },
                    -7. - mag_travel,
                ),
                Tile::Magazine,
                1.75,
            );
            d.angle = angle;
            d.scale.y = facing;
            d.visible = c.alive();
            d
        }
        Part::NearHand => Draw::new(near_hand, Tile::Hand, 1.9),
        Part::FarHand => Draw::new(far_hand, Tile::Hand, 1.9),
        Part::Badge => {
            let mut d = Draw::new(Vec2::new(-7. * facing, 39.), Tile::Badge, 1.65);
            d.scale = Vec2::splat(0.42);
            d
        }
        Part::FarJet | Part::NearJet => {
            let leg = if matches!(part, Part::FarJet) { 0 } else { 1 };
            let mut d = Draw::new(legs[leg].2 - Vec2::new(0., 12.), Tile::Exhaust, 0.9);
            d.visible = state == Motion::Jet;
            d.scale.y = 0.75 + 0.12 * (anim.age * 27.).sin();
            d
        }
        Part::HealthBack | Part::Health | Part::Marker => {
            let mut d = Draw::new(Vec2::new(0., 76.), Tile::Badge, 2.);
            d.visible = c.alive();
            d
        }
    };
    if matches!(
        part,
        Part::Pack | Part::Torso | Part::Head | Part::FarBoot | Part::NearBoot
    ) {
        d.scale.x *= facing;
    }
    // No cosmetic barrel can extend past nearby cover. The clipped texture keeps
    // its pivot (and all hand attachments) fixed; this does not shorten any ray.
    if matches!(
        part,
        Part::Weapon
            | Part::Bolt
            | Part::Magazine
            | Part::NearHand
            | Part::FarHand
            | Part::NearFore
            | Part::FarFore
    ) && clearance(p.body, c.aim, c.selected.tuning().barrel as f32) < 1.
    {
        d.visible = false;
    }
    if state == Motion::Dead {
        let k = (anim.death / 0.24).clamp(0., 1.);
        let angle = -facing * 0.9 * k;
        let pivot = Vec2::new(0., 5.);
        let rel = d.at - pivot;
        d.at = pivot
            + Vec2::new(
                rel.x * angle.cos() - rel.y * angle.sin(),
                rel.x * angle.sin() + rel.y * angle.cos(),
            );
        d.angle += angle;
        d.tint = Color::srgba(0.70, 0.69, 0.59, (0.55 - anim.death * 0.7).max(0.));
        if matches!(
            part,
            Part::Weapon
                | Part::Magazine
                | Part::FarJet
                | Part::NearJet
                | Part::Health
                | Part::HealthBack
                | Part::Marker
        ) {
            d.visible = false;
        }
    }
    d
}
#[allow(clippy::too_many_arguments)]
pub fn present(
    mut game: ResMut<Playground>,
    time: Res<Time<Real>>,
    art: Res<Artwork>,
    mut animation: ResMut<Animation>,
    mut parts: Query<(&PilotPart, &mut Sprite, &mut Transform, &mut Visibility)>,
    mut labels: Query<
        (&PilotLabel, &mut Text2d, &mut Transform, &mut Visibility),
        Without<PilotPart>,
    >,
) {
    let views = actors(&game);
    let dt = if game.focused || game.online.is_some() {
        time.delta_secs().min(0.1)
    } else {
        0.
    };
    for (i, view) in views.iter().enumerate() {
        let anim = &mut animation.0[i];
        if anim.advance(*view, game.feedback.epoch, dt) {
            game.feedback.clear_actor(if i == 0 {
                core::ActorId::One
            } else {
                core::ActorId::Two
            });
        }
    }
    for (part, mut sprite, mut t, mut visible) in &mut parts {
        let view = views[part.actor];
        let d = pose(view, animation.0[part.actor], part.part);
        *visible = if view.present && d.visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        *sprite = art.sprite(d.tile);
        sprite.color = d.tint;
        let base = Vec2::new(
            (view.movement.body.x + 18.) as f32,
            -(view.movement.body.y + 68.) as f32,
        );
        t.translation = (base + d.at).extend(d.z);
        t.rotation = Quat::from_rotation_z(d.angle);
        t.scale = d.scale.extend(1.);
        let accent = if part.actor == 0 { CYAN } else { OCHRE };
        if matches!(part.part, Part::Badge | Part::Marker) {
            sprite.color = color(accent);
        }
        if game.feedback.hit(part.actor) > 0.
            && view.combat.alive()
            && !matches!(part.part, Part::FarJet | Part::NearJet)
        {
            sprite.color = Color::srgb(1., 0.77, 0.62);
        }
        if matches!(part.part, Part::HealthBack | Part::Health | Part::Marker) {
            sprite.image = Handle::default();
            sprite.texture_atlas = None;
            let width = if matches!(part.part, Part::Health) {
                32. * f32::from(view.combat.health) / 100.
            } else {
                32.
            };
            sprite.custom_size = Some(Vec2::new(
                width,
                if matches!(part.part, Part::Marker) {
                    1.
                } else {
                    3.
                },
            ));
            sprite.color = color(if matches!(part.part, Part::HealthBack) {
                0x202b29
            } else {
                accent
            });
            t.scale = Vec3::ONE;
            t.rotation = Quat::IDENTITY;
            t.translation.x -= if matches!(part.part, Part::Health) {
                (32. - width) / 2.
            } else {
                0.
            };
            t.translation.y += if matches!(part.part, Part::Marker) {
                15.
            } else {
                0.
            };
        }
        if matches!(part.part, Part::Bolt) {
            sprite.image = Handle::default();
            sprite.texture_atlas = None;
            sprite.custom_size = Some(Vec2::new(2., 0.8));
            sprite.color = crate::artwork::color(crate::artwork::CREAM);
            let kick = game.feedback.kick(part.actor);
            t.translation -= Vec3::new(d.angle.cos() * kick, d.angle.sin() * kick, 0.);
            if clearance(
                view.movement.body,
                view.combat.aim,
                view.combat.selected.tuning().barrel as f32,
            ) < 9.
            {
                *visible = Visibility::Hidden;
            }
        }
        if matches!(part.part, Part::Weapon) {
            // Crop the atlas at the terrain intersection, retaining the left edge
            // and moving the quad center to leave local x=0 exactly at the origin.
            let allowed = clearance(
                view.movement.body,
                view.combat.aim,
                view.combat.selected.tuning().barrel as f32,
            );
            if allowed < view.combat.selected.tuning().barrel as f32 {
                let width = 32. + allowed;
                let index = d.tile as usize;
                let x = (index % 4) * 256;
                let y = (index / 4) * 256;
                sprite.texture_atlas = None;
                sprite.rect = Some(Rect::new(
                    x as f32,
                    y as f32,
                    x as f32 + width * 4.,
                    y as f32 + 256.,
                ));
                sprite.custom_size = Some(Vec2::new(width, 64.));
                let shift = (allowed - 32.) / 2.;
                t.translation += Vec3::new(d.angle.cos() * shift, d.angle.sin() * shift, 0.);
            }
        }
    }
    for (label, mut text, mut t, mut visible) in &mut labels {
        let v = views[label.0];
        text.0 = if label.0 == 0 {
            "YOU"
        } else if game.online.is_some() {
            "RIVAL"
        } else {
            "BOT"
        }
        .into();
        t.translation = Vec3::new(
            (v.movement.body.x + 18.) as f32,
            -v.movement.body.y as f32 + 16.,
            2.,
        );
        *visible = if v.present && v.combat.alive() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn state_priority_respects_life_support_and_thrust() {
        let mut p = core::World::new(&core::PRACTICE_ARENA).player;
        assert_eq!(motion(p, true), Motion::Idle);
        p.velocity.x = 50.;
        assert_eq!(motion(p, true), Motion::Run);
        p.grounded = false;
        assert_eq!(motion(p, true), Motion::Air);
        p.thrusting = true;
        assert_eq!(motion(p, true), Motion::Jet);
        assert_eq!(motion(p, false), Motion::Dead);
    }
    #[test]
    fn stance_cancels_world_displacement_and_never_sinks() {
        for sign in [-1., 1.] {
            let a = foot(0.1, sign, 5.);
            let b = foot(0.3, sign, 5.);
            assert!((b.x - a.x + 0.2 * 28. * sign).abs() < 0.001);
            for i in 0..100 {
                assert!(foot(i as f32 / 100., sign, 5.).y >= 5.);
            }
        }
    }
    #[test]
    fn legs_retain_length_for_entire_stride() {
        for i in 0..200 {
            let hip = Vec2::new(5., 26.);
            let ankle = foot(i as f32 / 200., 1., 5.);
            let knee = joint(hip, ankle, 13., 1.);
            assert!((hip.distance(knee) - 13.).abs() < 0.001);
            assert!((ankle.distance(knee) - 13.).abs() < 0.001);
        }
    }
    #[test]
    fn artwork_cover_query_cannot_skip_near_wall() {
        let body = core::Rect::new(2364., 1152., 36., 68.);
        assert_eq!(clearance(body, core::Vec2 { x: 1., y: 0. }, 27.), 18.);
    }
    #[test]
    fn reused_actor_slot_and_reset_discard_pose_history() {
        let mut a = Animator::default();
        let mut view = ActorView {
            movement: core::World::new(&core::PRACTICE_ARENA).player,
            combat: Combatant::new(false),
            present: true,
            generation: 1,
        };
        assert!(a.advance(view, 0, 0.01));
        view.movement.velocity.x = 200.;
        view.movement.body.x += 7.;
        assert!(!a.advance(view, 0, 0.01));
        assert!(a.phase > 0.);
        view.present = false;
        assert!(a.advance(view, 0, 0.01));
        view.present = true;
        view.generation = 2;
        assert!(a.advance(view, 0, 0.01));
        assert_eq!(a.phase, 0.);
        assert_eq!(a.death, 0.);
        view.movement.body.x += 3.;
        a.advance(view, 0, 0.01);
        assert!(a.phase > 0.);
        assert!(a.advance(view, 1, 0.01));
        assert_eq!(a.phase, 0.);
    }
    #[test]
    fn articulated_arms_keep_constant_length_at_all_aims_and_reload_phases() {
        let mut view = ActorView {
            movement: core::World::new(&core::PRACTICE_ARENA).player,
            combat: Combatant::new(false),
            present: true,
            generation: 1,
        };
        for weapon in [WeaponId::Pistol, WeaponId::M416] {
            view.combat.selected = weapon;
            for reload in [0, weapon.tuning().reload / 2, weapon.tuning().reload] {
                view.combat.weapons[weapon.index()].reload_ticks = reload;
                for step in 0..360 {
                    let angle = (step as f64).to_radians();
                    view.combat.aim = core::Vec2 {
                        x: angle.cos(),
                        y: angle.sin(),
                    };
                    for (part, length) in [
                        (Part::NearUpper, 12.),
                        (Part::NearFore, 12.),
                        (Part::FarUpper, 14.),
                        (Part::FarFore, 14.),
                    ] {
                        let d = pose(view, Animator::default(), part);
                        assert!((d.scale.y * 16. - length).abs() < 0.002);
                    }
                }
            }
        }
    }
}
