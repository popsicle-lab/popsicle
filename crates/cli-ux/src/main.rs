use std::fmt::Write as _;
use std::io::Write as _;

use cli_ux::{parse_args, run_command, Command, CommandResponse, OutputFormat, SelfHostDomain};

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
    // Exit quietly when the downstream pipe closes early (e.g. `popsicle help | head`).
    if std::io::stdout().write_all(buf.as_bytes()).is_err() {
        std::process::exit(0);
    }
}

fn output_format(command: &cli_ux::Command) -> OutputFormat {
    match command {
        cli_ux::Command::Doctor { format } => *format,
        _ => OutputFormat::Text,
    }
}

fn main() {
    let command = match parse_args(std::env::args().skip(1)) {
        Ok(command) => command,
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(2);
        }
    };
    let format = output_format(&command);

    let is_tool_run = matches!(command, Command::ToolRun { .. });

    // `help` needs no workspace; `init` may bootstrap a brand-new one.
    let domain_result = match command {
        Command::Help => {
            print_response(cli_ux::help_response(), format);
            return;
        }
        Command::Init => SelfHostDomain::open_or_bootstrap(),
        _ => SelfHostDomain::open(),
    };
    let mut domain = match domain_result {
        Ok(domain) => domain,
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(2);
        }
    };

    match run_command(&mut domain, command) {
        Ok(response) => {
            let tool_exit_code = if is_tool_run {
                response
                    .fields
                    .get("exit_code")
                    .and_then(|code| code.parse::<i32>().ok())
            } else {
                None
            };
            print_response(response, format);
            if let Some(code) = tool_exit_code {
                std::process::exit(code);
            }
        }
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    }
}
