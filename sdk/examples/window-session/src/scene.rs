use nieuw_os_graphics::{theme, Canvas, Font, Icon, Rect, Size, Weight};
use nieuw_os_surface_protocol::{Geometry, Rectangle};

pub const MAX_PIXELS: usize = 1_400_000;
pub const WINDOWS: usize = 2;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Invalid;
#[derive(Clone, Copy, Debug)]
pub struct Window {
    pub body: Rect,
    pub geometry: Geometry,
    pub offset: usize,
}
pub struct Scene {
    pub width: u32,
    pub height: u32,
    pub scale: u32,
    pub windows: [Window; WINDOWS],
    pub focused: usize,
    pub pointer: (u32, u32),
    drag: Option<(usize, i32, i32)>,
    pub moves: u64,
    pub switches: u64,
}
impl Scene {
    pub fn new(width: u32, height: u32, scale: u32, epochs: [u64; 2]) -> Result<Self, Invalid> {
        if !matches!(scale, 1 | 2)
            || width > 1920
            || height > 1080
            || width / scale < 800
            || height / scale < 500
            || epochs.contains(&0)
        {
            return Err(Invalid);
        }
        let (w, h) = (width / scale, height / scale);
        let mw = (w - 400).clamp(480, 960);
        let bh = (h - 210).clamp(260, 620);
        let definitions = [(28, 114, mw, bh), (w - 348, 140, 320, bh - 26)];
        let mut offset = 0;
        let windows = core::array::from_fn(|i| {
            let (x, y, ww, hh) = definitions[i];
            let window = Window {
                body: Rect::new(
                    (x * scale) as i32,
                    (y * scale) as i32,
                    ww * scale,
                    hh * scale,
                ),
                geometry: Geometry {
                    width,
                    height,
                    epoch: epochs[i],
                    assigned: Rectangle {
                        x: 0,
                        y: 0,
                        width: (ww * scale) as u16,
                        height: (hh * scale) as u16,
                    },
                },
                offset,
            };
            offset += (ww * hh * scale * scale) as usize;
            window
        });
        if offset > MAX_PIXELS {
            return Err(Invalid);
        }
        for window in windows {
            window.geometry.validate().map_err(|_| Invalid)?;
        }
        Ok(Self {
            width,
            height,
            scale,
            windows,
            focused: 0,
            pointer: (width / 2, height / 2),
            drag: None,
            moves: 0,
            switches: 0,
        })
    }
    pub fn frame(&self, index: usize) -> Rect {
        let b = self.windows[index].body;
        Rect::new(
            b.x,
            b.y - 42 * self.scale as i32,
            b.width,
            b.height + 42 * self.scale,
        )
    }
    pub fn paint_bounds(&self, index: usize) -> Rect {
        let f = self.frame(index);
        Rect::new(
            f.x,
            f.y,
            f.width + 4 * self.scale,
            f.height + 8 * self.scale,
        )
    }
    pub fn select(&mut self, index: usize) -> Result<(), Invalid> {
        if index >= WINDOWS {
            return Err(Invalid);
        }
        if index != self.focused {
            self.drag = None;
            self.focused = index;
            self.switches = self.switches.checked_add(1).ok_or(Invalid)?;
        }
        Ok(())
    }
    pub fn cycle(&mut self) -> Result<(), Invalid> {
        self.select(1 - self.focused)
    }
    pub fn cancel_pointer(&mut self) {
        self.drag = None;
    }
    /// Mouse policy receives the already decoded global position. Only a
    /// trusted title bar starts movement; application pixels cannot start it.
    /// Returns a body recipient, never a caller-chosen process identifier.
    pub fn mouse(
        &mut self,
        x: u32,
        y: u32,
        down: bool,
        held: bool,
    ) -> Result<Option<usize>, Invalid> {
        if x >= self.width || y >= self.height {
            return Err(Invalid);
        }
        self.pointer = (x, y);
        if let Some((index, dx, dy)) = self.drag {
            if !held {
                self.drag = None;
                return Ok(None);
            }
            let window = &mut self.windows[index];
            let maxx = self.width - window.body.width;
            let maxy = self.height - window.body.height - 72 * self.scale;
            let nx = (x as i32 - dx).clamp(0, maxx as i32);
            let ny = (y as i32 - dy).clamp(70 * self.scale as i32, maxy as i32);
            if window.body.x != nx || window.body.y != ny {
                window.body.x = nx;
                window.body.y = ny;
                self.moves = self.moves.checked_add(1).ok_or(Invalid)?;
            }
            return Ok(None);
        }
        if down {
            let s = self.scale;
            let dock = self.dock();
            for index in 0..WINDOWS {
                let button = Rect::new(
                    dock.x + (12 + index as i32 * 56) * s as i32,
                    dock.y + 8 * s as i32,
                    48 * s,
                    48 * s,
                );
                if button.contains(x as i32, y as i32) {
                    self.select(index)?;
                    return Ok(None);
                }
            }
            for index in [self.focused, 1 - self.focused] {
                let frame = self.frame(index);
                if frame.contains(x as i32, y as i32) {
                    self.select(index)?;
                    if y < self.windows[index].body.y as u32 {
                        let b = self.windows[index].body;
                        self.drag = Some((index, x as i32 - b.x, y as i32 - b.y));
                        return Ok(None);
                    }
                    return Ok(Some(index));
                }
            }
            return Ok(None);
        }
        let b = self.windows[self.focused].body;
        Ok(b.contains(x as i32, y as i32).then_some(self.focused))
    }
    pub fn dock(&self) -> Rect {
        Rect::new(
            self.width as i32 / 2 - 68 * self.scale as i32,
            self.height as i32 - 76 * self.scale as i32,
            136 * self.scale,
            64 * self.scale,
        )
    }
    pub fn copy_tile(
        &self,
        index: usize,
        rect: Rectangle,
        pixels: &[u8],
        backing: &mut [u32],
    ) -> Result<Rect, Invalid> {
        let window = self.windows.get(index).ok_or(Invalid)?;
        let (w, h) = (window.body.width, window.body.height);
        if rect.width == 0
            || rect.height == 0
            || rect
                .x
                .checked_add(u32::from(rect.width))
                .is_none_or(|v| v > w)
            || rect
                .y
                .checked_add(u32::from(rect.height))
                .is_none_or(|v| v > h)
            || pixels.len() != usize::from(rect.width) * usize::from(rect.height) * 4
            || window.offset + w as usize * h as usize > backing.len()
        {
            return Err(Invalid);
        }
        for row in 0..usize::from(rect.height) {
            for col in 0..usize::from(rect.width) {
                let at = (row * usize::from(rect.width) + col) * 4;
                let color = u32::from_le_bytes(pixels[at..at + 4].try_into().unwrap());
                if color >> 24 != 0 {
                    return Err(Invalid);
                }
            }
        }
        for row in 0..usize::from(rect.height) {
            for col in 0..usize::from(rect.width) {
                let at = (row * usize::from(rect.width) + col) * 4;
                backing[window.offset
                    + (rect.y as usize + row) * w as usize
                    + rect.x as usize
                    + col] = u32::from_le_bytes(pixels[at..at + 4].try_into().unwrap());
            }
        }
        Ok(Rect::new(
            window.body.x + rect.x as i32,
            window.body.y + rect.y as i32,
            u32::from(rect.width),
            u32::from(rect.height),
        ))
    }
    pub fn render(&self, rect: Rect, pixels: &mut [u32], backing: &[u32]) -> Result<(), Invalid> {
        let mut c = Canvas::new(rect, pixels).map_err(|_| Invalid)?;
        let palette = theme::MATERIAAL;
        theme::wallpaper(&mut c, Rect::new(0, 0, self.width, self.height), palette);
        let s = self.scale;
        let font = Font::new(Size::Body, Weight::Semibold, s == 2);
        c.fill_rect(Rect::new(0, 0, self.width, 40 * s), 0x122437);
        c.icon(
            Rect::new(22 * s as i32, 10 * s as i32, 20 * s, 20 * s),
            Icon::Material,
            0x8ed3c7,
            s,
        );
        c.text(54 * s as i32, 9 * s as i32, "Nieuw OS", font, 0xf3f8fa);
        c.text(
            172 * s as i32,
            10 * s as i32,
            "Werkruimte",
            Font::new(Size::Small, Weight::Regular, s == 2),
            0xb5cbd2,
        );
        for index in [1 - self.focused, self.focused] {
            let body = self.windows[index].body;
            let frame = self.frame(index);
            if !self.paint_bounds(index).intersects(rect) {
                continue;
            }
            c.rounded_rect(
                Rect::new(
                    frame.x + 4 * s as i32,
                    frame.y + 8 * s as i32,
                    frame.width,
                    frame.height,
                ),
                12 * s,
                0x132a38,
            );
            c.rounded_rect(
                frame,
                10 * s,
                if index == self.focused {
                    0xffffff
                } else {
                    0xe9eff3
                },
            );
            let title = if index == 0 {
                "Materiaal"
            } else {
                "Rekenmachine"
            };
            c.icon(
                Rect::new(
                    frame.x + 15 * s as i32,
                    frame.y + 12 * s as i32,
                    18 * s,
                    18 * s,
                ),
                if index == 0 {
                    Icon::Material
                } else {
                    Icon::Apps
                },
                0x167a70,
                s,
            );
            c.text(
                frame.x + 44 * s as i32,
                frame.y + 10 * s as i32,
                title,
                font,
                0x172a3a,
            );
            // Trusted app identity and focus marker live outside app bounds.
            if index == self.focused {
                c.fill_rect(
                    Rect::new(
                        frame.x + 44 * s as i32,
                        body.y - 3 * s as i32,
                        40 * s,
                        2 * s,
                    ),
                    0x167a70,
                );
            }
            c.fill_rect(body, palette.background);
            let left = rect.x.max(body.x);
            let top = rect.y.max(body.y);
            let right = (rect.x + rect.width as i32).min(body.x + body.width as i32);
            let bottom = (rect.y + rect.height as i32).min(body.y + body.height as i32);
            if self.windows[index].offset + body.width as usize * body.height as usize
                > backing.len()
            {
                return Err(Invalid);
            }
            for y in top..bottom {
                for x in left..right {
                    let at = self.windows[index].offset
                        + (y - body.y) as usize * body.width as usize
                        + (x - body.x) as usize;
                    c.pixel(x, y, backing[at]);
                }
            }
        }
        let dock = self.dock();
        c.rounded_rect(dock, 18 * s, 0xe9f0f2);
        for index in 0..WINDOWS {
            let r = Rect::new(
                dock.x + (12 + index as i32 * 56) * s as i32,
                dock.y + 8 * s as i32,
                48 * s,
                48 * s,
            );
            c.rounded_rect(
                r,
                12 * s,
                if index == self.focused {
                    0xd0e9e3
                } else {
                    0xf7f9fa
                },
            );
            c.icon(
                Rect::new(r.x + 12 * s as i32, r.y + 12 * s as i32, 24 * s, 24 * s),
                if index == 0 {
                    Icon::Material
                } else {
                    Icon::Apps
                },
                0x167a70,
                2 * s,
            );
        }
        let (x, y) = (self.pointer.0 as i32, self.pointer.1 as i32);
        c.line(x, y, x, y + 18 * s as i32, 3 * s, 0xffffff);
        c.line(x, y, x + 12 * s as i32, y + 12 * s as i32, 3 * s, 0xffffff);
        c.line(x, y, x + 3 * s as i32, y + 15 * s as i32, s, 0x172a3a);
        Ok(())
    }
}

