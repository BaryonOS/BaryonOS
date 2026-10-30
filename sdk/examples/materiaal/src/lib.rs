//! Native app orchestration. No display/raw-input capabilities, framebuffer
//! mappings, fabricated receive envelopes or allocator are required here.
#![no_std]
#![forbid(unsafe_code)]
pub mod bootstrap;
use nieuw_os_catalog_client::Transport;
use nieuw_os_input_client::{AckOutcome, Peek, Session as Input};
use nieuw_os_input_core::Event;
use nieuw_os_materiaal::{
    controller::{Controller, InputEnvelope, Screen, Status},
    render::Frame,
    TILE_PIXELS,
};
use nieuw_os_surface_protocol::{client::Session as Surface, Geometry};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Evidence {
    pub ready: bool,
    pub frames: u64,
    pub tiles: u64,
    pub keys: u64,
    pub mice: u64,
    pub input_acks: u64,
    pub superseded: u64,
    pub errors: u64,
    pub save_acks: u64,
    pub saved_revision: u64,
    pub saved_bytes: u64,
    pub saved_checksum: u64,
    /// Last actually completed current-generation paint, never an admission.
    pub presented_generation: u64,
    /// Diagnostic only: authenticated INP1 F12 Press, never Repeat/Release.
    pub f12_presses: u64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Observation {
    pub screen: u64,
    pub dirty: bool,
    pub draft_length: u64,
    pub draft_checksum: u64,
    pub object: u64,
    pub scale: u64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Binding,
    CounterExhausted,
    Input(nieuw_os_input_client::Error),
    Surface(nieuw_os_surface_protocol::client::Error),
    Controller(nieuw_os_materiaal::controller::Error),
    Cleanup(nieuw_os_sdk_runtime::Error),
    Unsaved,
}
fn increment(value: &mut u64) -> Result<(), Error> {
    *value = value.checked_add(1).ok_or(Error::CounterExhausted)?;
    Ok(())
}

pub struct App<'a, V: Transport, I: Transport, S: Transport> {
    controller: Controller<'a, V>,
    input: Input<I>,
    surface: Surface<S>,
    geometry: Geometry,
    evidence: Evidence,
    input_responded: bool,
    input_failed: bool,
    surface_failed: bool,
    frame: u64,
    next_tile: u64,
}
impl<'a, V: Transport, I: Transport, S: Transport> App<'a, V, I, S> {
    /// `geometry` must be the actual authenticated INFO response from this
    /// surface session. The caller has constructed three separately owned pairs.
    pub fn new(
        controller: Controller<'a, V>,
        input: Input<I>,
        surface: Surface<S>,
        geometry: Geometry,
    ) -> Result<Self, Error> {
        let view = controller.view();
        let binding = input.binding();
        if geometry.validate().is_err()
            || view.viewport.width() != u32::from(geometry.assigned.width)
            || view.viewport.height() != u32::from(geometry.assigned.height)
            || view.viewport.epoch() != geometry.epoch
            || binding.width != geometry.width
            || binding.height != geometry.height
        {
            return Err(Error::Binding);
        }
        Ok(Self {
            controller,
            input,
            surface,
            geometry,
            evidence: Evidence::default(),
            input_responded: false,
            input_failed: false,
            surface_failed: false,
            frame: 0,
            next_tile: 0,
        })
    }
    pub fn controller(&self) -> &Controller<'a, V> {
        &self.controller
    }
    pub fn evidence(&self) -> Evidence {
        let mut evidence = self.evidence;
        let view = self.controller.view();
        evidence.ready &= !view.dirty && evidence.presented_generation == view.generation;
        evidence
    }
    pub fn observation(&self) -> Observation {
        let view = self.controller.view();
        let draft = self.controller.document().draft();
        Observation {
            screen: match view.screen {
                Screen::Material => 1,
                Screen::Editor => 2,
                Screen::ConfirmClose => 3,
            },
            dirty: self.controller.document().is_dirty(),
            draft_length: draft.map_or(0, |d| d.contents.len() as u64),
            draft_checksum: draft.map_or(0, |d| {
                volume_core::checksum_update(volume_core::FNV_OFFSET, d.contents)
            }),
            object: draft.map_or(u64::MAX, |d| d.object),
            scale: u64::from(view.viewport.scale().factor()),
        }
    }
    pub const fn is_visible(&self) -> bool {
        self.evidence.frames != 0 && !self.surface_failed
    }
    pub const fn input_failed(&self) -> bool {
        self.input_failed
    }
    fn operation(
        &mut self,
        result: Result<(), nieuw_os_materiaal::controller::Error>,
    ) -> Result<(), Error> {
        if result.is_err() {
            increment(&mut self.evidence.errors)?;
        }
        // The controller records the precise storage/edit failure in its view;
        // retain the draft and continue serving cancel/back rather than exit.
        Ok(())
    }
    fn input_step(&mut self) -> Result<(), Error> {
        if self.input_failed || self.surface_failed {
            return Ok(());
        }
        // A bounded batch between tiles, never an unbounded event-drain loop.
        for _ in 0..4 {
            let offer = match self.input.peek() {
                Ok(Peek::Empty) => {
                    self.input_responded = true;
                    break;
                }
                Ok(Peek::Offer(offer)) => {
                    self.input_responded = true;
                    offer
                }
                Err(error) => {
                    self.input_failed = true;
                    self.evidence.ready = false;
                    self.controller
                        .cancel_pointer()
                        .map_err(Error::Controller)?;
                    increment(&mut self.evidence.errors)?;
                    return Err(Error::Input(error));
                }
            };
            let mut envelope = InputEnvelope {
                epoch: offer.epoch(),
                sequence: offer.sequence(),
                generation: offer.generation(),
                event: offer.event(),
            };
            let result = match envelope.event {
                Event::Mouse { input, x, y } => {
                    increment(&mut self.evidence.mice)?;
                    let r = self.geometry.assigned;
                    if x >= r.x
                        && y >= r.y
                        && x - r.x < u32::from(r.width)
                        && y - r.y < u32::from(r.height)
                    {
                        envelope.event = Event::Mouse {
                            input,
                            x: x - r.x,
                            y: y - r.y,
                        };
                        self.controller.deliver(envelope)
                    } else {
                        // Do not turn an outside release into a clamped click.
                        // Existing physical button changed-bits survive reentry.
                        self.controller.cancel_pointer()
                    }
                }
                Event::Key(key) => {
                    if key.key == ui_core::PhysicalKey::F12
                        && key.action == ui_core::KeyAction::Press
                    {
                        increment(&mut self.evidence.f12_presses)?;
                    }
                    increment(&mut self.evidence.keys)?;
                    self.controller.deliver(envelope)
                }
                Event::Cancel(_) => self.controller.deliver(envelope),
            };
            self.operation(result)?;
            match self.input.ack(&offer) {
                Ok(AckOutcome::Acknowledged) => increment(&mut self.evidence.input_acks)?,
                Ok(AckOutcome::Superseded) => increment(&mut self.evidence.superseded)?,
                Err(error) => {
                    self.input_failed = true;
                    self.evidence.ready = false;
                    self.controller
                        .cancel_pointer()
                        .map_err(Error::Controller)?;
                    increment(&mut self.evidence.errors)?;
                    return Err(Error::Input(error));
                }
            }
        }
        Ok(())
    }
    fn save_step(&mut self) -> Result<(), Error> {
        if self.controller.document().is_saving() {
            match self.controller.advance_save() {
                Ok(true) => {
                    // Only save_next's validated COMMIT response reaches here.
                    increment(&mut self.evidence.save_acks)?;
                    if let Some(draft) = self.controller.document().draft() {
                        self.evidence.saved_revision = draft.loaded_revision;
                        self.evidence.saved_bytes = draft.contents.len() as u64;
                        self.evidence.saved_checksum =
                            volume_core::checksum_update(volume_core::FNV_OFFSET, draft.contents);
                    }
                }
                Ok(false) => {}
                Err(_) => increment(&mut self.evidence.errors)?,
            }
        }
        Ok(())
    }
    fn paint_step(&mut self) -> Result<(), Error> {
        if self.surface_failed || !self.controller.view().dirty {
            return Ok(());
        }
        // A completed old frame never certifies a partial current paint.
        self.evidence.ready = false;
        let mut view = self.controller.view();
        if view.generation != self.frame {
            self.frame = view.generation;
            self.next_tile = 0;
        }
        if self.input_failed && !self.controller.document().is_uncertain() {
            view.status = Status::InputLost;
            view.can_save = false;
        }
        let frame = Frame::new(view);
        let rectangle = self
            .geometry
            .rectangle(self.next_tile)
            .map_err(|_| Error::Binding)?;
        let mut pixels = [0; TILE_PIXELS];
        let rendered = frame
            .render_tile(rectangle.x, rectangle.y, &mut pixels)
            .map_err(|_| Error::Binding)?;
        if rendered.width != u32::from(rectangle.width)
            || rendered.height != u32::from(rectangle.height)
        {
            return Err(Error::Binding);
        }
        let mut bytes = [0; 1024];
        for (p, chunk) in pixels
            .iter()
            .zip(bytes[..rectangle.bytes()].chunks_exact_mut(4))
        {
            chunk.copy_from_slice(&p.to_le_bytes());
        }
        let result = self.surface.blit(
            self.frame,
            self.next_tile,
            rectangle,
            &bytes[..rectangle.bytes()],
        );
        if let Err(error) = result {
            self.surface_failed = true;
            self.evidence.ready = false;
            let _ = self.input.close();
            let _ = self.controller.cancel_pointer();
            increment(&mut self.evidence.errors)?;
            return Err(Error::Surface(error));
        }
        increment(&mut self.evidence.tiles)?;
        self.next_tile += 1;
        if self.next_tile == self.geometry.tiles() {
            if !self.controller.presented(self.frame, self.geometry.epoch) {
                return Err(Error::Binding);
            }
            increment(&mut self.evidence.frames)?;
            self.evidence.presented_generation = self.frame;
            self.evidence.ready = self.input_responded && !self.input_failed;
        }
        Ok(())
    }
    /// Each iteration performs <=4 PEEK/ACK pairs, one pending save exchange,
    /// one bounded tile transaction. Individual IPC polling is also bounded.
    /// Input/surface failure leaves document memory alive for explicit recovery.
    pub fn step(&mut self) -> Result<(), Error> {
        let input = self.input_step();
        self.save_step()?;
        let paint = self.paint_step();
        input.and(paint)
    }
    pub fn stop_clean(&mut self) -> Result<(), Error> {
        if self.controller.document().is_dirty() || self.controller.document().is_saving() {
            return Err(Error::Unsaved);
        }
        // Attempt all endpoint owners even if one cleanup reports an error.
        let document = self.controller.close_clean().map_err(Error::Controller);
        let input = self.input.close().map_err(Error::Cleanup);
        let surface = self.surface.close().map_err(Error::Cleanup);
        document.and(input).and(surface)
    }
}
