mod admin;
mod checklist;
mod context;
mod doc;
mod extract;
mod git;
mod init;
mod issue;
mod item;
mod memory;
mod migrate;
mod module;
mod namespace;
mod pipeline;
mod prompt;
pub(crate) mod registry;
mod reinit;
mod skill;
mod spec;
mod sync;
mod tool;

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

    /// Manage documents (artifacts, checklists, extraction)
    #[command(subcommand)]
    Doc(doc::DocCommand),

    /// Git commit tracking and review management
    #[command(subcommand)]
    Git(git::GitCommand),

    /// Issue tracking: create, list, start, and manage requirements
    #[command(subcommand)]
    Issue(issue::IssueCommand),

    /// Work item tracking: bugs, user stories, and test cases (unified)
    #[command(subcommand)]
    Item(item::ItemCommand),

    /// Cloud sync: login, push, pull, and reconcile against popsicle-cloud
    #[command(subcommand)]
    Sync(sync::SyncCommand),

    /// Manage modules (self-contained skill & pipeline distributions)
    #[command(subcommand)]
    Module(module::ModuleCommand),

    /// Manage tools (action-oriented skills: commands and AI prompt templates)
    #[command(subcommand)]
    Tool(tool::ToolCommand),

    /// Manage specs (group related pipeline runs and documents)
    #[command(subcommand)]
    Spec(spec::SpecCommand),

    /// Package registry: search, publish, and discover modules & tools
    #[command(subcommand)]
    Registry(registry::RegistryCommand),

    /// Project context: view pipeline context or scan project for technical profile
    #[command(subcommand)]
    Context(context::ContextCommand),

    /// Manage project memories (bugs, decisions, patterns, gotchas)
    #[command(subcommand)]
    Memory(memory::MemoryCommand),

    /// Get the AI prompt for a skill at a specific workflow state
    Prompt(prompt::PromptArgs),

    /// Low-frequency admin commands (migrate, reinit, namespace)
    #[command(subcommand)]
    Admin(admin::AdminCommand),

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
        Command::Issue(sub) => issue::execute(sub, format),
        Command::Item(sub) => item::execute(sub, format),
        Command::Sync(sub) => sync::execute(sub, format),
        Command::Module(sub) => module::execute(sub, format),
        Command::Tool(sub) => tool::execute(sub, format),
        Command::Spec(sub) => spec::execute(sub, format),
        Command::Registry(sub) => registry::execute(sub, format),
        Command::Context(sub) => context::execute(sub, format),
        Command::Memory(sub) => memory::execute(sub, format),
        Command::Prompt(args) => prompt::execute(args, format),
        Command::Admin(sub) => admin::execute(sub, format),
        Command::Completions { .. } => Ok(()),
        #[cfg(feature = "ui")]
        Command::Ui => unreachable!(),
    }
}