pub struct Damage {
    bits: [u64; 128],
    width: u32,
    height: u32,
    cursor: usize,
}
impl Damage {
    pub fn new(width: u32, height: u32) -> Result<Self, Invalid> {
        if width == 0 || height == 0 || width > 1920 || height > 1080 {
            return Err(Invalid);
        }
        Ok(Self {
            bits: [0; 128],
            width,
            height,
            cursor: 0,
        })
    }
    pub fn mark(&mut self, r: Rect) {
        let columns = self.width.div_ceil(32);
        let left = r.x.clamp(0, self.width as i32) as u32 / 32;
        let top = r.y.clamp(0, self.height as i32) as u32 / 8;
        let right = (i64::from(r.x) + i64::from(r.width)).clamp(0, i64::from(self.width)) as u32;
        let bottom = (i64::from(r.y) + i64::from(r.height)).clamp(0, i64::from(self.height)) as u32;
        for y in top..bottom.div_ceil(8) {
            for x in left..right.div_ceil(32) {
                let at = (y * columns + x) as usize;
                if at < 8192 {
                    self.bits[at / 64] |= 1 << (at % 64);
                }
            }
        }
        self.cursor = 0;
    }
    pub fn next_tile(&mut self) -> Option<Rect> {
        let columns = self.width.div_ceil(32) as usize;
        let count = columns * self.height.div_ceil(8) as usize;
        while self.cursor < count {
            let at = self.cursor;
            self.cursor += 1;
            if self.bits[at / 64] & (1 << (at % 64)) != 0 {
                self.bits[at / 64] &= !(1 << (at % 64));
                let (x, y) = ((at % columns) as u32 * 32, (at / columns) as u32 * 8);
                return Some(Rect::new(
                    x as i32,
                    y as i32,
                    32.min(self.width - x),
                    8.min(self.height - y),
                ));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_backing_at_both_actual_scales() {
        for (w, h, s) in [(1280, 720, 1), (1920, 1080, 2), (1920, 1080, 1)] {
            let scene = Scene::new(w, h, s, [1, 2]).unwrap();
            let last = scene.windows[1];
            assert!(
                last.offset + last.body.width as usize * last.body.height as usize <= MAX_PIXELS
            );
            for i in 0..2 {
                let f = scene.frame(i);
                assert!(f.x >= 0 && f.y >= 0);
                assert!(f.x + f.width as i32 <= w as i32);
            }
        }
    }
    #[test]
    fn body_input_cannot_start_title_drag_and_focus_tracks_z_order() {
        let mut s = Scene::new(1280, 720, 1, [1, 2]).unwrap();
        let b = s.windows[1].body;
        assert_eq!(
            s.mouse(b.x as u32 + 20, b.y as u32 + 20, true, true)
                .unwrap(),
            Some(1)
        );
        assert_eq!(s.focused, 1);
        assert!(s.drag.is_none());
        assert_eq!(
            s.mouse(b.x as u32 + 20, b.y as u32 - 20, true, true)
                .unwrap(),
            None
        );
        s.mouse(800, 100, false, true).unwrap();
        assert_eq!(s.moves, 1);
        s.mouse(800, 100, false, false).unwrap();
        assert!(s.drag.is_none());
    }
    #[test]
    fn tile_rejection_is_atomic_and_scoped_to_own_backing() {
        let s = Scene::new(1280, 720, 1, [1, 2]).unwrap();
        let mut backing = std::vec![0x123456u32;MAX_PIXELS];
        let mut p = [0u8; 8];
        p[7] = 1;
        let r = Rectangle {
            x: 0,
            y: 0,
            width: 2,
            height: 1,
        };
        assert_eq!(s.copy_tile(0, r, &p, &mut backing), Err(Invalid));
        assert_eq!(backing[0], 0x123456);
        p[7] = 0;
        p[0] = 0x77;
        s.copy_tile(1, r, &p, &mut backing).unwrap();
        assert_eq!(backing[0], 0x123456);
        assert_eq!(backing[s.windows[1].offset], 0x77);
    }
    #[test]
    fn damage_clips_coordinates_and_visits_each_tile_once() {
        let mut d = Damage::new(1280, 720).unwrap();
        d.mark(Rect::new(-50, -50, 90, 60));
        assert_eq!(d.next_tile(), Some(Rect::new(0, 0, 32, 8)));
        let mut count = 1;
        while d.next_tile().is_some() {
            count += 1;
        }
        assert_eq!(count, 4);
        d.mark(Rect::new(1279, 719, u32::MAX, u32::MAX));
        assert_eq!(d.next_tile(), Some(Rect::new(1248, 712, 32, 8)));
        assert_eq!(d.next_tile(), None);
    }
}
