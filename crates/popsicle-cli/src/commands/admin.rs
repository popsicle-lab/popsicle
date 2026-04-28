use clap::Subcommand;

use crate::OutputFormat;

use super::{migrate, namespace, reinit};

/// Low-frequency administrative commands grouped under `popsicle admin …`.
///
/// These commands are split out from the top-level surface to keep
/// day-to-day usage uncluttered.
#[derive(Subcommand)]
pub enum AdminCommand {
    /// Import existing Markdown documents into a pipeline run
    Migrate(migrate::MigrateArgs),

    /// Re-initialize database: export data, recreate with latest schema, re-import
    Reinit(reinit::ReinitArgs),

    /// Manage namespaces (group related specs)
    #[command(subcommand)]
    Namespace(namespace::NamespaceCommand),
}

pub fn execute(cmd: AdminCommand, format: &OutputFormat) -> anyhow::Result<()> {
    match cmd {
        AdminCommand::Migrate(args) => migrate::execute(args, format),
        AdminCommand::Reinit(args) => reinit::execute(args, format),
        AdminCommand::Namespace(sub) => namespace::execute(sub, format),
    }
}
