mod commands;
#[cfg(feature = "ui")]
mod ui;

use clap::{CommandFactory, Parser};
use clap_complete::generate;

#[derive(Parser)]
#[command(
    name = "popsicle",
    about = "Popsicle — A spec-driven development orchestration engine",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: commands::Command,

    /// Output format
    #[arg(long, default_value = "text", global = true)]
    format: OutputFormat,
}

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        commands::Command::Completions { shell } => {
            let mut cmd = Cli::command();
            generate(*shell, &mut cmd, "popsicle", &mut std::io::stdout());
            return Ok(());
        }
        #[cfg(feature = "ui")]
        commands::Command::Ui => {
            ui::run();
            return Ok(());
        }
        _ => {}
    }

    commands::execute(cli.command, &cli.format)
}
