//! S09 user export (approved format, host-tested): turn a format-v1 archive
//! into a POSIX ustar stream for common tar readers outside Baryon OS. Spec:
//! `docs/implementation/S09-BACKUP-ARCHIVE.md`, "User export to ustar".
//!
//! Per object, in slot order, one regular file `<collection>/<name>` or
//! `<name>` holding its bytes as stored. A path that is not ASCII is preceded
//! by a pax `path` record (UTF-8). No directory entries: empty collections
//! are not exported. Lossy by design: kind, tags, revisions, times and slots
//! are dropped, so this is an export for other systems, not a recovery copy,
//! and nothing restores from it. Mode 0644, uid/gid 0 and mtime 0 (object
//! times are a per-domain logical clock), so equal archives give equal bytes.
//!
//! Two passes, because a [`Source`] is read once: [`check`] walks the whole
//! archive and judges every name that reaches the export, [`from_archive`]
//! walks it again and writes. A name that is unsafe or ambiguous on Windows,
//! macOS or Linux refuses the whole export before its first byte; nothing is
//! renamed.
use core::iter::once;

use crate::archive::{walk, Collections, Entry, Sink, Source, Summary};
use crate::{Status, COLLECTIONS, MAX_LENGTH, NAME, PER_DOMAIN};

pub const BLOCK: usize = 512;
const PATH: usize = 2 * NAME + 1;
// shortcut: every path fits the 100-byte ustar name field, so there is no prefix or long-name handling; upgrade when NAME grows.
const _: () = assert!(PATH <= 100);
// A pax record "NN path=<path>\n" is `path + 9` bytes, which keeps NN two digits.
const _: () = assert!(PATH + 9 <= 99);
const ZERO: [u8; BLOCK] = [0; BLOCK];

/// Refusal reasons, one bit each in [`Name::reasons`].
///
/// `< > : " | ? * / \` or a control character. `:` anywhere also covers the
/// Windows drive (`C:x`) and alternate-data-stream (`name:stream`) forms.
pub const CHARACTER: u8 = 1;
/// A Windows device name: `CON`, `PRN`, `AUX`, `NUL`, `COM0`-`COM9`,
/// `LPT0`-`LPT9` (also with `¹ ² ³`), `CONIN$` or `CONOUT$`, in any ASCII
/// case, as the part before the first `.` with trailing spaces removed. Also,
/// set by [`check`] and not by [`reasons`]: a root object or a collection
/// holding objects whose name is `PaxHeader` under [`collide`].
pub const RESERVED: u8 = 2;
/// Ends in `.` or a space, which Windows strips (this includes `.` and `..`).
pub const TRAILING: u8 = 4;
/// Equal to another name in the same directory under [`collide`]: two
/// objects in one collection, or two collections that hold objects.
pub const CASE: u8 = 8;
/// An uncollected object and a declared collection, also an empty one, share
/// one path under [`collide`]: a file and a directory.
pub const FILE_AND_DIRECTORY: u8 = 16;
/// A character outside the checked set, for which no case fold is defined.
pub const UNCHECKED: u8 = 32;

/// One judged name. An object has its `slot` and `collection` (0 for none);
/// a collection has its index in `slot` and `collection` 0.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Name {
    pub slot: usize,
    pub collection: u8,
    pub length: usize,
    pub bytes: [u8; NAME],
    /// Refusal bits; 0 means exportable.
    pub reasons: u8,
}

impl Name {
    pub fn name(&self) -> &[u8] {
        self.bytes.get(..self.length).unwrap_or_default()
    }
}

/// Every name of one archive with its refusal reasons.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Names {
    pub objects: usize,
    /// The first `objects` entries, in slot order.
    pub object: [Name; PER_DOMAIN],
    /// Collection `i + 1` at position `i`, length 0 when not declared. Only
    /// collections that hold an object become directories, so only they are
    /// judged by the single-name rules and [`CASE`].
    pub collection: [Name; COLLECTIONS],
}

impl Names {
    /// Whether any name is refused, in which case the export writes nothing.
    pub fn refused(&self) -> bool {
        self.object
            .iter()
            .take(self.objects)
            .chain(&self.collection)
            .any(|name| name.reasons != 0)
    }
}

/// Walk and check a whole archive (as `archive::verify` does) and judge every
/// name. A refusal is reported per name in [`Names`], not as an error.
pub fn check<S: Source>(
    source: &mut S,
    scratch: &mut [u8; MAX_LENGTH],
) -> Result<(Summary, Names), Status> {
    let mut names = Names::default();
    let summary = walk(source, scratch, |entry, collections, _| {
        names.collection = listed(collections);
        *names.object.get_mut(names.objects).ok_or(Status::Corrupt)? = object(entry);
        names.objects += 1;
        Ok(())
    })?;
    judge(&mut names);
    Ok((summary, names))
}

