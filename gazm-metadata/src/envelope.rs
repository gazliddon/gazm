//! Artifact envelope: `magic | version u16 | flags u16 | len u64 | payload`.
//!
//! Version 3 (current files): no header; `payload` is the raw bincode
//! `SourceDatabase` / `SymbolTree`.
//!
//! Version 4 (contract §4): bit 0 of `flags` marks a length-prefixed
//! `TargetInfo` header block at the start of the payload:
//!
//! ```text
//! len (u64, covers everything after the fixed 16 bytes)
//!   target_info_len (u64)
//!   TargetInfo (bincode)
//!   artifact payload (bincode)
//! ```

use serde::{Deserialize, Serialize};

/// On-disk magic values, one per artifact kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Magic {
    /// `GZMP` — source-map artifact (`.map`).
    SourceMap,
    /// `GZSY` — symbols artifact (`.sym`).
    Symbols,
}

impl Magic {
    pub const SOURCE_MAP_BYTES: [u8; 4] = *b"GZMP";
    pub const SYMBOLS_BYTES: [u8; 4] = *b"GZSY";

    pub fn as_bytes(self) -> [u8; 4] {
        match self {
            Magic::SourceMap => Self::SOURCE_MAP_BYTES,
            Magic::Symbols => Self::SYMBOLS_BYTES,
        }
    }

    pub fn from_bytes(bytes: [u8; 4]) -> Option<Self> {
        match &bytes {
            b"GZMP" => Some(Magic::SourceMap),
            b"GZSY" => Some(Magic::Symbols),
            _ => None,
        }
    }
}

/// Current on-disk format version written by `gazm` (writers.rs).
pub const ARTIFACT_VERSION: u16 = 3;
/// Format version that adds the optional `TargetInfo` header.
pub const ARTIFACT_VERSION_WITH_HEADER: u16 = 4;

/// Flag bit 0: a `TargetInfo` header block follows the fixed envelope.
pub const FLAG_HAS_TARGET_INFO: u16 = 0x0001;

/// Per-build target identity embedded at v4 (contract §4).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TargetInfo {
    pub target_name: String,
    pub cpu: CpuKind,
    pub mem_size: usize,
    pub exec_addr: Option<usize>,
    pub bin_references: Vec<BinReference>,
    pub checksums: Vec<RomChecksum>,
    pub sections: Vec<Section>,
    pub tool_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CpuKind {
    Cpu6809,
    Cpu6800,
    Cpu6502,
    Cpu65c02,
    CpuZ80,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BinReference {
    pub file: std::path::PathBuf,
    pub addr: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RomChecksum {
    pub name: String,
    pub addr: usize,
    pub size: usize,
    pub sha1: String,
}

/// Named memory region from the in-asm `section` directives (contract §6).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Section {
    pub name: String,
    pub logical_range: std::ops::Range<usize>,
    pub physical_range: std::ops::Range<usize>,
    pub access: AccessType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AccessType {
    Read,
    Write,
    ReadWrite,
}

/// A decoded artifact: the optional target header plus the raw payload.
#[derive(Debug)]
pub struct Artifact<'a> {
    pub magic: Magic,
    pub version: u16,
    pub flags: u16,
    pub target_info: Option<TargetInfo>,
    pub payload: &'a [u8],
}

/// Errors from `decode_artifact`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactError {
    TooShort,
    BadMagic { expected: Magic, found: [u8; 4] },
    UnknownMagic([u8; 4]),
    UnsupportedVersion(u16),
    UnsupportedFlags(u16),
    TruncatedPayload { declared: u64, available: usize },
    BadHeader,
}

impl std::fmt::Display for ArtifactError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArtifactError::TooShort => write!(f, "artifact too short for the envelope header"),
            ArtifactError::BadMagic { expected, found } => write!(
                f,
                "bad magic {:?}, expected {:?}",
                String::from_utf8_lossy(found),
                String::from_utf8_lossy(&expected.as_bytes())
            ),
            ArtifactError::UnknownMagic(m) => {
                write!(f, "unknown magic {:?}", String::from_utf8_lossy(m))
            }
            ArtifactError::UnsupportedVersion(v) => write!(f, "unsupported artifact version {v}"),
            ArtifactError::UnsupportedFlags(f_) => write!(f, "unsupported artifact flags {f_:04x}"),
            ArtifactError::TruncatedPayload {
                declared,
                available,
            } => write!(
                f,
                "payload truncated: header declares {declared} bytes, only {available} available"
            ),
            ArtifactError::BadHeader => write!(f, "malformed TargetInfo header"),
        }
    }
}

