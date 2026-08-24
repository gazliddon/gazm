//! Envelope round-trip and real-artifact tests.

use gazm_metadata::envelope::{
    decode_artifact, encode_artifact, AccessType, BinReference, CpuKind, Magic, RomChecksum,
    Section, StructSize, TargetInfo, ARTIFACT_VERSION, ARTIFACT_VERSION_NAMED_HEADER,
    ARTIFACT_VERSION_WITH_HEADER, FLAG_HAS_TARGET_INFO,
};
use gazm_metadata::target::Target;

fn sample_target_info() -> TargetInfo {
    TargetInfo {
        target_name: "stargate".into(),
        cpu: CpuKind::Cpu6809,
        mem_size: 94208,
        exec_addr: Some(0xf486),
        bin_references: vec![BinReference {
            file: "roms/01".into(),
            addr: 0x0000,
        }],
        checksums: vec![RomChecksum {
            name: "rom_1".into(),
            addr: 0x0000,
            size: 0x1000,
            sha1: "f003a5a9319c4eb8991fa2aae3f10c72d6b8e81a".into(),
        }],
        sections: vec![Section {
            name: "lo_rom".into(),
            logical_range: 0x0000..0x9000,
            physical_range: 0x0000..0x9000,
            access: AccessType::Read,
        }],
        tool_version: "0.9.17".into(),
        struct_sizes: vec![StructSize {
            name: "Proc".into(),
            size: 15,
        }],
    }
}

#[test]
fn v3_envelope_round_trips() {
    let payload = b"fake-bincode-payload";
    let bytes = encode_artifact(Magic::SourceMap, None, payload);
    assert_eq!(&bytes[0..4], b"GZMP");
    assert_eq!(
        u16::from_le_bytes(bytes[4..6].try_into().unwrap()),
        ARTIFACT_VERSION
    );
    assert_eq!(u16::from_le_bytes(bytes[6..8].try_into().unwrap()), 0);

    let artifact = decode_artifact(&bytes, Magic::SourceMap).unwrap();
    assert_eq!(artifact.magic, Magic::SourceMap);
    assert_eq!(artifact.version, ARTIFACT_VERSION);
    assert_eq!(artifact.target_info, None);
    assert_eq!(artifact.payload, payload);
}

#[test]
fn v4_envelope_round_trips_with_header() {
    let payload = b"fake-bincode-payload";
    let info = sample_target_info();
    let bytes = encode_artifact(Magic::Symbols, Some(&info), payload);
    assert_eq!(&bytes[0..4], b"GZSY");
    assert_eq!(
        u16::from_le_bytes(bytes[4..6].try_into().unwrap()),
        ARTIFACT_VERSION_NAMED_HEADER
    );
    assert_eq!(
        u16::from_le_bytes(bytes[6..8].try_into().unwrap()),
        FLAG_HAS_TARGET_INFO
    );

    let artifact = decode_artifact(&bytes, Magic::Symbols).unwrap();
    assert_eq!(artifact.version, ARTIFACT_VERSION_NAMED_HEADER);
    assert_eq!(artifact.target_info, Some(info));
    assert_eq!(artifact.payload, payload);
}

#[test]
fn v5_header_ignores_unknown_fields() {
    // The named-map header is forward-evolvable: a file written by a
    // future gazm with extra header fields must still decode, with the
    // known fields intact and unknown ones ignored.
    let info = sample_target_info();
    let payload = b"fake-bincode-payload";

    // The future layout: current fields plus one appended field.
    #[derive(serde::Serialize)]
    struct FutureTargetInfo {
        target_name: String,
        cpu: CpuKind,
        mem_size: usize,
        exec_addr: Option<usize>,
        bin_references: Vec<BinReference>,
        checksums: Vec<RomChecksum>,
        sections: Vec<Section>,
        tool_version: String,
        struct_sizes: Vec<StructSize>,
        future_field: u32,
    }
    let future = FutureTargetInfo {
        target_name: info.target_name.clone(),
        cpu: info.cpu.clone(),
        mem_size: info.mem_size,
        exec_addr: info.exec_addr,
        bin_references: info.bin_references.clone(),
        checksums: info.checksums.clone(),
        sections: info.sections.clone(),
        tool_version: info.tool_version.clone(),
        struct_sizes: info.struct_sizes.clone(),
        future_field: 42,
    };
    let header = rmp_serde::to_vec_named(&future).unwrap();

    let mut body = Vec::new();
    body.extend_from_slice(&(header.len() as u64).to_le_bytes());
    body.extend_from_slice(&header);
    body.extend_from_slice(payload);

    let mut bytes = Vec::new();
    bytes.extend_from_slice(&Magic::SourceMap.as_bytes());
    bytes.extend_from_slice(&ARTIFACT_VERSION_NAMED_HEADER.to_le_bytes());
    bytes.extend_from_slice(&FLAG_HAS_TARGET_INFO.to_le_bytes());
    bytes.extend_from_slice(&(body.len() as u64).to_le_bytes());
    bytes.extend_from_slice(&body);

    let artifact = decode_artifact(&bytes, Magic::SourceMap).unwrap();
    assert_eq!(artifact.version, ARTIFACT_VERSION_NAMED_HEADER);
    let decoded = artifact.target_info.expect("v5 header present");
    assert_eq!(decoded, info, "extra field is ignored, known fields intact");
    assert_eq!(artifact.payload, payload);
}

