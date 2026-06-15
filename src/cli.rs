use clap::{Parser, Subcommand, ValueEnum};

use crate::model::Scope;

#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about = "Manage PATH entries through a platform-specific backend"
)]
pub struct Cli {
    #[arg(long, global = true, value_enum, help = "Environment scope to manage")]
    pub scope: Option<ScopeArg>,

    #[arg(long, global = true, help = "Shortcut for --scope system")]
    pub system: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum ScopeArg {
    User,
    System,
}

impl ScopeArg {
    pub fn resolve(scope: Option<Self>, system: bool) -> Result<Scope, String> {
        match (scope, system) {
            (Some(Self::User), true) => Err("--scope user cannot be used with --system".to_owned()),
            (Some(Self::User), false) | (None, false) => Ok(Scope::User),
            (Some(Self::System), _) | (None, true) => Ok(Scope::System),
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(about = "Add a PATH entry and apply it")]
    Add {
        #[arg(long, help = "Stable management name, like a Docker object name")]
        name: String,
        #[arg(help = "Directory to append to PATH")]
        value: String,
        #[arg(
            long,
            help = "Human-readable note for remembering why this entry exists"
        )]
        tips: Option<String>,
        #[arg(long, help = "Create the entry but keep it out of PATH")]
        disabled: bool,
    },
    #[command(about = "List managed PATH entries")]
    List {
        #[arg(long, help = "Show disabled entries too")]
        all: bool,
    },
    #[command(about = "Show one managed PATH entry")]
    Show { name: String },
    #[command(about = "Remove a managed PATH entry and apply the change")]
    Remove { name: String },
    #[command(about = "Enable a managed PATH entry and apply it")]
    Enable { name: String },
    #[command(about = "Disable a managed PATH entry and apply it")]
    Disable { name: String },
    #[command(about = "Rebuild PATH from managed entries")]
    Apply,
    #[command(about = "Export managed entries as JSON")]
    Export,
}
