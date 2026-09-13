//! Original Ember Relay v2 data. Coordinates match the approved SVG and brief.
use crate::{Arena, Rect, Vec2, mixed::Quad};
pub const RECTS: &[Rect] = &[
    Rect::new(100.0, 1180.0, 320.0, 100.0),  // G01
    Rect::new(620.0, 1240.0, 180.0, 100.0),  // G03
    Rect::new(1100.0, 1100.0, 180.0, 120.0), // G05
    Rect::new(2020.0, 1220.0, 160.0, 100.0), // G07
    Rect::new(2400.0, 1120.0, 160.0, 120.0), // G09
    Rect::new(2900.0, 1200.0, 200.0, 120.0), // G11
    Rect::new(280.0, 1540.0, 420.0, 120.0),  // U01
    Rect::new(900.0, 1640.0, 400.0, 120.0),  // U03
    Rect::new(1580.0, 1580.0, 200.0, 56.0),  // U05
    Rect::new(1900.0, 1560.0, 300.0, 120.0), // U06
    Rect::new(2400.0, 1640.0, 480.0, 120.0), // U08
    Rect::new(3020.0, 1540.0, 100.0, 56.0),  // U10
    Rect::new(2570.0, 1390.0, 120.0, 32.0),  // U11
    Rect::new(810.0, 1440.0, 100.0, 32.0),   // U12
    Rect::new(300.0, 960.0, 160.0, 32.0),    // F01
    Rect::new(650.0, 820.0, 120.0, 32.0),    // F02
    Rect::new(1070.0, 900.0, 140.0, 32.0),   // F03
    Rect::new(1600.0, 1270.0, 120.0, 32.0),  // F04
    Rect::new(1640.0, 1010.0, 120.0, 32.0),  // F05
    Rect::new(1910.0, 760.0, 120.0, 32.0),   // F06
    Rect::new(2190.0, 930.0, 140.0, 32.0),   // F07
    Rect::new(1470.0, 650.0, 100.0, 32.0),   // F08
    Rect::new(2760.0, 880.0, 120.0, 32.0),   // F09
    Rect::new(1090.0, 1530.0, 160.0, 50.0),  // T01
    Rect::new(2460.0, 1530.0, 180.0, 50.0),  // T02
    Rect::new(360.0, 1124.0, 60.0, 56.0),    // C01
    Rect::new(-3200., -1900., 3200., 5700.), // left boundary
    Rect::new(3200., -1900., 3200., 5700.),  // right boundary
    Rect::new(-3200., -1900., 9600., 1900.), // ceiling; deliberately no floor
];
pub const QUADS: &[Quad] = &[
    Quad([
        Vec2 {
            x: 420.0,
            y: 1180.0,
        },
        Vec2 {
            x: 620.0,
            y: 1240.0,
        },
        Vec2 {
            x: 620.0,
            y: 1340.0,
        },
        Vec2 {
            x: 420.0,
            y: 1280.0,
        },
    ]), // G02
    Quad([
        Vec2 {
            x: 920.0,
            y: 1240.0,
        },
        Vec2 {
            x: 1100.0,
            y: 1100.0,
        },
        Vec2 {
            x: 1100.0,
            y: 1220.0,
        },
        Vec2 {
            x: 920.0,
            y: 1340.0,
        },
    ]), // G04
    Quad([
        Vec2 {
            x: 1280.0,
            y: 1100.0,
        },
        Vec2 {
            x: 1440.0,
            y: 1160.0,
        },
        Vec2 {
            x: 1440.0,
            y: 1280.0,
        },
        Vec2 {
            x: 1280.0,
            y: 1220.0,
        },
    ]), // G06
    Quad([
        Vec2 {
            x: 2180.0,
            y: 1220.0,
        },
        Vec2 {
            x: 2400.0,
            y: 1120.0,
        },
        Vec2 {
            x: 2400.0,
            y: 1240.0,
        },
        Vec2 {
            x: 2180.0,
            y: 1320.0,
        },
    ]), // G08
    Quad([
        Vec2 {
            x: 2700.0,
            y: 1120.0,
        },
        Vec2 {
            x: 2900.0,
            y: 1200.0,
        },
        Vec2 {
            x: 2900.0,
            y: 1320.0,
        },
        Vec2 {
            x: 2700.0,
            y: 1240.0,
        },
    ]), // G10
    Quad([
        Vec2 {
            x: 700.0,
            y: 1540.0,
        },
        Vec2 {
            x: 900.0,
            y: 1640.0,
        },
        Vec2 {
            x: 900.0,
            y: 1760.0,
        },
        Vec2 {
            x: 700.0,
            y: 1660.0,
        },
    ]), // U02
    Quad([
        Vec2 {
            x: 1300.0,
            y: 1640.0,
        },
        Vec2 {
            x: 1440.0,
            y: 1560.0,
        },
        Vec2 {
            x: 1440.0,
            y: 1680.0,
        },
        Vec2 {
            x: 1300.0,
            y: 1760.0,
        },
    ]), // U04
    Quad([
        Vec2 {
            x: 2200.0,
            y: 1560.0,
        },
        Vec2 {
            x: 2400.0,
            y: 1640.0,
        },
        Vec2 {
            x: 2400.0,
            y: 1760.0,
        },
        Vec2 {
            x: 2200.0,
            y: 1680.0,
        },
    ]), // U07
    Quad([
        Vec2 {
            x: 2880.0,
            y: 1640.0,
        },
        Vec2 {
            x: 3020.0,
            y: 1540.0,
        },
        Vec2 {
            x: 3020.0,
            y: 1660.0,
        },
        Vec2 {
            x: 2880.0,
            y: 1760.0,
        },
    ]), // U09
];
pub const SPAWNS: &[(&str, Vec2)] = &[
    (
        "S0",
        Vec2 {
            x: 200.0,
            y: 1112.0,
        },
    ),
    ("S1", Vec2 { x: 340.0, y: 892.0 }),
    (
        "S2",
        Vec2 {
            x: 1110.0,
            y: 832.0,
        },
    ),
    (
        "S3",
        Vec2 {
            x: 950.0,
            y: 1572.0,
        },
    ),
    (
        "S4",
        Vec2 {
            x: 1660.0,
            y: 1512.0,
        },
    ),
    (
        "S5",
        Vec2 {
            x: 1950.0,
            y: 692.0,
        },
    ),
    (
        "S6",
        Vec2 {
            x: 3000.0,
            y: 1132.0,
        },
    ),
    (
        "S7",
        Vec2 {
            x: 2770.0,
            y: 1572.0,
        },
    ),
    (
        "B0",
        Vec2 {
            x: 1170.0,
            y: 1032.0,
        },
    ),
];
pub const EMBER_RELAY: Arena = Arena {
    width: 3200.,
    height: 1900.,
    floor_y: 1640.,
    spawn: Vec2 { x: 200., y: 1112. },
    bot_spawn: Vec2 { x: 1170., y: 1032. },
    solids: RECTS,
    quads: QUADS,
};
/// Fail explicitly; there is no fallback spawn or geometry selection.
pub fn validate(arena: &Arena, spawns: &[(&str, Vec2)]) -> Result<(), String> {
    if ![arena.width, arena.height, arena.floor_y]
        .iter()
        .all(|v| v.is_finite() && *v > 0.)
    {
        return Err("Invalid map bounds".into());
    }
    for (i, r) in arena.solids.iter().enumerate() {
        if ![r.x, r.y, r.width, r.height].iter().all(|v| v.is_finite())
            || r.width <= 0.
            || r.height <= 0.
        {
            return Err(format!("Invalid rectangle {i}"));
        }
    }
    for (i, q) in arena.quads.iter().enumerate() {
        for j in 0..4 {
            let a = q.0[j];
            let b = q.0[(j + 1) % 4];
            let c = q.0[(j + 2) % 4];
            if ![a.x, a.y].iter().all(|v| v.is_finite())
                || a.x < 0.
                || a.y < 0.
                || a.x > arena.width
                || a.y > arena.height
                || (b.x - a.x) * (c.y - b.y) - (b.y - a.y) * (c.x - b.x) <= 1e-8
            {
                return Err(format!("Invalid convex quad {i}"));
            }
        }
    }
    let boundaries = [
        Rect::new(-arena.width, -arena.height, arena.width, arena.height * 3.),
        Rect::new(arena.width, -arena.height, arena.width, arena.height * 3.),
        Rect::new(-arena.width, -arena.height, arena.width * 3., arena.height),
    ];
    for (i, r) in arena.solids.iter().enumerate() {
        if !boundaries.contains(r)
            && (r.x < 0.
                || r.y < 0.
                || r.x + r.width > arena.width
                || r.y + r.height > arena.height)
        {
            return Err(format!("Rectangle {i} exceeds map bounds"));
        }
    }
    let shapes: Vec<Quad> = arena
        .solids
        .iter()
        .copied()
        .map(Quad::rect)
        .chain(arena.quads.iter().copied())
        .collect();
    for (i, &a) in shapes.iter().enumerate() {
        for (j, &b) in shapes.iter().enumerate().skip(i + 1) {
            if i < arena.solids.len()
                && j < arena.solids.len()
                && boundaries.contains(&arena.solids[i])
                && boundaries.contains(&arena.solids[j])
            {
                continue;
            } // Prescribed exterior boundary overlap is outside play space.
            if crate::mixed::quads_overlap(a, b) {
                return Err(format!("Solid interiors intersect: {i} and {j}"));
            }
        }
    }
    let roles = [
        ("player/recovery", arena.spawn),
        ("fixed bot", arena.bot_spawn),
    ];
    for &(id, p) in spawns.iter().chain(roles.iter()) {
        let b = Rect::new(p.x, p.y, 36., 68.);
        if !p.x.is_finite()
            || !p.y.is_finite()
            || p.x < 0.
            || p.y < 0.
            || p.x + 36. > arena.width
            || p.y + 68. > arena.height
            || crate::mixed::overlaps(b, arena)
            || !crate::mixed::supported(b, arena)
        {
            return Err(format!("Invalid standing spawn {id}"));
        }
    }
    Ok(())
}