impl std::error::Error for ArtifactError {}

/// Parse the envelope.  `expected` validates the magic; v3 files (no
/// header) yield `target_info: None`.  Unknown versions and flag bits
/// are rejected before any payload decoding.
pub fn decode_artifact(bytes: &[u8], expected: Magic) -> Result<Artifact<'_>, ArtifactError> {
    if bytes.len() < 16 {
        return Err(ArtifactError::TooShort);
    }
    let magic: [u8; 4] = bytes[0..4].try_into().unwrap();
    let found = Magic::from_bytes(magic).ok_or(ArtifactError::UnknownMagic(magic))?;
    if found != expected {
        return Err(ArtifactError::BadMagic {
            expected,
            found: magic,
        });
    }
    let version = u16::from_le_bytes(bytes[4..6].try_into().unwrap());
    let flags = u16::from_le_bytes(bytes[6..8].try_into().unwrap());
    let len = u64::from_le_bytes(bytes[8..16].try_into().unwrap());

    let payload_len = usize::try_from(len).map_err(|_| ArtifactError::TruncatedPayload {
        declared: len,
        available: bytes.len().saturating_sub(16),
    })?;
    let Some(end) = 16usize.checked_add(payload_len) else {
        return Err(ArtifactError::TruncatedPayload {
            declared: len,
            available: bytes.len().saturating_sub(16),
        });
    };
    if end > bytes.len() {
        return Err(ArtifactError::TruncatedPayload {
            declared: len,
            available: bytes.len().saturating_sub(16),
        });
    }
    let mut payload = &bytes[16..end];

    let target_info = if flags & FLAG_HAS_TARGET_INFO != 0 {
        if version < ARTIFACT_VERSION_WITH_HEADER {
            return Err(ArtifactError::UnsupportedVersion(version));
        }
        if payload.len() < 8 {
            return Err(ArtifactError::BadHeader);
        }
        let header_len = u64::from_le_bytes(payload[0..8].try_into().unwrap());
        let header_len = usize::try_from(header_len).map_err(|_| ArtifactError::BadHeader)?;
        if 8 + header_len > payload.len() {
            return Err(ArtifactError::BadHeader);
        }
        let (header_bytes, rest) = payload.split_at(8 + header_len);
        let info: TargetInfo =
            bincode::deserialize(&header_bytes[8..]).map_err(|_| ArtifactError::BadHeader)?;
        payload = rest;
        Some(info)
    } else {
        if flags != 0 {
            return Err(ArtifactError::UnsupportedFlags(flags));
        }
        None
    };

    Ok(Artifact {
        magic: found,
        version,
        flags,
        target_info,
        payload,
    })
}

/// Encode an artifact (writer side, used by tests to round-trip).
/// Mirrors gazm's `encode_artifact`; kept here so the reader can be
/// tested without depending on the `gazm` crate.
pub fn encode_artifact(magic: Magic, target_info: Option<&TargetInfo>, payload: &[u8]) -> Vec<u8> {
    let version = if target_info.is_some() {
        ARTIFACT_VERSION_WITH_HEADER
    } else {
        ARTIFACT_VERSION
    };
    let flags = if target_info.is_some() {
        FLAG_HAS_TARGET_INFO
    } else {
        0
    };
    let mut body = Vec::new();
    if let Some(info) = target_info {
        let header = bincode::serialize(info).expect("TargetInfo serializes");
        body.extend_from_slice(&(header.len() as u64).to_le_bytes());
        body.extend_from_slice(&header);
    }
    body.extend_from_slice(payload);

    let mut out = Vec::with_capacity(16 + body.len());
    out.extend_from_slice(&magic.as_bytes());
    out.extend_from_slice(&version.to_le_bytes());
    out.extend_from_slice(&flags.to_le_bytes());
    out.extend_from_slice(&(body.len() as u64).to_le_bytes());
    out.extend_from_slice(&body);
    out
}
