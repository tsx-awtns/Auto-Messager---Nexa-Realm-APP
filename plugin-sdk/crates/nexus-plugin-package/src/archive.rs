// Nexus Realm
// File: archive.rs
// Purpose: Deterministic .nexusplugin container reader and writer

use crate::error::PackageError;
use nexus_plugin_api::{
    forbidden_suffix_of, limits::MAX_PACKAGE_ENTRIES, path, MAX_MODULE_BYTES, MAX_PLUGIN_BYTES,
    MODULE_FILE, PACKAGE_INTEGRITY_FILE,
};
use std::collections::BTreeSet;
use std::sync::OnceLock;

pub const EXTENSION: &str = "nexusplugin";
pub const STORE_METHOD: u16 = 0;

const LOCAL_SIGNATURE: u32 = 0x0403_4b50;
const CENTRAL_SIGNATURE: u32 = 0x0201_4b50;
const END_SIGNATURE: u32 = 0x0605_4b50;
const UTF8_FLAG: u16 = 0x0800;
const DATA_DESCRIPTOR_FLAG: u16 = 0x0008;
const ENCRYPTED_FLAG: u16 = 0x0001;
const VERSION_NEEDED: u16 = 20;
const VERSION_MADE_BY: u16 = 20;
const DOS_DATE: u16 = 0x0021;
const DOS_TIME: u16 = 0;
const SYMLINK_MODE: u32 = 0xA000;
const MAX_COMMENT_BYTES: usize = 65_535;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    pub bytes: Vec<u8>,
}

impl Entry {
    pub fn new(name: &str, bytes: Vec<u8>) -> Result<Self, PackageError> {
        validate_entry_name(name)?;
        Ok(Self {
            name: name.to_string(),
            bytes,
        })
    }
}

pub fn is_allowed_entry(name: &str) -> bool {
    path::is_plugin_file(name) || name == PACKAGE_INTEGRITY_FILE
}

pub fn validate_entry_name(name: &str) -> Result<(), PackageError> {
    if name.ends_with('/') {
        return Err(PackageError::Entry(format!(
            "`{name}` is a directory entry; `.nexusplugin` packages contain files only"
        )));
    }
    path::validate_relative(name)
        .map_err(|_| PackageError::Entry(format!("`{name}` is not a safe package path")))?;
    if !is_allowed_entry(name) {
        return Err(PackageError::Entry(format!(
            "`{name}` is not allowed in a `.nexusplugin` package"
        )));
    }
    if forbidden_suffix_of(name).is_some() {
        return Err(PackageError::Entry(format!(
            "`{name}` is a forbidden native payload"
        )));
    }
    Ok(())
}

pub fn write_archive(entries: &[Entry]) -> Result<Vec<u8>, PackageError> {
    if entries.is_empty() || entries.len() > MAX_PACKAGE_ENTRIES {
        return Err(PackageError::Limit(
            "package entry count is out of range".to_string(),
        ));
    }
    let mut body: Vec<u8> = Vec::new();
    let mut directory: Vec<u8> = Vec::new();
    let mut names: BTreeSet<&str> = BTreeSet::new();
    let mut total = 0u64;
    for entry in entries {
        validate_entry_name(&entry.name)?;
        if !names.insert(entry.name.as_str()) {
            return Err(PackageError::Entry(format!(
                "duplicate entry `{}`",
                entry.name
            )));
        }
        if entry.name == MODULE_FILE && entry.bytes.len() as u64 > MAX_MODULE_BYTES {
            return Err(PackageError::Limit(format!(
                "`{MODULE_FILE}` exceeds the module size limit"
            )));
        }
        total = total
            .checked_add(entry.bytes.len() as u64)
            .ok_or_else(|| PackageError::Limit("package size overflowed".to_string()))?;
        if total > MAX_PLUGIN_BYTES {
            return Err(PackageError::Limit(
                "package content exceeds the plugin size limit".to_string(),
            ));
        }
        let name = entry.name.as_bytes();
        let size = u32::try_from(entry.bytes.len())
            .map_err(|_| PackageError::Limit(format!("`{}` is too large", entry.name)))?;
        let offset = u32::try_from(body.len())
            .map_err(|_| PackageError::Limit("package is too large".to_string()))?;
        let crc = crc32(&entry.bytes);
        push_u32(&mut body, LOCAL_SIGNATURE);
        push_u16(&mut body, VERSION_NEEDED);
        push_u16(&mut body, UTF8_FLAG);
        push_u16(&mut body, STORE_METHOD);
        push_u16(&mut body, DOS_TIME);
        push_u16(&mut body, DOS_DATE);
        push_u32(&mut body, crc);
        push_u32(&mut body, size);
        push_u32(&mut body, size);
        push_u16(&mut body, name.len() as u16);
        push_u16(&mut body, 0);
        body.extend_from_slice(name);
        body.extend_from_slice(&entry.bytes);

        push_u32(&mut directory, CENTRAL_SIGNATURE);
        push_u16(&mut directory, VERSION_MADE_BY);
        push_u16(&mut directory, VERSION_NEEDED);
        push_u16(&mut directory, UTF8_FLAG);
        push_u16(&mut directory, STORE_METHOD);
        push_u16(&mut directory, DOS_TIME);
        push_u16(&mut directory, DOS_DATE);
        push_u32(&mut directory, crc);
        push_u32(&mut directory, size);
        push_u32(&mut directory, size);
        push_u16(&mut directory, name.len() as u16);
        push_u16(&mut directory, 0);
        push_u16(&mut directory, 0);
        push_u16(&mut directory, 0);
        push_u16(&mut directory, 0);
        push_u32(&mut directory, 0);
        push_u32(&mut directory, offset);
        directory.extend_from_slice(name);
    }
    let central_offset = u32::try_from(body.len())
        .map_err(|_| PackageError::Limit("package is too large".to_string()))?;
    let central_size = u32::try_from(directory.len())
        .map_err(|_| PackageError::Limit("package is too large".to_string()))?;
    let count = u16::try_from(entries.len())
        .map_err(|_| PackageError::Limit("package has too many entries".to_string()))?;
    let mut out = body;
    out.extend_from_slice(&directory);
    push_u32(&mut out, END_SIGNATURE);
    push_u16(&mut out, 0);
    push_u16(&mut out, 0);
    push_u16(&mut out, count);
    push_u16(&mut out, count);
    push_u32(&mut out, central_size);
    push_u32(&mut out, central_offset);
    push_u16(&mut out, 0);
    Ok(out)
}

