//! Conservative source formatter for Gazm files.

use std::{
    collections::{BTreeSet, HashMap},
    fs,
    path::{Path, PathBuf},
};

use grl_sources::{AsmSource, SourceFile};

use crate::{
    cpu6809::frontend::{AddrModeParseType, IndexParseType, NodeKind6809},
    error::GResult,
    frontend::{AstNodeKind, CpuSpecific, Node, TokenizeRequest},
    opts::Opts,
};

// Columns are zero-based internally.  This places every opcode at visible
// column 17, whether or not the line has a label.
const OPCODE_COLUMN: usize = 16;
const COMMENT_COLUMN: usize = 50;

/// Check whether two AST trees are semantically identical, ignoring source
/// span/location coordinates.
pub fn ast_semantically_equal(a: &Node, b: &Node) -> bool {
    let items_match = match (&a.item, &b.item) {
        (AstNodeKind::TokenizedFile(..), AstNodeKind::TokenizedFile(..)) => true,
        (item_a, item_b) => item_a == item_b,
    };
    if !items_match {
        return false;
    }
    if a.children.len() != b.children.len() {
        return false;
    }
    a.children
        .iter()
        .zip(b.children.iter())
        .all(|(child_a, child_b)| ast_semantically_equal(child_a, child_b))
}

pub fn format_file(path: &Path, opts: &Opts) -> GResult<()> {
    let source =
        fs::read_to_string(path).map_err(|e| format!("Unable to read {}: {e}", path.display()))?;
    if source.trim().is_empty() {
        return Ok(());
    }

    let source_file_orig = SourceFile::new(path, &source, AsmSource::FromStr);
    let orig_res = TokenizeRequest::for_single_source_file(source_file_orig, opts).to_result();

    let formatted = format_text(&source);
    if formatted == source {
        return Ok(());
    }

    // Safety invariant: formatting must NEVER alter the semantic AST
    if !orig_res.has_errors() {
        let source_file_fmt = SourceFile::new(path, &formatted, AsmSource::FromStr);
        let fmt_res = TokenizeRequest::for_single_source_file(source_file_fmt, opts).to_result();
        if fmt_res.has_errors() || !ast_semantically_equal(&orig_res.node, &fmt_res.node) {
            return Err(crate::error::GazmErrorKind::Misc(format!(
                "Formatter invariant failure for {}: formatted AST differs from original AST",
                path.display()
            )));
        }
    }

    fs::write(path, formatted).map_err(|e| format!("Unable to write {}: {e}", path.display()))?;
    Ok(())
}

/// Format the project file and every source reachable through its `include`
/// directives. Files are visited once, so shared includes are safe and
/// unrelated files in the source directory are left untouched.
pub fn format_project(project: &Path, opts: &Opts) -> GResult<()> {
    let mut files = BTreeSet::new();
    collect_includes(project, &mut files)?;
    for file in files {
        format_file(&file, opts)?;
    }
    Ok(())
}

fn collect_includes(path: &Path, files: &mut BTreeSet<PathBuf>) -> GResult<()> {
    let path = path.to_path_buf();
    if !files.insert(path.clone()) {
        return Ok(());
    }
    let source =
        fs::read_to_string(&path).map_err(|e| format!("Unable to read {}: {e}", path.display()))?;
    for line in source.lines() {
        let code = line.split(';').next().unwrap_or_default().trim();
        let Some(rest) = code.strip_prefix("include") else {
            continue;
        };
        if !rest.chars().next().is_some_and(char::is_whitespace) {
            continue;
        }
        let name = rest.split_whitespace().next().unwrap_or_default();
        let name = name.trim_matches(['"', '\'']);
        if name.is_empty() {
            continue;
        }
        collect_includes(&path.parent().unwrap_or(Path::new(".")).join(name), files)?;
    }
    Ok(())
}

pub fn format_text(source: &str) -> String {
    let source_file = SourceFile::new("<format>", source, AsmSource::FromStr);
    let result = TokenizeRequest::for_single_source_file(source_file, &Opts::default()).to_result();

    let mut line_modes = HashMap::new();
    if !result.has_errors() {
        for node_info in result.node.iter() {
            let item = &node_info.node.item;
            let line = node_info.node.ctx.line();
            if let AstNodeKind::TargetSpecific(CpuSpecific::Cpu6809(NodeKind6809::OpCode(
                _id,
                amode,
            ))) = item
            {
                line_modes.insert(line, *amode);
            }
        }
    }

    let had_final_newline = source.ends_with('\n');
    let mut depth = 0;
    let mut formatted_lines = Vec::new();
    for (line_idx, line) in source.lines().enumerate() {
        let (formatted, next_depth) = format_line(line, line_modes.get(&line_idx).copied(), depth);
        formatted_lines.push(formatted);
        depth = next_depth;
    }
    let mut output = formatted_lines.join("\n");
    if had_final_newline {
        output.push('\n');
    }
    output
}

