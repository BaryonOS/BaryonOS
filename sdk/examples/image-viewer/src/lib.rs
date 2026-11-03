//! App-local image state and scalar tiles; no object, input or display authority.
//! The future native adapter authenticates service replies and owns all handles.
#![no_std]
#![forbid(unsafe_code)]

use nieuw_os_image_codec::{decode, Cancellation, Image, Limits, Pixels};
use nieuw_os_input_core::Event;
use nieuw_os_surface_protocol::{Geometry, Rectangle};
use ui_core::{KeyAction, PhysicalKey};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Selection {
    pub object: u64,
    pub revision: u64,
    pub byte_budget: usize,
    pub pixel_budget: usize,
}

/// Correlation only, never a capability or proof of the selected object's origin.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Load {
    generation: u64,
    selection: Selection,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Failure {
    Denied,
    Revoked,
    Stale,
    Io,
    TooLarge,
    Corrupt,
    Unsupported,
    OutOfMemory,
    Cancelled,
    Decoder(nieuw_os_image_codec::Error),
}

impl Failure {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Denied => "Geen toegang tot deze afbeelding.",
            Self::Revoked => "Toegang tot de afbeelding is ingetrokken.",
            Self::Stale => "De gekozen versie is gewijzigd. Open opnieuw.",
            Self::Io => "De afbeelding kon niet worden gelezen.",
            Self::TooLarge => "De afbeelding past niet binnen het afgesproken geheugenbudget.",
            Self::Corrupt => "De afbeelding is beschadigd.",
            Self::Unsupported => "Dit afbeeldingsformaat wordt niet ondersteund.",
            Self::OutOfMemory => "Onvoldoende geheugen om de afbeelding te openen.",
            Self::Cancelled => "Openen geannuleerd.",
            Self::Decoder(_) => "De afbeelding kon niet worden verwerkt.",
        }
    }
}

