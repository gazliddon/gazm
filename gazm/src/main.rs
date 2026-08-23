use gazm::{
    assembler::Assembler,
    cli::{parse_command_line, styling::get_banner},
    error::{ErrorCollectorTrait, GazmErrorKind},
    frontend,
    messages::{self, format_count, format_duration, TimingReport},
    opts::{BuildType, Opts},
    status_mess,
};
use serde::Serialize;
use std::io::Write;

fn stage_time(report: &TimingReport, name: &str) -> Option<std::time::Duration> {
    report
        .stages
        .iter()
        .find(|(stage, _)| stage.starts_with(name))
        .map(|(_, duration)| *duration)
}

fn print_timing_report(rows: &[(Opts, TimingReport)]) {
    if rows.is_empty() || rows.iter().all(|(_, report)| report.stages.is_empty()) {
        return;
    }
    eprintln!(" INFO Timing");
    eprintln!(
        "       {:<12} {:>5} {:>10} {:>10} {:>10} {:>10} {:>10} {:>10} {:>10} {:>10}",
        "Target",
        "Files",
        "Lines",
        "Cache",
        "Rate",
        "Lexing",
        "Semantic",
        "Compile",
        "Writing",
        "Total"
    );
    for (opts, report) in rows {
        let target = opts.target_name.as_deref().unwrap_or("target");
        let lexing = stage_time(report, "Lexing");
        let semantic = stage_time(report, "Semantic analysis");
        let compile = stage_time(report, "Compiling");
        let writing = stage_time(report, "Writing files");
        let total: std::time::Duration = report.stages.iter().map(|(_, time)| *time).sum();
        let rate = lexing
            .filter(|time| time.as_secs_f64() > 0.0)
            .map(|time| {
                format!(
                    "{:.1}k/s",
                    report.source_lines as f64 / time.as_secs_f64() / 1_000.0
                )
            })
            .unwrap_or_else(|| "-".into());
        eprintln!(
            "       {:<12} {:>5} {:>10} {:>10} {:>10} {:>10} {:>10} {:>10} {:>10} {:>10}",
            target,
            format_count(report.source_files),
            format_count(report.source_lines),
            format!("{}/{}", report.token_cache_hits, report.token_cache_misses),
            rate,
            lexing.map(format_duration).unwrap_or_else(|| "-".into()),
            semantic.map(format_duration).unwrap_or_else(|| "-".into()),
            compile.map(format_duration).unwrap_or_else(|| "-".into()),
            writing.map(format_duration).unwrap_or_else(|| "-".into()),
            format_duration(total)
        );
    }
}

#[derive(Serialize)]
struct TimingRecord<'a> {
    timestamp_unix_ms: u128,
    target: &'a str,
    source_files: usize,
    source_lines: usize,
    token_cache_hits: usize,
    token_cache_misses: usize,
    lexing_ms: Option<f64>,
    semantic_ms: Option<f64>,
    compile_ms: Option<f64>,
    writing_ms: Option<f64>,
    total_ms: f64,
}

fn append_timing_history(
    path: &std::path::Path,
    rows: &[(Opts, TimingReport)],
) -> std::io::Result<()> {
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    for (opts, report) in rows {
        if report.stages.is_empty() {
            continue;
        }
        let duration_ms =
            |name: &str| stage_time(report, name).map(|duration| duration.as_secs_f64() * 1_000.0);
        let total_ms = report
            .stages
            .iter()
            .map(|(_, duration)| duration.as_secs_f64())
            .sum::<f64>()
            * 1_000.0;
        let record = TimingRecord {
            timestamp_unix_ms: timestamp,
            target: opts.target_name.as_deref().unwrap_or("target"),
            source_files: report.source_files,
            source_lines: report.source_lines,
            token_cache_hits: report.token_cache_hits,
            token_cache_misses: report.token_cache_misses,
            lexing_ms: duration_ms("Lexing"),
            semantic_ms: duration_ms("Semantic analysis"),
            compile_ms: duration_ms("Compiling"),
            writing_ms: duration_ms("Writing files"),
            total_ms,
        };
        serde_json::to_writer(&mut file, &record).map_err(std::io::Error::other)?;
        file.write_all(b"\n")?;
    }
    Ok(())
}