fn format_line(line: &str, amode: Option<AddrModeParseType>, depth: usize) -> (String, usize) {
    let trimmed = line.trim();
    if trimmed.is_empty()
        || trimmed.starts_with(';')
        || trimmed.starts_with("//")
        || trimmed.starts_with("/*")
        || trimmed.starts_with('*')
    {
        return (trimmed.to_owned(), depth);
    }

    let (code, comment) = split_comment(trimmed);
    let code = code.trim();
    if code.is_empty() {
        return (comment.unwrap_or_default().to_owned(), depth);
    }

    let open_braces = code.matches('{').count();
    let close_braces = code.matches('}').count();
    let leading_close = if code.starts_with('}') { 1 } else { 0 };

    let effective_depth = depth.saturating_sub(leading_close);
    let base_indent = OPCODE_COLUMN + effective_depth * 4;

    let (label, statement) = match code.split_once(':') {
        Some((candidate, rest))
            if !candidate.chars().any(char::is_whitespace) && !candidate.starts_with("::") =>
        {
            (Some(format!("{}:", candidate.trim())), rest.trim())
        }
        _ => (None, code),
    };
    let mut fields = statement.splitn(2, char::is_whitespace);
    let mnemonic = fields.next().unwrap_or_default();
    let raw_operand = fields.next().unwrap_or_default().trim();

    let operand = match amode {
        Some(AddrModeParseType::RegisterSet) | Some(AddrModeParseType::RegisterPair(..)) => {
            raw_operand.to_ascii_lowercase()
        }
        Some(AddrModeParseType::Indexed(imode, _)) => lowercase_indexed_text(raw_operand, imode),
        _ => raw_operand.to_owned(),
    };

    let mut result = String::new();
    if let Some(label) = label {
        result.push_str(&label);
        if label.len() < base_indent {
            result.push_str(&" ".repeat(base_indent - label.len()));
        } else if mnemonic.eq_ignore_ascii_case("equ")
            || mnemonic.eq_ignore_ascii_case("set")
            || mnemonic == "="
        {
            result.push(' ');
        } else {
            result.push('\n');
            result.push_str(&" ".repeat(base_indent));
        }
    } else {
        result.push_str(&" ".repeat(base_indent));
    }
    result.push_str(mnemonic);
    if !operand.is_empty() {
        result.push(' ');
        result.push_str(&operand);
    }
    if let Some(comment) = comment {
        let line_len = result.rsplit('\n').next().unwrap_or_default().len();
        if line_len < COMMENT_COLUMN {
            result.push_str(&" ".repeat(COMMENT_COLUMN - line_len));
        } else {
            result.push_str("  ");
        }
        result.push_str(comment.trim());
    }

    let next_depth = (depth + open_braces).saturating_sub(close_braces);
    (result.trim_end().to_owned(), next_depth)
}

fn lowercase_indexed_text(raw: &str, imode: IndexParseType) -> String {
    match imode {
        IndexParseType::Zero(..)
        | IndexParseType::PostInc(..)
        | IndexParseType::PostIncInc(..)
        | IndexParseType::PreDec(..)
        | IndexParseType::PreDecDec(..)
        | IndexParseType::AddA(..)
        | IndexParseType::AddB(..)
        | IndexParseType::AddD(..) => raw.to_ascii_lowercase(),
        IndexParseType::ConstantOffset(..)
        | IndexParseType::Constant5BitOffset(..)
        | IndexParseType::ConstantByteOffset(..)
        | IndexParseType::ConstantWordOffset(..)
        | IndexParseType::PCOffset
        | IndexParseType::PcOffsetWord(..)
        | IndexParseType::PcOffsetByte(..) => {
            if let Some((expr_part, reg_part)) = raw.rsplit_once(',') {
                format!("{expr_part},{}", reg_part.to_ascii_lowercase())
            } else {
                raw.to_owned()
            }
        }
        IndexParseType::ExtendedIndirect => raw.to_owned(),
    }
}

fn split_comment(line: &str) -> (&str, Option<&str>) {
    let mut in_string = false;
    let mut escaped = false;
    for (index, ch) in line.char_indices() {
        if ch == '"' && !escaped {
            in_string = !in_string;
        }
        if ch == ';' && !in_string {
            return (&line[..index], Some(&line[index..]));
        }
        escaped = ch == '\\' && !escaped;
        if ch != '\\' {
            escaped = false;
        }
    }
    (line, None)
}