/// Write the export of an archive that [`check`] reported as `names`. The
/// reasons are judged again here, so a caller cannot clear them: any refusal
/// gives `Invalid` before the first byte. The second walk must meet the same
/// slots, names and collections, or it stops with `Stale`. A source that
/// changed or broke after [`check`] can therefore leave files written without
/// the two end-of-archive blocks, which follow only after the whole archive
/// checked out again; output from any error is not an export and must be
/// discarded.
pub fn from_archive<S: Source, K: Sink>(
    source: &mut S,
    scratch: &mut [u8; MAX_LENGTH],
    names: &Names,
    sink: &mut K,
) -> Result<Summary, Status> {
    if names.objects > PER_DOMAIN {
        return Err(Status::Invalid);
    }
    let mut judged = *names;
    judge(&mut judged);
    if judged.refused() {
        return Err(Status::Invalid);
    }
    let mut count = 0;
    let summary = walk(source, scratch, |entry, collections, content| {
        let expected = names.object[..names.objects].get(count);
        if expected.is_none_or(|name| !same(name, &object(entry)))
            || !listed(collections)
                .iter()
                .zip(&names.collection)
                .all(|(a, b)| same(a, b))
        {
            return Err(Status::Stale);
        }
        count += 1;
        let (buffer, length) = path(entry, collections);
        let path = &buffer[..length];
        if !path.is_ascii() {
            let size = length + 9;
            let mut record = [0; PATH + 9];
            record[0] = b'0' + (size / 10) as u8;
            record[1] = b'0' + (size % 10) as u8;
            record[2..8].copy_from_slice(b" path=");
            record[8..8 + length].copy_from_slice(path);
            record[8 + length] = b'\n';
            header(sink, b"PaxHeader", b'x', size)?;
            sink.put(&record[..size])?;
            sink.put(&ZERO[..BLOCK - size])?;
        }
        header(sink, path, b'0', content.len())?;
        sink.put(content)?;
        sink.put(&ZERO[..(BLOCK - content.len() % BLOCK) % BLOCK])
    })?;
    if count != names.objects {
        return Err(Status::Stale);
    }
    sink.put(&ZERO)?;
    sink.put(&ZERO)?;
    Ok(summary)
}

/// The reasons one name (an object or collection name, not a path) is refused
/// on its own; [`check`] adds [`CASE`], [`FILE_AND_DIRECTORY`] and the
/// root-position [`RESERVED`] rule for `PaxHeader`. Names that
/// the archive format already refuses (`/`, `\`, control characters, `.`,
/// `..`) are classified too.
pub fn reasons(name: &[u8]) -> u8 {
    let Ok(text) = core::str::from_utf8(name) else {
        return UNCHECKED;
    };
    let mut bits = 0;
    for c in text.chars() {
        if c.is_control() || "<>:\"|?*/\\".contains(c) {
            bits |= CHARACTER;
        } else if !allowed(c) {
            bits |= UNCHECKED;
        }
    }
    if text.ends_with(['.', ' ']) {
        bits |= TRAILING;
    }
    if reserved(text) {
        bits |= RESERVED;
    }
    bits
}

/// Whether two names are one name to a case-insensitive file system, as far
/// as the checked set goes: letter pairs of ASCII, Latin-1 and Latin
/// Extended-A, and `ß` = `ss`, `ſ` = `s`, `ı` = `İ` = `i`, `ÿ` = `Ÿ`. No
/// Unicode normalisation is applied; the checked set holds no combining mark,
/// so two different names within it are never canonically equivalent.
pub fn collide(a: &[u8], b: &[u8]) -> bool {
    let folded = |name| {
        core::str::from_utf8(name)
            .unwrap_or_default()
            .chars()
            .flat_map(fold)
    };
    folded(a).eq(folded(b))
}

/// The checked set: printable ASCII, Latin-1 Supplement and Latin Extended-A
/// without the invisible soft hyphen, and Dingbats (no letters, no case).
fn allowed(c: char) -> bool {
    matches!(c, ' '..='~' | '\u{A0}'..='\u{17F}' | '\u{2700}'..='\u{27BF}') && c != '\u{AD}'
}

/// Lower case within the checked set; `ß` becomes `ss`. `µ`, `ĸ` and `ŉ` stay:
/// what they fold to lies outside the set.
fn fold(c: char) -> impl Iterator<Item = char> {
    let code = u32::from(c);
    let lower = match c {
        'ß' => return once('s').chain(Some('s')),
        'İ' | 'ı' => 'i',
        'ſ' => 's',
        'Ÿ' => 'ÿ',
        // ASCII and Latin-1 capitals, without U+00D7 (multiplication sign).
        'A'..='Z' | '\u{C0}'..='\u{D6}' | '\u{D8}'..='\u{DE}' => {
            char::from_u32(code + 0x20).unwrap_or(c)
        }
        // Latin Extended-A pairs: capital on the even code point here...
        '\u{100}'..='\u{137}' | '\u{14A}'..='\u{177}' => char::from_u32(code | 1).unwrap_or(c),
        // ...and on the odd one here.
        '\u{139}'..='\u{148}' | '\u{179}'..='\u{17E}' => {
            char::from_u32(code + (code & 1)).unwrap_or(c)
        }
        _ => c,
    };
    once(lower).chain(None)
}

