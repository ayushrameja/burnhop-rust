//! Swept AABB collision: earliest time of impact followed by tangential sliding.
//! Static rectangles and a non-overlapping starting body are required.
use crate::{Rect, Vec2};

const EPS: f64 = 1e-8;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Contacts {
    pub hit_x: bool,
    pub hit_y: bool,
    pub grounded: bool,
}

pub fn supported(body: &Rect, solids: &[Rect]) -> bool {
    solids.iter().any(|s| {
        (body.y + body.height - s.y).abs() <= EPS
            && body.x + body.width > s.x + EPS
            && body.x < s.x + s.width - EPS
    })
}

fn axis_times(
    low: f64,
    high: f64,
    solid_low: f64,
    solid_high: f64,
    delta: f64,
) -> Option<(f64, f64)> {
    if delta.abs() < EPS {
        // Tangential touching is legal, including walking across a platform top.
        if high <= solid_low + EPS || low >= solid_high - EPS {
            None
        } else {
            Some((f64::NEG_INFINITY, f64::INFINITY))
        }
    } else {
        let a = (solid_low - high) / delta;
        let b = (solid_high - low) / delta;
        Some((a.min(b), a.max(b)))
    }
}

fn sweep(body: &Rect, delta: Vec2, solid: &Rect) -> Option<(f64, bool, bool)> {
    let (enter_x, exit_x) = axis_times(
        body.x,
        body.x + body.width,
        solid.x,
        solid.x + solid.width,
        delta.x,
    )?;
    let (enter_y, exit_y) = axis_times(
        body.y,
        body.y + body.height,
        solid.y,
        solid.y + solid.height,
        delta.y,
    )?;
    let enter = enter_x.max(enter_y);
    let exit = exit_x.min(exit_y);
    // Ignore initial separation, movement away from contact, and a corner grazed
    // for only an instant. Equal entry times into both axes block both normals.
    if !(-EPS..=1.0).contains(&enter) || enter >= exit - EPS {
        return None;
    }
    Some((
        enter.max(0.0),
        enter_x >= enter_y - EPS,
        enter_y >= enter_x - EPS,
    ))
}

/// Sweeps the entire displacement, including velocities far above gameplay caps.
/// Each impact removes at least one moving axis, so three passes suffice for
/// two axes plus the remaining free slide. Equal-time contacts are combined,
/// independent of rectangle ordering. Support is checked at the final position.
pub fn move_body(body: &mut Rect, mut delta: Vec2, solids: &[Rect]) -> Contacts {
    let mut contacts = Contacts::default();
    for _ in 0..3 {
        if delta.x.abs() < EPS && delta.y.abs() < EPS {
            break;
        }
        let mut earliest = 1.0;
        let (mut block_x, mut block_y) = (false, false);
        for solid in solids {
            if let Some((time, x, y)) = sweep(body, delta, solid) {
                if time < earliest - EPS {
                    earliest = time;
                    block_x = x;
                    block_y = y;
                } else if (time - earliest).abs() <= EPS {
                    block_x |= x;
                    block_y |= y;
                }
            }
        }
        body.x += delta.x * earliest;
        body.y += delta.y * earliest;
        contacts.hit_x |= block_x;
        contacts.hit_y |= block_y;
        delta.x = if block_x {
            0.0
        } else {
            delta.x * (1.0 - earliest)
        };
        delta.y = if block_y {
            0.0
        } else {
            delta.y * (1.0 - earliest)
        };
        if !block_x && !block_y {
            break;
        }
    }
    contacts.grounded = supported(body, solids);
    contacts
}
