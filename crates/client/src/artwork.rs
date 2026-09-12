//! Original polygon artwork, rasterized once into one shared 4x atlas.
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

pub const INK: u32 = 0x202b29;
pub const SAGE: u32 = 0x8d9b70;
pub const CREAM: u32 = 0xede4c9;
pub const CYAN: u32 = 0x75d4d0;
pub const OCHRE: u32 = 0xdbab6a;
pub fn color(hex: u32) -> Color {
    Color::srgb_u8((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}
#[derive(Clone, Copy)]
#[repr(usize)]
pub enum Tile {
    Head,
    Torso,
    Pack,
    Limb,
    Boot,
    Hand,
    Pistol,
    Rifle,
    Magazine,
    Exhaust,
    Badge,
}
#[derive(Resource)]
pub struct Artwork {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
}
impl Artwork {
    pub fn sprite(&self, tile: Tile) -> Sprite {
        Sprite {
            image: self.image.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: self.layout.clone(),
                index: tile as usize,
            }),
            custom_size: Some(Vec2::splat(64.)),
            ..default()
        }
    }
}
const SCALE: usize = 4;
const CELL: usize = 64 * SCALE;
const SIDE: usize = CELL * 4;
struct Canvas {
    bytes: Vec<u8>,
    tile: usize,
}
impl Canvas {
    fn poly(&mut self, points: &[(f32, f32)], fill: u32) {
        let scaled: Vec<_> = points
            .iter()
            .map(|&(x, y)| {
                if (Tile::Pistol as usize..=Tile::Magazine as usize).contains(&self.tile) {
                    (x * 68. / 85.94, y * 68. / 85.94)
                } else {
                    (x, y)
                }
            })
            .collect();
        let points = &scaled;
        let min_x = points.iter().map(|p| p.0).fold(f32::INFINITY, f32::min);
        let max_x = points.iter().map(|p| p.0).fold(f32::NEG_INFINITY, f32::max);
        let min_y = points.iter().map(|p| p.1).fold(f32::INFINITY, f32::min);
        let max_y = points.iter().map(|p| p.1).fold(f32::NEG_INFINITY, f32::max);
        let range = |a: f32, b: f32| {
            (((a + 32.) * SCALE as f32).floor().max(0.) as usize)
                ..(((b + 32.) * SCALE as f32).ceil().min(CELL as f32) as usize)
        };
        for y in range(min_y, max_y) {
            for x in range(min_x, max_x) {
                let px = (x as f32 + 0.5) / SCALE as f32 - 32.;
                let py = (y as f32 + 0.5) / SCALE as f32 - 32.;
                let mut inside = false;
                for (a, b) in points
                    .iter()
                    .zip(points.iter().cycle().skip(1))
                    .take(points.len())
                {
                    if (a.1 > py) != (b.1 > py) && px < (b.0 - a.0) * (py - a.1) / (b.1 - a.1) + a.0
                    {
                        inside = !inside;
                    }
                }
                if inside {
                    let at = (((self.tile / 4) * CELL + y) * SIDE + (self.tile % 4) * CELL + x) * 4;
                    self.bytes[at..at + 4].copy_from_slice(&[
                        (fill >> 16) as u8,
                        (fill >> 8) as u8,
                        fill as u8,
                        255,
                    ]);
                }
            }
        }
    }
    fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, c: u32) {
        self.poly(&[(x, y), (x + w, y), (x + w, y + h), (x, y + h)], c);
    }
}
pub fn create(images: &mut Assets<Image>, layouts: &mut Assets<TextureAtlasLayout>) -> Artwork {
    let mut c = Canvas {
        bytes: vec![0; SIDE * SIDE * 4],
        tile: 0,
    };
    // Head faces right. Cap, ear, cheek, brow, white eye, pupil, nose and beard.
    c.poly(
        &[
            (-11., -7.),
            (-8., -12.),
            (7., -12.),
            (12., -7.),
            (12., 5.),
            (7., 11.),
            (-4., 12.),
            (-10., 7.),
            (-12., 0.),
        ],
        INK,
    );
    c.poly(
        &[
            (-8., -7.),
            (7., -9.),
            (10., -6.),
            (10., 5.),
            (6., 9.),
            (-3., 9.),
            (-8., 5.),
        ],
        0xddb47d,
    );
    c.poly(
        &[
            (-8., -5.),
            (-4., -5.),
            (-3., 7.),
            (2., 10.),
            (-5., 9.),
            (-9., 4.),
        ],
        0xa87751,
    );
    c.rect(-11., -1., 4., 6., 0xddb47d);
    c.rect(-9., 0., 1., 3., 0x936747);
    c.poly(
        &[
            (-11., -7.),
            (-10., -13.),
            (-4., -16.),
            (5., -15.),
            (11., -12.),
            (14., -7.),
        ],
        INK,
    );
    c.poly(
        &[
            (-9., -9.),
            (-8., -12.),
            (-3., -14.),
            (5., -13.),
            (10., -10.),
            (11., -8.),
        ],
        SAGE,
    );
    c.poly(
        &[(-12., -8.), (8., -9.), (15., -6.), (14., -4.), (-12., -5.)],
        INK,
    );
    c.poly(
        &[(-10., -7.), (8., -7.), (13., -5.5), (-10., -5.5)],
        0xb4bc8b,
    );
    c.rect(-6., -12., 3., 2., CREAM);
    c.poly(
        &[
            (0., -3.),
            (5., -4.),
            (9., -2.),
            (8., 1.),
            (2., 1.),
            (0., -1.),
        ],
        INK,
    );
    c.poly(
        &[(1., -2.), (5., -3.), (8., -1.5), (7., 0.), (2., 0.)],
        CREAM,
    );
    c.rect(5., -2.8, 1.8, 3., INK);
    c.rect(6., -2.5, 0.6, 1., CREAM);
    c.poly(
        &[(8., 0.), (11., 2.), (13., 3.), (12., 4.), (8., 3.)],
        0xa87751,
    );
    c.poly(
        &[
            (-6., 4.),
            (-2., 6.),
            (4., 5.),
            (9., 6.),
            (7., 10.),
            (0., 11.),
            (-5., 8.),
        ],
        0x624d38,
    );
    c.rect(0., 6.5, 6., 1., CREAM);
    c.tile = Tile::Torso as usize;
    c.poly(
        &[
            (-10., -11.),
            (-4., -14.),
            (5., -13.),
            (10., -9.),
            (10., 9.),
            (7., 12.),
            (-9., 12.),
            (-11., 6.),
        ],
        INK,
    );
    c.poly(
        &[
            (-8., -9.),
            (-3., -11.),
            (5., -10.),
            (8., -6.),
            (7., 7.),
            (-8., 7.),
        ],
        SAGE,
    );
    c.poly(
        &[
            (-7., -10.),
            (-2., -4.),
            (3., -10.),
            (6., -8.),
            (2., -2.),
            (-5., -3.),
        ],
        0xb4bc8b,
    );
    c.rect(-9., -3., 17., 3., INK);
    c.rect(-7., -2., 12., 1., 0x566454);
    c.rect(-7., 2., 7., 4., 0x687657);
    c.rect(2., 1., 5., 5., 0x687657);
    c.rect(-10., 8., 19., 4., 0x34433b);
    c.rect(-1., 8., 4., 3., OCHRE);
    c.rect(0., 9., 2., 1., INK);
    c.tile = Tile::Pack as usize;
    c.poly(
        &[
            (-5., -11.),
            (3., -12.),
            (6., -7.),
            (6., 10.),
            (-5., 11.),
            (-7., 6.),
            (-7., -6.),
        ],
        INK,
    );
    c.rect(-5., -8., 8., 15., 0x56685c);
    c.rect(-4., -5., 5., 7., 0x728373);
    c.rect(-5., 7., 7., 2., 0xb4bc8b);
    c.tile = Tile::Limb as usize;
    c.poly(
        &[
            (-3., -8.),
            (2., -8.),
            (4., -5.),
            (4., 5.),
            (2., 8.),
            (-3., 8.),
            (-4., 5.),
            (-4., -5.),
        ],
        INK,
    );
    c.poly(
        &[
            (-2., -6.),
            (1., -6.),
            (2., -3.),
            (2., 5.),
            (0., 6.),
            (-2., 5.),
        ],
        0x567664,
    );
    c.rect(-2., -5., 1.2, 8., 0x86a18a);
    c.tile = Tile::Boot as usize;
    c.poly(
        &[
            (-6., -4.),
            (2., -4.),
            (4., -1.),
            (8., 0.),
            (9., 4.),
            (-7., 4.),
            (-7., -1.),
        ],
        INK,
    );
    c.poly(
        &[
            (-5., -3.),
            (1., -3.),
            (2., 0.),
            (6., 1.),
            (6., 2.),
            (-5., 2.),
        ],
        0x526961,
    );
    c.rect(-5., -2., 5., 1., 0xb1baa3);
    c.rect(-5., 3., 12., 1., 0xb1baa3);
    c.rect(-3., 4., 5., 1., INK);
    c.tile = Tile::Hand as usize;
    c.poly(
        &[
            (-4., -3.),
            (2., -4.),
            (4., -1.),
            (3., 3.),
            (-2., 4.),
            (-4., 1.),
        ],
        INK,
    );
    c.poly(
        &[(-2., -2.), (1., -2.), (2., 0.), (1., 2.), (-2., 1.)],
        0xddb47d,
    );
    // Weapon coordinates are scaled by the same 68/85.94 used by approved barrels.
    c.tile = Tile::Pistol as usize;
    c.poly(
        &[
            (-3., -4.),
            (14., -4.),
            (17., -2.),
            (17., 2.),
            (8., 2.),
            (5., 12.),
            (-1., 11.),
            (1., 3.),
            (-3., 2.),
        ],
        INK,
    );
    c.rect(-1., -3., 15., 3., 0x819286);
    c.rect(0., -4., 3., 1., CREAM);
    c.rect(12., -5., 2., 2., INK);
    c.poly(&[(1., 4.), (5., 4.), (3., 9.), (0., 9.)], 0x687657);
    c.rect(9., -2., 4., 1., CREAM);
    c.tile = Tile::Rifle as usize;
    c.poly(
        &[
            (-18., -5.),
            (-8., -5.),
            (-7., -2.),
            (-2., -2.),
            (-2., -5.),
            (26., -5.),
            (28., -2.),
            (34., -2.),
            (34., 2.),
            (15., 3.),
            (12., 6.),
            (7., 6.),
            (6., 13.),
            (1., 11.),
            (2., 4.),
            (-8., 4.),
            (-16., 8.),
            (-18., 6.),
        ],
        INK,
    );
    c.poly(
        &[
            (-16., -3.),
            (-10., -3.),
            (-9., 0.),
            (-4., 0.),
            (-4., 2.),
            (-10., 2.),
            (-15., 5.),
            (-16., 5.),
        ],
        0x68786d,
    );
    c.rect(-1., -3., 15., 5., 0x71857a);
    c.rect(15., -3., 11., 5., 0xa89f7d);
    c.rect(0., -5., 25., 1.5, 0xb6bca5);
    c.rect(4., -8., 7., 3., INK);
    c.rect(5., -7., 4., 1., 0x8d9b70);
    c.rect(29., -5., 2., 5., INK);
    c.rect(17., -1., 1., 2., INK);
    c.rect(20., -1., 1., 2., INK);
    c.rect(23., -1., 1., 2., INK);
    c.rect(8., -1.5, 4., 1.5, CREAM);
    c.tile = Tile::Magazine as usize;
    c.poly(&[(-3., -5.), (3., -5.), (3., 4.), (1., 6.), (-4., 4.)], INK);
    c.rect(-1.5, -3., 3., 6., 0x64776b);
    c.rect(-1., -2., 1., 4., 0xb4bc8b);
    c.tile = Tile::Exhaust as usize;
    c.poly(
        &[(-3., -8.), (3., -8.), (2., 0.), (0., 12.), (-2., 0.)],
        0x458d92,
    );
    c.poly(
        &[(-2., -8.), (2., -8.), (1., 0.), (0., 7.), (-1., 0.)],
        CYAN,
    );
    c.poly(&[(-1., -8.), (1., -8.), (0., 0.)], CREAM);
    c.tile = Tile::Badge as usize;
    c.poly(
        &[(-4., -4.), (4., -4.), (4., 1.), (0., 5.), (-4., 1.)],
        0xffffff,
    );
    let image = images.add(Image::new(
        Extent3d {
            width: SIDE as u32,
            height: SIDE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        c.bytes,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    ));
    let layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(CELL as u32),
        4,
        4,
        None,
        None,
    ));
    Artwork { image, layout }
}
