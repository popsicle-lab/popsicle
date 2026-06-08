use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand, ValueEnum};
use colored::Colorize;
use serde::Serialize;

use intent_core::analysis::{
    coverage_report, diff as ana_diff, explain as ana_explain, impact as ana_impact,
    testspec as ana_testspec, Change, Lifecycle, ModificationKind,
};
use intent_core::smt::{verify_vc, VerifyResult};
use intent_core::typeck::check_program;
use intent_core::vcgen::{generate_vcs, VcKind};
use intent_core::DiagLevel;
use intent_syntax::ast::Declaration;
use intent_syntax::parse;

#[derive(Parser)]
#[command(
    name = "intent",
    version,
    about = "intent-lang: requirements modeling DSL with formal verification"
)]
struct Cli {
    /// Output format (applies to all subcommands)
    #[arg(long, value_enum, default_value_t = OutputFormat::Text, global = true)]
    format: OutputFormat,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse, type-check, and verify an .intent file
    Check {
        file: PathBuf,
        /// Show SMT-LIB2 encoding (debug)
        #[arg(long)]
        show_smt: bool,
        /// Show applied safety rules
        #[arg(long)]
        show_safety: bool,
        /// Include @asis intents (default: skip them)
        #[arg(long)]
        include_asis: bool,
    },
    /// Parse and dump AST (debug)
    Parse { file: PathBuf },
    /// Run completeness analysis on `coverage` blocks
    Coverage { file: PathBuf },
    /// Emit a test specification (scenarios per intent) — for downstream tools
    Testspec { file: PathBuf },
    /// Diff two .intent files; classify changes (loosened / tightened / reshaped)
    Diff { old: PathBuf, new: PathBuf },
    /// Walk diff to identify affected goals and coverage scenarios
    Impact { old: PathBuf, new: PathBuf },
    /// Render a plain-English explanation of an intent / safety / goal
    Explain { file: PathBuf, target: String },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check {
            file,
            show_smt,
            show_safety,
            include_asis,
        } => cmd_check(&file, show_smt, show_safety, include_asis, cli.format),
        Commands::Parse { file } => cmd_parse(&file),
        Commands::Coverage { file } => cmd_coverage(&file, cli.format),
        Commands::Testspec { file } => cmd_testspec(&file, cli.format),
        Commands::Diff { old, new } => cmd_diff(&old, &new, cli.format),
        Commands::Impact { old, new } => cmd_impact(&old, &new, cli.format),
        Commands::Explain { file, target } => cmd_explain(&file, &target, cli.format),
    }
}

fn read_file(path: &PathBuf) -> String {
    match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "{} cannot read {}: {e}",
                "error:".red().bold(),
                path.display()
            );
            process::exit(1);
        }
    }
}

fn parse_or_die(path: &PathBuf) -> intent_syntax::ast::Program {
    let source = read_file(path);
    match parse(&source) {
        Ok(p) => p,
        Err(e) => {
            let (line, col) = offset_to_line_col(&source, e.span.start);
            eprintln!(
                "  {} {} (at {}:{}:{})",
                "❌".red(),
                e.message,
                path.display(),
                line,
                col
            );
            process::exit(1);
        }
    }
}

// ── check ───────────────────────────────────────────────────────

#[derive(Serialize)]
struct CheckJson {
    file: String,
    diagnostics: Vec<DiagJson>,
    results: Vec<VcJson>,
    ok: bool,
}

#[derive(Serialize)]
struct DiagJson {
    level: String,
    code: String,
    message: String,
    line: usize,
    col: usize,
}

#[derive(Serialize)]
struct VcJson {
    name: String,
    kind: String,
    status: String,
    detail: Option<String>,
    /// One of: "primary", "asis-skipped"
    track: String,
}

