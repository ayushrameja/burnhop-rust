//! Decorative range treatment. Only the unchanged core arena defines collision.
use crate::{
    artwork::{CREAM, CYAN, color},
    position,
};
use bevy::{asset::RenderAssetUsages, mesh::PrimitiveTopology, prelude::*};
use burnhop_gameplay_core::PRACTICE_ARENA;
#[derive(Component)]
pub struct Depth {
    home: Vec3,
    factor: f32,
}
fn slab(commands: &mut Commands, x: f32, y: f32, w: f32, h: f32, z: f32, c: u32) {
    commands.spawn((
        Sprite::from_color(color(c), Vec2::new(w, h)),
        Transform::from_xyz(x + w / 2., -y - h / 2., z),
    ));
}
fn polygon(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    points: &[[f32; 2]],
    z: f32,
    c: u32,
    depth: Option<f32>,
) {
    let mut vertices = Vec::new();
    for i in 1..points.len() - 1 {
        for p in [points[0], points[i], points[i + 1]] {
            vertices.push([p[0], -p[1], 0.]);
        }
    }
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    let normals = vec![[0., 0., 1.]; vertices.len()];
    let uvs = vec![[0., 0.]; vertices.len()];
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    let mut e = commands.spawn((
        Mesh2d(meshes.add(mesh)),
        MeshMaterial2d(materials.add(color(c))),
        Transform::from_xyz(0., 0., z),
    ));
    if let Some(factor) = depth {
        e.insert(Depth {
            home: Vec3::new(0., 0., z),
            factor,
        });
    }
}
fn label(commands: &mut Commands, text: &str, x: f32, y: f32, size: f32, c: u32) {
    commands.spawn((
        Text2d::new(text),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(color(c)),
        Transform::from_xyz(x, -y, 0.6),
    ));
}
pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Large quiet shapes; no bright horizontal edges that can pass for platforms.
    for (layer, base, peaks, c, factor) in [
        (
            0,
            1280.,
            vec![
                (-900., 980.),
                (-200., 790.),
                (120., 890.),
                (470., 600.),
                (800., 900.),
                (1190., 890.),
                (1580., 830.),
                (1930., 540.),
                (2300., 760.),
                (3000., 630.),
                (3800., 960.),
            ],
            0x293b39,
            0.18,
        ),
        (
            1,
            1380.,
            vec![
                (-700., 1100.),
                (-100., 980.),
                (300., 1060.),
                (680., 865.),
                (1020., 1060.),
                (1400., 900.),
                (1790., 1100.),
                (2210., 900.),
                (2800., 1080.),
                (3600., 940.),
            ],
            0x344943,
            0.35,
        ),
        (
            2,
            1450.,
            vec![
                (-500., 1170.),
                (150., 1090.),
                (500., 1180.),
                (870., 1050.),
                (1280., 1160.),
                (1700., 1080.),
                (2120., 1190.),
                (2900., 1080.),
                (3600., 1210.),
            ],
            0x2c4039,
            0.6,
        ),
    ] {
        for pair in peaks.windows(2) {
            let a = pair[0];
            let b = pair[1];
            polygon(
                &mut commands,
                &mut meshes,
                &mut materials,
                &[[a.0, a.1], [b.0, b.1], [b.0, base], [a.0, base]],
                -10. + layer as f32,
                c,
                Some(factor),
            );
        }
    }
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(54.))),
        MeshMaterial2d(materials.add(color(0x7b8870))),
        Transform::from_xyz(1040., -735., -11.),
        Depth {
            home: Vec3::new(1040., -735., -11.),
            factor: 0.08,
        },
    ));
    polygon(
        &mut commands,
        &mut meshes,
        &mut materials,
        &[[470., 600.], [550., 815.], [800., 900.]],
        -9.9,
        0x30413c,
        Some(0.18),
    );
    polygon(
        &mut commands,
        &mut meshes,
        &mut materials,
        &[[680., 865.], [795., 1030.], [1020., 1060.]],
        -8.9,
        0x3a5046,
        Some(0.35),
    );
    // Sparse distant shrubs and survey masts, always behind the actual arena.
    for i in 0..23 {
        let x = i as f32 * 121. - 100.;
        let h = 28. + (i * 17 % 43) as f32;
        polygon(
            &mut commands,
            &mut meshes,
            &mut materials,
            &[
                [x - 15., 1218.],
                [x, 1218. - h],
                [x + 7., 1200.],
                [x + 19., 1218.],
            ],
            -3.,
            0x243830,
            Some(0.8),
        );
    }
    for (i, rect) in PRACTICE_ARENA.solids.iter().enumerate() {
        commands.spawn((
            Sprite::from_color(
                color(0x41473d),
                Vec2::new(rect.width as f32, rect.height as f32),
            ),
            Transform::from_translation(position(*rect, 0.)),
        ));
        let (x, y, w, h) = (
            rect.x as f32,
            rect.y as f32,
            rect.width as f32,
            rect.height as f32,
        );
        if i < 5 {
            slab(&mut commands, x, y, w, 3., 0.4, 0xb4bc8b);
            slab(&mut commands, x, y + 3., w, 8., 0.3, 0x7a8b63);
            slab(&mut commands, x, y + 11., w, 3., 0.3, 0x29352e);
            if i < 4 {
                slab(
                    &mut commands,
                    x + 1.,
                    y + 15.,
                    w - 2.,
                    h - 17.,
                    0.1,
                    0x626451,
                );
                for n in 0..(w as usize / 65) {
                    let bx = x + n as f32 * 65.;
                    polygon(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &[
                            [bx + 3., y + 16.],
                            [bx + 39., y + 17.],
                            [bx + 61., y + h - 4.],
                            [bx + 16., y + h - 3.],
                        ],
                        0.2,
                        if n % 2 == 0 { 0x6e7059 } else { 0x515845 },
                        None,
                    );
                }
                slab(&mut commands, x + 12., y + 7., 22., 3., 0.5, CREAM);
                slab(&mut commands, x + w - 34., y + 7., 22., 3., 0.5, CREAM);
                for n in 0..3 {
                    slab(
                        &mut commands,
                        x + 46. + n as f32 * 8.,
                        y + h - 9.,
                        3.,
                        3.,
                        0.5,
                        0x343c32,
                    );
                }
                label(
                    &mut commands,
                    &format!("0{}", i + 1),
                    x + w - 23.,
                    y + 28.,
                    9.,
                    0xb3b899,
                );
            }
        }
    }
    // Ground-face insets never alter or spill above the collision surface.
    for i in 0..64 {
        let x = (i * 137 % 2400) as f32;
        let y = 1240. + (i * 37 % 84) as f32;
        polygon(
            &mut commands,
            &mut meshes,
            &mut materials,
            &[
                [x, y],
                [x + 12., y - 3.],
                [x + 23., y + 3.],
                [x + 8., y + 7.],
            ],
            0.2,
            if i % 3 == 0 { 0x656c54 } else { 0x353e34 },
            None,
        );
    }
    // Low contrast range sign beside, rather than directly behind, the spawn pilot.
    slab(&mut commands, 182., 1101., 5., 119., -1., 0x5d6d58);
    slab(&mut commands, 300., 1101., 5., 119., -1., 0x5d6d58);
    slab(&mut commands, 153., 1080., 182., 78., -0.9, 0x1f302b);
    slab(&mut commands, 158., 1085., 172., 68., -0.8, 0x465a42);
    slab(&mut commands, 168., 1095., 4., 4., -0.7, 0xb4bc8b);
    slab(&mut commands, 316., 1139., 4., 4., -0.7, 0xb4bc8b);
    label(&mut commands, "BURNHOP", 244., 1106., 19., CREAM);
    label(&mut commands, "FIELD RANGE / 01", 244., 1129., 9., 0xb4bc8b);
    slab(&mut commands, 376., 1217., 64., 3., 0.7, CYAN);
    slab(&mut commands, 894., 1217., 64., 3., 0.7, 0xdbab6a);
    label(
        &mut commands,
        "KEEP YOUR BOOTS LIGHT",
        620.,
        1274.,
        11.,
        0x8f9779,
    );
}
pub fn parallax(
    camera: Single<&Transform, (With<Camera2d>, Without<Depth>)>,
    mut layers: Query<(&Depth, &mut Transform), Without<Camera2d>>,
) {
    let offset = Vec3::new(camera.translation.x - 640., camera.translation.y + 955., 0.);
    for (depth, mut t) in &mut layers {
        t.translation = depth.home + offset * (1. - depth.factor);
    }
}