/// Windows device names, matched on the part before the first `.` with
/// trailing spaces removed, in any ASCII case.
fn reserved(text: &str) -> bool {
    let base = text
        .split_once('.')
        .map_or(text, |(base, _)| base)
        .trim_end_matches(' ');
    let mut number = base.get(3..).unwrap_or_default().chars();
    let port = base
        .get(..3)
        .is_some_and(|stem| stem.eq_ignore_ascii_case("COM") || stem.eq_ignore_ascii_case("LPT"))
        && matches!(
            (number.next(), number.next()),
            (Some('0'..='9' | '¹' | '²' | '³'), None)
        );
    port || ["CON", "PRN", "AUX", "NUL", "CONIN$", "CONOUT$"]
        .iter()
        .any(|device| base.eq_ignore_ascii_case(device))
}

/// Set every name's reasons from its bytes and the other names alone.
fn judge(names: &mut Names) {
    let all = *names;
    let objects = &all.object[..all.objects];
    let used: [bool; COLLECTIONS] = core::array::from_fn(|i| {
        objects
            .iter()
            .any(|object| usize::from(object.collection) == i + 1)
    });
    for (i, object) in names.object[..all.objects].iter_mut().enumerate() {
        object.reasons = reasons(object.name());
        if objects.iter().enumerate().any(|(j, other)| {
            j != i && other.collection == object.collection && collide(other.name(), object.name())
        }) {
            object.reasons |= CASE;
        }
        if object.collection == 0
            && all
                .collection
                .iter()
                .any(|collection| collide(collection.name(), object.name()))
        {
            object.reasons |= FILE_AND_DIRECTORY;
        }
        // The exporter's own pax entry name: a pax-unaware reader extracts it at the root.
        if object.collection == 0 && collide(object.name(), b"PaxHeader") {
            object.reasons |= RESERVED;
        }
    }
    for (i, collection) in names.collection.iter_mut().enumerate() {
        collection.reasons = 0;
        if used[i] {
            collection.reasons = reasons(collection.name());
            // The exporter's own pax entry name: a pax-unaware reader extracts it at the root.
            if collide(collection.name(), b"PaxHeader") {
                collection.reasons |= RESERVED;
            }
            if (0..COLLECTIONS)
                .any(|j| j != i && used[j] && collide(all.collection[j].name(), collection.name()))
            {
                collection.reasons |= CASE;
            }
        }
        if objects
            .iter()
            .any(|object| object.collection == 0 && collide(object.name(), collection.name()))
        {
            collection.reasons |= FILE_AND_DIRECTORY;
        }
    }
}

fn object(entry: &Entry) -> Name {
    Name {
        slot: entry.slot,
        collection: entry.collection,
        length: entry.name_length,
        bytes: entry.name,
        reasons: 0,
    }
}

fn listed(collections: &Collections) -> [Name; COLLECTIONS] {
    core::array::from_fn(|i| Name {
        slot: i + 1,
        length: if collections.present[i] {
            collections.lengths[i]
        } else {
            0
        },
        bytes: collections.names[i],
        ..Name::default()
    })
}

fn same(a: &Name, b: &Name) -> bool {
    (a.slot, a.collection, a.name()) == (b.slot, b.collection, b.name())
}

fn path(entry: &Entry, collections: &Collections) -> ([u8; PATH], usize) {
    let name = entry.name();
    let mut out = [0; PATH];
    let mut length = 0;
    if entry.collection != 0 {
        let directory = collections.name(usize::from(entry.collection));
        out[..directory.len()].copy_from_slice(directory);
        out[directory.len()] = b'/';
        length = directory.len() + 1;
    }
    out[length..length + name.len()].copy_from_slice(name);
    (out, length + name.len())
}

fn header<K: Sink>(sink: &mut K, name: &[u8], kind: u8, size: usize) -> Result<(), Status> {
    let mut block = [0; BLOCK];
    block[..name.len()].copy_from_slice(name);
    octal(&mut block[100..108], 0o644);
    octal(&mut block[108..116], 0);
    octal(&mut block[116..124], 0);
    octal(&mut block[124..136], size as u32);
    octal(&mut block[136..148], 0);
    block[148..156].fill(b' ');
    block[156] = kind;
    block[257..263].copy_from_slice(b"ustar\0");
    block[263..265].copy_from_slice(b"00");
    let sum = block.iter().map(|&byte| u32::from(byte)).sum();
    octal(&mut block[148..155], sum);
    sink.put(&block)
}

/// Zero-padded octal digits followed by NUL. Callers keep values in range:
/// sizes are at most MAX_LENGTH and a header sum is at most 512 * 255.
fn octal(field: &mut [u8], mut value: u32) {
    let digits = field.len() - 1;
    for digit in field[..digits].iter_mut().rev() {
        *digit = b'0' + (value & 7) as u8;
        value >>= 3;
    }
    field[digits] = 0;
}
