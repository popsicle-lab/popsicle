mod bug;
mod context;
mod discussion;
mod doc;
mod extract;
mod git;
mod init;
mod issue;
mod memory;
mod migrate;
mod pipeline;
mod prompt;
mod skill;
mod story;
mod test;

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

    /// Manage discussion sessions (multi-role debate persistence)
    #[command(subcommand)]
    Discussion(discussion::DiscussionCommand),

    /// Issue tracking: create, list, start, and manage requirements
    #[command(subcommand)]
    Issue(issue::IssueCommand),

    /// Bug tracking: create, list, record from test failures
    #[command(subcommand)]
    Bug(bug::BugCommand),

    /// User story management: create, extract from PRD, track acceptance criteria
    #[command(subcommand)]
    Story(story::StoryCommand),

    /// Test case management: create, extract from specs, record results
    #[command(subcommand)]
    Test(test::TestCommand),

    /// Extract structured entities (user stories, test cases, bugs) from documents
    #[command(subcommand)]
    Extract(extract::ExtractCommand),

    /// Import existing Markdown documents into a pipeline run
    Migrate(migrate::MigrateArgs),

    /// Project context: view pipeline context or scan project for technical profile
    #[command(subcommand)]
    Context(context::ContextCommand),

    /// Manage project memories (bugs, decisions, patterns, gotchas)
    #[command(subcommand)]
    Memory(memory::MemoryCommand),

    /// Get the AI prompt for a skill at a specific workflow state
    Prompt(prompt::PromptArgs),

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },

    /// Launch the graphical UI
    #[cfg(feature = "ui")]
    Ui,
}

pub fn execute(cmd: Command, format: &OutputFormat) -> anyhow::Result<()> {
    match cmd {
        Command::Init(args) => init::execute(args, format),
        Command::Skill(sub) => skill::execute(sub, format),
        Command::Pipeline(sub) => pipeline::execute(sub, format),
        Command::Doc(sub) => doc::execute(sub, format),
        Command::Git(sub) => git::execute(sub, format),
        Command::Discussion(sub) => discussion::execute(sub, format),
        Command::Issue(sub) => issue::execute(sub, format),
        Command::Bug(sub) => bug::execute(sub, format),
        Command::Story(sub) => story::execute(sub, format),
        Command::Test(sub) => test::execute(sub, format),
        Command::Extract(sub) => extract::execute(sub, format),
        Command::Migrate(args) => migrate::execute(args, format),
        Command::Context(sub) => context::execute(sub, format),
        Command::Memory(sub) => memory::execute(sub, format),
        Command::Prompt(args) => prompt::execute(args, format),
        Command::Completions { .. } => Ok(()),
        #[cfg(feature = "ui")]
        Command::Ui => unreachable!(),
    }
}
