//! Bounded continuous SAT for the original map's rectangles and convex quads.
//! The range continues to use its original three-pass AABB solver.
use crate::{Arena, Contacts, Rect, Vec2};
const EPS: f64 = 1e-8;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quad(pub [Vec2; 4]);
impl Quad {
    pub fn rect(r: Rect) -> Self {
        Self([
            Vec2 { x: r.x, y: r.y },
            Vec2 {
                x: r.x + r.width,
                y: r.y,
            },
            Vec2 {
                x: r.x + r.width,
                y: r.y + r.height,
            },
            Vec2 {
                x: r.x,
                y: r.y + r.height,
            },
        ])
    }
    fn axes(self) -> [Vec2; 6] {
        let mut axes = [Vec2 { x: 1., y: 0. }; 6];
        axes[1] = Vec2 { x: 0., y: 1. };
        for i in 0..4 {
            let a = self.0[i];
            let b = self.0[(i + 1) % 4];
            let len = (b.x - a.x).hypot(b.y - a.y);
            axes[i + 2] = Vec2 {
                x: (a.y - b.y) / len,
                y: (b.x - a.x) / len,
            };
        }
        axes
    }
    fn projection(self, axis: Vec2) -> (f64, f64) {
        self.0
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), p| {
                let d = dot(*p, axis);
                (lo.min(d), hi.max(d))
            })
    }
}
fn shapes(arena: &Arena) -> impl Iterator<Item = Quad> + '_ {
    arena
        .solids
        .iter()
        .copied()
        .map(Quad::rect)
        .chain(arena.quads.iter().copied())
}
fn dot(a: Vec2, b: Vec2) -> f64 {
    a.x * b.x + a.y * b.y
}
fn projection(body: Rect, axis: Vec2) -> (f64, f64) {
    let c = dot(crate::body_center(body), axis);
    let r = axis.x.abs() * body.width / 2. + axis.y.abs() * body.height / 2.;
    (c - r, c + r)
}
pub fn overlaps_quad(body: Rect, q: Quad) -> bool {
    q.axes().iter().all(|&a| {
        let (bl, bh) = projection(body, a);
        let (lo, hi) = q.projection(a);
        bh > lo + EPS && bl < hi - EPS
    })
}
/// Strict interior intersection; shared authored edges are legal.
pub fn quads_overlap(a: Quad, b: Quad) -> bool {
    a.axes().into_iter().chain(b.axes()).all(|axis| {
        let (al, ah) = a.projection(axis);
        let (bl, bh) = b.projection(axis);
        ah > bl + EPS && al < bh - EPS
    })
}
pub fn overlaps(body: Rect, arena: &Arena) -> bool {
    shapes(arena).any(|q| overlaps_quad(body, q))
}
#[derive(Clone, Copy, Debug)]
struct Hit {
    time: f64,
    normal: Vec2,
}
fn sweep(body: Rect, d: Vec2, q: Quad) -> Option<Hit> {
    let (mut entry, mut exit) = (f64::NEG_INFINITY, f64::INFINITY);
    let mut normal = Vec2::default();
    for a in q.axes() {
        let (bl, bh) = projection(body, a);
        let (lo, hi) = q.projection(a);
        let speed = dot(d, a);
        if speed.abs() < EPS {
            if bh <= lo + EPS || bl >= hi - EPS {
                return None;
            }
            continue;
        }
        let (u, v) = ((lo - bh) / speed, (hi - bl) / speed);
        let en = u.min(v);
        let n = if speed > 0. {
            Vec2 { x: -a.x, y: -a.y }
        } else {
            a
        };
        if en > entry + EPS || ((en - entry).abs() <= EPS && n.y < normal.y) {
            entry = en;
            normal = n;
        }
        exit = exit.min(u.max(v));
        if entry > exit + EPS {
            return None;
        }
    }
    if !(-EPS..=1.).contains(&entry) || entry >= exit - EPS || exit < 0. {
        None
    } else {
        Some(Hit {
            time: entry.max(0.),
            normal,
        })
    }
}
fn first(body: Rect, d: Vec2, arena: &Arena) -> Option<Hit> {
    let mut hit: Option<Hit> = None;
    let bounds = Rect::new(
        body.x.min(body.x + d.x),
        body.y.min(body.y + d.y),
        body.width + d.x.abs(),
        body.height + d.y.abs(),
    );
    for q in shapes(arena) {
        let (lo, hi) = q.projection(Vec2 { x: 1., y: 0. });
        if bounds.x + bounds.width < lo - EPS || bounds.x > hi + EPS {
            continue;
        }
        let (lo, hi) = q.projection(Vec2 { x: 0., y: 1. });
        if bounds.y + bounds.height < lo - EPS || bounds.y > hi + EPS {
            continue;
        }
        if let Some(h) = sweep(body, d, q)
            && hit.is_none_or(|old| {
                h.time < old.time - EPS
                    || ((h.time - old.time).abs() <= EPS && h.normal.y < old.normal.y)
            })
        {
            hit = Some(h);
        }
    }
    hit
}
pub fn supported(body: Rect, arena: &Arena) -> bool {
    first(body, Vec2 { x: 0., y: 1e-5 }, arena).is_some_and(|h| h.normal.y <= -0.55)
}
#[derive(Clone, Copy, Debug, Default)]
pub struct MixedContacts {
    pub contacts: Contacts,
    pub cap_hit: bool,
    pub iterations: u8,
}
pub fn move_body(body: &mut Rect, delta: Vec2, arena: &Arena, was_grounded: bool) -> MixedContacts {
    let mut result = MixedContacts::default();
    let mut d = delta;
    let mut touched = false;
    for i in 0..10 {
        if d.x.abs() + d.y.abs() < EPS {
            break;
        }
        result.iterations = i + 1;
        let Some(h) = first(*body, d, arena) else {
            body.x += d.x;
            body.y += d.y;
            d = Vec2::default();
            break;
        };
        body.x += d.x * h.time;
        body.y += d.y * h.time;
        d.x *= 1. - h.time;
        d.y *= 1. - h.time;
        let n = h.normal;
        if n.y <= -0.55 {
            d.y = -n.x * d.x / n.y;
            result.contacts.hit_y = true;
            touched = true;
        } else {
            let into = dot(d, n);
            d.x -= n.x * into;
            d.y -= n.y * into;
            result.contacts.hit_x |= n.x.abs() > EPS;
            result.contacts.hit_y |= n.y.abs() > EPS;
        }
    }
    result.cap_hit = result.iterations == 10 && d.x.abs() + d.y.abs() >= EPS;
    let snap = if was_grounded && delta.y >= 0. {
        delta.x.abs() * 1.52 + 0.5
    } else {
        1e-5
    };
    if (touched || (was_grounded && delta.y >= 0.))
        && let Some(h) = first(*body, Vec2 { x: 0., y: snap }, arena)
        && h.normal.y <= -0.55
    {
        body.y += snap * h.time;
        result.contacts.grounded = true;
        result.contacts.hit_y = true;
    }
    result
}
pub fn ray_quad(origin: Vec2, d: Vec2, q: Quad, range: f64) -> Option<f64> {
    let (mut near, mut far) = (0.0_f64, range);
    for i in 0..4 {
        let a = q.0[i];
        let b = q.0[(i + 1) % 4];
        let n = Vec2 {
            x: a.y - b.y,
            y: b.x - a.x,
        };
        let dist = dot(
            Vec2 {
                x: origin.x - a.x,
                y: origin.y - a.y,
            },
            n,
        );
        let speed = dot(d, n);
        if speed.abs() < EPS {
            if dist < -EPS {
                return None;
            }
        } else {
            let time = -dist / speed;
            if speed > 0. {
                near = near.max(time);
            } else {
                far = far.min(time);
            }
            if near > far {
                return None;
            }
        }
    }
    (far >= 0.).then_some(near)
}
/// Identical terrain for bullets, bot visibility and cosmetic barrel clipping.
pub fn cover(origin: Vec2, d: Vec2, range: f64, arena: &Arena) -> Option<f64> {
    arena
        .solids
        .iter()
        .filter_map(|&r| crate::ray_rect(origin, d, r, range))
        .chain(
            arena
                .quads
                .iter()
                .filter_map(|&q| ray_quad(origin, d, q, range)),
        )
        .reduce(f64::min)
}
pub fn nearest_hit(
    origin: Vec2,
    d: Vec2,
    range: f64,
    shooter: crate::ActorId,
    arena: &Arena,
    bodies: &[(crate::ActorId, Rect)],
) -> (f64, crate::Impact) {
    // Keep the original exact range path, including tie policy.
    if arena.quads.is_empty() {
        return crate::nearest_hit(origin, d, range, shooter, arena.solids, bodies);
    }
    let cover = cover(origin, d, range, arena);
    let (mut distance, mut impact) = cover.map_or((range, crate::Impact::Range), |v| {
        (v, crate::Impact::Terrain)
    });
    for &(id, body) in bodies {
        if id == shooter {
            continue;
        }
        if let Some(hit) = crate::ray_rect(origin, d, body, range)
            && (hit < distance - EPS
                || (matches!(impact,crate::Impact::Body(other) if id<other)
                    && (hit - distance).abs() <= EPS))
        {
            distance = hit;
            impact = crate::Impact::Body(id);
        }
    }
    (distance, impact)
}