fn do_build(opts: &Opts, show_banner: bool) -> Result<(), GazmErrorKind> {
    let mut asm = Assembler::new(opts.clone());

    match opts.build_type {
        BuildType::Test => {
            status_mess!("Testing! {}", opts.project_file.to_string_lossy());
            frontend::test_it(opts);
            status_mess!("Done!");
        }

        BuildType::Format => {
            let project_path = if let Some(dir) = &opts.assemble_dir {
                dir.join(&opts.project_file)
            } else {
                opts.project_file.clone()
            };
            status_mess!("Formatting {}", project_path.to_string_lossy());
            if opts.format_project {
                gazm::fmt::format_project(&project_path, opts)?;
            } else {
                gazm::fmt::format_file(&project_path, opts)?;
            }
        }

        BuildType::Lsp => {
            unreachable!("LSP is dispatched once for all configured targets")
        }

        // Build of check to see if build is okay
        BuildType::Build | BuildType::Check => {
            if show_banner {
                status_mess!("{}", get_banner());
            }
            let target_name = opts.target_name.as_deref().unwrap_or_else(|| {
                opts.project_file
                    .file_stem()
                    .and_then(|name| name.to_str())
                    .unwrap_or("target")
            });
            status_mess!("Target: {} ({:?})", target_name, opts.cpu);
            let target_span = tracing::info_span!("target", name = target_name, cpu = ?opts.cpu);
            let _target_guard = target_span.enter();
            status_mess!("Verbosity: {:?}", &opts.verbose);

            if opts.no_async {
                status_mess!("Async: NO ASYNC");
            }

            asm.assemble()?;

            // Only write outputs if this is of buildtype Build
            if opts.build_type == BuildType::Build {
                asm.write_outputs()?;
            }
        }
    };

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = parse_command_line();
    let trace_file = matches.get_one::<std::path::PathBuf>("trace-file").cloned();
    let timings_file = matches
        .get_one::<std::path::PathBuf>("timings-file")
        .cloned();

    let opts = Opts::from_arg_matches(matches)?;

    let is_lsp = opts.first().is_some_and(|o| o.build_type == BuildType::Lsp);
    let mut _trace_guard = None;
    if let Some(first) = opts.first() {
        if !is_lsp {
            _trace_guard = messages::init(first, trace_file.as_deref());
        }
        if matches!(first.build_type, BuildType::Build | BuildType::Check) {
            status_mess!("{}", get_banner());
        }
    }

    if is_lsp {
        gazm::lsp::do_lsp(&opts)?;
        return Ok(());
    }

    let results = if opts.len() > 1
        && opts
            .first()
            .is_some_and(|o| matches!(o.build_type, BuildType::Build | BuildType::Check))
    {
        std::thread::scope(|scope| {
            let handles = opts.into_iter().map(|opts| {
                scope.spawn(move || {
                    let (ret, output, timing) = messages::capture(|| do_build(&opts, false));
                    (opts, ret, output, timing)
                })
            });
            handles
                .map(|handle| handle.join().expect("target build thread panicked"))
                .collect::<Vec<_>>()
        })
    } else {
        opts.into_iter()
            .map(|opts| {
                let (ret, output, timing) = messages::capture(|| do_build(&opts, false));
                (opts, ret, output, timing)
            })
            .collect()
    };

    let mut failed = false;
    let mut timing_rows = Vec::new();
    for (opts, ret, output, timing) in results {
        eprint!("{output}");
        timing_rows.push((opts.clone(), timing));
        match ret {
            Err(GazmErrorKind::Diagnostics(diagnostics)) => {
                failed = true;
                for diagnostic in diagnostics.as_slice() {
                    diagnostic.print_pretty(opts.verbose_errors)
                }
            }

            Err(GazmErrorKind::UserErrors(user_errors)) => {
                failed = true;
                for e in user_errors.to_vec() {
                    e.as_ref().print_pretty(opts.verbose_errors)
                }
            }

            Err(GazmErrorKind::Diagnostic(diagnostic)) => {
                failed = true;
                diagnostic.print_pretty(opts.verbose_errors);
            }

            Err(e) => {
                failed = true;
                println!("{e}");
            }

            Ok(..) => {}
        }
    }

    print_timing_report(&timing_rows);
    if let Some(path) = timings_file {
        if let Err(error) = append_timing_history(&path, &timing_rows) {
            eprintln!(" WARN Unable to write timing history {:?}: {error}", path);
        }
    }

    if failed {
        return Err("one or more targets failed".into());
    }

    Ok(())
}

#[cfg(test)]
#[allow(dead_code)]
mod test {
    // use crate::Assembler;
    use std::path::PathBuf;

    use super::*;

    fn make_opts(file_name: &str) -> Opts {
        let mut ret = Opts::default();
        ret.project_file = PathBuf::from(file_name);
        ret.build_type = BuildType::Check;
        ret
    }

    // TODO Reinstate this test and make circular includes error
    // #[test]
    fn test_circ() {
        let opts = make_opts("assets/test_src/circular_inc.gazm");
        let mut asm = Assembler::new(opts.clone());
        let res = asm.assemble();
        assert!(res.is_ok());
    }
}
