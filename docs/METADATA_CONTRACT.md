# Gazm metadata contract (artifacts + reader library)

Status: **implemented** — the writer side (gazm) emits v4 artifacts with the
`TargetInfo` header behind the `metadata` switch; the reader library
(`gazm-metadata`) is built and tested against this document. The consumer
(the `williams-emu` Stargate emulator's in-app debugger) path-depends on the
reader.

## 1. Use case and intent

`williams-emu` is a cycle-accurate Stargate (Williams 6809) emulator with an
in-app debugger. The debugger needs, from a gazm build of a game:

- **Symbol queries** — resolve an address to the nearest symbol (for
  disassembly annotations), a name to its address (breakpoints by symbol),
  and list symbols in an address range.
- **Source lookups** — map an address to `(file, line)`, map `file:line` to
  addresses (breakpoints in source, step-until-line), and load the actual
  source line text (the map stores absolute paths; they resolve on the
  developer's machine).
- **Instruction-boundary map** — a sorted index of instruction start
  addresses so the disassembler can seed its decode and annotate each
  instruction with its source line.
- **Per-target identity** — a game like Stargate is a *multi-target* build:
  the main board (`stargate`, 6809) and the sound board (`sound`, 6800)
  each produce their own `.map`/`.sym` pair. The artifacts must identify
  which target/CPU they belong to so the debugger can pick the right
  disassembler and keep address spaces separate.

The reader must be **a small library with minimal dependencies** so the
emulator can link it without pulling in the full gazm CLI/assembler stack
(clap, termimad, logos, ariadne, ...). Writer and reader live in the same
repo so the format cannot drift, and the reader is tested by round-tripping
against the writer plus the real Stargate artifacts.

## 2. Non-goals

- No assembler, evaluator, LSP, or rendering functionality in the reader.
- No dependency on the grl-sources/grl-symbols types leaking into the
  reader's *public* API — the reader owns plain data types
  (`Symbol`, `SourceLocation`, `InstructionBoundary`, `Section`).
  The grl crates may be used internally for deserialization only.
- No new filesystem behavior beyond explicit, caller-driven source loads.

## 3. Artifact envelope (current, v3)

Both artifacts share one envelope written by `encode_artifact` in
`gazm/src/assembler/writers.rs`. Little-endian:

| Bytes | Meaning |
| --- | --- |
| 0..4 | Magic: `GZSY` for symbols, `GZMP` for source maps |
| 4..6 | Format version (`u16`), **currently `3`** (the older doc said 1; files are 3) |
| 6..8 | Reserved flags (`u16`, currently `0`) |
| 8..16 | Payload length (`u64`) |
| 16.. | bincode 1.x payload |

Payloads:

- `.map` (`GZMP`): `SourceDatabase` (grl-sources). After
  `bincode::deserialize`, call `rebuild_indexes()` (the derived serde
  impl skips runtime caches).
- `.sym` (`GZSY`): `SymbolTree` (grl-symbols) via its manual
  `Serialize`/`Deserialize` impl (`Seriablizable` mirror). Deserializing
  rebuilds the internal tree.

Consumers reject unknown magic, versions, flags, or payload lengths before
bincode.

## 4. Artifact envelope: `TargetInfo` header (v4 bincode, v5 named map)

Same envelope, plus an optional header block inserted between the fixed
16 bytes and the payload. Bit 0 of the reserved `flags` u16 = header
present. Headerless files stay version **3** so existing consumers see
no change. Header-bearing files are version **4** (frozen bincode layout)
or **5** (rmp-serde named map — the current format).

```rust
pub struct TargetInfo {
    pub target_name: String,        // e.g. "stargate", "sound"
    pub cpu: CpuKind,               // serde enum: 6809 / 6800 / 6502 / 65c02 / Z80
    pub mem_size: usize,            // e.g. 94208, 65536
    pub exec_addr: Option<usize>,   // entry point (already in SourceDatabase; lift it)
    pub bin_references: Vec<BinReference>,  // { file: PathBuf, addr: usize }
    pub checksums: Vec<RomChecksum>,        // { name, addr, size, sha1 }
    pub sections: Vec<Section>,             // named memory regions (see §6)
    pub tool_version: String,       // gazm version that wrote the file
    pub struct_sizes: Vec<StructSize>,      // { name, size } per struct
}
```

Compatibility: v3 files (no header, flags = 0) must still load; the reader
returns `TargetInfo = None` for them. v4 files (gazm ≤ 0.11.0, bincode,
positional, no `struct_sizes`) must still load — the reader keeps the
frozen v4 layout as a compat struct and decodes those with empty
`struct_sizes`.

**Evolvability (v5):** the header is serialized with
`rmp_serde::to_vec_named` — a *named map*, not positional bincode. New
fields can be added freely: readers **default missing fields** (old file,
new reader) and **ignore unknown fields** (new file, old reader), so
neither direction breaks. Do not reorder or remove existing field names.
(Payloads stay bincode — they are large and rarely change shape.)

`struct_sizes` is a plain name→size list (`Proc -> 15`); the debugger can
ignore it until it displays structs.

## 5. Config: one switch for "write metadata or nothing"

One boolean per target:

```toml
[[targets]]
name = "stargate"
metadata = true          # or omit/false -> write nothing
```

- `metadata = true` writes the whole bundle: `<target>.map`, `<target>.sym`
  (names derived from the target name; the files land relative to the
  build directory). The v4 `TargetInfo` header is part of the bundle.
- `metadata = false`/absent writes nothing — no map, no syms, no header.
- The migration is complete: the old per-target `source-mapping`/
  `syms-file` explicit paths are **removed** (they were back-compat during
  the migration and are gone as of the 0.10.x cleanup). Stargate and the
  sound board both use `metadata = true`.
- `--json-output`/`--pretty-json` remain orthogonal formatting toggles for
  the same bundle.
- `as6809_sym` removed (it was unused).

## 6. Sections: from the assembler, not the config

The `[sections]` TOML block in `gazm.toml` is **dead config**: nothing
parses it (`TomlConfig`/`Opts` have no sections field; `SectionToml` in
`sections.rs` is unreferenced; `Sections::from_file` is never called).
Remove it and `SectionToml`.

The real sections come from the in-source `section` directive, e.g.:

```asm
;; src/core/main.gazm
section lo_rom
section dp_ram, start = BGSAV
```

`Section(name)` is a real AST node; the sizer tracks
`SectionDescriptor { name, logical_range, physical_range, access_type }`
(already serde-able). Persist the final descriptors into the v4 header's
`sections` field. This gives the debugger named memory regions and
read-only/read-write hints for free.

## 7. Reader library: `gazm-metadata`

A new small crate in this workspace. Dependencies: `bincode`, `serde`,
and internally `grl-sources` + `grl-symbols` (deserialization only).
No CLI, no assembler deps.

Public API (plain owned types; no grl types leak):

```rust
// envelope
pub fn decode_artifact(bytes: &[u8], expected: Magic)
    -> Result<Artifact<TargetInfo>>;      // validates magic/version/flags/len

// per-target bundle
pub struct Target {
    pub info: Option<TargetInfo>,
    pub source_map: SourceMap,
    pub symbols: Symbols,
}
impl Target {
    pub fn load(map_bytes: &[u8], sym_bytes: &[u8]) -> Result<Target>;
    // validates both envelopes; if both have TargetInfo, they must agree
    // on target_name + cpu (+ version)
}

// queries
pub struct SourceMap { /* wraps SourceDatabase + rebuilt indexes */ }
impl SourceMap {
    pub fn location_at(&self, addr: usize) -> Option<SourceLocation>; // file, line
    pub fn instruction_boundaries(&self) -> &[InstructionBoundary];   // sorted by addr
    pub fn boundary_at(&self, addr: usize) -> Option<&InstructionBoundary>;
    pub fn addresses_for(&self, file: FileId, line: usize) -> Vec<usize>; // reverse
    pub fn source_text(&self, loc: &SourceLocation) -> Option<String>;    // on-demand load
}
pub struct Symbols { /* wraps SymbolTree */ }
impl Symbols {
    pub fn symbol_at(&self, addr: usize) -> Option<&Symbol>;        // nearest at-or-before
    pub fn exact_symbol(&self, addr: usize) -> Option<&Symbol>;
    pub fn address_of(&self, name: &str) -> Option<usize>;
    pub fn symbols_in_range(&self, start: usize, end: usize) -> Vec<&Symbol>;
}
pub struct InstructionBoundary { pub addr: usize, pub len: usize, pub file: FileId, pub line: usize }
```

Multi-target: a `Project` type holds `Vec<Target>` keyed by name; the
debugger loads one `Target` per emulated CPU. Each target's address space
is independent.

## 8. Tests the reader must pass

- Round-trip: `encode_artifact` (with and without `TargetInfo`) ->
  `decode_artifact` -> identical data.
- Real-file fixtures: `stargate.map`/`stargate.sym` (6809, multi-section)
  and `sound.map`/`sound.sym` (6800) must load; spot-check known symbols
  (e.g. the reset/IRQ vectors) and known source locations.
- v3 (no header) and v4 (header) both load; mismatched map/sym pairs are
  rejected when both carry headers.
- `instruction_boundaries()` is sorted; every boundary's source line
  resolves.

## 9. Implementation order (writer side)

1. ✅ Remove dead `[sections]` config + `SectionToml`; wire in-asm sections
   into the artifact payload (v4 header). `sections.rs` now holds only
   `SectionDescriptor`; the sizer persists the final descriptors into
   `AsmOut.sections`.
2. ✅ Add `TargetInfo` + optional header to `encode_artifact`; version is 4
   with the header, 3 without (so v3 output stays byte-compatible with
   existing consumers); flag bit 0 advertises the header.
3. ✅ Replace `source-mapping`/`syms-file` with the single `metadata`
   switch (explicit paths kept for back-compat during migration).
4. Expose `gazm-metadata` from this workspace; williams-emu path-depends
   on it and consumes it in the debugger.
