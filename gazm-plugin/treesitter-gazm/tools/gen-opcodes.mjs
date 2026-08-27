// Regenerate `opcodes.json` as the union of every CPU's instruction table
// so the single `gazm` grammar parses and highlights all dialects
// (6809 default, 6800, 68000 with size suffixes, 65C02, Z80, JRISC).
//
// Usage: node tools/gen-opcodes.mjs
//
// The per-CPU tables are the crate resources the assembler itself uses:
//   crates/emu6800/resources/opcodes6800.json
//   crates/emu6809/resources/opcodes6809.json
//   crates/emu68000/resources/opcodes68000.json
//   crates/emu6502/resources/opcodes6502.json
//   crates/emuz80/resources/opcodesZ80.json
// plus the JRISC table in gazm/src/cpujrisc/isa.rs (mirrored here).

import { readFileSync, writeFileSync } from 'fs';
import { dirname, join } from 'path';
import { fileURLToPath } from 'url';

const here = dirname(fileURLToPath(import.meta.url));
const pkg = join(here, '..');
const root = join(pkg, '..', '..', '..'); // gazm-plugin -> gazm -> gazm-next
const crates = join(root, 'crates');

function load(path) {
  return JSON.parse(readFileSync(path, 'utf8'));
}

// (name, path, how to extract mnemonic+addr_mode entries)
const tables = [
  {
    cpu: '6809',
    entries: () => load(join(crates, 'emu6809/resources/opcodes6809.json')).instructions,
  },
  {
    cpu: '6800',
    // dict keyed by mnemonic spelling
    entries: () => Object.entries(load(join(crates, 'emu6800/resources/opcodes6800.json')).instructions)
      .map(([action, rec]) => ({ action, addr_mode: rec.addr_mode || 'Inherent' })),
  },
  {
    cpu: '68000',
    entries: () => load(join(crates, 'emu68000/resources/opcodes68000.json')).instructions
      .map((i) => ({
        action: i.mnemonic,
        size: i.size,
        addr_mode: i.form,
      })),
  },
  {
    cpu: '6502',
    entries: () => Object.entries(load(join(crates, 'emu6502/resources/opcodes6502.json')).instructions)
      .map(([action, rec]) => ({ action, addr_mode: rec.addr_mode || 'Implied' })),
  },
  {
    cpu: 'z80',
    entries: () => load(join(crates, 'emuz80/resources/opcodesZ80.json')).instructions
      .map((i) => ({ action: i.mnemonic, addr_mode: i.addr_mode || 'Inherent' })),
  },
];

// The JRISC vocabulary (gazm/src/cpujrisc/isa.rs `OPS`).
const JRISC = [
  'add', 'addc', 'addq', 'addqt', 'sub', 'subc', 'subq', 'subqt', 'neg', 'and',
  'or', 'xor', 'not', 'btst', 'bset', 'bclr', 'mult', 'imult', 'imultn',
  'resmac', 'imacn', 'div', 'abs', 'sh', 'shlq', 'shrq', 'sha', 'sharq',
  'ror', 'rorq', 'rolq', 'cmp', 'cmpq', 'sat8', 'subqmod', 'sat16', 'sat16s',
  'moveq', 'moveta', 'movefa', 'movei', 'loadb', 'loadw', 'loadp', 'sat32s',
  'storeb', 'storew', 'storep', 'mirror', 'jump', 'jr', 'mmult', 'mtoi',
  'normi', 'nop', 'sat24', 'unpack', 'pack', 'addqmod', 'move', 'load',
  'store',
];

const out = {
  actions: {},
  addr_modes: {},
  instructions: [],
};

for (const { cpu, entries } of tables) {
  for (const e of entries()) {
    if (!e.action) continue;
    const action = String(e.action);
    const addr_mode = String(e.addr_mode || 'Inherent');
    out.instructions.push({ action, addr_mode, cpu });
    // 68000 sized spellings: `move.b`, `move.w`, `move.l`.
    if (cpu === '68000' && e.size && e.size !== '-') {
      out.instructions.push({ action: `${action}.${e.size.toLowerCase()}`, addr_mode, cpu });
    }
  }
}
for (const action of JRISC) {
  out.instructions.push({ action, addr_mode: 'Jrisc', cpu: 'jrisc' });
}

// Keep the grammar's 6809-specific addr-mode filters working.
out.instructions.sort((a, b) => a.action.localeCompare(b.action));
writeFileSync(join(pkg, 'opcodes.json'), JSON.stringify(out, null, 2));
console.log(`opcodes.json: ${out.instructions.length} entries`);