#[test]
fn v4_header_without_struct_sizes_still_decodes() {
    // A v4 file written before `struct_sizes` existed (contract §4
    // append-only rule): the reader must decode it with empty sizes.
    #[derive(serde::Serialize)]
    struct OldTargetInfo {
        target_name: String,
        cpu: CpuKind,
        mem_size: usize,
        exec_addr: Option<usize>,
        bin_references: Vec<BinReference>,
        checksums: Vec<RomChecksum>,
        sections: Vec<Section>,
        tool_version: String,
    }
    let old = OldTargetInfo {
        target_name: "demo".into(),
        cpu: CpuKind::Cpu6809,
        mem_size: 65536,
        exec_addr: None,
        bin_references: vec![],
        checksums: vec![],
        sections: vec![],
        tool_version: "0.11.0".into(),
    };
    let header = bincode::serialize(&old).unwrap();
    let mut body = Vec::new();
    body.extend_from_slice(&(header.len() as u64).to_le_bytes());
    body.extend_from_slice(&header);
    body.extend_from_slice(b"payload");

    let mut bytes = Vec::new();
    bytes.extend_from_slice(&Magic::SourceMap.as_bytes());
    bytes.extend_from_slice(&ARTIFACT_VERSION_WITH_HEADER.to_le_bytes());
    bytes.extend_from_slice(&FLAG_HAS_TARGET_INFO.to_le_bytes());
    bytes.extend_from_slice(&(body.len() as u64).to_le_bytes());
    bytes.extend_from_slice(&body);

    let artifact = decode_artifact(&bytes, Magic::SourceMap).unwrap();
    let info = artifact.target_info.expect("v4 header present");
    assert_eq!(info.target_name, "demo");
    assert!(
        info.struct_sizes.is_empty(),
        "old files decode with empty struct_sizes, got {:?}",
        info.struct_sizes
    );
}

#[test]
fn wrong_magic_is_rejected() {
    let bytes = encode_artifact(Magic::SourceMap, None, b"x");
    assert!(decode_artifact(&bytes, Magic::Symbols).is_err());
}

#[test]
fn truncated_payload_is_rejected() {
    let bytes = encode_artifact(Magic::SourceMap, None, b"0123456789abcdef");
    // Corrupt the declared length to exceed the buffer.
    let mut bad = bytes.clone();
    bad[8..16].copy_from_slice(&u64::MAX.to_le_bytes());
    assert!(decode_artifact(&bad, Magic::SourceMap).is_err());
}

/// Locate the real Stargate artifacts.  With `metadata = true` the
/// derived names write to the build cwd (the game root), not to
/// `roms/`.  Set `STARGATE_DIR` to the build cwd, or default to the
/// developer's checkout.
fn stargate_dir() -> Option<std::path::PathBuf> {
    let dir = std::env::var("STARGATE_DIR")
        .map(std::path::PathBuf::from)
        .ok()
        .or_else(|| {
            let p = std::path::PathBuf::from(env!("HOME")).join("development/stargate");
            p.exists().then_some(p)
        })?;
    // Skip gracefully when the artifacts aren't actually there (the
    // checkout only has them after an in-place build).
    (dir.join("stargate.map").exists() && dir.join("stargate.sym").exists()).then_some(dir)
}

fn read_artifact(dir: &std::path::Path, name: &str) -> Vec<u8> {
    std::fs::read(dir.join(name)).expect("artifact file")
}

