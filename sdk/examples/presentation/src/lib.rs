//! Plain-text slide selection and real rendering through shared native graphics.
//! No file, surface protocol, input endpoint, launch or resource authority.
#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;

use alloc::vec::Vec;
use nieuw_os_graphics::{theme::MATERIAAL, Canvas, Font, Rect, Size, Weight};

pub const MAX_INPUT: usize = 32 * 1024;
pub const MAX_SLIDES: usize = 32;
pub const WIDTH: usize = 640;
pub const HEIGHT: usize = 360;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InputLimit,
    UnsupportedControl,
    EmptySlide,
    SlideLimit,
    ContentLimit,
    OutOfMemory,
    Selection,
    Scale,
    Buffer,
    TextOverflow,
}

#[derive(Clone, Copy, Debug)]
pub struct Slide<'a> {
    pub title: &'a str,
    pub body: &'a str,
}

pub struct Deck<'a> {
    source: &'a str,
    slides: Vec<Slide<'a>>,
    selected: usize,
}

impl<'a> Deck<'a> {
    /// Strict LF text: first line is title; subsequent lines are body.
    /// Slides are separated by the exact line "\n---\n". Not Markdown/PPTX.
    pub fn parse(source: &'a str) -> Result<Self, Error> {
        if source.len() > MAX_INPUT {
            return Err(Error::InputLimit);
        }
        if source.chars().any(|ch| ch.is_control() && ch != '\n') {
            return Err(Error::UnsupportedControl);
        }
        let mut slides = Vec::new();
        slides
            .try_reserve_exact(MAX_SLIDES)
            .map_err(|_| Error::OutOfMemory)?;
        for part in source.split("\n---\n") {
            if slides.len() == MAX_SLIDES {
                return Err(Error::SlideLimit);
            }
            let (title, body) = part.split_once('\n').unwrap_or((part, ""));
            if title.trim().is_empty() || title == "---" {
                return Err(Error::EmptySlide);
            }
            if title.len() > 256
                || body.lines().count() > 12
                || body.lines().any(|line| line.len() > 512 || line == "---")
            {
                return Err(Error::ContentLimit);
            }
            slides.push(Slide { title, body });
        }
        Ok(Self {
            source,
            slides,
            selected: 0,
        })
    }

    pub const fn source(&self) -> &'a str {
        self.source
    }
    pub fn count(&self) -> usize {
        self.slides.len()
    }
    pub const fn selected(&self) -> usize {
        self.selected
    }
    pub fn slide(&self) -> Slide<'a> {
        self.slides[self.selected]
    }
    pub fn select(&mut self, index: usize) -> Result<(), Error> {
        if index >= self.slides.len() {
            return Err(Error::Selection);
        }
        self.selected = index;
        Ok(())
    }
    pub fn next_slide(&mut self) -> bool {
        if self.selected + 1 == self.slides.len() {
            return false;
        }
        self.selected += 1;
        true
    }
    pub fn previous_slide(&mut self) -> bool {
        if self.selected == 0 {
            return false;
        }
        self.selected -= 1;
        true
    }

    /// Preflight the complete selected text and destination before any write.
    /// Unsupported glyphs use the existing visible atlas fallback, not deletion.
    pub fn render(&self, pixels: &mut [u32], scale: usize) -> Result<(), Error> {
        if scale != 1 && scale != 2 {
            return Err(Error::Scale);
        }
        let width = WIDTH * scale;
        let height = HEIGHT * scale;
        if pixels.len() < width * height {
            return Err(Error::Buffer);
        }
        let title_font = Font::new(Size::Title, Weight::Semibold, scale == 2);
        let body_font = Font::new(Size::Body, Weight::Regular, scale == 2);
        let footer_font = Font::new(Size::Small, Weight::Regular, scale == 2);
        let slide = self.slide();
        let margin = 32 * scale;
        let available = (width - margin * 2) as u32;
        let line_height = body_font.height() as usize + 8 * scale;
        let body_top = 108 * scale;
        let lines = slide.body.lines().count();
        if title_font.measure(slide.title) > available
            || title_font.height() as usize > 64 * scale
            || slide
                .body
                .lines()
                .any(|line| body_font.measure(line) > available)
            || (lines > 0
                && body_top + (lines - 1) * line_height + body_font.height() as usize
                    > height - 48 * scale)
        {
            return Err(Error::TextOverflow);
        }
        let mut canvas = Canvas::new(Rect::new(0, 0, width as u32, height as u32), pixels)
            .map_err(|_| Error::Buffer)?;
        canvas.clear(MATERIAAL.surface);
        canvas.text(
            margin as i32,
            (24 * scale) as i32,
            slide.title,
            title_font,
            MATERIAAL.text,
        );
        canvas.fill_rect(
            Rect::new(
                margin as i32,
                (88 * scale) as i32,
                (48 * scale) as u32,
                (3 * scale) as u32,
            ),
            MATERIAAL.accent,
        );
        for (row, line) in slide.body.lines().enumerate() {
            canvas.text(
                margin as i32,
                (body_top + row * line_height) as i32,
                line,
                body_font,
                MATERIAAL.text,
            );
        }
        let (page, count) = ((self.selected + 1) as u8, self.slides.len() as u8);
        let counter = [
            b'0' + page / 10,
            b'0' + page % 10,
            b'/',
            b'0' + count / 10,
            b'0' + count % 10,
        ];
        let text = core::str::from_utf8(&counter).expect("ASCII page counter");
        canvas.text(
            (width - margin - footer_font.measure(text) as usize) as i32,
            (height - 28 * scale) as i32,
            text,
            footer_font,
            MATERIAAL.muted,
        );
        Ok(())
    }
}
