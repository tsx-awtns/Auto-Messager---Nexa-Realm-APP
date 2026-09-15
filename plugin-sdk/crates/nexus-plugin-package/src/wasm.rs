// Nexus Realm
// File: wasm.rs
// Purpose: Bounded WebAssembly module inspection for structural validation

use crate::error::PackageError;

const MAGIC: [u8; 4] = [0x00, 0x61, 0x73, 0x6d];
const SUPPORTED_VERSION: u32 = 1;
const MAX_SECTIONS: usize = 64;
const MAX_ENTRIES: usize = 4096;
const MAX_NAME_BYTES: usize = 512;

pub struct WasmInspection {
    pub version: u32,
    pub size: u64,
    pub exports: Vec<String>,
    pub imports: Vec<String>,
}

impl WasmInspection {
    pub fn missing_abi_exports(&self) -> Vec<&'static str> {
        nexus_plugin_api::abi::REQUIRED_EXPORTS
            .into_iter()
            .filter(|name| !self.exports.iter().any(|export| export == name))
            .collect()
    }
}

pub fn inspect(bytes: &[u8]) -> Result<WasmInspection, PackageError> {
    if bytes.len() < 8 {
        return Err(PackageError::Module(
            "module is shorter than a WebAssembly header".to_string(),
        ));
    }
    if bytes[0..4] != MAGIC {
        return Err(PackageError::Module(
            "module does not start with the WebAssembly magic bytes".to_string(),
        ));
    }
    let version = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    if version != SUPPORTED_VERSION {
        return Err(PackageError::Module(format!(
            "module declares WebAssembly version {version}; only version 1 core modules are supported"
        )));
    }
    let mut inspection = WasmInspection {
        version,
        size: bytes.len() as u64,
        exports: Vec::new(),
        imports: Vec::new(),
    };
    let mut cursor = 8usize;
    let mut sections = 0usize;
    while cursor < bytes.len() {
        sections += 1;
        if sections > MAX_SECTIONS {
            return Err(PackageError::Module(
                "module declares too many sections".to_string(),
            ));
        }
        let id = bytes[cursor];
        cursor += 1;
        let size = read_leb(bytes, &mut cursor)? as usize;
        let end = cursor
            .checked_add(size)
            .ok_or_else(|| PackageError::Module("section size overflows".to_string()))?;
        if end > bytes.len() {
            return Err(PackageError::Module(
                "section extends past the end of the module".to_string(),
            ));
        }
        match id {
            2 => read_imports(&bytes[cursor..end], &mut inspection.imports)?,
            7 => read_exports(&bytes[cursor..end], &mut inspection.exports)?,
            _ => {}
        }
        cursor = end;
    }
    Ok(inspection)
}

fn read_imports(section: &[u8], out: &mut Vec<String>) -> Result<(), PackageError> {
    let mut cursor = 0usize;
    let count = read_leb(section, &mut cursor)? as usize;
    if count > MAX_ENTRIES {
        return Err(PackageError::Module(
            "import section declares too many entries".to_string(),
        ));
    }
    for _ in 0..count {
        let module = read_name(section, &mut cursor)?;
        let field = read_name(section, &mut cursor)?;
        out.push(format!("{module}::{field}"));
        let kind = read_byte(section, &mut cursor)?;
        match kind {
            0x00 => {
                read_leb(section, &mut cursor)?;
            }
            0x01 => {
                let element = read_byte(section, &mut cursor)?;
                if element != 0x70 {
                    return Err(PackageError::Module(
                        "unsupported table element type".to_string(),
                    ));
                }
                read_limits(section, &mut cursor)?;
            }
            0x02 => {
                read_limits(section, &mut cursor)?;
            }
            0x03 => {
                read_byte(section, &mut cursor)?;
                let mutable = read_byte(section, &mut cursor)?;
                if mutable > 1 {
                    return Err(PackageError::Module(
                        "invalid global mutability flag".to_string(),
                    ));
                }
            }
            _ => {
                return Err(PackageError::Module(
                    "unknown import kind".to_string(),
                ))
            }
        }
    }
    Ok(())
}

fn read_exports(section: &[u8], out: &mut Vec<String>) -> Result<(), PackageError> {
    let mut cursor = 0usize;
    let count = read_leb(section, &mut cursor)? as usize;
    if count > MAX_ENTRIES {
        return Err(PackageError::Module(
            "export section declares too many entries".to_string(),
        ));
    }
    for _ in 0..count {
        let name = read_name(section, &mut cursor)?;
        let kind = read_byte(section, &mut cursor)?;
        if kind > 0x03 {
            return Err(PackageError::Module("unknown export kind".to_string()));
        }
        read_leb(section, &mut cursor)?;
        out.push(name);
    }
    Ok(())
}

fn read_limits(section: &[u8], cursor: &mut usize) -> Result<(), PackageError> {
    let flags = read_byte(section, cursor)?;
    match flags {
        0x00 => {
            read_leb(section, cursor)?;
        }
        0x01 => {
            read_leb(section, cursor)?;
            read_leb(section, cursor)?;
        }
        _ => return Err(PackageError::Module("invalid limits flags".to_string())),
    }
    Ok(())
}

fn read_name(section: &[u8], cursor: &mut usize) -> Result<String, PackageError> {
    let length = read_leb(section, cursor)? as usize;
    if length > MAX_NAME_BYTES {
        return Err(PackageError::Module(
            "name is longer than allowed".to_string(),
        ));
    }
    let end = cursor
        .checked_add(length)
        .ok_or_else(|| PackageError::Module("name length overflows".to_string()))?;
    if end > section.len() {
        return Err(PackageError::Module(
            "name extends past the section end".to_string(),
        ));
    }
    let name = std::str::from_utf8(&section[*cursor..end])
        .map_err(|_| PackageError::Module("name is not valid UTF-8".to_string()))?;
    *cursor = end;
    Ok(name.to_string())
}

fn read_byte(section: &[u8], cursor: &mut usize) -> Result<u8, PackageError> {
    let byte = *section
        .get(*cursor)
        .ok_or_else(|| PackageError::Module("unexpected end of section".to_string()))?;
    *cursor += 1;
    Ok(byte)
}

fn read_leb(section: &[u8], cursor: &mut usize) -> Result<u32, PackageError> {
    let mut result = 0u32;
    let mut shift = 0u32;
    loop {
        let byte = read_byte(section, cursor)?;
        if shift == 28 && byte > 0x0f {
            return Err(PackageError::Module("integer is too large".to_string()));
        }
        result |= u32::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Ok(result);
        }
        shift += 7;
        if shift > 28 {
            return Err(PackageError::Module("integer is too large".to_string()));
        }
    }
}