#[test]
fn real_stargate_artifacts_load() {
    let Some(dir) = stargate_dir() else {
        eprintln!("skipping: no STARGATE_DIR and no default checkout");
        return;
    };
    let map = read_artifact(&dir, "stargate.map");
    let sym = read_artifact(&dir, "stargate.sym");
    let target = Target::load(&map, &sym).expect("stargate target loads");

    // v4 header: target identity, sections from the in-asm directives,
    // and ROM checksums.
    let info = target.info.as_ref().expect("v4 TargetInfo header present");
    assert_eq!(info.target_name, "stargate");
    assert_eq!(info.cpu, CpuKind::Cpu6809);
    assert_eq!(info.mem_size, 94_208);
    assert!(!info.sections.is_empty(), "sections from in-asm directives");
    assert!(!info.checksums.is_empty(), "ROM checksums present");
    // Stargate has no `exec_addr` directive, so the header (and the
    // payload) carry None.
    assert_eq!(info.exec_addr, None);
    // Struct totals: Proc = link(2)+addr(2)+time(1)+type(1)+cod(1)+
    // data(1)+d1..d7(7) = 15; Smap = 4 fields.
    let sizes: std::collections::HashMap<_, _> = info
        .struct_sizes
        .iter()
        .map(|s| (s.name.as_str(), s.size))
        .collect();
    assert_eq!(sizes.get("Proc"), Some(&15), "struct sizes: {:?}", sizes);
    assert_eq!(sizes.get("Smap"), Some(&4), "struct sizes: {:?}", sizes);

    // The ROM map should have many instruction boundaries.  Note: the
    // writer marks *every* compiled entry as OpCode (data directives
    // included), so this is a hint set, not a pure instruction map.
    assert!(target.source_map.boundaries().len() > 10_000);
    // Boundaries are sorted.
    let bs = target.source_map.boundaries();
    assert!(bs.windows(2).all(|w| w[0].addr <= w[1].addr));

    // The symbol table should be populated (thousands of labels).
    assert!(target.symbols.all().len() > 1_000);

    // Reverse lookup: a boundary's own (file, line) must resolve back
    // to its address.
    let bs = target.source_map.boundaries();
    let probe = &bs[bs.len() / 2];
    let addrs = target.source_map.addresses_for(probe.file_id, probe.line);
    assert!(
        addrs.contains(&probe.addr),
        "line must resolve to its address"
    );
}

#[test]
fn real_sound_artifacts_load() {
    let Some(dir) = stargate_dir() else {
        eprintln!("skipping: no STARGATE_DIR and no default checkout");
        return;
    };
    let map = read_artifact(&dir, "sound.map");
    let sym = read_artifact(&dir, "sound.sym");
    let target = Target::load(&map, &sym).expect("sound target loads");

    let info = target.info.as_ref().expect("v4 TargetInfo header present");
    assert_eq!(info.target_name, "sound");
    assert_eq!(info.cpu, CpuKind::Cpu6800);
    // The sound source uses no in-asm section directives, so zero
    // sections is correct here.

    assert!(target.source_map.boundaries().len() > 500);
    assert!(target.symbols.all().len() > 100);
}

#[test]
fn real_stargate_symbol_queries() {
    let Some(dir) = stargate_dir() else {
        eprintln!("skipping: no STARGATE_DIR and no default checkout");
        return;
    };
    let map = read_artifact(&dir, "stargate.map");
    let sym = read_artifact(&dir, "stargate.sym");
    let target = Target::load(&map, &sym).expect("stargate target loads");

    // A known label: the IRQ vector handler is at $9C6B; look it up by
    // name and by address, and check nearest-symbol queries work.
    if let Some(addr) = target.symbols.address_of("IRQ_HANDLER") {
        assert_eq!(
            target.symbols.exact_symbol(addr).map(|s| s.value),
            Some(addr)
        );
        let at = target.symbols.symbol_at(addr).unwrap();
        assert_eq!(at.value, addr);
    } else {
        // Fall back: every address should still resolve to *some* label.
        let s = target.symbols.symbol_at(0x9c6b).unwrap();
        assert!(s.value <= 0x9c6b);
    }

    // The beam handler lives in RAM code at $15F8; its source mapping
    // should resolve to a real file:line.
    if let Some(loc) = target.source_map.location_at_physical(0x15f8) {
        assert!(!loc.file_name.is_empty());
        assert!(loc.line >= 1);
    }
}
