//! The BaryonOS mark, drawn per pixel from the fixed-point tables that
//! `ui/design/brand/generate_mark.py` emits into `mark_geometry.rs`.
//! Integer only; every pixel depends on its own coordinates, so tiles match.
use crate::{blend, Canvas, Rect};

pub(crate) struct Geometry {
    pub offset: (i32, i32),
    pub nodes: [(i32, i32); 3],
    pub node: i32,
    pub clear: i32,
    pub outer: [(i32, i32); 3],
    pub inner: [(i32, i32); 3],
    pub blades: &'static [[(i32, i32); 3]],
}

/// Full mark with blades (40 px and larger) or the compact reduced-size form.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Variant {
    Full,
    Compact,
}

/// Node colours in red, green, blue order; the bond shades from `bond_light`
/// at the upper left to `bond_shade` at the lower right.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Colors {
    pub nodes: [u32; 3],
    pub bond_light: u32,
    pub bond_shade: u32,
}

/// The canonical dark-surface colours from docs/design/BRAND.md.
pub const ON_DARK: Colors = Colors {
    nodes: [0xff4a3d, 0x33c46b, 0x2585ff],
    bond_light: 0xf4f6f9,
    bond_shade: 0xa7b0bc,
};

const SUB: i64 = 4; // 4x4 samples per pixel
const ONE: i64 = 65536;

/// Draw the mark centred in `box_` with node circumradius `radius` pixels.
pub fn draw(c: &mut Canvas<'_>, box_: Rect, radius: u32, variant: Variant, colors: Colors) {
    if radius == 0 || radius > 4096 || box_.width > 1_048_576 || box_.height > 1_048_576 {
        return;
    }
    let g = match variant {
        Variant::Full => &crate::mark_geometry::FULL,
        Variant::Compact => &crate::mark_geometry::COMPACT,
    };
    let r = i64::from(radius);
    // Sample space: pixels * 2*SUB, so sample centres sit on odd integers.
    let k = 2 * SUB;
    let scale = |v: i32| i64::from(v) * r * k / ONE;
    let cx = (i64::from(box_.x) * 2 + i64::from(box_.width)) * SUB + scale(g.offset.0);
    let cy = (i64::from(box_.y) * 2 + i64::from(box_.height)) * SUB + scale(g.offset.1);
    let at = |p: (i32, i32)| (cx + scale(p.0), cy + scale(p.1));
    let nodes = g.nodes.map(at);
    let outer = g.outer.map(at);
    let inner = g.inner.map(at);
    let node2 = scale(g.node).pow(2);
    let clear2 = scale(g.clear).pow(2);
    let extent = (r * 3 / 2 + 2) * k;
    let bounds = Rect::new(
        clamp_i32((cx - extent) / k),
        clamp_i32((cy - extent) / k),
        (2 * extent / k) as u32,
        (2 * extent / k) as u32,
    );
    let Some((left, top, right, bottom)) = c.clipped(bounds) else {
        return;
    };
    let span = 4 * extent.max(1);
    for y in top..bottom {
        for x in left..right {
            let mut count = [0u32; 4]; // red, green, blue, bond
            for sy in 0..SUB {
                for sx in 0..SUB {
                    let px = i64::from(x) * k + 2 * sx + 1;
                    let py = i64::from(y) * k + 2 * sy + 1;
                    let d2 = |n: (i64, i64)| (px - n.0).pow(2) + (py - n.1).pow(2);
                    if let Some(i) = (0..3).find(|&i| d2(nodes[i]) <= node2) {
                        count[i] += 1;
                    } else if nodes.iter().all(|&n| d2(n) > clear2)
                        && ((inside(outer, px, py) && !inside(inner, px, py))
                            || g.blades.iter().any(|b| inside(b.map(at), px, py)))
                    {
                        count[3] += 1;
                    }
                }
            }
            let samples = (SUB * SUB) as u32;
            for (i, &n) in count[..3].iter().enumerate() {
                c.blend_pixel(x, y, colors.nodes[i], (n * 255 / samples) as u8);
            }
            if count[3] > 0 {
                let along = (i64::from(x) * k - cx) + (i64::from(y) * k - cy) + span / 2;
                let t = (along.clamp(0, span) * 255 / span) as u8;
                let shade = blend(colors.bond_shade, colors.bond_light, t);
                c.blend_pixel(x, y, shade, (count[3] * 255 / samples) as u8);
            }
        }
    }
}

fn inside(t: [(i64, i64); 3], x: i64, y: i64) -> bool {
    let edge = |a: (i64, i64), b: (i64, i64)| (b.0 - a.0) * (y - a.1) - (b.1 - a.1) * (x - a.0);
    let (e0, e1, e2) = (edge(t[0], t[1]), edge(t[1], t[2]), edge(t[2], t[0]));
    (e0 >= 0 && e1 >= 0 && e2 >= 0) || (e0 <= 0 && e1 <= 0 && e2 <= 0)
}

fn clamp_i32(v: i64) -> i32 {
    v.clamp(i64::from(i32::MIN / 2), i64::from(i32::MAX / 2)) as i32
}