fn cmd_check(
    path: &PathBuf,
    show_smt: bool,
    show_safety: bool,
    include_asis: bool,
    fmt: OutputFormat,
) {
    let source = read_file(path);
    let filename = path.file_name().unwrap_or_default().to_string_lossy();

    if matches!(fmt, OutputFormat::Text) {
        println!("\n  {} {}...\n", "Checking".bold(), filename.cyan());
    }

    let prog = match parse(&source) {
        Ok(p) => p,
        Err(e) => {
            let (line, col) = offset_to_line_col(&source, e.span.start);
            match fmt {
                OutputFormat::Text => {
                    eprintln!(
                        "  {} {}\n    --> {}:{}:{}\n",
                        "❌".red(),
                        e.message,
                        filename,
                        line,
                        col
                    );
                }
                OutputFormat::Json => {
                    let out = CheckJson {
                        file: filename.to_string(),
                        diagnostics: vec![DiagJson {
                            level: "error".to_string(),
                            code: "PARSE".to_string(),
                            message: e.message,
                            line,
                            col,
                        }],
                        results: vec![],
                        ok: false,
                    };
                    println!("{}", serde_json::to_string_pretty(&out).unwrap());
                }
            }
            process::exit(1);
        }
    };

    let diags = check_program(&prog);
    let has_errors = diags.iter().any(|d| d.level == DiagLevel::Error);
    let mut diag_jsons = Vec::new();
    for d in &diags {
        let (line, col) = offset_to_line_col(&source, d.span.start);
        let level_str = match d.level {
            DiagLevel::Error => "error",
            DiagLevel::Warning => "warning",
            DiagLevel::Info => "info",
        };
        diag_jsons.push(DiagJson {
            level: level_str.to_string(),
            code: d.code.clone(),
            message: d.message.clone(),
            line,
            col,
        });
        if matches!(fmt, OutputFormat::Text) {
            let prefix = match d.level {
                DiagLevel::Error => "❌".red().to_string(),
                DiagLevel::Warning => "⚠️".yellow().to_string(),
                DiagLevel::Info => "ℹ️".blue().to_string(),
            };
            eprintln!(
                "  {} {}[{}]: {}\n    --> {}:{}:{}\n",
                prefix,
                match d.level {
                    DiagLevel::Error => "error".red().bold().to_string(),
                    DiagLevel::Warning => "warning".yellow().bold().to_string(),
                    DiagLevel::Info => "info".blue().bold().to_string(),
                },
                d.code,
                d.message,
                filename,
                line,
                col
            );
        }
    }
    if has_errors {
        if matches!(fmt, OutputFormat::Json) {
            let out = CheckJson {
                file: filename.to_string(),
                diagnostics: diag_jsons,
                results: vec![],
                ok: false,
            };
            println!("{}", serde_json::to_string_pretty(&out).unwrap());
        }
        process::exit(1);
    }

    // Build asis exclusion set (RFC A2)
    let asis_intents: std::collections::HashSet<String> = prog
        .declarations
        .iter()
        .filter_map(|d| match &d.node {
            Declaration::Intent(i)
                if matches!(intent_core::analysis::intent_lifecycle(i), Lifecycle::AsIs) =>
            {
                Some(i.name.clone())
            }
            _ => None,
        })
        .collect();

    let vcs = generate_vcs(&prog);

    let mut all_ok = true;
    let mut vc_jsons = Vec::new();

    for vc in &vcs {
        let kind_str = match vc.kind {
            VcKind::Intent => "intent",
            VcKind::Theorem => "theorem",
        };

        let is_asis = asis_intents.contains(&vc.name);
        let track = if is_asis { "asis-skipped" } else { "primary" };

        if is_asis && !include_asis {
            if matches!(fmt, OutputFormat::Text) {
                println!(
                    "  {} {} {} — {} (legacy track; pass --include-asis to verify)",
                    "🟡".yellow(),
                    kind_str,
                    vc.name.yellow().bold(),
                    "skipped".yellow()
                );
            }
            vc_jsons.push(VcJson {
                name: vc.name.clone(),
                kind: kind_str.to_string(),
                status: "asis-skipped".to_string(),
                detail: None,
                track: track.to_string(),
            });
            continue;
        }

        if matches!(fmt, OutputFormat::Text) && show_safety && !vc.safety_rules.is_empty() {
            println!(
                "  {} applied safety rules for {}:",
                "ℹ️".blue(),
                vc.name.cyan()
            );
            for rule in &vc.safety_rules {
                println!("    - {}.invariant[{}]", rule.safety_name, rule.index);
            }
            println!();
        }

        if matches!(fmt, OutputFormat::Text) && show_smt {
            let mut encoder = intent_core::smt::SmtEncoder::new(&prog);
            encoder.encode_vc(vc, &prog);
            println!(
                "  {} SMT for {}:\n{}\n",
                "🔍".blue(),
                vc.name.cyan(),
                encoder.get_output()
            );
        }

        if let Some(reason) = &vc.unsupported {
            if matches!(fmt, OutputFormat::Text) {
                println!(
                    "  {} {} {} — {} ({})",
                    "⚠️".yellow(),
                    kind_str,
                    vc.name.yellow().bold(),
                    "skipped".yellow(),
                    reason
                );
            }
            vc_jsons.push(VcJson {
                name: vc.name.clone(),
                kind: kind_str.to_string(),
                status: "skipped".to_string(),
                detail: Some(reason.clone()),
                track: track.to_string(),
            });
            continue;
        }

        let result = verify_vc(vc, &prog);
        let (status, detail) = match &result {
            VerifyResult::Verified => ("verified".to_string(), None),
            VerifyResult::Failed { counterexample } => {
                all_ok = false;
                ("failed".to_string(), Some(counterexample.clone()))
            }
            VerifyResult::Unknown { reason } => {
                all_ok = false;
                ("unknown".to_string(), Some(reason.clone()))
            }
            VerifyResult::Error { message } => {
                all_ok = false;
                ("error".to_string(), Some(message.clone()))
            }
        };

        if matches!(fmt, OutputFormat::Text) {
            match &result {
                VerifyResult::Verified => println!(
                    "  {} {} {} — {}",
                    "✅".green(),
                    kind_str,
                    vc.name.green().bold(),
                    "verified".green()
                ),
                VerifyResult::Failed { counterexample } => {
                    println!(
                        "  {} {} {} — {}",
                        "❌".red(),
                        kind_str,
                        vc.name.red().bold(),
                        "FAILED".red().bold()
                    );
                    if !counterexample.is_empty() {
                        println!("\n     {}", "Counterexample:".yellow());
                        for line in counterexample.lines().take(20) {
                            println!("       {line}");
                        }
                        println!();
                    }
                }
                VerifyResult::Unknown { reason } => println!(
                    "  {} {} {} — {} ({})",
                    "⚠️".yellow(),
                    kind_str,
                    vc.name.yellow().bold(),
                    "unknown".yellow(),
                    reason.lines().next().unwrap_or("")
                ),
                VerifyResult::Error { message } => println!(
                    "  {} {} {} — {}",
                    "❌".red(),
                    kind_str,
                    vc.name.red().bold(),
                    message.red()
                ),
            }
        }

        vc_jsons.push(VcJson {
            name: vc.name.clone(),
            kind: kind_str.to_string(),
            status,
            detail,
            track: track.to_string(),
        });
    }

    if matches!(fmt, OutputFormat::Text) {
        println!();
    }

    if matches!(fmt, OutputFormat::Json) {
        let out = CheckJson {
            file: filename.to_string(),
            diagnostics: diag_jsons,
            results: vc_jsons,
            ok: all_ok,
        };
        println!("{}", serde_json::to_string_pretty(&out).unwrap());
    }

    if !all_ok {
        process::exit(1);
    }
}

