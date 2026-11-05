//! Generic orchestration over the real typed INP1/SUR1 sessions. A host
//! Transport can test this contract, but it is never part of the native ELF.
use crate::{
    model::{Model, Notice},
    render::{self, Layout, Rect, Viewport},
};
use nieuw_os_input_client::{
    clipboard, AckOutcome, Event, Offer, Peek, Session as Input, Transport,
};
use nieuw_os_surface_protocol::{client::Session as Surface, Geometry};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Evidence {
    pub ready: bool,
    pub frames: u64,
    pub tiles: u64,
    pub keys: u64,
    pub mice: u64,
    pub input_acks: u64,
    pub superseded: u64,
    pub errors: u64,
    pub presented_generation: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Binding,
    Counter,
    Input(nieuw_os_input_client::Error),
    Surface(nieuw_os_surface_protocol::client::Error),
    Cleanup(nieuw_os_sdk_runtime::Error),
}
fn increment(n: &mut u64) -> Result<(), Error> {
    *n = n.checked_add(1).ok_or(Error::Counter)?;
    Ok(())
}
pub struct App<I: Transport, S: Transport> {
    model: Model,
    input: Input<I>,
    surface: Surface<S>,
    geometry: Geometry,
    viewport: Viewport,
    evidence: Evidence,
    input_responded: bool,
    input_empty: bool,
    step_succeeded: bool,
    input_failed: bool,
    surface_failed: bool,
    frame: u64,
    next_tile: u64,
    clipboard_enabled: bool,
    retained_enabled: bool,
}
impl<I: Transport, S: Transport> App<I, S> {
    pub fn new(
        mut input: Input<I>,
        mut surface: Surface<S>,
        geometry: Geometry,
        scale: u32,
    ) -> Result<Self, Error> {
        let binding = input.binding();
        if geometry.validate().is_err()
            || binding.width != geometry.width
            || binding.height != geometry.height
        {
            let _ = input.close();
            let _ = surface.close();
            return Err(Error::Binding);
        }
        let Some(viewport) = Viewport::new(
            u32::from(geometry.assigned.width),
            u32::from(geometry.assigned.height),
            scale,
        ) else {
            let _ = input.close();
            let _ = surface.close();
            return Err(Error::Binding);
        };
        Ok(Self {
            model: Model::new(),
            input,
            surface,
            geometry,
            viewport,
            evidence: Evidence::default(),
            input_responded: false,
            input_empty: false,
            step_succeeded: false,
            input_failed: false,
            surface_failed: false,
            frame: 0,
            next_tile: 0,
            clipboard_enabled: false,
            retained_enabled: false,
        })
    }
    pub fn model(&self) -> &Model {
        &self.model
    }
    /// Selects the window compositor extension; broker checks remain required.
    pub fn enable_clipboard(&mut self) {
        self.clipboard_enabled = true;
    }
    /// Requires the jointly built compositor's private retained backing.
    pub fn enable_retained(&mut self) -> Result<(), Error> {
        self.surface.enable_retained().map_err(Error::Surface)?;
        self.retained_enabled = true;
        Ok(())
    }
    pub fn surface_counts(&self) -> nieuw_os_surface_protocol::client::SurfaceCounts {
        self.surface.counts()
    }
    /// An acknowledged complete frame and a successful PEEK Empty turn are
    /// required. Exhausted input batches, failed transports and presentation
    /// recovery cannot be mistaken for quiescence.
    pub fn can_idle(&self) -> bool {
        self.step_succeeded
            && self.input_empty
            && !self.input_failed
            && !self.surface_failed
            && !self.input.is_closed()
            && self.input.outstanding().is_none()
            && !self.surface.is_closed()
            && !self.model.exhausted()
            && self.evidence.ready
            && self.evidence.presented_generation == self.model.generation()
            && self.frame == self.model.generation()
            && self.next_tile == self.geometry.tiles()
    }
    pub const fn viewport(&self) -> Viewport {
        self.viewport
    }
    pub fn evidence(&self) -> Evidence {
        let mut e = self.evidence;
        e.ready &= self.model.generation() == e.presented_generation && !self.model.exhausted();
        e
    }
    fn clipboard_offer(
        &mut self,
        offer: &Offer,
    ) -> Result<(Option<clipboard::Text>, Option<Notice>), Error> {
        use ui_core::{KeyAction, PhysicalKey};
        let Event::Key(key) = offer.event() else {
            return Ok((None, None));
        };
        let m = key.modifiers;
        if !self.clipboard_enabled
            || key.action != KeyAction::Press
            || !m.control()
            || m.shift()
            || m.alt()
            || m.super_key()
        {
            return Ok((None, None));
        }
        let (result, feedback) = match key.key {
            PhysicalKey::C => {
                let selected = self.model.clipboard_selection();
                let empty = selected.is_empty();
                (
                    self.input.clipboard_copy(offer, selected).map(|()| None),
                    Some(if empty {
                        Notice::ClipboardSelection
                    } else {
                        Notice::ClipboardCopied
                    }),
                )
            }
            PhysicalKey::V => (self.input.clipboard_paste(offer).map(Some), None),
            _ => return Ok((None, None)),
        };
        match result {
            Ok(text) => Ok((text, feedback)),
            Err(clipboard::Error::Input(error)) => {
                self.input_failed = true;
                self.evidence.ready = false;
                self.model.input_lost();
                increment(&mut self.evidence.errors)?;
                Err(Error::Input(error))
            }
            Err(error) => Ok((
                None,
                Some(match error {
                    clipboard::Error::Empty if feedback == Some(Notice::ClipboardSelection) => {
                        Notice::ClipboardSelection
                    }
                    clipboard::Error::Empty => Notice::ClipboardEmpty,
                    clipboard::Error::TooLong => Notice::ClipboardTooLong,
                    _ => Notice::ClipboardUnavailable,
                }),
            )),
        }
    }
    fn input_step(&mut self) -> Result<(), Error> {
        self.input_empty = false;
        if self.input_failed || self.surface_failed {
            return Ok(());
        }
        for _ in 0..4 {
            let offer = match self.input.peek() {
                Ok(Peek::Empty) => {
                    self.input_responded = true;
                    self.input_empty = true;
                    break;
                }
                Ok(Peek::Offer(offer)) => {
                    self.input_responded = true;
                    offer
                }
                Err(error) => {
                    self.input_failed = true;
                    self.evidence.ready = false;
                    self.model.input_lost();
                    increment(&mut self.evidence.errors)?;
                    return Err(Error::Input(error));
                }
            };
            match offer.event() {
                Event::Key(key) => {
                    increment(&mut self.evidence.keys)?;
                    self.model.focus_gained();
                    self.model.keyboard(key);
                }
                Event::Cancel(_) => self.model.cancel_focus(),
                Event::Mouse { input, x, y } => {
                    increment(&mut self.evidence.mice)?;
                    self.model.focus_gained();
                    let r = self.geometry.assigned;
                    if input.x_overflow
                        || input.y_overflow
                        || x < r.x
                        || y < r.y
                        || x - r.x >= u32::from(r.width)
                        || y - r.y >= u32::from(r.height)
                    {
                        self.model.cancel_pointer();
                    } else {
                        let (x, y) = (
                            (x - r.x) / self.viewport.scale(),
                            (y - r.y) / self.viewport.scale(),
                        );
                        let layout = Layout::new(self.viewport);
                        let (down, up) = (
                            input.changed.left() && input.buttons.left(),
                            input.changed.left() && !input.buttons.left(),
                        );
                        if layout.result_bounds(self.viewport.scale()).contains(x, y) && down {
                            self.model.command(crate::model::Command::SelectResult);
                        } else if layout.expression.contains(x, y) {
                            let position = render::expression_caret_at(
                                &self.model,
                                layout,
                                self.viewport.scale(),
                                x,
                            );
                            self.model.caret_at(position, down, input.buttons.left());
                        } else {
                            self.model.pointer(layout.hit(x, y), down, up);
                        }
                    }
                }
            }
            let (paste, feedback) = self.clipboard_offer(&offer)?;
            match self.input.ack(&offer) {
                Ok(AckOutcome::Acknowledged) => {
                    increment(&mut self.evidence.input_acks)?;
                    if let Some(text) = paste {
                        // Model validates the complete replacement before mutation.
                        let _ = self.model.paste_clipboard(text.as_str());
                    }
                    if let Some(notice) = feedback {
                        self.model.clipboard_notice(notice);
                    }
                }
                Ok(AckOutcome::Superseded) => {
                    increment(&mut self.evidence.superseded)?;
                    self.model.cancel_focus();
                }
                Err(error) => {
                    self.input_failed = true;
                    self.evidence.ready = false;
                    self.model.input_lost();
                    increment(&mut self.evidence.errors)?;
                    return Err(Error::Input(error));
                }
            }
        }
        Ok(())
    }
    fn paint_tile(&mut self) -> Result<(), Error> {
        if self.surface_failed || self.model.generation() == self.evidence.presented_generation {
            return Ok(());
        }
        self.evidence.ready = false;
        if self.frame != self.model.generation() {
            self.frame = self.model.generation();
            self.next_tile = 0;
        }
        let damage = self.model.paint_damage(self.viewport);
        let mut retained = 0;
        if self.retained_enabled && self.surface.retained_base().is_some() && !self.input_failed {
            while retained < nieuw_os_surface_protocol::MAX_RETAIN_TILES {
                let index = self
                    .next_tile
                    .checked_add(u64::from(retained))
                    .ok_or(Error::Binding)?;
                if index >= self.geometry.tiles() {
                    break;
                }
                let r = self.geometry.rectangle(index).map_err(|_| Error::Binding)?;
                if damage.intersects(r.x, r.y, u32::from(r.width), u32::from(r.height)) {
                    break;
                }
                retained += 1;
            }
        }
        let result = if retained != 0 {
            self.surface.retain(self.frame, self.next_tile, retained)
        } else {
            let r = self
                .geometry
                .rectangle(self.next_tile)
                .map_err(|_| Error::Binding)?;
            let mut pixels = [0u32; 256];
            render::render_tile(
                &self.model,
                self.viewport,
                Rect {
                    x: r.x,
                    y: r.y,
                    width: u32::from(r.width),
                    height: u32::from(r.height),
                },
                &mut pixels,
            )
            .map_err(|_| Error::Binding)?;
            let mut bytes = [0u8; 1024];
            for (pixel, chunk) in pixels.iter().zip(bytes[..r.bytes()].chunks_exact_mut(4)) {
                chunk.copy_from_slice(&pixel.to_le_bytes());
            }
            self.surface
                .blit(self.frame, self.next_tile, r, &bytes[..r.bytes()])
        };
        if let Err(error) = result {
            self.surface_failed = true;
            self.evidence.ready = false;
            let _ = self.input.close();
            self.model.cancel_focus();
            increment(&mut self.evidence.errors)?;
            return Err(Error::Surface(error));
        }
        if retained == 0 {
            increment(&mut self.evidence.tiles)?;
        }
        self.next_tile = self
            .next_tile
            .checked_add(u64::from(retained.max(1)))
            .ok_or(Error::Binding)?;
        if self.next_tile == self.geometry.tiles() {
            if !self.model.presented(self.frame) {
                return Err(Error::Binding);
            }
            increment(&mut self.evidence.frames)?;
            self.evidence.presented_generation = self.frame;
            self.evidence.ready = self.input_responded && !self.input_failed;
        }
        Ok(())
    }
    /// At most four input offers and eight scalar tile transactions. A partial
    /// frame never certifies a new current generation. No clipboard/files/IPC
    /// arithmetic backend or automatic replay is hidden in this iteration.
    pub fn step(&mut self) -> Result<(), Error> {
        self.step_succeeded = false;
        if self.model.exhausted() {
            return Err(Error::Counter);
        }
        let input = self.input_step();
        if self.model.exhausted() {
            return Err(Error::Counter);
        }
        for _ in 0..8 {
            self.paint_tile()?;
        }
        self.step_succeeded = input.is_ok();
        input
    }
    /// Calculator work is volatile. STOP closes both owners even if the first
    /// close fails; it does not save, discard a document or touch storage.
    pub fn stop_clean(&mut self) -> Result<(), Error> {
        self.step_succeeded = false;
        let input = self.input.close().map_err(Error::Cleanup);
        let surface = self.surface.close().map_err(Error::Cleanup);
        self.evidence.ready = false;
        input.and(surface)
    }
    pub fn detach_presentation(&mut self) -> Result<(), Error> {
        self.step_succeeded = false;
        self.input_empty = false;
        self.input_failed = true;
        self.surface_failed = true;
        self.evidence.ready = false;
        let input = self.input.retire().map_err(Error::Cleanup);
        let surface = self.surface.retire().map_err(Error::Cleanup);
        input.and(surface)
    }
    pub fn rebind_presentation(
        &mut self,
        mut input: Input<I>,
        mut surface: Surface<S>,
        geometry: Geometry,
    ) -> Result<(), Error> {
        let binding = input.binding();
        if !self.surface_failed
            || !self.input_failed
            || geometry.validate().is_err()
            || geometry.epoch == self.geometry.epoch
            || binding.epoch == self.input.binding().epoch
            || binding.width != geometry.width
            || binding.height != geometry.height
        {
            let _ = input.close();
            let _ = surface.close();
            return Err(Error::Binding);
        }
        let Some(viewport) = Viewport::new(
            u32::from(geometry.assigned.width),
            u32::from(geometry.assigned.height),
            self.viewport.scale(),
        ) else {
            let _ = input.close();
            let _ = surface.close();
            return Err(Error::Binding);
        };
        if self.retained_enabled {
            if let Err(error) = surface.enable_retained() {
                let _ = input.close();
                let _ = surface.close();
                return Err(Error::Surface(error));
            }
        }
        self.model.force_full_repaint();
        self.input = input;
        self.surface = surface;
        self.geometry = geometry;
        self.viewport = viewport;
        self.input_failed = false;
        self.surface_failed = false;
        self.input_responded = false;
        self.input_empty = false;
        self.step_succeeded = false;
        self.frame = 0;
        self.next_tile = 0;
        self.evidence.presented_generation = 0;
        self.evidence.ready = false;
        Ok(())
    }
}
