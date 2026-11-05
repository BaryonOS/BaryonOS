//! Host renderer comparison against the retained full composite reference.
//! No QEMU, syscall, scanout, boot or product-latency measurement.
use nieuw_os_graphics::Rect;
use nieuw_os_window_session::{
    scene::{Damage, Scene, MAX_PIXELS},
    shell::Page,
};
use std::{hint::black_box, time::Instant};

fn raster(scene: &Scene, backing: &[u32], fast: bool) -> u128 {
    let start = Instant::now();
    for y in (0..scene.height).step_by(8) {
        for x in (0..scene.width).step_by(32) {
            let r = Rect::new(
                x as i32,
                y as i32,
                32.min(scene.width - x),
                8.min(scene.height - y),
            );
            let mut pixels = [0; 256];
            if fast {
                scene.render(r, &mut pixels, backing).unwrap();
            } else {
                scene.render_reference(r, &mut pixels, backing).unwrap();
            }
            black_box(&pixels);
        }
    }
    start.elapsed().as_micros()
}
fn panel_raster(scene: &Scene, backing: &[u32]) -> (u128, usize) {
    let mut damage = Damage::new(scene.width, scene.height).unwrap();
    damage.mark(scene.shell.panel(scene.width, scene.height, scene.scale));
    let mut tiles = 0;
    let start = Instant::now();
    while let Some(rect) = damage.next_tile() {
        let mut pixels = [0; 256];
        scene.render(rect, &mut pixels, backing).unwrap();
        black_box(&pixels);
        tiles += 1;
    }
    (start.elapsed().as_micros(), tiles)
}
fn main() {
    let backing: Vec<_> = (0..MAX_PIXELS).map(|i| (i as u32) & 0x00ff_ffff).collect();
    for (width, height, scale) in [(1280, 720, 1), (1920, 1080, 2)] {
        let mut scene = Scene::new(width, height, scale, [1, 2]).unwrap();
        scene.shell.set_clock(Some(1_793_649_600));
        for page in [Page::Lock, Page::Search] {
            scene.shell.open(page);
            // Warm both real paths before recording alternating samples.
            black_box(raster(&scene, &backing, false));
            black_box(raster(&scene, &backing, true));
            for sample in 0..3 {
                let (reference_us, covered_us) = if sample % 2 == 0 {
                    (
                        raster(&scene, &backing, false),
                        raster(&scene, &backing, true),
                    )
                } else {
                    let covered = raster(&scene, &backing, true);
                    (raster(&scene, &backing, false), covered)
                };
                let (panel_us, panel_tiles) = panel_raster(&scene, &backing);
                println!("{{\"scope\":\"host-render-only\",\"width\":{width},\"height\":{height},\"scale\":{scale},\"page\":\"{page:?}\",\"sample\":{sample},\"reference_us\":{reference_us},\"covered_us\":{covered_us},\"panel_us\":{panel_us},\"panel_tiles\":{panel_tiles}}}");
            }
        }
    }
}
