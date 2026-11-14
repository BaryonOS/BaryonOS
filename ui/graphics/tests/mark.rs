use nieuw_os_graphics::mark::{self, Variant, ON_DARK};
use nieuw_os_graphics::{Canvas, Rect};

const SIZE: i32 = 96;
const BOX: Rect = Rect::new(0, 0, 96, 96);

fn render(variant: Variant, tile: Rect) -> [u32; 32 * 8] {
    let mut px = [0; 32 * 8];
    let mut c = Canvas::new(tile, &mut px).unwrap();
    mark::draw(&mut c, BOX, 36, variant, ON_DARK);
    px
}

fn full(variant: Variant) -> [u32; 96 * 96] {
    let mut px = [0; 96 * 96];
    mark::draw(
        &mut Canvas::new(BOX, &mut px).unwrap(),
        BOX,
        36,
        variant,
        ON_DARK,
    );
    px
}

fn at(px: &[u32], x: i32, y: i32) -> u32 {
    px[(y * SIZE + x) as usize]
}

#[test]
fn tiles_match_one_full_render() {
    for variant in [Variant::Full, Variant::Compact] {
        let whole = full(variant);
        for ty in (0..SIZE).step_by(8) {
            for tx in (0..SIZE).step_by(32) {
                let tile = render(variant, Rect::new(tx, ty, 32, 8));
                for row in 0..8 {
                    let start = ((ty + row) * SIZE + tx) as usize;
                    assert_eq!(&whole[start..start + 32], &tile[row as usize * 32..][..32]);
                }
            }
        }
    }
}

#[test]
fn nodes_bond_clearance_and_blades_land_where_the_brand_geometry_puts_them() {
    let f = full(Variant::Full);
    assert_eq!(at(&f, 41, 19), ON_DARK.nodes[0], "red node");
    assert_eq!(at(&f, 17, 76), ON_DARK.nodes[1], "green node");
    assert_eq!(at(&f, 78, 69), ON_DARK.nodes[2], "blue node");
    assert_eq!(at(&f, 45, 55), 0, "open centre of the bond triangle");
    assert_eq!(at(&f, 42, 28), 0, "clearance ring carves the bond corner");
    let bond = at(&f, 55, 47);
    assert!(
        bond != 0 && (bond >> 16) >= 0xa7 && (bond >> 16) <= 0xf4,
        "silver bond: {bond:06x}"
    );
    assert_ne!(at(&f, 53, 32), 0, "blade outside the bond");
    let c = full(Variant::Compact);
    assert_eq!(at(&c, 53, 32), 0, "compact mark has no blades");
    assert_eq!(at(&c, 41, 19), ON_DARK.nodes[0]);
    assert!(f.iter().chain(c.iter()).all(|p| p >> 24 == 0));
}

#[test]
fn degenerate_inputs_draw_nothing() {
    let mut px = [0x123456; 32 * 8];
    let mut c = Canvas::new(Rect::new(0, 0, 32, 8), &mut px).unwrap();
    mark::draw(&mut c, BOX, 0, Variant::Full, ON_DARK);
    mark::draw(
        &mut c,
        Rect::new(i32::MIN, i32::MIN, u32::MAX, u32::MAX),
        36,
        Variant::Full,
        ON_DARK,
    );
    mark::draw(
        &mut c,
        Rect::new(i32::MAX - 4, i32::MAX - 4, 8, 8),
        4096,
        Variant::Full,
        ON_DARK,
    );
    assert!(px.iter().all(|&p| p == 0x123456));
}
