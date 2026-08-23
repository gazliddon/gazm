# Symbol and source-map artifacts

Gazm writes the symbol table and source map as versioned bincode artifacts by
default. Use `--json-output` to write JSON instead (`--pretty-json` additionally
formats that JSON with indentation).

Binary artifacts have this little-endian header before the bincode payload:

| Bytes | Meaning |
| --- | --- |
| 0..4 | Magic: `GZSY` for symbols, `GZMP` for source maps |
| 4..6 | Format version (`u16`), currently `1` |
| 6..8 | Reserved flags (`u16`, currently `0`) |
| 8..16 | Payload length (`u64`) |
| 16.. | bincode 1.x payload |

Consumers should reject unknown magic, versions, flags, or payload lengths
before attempting bincode deserialization.