// ── coverage ────────────────────────────────────────────────────

fn cmd_coverage(path: &PathBuf, fmt: OutputFormat) {
    let prog = parse_or_die(path);
    let _ = check_program(&prog);
    let report = coverage_report(&prog);
    match fmt {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
        }
        OutputFormat::Text => {
            println!("\n  {} {}\n", "Coverage".bold(), path.display());
            if report.coverages.is_empty() {
                println!(
                    "  {} no `coverage` declarations found in this file",
                    "ℹ️".blue()
                );
                return;
            }
            for s in &report.coverages {
                println!(
                    "  {} {} — {}/{} combinations covered",
                    if s.uncovered.is_empty() {
                        "✅".green().to_string()
                    } else {
                        "⚠️".yellow().to_string()
                    },
                    s.name.cyan().bold(),
                    s.covered,
                    s.total_combinations
                );
                if !s.uncovered.is_empty() {
                    println!("    {}", "Uncovered combinations:".yellow());
                    for combo in s.uncovered.iter().take(20) {
                        let parts: Vec<String> =
                            combo.iter().map(|(k, v)| format!("{k}={v}")).collect();
                        println!("      • {}", parts.join(", "));
                    }
                    if s.uncovered.len() > 20 {
                        println!("      … {} more", s.uncovered.len() - 20);
                    }
                }
                println!();
            }
        }
    }
}

// ── testspec ────────────────────────────────────────────────────

fn cmd_testspec(path: &PathBuf, fmt: OutputFormat) {
    let prog = parse_or_die(path);
    let _ = check_program(&prog);
    let spec = ana_testspec(&prog);
    match fmt {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&spec).unwrap());
        }
        OutputFormat::Text => {
            println!("\n  {} {}\n", "Testspec".bold(), path.display());
            for it in &spec.intents {
                let lc = match it.lifecycle {
                    Lifecycle::AsIs => "[asis]".yellow().to_string(),
                    Lifecycle::ToBe => "[tobe]".cyan().to_string(),
                    Lifecycle::Current => "".to_string(),
                };
                println!("  {} {} {}", "intent".bold(), it.intent.cyan(), lc);
                println!("    params: {}", it.params.join(", "));
                for (i, sc) in it.scenarios.iter().enumerate() {
                    println!("    {:2}. {}", i + 1, sc.label.bold());
                    if !sc.assumptions.is_empty() {
                        println!("        given: {}", sc.assumptions.join(" && "));
                    }
                    println!("        expect: {}", sc.expected);
                }
                println!();
            }
        }
    }
}

