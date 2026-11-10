//! Interaction over the real streaming DocumentSession. InputEnvelope is an
//! adapter contract, not authentication: the native adapter must validate the
//! kernel envelope and INP1 reply before calling deliver().
use crate::{
    design,
    render::{
        Error as RenderError, Frame, Layout, SemanticActions, SemanticId, SemanticNode,
        MAX_SEMANTIC_NODES,
    },
    text::{self, VisualPosition},
    workspace::Workspace,
    Rect, Viewport,
};
use nieuw_os_catalog_client::Transport;
use nieuw_os_document_session::{
    volume::{
        Buffers, DocumentSession, Error as StorageError, ListEntry, ListFilter, SaveProgress,
        VolumeSession,
    },
    EditMode, SaveOutcome,
};
use nieuw_os_input_core::Event;
use ui_core::{KeyAction, KeyboardEvent, PhysicalKey};
use volume_core::PER_DOMAIN;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Screen {
    Material,
    Editor,
    ConfirmClose,
    Rename,
    Organize,
    History,
    Filter,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DocumentTool {
    #[default]
    Write,
    Read,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Focus {
    New,
    Refresh,
    Previous,
    Next,
    Object(usize),
    Back,
    Document,
    Save,
    CancelSave,
    KeepEditing,
    SaveAndBack,
    Discard,
    Search,
    Filter,
    Rename,
    Organize,
    History,
    FormInput,
    FormConfirm,
    FormCancel,
    CollectionSlot(usize),
    FilterSlot(usize),
    HistoryVersion(usize),
    RestoreHistory,
    Tool,
    ContextMenu,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Status {
    Ready,
    ReadOnly,
    Opened,
    Dirty,
    Saving { completed: usize, total: usize },
    Saved { revision: u64 },
    Cancelled,
    Renamed,
    Organized { revision: u64 },
    Restored { revision: u64 },
    RestoreRefreshFailed { revision: u64 },
    NeedsSave,
    PreviewUnavailable,
    CollectionSavedAssignmentFailed,
    InputLost,
    ClipboardCopied,
    ClipboardEmpty,
    ClipboardSelection,
    ClipboardTooLong,
    ClipboardUnavailable,
    Failed(Error),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Storage(StorageError),
    Cleanup(nieuw_os_sdk_runtime::Error),
    Accessibility(RenderError),
    InvalidUtf8,
    InvalidInput,
    StaleInput,
    InputEpoch,
    NoFreeObject,
    CounterExhausted,
}
#[derive(Clone, Copy, Debug)]
pub struct InputEnvelope {
    pub epoch: u64,
    pub sequence: u64,
    pub generation: u64,
    pub event: Event,
}

#[derive(Clone, Copy)]
pub struct View<'a> {
    pub draw_pointer: bool,
    pub high_contrast: bool,
    pub desktop: Option<&'a crate::desktop::Desktop>,
    pub viewport: Viewport,
    pub generation: u64,
    pub dirty: bool,
    pub screen: Screen,
    pub tool: DocumentTool,
    pub focus: Focus,
    pub status: Status,
    pub objects: &'a [ListEntry],
    pub workspace: &'a Workspace,
    pub list_loaded: bool,
    pub list_scroll: usize,
    pub contents: Option<&'a str>,
    pub title: &'a [u8],
    pub revision: u64,
    pub caret: usize,
    pub caret_trailing: bool,
    pub anchor: usize,
    pub scroll: usize,
    pub editable: bool,
    pub can_save: bool,
    pub can_inspect: bool,
    pub saving: bool,
    pub document_dirty: bool,
    pub cursor: Option<(u32, u32)>,
    pub document_menu: Option<DocumentMenu>,
}

/// App-local logical geometry; no input or storage authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DocumentMenu {
    pub bounds: Rect,
    pub selected: usize,
}
impl DocumentMenu {
    pub fn item(self, index: usize) -> Rect {
        Rect {
            x: self.bounds.x + 10,
            y: self.bounds.y + 10 + index.min(3) as u32 * 48,
            width: self.bounds.width - 20,
            height: 44,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PaintDamage(Option<Rect>);
impl PaintDamage {
    pub fn intersects(self, x: u32, y: u32, width: u32, height: u32) -> bool {
        self.0.is_some_and(|rect| {
            rect.intersects(Rect {
                x,
                y,
                width,
                height,
            })
        })
    }
    fn include(&mut self, rect: Rect) {
        self.0 = Some(match self.0 {
            None => rect,
            Some(old) => {
                let x = old.x.min(rect.x);
                let y = old.y.min(rect.y);
                Rect {
                    x,
                    y,
                    width: (old.x + old.width).max(rect.x + rect.width) - x,
                    height: (old.y + old.height).max(rect.y + rect.height) - y,
                }
            }
        });
    }
}

pub struct Controller<'a, T> {
    draw_pointer: bool,
    high_contrast: bool,
    desktop: Option<crate::desktop::Desktop>,
    document: DocumentSession<'a, T>,
    editable: bool,
    viewport: Viewport,
    screen: Screen,
    tool: DocumentTool,
    focus: Focus,
    status: Status,
    objects: [ListEntry; PER_DOMAIN],
    unfiltered: [ListEntry; PER_DOMAIN],
    unfiltered_count: usize,
    workspace: Workspace,
    object_count: usize,
    list_loaded: bool,
    list_scroll: usize,
    caret: usize,
    caret_trailing: bool,
    anchor: usize,
    scroll: usize,
    generation: u64,
    dirty: bool,
    paint_damage: PaintDamage,
    render_exhausted: bool,
    input_epoch: u64,
    awaiting_input: bool,
    last_input: u64,
    cancelled_through: u64,
    focus_generation: u64,
    cursor: Option<(u32, u32)>,
    dragging: bool,
    pressed: Option<Focus>,
    back_after_save: bool,
    document_menu: Option<DocumentMenu>,
    menu_pressed: Option<usize>,
}

impl<'a, T: Transport> Controller<'a, T> {
    pub fn new(
        session: VolumeSession<T>,
        mode: EditMode,
        buffers: Buffers<'a>,
        viewport: Viewport,
        input_epoch: u64,
    ) -> Result<Self, Error> {
        // Construct DocumentSession first: it owns cleanup on invalid buffers.
        let mut document = DocumentSession::new(session, mode, buffers).map_err(Error::Storage)?;
        if input_epoch == 0 {
            let _ = document.close();
            return Err(Error::InputEpoch);
        }
        Ok(Self {
            draw_pointer: true,
            high_contrast: false,
            desktop: None,
            document,
            editable: mode == EditMode::ReadWrite,
            viewport,
            screen: Screen::Material,
            tool: DocumentTool::Write,
            focus: if mode == EditMode::ReadWrite {
                Focus::New
            } else {
                Focus::Refresh
            },
            status: if mode == EditMode::ReadWrite {
                Status::Ready
            } else {
                Status::ReadOnly
            },
            objects: [ListEntry::EMPTY; PER_DOMAIN],
            unfiltered: [ListEntry::EMPTY; PER_DOMAIN],
            unfiltered_count: 0,
            workspace: Workspace::new(),
            object_count: 0,
            list_loaded: false,
            list_scroll: 0,
            caret: 0,
            caret_trailing: false,
            anchor: 0,
            scroll: 0,
            generation: 1,
            dirty: true,
            paint_damage: PaintDamage(Some(viewport.bounds())),
            render_exhausted: false,
            input_epoch,
            awaiting_input: false,
            last_input: 0,
            cancelled_through: 0,
            focus_generation: 0,
            cursor: None,
            dragging: false,
            pressed: None,
            back_after_save: false,
            document_menu: None,
            menu_pressed: None,
        })
    }

    fn invalidate(&mut self) -> Result<(), Error> {
        self.paint_damage = PaintDamage(Some(self.viewport.bounds()));
        self.invalidate_generation()
    }
    fn invalidate_generation(&mut self) -> Result<(), Error> {
        self.dirty = true;
        let Some(next) = self.generation.checked_add(1) else {
            self.render_exhausted = true;
            return Err(Error::CounterExhausted);
        };
        self.generation = next;
        Ok(())
    }
    fn failed(&mut self, error: Error) -> Error {
        if !self.document.is_saving() {
            let can_save = self.view().can_save;
            if self.focus == Focus::CancelSave
                || (!can_save && matches!(self.focus, Focus::Save | Focus::SaveAndBack))
            {
                self.focus = Focus::Document;
            } else if self.focus == Focus::SaveAndBack && self.screen != Screen::ConfirmClose {
                // SaveAndBack exists only in the close dialog; keep focus on a
                // control of the current screen so Enter/typing stay live.
                self.focus = Focus::Save;
            }
        }
        self.status = Status::Failed(if self.document.is_uncertain() {
            Error::Storage(StorageError::Uncertain)
        } else {
            error
        });
        // Counter exhaustion must never clear outstanding repaint work.
        let _ = self.invalidate();
        error
    }
    pub fn document(&self) -> &DocumentSession<'a, T> {
        &self.document
    }
    /// Explicit endpoint cleanup only after the caller has dealt with dirty
    /// work. No hidden commit, abort or discard occurs in this function/drop.
    pub fn close_clean(&mut self) -> Result<(), Error> {
        if self.dismiss_document_menu() {
            self.invalidate()?;
        }
        if self.document.is_saving() {
            return Err(self.failed(Error::Storage(StorageError::Busy)));
        }
        if self.document.is_dirty() {
            return Err(self.failed(Error::Storage(StorageError::UnsavedChanges)));
        }
        self.document
            .close()
            .map_err(|e| self.failed(Error::Cleanup(e)))
    }
    pub fn view(&self) -> View<'_> {
        let draft = self.document.draft();
        View {
            draw_pointer: self.draw_pointer,
            high_contrast: self.high_contrast,
            desktop: self.desktop.as_ref(),
            viewport: self.viewport,
            generation: self.generation,
            dirty: self.dirty,
            screen: self.screen,
            tool: self.tool,
            focus: self.focus,
            status: self.status,
            objects: &self.objects[..self.object_count],
            workspace: &self.workspace,
            list_loaded: self.list_loaded,
            list_scroll: self.list_scroll,
            contents: draft.and_then(|d| core::str::from_utf8(d.contents).ok()),
            title: draft.map_or(&[], |d| d.name),
            revision: draft.map_or(0, |d| d.loaded_revision),
            caret: self.caret,
            caret_trailing: self.caret_trailing,
            anchor: self.anchor,
            scroll: self.scroll,
            editable: self.editable
                && (self.screen == Screen::Material || self.tool == DocumentTool::Write),
            can_save: self.editable
                && (!matches!(
                    self.screen,
                    Screen::Editor | Screen::ConfirmClose | Screen::History
                ) || self.tool == DocumentTool::Write)
                && draft
                    .and_then(|d| core::str::from_utf8(d.contents).ok())
                    .is_some()
                && draft.is_some()
                && !self.document.is_closed()
                && !self.document.is_uncertain()
                && !self.document.is_conflicted(),
            can_inspect: draft.is_some()
                && !self.document.is_closed()
                && !self.document.is_uncertain()
                && !self.document.is_conflicted(),
            saving: self.document.is_saving(),
            document_dirty: self.document.is_dirty(),
            cursor: self.cursor,
            document_menu: self.document_menu,
        }
    }
    /// Presentation only: does not change input, document or storage authority.
    /// The caller owns authenticated settings routing; no key event is emitted.
    pub fn set_high_contrast(&mut self, enabled: bool) -> Result<(), Error> {
        if self.high_contrast != enabled {
            self.high_contrast = enabled;
            self.invalidate()?;
        }
        Ok(())
    }

    /// The validated native adapter enables this only when a separate compositor
    /// draws the system cursor. Local hover/input state is preserved.
    pub fn set_compositor_cursor(&mut self, enabled: bool) -> Result<(), Error> {
        let draw_pointer = !enabled;
        if self.draw_pointer != draw_pointer {
            self.draw_pointer = draw_pointer;
            self.invalidate()?;
        }
        Ok(())
    }
    pub fn presented(&mut self, generation: u64, display_epoch: u64) -> bool {
        if !self.render_exhausted
            && generation == self.generation
            && display_epoch == self.viewport.epoch()
        {
            self.dirty = false;
            self.paint_damage = PaintDamage::default();
            true
        } else {
            false
        }
    }
    pub fn paint_damage(&self) -> PaintDamage {
        self.paint_damage
    }
    /// Explicit public VM profile only. This does not revoke document caps or
    /// protect data; secure login requires an independent native session broker.
    pub fn enable_vm_desktop(&mut self) -> Result<(), Error> {
        if self.document.is_dirty() || self.document.is_saving() {
            return Err(Error::InvalidInput);
        }
        self.desktop = Some(crate::desktop::Desktop::new());
        self.dismiss_document_menu();
        self.invalidate()
    }
    fn desktop_action(&mut self, action: crate::desktop::Action) -> Result<(), Error> {
        if action == crate::desktop::Action::OpenMaterial {
            self.screen = Screen::Material;
            self.focus = if self.editable {
                Focus::New
            } else {
                Focus::Refresh
            };
            self.refresh()?;
        }
        self.invalidate()
    }
    pub fn resize(&mut self, viewport: Viewport) -> Result<(), Error> {
        self.dismiss_document_menu();
        self.viewport = viewport;
        if let Some(desktop) = &mut self.desktop {
            desktop.cancel_pointer();
        }
        self.dragging = false;
        self.pressed = None;
        self.cursor = self
            .cursor
            .filter(|(x, y)| *x < viewport.width() && *y < viewport.height());
        self.ensure_caret();
        self.invalidate()
    }
    /// The native adapter may call this only after independently obtaining and
    /// authenticating a fresh input-service epoch and fresh endpoint grants.
    /// This changes no capabilities and proves no device reset by itself. The
    /// document session/buffers, draft, selection and display epoch survive.
    /// Old PEEK/ACK work belongs to the old endpoint pair and must be closed by
    /// the adapter, never resent to the newly bound service.
    pub fn rebind_input(&mut self, new_epoch: u64) -> Result<(), Error> {
        if new_epoch == 0 || new_epoch == self.input_epoch {
            return Err(Error::InputEpoch);
        }
        self.dismiss_document_menu();
        if let Some(desktop) = &mut self.desktop {
            desktop.cancel_input();
        }
        self.input_epoch = new_epoch;
        self.last_input = 0;
        self.cancelled_through = 0;
        self.focus_generation = 0;
        self.dragging = false;
        self.pressed = None;
        self.cursor = None;
        self.awaiting_input = true;
        if !self.document.is_uncertain()
            && !self.document.is_conflicted()
            && !self.document.is_closed()
        {
            self.status = Status::InputLost;
        }
        self.invalidate()
    }
    /// Pointer left the assigned surface or its pointer stream was cancelled.
    /// This changes no keyboard focus, input epoch/generation, selection, draft
    /// or storage status. A held button on reentry cannot create a new press;
    /// the adapter passes only actual source changed/down bits, never clamped
    /// outside coordinates or a synthetic press.
    pub fn cancel_pointer(&mut self) -> Result<(), Error> {
        if let Some(desktop) = &mut self.desktop {
            desktop.cancel_pointer();
        }
        if !self.dragging
            && self.pressed.is_none()
            && self.cursor.is_none()
            && self.menu_pressed.is_none()
        {
            return Ok(());
        }
        self.menu_pressed = None;
        self.dragging = false;
        self.pressed = None;
        self.cursor = None;
        self.invalidate()
    }
    pub fn refresh(&mut self) -> Result<(), Error> {
        let mut objects = [ListEntry::EMPTY; PER_DOMAIN];
        match self
            .document
            .list_filtered(self.workspace.filter, &mut objects)
        {
            Ok(count) => {
                self.unfiltered = objects;
                self.unfiltered_count = count;
                self.filter_query();
                self.list_loaded = true;
                self.list_scroll = self.list_scroll.min(count.saturating_sub(1));
                self.invalidate()
            }
            Err(error) => {
                self.list_loaded = false;
                Err(self.failed(Error::Storage(error)))
            }
        }
    }
    pub fn open(&mut self, object: u64) -> Result<(), Error> {
        self.dismiss_document_menu();
        if let Err(error) = self.document.open(object) {
            return Err(self.failed(Error::Storage(error)));
        }
        self.workspace.metadata = Some(
            self.document
                .metadata()
                .map_err(|e| self.failed(Error::Storage(e)))?,
        );
        self.screen = Screen::Editor;
        self.tool = DocumentTool::Write;
        self.focus = Focus::Document;
        self.caret = 0;
        self.caret_trailing = false;
        self.anchor = 0;
        self.scroll = 0;
        self.back_after_save = false;
        if self.view().contents.is_none() {
            return Err(self.failed(Error::InvalidUtf8));
        }
        self.status = Status::Opened;
        self.invalidate()
    }
    pub fn create(&mut self) -> Result<(), Error> {
        self.dismiss_document_menu();
        // Allocation considers the complete granted domain, never a filtered view.
        let mut all = [ListEntry::EMPTY; PER_DOMAIN];
        let count = self
            .document
            .list(&mut all)
            .map_err(|e| self.failed(Error::Storage(e)))?;
        let Some(object) = (0..PER_DOMAIN).find(|index| {
            !all[..count]
                .iter()
                .any(|entry| entry.object as usize == *index)
        }) else {
            return Err(self.failed(Error::NoFreeObject));
        };
        if let Err(error) = self.document.create(object as u64, b"Nieuw document") {
            return Err(self.failed(Error::Storage(error)));
        }
        self.screen = Screen::Editor;
        self.tool = DocumentTool::Write;
        self.focus = Focus::Document;
        self.caret = 0;
        self.caret_trailing = false;
        self.anchor = 0;
        self.scroll = 0;
        self.back_after_save = false;
        self.status = Status::Dirty;
        self.workspace.metadata = None;
        self.invalidate()
    }
    fn filter_query(&mut self) {
        self.objects.fill(ListEntry::EMPTY);
        self.object_count = 0;
        for entry in &self.unfiltered[..self.unfiltered_count] {
            if self.workspace.matches(&entry.name[..entry.name_length]) {
                self.objects[self.object_count] = *entry;
                self.object_count += 1;
            }
        }
        self.list_scroll = self.list_scroll.min(self.object_count.saturating_sub(1));
    }
    pub fn set_search_query(&mut self, value: &str) -> Result<(), Error> {
        self.dismiss_document_menu();
        self.pressed = None;
        self.dragging = false;
        if self.screen != Screen::Material
            || value.len() > crate::workspace::QUERY
            || value.chars().any(char::is_control)
        {
            return Err(Error::InvalidInput);
        }
        self.workspace.query.fill(0);
        self.workspace.query[..value.len()].copy_from_slice(value.as_bytes());
        self.workspace.query_len = value.len();
        self.filter_query();
        self.focus = Focus::Search;
        self.invalidate()
    }
    fn load_collections(&mut self) -> Result<(), Error> {
        for i in 0..volume_core::COLLECTIONS {
            self.workspace.collections[i] = self
                .document
                .collection_stat(i + 1)
                .map_err(|e| self.failed(Error::Storage(e)))?;
        }
        Ok(())
    }
    pub fn set_form_value(&mut self, value: &str) -> Result<(), Error> {
        self.dismiss_document_menu();
        self.pressed = None;
        self.dragging = false;
        if !matches!(self.screen, Screen::Rename | Screen::Organize)
            || !self.workspace.set_form(value)
        {
            return Err(Error::InvalidInput);
        }
        self.workspace.form_selected = false;
        self.focus = Focus::FormInput;
        self.invalidate()
    }
    pub fn begin_rename(&mut self) -> Result<(), Error> {
        self.dismiss_document_menu();
        self.pressed = None;
        self.dragging = false;
        if self.screen != Screen::Editor || !self.view().can_save || self.document.is_saving() {
            return Err(Error::InvalidInput);
        }
        let draft = self.document.draft().ok_or(Error::InvalidInput)?;
        self.workspace.form.fill(0);
        self.workspace.form[..draft.name.len()].copy_from_slice(draft.name);
        self.workspace.form_len = draft.name.len();
        self.workspace.form_selected = true;
        self.screen = Screen::Rename;
        self.focus = Focus::FormInput;
        self.invalidate()
    }
    fn clean_operation(&mut self, write: bool) -> Result<bool, Error> {
        if self.screen != Screen::Editor
            || !self.view().can_inspect
            || (write && !self.view().editable)
            || self.document.is_saving()
        {
            return Err(Error::InvalidInput);
        }
        if self.document.is_dirty() {
            self.status = Status::NeedsSave;
            self.invalidate()?;
            return Ok(false);
        }
        self.workspace.metadata = Some(
            self.document
                .metadata()
                .map_err(|e| self.failed(Error::Storage(e)))?,
        );
        Ok(true)
    }
    pub fn begin_organize(&mut self) -> Result<(), Error> {
        self.dismiss_document_menu();
        self.pressed = None;
        self.dragging = false;
        if !self.clean_operation(true)? {
            return Ok(());
        }
        self.load_collections()?;
        let current = self
            .workspace
            .metadata
            .map_or(0, |s| usize::from(s.collection));
        self.screen = Screen::Organize;
        self.select_collection(current.saturating_sub(1).min(volume_core::COLLECTIONS - 1))
    }
    pub fn select_collection(&mut self, index: usize) -> Result<(), Error> {
        self.pressed = None;
        self.dragging = false;
        if self.screen != Screen::Organize || index >= volume_core::COLLECTIONS {
            return Err(Error::InvalidInput);
        }
        self.workspace.selected_collection = index;
        let entry = self.workspace.collections[index];
        self.workspace.form.fill(0);
        self.workspace.form[..entry.name_length].copy_from_slice(entry.name());
        self.workspace.form_len = entry.name_length;
        self.workspace.form_selected = true;
        self.focus = Focus::FormInput;
        self.invalidate()
    }
    pub fn confirm_form(&mut self) -> Result<(), Error> {
        self.pressed = None;
        self.dragging = false;
        match self.screen {
            Screen::Rename => {
                self.document
                    .rename_draft(&self.workspace.form[..self.workspace.form_len])
                    .map_err(|e| self.failed(Error::Storage(e)))?;
                self.status = Status::Renamed;
            }
            Screen::Organize => {
                let stat = self.workspace.metadata.ok_or(Error::InvalidInput)?;
                let index = self.workspace.selected_collection;
                let collection = self.workspace.collections[index];
                let form = self.workspace.form;
                let name = &form[..self.workspace.form_len];
                let mut collection_changed = false;
                if name.is_empty() {
                    return Err(Error::InvalidInput);
                }
                if !collection.present || collection.name() != name {
                    let result =
                        self.document
                            .update_collection(index + 1, collection.revision, name);
                    let revision = result.map_err(|e| self.failed(Error::Storage(e)))?;
                    self.workspace.collections[index].revision = revision;
                    self.workspace.collections[index].present = true;
                    self.workspace.collections[index].name.fill(0);
                    self.workspace.collections[index].name[..name.len()].copy_from_slice(name);
                    self.workspace.collections[index].name_length = name.len();
                    collection_changed = true;
                }
                let revision = match self.document.set_metadata(
                    stat.revision,
                    stat.tags,
                    stat.kind,
                    (index + 1) as u8,
                ) {
                    Ok(revision) => revision,
                    Err(e) => {
                        let error = self.failed(Error::Storage(e));
                        if collection_changed {
                            self.status = Status::CollectionSavedAssignmentFailed;
                        }
                        return Err(error);
                    }
                };
                let mut current = stat;
                current.revision = revision;
                current.collection = (index + 1) as u8;
                self.workspace.metadata = Some(current);
                self.status = Status::Organized { revision };
            }
            _ => return Err(Error::InvalidInput),
        }
        self.screen = Screen::Editor;
        self.focus = Focus::Document;
        self.workspace.form.fill(0);
        self.workspace.form_len = 0;
        self.pressed = None;
        self.dragging = false;
        self.invalidate()
    }
    pub fn begin_history(&mut self) -> Result<(), Error> {
        self.dismiss_document_menu();
        self.pressed = None;
        self.dragging = false;
        if !self.clean_operation(false)? {
            return Ok(());
        }
        self.screen = Screen::History;
        self.select_history(1)
    }
    pub fn select_history(&mut self, selector: usize) -> Result<(), Error> {
        self.pressed = None;
        self.dragging = false;
        if self.screen != Screen::History || selector == 0 || selector > volume_core::HISTORY {
            return Err(Error::InvalidInput);
        }
        let stat = self.workspace.metadata.ok_or(Error::InvalidInput)?;
        self.workspace.clear_preview();
        self.workspace.selected_history = selector;
        self.focus = Focus::HistoryVersion(selector);
        if selector > stat.history_count {
            self.status = Status::PreviewUnavailable;
            self.focus = Focus::FormCancel;
            return self.invalidate();
        }
        let result =
            self.document
                .read_history_prefix(stat.revision, selector, &mut self.workspace.preview);
        let prefix = result.map_err(|e| self.failed(Error::Storage(e)))?;
        self.workspace.preview_len = prefix.copied;
        self.workspace.preview_total = prefix.total;
        self.workspace.preview_complete = true; // verified prefix, explicitly labelled when shortened
        if !self.workspace.preview_complete {
            self.status = Status::PreviewUnavailable;
        }
        self.invalidate()
    }
    pub fn restore_selected_history(&mut self) -> Result<(), Error> {
        self.pressed = None;
        self.dragging = false;
        // The read tool may inspect a verified historical preview, but it
        // never acquires write authority through a direct restore call.
        if self.tool == DocumentTool::Read
            || self.screen != Screen::History
            || !self.workspace.preview_complete
        {
            return Err(Error::InvalidInput);
        }
        let stat = self.workspace.metadata.ok_or(Error::InvalidInput)?;
        let result = self
            .document
            .restore_history(stat.revision, self.workspace.selected_history);
        match result {
            Ok(revision) => self.status = Status::Restored { revision },
            Err(StorageError::CommittedRefreshFailed { revision }) => {
                self.status = Status::RestoreRefreshFailed { revision };
                self.workspace.metadata = None;
                self.screen = Screen::Editor;
                self.focus = Focus::Document;
                self.workspace.clear_preview();
                return self.invalidate();
            }
            Err(e) => return Err(self.failed(Error::Storage(e))),
        }
        self.workspace.metadata = None;
        if let Some(draft) = self.document.draft() {
            let revision = draft.loaded_revision;
            match self.document.metadata() {
                Ok(stat) if stat.revision == revision => self.workspace.metadata = Some(stat),
                Ok(_) => {}
                Err(_) => self.status = Status::RestoreRefreshFailed { revision },
            }
        }
        self.workspace.clear_preview();
        self.caret = 0;
        self.caret_trailing = false;
        self.anchor = 0;
        self.scroll = 0;
        self.screen = Screen::Editor;
        self.focus = Focus::Document;
        self.invalidate()
    }
    fn cancel_workspace(&mut self) -> Result<(), Error> {
        self.pressed = None;
        self.dragging = false;
        self.screen = if self.screen == Screen::Filter {
            Screen::Material
        } else {
            Screen::Editor
        };
        self.focus = if self.screen == Screen::Material {
            Focus::Search
        } else {
            Focus::Document
        };
        self.workspace.form.fill(0);
        self.workspace.form_len = 0;
        self.workspace.clear_preview();
        self.invalidate()
    }
    pub fn choose_filter(&mut self, slot: usize) -> Result<(), Error> {
        self.dismiss_document_menu();
        self.pressed = None;
        self.dragging = false;
        if self.screen != Screen::Filter {
            return Err(Error::InvalidInput);
        }
        self.workspace.filter = match slot {
            0 => ListFilter::All,
            1 => ListFilter::Uncollected,
            2.. if slot - 2 < volume_core::COLLECTIONS
                && self.workspace.collections[slot - 2].present =>
            {
                ListFilter::Collection((slot - 1) as u8)
            }
            _ => return Err(Error::InvalidInput),
        };
        self.screen = Screen::Material;
        self.focus = Focus::Filter;
        self.refresh()
    }
    pub fn workspace_controls(&self, mut visit: impl FnMut(Focus, crate::Rect, bool)) {
        let l = Layout::for_screen(self.viewport, self.screen);
        match self.screen {
            Screen::Rename | Screen::Organize => {
                visit(Focus::FormInput, l.form_input, true);
                visit(Focus::FormConfirm, l.form_confirm, self.view().can_save);
                if self.screen == Screen::Organize {
                    for i in 0..volume_core::COLLECTIONS {
                        visit(Focus::CollectionSlot(i), l.collection_slot(i), true);
                    }
                }
            }
            Screen::History => {
                if let Some(stat) = self.workspace.metadata {
                    for i in 1..=stat.history_count {
                        visit(Focus::HistoryVersion(i), l.history_slot(i), true);
                    }
                }
                visit(
                    Focus::RestoreHistory,
                    l.form_confirm,
                    self.workspace.preview_complete && self.view().can_save,
                );
            }
            Screen::Filter => {
                for i in 0..volume_core::COLLECTIONS + 2 {
                    if i < 2 || self.workspace.collections[i - 2].present {
                        visit(Focus::FilterSlot(i), l.filter_slot(i), true);
                    }
                }
            }
            _ => return,
        }
        visit(Focus::FormCancel, l.form_cancel, true);
    }
    fn semantic_actions(
        &self,
        id: SemanticId,
        expected_generation: u64,
    ) -> Result<(bool, SemanticActions), Error> {
        if expected_generation != self.generation {
            return Err(Error::StaleInput);
        }
        let mut nodes = [SemanticNode::EMPTY; MAX_SEMANTIC_NODES];
        let count = Frame::new(self.view())
            .semantics(&mut nodes)
            .map_err(Error::Accessibility)?;
        nodes[..count]
            .iter()
            .find(|node| node.id == id)
            .map(|node| (node.enabled, node.actions))
            .ok_or(Error::InvalidInput)
    }

    /// Activate an action advertised by this controller's current semantic
    /// tree. This stays within the owning app and grants no external access.
    pub fn activate_semantic(
        &mut self,
        id: SemanticId,
        expected_generation: u64,
    ) -> Result<(), Error> {
        let (enabled, actions) = self.semantic_actions(id, expected_generation)?;
        if !enabled || !actions.activate {
            return Err(Error::InvalidInput);
        }
        if self.document_menu.is_some() {
            let index = match id {
                SemanticId::SelectAll => 0,
                SemanticId::DocumentTool => 1,
                SemanticId::Save => 2,
                SemanticId::CancelDocumentMenu => 3,
                _ => return Err(Error::InvalidInput),
            };
            return self.activate_menu_item(index);
        }
        if let SemanticId::Desktop(target) = id {
            let desktop = self.desktop.as_mut().ok_or(Error::InvalidInput)?;
            let action = desktop.activate(target);
            return self.desktop_action(action);
        }
        let focus = match id {
            SemanticId::NewDocument => Focus::New,
            SemanticId::Refresh => Focus::Refresh,
            SemanticId::PreviousPage => Focus::Previous,
            SemanticId::NextPage => Focus::Next,
            SemanticId::Object(object) => Focus::Object(
                self.objects[..self.object_count]
                    .iter()
                    .position(|entry| entry.object == object)
                    .ok_or(Error::InvalidInput)?,
            ),
            SemanticId::Back => Focus::Back,
            SemanticId::Save if self.document.is_saving() => Focus::CancelSave,
            SemanticId::Save => Focus::Save,
            SemanticId::SaveAndBack => Focus::SaveAndBack,
            SemanticId::KeepEditing => Focus::KeepEditing,
            SemanticId::Discard => Focus::Discard,
            SemanticId::LibrarySearch => Focus::Search,
            SemanticId::CollectionFilter => Focus::Filter,
            SemanticId::Rename => Focus::Rename,
            SemanticId::Organize => Focus::Organize,
            SemanticId::History => Focus::History,
            SemanticId::FormInput => Focus::FormInput,
            SemanticId::FormConfirm => Focus::FormConfirm,
            SemanticId::FormCancel => Focus::FormCancel,
            SemanticId::CollectionSlot(i) => Focus::CollectionSlot(i),
            SemanticId::FilterSlot(i) => Focus::FilterSlot(i),
            SemanticId::HistoryVersion(i) => Focus::HistoryVersion(i),
            SemanticId::RestoreHistory => Focus::RestoreHistory,
            SemanticId::DocumentTool => Focus::Tool,
            SemanticId::DocumentMenu => Focus::ContextMenu,
            _ => return Err(Error::InvalidInput),
        };
        // An accepted semantic action may replace the screen or object list;
        // a pending pointer press/drag is bound to the old context.
        self.dragging = false;
        self.pressed = None;
        self.focus = focus;
        self.activate(focus)
    }

    fn activate_focus_semantically(&mut self, focus: Focus) -> Result<(), Error> {
        let id = match focus {
            Focus::New => SemanticId::NewDocument,
            Focus::Refresh => SemanticId::Refresh,
            Focus::Previous => SemanticId::PreviousPage,
            Focus::Next => SemanticId::NextPage,
            Focus::Object(index) => SemanticId::Object(
                self.objects
                    .get(index)
                    .filter(|_| index < self.object_count)
                    .map(|entry| entry.object)
                    .ok_or(Error::InvalidInput)?,
            ),
            Focus::Back => SemanticId::Back,
            Focus::Save | Focus::CancelSave => SemanticId::Save,
            Focus::KeepEditing => SemanticId::KeepEditing,
            Focus::SaveAndBack => SemanticId::SaveAndBack,
            Focus::Discard => SemanticId::Discard,
            Focus::Search => SemanticId::LibrarySearch,
            Focus::Filter => SemanticId::CollectionFilter,
            Focus::Rename => SemanticId::Rename,
            Focus::Organize => SemanticId::Organize,
            Focus::History => SemanticId::History,
            Focus::FormInput => SemanticId::FormInput,
            Focus::FormConfirm => SemanticId::FormConfirm,
            Focus::FormCancel => SemanticId::FormCancel,
            Focus::CollectionSlot(i) => SemanticId::CollectionSlot(i),
            Focus::FilterSlot(i) => SemanticId::FilterSlot(i),
            Focus::HistoryVersion(i) => SemanticId::HistoryVersion(i),
            Focus::RestoreHistory => SemanticId::RestoreHistory,
            Focus::Tool => SemanticId::DocumentTool,
            Focus::ContextMenu => SemanticId::DocumentMenu,
            Focus::Document => return Err(Error::InvalidInput),
        };
        self.activate_semantic(id, self.generation)
    }

    /// Replace the current text-editor value only when the live semantic tree
    /// advertises that action. Persistent storage still requires an explicit save.
    pub fn set_semantic_value(
        &mut self,
        id: SemanticId,
        expected_generation: u64,
        value: &str,
    ) -> Result<(), Error> {
        let (enabled, actions) = self.semantic_actions(id, expected_generation)?;
        if let SemanticId::Desktop(target) = id {
            if !enabled || !actions.set_value {
                return Err(Error::InvalidInput);
            }
            let desktop = self.desktop.as_mut().ok_or(Error::InvalidInput)?;
            let accepted = match target {
                crate::desktop::Target::SearchField => desktop.set_query(value),
                crate::desktop::Target::Password => desktop.set_password(value),
                _ => false,
            };
            if !accepted {
                return Err(Error::InvalidInput);
            }
            return self.invalidate();
        }
        if enabled && actions.set_value {
            match id {
                SemanticId::LibrarySearch => return self.set_search_query(value),
                SemanticId::FormInput => return self.set_form_value(value),
                _ => {}
            }
        }
        if id != SemanticId::Document || !enabled || !actions.set_value {
            return Err(Error::InvalidInput);
        }
        // Replaced text invalidates any pending pointer press/drag selection.
        self.dragging = false;
        self.pressed = None;
        let current_length = self.view().contents.ok_or(Error::InvalidUtf8)?.len();
        self.document
            .replace_range(0..current_length, value.as_bytes())
            .map_err(|error| self.failed(Error::Storage(error)))?;
        self.focus = Focus::Document;
        self.caret = value.len();
        self.caret_trailing = false;
        self.anchor = self.caret;
        self.update_edit_status();
        self.ensure_caret();
        self.invalidate()
    }

    /// Both tools borrow the same document owner; switching never saves,
    /// reopens, copies, or changes the document's storage authority.
    pub fn switch_tool(&mut self) -> Result<(), Error> {
        if self.screen != Screen::Editor || self.document.is_saving() {
            return Err(Error::InvalidInput);
        }
        self.dismiss_document_menu();
        self.tool = match self.tool {
            DocumentTool::Write => DocumentTool::Read,
            DocumentTool::Read => DocumentTool::Write,
        };
        self.dragging = false;
        self.pressed = None;
        self.focus = Focus::Document;
        self.invalidate()
    }

    pub fn save(&mut self) -> Result<(), Error> {
        // The read tool never acquires write authority, including direct callers.
        if self.tool == DocumentTool::Read {
            return Err(Error::InvalidInput);
        }
        if self.dismiss_document_menu() {
            self.invalidate()?;
        }
        if !self.document.is_dirty() {
            // Keep the SDK's available/read-only/uncertainty checks even for
            // unchanged bytes; an old loaded revision is not a new success.
            let outcome = self
                .document
                .save()
                .map_err(|e| self.failed(Error::Storage(e)))?;
            let revision = match outcome {
                SaveOutcome::Unchanged { revision } | SaveOutcome::Saved { revision } => revision,
            };
            self.status = Status::Saved { revision };
            return self.invalidate();
        }
        if let Err(error) = self.document.begin_save() {
            return Err(self.failed(Error::Storage(error)));
        }
        self.status = Status::Saving {
            completed: 0,
            total: self.document.draft().map_or(0, |d| d.contents.len()),
        };
        self.focus = Focus::CancelSave;
        self.invalidate()
    }
    /// Performs at most one real APPEND/COMMIT exchange. The adapter can
    /// service input/render and invoke cancel between successive calls.
    pub fn advance_save(&mut self) -> Result<bool, Error> {
        match self.document.save_next() {
            Ok(SaveProgress::Writing { completed, total }) => {
                self.status = Status::Saving { completed, total };
                self.invalidate()?;
                Ok(false)
            }
            Ok(SaveProgress::Saved { revision }) => {
                self.status = Status::Saved { revision };
                self.focus = Focus::Document;
                if self.back_after_save {
                    self.back_after_save = false;
                    self.screen = Screen::Material;
                    self.focus = Focus::New;
                    self.refresh()?;
                }
                self.invalidate()?;
                Ok(true)
            }
            Err(error) => {
                self.back_after_save = false;
                Err(self.failed(Error::Storage(error)))
            }
        }
    }
    pub fn cancel_save(&mut self) -> Result<(), Error> {
        if let Err(error) = self.document.cancel_save() {
            return Err(self.failed(Error::Storage(error)));
        }
        self.back_after_save = false;
        self.status = Status::Cancelled;
        self.focus = Focus::Document;
        self.invalidate()
    }
    pub fn request_back(&mut self) -> Result<(), Error> {
        self.dismiss_document_menu();
        if self.document.is_saving() {
            return Err(self.failed(Error::Storage(StorageError::Busy)));
        }
        if self.document.is_dirty() {
            self.screen = Screen::ConfirmClose;
            self.focus = Focus::KeepEditing;
        } else {
            self.screen = Screen::Material;
            // Same initial focus as the constructor: New is disabled read-only.
            self.focus = if self.editable {
                Focus::New
            } else {
                Focus::Refresh
            };
            self.refresh()?;
        }
        self.dragging = false;
        self.pressed = None;
        self.invalidate()
    }
    fn activate(&mut self, focus: Focus) -> Result<(), Error> {
        match focus {
            Focus::Search => {
                self.focus = Focus::Search;
                self.invalidate()
            }
            Focus::Filter => {
                self.load_collections()?;
                self.screen = Screen::Filter;
                self.focus = Focus::FilterSlot(0);
                self.invalidate()
            }
            Focus::Rename => self.begin_rename(),
            Focus::Organize => self.begin_organize(),
            Focus::History => self.begin_history(),
            Focus::FormInput => {
                self.focus = Focus::FormInput;
                self.invalidate()
            }
            Focus::FormConfirm => self.confirm_form(),
            Focus::FormCancel => self.cancel_workspace(),
            Focus::CollectionSlot(i) => self.select_collection(i),
            Focus::FilterSlot(i) => self.choose_filter(i),
            Focus::HistoryVersion(i) => self.select_history(i),
            Focus::RestoreHistory => self.restore_selected_history(),
            Focus::New => self.create(),
            Focus::Refresh => self.refresh(),
            Focus::Previous | Focus::Next => {
                let rows = Layout::new(self.viewport).list_rows();
                self.list_scroll = if focus == Focus::Previous {
                    self.list_scroll.saturating_sub(rows)
                } else {
                    self.list_scroll
                        .saturating_add(rows)
                        .min(self.object_count.saturating_sub(1))
                };
                self.focus = if self.object_count > 0 {
                    Focus::Object(self.list_scroll)
                } else {
                    Focus::New
                };
                self.invalidate()
            }
            Focus::Object(index) => self
                .objects
                .get(index)
                .filter(|_| index < self.object_count)
                .map(|e| e.object)
                .ok_or(Error::InvalidInput)
                .and_then(|id| self.open(id)),
            Focus::Back => self.request_back(),
            Focus::Document => Ok(()),
            Focus::Save => self.save(),
            Focus::CancelSave => self.cancel_save(),
            Focus::KeepEditing => {
                self.screen = Screen::Editor;
                self.focus = Focus::Document;
                self.invalidate()
            }
            Focus::SaveAndBack => {
                if !self.view().can_save {
                    return Err(Error::InvalidInput);
                }
                self.screen = Screen::Editor;
                self.save()?;
                self.back_after_save = true;
                Ok(())
            }
            Focus::Discard => {
                self.document
                    .discard_changes()
                    .map_err(|e| self.failed(Error::Storage(e)))?;
                self.screen = Screen::Material;
                self.focus = Focus::New;
                self.status = Status::Ready;
                self.refresh()
            }
            Focus::Tool => self.switch_tool(),
            Focus::ContextMenu => {
                let button = Layout::new(self.viewport).context_menu;
                self.open_document_menu(Some((button.x, button.y + button.height)))
            }
        }
    }
    fn dismiss_document_menu(&mut self) -> bool {
        self.menu_pressed = None;
        self.document_menu.take().is_some()
    }
    fn open_document_menu(&mut self, pointer: Option<(u32, u32)>) -> Result<(), Error> {
        if self.screen != Screen::Editor
            || self.document.is_saving()
            || self.document.is_closed()
            || self.view().contents.is_none()
            || self.desktop.as_ref().is_some_and(|d| d.overlay())
        {
            return Ok(());
        }
        let layout = Layout::for_screen(self.viewport, self.screen);
        let (x, y) = pointer.unwrap_or_else(|| {
            let at = layout.text_metrics().position_with_affinity(
                self.view().contents.unwrap_or(""),
                self.caret,
                self.caret_trailing,
            );
            (
                layout.text_left() + (at.x_64 / 64) as u32 / self.viewport.scale().factor(),
                layout.text_top()
                    + at.row.saturating_sub(self.scroll).min(layout.rows()) as u32
                        * design::LINE_HEIGHT
                    + design::LINE_HEIGHT,
            )
        });
        self.document_menu = Some(DocumentMenu {
            bounds: Rect {
                x: x.clamp(8, self.viewport.logical_width() - 256),
                y: y.clamp(8, self.viewport.logical_height() - 216),
                width: 248,
                height: 208,
            },
            selected: 0,
        });
        self.dragging = false;
        self.pressed = None;
        self.menu_pressed = None;
        self.invalidate()
    }
    fn activate_menu_item(&mut self, index: usize) -> Result<(), Error> {
        if self.document_menu.is_none() || index > 3 || (index == 2 && !self.view().can_save) {
            return Err(Error::InvalidInput);
        }
        self.dismiss_document_menu();
        self.dragging = false;
        self.pressed = None;
        self.invalidate()?;
        match index {
            0 => {
                self.anchor = 0;
                self.caret = self.view().contents.map_or(0, str::len);
                self.focus = Focus::Document;
                self.ensure_caret();
                Ok(())
            }
            1 => self.switch_tool(),
            2 => self.save(),
            _ => Ok(()),
        }
    }
    fn menu_key(&mut self, event: KeyboardEvent) -> Result<(), Error> {
        self.menu_pressed = None;
        match event.key {
            PhysicalKey::Escape if event.action == KeyAction::Press => {
                self.dismiss_document_menu();
                self.invalidate()
            }
            PhysicalKey::Enter | PhysicalKey::KeypadEnter if event.action == KeyAction::Press => {
                self.activate_menu_item(self.document_menu.unwrap().selected)
            }
            PhysicalKey::Up
            | PhysicalKey::Down
            | PhysicalKey::Left
            | PhysicalKey::Right
            | PhysicalKey::Tab => {
                let reverse = matches!(event.key, PhysicalKey::Up | PhysicalKey::Left)
                    || (event.key == PhysicalKey::Tab && event.modifiers.shift());
                let can_save = self.view().can_save;
                let menu = self.document_menu.as_mut().unwrap();
                loop {
                    menu.selected = (menu.selected + if reverse { 3 } else { 1 }) % 4;
                    if menu.selected != 2 || can_save {
                        break;
                    }
                }
                self.invalidate()
            }
            _ => Ok(()),
        }
    }
    fn ensure_caret(&mut self) {
        let Some(text) = self
            .document
            .draft()
            .and_then(|d| core::str::from_utf8(d.contents).ok())
        else {
            return;
        };
        self.caret = text::boundary(text, self.caret);
        self.anchor = text::boundary(text, self.anchor);
        let layout = Layout::for_screen(self.viewport, self.screen);
        let row = layout
            .text_metrics()
            .position_with_affinity(text, self.caret, self.caret_trailing)
            .row;
        if row < self.scroll {
            self.scroll = row;
        }
        if row >= self.scroll + layout.rows() {
            self.scroll = row + 1 - layout.rows();
        }
    }
    pub fn insert(&mut self, replacement: &str) -> Result<(), Error> {
        if self.tool == DocumentTool::Read || self.document_menu.is_some() {
            return Err(Error::InvalidInput);
        }
        if self.screen != Screen::Editor || self.view().contents.is_none() {
            return Err(self.failed(Error::InvalidUtf8));
        }
        let range = text::selection(self.anchor, self.caret);
        let start = range.start;
        let layout = Layout::for_screen(self.viewport, self.screen);
        let metrics = layout.text_metrics();
        let old = self.view();
        let contents = old.contents.ok_or(Error::InvalidUtf8)?;
        let first_row = metrics.position_with_affinity(contents, start, true).row;
        let last_row = metrics.position(contents, contents.len()).row;
        let previous = (old.status, old.can_save, old.document_dirty, old.scroll);
        self.document
            .replace_range(range, replacement.as_bytes())
            .map_err(|e| self.failed(Error::Storage(e)))?;
        self.caret = start + replacement.len();
        self.caret_trailing = false;
        self.anchor = self.caret;
        self.update_edit_status();
        self.ensure_caret();
        let view = self.view();
        if previous != (view.status, view.can_save, view.document_dirty, view.scroll)
            || self
                .desktop
                .as_ref()
                .is_some_and(|desktop| desktop.overlay())
        {
            return self.invalidate();
        }
        let contents = view.contents.ok_or(Error::InvalidUtf8)?;
        let first_row = first_row.min(metrics.position_with_affinity(contents, start, true).row);
        let last_row = last_row.max(metrics.position(contents, contents.len()).row);
        let top = first_row.saturating_sub(self.scroll).min(layout.rows());
        let bottom = last_row
            .saturating_add(1)
            .saturating_sub(self.scroll)
            .min(layout.rows());
        // Keep the union until the whole current frame has been acknowledged.
        // Superseding a partially painted frame must not retain its stale rows.
        if bottom > top {
            self.paint_damage.include(
                Rect {
                    x: layout.editor.x,
                    y: layout.text_top() + top as u32 * design::LINE_HEIGHT,
                    width: layout.editor.width,
                    height: (bottom - top) as u32 * design::LINE_HEIGHT,
                }
                .physical(self.viewport.scale()),
            );
        }
        self.invalidate_generation()
    }
    fn clipboard_edit_context(&self) -> bool {
        self.screen == Screen::Editor
            && self.document_menu.is_none()
            && self.focus == Focus::Document
            && !self
                .desktop
                .as_ref()
                .is_some_and(|desktop| desktop.overlay())
            && !self.document.is_saving()
            && !self.document.is_closed()
            && !self.document.is_uncertain()
            && !self.document.is_conflicted()
    }
    /// Returns only the current explicit selection. The native input broker
    /// separately authenticates the actual copy gesture and active window.
    pub fn clipboard_selection(&self) -> Option<&str> {
        if !self.clipboard_edit_context() {
            return None;
        }
        let range = text::selection(self.anchor, self.caret);
        if range.is_empty() {
            return None;
        }
        self.view().contents?.get(range)
    }
    /// Called only after clipboard delivery and the original input ACK.
    /// DocumentSession performs its existing atomic size/rights validation.
    pub fn paste_clipboard(&mut self, value: &str) -> Result<(), Error> {
        if !self.clipboard_edit_context() || !self.view().editable || value.is_empty() {
            return Ok(());
        }
        self.insert(value)
    }
    pub fn clipboard_feedback(&mut self, status: Status) -> Result<(), Error> {
        if !matches!(
            status,
            Status::ClipboardCopied
                | Status::ClipboardEmpty
                | Status::ClipboardSelection
                | Status::ClipboardTooLong
                | Status::ClipboardUnavailable
        ) || self.document_menu.is_some()
            || self.document.is_saving()
            || self.document.is_closed()
            || self.document.is_uncertain()
            || self.document.is_conflicted()
        {
            return Ok(());
        }
        self.status = status;
        self.invalidate()
    }
    fn update_edit_status(&mut self) {
        self.status = if self.document.is_uncertain() {
            Status::Failed(Error::Storage(StorageError::Uncertain))
        } else if self.document.is_conflicted() {
            Status::Failed(Error::Storage(StorageError::Conflict))
        } else if self.document.is_closed() {
            Status::Failed(Error::Storage(StorageError::Closed))
        } else if self.document.is_dirty() {
            Status::Dirty
        } else {
            Status::Opened
        };
    }
    fn move_caret(&mut self, next: usize, extend: bool) -> Result<(), Error> {
        self.caret = next;
        if !extend {
            self.anchor = self.caret;
        }
        self.ensure_caret();
        self.invalidate()
    }
    fn tab(&mut self, reverse: bool) -> Result<(), Error> {
        let mut choices = [Focus::New; PER_DOMAIN + 12];
        let count = match self.screen {
            Screen::Material => {
                let mut prefix = if self.editable {
                    choices[0] = Focus::New;
                    choices[1] = Focus::Refresh;
                    2
                } else {
                    choices[0] = Focus::Refresh;
                    1
                };
                if self.list_scroll > 0 {
                    choices[prefix] = Focus::Previous;
                    prefix += 1;
                }
                if self.list_scroll + Layout::new(self.viewport).list_rows() < self.object_count {
                    choices[prefix] = Focus::Next;
                    prefix += 1;
                }
                for (i, choice) in choices[prefix..]
                    .iter_mut()
                    .take(self.object_count)
                    .enumerate()
                {
                    *choice = Focus::Object(i);
                }
                let end = self.object_count + prefix;
                choices[end] = Focus::Search;
                choices[end + 1] = Focus::Filter;
                end + 2
            }
            Screen::Editor => {
                choices[0] = Focus::Back;
                choices[1] = Focus::Document;
                choices[2] = if self.document.is_saving() {
                    Focus::CancelSave
                } else {
                    Focus::Save
                };
                let count = if self.view().can_save || self.document.is_saving() {
                    if self.viewport.logical_width() >= 700 {
                        choices[3] = Focus::Rename;
                        choices[4] = Focus::Organize;
                        choices[5] = Focus::History;
                        6
                    } else {
                        3
                    }
                } else if self.viewport.logical_width() >= 700 && self.view().can_inspect {
                    choices[2] = Focus::History;
                    3
                } else {
                    2
                };
                if self.document.is_saving() {
                    count
                } else {
                    choices[count] = Focus::Tool;
                    choices[count + 1] = Focus::ContextMenu;
                    count + 2
                }
            }
            Screen::Rename | Screen::Organize => {
                choices[0] = Focus::FormInput;
                let mut count = 1;
                if !self.workspace.form().is_empty() && self.view().can_save {
                    choices[count] = Focus::FormConfirm;
                    count += 1;
                }
                choices[count] = Focus::FormCancel;
                count += 1;
                if self.screen == Screen::Organize {
                    for i in 0..volume_core::COLLECTIONS {
                        choices[count] = Focus::CollectionSlot(i);
                        count += 1;
                    }
                }
                count
            }
            Screen::Filter => {
                let mut count = 0;
                for i in 0..volume_core::COLLECTIONS + 2 {
                    if i < 2 || self.workspace.collections[i - 2].present {
                        choices[count] = Focus::FilterSlot(i);
                        count += 1;
                    }
                }
                choices[count] = Focus::FormCancel;
                count + 1
            }
            Screen::History => {
                choices[0] = Focus::HistoryVersion(self.workspace.selected_history);
                let mut count = 1;
                if self.workspace.preview_complete && self.view().can_save {
                    choices[count] = Focus::RestoreHistory;
                    count += 1;
                }
                choices[count] = Focus::FormCancel;
                count += 1;
                if self.workspace.metadata.is_some_and(|s| s.history_count > 1) {
                    choices[count] =
                        Focus::HistoryVersion(if self.workspace.selected_history == 1 {
                            2
                        } else {
                            1
                        });
                    count += 1;
                }
                count
            }
            Screen::ConfirmClose => {
                if self.view().can_save {
                    choices[0] = Focus::SaveAndBack;
                    choices[1] = Focus::KeepEditing;
                    choices[2] = Focus::Discard;
                    3
                } else {
                    choices[0] = Focus::KeepEditing;
                    choices[1] = Focus::Discard;
                    2
                }
            }
        };
        let at = choices[..count]
            .iter()
            .position(|f| *f == self.focus)
            .unwrap_or(0);
        self.focus = choices[if reverse {
            (at + count - 1) % count
        } else {
            (at + 1) % count
        }];
        if let Focus::Object(index) = self.focus {
            let rows = Layout::new(self.viewport).list_rows();
            if index < self.list_scroll {
                self.list_scroll = index;
            }
            if index >= self.list_scroll + rows {
                self.list_scroll = index + 1 - rows;
            }
        }
        self.invalidate()
    }
    fn edit_workspace_field(&mut self, event: KeyboardEvent, query: bool) -> Result<(), Error> {
        if event.modifiers.control() && event.key == PhysicalKey::A {
            if query {
                return self.set_search_query("");
            }
            self.workspace.form_selected = true;
            return self.invalidate();
        }
        if event.modifiers.control() || event.modifiers.alt() || event.modifiers.super_key() {
            return Ok(());
        }
        let mut buffer = [0; crate::workspace::QUERY];
        let value = if query {
            self.workspace.query()
        } else {
            self.workspace.form()
        };
        buffer[..value.len()].copy_from_slice(value.as_bytes());
        let mut length = value.len();
        if !query && self.workspace.form_selected {
            length = 0;
        }
        if event.key == PhysicalKey::Backspace {
            let text = core::str::from_utf8(&buffer[..length]).map_err(|_| Error::InvalidUtf8)?;
            length = text::previous(text, length);
        } else if let Some(byte) = us_character(
            event.key,
            event.modifiers.shift(),
            event.modifiers.caps_lock,
        ) {
            let limit = if query {
                crate::workspace::QUERY
            } else {
                volume_core::NAME
            };
            if length >= limit {
                return Ok(());
            }
            buffer[length] = byte;
            length += 1;
        } else {
            return Ok(());
        }
        let value = core::str::from_utf8(&buffer[..length]).map_err(|_| Error::InvalidUtf8)?;
        if query {
            self.set_search_query(value)
        } else {
            self.set_form_value(value)
        }
    }
    fn key(&mut self, event: KeyboardEvent) -> Result<(), Error> {
        if !matches!(event.action, KeyAction::Press | KeyAction::Repeat) {
            return Ok(());
        }
        let press = event.action == KeyAction::Press;
        if self.document_menu.is_some() {
            return self.menu_key(event);
        }
        if event.key == PhysicalKey::Menu
            || (event.key == PhysicalKey::F10 && event.modifiers.shift())
        {
            return if press {
                self.open_document_menu(None)
            } else {
                Ok(())
            };
        }
        if event.key == PhysicalKey::F4 && self.screen == Screen::Editor {
            return if press && !self.document.is_saving() {
                self.activate_focus_semantically(Focus::Tool)
            } else {
                Ok(())
            };
        }
        if self.screen == Screen::Material {
            if event.key == PhysicalKey::PageDown
                && self.list_scroll + Layout::new(self.viewport).list_rows() < self.object_count
            {
                return self.activate_focus_semantically(Focus::Next);
            }
            if event.key == PhysicalKey::PageUp && self.list_scroll > 0 {
                return self.activate_focus_semantically(Focus::Previous);
            }
        }
        if press
            && event.modifiers.control()
            && event.key == PhysicalKey::F
            && self.screen == Screen::Material
        {
            self.focus = Focus::Search;
            return self.invalidate();
        }
        if press && self.screen == Screen::Editor {
            if event.key == PhysicalKey::F2 {
                return self.begin_rename();
            }
            if event.modifiers.control() && event.key == PhysicalKey::G {
                return self.begin_organize();
            }
            // Ctrl+Alt+H is the host's trusted high-contrast toggle, not History.
            if event.modifiers.alt() && !event.modifiers.control() && event.key == PhysicalKey::H {
                return self.begin_history();
            }
        }
        if event.key == PhysicalKey::Escape
            && press
            && matches!(
                self.screen,
                Screen::Rename | Screen::Organize | Screen::History | Screen::Filter
            )
        {
            return self.cancel_workspace();
        }
        if self.screen == Screen::History
            && matches!(event.key, PhysicalKey::PageUp | PhysicalKey::PageDown)
        {
            if let Some(value) = self.workspace.preview() {
                let l = Layout::for_screen(self.viewport, self.screen);
                let scale = self.viewport.scale();
                let font = nieuw_os_graphics::Font::new(
                    nieuw_os_graphics::Size::Body,
                    nieuw_os_graphics::Weight::Regular,
                    scale == crate::Scale::Percent200,
                );
                let metrics =
                    text::Proportional::new(font, (l.preview.width - 24) * scale.factor());
                let total = metrics.position(value, value.len()).row + 1;
                let rows =
                    (l.preview.height.saturating_sub(8) / design::LINE_HEIGHT).max(1) as usize;
                self.workspace.preview_scroll = if event.key == PhysicalKey::PageUp {
                    self.workspace.preview_scroll.saturating_sub(rows)
                } else {
                    self.workspace
                        .preview_scroll
                        .saturating_add(rows)
                        .min(total.saturating_sub(rows))
                };
                self.pressed = None;
                self.dragging = false;
                return self.invalidate();
            }
        }
        if event.key == PhysicalKey::Tab && press {
            return self.tab(event.modifiers.shift());
        }
        if event.key == PhysicalKey::Escape && press {
            return match self.screen {
                Screen::ConfirmClose => self.activate_focus_semantically(Focus::KeepEditing),
                Screen::Editor => self.activate_focus_semantically(Focus::Back),
                _ => Ok(()),
            };
        }
        if event.modifiers.control() && press && self.screen == Screen::Editor {
            if event.key == PhysicalKey::S {
                return if self.view().can_save {
                    self.save()
                } else {
                    Ok(())
                };
            }
            if event.key == PhysicalKey::A && self.focus == Focus::Document {
                self.anchor = 0;
                self.caret = self.view().contents.map_or(0, str::len);
                self.ensure_caret();
                return self.invalidate();
            }
        }
        if self.screen == Screen::Material && self.focus == Focus::Search {
            if matches!(event.key, PhysicalKey::Enter | PhysicalKey::KeypadEnter) {
                if let Some(entry) = self.objects.first().filter(|_| self.object_count > 0) {
                    return self.open(entry.object);
                }
                return self.invalidate();
            }
            return self.edit_workspace_field(event, true);
        }
        if matches!(self.screen, Screen::Rename | Screen::Organize)
            && self.focus == Focus::FormInput
        {
            if matches!(event.key, PhysicalKey::Enter | PhysicalKey::KeypadEnter) {
                return self.confirm_form();
            }
            if self.screen == Screen::Organize
                && matches!(event.key, PhysicalKey::Up | PhysicalKey::Down)
            {
                let i = self.workspace.selected_collection;
                return self.select_collection(if event.key == PhysicalKey::Up {
                    (i + volume_core::COLLECTIONS - 1) % volume_core::COLLECTIONS
                } else {
                    (i + 1) % volume_core::COLLECTIONS
                });
            }
            return self.edit_workspace_field(event, false);
        }
        if self.focus != Focus::Document || self.screen != Screen::Editor {
            if press
                && matches!(
                    event.key,
                    PhysicalKey::Enter | PhysicalKey::KeypadEnter | PhysicalKey::Space
                )
            {
                return self.activate_focus_semantically(self.focus);
            }
            return Ok(());
        }
        if self.document.is_saving() {
            return Ok(());
        }
        let Some(contents) = self.view().contents else {
            return Err(self.failed(Error::InvalidUtf8));
        };
        let selected = text::selection(self.anchor, self.caret);
        let metrics = Layout::for_screen(self.viewport, self.screen).text_metrics();
        let current = metrics.position_with_affinity(contents, self.caret, self.caret_trailing);
        let mut target_row = current.row;
        let next = match event.key {
            PhysicalKey::Left => Some(if !event.modifiers.shift() && !selected.is_empty() {
                selected.start
            } else {
                text::previous(contents, self.caret)
            }),
            PhysicalKey::Right => Some(if !event.modifiers.shift() && !selected.is_empty() {
                selected.end
            } else {
                text::next(contents, self.caret)
            }),
            PhysicalKey::Home => Some(metrics.byte_at(
                contents,
                VisualPosition {
                    row: current.row,
                    x_64: 0,
                },
            )),
            PhysicalKey::End => Some(metrics.byte_at(
                contents,
                VisualPosition {
                    row: current.row,
                    x_64: u64::MAX,
                },
            )),
            PhysicalKey::Up | PhysicalKey::Down => {
                target_row = if event.key == PhysicalKey::Up {
                    current.row.saturating_sub(1)
                } else {
                    current.row + 1
                };
                Some(metrics.byte_at(
                    contents,
                    VisualPosition {
                        row: target_row,
                        x_64: current.x_64,
                    },
                ))
            }
            _ => None,
        };
        if let Some(next) = next {
            self.caret_trailing = event.key == PhysicalKey::End
                || (matches!(event.key, PhysicalKey::Up | PhysicalKey::Down)
                    && metrics.position(contents, next).row > target_row);
            return self.move_caret(next, event.modifiers.shift());
        }
        if event.modifiers.control() || event.modifiers.alt() || event.modifiers.super_key() {
            return Ok(());
        }
        if self.tool == DocumentTool::Read {
            return Ok(());
        }
        match event.key {
            PhysicalKey::Backspace => {
                if selected.is_empty() {
                    self.anchor = text::previous(contents, self.caret);
                }
                self.insert("")
            }
            PhysicalKey::Delete => {
                if selected.is_empty() {
                    self.anchor = text::next(contents, self.caret);
                }
                self.insert("")
            }
            PhysicalKey::Enter | PhysicalKey::KeypadEnter => self.insert("\n"),
            key => {
                if let Some(byte) =
                    us_character(key, event.modifiers.shift(), event.modifiers.caps_lock)
                {
                    let bytes = [byte];
                    let character =
                        core::str::from_utf8(&bytes).map_err(|_| Error::InvalidInput)?;
                    self.insert(character)
                } else {
                    Ok(())
                }
            }
        }
    }
    fn hit(&self, x: u32, y: u32) -> Option<Focus> {
        let layout = Layout::for_screen(self.viewport, self.screen);
        if matches!(
            self.screen,
            Screen::Rename | Screen::Organize | Screen::History | Screen::Filter
        ) {
            let mut hit = None;
            self.workspace_controls(|focus, bounds, enabled| {
                if enabled && bounds.contains(x, y) {
                    hit = Some(focus);
                }
            });
            return hit;
        }
        if self.screen == Screen::Material {
            if layout.search.contains(x, y) {
                return Some(Focus::Search);
            }
            if layout.filter.contains(x, y) {
                return Some(Focus::Filter);
            }
        }
        if self.screen == Screen::Editor && self.viewport.logical_width() >= 700 {
            for (bounds, focus) in [
                (layout.rename, Focus::Rename),
                (layout.organize, Focus::Organize),
                (layout.history, Focus::History),
            ] {
                if bounds.contains(x, y) {
                    return Some(focus);
                }
            }
        }
        match self.screen {
            Screen::Material => {
                if self.editable && layout.new.contains(x, y) {
                    return Some(Focus::New);
                }
                if layout.refresh.contains(x, y) {
                    return Some(Focus::Refresh);
                }
                if self.list_scroll > 0 && layout.list_previous.contains(x, y) {
                    return Some(Focus::Previous);
                }
                if self.list_scroll + layout.list_rows() < self.object_count
                    && layout.list_next.contains(x, y)
                {
                    return Some(Focus::Next);
                }
                for index in self.list_scroll..self.object_count {
                    let rect = layout.object(index - self.list_scroll);
                    if rect.y + rect.height <= layout.list_bottom && rect.contains(x, y) {
                        return Some(Focus::Object(index));
                    }
                }
                None
            }
            Screen::Editor => {
                if layout.back.contains(x, y) {
                    Some(Focus::Back)
                } else if layout.tool.contains(x, y) && !self.document.is_saving() {
                    Some(Focus::Tool)
                } else if layout.context_menu.contains(x, y) && !self.document.is_saving() {
                    Some(Focus::ContextMenu)
                } else if layout.save.contains(x, y)
                    && (self.view().can_save || self.document.is_saving())
                {
                    Some(if self.document.is_saving() {
                        Focus::CancelSave
                    } else {
                        Focus::Save
                    })
                } else if layout.editor.contains(x, y) {
                    Some(Focus::Document)
                } else {
                    None
                }
            }
            Screen::Rename | Screen::Organize | Screen::History | Screen::Filter => None,
            Screen::ConfirmClose => [
                (layout.modal_save, Focus::SaveAndBack),
                (layout.modal_cancel, Focus::KeepEditing),
                (layout.modal_discard, Focus::Discard),
            ]
            .into_iter()
            .find_map(|(rect, focus)| {
                (rect.contains(x, y) && (focus != Focus::SaveAndBack || self.view().can_save))
                    .then_some(focus)
            }),
        }
    }
    fn point_caret(&mut self, x: u32, y: u32, extend: bool) {
        if let Some(contents) = self.view().contents {
            let layout = Layout::for_screen(self.viewport, self.screen);
            let x_64 = u64::from(x.saturating_sub(layout.text_left()))
                * u64::from(self.viewport.scale().factor())
                * 64;
            let row = y.saturating_sub(layout.text_top()) as usize / design::LINE_HEIGHT as usize
                + self.scroll;
            let next = layout
                .text_metrics()
                .byte_at(contents, VisualPosition { row, x_64 });
            let trailing = layout.text_metrics().position(contents, next).row > row;
            self.caret = next;
            self.caret_trailing = trailing;
            if !extend {
                self.anchor = self.caret;
            }
            self.ensure_caret();
        }
    }
    pub fn deliver(&mut self, envelope: InputEnvelope) -> Result<(), Error> {
        if envelope.epoch != self.input_epoch {
            return Err(Error::InputEpoch);
        }
        if envelope.sequence == 0 || envelope.generation == 0 {
            return Err(Error::InvalidInput);
        }
        if let Event::Mouse { x, y, .. } = envelope.event {
            if x >= self.viewport.width() || y >= self.viewport.height() {
                return Err(Error::InvalidInput);
            }
        }
        if envelope.sequence <= self.last_input {
            return Err(Error::StaleInput);
        }
        self.last_input = envelope.sequence;
        if let Event::Cancel(_) = envelope.event {
            self.cancelled_through = self.cancelled_through.max(envelope.generation);
            if envelope.generation >= self.focus_generation {
                if let Some(desktop) = &mut self.desktop {
                    desktop.cancel_input();
                }
                self.dragging = false;
                self.pressed = None;
                self.cursor = None;
                self.dismiss_document_menu();
                if !self.document.is_uncertain()
                    && !self.document.is_conflicted()
                    && !self.document.is_closed()
                {
                    self.status = Status::InputLost;
                }
                self.invalidate()?;
            }
            return Ok(());
        }
        if envelope.generation <= self.cancelled_through
            || envelope.generation < self.focus_generation
        {
            return Err(Error::StaleInput);
        }
        if envelope.generation != self.focus_generation {
            if let Some(desktop) = &mut self.desktop {
                desktop.cancel_input();
            }
            self.dragging = false;
            self.pressed = None;
            if self.dismiss_document_menu() {
                self.invalidate()?;
            }
            self.focus_generation = envelope.generation;
        }
        if self.awaiting_input && envelope.event != Event::PointerLeave {
            self.awaiting_input = false;
            if !self.document.is_uncertain()
                && !self.document.is_conflicted()
                && !self.document.is_closed()
            {
                self.status = if self.document.is_dirty() {
                    Status::Dirty
                } else if self.document.draft().is_some() {
                    Status::Opened
                } else if self.editable {
                    Status::Ready
                } else {
                    Status::ReadOnly
                };
            }
            self.invalidate()?;
        }
        match envelope.event {
            Event::PointerLeave => self.cancel_pointer(),
            event if self.desktop.as_ref().is_some_and(|d| d.overlay()) => {
                self.dismiss_document_menu();
                if let Event::Mouse { x, y, .. } = event {
                    self.cursor = Some((x, y));
                }
                let action = self.desktop.as_mut().unwrap().deliver(event, self.viewport);
                self.desktop_action(action)
            }
            Event::Key(key)
                if key.key == PhysicalKey::Escape
                    && key.action == KeyAction::Press
                    && self.screen == Screen::Material
                    && self.desktop.is_some() =>
            {
                self.desktop.as_mut().unwrap().home();
                self.invalidate()
            }
            Event::Key(key) => self.key(key),
            Event::Mouse { input, x, y } => {
                self.cursor = Some((x, y));
                let x = x / self.viewport.scale().factor();
                let y = y / self.viewport.scale().factor();
                if let Some(menu) = self.document_menu {
                    let hit = (0..4).find(|i| menu.item(*i).contains(x, y));
                    if input.changed.left() && input.buttons.left() {
                        self.menu_pressed = hit.filter(|i| *i != 2 || self.view().can_save);
                        if let Some(index) = self.menu_pressed {
                            self.document_menu.as_mut().unwrap().selected = index;
                        }
                        if !menu.bounds.contains(x, y) {
                            self.dismiss_document_menu();
                        }
                    } else if input.changed.right() && input.buttons.right() {
                        self.menu_pressed = None;
                        if !menu.bounds.contains(x, y) {
                            self.dismiss_document_menu();
                        }
                    }
                    if input.changed.left() && !input.buttons.left() {
                        if let Some(index) = self.menu_pressed.take().filter(|i| Some(*i) == hit) {
                            self.activate_menu_item(index)?;
                        }
                    }
                    return self.invalidate();
                }
                if input.changed.right()
                    && input.buttons.right()
                    && Layout::new(self.viewport).editor.contains(x, y)
                {
                    return self.open_document_menu(Some((x, y)));
                }
                if input.changed.left() && input.buttons.left() {
                    self.pressed = self.hit(x, y);
                    if let Some(focus) = self.pressed {
                        self.focus = focus;
                    }
                    if self.pressed == Some(Focus::Document) {
                        self.point_caret(x, y, false);
                        self.dragging = true;
                    }
                } else if self.dragging && input.buttons.left() {
                    self.point_caret(x, y, true);
                }
                if input.changed.left() && !input.buttons.left() {
                    self.dragging = false;
                    let pressed = self.pressed.take();
                    if pressed.is_some() && pressed == self.hit(x, y) {
                        self.activate(pressed.unwrap())?;
                    }
                }
                self.invalidate()
            }
            Event::Cancel(_) => Ok(()),
        }
    }
}

pub(crate) fn us_character(key: PhysicalKey, shift: bool, caps: bool) -> Option<u8> {
    use PhysicalKey::*;
    let letter = match key {
        A => b'a',
        B => b'b',
        C => b'c',
        D => b'd',
        E => b'e',
        F => b'f',
        G => b'g',
        H => b'h',
        I => b'i',
        J => b'j',
        K => b'k',
        L => b'l',
        M => b'm',
        N => b'n',
        O => b'o',
        P => b'p',
        Q => b'q',
        R => b'r',
        S => b's',
        T => b't',
        U => b'u',
        V => b'v',
        W => b'w',
        X => b'x',
        Y => b'y',
        Z => b'z',
        _ => 0,
    };
    if letter != 0 {
        return Some(if shift ^ caps {
            letter.to_ascii_uppercase()
        } else {
            letter
        });
    }
    let (plain, shifted) = match key {
        Digit1 => (b'1', b'!'),
        Digit2 => (b'2', b'@'),
        Digit3 => (b'3', b'#'),
        Digit4 => (b'4', b'$'),
        Digit5 => (b'5', b'%'),
        Digit6 => (b'6', b'^'),
        Digit7 => (b'7', b'&'),
        Digit8 => (b'8', b'*'),
        Digit9 => (b'9', b'('),
        Digit0 => (b'0', b')'),
        Minus => (b'-', b'_'),
        Equal => (b'=', b'+'),
        LeftBracket => (b'[', b'{'),
        RightBracket => (b']', b'}'),
        Backslash => (b'\\', b'|'),
        Semicolon => (b';', b':'),
        Quote => (b'\'', b'"'),
        Backquote => (b'`', b'~'),
        Comma => (b',', b'<'),
        Period => (b'.', b'>'),
        Slash => (b'/', b'?'),
        Space => (b' ', b' '),
        _ => return None,
    };
    Some(if shift { shifted } else { plain })
}