pub fn read_archive(bytes: &[u8]) -> Result<Vec<Entry>, PackageError> {
    let end = find_end_record(bytes)?;
    let total = read_u16(bytes, end + 10)? as usize;
    let central_size = read_u32(bytes, end + 12)? as usize;
    let central_offset = read_u32(bytes, end + 16)? as usize;
    if read_u16(bytes, end + 4)? != 0 || read_u16(bytes, end + 6)? != 0 {
        return Err(PackageError::Archive(
            "multi-disk archives are not supported".to_string(),
        ));
    }
    if total == 0 || total > MAX_PACKAGE_ENTRIES {
        return Err(PackageError::Limit(
            "package entry count is out of range".to_string(),
        ));
    }
    let central_end = central_offset
        .checked_add(central_size)
        .ok_or_else(|| PackageError::Archive("central directory overflows".to_string()))?;
    if central_end > end {
        return Err(PackageError::Archive(
            "central directory extends past the end record".to_string(),
        ));
    }
    let mut entries = Vec::new();
    let mut names: BTreeSet<String> = BTreeSet::new();
    let mut cursor = central_offset;
    let mut total_bytes = 0u64;
    for _ in 0..total {
        if read_u32(bytes, cursor)? != CENTRAL_SIGNATURE {
            return Err(PackageError::Archive(
                "central directory entry is malformed".to_string(),
            ));
        }
        let flags = read_u16(bytes, cursor + 8)?;
        let method = read_u16(bytes, cursor + 10)?;
        let crc = read_u32(bytes, cursor + 16)?;
        let compressed = read_u32(bytes, cursor + 20)? as usize;
        let size = read_u32(bytes, cursor + 24)? as usize;
        let name_length = read_u16(bytes, cursor + 28)? as usize;
        let extra_length = read_u16(bytes, cursor + 30)? as usize;
        let comment_length = read_u16(bytes, cursor + 32)? as usize;
        let external = read_u32(bytes, cursor + 38)?;
        let local_offset = read_u32(bytes, cursor + 42)? as usize;
        let name_start = cursor + 46;
        let name_end = name_start
            .checked_add(name_length)
            .ok_or_else(|| PackageError::Archive("entry name overflows".to_string()))?;
        if name_end > bytes.len() {
            return Err(PackageError::Archive("entry name is truncated".to_string()));
        }
        let name = std::str::from_utf8(&bytes[name_start..name_end])
            .map_err(|_| PackageError::Entry("entry name is not valid UTF-8".to_string()))?
            .to_string();
        validate_entry_name(&name)?;
        if flags & (DATA_DESCRIPTOR_FLAG | ENCRYPTED_FLAG) != 0 {
            return Err(PackageError::Entry(format!(
                "`{name}` uses an unsupported archive feature"
            )));
        }
        if method != STORE_METHOD {
            return Err(PackageError::Entry(format!(
                "`{name}` uses an unsupported compression method"
            )));
        }
        if external >> 16 & 0xF000 == SYMLINK_MODE {
            return Err(PackageError::Entry(format!(
                "`{name}` is a symbolic link entry"
            )));
        }
        if compressed != size {
            return Err(PackageError::Archive(format!(
                "`{name}` declares mismatched sizes"
            )));
        }
        if size as u64 > MAX_PLUGIN_BYTES {
            return Err(PackageError::Limit(format!("`{name}` is too large")));
        }
        if name == MODULE_FILE && size as u64 > MAX_MODULE_BYTES {
            return Err(PackageError::Limit(format!(
                "`{name}` exceeds the module limit"
            )));
        }
        total_bytes = total_bytes
            .checked_add(size as u64)
            .ok_or_else(|| PackageError::Limit("package size overflowed".to_string()))?;
        if total_bytes > MAX_PLUGIN_BYTES {
            return Err(PackageError::Limit(
                "package content exceeds the plugin size limit".to_string(),
            ));
        }
        if !names.insert(name.clone()) {
            return Err(PackageError::Entry(format!("duplicate entry `{name}`")));
        }
        let data = local_data(bytes, local_offset, &name, size)?;
        if crc32(data) != crc {
            return Err(PackageError::Integrity(format!(
                "`{name}` fails its CRC-32 check"
            )));
        }
        entries.push(Entry {
            name,
            bytes: data.to_vec(),
        });
        cursor = name_end
            .checked_add(extra_length)
            .and_then(|value| value.checked_add(comment_length))
            .ok_or_else(|| PackageError::Archive("central directory overflows".to_string()))?;
        if cursor > central_end {
            return Err(PackageError::Archive(
                "central directory entry extends past the directory".to_string(),
            ));
        }
    }
    Ok(entries)
}

