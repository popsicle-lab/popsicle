use std::fmt::Write as _;
use std::io::Write as _;

use cli_ux::{
    parse_cli, run_command, run_command_stateless, CliError, Command, CommandResponse,
    OutputFormat, SelfHostDomain,
};

fn print_response(response: CommandResponse, format: OutputFormat) {
    let mut buf = String::new();
    if matches!(format, OutputFormat::Json) {
        let mut obj = serde_json::Map::new();
        obj.insert("status".into(), response.status.into());
        if let Some(next) = response.next_step {
            obj.insert("next".into(), next.into());
        }
        for (k, v) in response.fields {
            obj.insert(k, v.into());
        }
        let _ = writeln!(buf, "{}", serde_json::Value::Object(obj));
    } else {
        let _ = writeln!(buf, "status: {}", response.status);
        for (key, value) in response.fields {
            let _ = writeln!(buf, "{key}: {value}");
        }
        if let Some(next_step) = response.next_step {
            let _ = writeln!(buf, "next: {next_step}");
        }
    }
    if std::io::stdout().write_all(buf.as_bytes()).is_err() {
        std::process::exit(0);
    }
}

fn print_error(err: &CliError, format: OutputFormat) {
    if matches!(format, OutputFormat::Json) {
        let obj = serde_json::json!({
            "status": "error",
            "category": err.category,
            "object": err.object_ref,
            "message": err.message,
            "next": err.next_step,
        });
        eprintln!("{obj}");
    } else {
        eprintln!("{err}");
    }
}

fn needs_workspace(command: &Command) -> bool {
    matches!(
        command,
        Command::Doctor { .. }
            | Command::Init
            | Command::IssueCreate { .. }
            | Command::IssueList
            | Command::IssueShow { .. }
            | Command::IssueClose { .. }
            | Command::IssueStart { .. }
            | Command::DocCreate { .. }
            | Command::DocList { .. }
            | Command::DocShow { .. }
            | Command::DocCheck { .. }
            | Command::PipelineStatus { .. }
            | Command::PipelineNext { .. }
            | Command::StageComplete { .. }
            | Command::ToolRun { .. }
            | Command::Admin(_)
            | Command::ProjectCurrent
    )
}

fn main() {
    let raw_args: Vec<String> = std::env::args().skip(1).collect();

    // Double-clicking Popsicle.app invokes the binary with no argv; open the UI instead of help.
    #[cfg(feature = "ui")]
    if raw_args.is_empty() && cli_ux::cli_install::launched_from_app_bundle() {
        cli_ux::cli_install::ensure_silent_if_app_bundle();
        cli_ux::ui::run(None);
        return;
    }

    let parsed = match parse_cli(raw_args) {
        Ok(p) => p,
        Err(err) => {
            print_error(&err, OutputFormat::default());
            std::process::exit(2);
        }
    };
    let format = parsed.globals.format;
    let cli_project = parsed.globals.project.clone();
    let command = parsed.command;

    #[cfg(feature = "ui")]
    if let Command::Ui { project } = &command {
        let ui_project = cli_project
            .as_deref()
            .or(project.as_deref())
            .map(str::to_string);
        cli_ux::ui::run(ui_project);
        return;
    }
    #[cfg(not(feature = "ui"))]
    if matches!(command, Command::Ui { .. }) {
        let err = CliError::actionable(
            "feature-disabled",
            "ui",
            "rebuild with `cargo build --features ui -p cli-ux`",
            "desktop UI requires the `ui` Cargo feature",
        );
        print_error(&err, format);
        std::process::exit(2);
    }

    if matches!(command, Command::Help) {
        let lang = if let Ok(domain) = SelfHostDomain::open_with(cli_project.as_deref()) {
            domain.project_language()
        } else {
            cli_ux::project_config::detect_default_language()
        };
        print_response(cli_ux::help_response_for(lang), format);
        return;
    }

    let is_tool_run = matches!(command, Command::ToolRun { .. });

    if !needs_workspace(&command) {
        return dispatch_stateless(command, format);
    }

    let domain_result = match &command {
        Command::Init => SelfHostDomain::open_or_bootstrap_with(cli_project.as_deref()),
        _ => SelfHostDomain::open_with_lazy(cli_project.as_deref()),
    };
    let mut domain = match domain_result {
        Ok(domain) => domain,
        Err(err) => {
            print_error(&err, format);
            std::process::exit(2);
        }
    };

    dispatch(&mut domain, command, format, is_tool_run);
}

fn dispatch_stateless(command: Command, format: OutputFormat) {
    match run_command_stateless(command) {
        Ok(response) => print_response(response, format),
        Err(err) => {
            print_error(&err, format);
            std::process::exit(1);
        }
    }
}

fn dispatch(
    domain: &mut SelfHostDomain,
    command: Command,
    format: OutputFormat,
    is_tool_run: bool,
) {
    match run_command(domain, command) {
        Ok(response) => {
            let tool_exit_code = if is_tool_run {
                response
                    .fields
                    .get("exit_code")
                    .and_then(|code| code.parse::<i32>().ok())
            } else {
                None
            };
            let failed = response.status == "failed";
            print_response(response, format);
            if let Some(code) = tool_exit_code {
                std::process::exit(code);
            }
            if failed {
                std::process::exit(1);
            }
        }
        Err(err) => {
            print_error(&err, format);
            std::process::exit(1);
        }
    }
}