impl From<nieuw_os_image_codec::Error> for Failure {
    fn from(error: nieuw_os_image_codec::Error) -> Self {
        use nieuw_os_image_codec::Error as E;
        match error {
            E::InputLimit | E::Dimensions | E::WorkLimit | E::MetadataLimit => Self::TooLarge,
            E::InvalidData | E::OutputSize => Self::Corrupt,
            E::UnknownFormat
            | E::UnsupportedAnimation
            | E::UnsupportedColor
            | E::UnsupportedPrecision => Self::Unsupported,
            E::Cancelled => Self::Cancelled,
            E::OutOfMemory => Self::OutOfMemory,
            _ => Self::Decoder(error),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum State {
    Empty,
    Loading,
    Ready,
    Failed(Failure),
    Closed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Configuration,
    Closed,
    ObsoleteLoad,
    Exhausted,
    NoImage,
    Tile,
    Capacity,
    ObsoleteFrame,
    Load(Failure),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Zoom {
    Fit,
    Percent(u16),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Command {
    Fit,
    Actual,
    ZoomIn,
    ZoomOut,
    Pan { x: i32, y: i32 },
    Cancel,
    Close,
}

/// Returned only after a tile has actually been rendered. The adapter may
/// acknowledge it ONLY after the real SUR1 blit/COMMIT has succeeded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Tile {
    generation: u64,
    epoch: u64,
    index: u64,
    rectangle: Rectangle,
}

impl Tile {
    pub const fn frame(self) -> u64 {
        self.generation
    }
    pub const fn index(self) -> u64 {
        self.index
    }
    pub const fn rectangle(self) -> Rectangle {
        self.rectangle
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Semantics {
    pub state: State,
    pub status: &'static str,
    pub image_size: Option<(usize, usize)>,
    pub zoom: Zoom,
    pub focused: bool,
    pub presented: bool,
    pub metadata_transforms_applied: bool,
}

pub struct Viewer<'a> {
    geometry: Geometry,
    scale: u32,
    state: State,
    selection: Option<Selection>,
    image: Option<Image<'a>>,
    generation: u64,
    zoom: Zoom,
    pan_x: u32,
    pan_y: u32,
    drag: Option<(u32, u32)>,
    focused: bool,
    next_ack: u64,
    presented: bool,
}

impl<'a> Viewer<'a> {
    pub fn new(geometry: Geometry, scale: u32) -> Result<Self, Error> {
        geometry.validate().map_err(|_| Error::Configuration)?;
        if !matches!(scale, 1 | 2) {
            return Err(Error::Configuration);
        }

        Ok(Self {
            geometry,
            scale,
            state: State::Empty,
            selection: None,
            image: None,
            generation: 1,
            zoom: Zoom::Fit,
            pan_x: 0,
            pan_y: 0,
            drag: None,
            focused: false,
            next_ack: 0,
            presented: false,
        })
    }

    pub const fn state(&self) -> State {
        self.state
    }
    pub const fn generation(&self) -> u64 {
        self.generation
    }
    pub const fn selection(&self) -> Option<Selection> {
        self.selection
    }
    pub const fn pan(&self) -> (u32, u32) {
        (self.pan_x, self.pan_y)
    }
    pub fn image(&self) -> Option<&Image<'a>> {
        self.image.as_ref()
    }

    fn changed(&mut self) -> Result<(), Error> {
        self.presented = false;
        self.next_ack = 0;
        self.generation = match self.generation.checked_add(1) {
            Some(next) => next,
            None => {
                self.image = None;
                self.selection = None;
                self.drag = None;
                self.state = State::Closed;
                return Err(Error::Exhausted);
            }
        };
        Ok(())
    }

    fn live(&self) -> Result<(), Error> {
        if self.state == State::Closed {
            Err(Error::Closed)
        } else {
            Ok(())
        }
    }

    pub fn begin(&mut self, selection: Selection) -> Result<Load, Error> {
        self.live()?;
        if selection.revision == 0 || selection.byte_budget == 0 || selection.pixel_budget == 0 {
            return Err(Error::Configuration);
        }
        self.changed()?;
        self.image = None;
        self.selection = Some(selection);
        self.state = State::Loading;
        self.zoom = Zoom::Fit;
        self.pan_x = 0;
        self.pan_y = 0;
        self.drag = None;
        Ok(Load {
            generation: self.generation,
            selection,
        })
    }

    fn current(&self, load: Load) -> Result<(), Error> {
        self.live()?;
        if self.state != State::Loading
            || load.generation != self.generation
            || self.selection != Some(load.selection)
        {
            Err(Error::ObsoleteLoad)
        } else {
            Ok(())
        }
    }

    pub fn fail(&mut self, load: Load, failure: Failure) -> Result<(), Error> {
        self.current(load)?;
        self.changed()?;
        self.image = None;
        self.drag = None;
        self.state = State::Failed(failure);
        Ok(())
    }

    /// Synchronous decode, not a responsive UI event loop. A native adapter
    /// still needs bounded private process memory and real job cancellation.
    pub fn decode(
        &mut self,
        load: Load,
        observed: Selection,
        bytes: &'a [u8],
        limits: Limits,
        cancellation: &Cancellation,
    ) -> Result<(), Error> {
        self.current(load)?;
        if observed != load.selection {
            return self.reject(load, Failure::Stale);
        }
        if bytes.len() > load.selection.byte_budget {
            return self.reject(load, Failure::TooLarge);
        }
        let limits = Limits {
            pixels: limits.pixels.min(load.selection.pixel_budget),
            ..limits
        };
        match decode(bytes, limits, cancellation) {
            Ok(image) => self.decoded(load, observed, image),
            Err(error) => self.reject(load, error.into()),
        }
    }

    fn reject(&mut self, load: Load, failure: Failure) -> Result<(), Error> {
        self.fail(load, failure)?;
        Err(Error::Load(failure))
    }

    /// Accept an already decoded result from the caller's isolated decoder.
    /// `observed` is correlation data, NOT authenticated storage provenance.
    pub fn decoded(
        &mut self,
        load: Load,
        observed: Selection,
        image: Image<'a>,
    ) -> Result<(), Error> {
        self.current(load)?;
        if observed != load.selection {
            return self.reject(load, Failure::Stale);
        }
        if image.source.len() > load.selection.byte_budget {
            return self.reject(load, Failure::TooLarge);
        }
        let samples = image
            .width
            .checked_mul(image.height)
            .and_then(|n| n.checked_mul(4));
        let actual = match &image.pixels {
            Pixels::Rgba8(pixels) => pixels.len(),
            Pixels::Rgba16(pixels) => pixels.len(),
        };
        if image.width == 0
            || image.height == 0
            || image.width > 16384
            || image.height > 16384
            || samples != Some(actual)
        {
            return self.reject(load, Failure::Corrupt);
        }
        if image.width * image.height > load.selection.pixel_budget {
            return self.reject(load, Failure::TooLarge);
        }
        self.changed()?;
        self.image = Some(image);
        self.state = State::Ready;
        Ok(())
    }

    /// Drop future access to held pixels. This does not revoke kernel grants
    /// or retract copies already displayed; the owner must retire its surface.
    pub fn revoke(&mut self) -> Result<(), Error> {
        self.live()?;
        self.changed()?;
        self.image = None;
        self.selection = None;
        self.drag = None;
        self.state = State::Failed(Failure::Revoked);
        Ok(())
    }

    pub fn close(&mut self) -> Result<(), Error> {
        if self.state == State::Closed {
            return Ok(());
        }
        self.changed()?;
        self.image = None;
        self.selection = None;
        self.drag = None;
        self.focused = false;
        self.state = State::Closed;
        Ok(())
    }

    pub fn semantics(&self) -> Semantics {
        Semantics {
            state: self.state,
            status: match self.state {
                State::Empty => "Geen afbeelding geopend.",
                State::Loading => "Afbeelding openen.",
                State::Ready => "Afbeelding geopend.",
                State::Failed(failure) => failure.label(),
                State::Closed => "Afbeeldingsviewer gesloten.",
            },
            image_size: self.image.as_ref().map(|image| (image.width, image.height)),
            zoom: self.zoom,
            focused: self.focused,
            presented: self.presented,
            metadata_transforms_applied: self
                .image
                .as_ref()
                .is_some_and(|image| image.metadata_transforms_applied),
        }
    }

    fn dimensions(&self) -> Result<(u32, u32), Error> {
        let image = self.image.as_ref().ok_or(Error::NoImage)?;
        let (w, h) = (image.width as u64, image.height as u64);
        let (vw, vh) = (
            u64::from(self.geometry.assigned.width),
            u64::from(self.geometry.assigned.height),
        );
        let (width, height) = match self.zoom {
            Zoom::Fit if vw * h <= vh * w => (vw, (h * vw / w).max(1)),
            Zoom::Fit => ((w * vh / h).max(1), vh),
            Zoom::Percent(percent) => (
                (w * u64::from(percent) * u64::from(self.scale) / 100).max(1),
                (h * u64::from(percent) * u64::from(self.scale) / 100).max(1),
            ),
        };
        Ok((
            u32::try_from(width).map_err(|_| Error::Configuration)?,
            u32::try_from(height).map_err(|_| Error::Configuration)?,
        ))
    }

    fn pan_limits(&self) -> Result<(u32, u32), Error> {
        let (w, h) = self.dimensions()?;
        Ok((
            w.saturating_sub(u32::from(self.geometry.assigned.width)),
            h.saturating_sub(u32::from(self.geometry.assigned.height)),
        ))
    }

    pub fn command(&mut self, command: Command) -> Result<(), Error> {
        self.live()?;
        if command == Command::Close {
            return self.close();
        }
        if command == Command::Cancel {
            self.changed()?;
            self.image = None;
            self.selection = None;
            self.drag = None;
            self.state = State::Failed(Failure::Cancelled);
            return Ok(());
        }
        if self.state != State::Ready {
            return Err(Error::NoImage);
        }
        if let Command::Pan { x, y } = command {
            let (max_x, max_y) = self.pan_limits()?;
            let next_x = (i64::from(self.pan_x) + i64::from(x)).clamp(0, i64::from(max_x)) as u32;
            let next_y = (i64::from(self.pan_y) + i64::from(y)).clamp(0, i64::from(max_y)) as u32;
            if (next_x, next_y) != (self.pan_x, self.pan_y) {
                self.changed()?;
                self.pan_x = next_x;
                self.pan_y = next_y;
            }
            return Ok(());
        }
        let zoom = match command {
            Command::Fit => Zoom::Fit,
            Command::Actual => Zoom::Percent(100),
            Command::ZoomIn | Command::ZoomOut => {
                let (numerator, denominator) = match self.zoom {
                    Zoom::Percent(percent) => (u64::from(percent), 1),
                    Zoom::Fit => {
                        let image = self.image.as_ref().ok_or(Error::NoImage)?;
                        let (w, h) = (image.width as u64, image.height as u64);
                        let (vw, vh) = (
                            u64::from(self.geometry.assigned.width),
                            u64::from(self.geometry.assigned.height),
                        );
                        if vw * h <= vh * w {
                            (vw * 100, w * u64::from(self.scale))
                        } else {
                            (vh * 100, h * u64::from(self.scale))
                        }
                    }
                };
                let steps = [25u16, 50, 100, 200, 400, 800];
                let next = if command == Command::ZoomIn {
                    steps
                        .into_iter()
                        .find(|p| u64::from(*p) * denominator > numerator)
                } else {
                    steps
                        .into_iter()
                        .rev()
                        .find(|p| u64::from(*p) * denominator < numerator)
                };
                match next {
                    Some(percent) => Zoom::Percent(percent),
                    None => return Ok(()),
                }
            }
            _ => return Err(Error::Configuration),
        };
        if self.zoom != zoom {
            self.changed()?;
            self.zoom = zoom;
            let (x, y) = self.pan_limits()?;
            self.pan_x = x / 2;
            self.pan_y = y / 2;
            self.drag = None;
        }
        Ok(())
    }

    /// Only call with a real, validated INP1 offer. The adapter owns epoch,
    /// sequence, ACK and superseded-offer handling; this function grants none.
    pub fn input(&mut self, event: Event) -> Result<(), Error> {
        self.live()?;
        match event {
            Event::PointerLeave => self.drag = None,
            Event::Cancel(_) => {
                self.drag = None;
                self.focused = false;
            }
            Event::Key(key) => {
                self.focused = true;
                if matches!(key.action, KeyAction::Release)
                    || key.modifiers.alt()
                    || key.modifiers.super_key()
                    || key.modifiers.control()
                {
                    return Ok(());
                }
                let step = (32 * self.scale) as i32;
                let command = match key.key {
                    PhysicalKey::Escape => Some(Command::Close),
                    PhysicalKey::Equal | PhysicalKey::KeypadAdd => Some(Command::ZoomIn),
                    PhysicalKey::Minus | PhysicalKey::KeypadSubtract => Some(Command::ZoomOut),
                    PhysicalKey::Digit0 | PhysicalKey::Home => Some(Command::Fit),
                    PhysicalKey::Digit1 => Some(Command::Actual),
                    PhysicalKey::Left => Some(Command::Pan { x: -step, y: 0 }),
                    PhysicalKey::Right => Some(Command::Pan { x: step, y: 0 }),
                    PhysicalKey::Up => Some(Command::Pan { x: 0, y: -step }),
                    PhysicalKey::Down => Some(Command::Pan { x: 0, y: step }),
                    _ => None,
                };
                if let Some(command) = command {
                    if self.state == State::Ready || command == Command::Close {
                        self.command(command)?;
                    }
                }
            }
            Event::Mouse { input, x, y } => {
                let r = self.geometry.assigned;
                if input.x_overflow
                    || input.y_overflow
                    || x < r.x
                    || y < r.y
                    || x - r.x >= u32::from(r.width)
                    || y - r.y >= u32::from(r.height)
                {
                    self.drag = None;
                    return Ok(());
                }
                self.focused = true;
                if self.state != State::Ready || !input.buttons.left() {
                    self.drag = None;
                } else if input.changed.left() {
                    self.drag = Some((x, y));
                } else if let Some((old_x, old_y)) = self.drag {
                    self.command(Command::Pan {
                        x: old_x as i32 - x as i32,
                        y: old_y as i32 - y as i32,
                    })?;
                    self.drag = Some((x, y));
                }
            }
        }
        Ok(())
    }

    /// Render source samples directly into one caller-owned BGRX tile. No
    /// whole-image conversion allocation; original 16-bit samples are retained.
    pub fn render_tile(&self, index: u64, background: u32, out: &mut [u32]) -> Result<Tile, Error> {
        if self.state != State::Ready {
            return Err(Error::NoImage);
        }
        if background & 0xff00_0000 != 0 {
            return Err(Error::Configuration);
        }
        let rectangle = self.geometry.rectangle(index).map_err(|_| Error::Tile)?;
        let count = rectangle.bytes() / 4;
        if out.len() < count {
            return Err(Error::Capacity);
        }
        let image = self.image.as_ref().ok_or(Error::NoImage)?;
        let (w, h) = self.dimensions()?;
        let (vw, vh) = (
            u32::from(self.geometry.assigned.width),
            u32::from(self.geometry.assigned.height),
        );
        let left = i64::from(vw.saturating_sub(w) / 2) - i64::from(self.pan_x);
        let top = i64::from(vh.saturating_sub(h) / 2) - i64::from(self.pan_y);
        for row in 0..u32::from(rectangle.height) {
            for col in 0..u32::from(rectangle.width) {
                let x = i64::from(rectangle.x + col) - left;
                let y = i64::from(rectangle.y + row) - top;
                let pixel = if x < 0 || y < 0 || x >= i64::from(w) || y >= i64::from(h) {
                    background
                } else {
                    let sx = x as usize * image.width / w as usize;
                    let sy = y as usize * image.height / h as usize;
                    let at = (sy * image.width + sx) * 4;
                    let sample = |channel: usize| -> u32 {
                        match &image.pixels {
                            Pixels::Rgba8(p) => u32::from(p[at + channel]),
                            Pixels::Rgba16(p) => (u32::from(p[at + channel]) + 128) / 257,
                        }
                    };
                    let alpha = sample(3);
                    let blend = |channel: usize, shift: u32| {
                        (sample(channel) * alpha
                            + ((background >> shift) & 255) * (255 - alpha)
                            + 127)
                            / 255
                    };
                    (blend(0, 16) << 16) | (blend(1, 8) << 8) | blend(2, 0)
                };
                out[(row * u32::from(rectangle.width) + col) as usize] = pixel;
            }
        }
        Ok(Tile {
            generation: self.generation,
            epoch: self.geometry.epoch,
            index,
            rectangle,
        })
    }

    pub fn acknowledge(&mut self, tile: Tile) -> Result<(), Error> {
        if self.state != State::Ready
            || tile.generation != self.generation
            || tile.epoch != self.geometry.epoch
            || tile.index != self.next_ack
            || self.geometry.rectangle(tile.index).ok() != Some(tile.rectangle)
        {
            return Err(Error::ObsoleteFrame);
        }
        self.next_ack += 1;
        self.presented = self.next_ack == self.geometry.tiles();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exhausted_generation_fails_closed_without_wrapping() {
        let geometry = Geometry {
            width: 32,
            height: 24,
            assigned: Rectangle {
                x: 0,
                y: 0,
                width: 32,
                height: 24,
            },
            epoch: 1,
        };
        let mut viewer = Viewer::new(geometry, 1).unwrap();
        viewer.generation = u64::MAX;
        assert_eq!(
            viewer.begin(Selection {
                object: 0,
                revision: 1,
                byte_budget: 1,
                pixel_budget: 1,
            }),
            Err(Error::Exhausted)
        );
        assert_eq!(viewer.generation(), u64::MAX);
        assert_eq!(viewer.state(), State::Closed);
        assert!(viewer.image().is_none());
        assert!(!viewer.semantics().presented);
    }
}
