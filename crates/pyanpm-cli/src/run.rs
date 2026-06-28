use std::path::PathBuf;

use pyanpm_core::{PyanpmError, PyanpmService};

use crate::cli::{ActivityCommands, CacheCommands, Cli, Commands};
use crate::output::{self, OutputOptions};

pub fn run(cli: Cli) -> i32 {
    let Cli {
        json,
        quiet,
        verbose: _verbose,
        plugins_dir,
        command,
    } = cli;

    let service = match PyanpmService::global(plugins_dir) {
        Ok(service) => service,
        Err(error) => {
            eprintln!("{error}");
            return 1;
        }
    };
    let state_dir = service.state_dir().to_path_buf();
    let output = OutputOptions { json, quiet };

    dispatch(command, service, state_dir, output)
}

fn dispatch(
    command: Commands,
    service: PyanpmService,
    state_dir: PathBuf,
    output: OutputOptions,
) -> i32 {
    match command {
        Commands::Init { force } => output::execute(
            "init",
            &state_dir,
            output,
            || service.init(force),
            output::print_init,
        ),
        Commands::Add {
            plugin_ref,
            version,
            git,
        } => output::execute(
            "add",
            &state_dir,
            output,
            || service.add(&plugin_ref, version, git.to_core()),
            output::print_install,
        ),
        Commands::Install => output::execute(
            "install",
            &state_dir,
            output,
            || service.install(),
            output::print_install,
        ),
        Commands::List => output::execute(
            "list",
            &state_dir,
            output,
            || service.list(),
            output::print_list,
        ),
        Commands::Doctor { write_probe } => output::execute(
            "doctor",
            &state_dir,
            output,
            || service.doctor_with_options(write_probe),
            output::print_doctor,
        ),
        Commands::ValidateSource { source_ref, git } => output::execute(
            "validate-source",
            &state_dir,
            output,
            || service.validate_source(&source_ref, git.to_core()),
            output::print_validate_source,
        ),
        Commands::Diff => output::execute(
            "diff",
            &state_dir,
            output,
            || service.diff(),
            output::print_diff,
        ),
        Commands::Remove {
            plugin_name,
            keep_manifest,
            yes,
        } => output::execute(
            "remove",
            &state_dir,
            output,
            || require_confirmation(yes, "pass --yes to remove a managed plugin").and_then(|_| service.remove(&plugin_name, keep_manifest)),
            output::print_remove,
        ),
        Commands::Reinstall { plugin_name } => output::execute(
            "reinstall",
            &state_dir,
            output,
            || service.reinstall(&plugin_name),
            output::print_install,
        ),
        Commands::Update {
            plugin_name,
            all,
            dry_run,
        } => output::execute(
            "update",
            &state_dir,
            output,
            || service.update(plugin_name.as_deref(), all, dry_run),
            output::print_update,
        ),
        Commands::Activity { command } => match command {
            ActivityCommands::List => output::execute(
                "activity-list",
                &state_dir,
                output,
                || service.activity_list(),
                output::print_activity_list,
            ),
            ActivityCommands::Show { activity_id } => output::execute(
                "activity-show",
                &state_dir,
                output,
                || service.activity_show(&activity_id),
                output::print_activity_show,
            ),
            ActivityCommands::Clear { yes } => output::execute(
                "activity-clear",
                &state_dir,
                output,
                || require_confirmation(yes, "pass --yes to clear activity history").and_then(|_| service.activity_clear()),
                output::print_activity_clear,
            ),
        },
        Commands::Cache { command } => match command {
            CacheCommands::List => output::execute(
                "cache-list",
                &state_dir,
                output,
                || service.cache_list(),
                output::print_cache_list,
            ),
            CacheCommands::Info { cache_id } => output::execute(
                "cache-info",
                &state_dir,
                output,
                || service.cache_info(&cache_id),
                output::print_cache_info,
            ),
            CacheCommands::Evict { cache_id, yes } => output::execute(
                "cache-evict",
                &state_dir,
                output,
                || require_confirmation(yes, "pass --yes to evict a cache entry").and_then(|_| service.cache_evict(&cache_id)),
                output::print_cache_mutation,
            ),
            CacheCommands::Prune { yes } => output::execute(
                "cache-prune",
                &state_dir,
                output,
                || require_confirmation(yes, "pass --yes to prune cache entries").and_then(|_| service.cache_prune()),
                output::print_cache_mutation,
            ),
        },
    }
}

fn require_confirmation(yes: bool, message: &str) -> Result<(), PyanpmError> {
    if yes {
        Ok(())
    } else {
        Err(PyanpmError::ConfirmationRequired(message.to_owned()))
    }
}
