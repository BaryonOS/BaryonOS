use crate::{Error, MAX_XML_BYTES};
use alloc::vec::Vec;
use zune_inflate::{DeflateDecoder, DeflateOptions};

pub const MAX_PACKAGE_BYTES: usize = 16 * 1024 * 1024;
const MAX_PARTS: usize = 512;
const MAX_EXPANDED_BYTES: usize = 16 * 1024 * 1024;

pub struct Part<'a> {
    pub name: &'a str,
    pub bytes: Vec<u8>,
}

pub fn parts(input: &[u8]) -> Result<Vec<Part<'_>>, Error> {
    if input.len() > MAX_PACKAGE_BYTES {
        return Err(Error::PackageLimit);
    }
    let eocd = (0..input.len().saturating_sub(21))
        .rev()
        .take(65536)
        .find(|&at| {
            input.get(at..at + 4) == Some(b"PK\x05\x06")
                && usize::from(u16_at(input, at + 20).unwrap_or(0)) + at + 22 == input.len()
        })
        .ok_or(Error::InvalidPackage)?;
    if u16_at(input, eocd + 4)? != 0 || u16_at(input, eocd + 6)? != 0 {
        return Err(Error::UnsupportedPackage);
    }
    let count = usize::from(u16_at(input, eocd + 10)?);
    if count == 0 || count > MAX_PARTS {
        return Err(Error::PackageLimit);
    }
    if usize::from(u16_at(input, eocd + 8)?) != count {
        return Err(Error::InvalidPackage);
    }
    let cd_size = u32_at(input, eocd + 12)? as usize;
    let cd_start = u32_at(input, eocd + 16)? as usize;
    if cd_start.checked_add(cd_size) != Some(eocd) {
        return Err(Error::InvalidPackage);
    }
    let mut result: Vec<Part<'_>> = Vec::new();
    result
        .try_reserve_exact(count)
        .map_err(|_| Error::OutOfMemory)?;
    let mut cd = cd_start;
    let mut local = 0;
    let mut expanded = 0usize;
    for _ in 0..count {
        let header = input
            .get(cd..cd.checked_add(46).ok_or(Error::InvalidPackage)?)
            .filter(|header| header.starts_with(b"PK\x01\x02"))
            .ok_or(Error::InvalidPackage)?;
        let flags = u16_at(header, 8)?;
        let method = u16_at(header, 10)?;
        let crc = u32_at(header, 16)?;
        let compressed = u32_at(header, 20)? as usize;
        let size = u32_at(header, 24)? as usize;
        let name_len = usize::from(u16_at(header, 28)?);
        let extra_len = usize::from(u16_at(header, 30)?);
        let comment_len = usize::from(u16_at(header, 32)?);
        if flags & !0x080e != 0
            || !matches!(method, 0 | 8)
            || u16_at(header, 34)? != 0
            || u32_at(header, 42)? as usize != local
        {
            return Err(Error::UnsupportedPackage);
        }
        expanded = expanded.checked_add(size).ok_or(Error::PackageLimit)?;
        if size > MAX_XML_BYTES || expanded > MAX_EXPANDED_BYTES {
            return Err(Error::PackageLimit);
        }
        let name_start = cd + 46;
        let name_end = name_start
            .checked_add(name_len)
            .ok_or(Error::InvalidPackage)?;
        let name_bytes = input
            .get(name_start..name_end)
            .ok_or(Error::InvalidPackage)?;
        let name = core::str::from_utf8(name_bytes).map_err(|_| Error::UnsupportedPackage)?;
        if name.is_empty()
            || name.starts_with('/')
            || name.contains('\\')
            || name.split('/').any(|piece| matches!(piece, "." | ".."))
            || name.chars().any(|c| c.is_control())
            || result
                .iter()
                .any(|part| part.name.eq_ignore_ascii_case(name))
        {
            return Err(Error::UnsupportedPackage);
        }
        let extras_end = name_end
            .checked_add(extra_len)
            .ok_or(Error::InvalidPackage)?;
        validate_extra(
            input
                .get(name_end..extras_end)
                .ok_or(Error::InvalidPackage)?,
        )?;
        cd = extras_end
            .checked_add(comment_len)
            .filter(|&end| end <= eocd)
            .ok_or(Error::InvalidPackage)?;
        let lh = input
            .get(local..local.checked_add(30).ok_or(Error::InvalidPackage)?)
            .filter(|header| header.starts_with(b"PK\x03\x04"))
            .ok_or(Error::InvalidPackage)?;
        if u16_at(lh, 6)? != flags
            || u16_at(lh, 8)? != method
            || usize::from(u16_at(lh, 26)?) != name_len
        {
            return Err(Error::InvalidPackage);
        }
        let local_name_start = local + 30;
        let local_name_end = local_name_start
            .checked_add(name_len)
            .ok_or(Error::InvalidPackage)?;
        if input.get(local_name_start..local_name_end) != Some(name_bytes) {
            return Err(Error::InvalidPackage);
        }
        let data_start = local_name_end
            .checked_add(usize::from(u16_at(lh, 28)?))
            .ok_or(Error::InvalidPackage)?;
        validate_extra(
            input
                .get(local_name_end..data_start)
                .ok_or(Error::InvalidPackage)?,
        )?;
        for (at, expected) in [(14, crc), (18, compressed as u32), (22, size as u32)] {
            let actual = u32_at(lh, at)?;
            if actual != expected && (flags & 8 == 0 || actual != 0) {
                return Err(Error::InvalidPackage);
            }
        }
        local = data_start
            .checked_add(compressed)
            .filter(|&end| end <= cd_start)
            .ok_or(Error::InvalidPackage)?;
        let compressed_bytes = input.get(data_start..local).ok_or(Error::InvalidPackage)?;
        let decoded = if method == 0 {
            if compressed != size {
                return Err(Error::InvalidPackage);
            }
            let mut bytes = Vec::new();
            bytes
                .try_reserve_exact(size)
                .map_err(|_| Error::OutOfMemory)?;
            bytes.extend_from_slice(compressed_bytes);
            bytes
        } else {
            let options = DeflateOptions::default()
                .set_limit(size)
                .set_size_hint(size.min(32768));
            DeflateDecoder::new_with_options(compressed_bytes, options)
                .decode_deflate()
                .map_err(|_| Error::InvalidCompression)?
        };
        if decoded.len() != size || crc32(&decoded) != crc {
            return Err(Error::InvalidPackage);
        }
        if flags & 8 != 0 {
            if input.get(local..local + 4) == Some(b"PK\x07\x08") {
                local += 4;
            }
            let descriptor = input.get(local..local + 12).ok_or(Error::InvalidPackage)?;
            if u32_at(descriptor, 0)? != crc
                || u32_at(descriptor, 4)? as usize != compressed
                || u32_at(descriptor, 8)? as usize != size
            {
                return Err(Error::InvalidPackage);
            }
            local += 12;
        }
        if local > cd_start {
            return Err(Error::InvalidPackage);
        }
        result.push(Part {
            name,
            bytes: decoded,
        });
    }
    if cd != eocd || local != cd_start {
        return Err(Error::InvalidPackage);
    }
    Ok(result)
}

fn validate_extra(mut bytes: &[u8]) -> Result<(), Error> {
    while !bytes.is_empty() {
        if u16_at(bytes, 0)? == 1 {
            return Err(Error::UnsupportedPackage);
        }
        let len = usize::from(u16_at(bytes, 2)?);
        bytes = bytes
            .get(4..)
            .and_then(|b| b.get(len..))
            .ok_or(Error::InvalidPackage)?;
    }
    Ok(())
}

fn u16_at(input: &[u8], at: usize) -> Result<u16, Error> {
    let bytes = input
        .get(at..at.checked_add(2).ok_or(Error::InvalidPackage)?)
        .ok_or(Error::InvalidPackage)?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}

fn u32_at(input: &[u8], at: usize) -> Result<u32, Error> {
    let bytes = input
        .get(at..at.checked_add(4).ok_or(Error::InvalidPackage)?)
        .ok_or(Error::InvalidPackage)?;
    Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

// ZIP CRC-32/ISO-HDLC detects corruption; it is not authentication.
fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = !0u32;
    for &byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb88320 & 0u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}