// ── diff ────────────────────────────────────────────────────────

fn cmd_diff(old: &PathBuf, new: &PathBuf, fmt: OutputFormat) {
    let p_old = parse_or_die(old);
    let p_new = parse_or_die(new);
    let report = ana_diff(&p_old, &p_new);
    match fmt {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
        }
        OutputFormat::Text => print_diff(&report),
    }
}

fn print_diff(r: &intent_core::analysis::DiffReport) {
    println!(
        "\n  {} {} added · {} removed · {} modified · {} potentially-breaking\n",
        "Diff:".bold(),
        r.summary.added,
        r.summary.removed,
        r.summary.modified,
        r.summary.potentially_breaking.to_string().red().bold()
    );
    for c in &r.changes {
        match c {
            Change::Added { decl_kind, name } => {
                println!("  {} {} {}", "➕".green(), decl_kind, name.green().bold());
            }
            Change::Removed { decl_kind, name } => {
                println!("  {} {} {}", "➖".red(), decl_kind, name.red().bold());
            }
            Change::Modified {
                decl_kind,
                name,
                classification,
                details,
            } => {
                let label = match classification {
                    ModificationKind::Loosened => "loosened".green().to_string(),
                    ModificationKind::Tightened => "TIGHTENED".red().bold().to_string(),
                    ModificationKind::Reshaped => "reshaped".yellow().to_string(),
                };
                println!(
                    "  {} {} {} — {}",
                    "✎".yellow(),
                    decl_kind,
                    name.cyan().bold(),
                    label
                );
                for d in details {
                    println!("      {d}");
                }
            }
        }
    }
    println!();
}

// ── impact ──────────────────────────────────────────────────────

fn cmd_impact(old: &PathBuf, new: &PathBuf, fmt: OutputFormat) {
    let p_old = parse_or_die(old);
    let p_new = parse_or_die(new);
    let report = ana_impact(&p_old, &p_new);
    match fmt {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
        }
        OutputFormat::Text => {
            print_diff(&report.diff);
            println!(
                "  {} {}",
                "Affected goals:".bold(),
                if report.affected_goals.is_empty() {
                    "(none)".to_string()
                } else {
                    report.affected_goals.join(", ")
                }
            );
            println!(
                "  {} {}",
                "Affected coverages:".bold(),
                if report.affected_coverages.is_empty() {
                    "(none)".to_string()
                } else {
                    report.affected_coverages.join(", ")
                }
            );
            println!();
        }
    }
}

// ── explain ─────────────────────────────────────────────────────

fn cmd_explain(path: &PathBuf, target: &str, fmt: OutputFormat) {
    let prog = parse_or_die(path);
    let _ = check_program(&prog);
    match ana_explain(&prog, target) {
        None => {
            eprintln!(
                "  {} no intent/safety/goal named `{}` in {}",
                "❌".red(),
                target,
                path.display()
            );
            process::exit(1);
        }
        Some(report) => match fmt {
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&report).unwrap());
            }
            OutputFormat::Text => {
                println!(
                    "\n  {} {} ({})\n",
                    "Explain".bold(),
                    report.target.cyan().bold(),
                    report.kind
                );
                println!("  {}\n", report.plain_english);
                if !report.clauses.is_empty() {
                    println!("  {}", "Clauses:".bold());
                    for c in &report.clauses {
                        println!("    • [{}] {}", c.kind, c.formal);
                        println!("        ↳ {}", c.natural);
                    }
                    println!();
                }
                if let Some(e) = &report.satisfying_example {
                    println!("  {} {}", "Satisfying example:".green(), e);
                }
                if let Some(e) = &report.violating_example {
                    println!("  {} {}", "Violating example:".red(), e);
                }
                println!();
            }
        },
    }
}

// ── parse ───────────────────────────────────────────────────────

fn cmd_parse(path: &PathBuf) {
    let source = read_file(path);
    match parse(&source) {
        Ok(prog) => {
            println!("{:#?}", prog);
        }
        Err(e) => {
            eprintln!("Parse error: {e}");
            process::exit(1);
        }
    }
}

fn offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    for (i, ch) in source.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}
