//! Text-level rmac macro expansion for the JRISC backend.
//!
//! rmac's GPU assembler uses classic text macros:
//!
//! ```text
//!         MACRO sdiv                 ; Divide, keeping sign
//!         move \2,r27
//!         abs \2
//!         div \1,\2
//!         btst #31,r27
//!         jr z,\~sdtt
//!         nop
//!         neg \2
//! \~sdtt:
//!         ENDM
//! ```
//!
//! `\N` substitutes the N-th invocation argument (comma-separated);
//! `\~name` expands to a per-invocation unique label so repeated
//! invocations don't collide. The whole source text is expanded before
//! lexing, so the rest of the pipeline never sees a backslash.

/// Expand every `MACRO` definition and invocation in `text`.
/// Definitions are collected first (order-independent), then every
/// top-level invocation line is replaced by its substituted body.
pub fn expand_jrisc_macros(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();

    // Pass 1: collect macro definitions.
    // `body` maps lowercase macro name -> raw body lines.
    let mut defs: Vec<(String, Vec<String>)> = Vec::new();
    {
        let mut i = 0;
        while i < lines.len() {
            if first_word(lines[i]).is_some_and(|w| w.eq_ignore_ascii_case("macro")) {
                // `MACRO name` — the name is the second word.
                let name = second_word(lines[i])
                    .unwrap_or_default()
                    .to_ascii_lowercase();
                let mut body: Vec<String> = Vec::new();
                i += 1;
                while i < lines.len() {
                    if first_word(lines[i]).is_some_and(|w| w.eq_ignore_ascii_case("endm")) {
                        i += 1;
                        break;
                    }
                    body.push(lines[i].to_string());
                    i += 1;
                }
                if !name.is_empty() {
                    defs.push((name, body));
                }
            } else {
                i += 1;
            }
        }
    }

    if defs.is_empty() {
        return text.to_string();
    }

    // Pass 2: expand invocations, skipping macro-definition regions.
    let mut uniq = 0usize;
    let mut in_def = false;
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    for line in &lines {
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with(';') || trimmed.starts_with('*') {
            if in_def {
                continue;
            }
            out.push((*line).to_string());
            continue;
        }
        let (label, rest_of_line) = strip_label_prefix(trimmed);
        let word = rest_of_line
            .trim_start()
            .split(|c: char| c.is_whitespace())
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();
        if in_def {
            if word == "endm" {
                in_def = false;
            }
            continue;
        }
        if word == "macro" {
            in_def = true;
            continue;
        }
        let Some((_, body)) = defs.iter().find(|(n, _)| *n == word) else {
            out.push((*line).to_string());
            continue;
        };

        // Parse the invocation arguments: everything after the name,
        // split on commas, with any trailing comment stripped.
        let rest_trimmed = rest_of_line.trim_start();
        let rest = &rest_trimmed[word.len().min(rest_trimmed.len())..];
        let rest = rest.split(';').next().unwrap_or("");
        let args: Vec<&str> = rest.split(',').map(|a| a.trim()).collect();

        let uniq_id = uniq;
        uniq += 1;

        for (i, body_line) in body.iter().enumerate() {
            let mut expanded = expand_body_line(body_line, &args, uniq_id);
            if i == 0 {
                if let Some(lbl) = label {
                    expanded = format!("{lbl}: {expanded}");
                }
            }
            out.push(expanded);
        }
    }

    // The `.lines()` iterator strips line terminators; rejoin with the
    // original style. We cannot cheaply detect the original terminator
    // here, so default to '\n' — the lexer treats '\r\n' and '\n' alike
    // (the skip pattern includes both).
    let _ = text.contains("\r\n");
    out.join("\n")
}

/// Substitute `\N` args and `\~label` uniques in one macro body line.
fn expand_body_line(line: &str, args: &[&str], uniq: usize) -> String {
    let mut out = String::with_capacity(line.len() + 8);
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() {
            if chars[i + 1] == '~' {
                // `\~label` — the label name runs to the next
                // non-identifier character.
                let mut j = i + 2;
                while j < chars.len() && (chars[j].is_alphanumeric() || chars[j] == '_') {
                    j += 1;
                }
                let name: String = chars[i + 2..j].iter().collect();
                out.push_str(&format!("{name}__u{uniq}"));
                i = j;
                continue;
            }
            if chars[i + 1].is_ascii_digit() {
                let mut j = i + 1;
                while j < chars.len() && chars[j].is_ascii_digit() {
                    j += 1;
                }
                let n: String = chars[i + 1..j].iter().collect();
                let n: usize = n.parse().unwrap_or(0);
                if (1..=args.len()).contains(&n) {
                    out.push_str(args[n - 1]);
                }
                i = j;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// Split `label: rest` off a line (an optional identifier followed by a
/// colon at the start). Returns the label (without the colon) and the rest.
fn strip_label_prefix(line: &str) -> (Option<&str>, &str) {
    if let Some(colon) = line.find(':') {
        let head = &line[..colon];
        if !head.is_empty()
            && head
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '.')
        {
            return (Some(head), &line[colon + 1..]);
        }
    }
    (None, line)
}

/// The first whitespace-delimited word of a line (comments not stripped).
fn first_word(line: &str) -> Option<&str> {
    line.split_whitespace().next()
}

/// The second whitespace-delimited word of a line.
fn second_word(line: &str) -> Option<&str> {
    line.split_whitespace().nth(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_basic_macro() {
        let src = "\
        MACRO foo
        move \\1,r27
        jr z,\\~here
        nop
\\~here:
        ENDM
        foo r5
        foo r6
";
        let out = expand_jrisc_macros(src);
        assert!(out.contains("move r5,r27"));
        assert!(out.contains("move r6,r27"));
        assert!(out.contains("jr z,here__u0"));
        assert!(out.contains("jr z,here__u1"));
        assert!(out.contains("here__u0:"));
        assert!(out.contains("here__u1:"));
        assert!(!out.contains("MACRO"));
        assert!(!out.contains("ENDM"));
        assert!(!out.contains('\\'), "output: {out}");
    }

    #[test]
    fn multiple_args() {
        let src = "\
        MACRO fmult
        mult \\1,\\2
        ENDM
        fmult x1_n,x2_n
";
        let out = expand_jrisc_macros(src);
        assert!(out.contains("mult x1_n,x2_n"));
    }

    #[test]
    fn leaves_plain_lines_alone() {
        let src = "move r1,r0\nload (r0),r1\n";
        assert_eq!(expand_jrisc_macros(src), src);
    }
}
