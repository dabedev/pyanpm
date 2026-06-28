use clap::{Parser, Subcommand, ValueEnum};
use pyanpm_core::{GitRefKind, GitSourceOptions};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum GitRefKindArg {
    Branch,
    Tag,
    Commit,
}

impl From<GitRefKindArg> for GitRefKind {
    fn from(value: GitRefKindArg) -> Self {
        match value {
            GitRefKindArg::Branch => GitRefKind::Branch,
            GitRefKindArg::Tag => GitRefKind::Tag,
            GitRefKindArg::Commit => GitRefKind::Commit,
        }
    }
}

#[derive(Debug, Clone, Parser, Default)]
pub struct GitArgs {
    #[arg(long, value_enum)]
    pub git_ref_kind: Option<GitRefKindArg>,
    #[arg(long)]
    pub git_ref: Option<String>,
    #[arg(long)]
    pub git_subdir: Option<String>,
}

impl GitArgs {
    pub fn to_core(&self) -> GitSourceOptions {
        GitSourceOptions {
            git_ref_kind: self.git_ref_kind.map(Into::into),
            git_ref: self.git_ref.clone(),
            git_subdir: self.git_subdir.clone(),
        }
    }
}

#[derive(Debug, Parser)]
#[command(name = "pyanpm", about = "Manage Roblox Studio plugins from the global pyanPM manifest.")]
pub struct Cli {
    #[arg(long, global = true)]
    pub json: bool,
    #[arg(long, global = true)]
    pub quiet: bool,
    #[arg(long, global = true)]
    pub verbose: bool,
    #[arg(long, global = true)]
    pub plugins_dir: Option<std::path::PathBuf>,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Init {
        #[arg(long)]
        force: bool,
    },
    Add {
        plugin_ref: String,
        #[arg(long)]
        version: Option<String>,
        #[command(flatten)]
        git: GitArgs,
    },
    Install,
    List,
    Doctor {
        #[arg(long)]
        write_probe: bool,
    },
    ValidateSource {
        source_ref: String,
        #[command(flatten)]
        git: GitArgs,
    },
    Diff,
    Remove {
        plugin_name: String,
        #[arg(long)]
        keep_manifest: bool,
        #[arg(long)]
        yes: bool,
    },
    Reinstall {
        plugin_name: String,
    },
    Update {
        plugin_name: Option<String>,
        #[arg(long)]
        all: bool,
        #[arg(long)]
        dry_run: bool,
    },
    Activity {
        #[command(subcommand)]
        command: ActivityCommands,
    },
    Cache {
        #[command(subcommand)]
        command: CacheCommands,
    },
}

#[derive(Debug, Subcommand)]
pub enum ActivityCommands {
    List,
    Show {
        activity_id: String,
    },
    Clear {
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum CacheCommands {
    List,
    Info {
        cache_id: String,
    },
    Evict {
        cache_id: String,
        #[arg(long)]
        yes: bool,
    },
    Prune {
        #[arg(long)]
        yes: bool,
    },
}
