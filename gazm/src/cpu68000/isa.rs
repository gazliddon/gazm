#![deny(unused_imports)]
//! The 68000 instruction table: emu68000's MAME-validated entries, indexed
//! for assembly by (mnemonic, size). The entries are sorted by mask
//! specificity (the emulator's order), so "first matching row" reproduces
//! the disassembler's — and MadMac's — encoding choice.

use std::collections::HashMap;
use std::sync::OnceLock;

use emu68000::isa::{Dbase, Form, Size};

/// (mnemonic, size-or-none) -> row indices into the sorted entries.
pub struct Table {
    pub db: &'static Dbase,
    by_key: HashMap<(String, Option<Size>), Vec<usize>>,
    /// bare branch mnemonics (form Bcc8) gain a ".w" spelling for the
    /// 16-bit form.
    pub branch_w: Vec<&'static str>,
    /// Immediate-operand aliases: `sub #imm,<ea>` assembles as `subi`
    /// (MadMac/rmac behaviour). Keyed by the base mnemonic.
    pub imm_aliases: &'static [(&'static str, &'static str)],
}

pub static TABLE: OnceLock<Table> = OnceLock::new();

/// The initialised table, built on first use.
pub fn table() -> &'static Table {
    TABLE.get_or_init(Table::build)
}

/// Mnemonics whose immediate forms use the `*i` spelling (MadMac), and
/// whose An-destination forms use the `*a` spelling.
static IMM_ALIASES: &[(&str, &str)] = &[
    ("sub", "subi"),
    ("add", "addi"),
    ("cmp", "cmpi"),
    ("and", "andi"),
    ("or", "ori"),
    ("eor", "eori"),
];

/// Mnemonics whose An-destination form uses the `*a` spelling:
/// `cmp a1,a0` assembles as `cmpa.l A1,A0` (MadMac/rmac).
static AN_ALIASES: &[(&str, &str)] = &[("add", "adda"), ("sub", "suba"), ("cmp", "cmpa")];

fn size_key(size: Size) -> Option<Size> {
    match size {
        Size::None => None,
        s => Some(s),
    }
}

impl Table {
    fn build() -> Table {
        let db = Dbase::get();
        let mut by_key: HashMap<(String, Option<Size>), Vec<usize>> = HashMap::new();
        let mut branch_w: Vec<&'static str> = Vec::new();
        for (i, insn) in db.entries.iter().enumerate() {
            let key = (insn.mnemonic.to_string(), size_key(insn.size));
            by_key.entry(key).or_default().push(i);
            if insn.form == Form::Bcc8 {
                branch_w.push(insn.mnemonic);
            }
        }
        Table {
            db,
            by_key,
            branch_w,
            imm_aliases: IMM_ALIASES,
        }
    }

    pub fn row(&self, idx: usize) -> &'static emu68000::isa::Insn {
        &self.db.entries[idx]
    }

    /// Rows for the immediate-operand alias of `mn` (e.g. `sub` ->
    /// `subi`): used when `mn`'s own rows don't match an immediate
    /// operand.
    pub fn alias_rows(&self, mn: &str, suffix: Option<Size>) -> Option<Vec<usize>> {
        let alias = IMM_ALIASES
            .iter()
            .find(|(base, _)| *base == mn)
            .map(|(_, alias)| *alias)?;
        self.lookup(alias, suffix)
    }

    /// Rows for the An-destination alias of `mn` (e.g. `cmp` -> `cmpa`):
    /// used when `mn`'s own rows don't match an An destination.
    pub fn an_alias_rows(&self, mn: &str, suffix: Option<Size>) -> Option<Vec<usize>> {
        let alias = AN_ALIASES
            .iter()
            .find(|(base, _)| *base == mn)
            .map(|(_, alias)| *alias)?;
        self.lookup(alias, suffix)
    }

    /// The 16-bit branch row for a Bcc8 mnemonic (used by the sane-mode
    /// auto-extension of out-of-range short branches).
    pub fn branch_w_row(&self, mn: &str) -> Option<usize> {
        self.db
            .entries
            .iter()
            .position(|i| i.mnemonic == mn && i.form == Form::Bcc16)
    }

    /// Candidate rows for a mnemonic + optional size suffix. A bare
    /// mnemonic resolves to the size-"-" rows (inherent/branch forms), or
    /// the `.w` rows when the mnemonic has sized forms only (MadMac's
    /// word default). A `.w` suffix on a size-"-" mnemonic selects the
    /// 16-bit branch forms. Bare branch mnemonics prefer the 8-bit form
    /// (MadMac auto-extends at size time).
    pub fn lookup(&self, mn: &str, suffix: Option<Size>) -> Option<Vec<usize>> {
        let key = |s: Option<Size>| (mn.to_string(), s);
        match suffix {
            None => {
                let mut rows = self
                    .by_key
                    .get(&key(None))
                    .cloned()
                    .or_else(|| self.by_key.get(&key(Some(Size::W))).cloned())
                    // L-only mnemonics (moveq) default to long
                    .or_else(|| self.by_key.get(&key(Some(Size::L))).cloned())?;
                // stable partition: the 8-bit branch form first
                rows.sort_by_key(|&i| !matches!(self.row(i).form, Form::Bcc8));
                Some(rows)
            }
            Some(s) => {
                if let Some(rows) = self.by_key.get(&key(Some(s))) {
                    return Some(rows.clone());
                }
                // `.w` on a branch mnemonic -> the 16-bit form
                let branch_w: Vec<usize> = self
                    .by_key
                    .get(&key(None))
                    .map(|rows| {
                        rows.iter()
                            .copied()
                            .filter(|&i| matches!(self.row(i).form, Form::Bcc16 | Form::Dbcc))
                            .collect()
                    })
                    .unwrap_or_default();
                if !branch_w.is_empty() {
                    return Some(branch_w);
                }
                // A size suffix on a size-less mnemonic (`btst.b`,
                // MadMac) is just an annotation: fall back to the bare
                // rows.
                self.by_key.get(&key(None)).cloned()
            }
        }
    }
}

/// The mnemonic spellings the identifier classifier recognises: every
/// bare mnemonic, every `mnemonic.size` spelling, and `.w` for the
/// 8-bit branch mnemonics (the 16-bit spelling).
pub static MNEMONICS: OnceLock<std::collections::HashSet<String>> = OnceLock::new();

pub fn mnemonic_set() -> &'static std::collections::HashSet<String> {
    MNEMONICS.get_or_init(|| {
        let db = Dbase::get();
        let mut set = std::collections::HashSet::new();
        for insn in &db.entries {
            set.insert(insn.mnemonic.to_string());
            if insn.size != Size::None {
                set.insert(format!("{}.{}", insn.mnemonic, insn.size.suffix()));
            } else {
                // MadMac accepts a size suffix on any mnemonic; for
                // size-less rows the suffix is an annotation (`btst.b`,
                // `bra.w`).
                set.insert(format!("{}.b", insn.mnemonic));
                set.insert(format!("{}.w", insn.mnemonic));
                set.insert(format!("{}.l", insn.mnemonic));
            }
            if insn.form == Form::Bcc8 {
                set.insert(format!("{}.w", insn.mnemonic));
            }
        }
        set
    })
}