fn local_data<'a>(
    bytes: &'a [u8],
    offset: usize,
    name: &str,
    size: usize,
) -> Result<&'a [u8], PackageError> {
    if read_u32(bytes, offset)? != LOCAL_SIGNATURE {
        return Err(PackageError::Archive(format!(
            "`{name}` has no usable local header"
        )));
    }
    let name_length = read_u16(bytes, offset + 26)? as usize;
    let extra_length = read_u16(bytes, offset + 28)? as usize;
    let name_start = offset + 30;
    let name_end = name_start
        .checked_add(name_length)
        .ok_or_else(|| PackageError::Archive("local header overflows".to_string()))?;
    if name_end > bytes.len() {
        return Err(PackageError::Archive(
            "local header is truncated".to_string(),
        ));
    }
    let local_name = std::str::from_utf8(&bytes[name_start..name_end])
        .map_err(|_| PackageError::Entry("local header name is not valid UTF-8".to_string()))?;
    if local_name != name {
        return Err(PackageError::Archive(format!(
            "`{name}` does not match its local header name"
        )));
    }
    let data_start = name_end
        .checked_add(extra_length)
        .ok_or_else(|| PackageError::Archive("local header overflows".to_string()))?;
    let data_end = data_start
        .checked_add(size)
        .ok_or_else(|| PackageError::Archive("entry size overflows".to_string()))?;
    if data_end > bytes.len() {
        return Err(PackageError::Archive(format!("`{name}` data is truncated")));
    }
    Ok(&bytes[data_start..data_end])
}

fn find_end_record(bytes: &[u8]) -> Result<usize, PackageError> {
    if bytes.len() < 22 {
        return Err(PackageError::Archive("archive is too small".to_string()));
    }
    let lowest = bytes.len().saturating_sub(22 + MAX_COMMENT_BYTES);
    let mut offset = bytes.len() - 22;
    loop {
        if read_u32(bytes, offset)? == END_SIGNATURE {
            let comment = read_u16(bytes, offset + 20)? as usize;
            if offset + 22 + comment == bytes.len() {
                return Ok(offset);
            }
        }
        if offset == lowest {
            return Err(PackageError::Archive(
                "archive has no end-of-central-directory record".to_string(),
            ));
        }
        offset -= 1;
    }
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, PackageError> {
    let end = offset
        .checked_add(2)
        .ok_or_else(|| PackageError::Archive("archive offset overflows".to_string()))?;
    let slice = bytes
        .get(offset..end)
        .ok_or_else(|| PackageError::Archive("archive is truncated".to_string()))?;
    Ok(u16::from_le_bytes([slice[0], slice[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, PackageError> {
    let end = offset
        .checked_add(4)
        .ok_or_else(|| PackageError::Archive("archive offset overflows".to_string()))?;
    let slice = bytes
        .get(offset..end)
        .ok_or_else(|| PackageError::Archive("archive is truncated".to_string()))?;
    Ok(u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]))
}

fn push_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn push_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

pub fn crc32(bytes: &[u8]) -> u32 {
    let table = crc_table();
    let mut crc = 0xFFFF_FFFFu32;
    for byte in bytes {
        crc = table[((crc ^ u32::from(*byte)) & 0xff) as usize] ^ (crc >> 8);
    }
    crc ^ 0xFFFF_FFFF
}

fn crc_table() -> &'static [u32; 256] {
    static TABLE: OnceLock<[u32; 256]> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut table = [0u32; 256];
        let mut index = 0usize;
        while index < 256 {
            let mut value = index as u32;
            let mut bit = 0;
            while bit < 8 {
                value = if value & 1 == 1 {
                    0xEDB8_8320 ^ (value >> 1)
                } else {
                    value >> 1
                };
                bit += 1;
            }
            table[index] = value;
            index += 1;
        }
        table
    })
}
