mod context;
mod doc;
mod git;
mod init;
mod migrate;
mod pipeline;
mod prompt;
mod skill;

use clap::Subcommand;
use clap_complete::Shell;

use crate::OutputFormat;

#[derive(Subcommand)]
pub enum Command {
    /// Initialize a new Popsicle project in the current directory
    Init(init::InitArgs),

    /// Manage Skills
    #[command(subcommand)]
    Skill(skill::SkillCommand),

    /// Manage pipelines
    #[command(subcommand)]
    Pipeline(pipeline::PipelineCommand),

    /// Manage documents (artifacts)
    #[command(subcommand)]
    Doc(doc::DocCommand),

    /// Git commit tracking and review management
    #[command(subcommand)]
    Git(git::GitCommand),

    /// Import existing Markdown documents into a pipeline run
    Migrate(migrate::MigrateArgs),

    /// Output the full context of a pipeline run (for AI agents)
    Context(context::ContextArgs),

    /// Get the AI prompt for a skill at a specific workflow state
    Prompt(prompt::PromptArgs),

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },
}

pub fn execute(cmd: Command, format: &OutputFormat) -> anyhow::Result<()> {
    match cmd {
        Command::Init(args) => init::execute(args, format),
        Command::Skill(sub) => skill::execute(sub, format),
        Command::Pipeline(sub) => pipeline::execute(sub, format),
        Command::Doc(sub) => doc::execute(sub, format),
        Command::Git(sub) => git::execute(sub, format),
        Command::Migrate(args) => migrate::execute(args, format),
        Command::Context(args) => context::execute(args, format),
        Command::Prompt(args) => prompt::execute(args, format),
        Command::Completions { .. } => Ok(()),
    }
}
