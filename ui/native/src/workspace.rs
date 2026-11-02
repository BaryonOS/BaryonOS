//! Bounded presentation state for real volume objects; never an authority.
use nieuw_os_document_session::volume::{CollectionInfo, ListFilter, Stat};
use volume_core::{COLLECTIONS, NAME};

pub const QUERY: usize = 64;
pub const PREVIEW: usize = 2048;

pub struct Workspace {
    pub(crate) query: [u8; QUERY],
    pub(crate) query_len: usize,
    pub(crate) form: [u8; NAME],
    pub(crate) form_len: usize,
    pub(crate) form_selected: bool,
    pub(crate) collections: [CollectionInfo; COLLECTIONS],
    pub(crate) metadata: Option<Stat>,
    pub(crate) filter: ListFilter,
    pub(crate) selected_collection: usize,
    pub(crate) selected_history: usize,
    pub(crate) preview: [u8; PREVIEW],
    pub(crate) preview_len: usize,
    pub(crate) preview_scroll: usize,
    pub(crate) preview_complete: bool,
    pub(crate) preview_total: usize,
}
impl Default for Workspace {
    fn default() -> Self {
        Self::new()
    }
}
impl Workspace {
    pub const fn new() -> Self {
        Self {
            query: [0; QUERY],
            query_len: 0,
            form: [0; NAME],
            form_len: 0,
            form_selected: false,
            collections: [CollectionInfo {
                revision: 0,
                request_id: 0,
                present: false,
                name_length: 0,
                name: [0; NAME],
            }; COLLECTIONS],
            metadata: None,
            filter: ListFilter::All,
            selected_collection: 0,
            selected_history: 1,
            preview: [0; PREVIEW],
            preview_len: 0,
            preview_scroll: 0,
            preview_complete: false,
            preview_total: 0,
        }
    }
    pub fn query(&self) -> &str {
        core::str::from_utf8(&self.query[..self.query_len]).unwrap_or("")
    }
    pub fn form(&self) -> &str {
        core::str::from_utf8(&self.form[..self.form_len]).unwrap_or("")
    }
    pub fn form_selected(&self) -> bool {
        self.form_selected
    }
    pub fn metadata(&self) -> Option<Stat> {
        self.metadata
    }
    pub fn collections(&self) -> &[CollectionInfo; COLLECTIONS] {
        &self.collections
    }
    pub fn filter(&self) -> ListFilter {
        self.filter
    }
    pub fn selected_collection(&self) -> usize {
        self.selected_collection
    }
    pub fn selected_history(&self) -> usize {
        self.selected_history
    }
    pub fn preview_total(&self) -> usize {
        self.preview_total
    }
    pub fn preview_truncated(&self) -> bool {
        self.preview_len < self.preview_total
    }
    pub fn preview_scroll(&self) -> usize {
        self.preview_scroll
    }
    pub fn preview(&self) -> Option<&str> {
        self.preview_complete
            .then(|| core::str::from_utf8(&self.preview[..self.preview_len]).ok())
            .flatten()
    }
    pub(crate) fn clear_preview(&mut self) {
        self.preview.fill(0);
        self.preview_len = 0;
        self.preview_scroll = 0;
        self.preview_complete = false;
        self.preview_total = 0;
    }
    pub(crate) fn set_form(&mut self, value: &str) -> bool {
        if value.len() > NAME || value.chars().any(char::is_control) {
            return false;
        }
        self.form.fill(0);
        self.form[..value.len()].copy_from_slice(value.as_bytes());
        self.form_len = value.len();
        true
    }
    pub(crate) fn matches(&self, name: &[u8]) -> bool {
        self.query_len == 0
            || name
                .windows(self.query_len)
                .any(|part| part.eq_ignore_ascii_case(&self.query[..self.query_len]))
    }
}
