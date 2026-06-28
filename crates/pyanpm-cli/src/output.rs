use std::path::Path;

use pyanpm_core::{
    missing_manifest_message, ActivityClearResult, ActivityListResult, ActivityShowResult, CacheInfoResult,
    CacheListResult, CacheMutationResult, DiffResult, DoctorResult, InitResult, InstallResult, JsonEnvelope,
    ListResult, ProgressEvent, PyanpmError, RemoveResult, UpdateResult, ValidateSourceResult,
};
use serde::Serialize;

#[derive(Debug, Clone, Copy)]
pub struct OutputOptions {
    pub json: bool,
    pub quiet: bool,
}

pub fn execute<T, F, P>(
    command: &'static str,
    project_dir: &Path,
    output: OutputOptions,
    operation: F,
    print_human: P,
) -> i32
where
    T: Serialize,
    F: FnOnce() -> Result<T, PyanpmError>,
    P: FnOnce(&T),
{
    match operation() {
        Ok(result) => {
            if output.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&JsonEnvelope::success(command, result))
                        .expect("serialize json output")
                );
            } else if !output.quiet {
                print_human(&result);
            }
            0
        }
        Err(error) => {
            if output.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(
                        &JsonEnvelope::<serde_json::Value>::failure(command, &error)
                    )
                    .expect("serialize error output")
                );
            } else if matches!(error, PyanpmError::ManifestMissing(_)) {
                eprintln!("{}", missing_manifest_message(project_dir));
            } else {
                eprintln!("{error}");
            }
            1
        }
    }
}

pub fn print_init(result: &InitResult) {
    println!("Created manifest at {}", result.manifest_path);
    print_completion(&result.completion);
}

pub fn print_install(result: &InstallResult) {
    print_progress(&result.progress);
    println!(
        "Installed {} plugin(s) into {}",
        result.installed.len(),
        result.plugins_dir
    );
    for plugin in &result.installed {
        println!("- {} -> {}", plugin.name, plugin.target_path);
    }
    print_completion(&result.completion);
}

pub fn print_list(result: &ListResult) {
    if result.plugins.is_empty() {
        println!("No managed plugins found.");
        return;
    }

    for plugin in &result.plugins {
        println!(
            "{}\t{}\t{}\t{}",
            plugin.name,
            plugin.status,
            plugin.installed_version.as_deref().unwrap_or("-"),
            plugin.target_path.as_deref().unwrap_or("-")
        );
    }
}

pub fn print_doctor(result: &DoctorResult) {
    let status = if result.report.healthy {
        "healthy"
    } else {
        "issues found"
    };
    println!("Doctor status: {status}");
    for finding in &result.report.findings {
        println!(
            "[{:?}] {}: {}",
            finding.level, finding.code, finding.message
        );
    }
    print_completion(&result.completion);
}

pub fn print_validate_source(result: &ValidateSourceResult) {
    println!(
        "Validation status: {}",
        if result.valid { "valid" } else { "invalid" }
    );
    if let Some(normalized) = &result.normalized_source_ref {
        println!("Normalized source: {normalized}");
    }
    if let Some(plugin_name) = &result.plugin_name {
        println!("Plugin name: {plugin_name}");
    }
    for issue in &result.errors {
        println!(
            "[{:?}] {} {}: {}",
            issue.severity, issue.field, issue.code, issue.message
        );
    }
}

pub fn print_diff(result: &DiffResult) {
    if result.snapshots.is_empty() {
        println!("No plugin state snapshots found.");
        return;
    }
    for snapshot in &result.snapshots {
        println!(
            "{}\t{:?}\t{}\t{}",
            snapshot.plugin_name,
            snapshot.state,
            snapshot.recommended_action,
            snapshot.installed_path.as_deref().unwrap_or("-")
        );
    }
}

pub fn print_remove(result: &RemoveResult) {
    print_progress(&result.progress);
    println!(
        "Removed plugin {} (manifest removed: {}, target: {})",
        result.removed.name,
        result.removed.removed_manifest_entry,
        result
            .removed
            .removed_target_path
            .as_deref()
            .unwrap_or("-")
    );
    print_completion(&result.completion);
}

pub fn print_update(result: &UpdateResult) {
    print_progress(&result.progress);
    if result.dry_run {
        println!("Dry-run update candidates:");
    } else {
        println!("Updated {} plugin(s)", result.updated.len());
    }
    for candidate in &result.candidates {
        println!(
            "{}\t{}\t{}\t{}",
            candidate.plugin_name,
            candidate.current_version.as_deref().unwrap_or("-"),
            candidate.candidate_version.as_deref().unwrap_or("-"),
            candidate.reason
        );
    }
    print_completion(&result.completion);
}

pub fn print_activity_list(result: &ActivityListResult) {
    if result.records.is_empty() {
        println!("No activity records found.");
        return;
    }
    for record in &result.records {
        println!(
            "{}\t{:?}\t{}\t{}",
            record.id, record.status, record.command_name, record.summary
        );
    }
}

pub fn print_activity_show(result: &ActivityShowResult) {
    let record = &result.record;
    println!("id: {}", record.id);
    println!("command: {}", record.command_name);
    println!("state: {}", record.state_path);
    println!("status: {:?}", record.status);
    println!("summary: {}", record.summary);
    if let Some(error_message) = &record.error_message {
        println!("error: {error_message}");
    }
}

pub fn print_activity_clear(result: &ActivityClearResult) {
    println!("Cleared {} activity record(s).", result.cleared);
}

pub fn print_cache_list(result: &CacheListResult) {
    println!("Total cache size: {} bytes", result.total_size_bytes);
    for entry in &result.entries {
        println!(
            "{}\t{}\t{}\t{}",
            entry.cache_id, entry.plugin_name, entry.size_bytes, entry.path
        );
    }
}

pub fn print_cache_info(result: &CacheInfoResult) {
    let entry = &result.entry;
    println!("cache id: {}", entry.cache_id);
    println!("plugin: {}", entry.plugin_name);
    println!("checksum: {}", entry.checksum);
    println!("size: {}", entry.size_bytes);
    println!("path: {}", entry.path);
}

pub fn print_cache_mutation(result: &CacheMutationResult) {
    println!("Removed {} cache entrie(s).", result.removed.len());
    for entry in &result.removed {
        println!("- {entry}");
    }
    if !result.protected.is_empty() {
        println!("Protected:");
        for entry in &result.protected {
            println!("- {entry}");
        }
    }
    print_completion(&result.completion);
}

fn print_progress(progress: &[ProgressEvent]) {
    for event in progress {
        let plugin = event.plugin_name.as_deref().unwrap_or("-");
        println!("[{:?}] {} {}: {}", event.severity, plugin, event.stage, event.message);
    }
}

fn print_completion(completion: &pyanpm_core::CompletionSummary) {
    println!(
        "Summary: {} (affected: {}, warnings: {}, errors: {}, duration: {}ms)",
        completion.summary,
        completion.affected_plugin_count,
        completion.warning_count,
        completion.error_count,
        completion.duration_ms
    );
}