#[cfg(test)]
mod tests {
    use super::format_text;

    #[test]
    fn aligns_labels_and_opcodes() {
        assert_eq!(
            format_text("start: lda #1\n    rts\n"),
            "start:          lda #1\n                rts\n"
        );
    }

    #[test]
    fn preserves_comments_and_strings() {
        assert_eq!(
            format_text("fcc \"a;b\" ; comment\n\n"),
            "                fcc \"a;b\"                         ; comment\n\n"
        );
    }

    #[test]
    fn aligns_inline_comments_at_column_51() {
        let formatted = format_text("        lda #1 ; comment\n");
        assert_eq!(formatted.as_bytes()[50], b';');
    }

    #[test]
    fn puts_long_labels_on_their_own_line() {
        assert_eq!(
            format_text("scan_next_object: ldx ,X\n"),
            "scan_next_object:\n                ldx ,x\n"
        );
    }

    #[test]
    fn lowercases_registers_but_not_symbols_or_strings() {
        assert_eq!(
            format_text("        STX proc::data,U ; PC stays prose\n"),
            "                STX proc::data,u                  ; PC stays prose\n"
        );
        assert_eq!(
            format_text("        lda #PC + SYMBOL + \"PC\"\n"),
            "                lda #PC + SYMBOL + \"PC\"\n"
        );
        assert_eq!(
            format_text("        pshs A,B,X,Y\n"),
            "                pshs a,b,x,y\n"
        );
        assert_eq!(
            format_text("        tfr A,B\n"),
            "                tfr a,b\n"
        );
    }

    #[test]
    fn preserves_character_literals_and_symbols_in_directives() {
        let input = "                FCB $5b^$5a,  'C'^$5a, $5C^$5a\n                FCB 'I'^$5a,  'A'^$5a, 'M'^$5a,'S'^$5a , ' '^$5a\n";
        let expected = "                FCB $5b^$5a,  'C'^$5a, $5C^$5a\n                FCB 'I'^$5a,  'A'^$5a, 'M'^$5a,'S'^$5a , ' '^$5a\n";
        assert_eq!(format_text(input), expected);
    }

    #[test]
    fn format_preserves_ast_semantics() {
        use crate::{frontend::TokenizeRequest, opts::Opts};
        use grl_sources::{AsmSource, SourceFile};

        let cases = [
            "start: lda #1\n    rts\n",
            "                FCB $5b^$5a,  'C'^$5a, $5C^$5a\n                FCB 'I'^$5a,  'A'^$5a, 'M'^$5a,'S'^$5a , ' '^$5a\n",
            "        STX proc::data,U ; PC stays prose\n",
            "        lda #PC + SYMBOL + 'P'\n",
            "        pshs A,B,X,Y\n",
            "        tfr A,B\n",
            "scan_next_object:\n                ldx ,X\n",
            "FOO_A:          equ 42\n                lda #FOO_A\n",
            "st_controls_inactive: equ 1 << 6\n",
            "very_long_custom_label_name: equ $1234\n",
        ];

        let opts = Opts::default();
        for code in cases {
            let orig_file = SourceFile::new("<orig>", code, AsmSource::FromStr);
            let orig_res = TokenizeRequest::for_single_source_file(orig_file, &opts).to_result();
            assert!(!orig_res.has_errors(), "Parsing failed for case:\n{code}");

            let formatted = format_text(code);
            let fmt_file = SourceFile::new("<fmt>", &formatted, AsmSource::FromStr);
            let fmt_res = TokenizeRequest::for_single_source_file(fmt_file, &opts).to_result();
            assert!(
                super::ast_semantically_equal(&orig_res.node, &fmt_res.node),
                "AST mutated by formatter on input:\n{code}\nFormatted:\n{formatted}"
            );
        }
    }

    #[test]
    fn aligns_equates_in_column() {
        let input = ".DYN_OPTR: EQU proc::data\n.VEL: EQU proc::data+2\n.SHOT: EQU proc::data+3\n";
        let expected = ".DYN_OPTR:      EQU proc::data\n.VEL:           EQU proc::data+2\n.SHOT:          EQU proc::data+3\n";
        assert_eq!(format_text(input), expected);
    }

    #[test]
    fn formats_grouped_imports_with_indentation() {
        let input = "import ::core::{\nSLEEP, COLTAB,\nGETOB, OFSHIT,\n}\n";
        let expected = "                import ::core::{\n                    SLEEP, COLTAB,\n                    GETOB, OFSHIT,\n                }\n";
        assert_eq!(format_text(input), expected);
    }
}
