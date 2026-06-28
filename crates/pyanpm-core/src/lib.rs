pub mod activity;
pub mod audit;
pub mod cache;
pub mod command;
pub mod config;
pub mod error;
pub mod install;
pub mod operations;
pub mod plugin;
pub mod source;

pub use command::{
    missing_manifest_message, DoctorResult, JsonEnvelope, PyanpmService,
};
pub use config::{GitRefKind, GitSourceOptions};
pub use error::{PyanpmError, Result};
pub use operations::*;

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::process::Command;

    use fs_err as fs;
    use tempfile::TempDir;

    use crate::cache::checksum_path;
    use crate::command::PyanpmService;
    use crate::config::{GitRefKind, GitSourceOptions, Lockfile, Manifest, PackageMetadata};
    use crate::plugin::{validate_plugin_name, PluginSourceKind};

    #[test]
    fn validates_plugin_names() {
        assert!(validate_plugin_name("pyandb").is_ok());
        assert!(validate_plugin_name("bad/name").is_err());
        assert!(validate_plugin_name("CON").is_err());
    }

    #[test]
    fn parses_manifest_roundtrip() {
        let temp = TempDir::new().expect("temp dir");
        let manifest = Manifest::default();
        manifest.save(temp.path()).expect("save manifest");

        let loaded = Manifest::load(temp.path()).expect("load manifest");
        assert!(loaded.plugins.is_empty());
    }

    #[test]
    fn reads_package_metadata() {
        let temp = TempDir::new().expect("temp dir");
        fs::write(
            temp.path().join("pyanpm.plugin.toml"),
            "name = \"example\"\nversion = \"1.0.0\"\nentry = \"plugin.rbxm\"\n",
        )
        .expect("write metadata");

        let metadata = PackageMetadata::read_from(temp.path()).expect("read metadata");
        assert_eq!(metadata.name, "example");
        assert_eq!(metadata.version.as_deref(), Some("1.0.0"));
    }

    #[test]
    fn installs_local_file_and_writes_lockfile() {
        let temp = TempDir::new().expect("temp dir");
        let project_dir = temp.path().join("project");
        let plugins_dir = temp.path().join("plugins");
        fs::create_dir_all(&project_dir).expect("create project dir");
        fs::create_dir_all(&plugins_dir).expect("create plugins dir");

        let plugin_file = temp.path().join("sample-plugin.rbxm");
        fs::write(&plugin_file, b"rbxm-data").expect("write plugin file");

        let service = PyanpmService::new(&project_dir, Some(plugins_dir.clone()));
        service.init(false).expect("init");
        service
            .add(
                &format!("file:{}", plugin_file.display()),
                Some("0.1.0".to_owned()),
                GitSourceOptions::default(),
            )
            .expect("add plugin");

        let installed_path = plugins_dir.join("sample-plugin.rbxm");
        assert!(installed_path.exists());

        let lockfile = Lockfile::load(&project_dir).expect("load lockfile");
        assert_eq!(lockfile.plugins.len(), 1);
        assert_eq!(lockfile.plugins[0].name, "sample-plugin");
        let expected_target = installed_path
            .canonicalize()
            .expect("canonical installed path")
            .display()
            .to_string();
        assert_eq!(lockfile.plugins[0].target_path, expected_target);
        assert_eq!(
            lockfile.plugins[0].checksum,
            checksum_path(Path::new(&lockfile.plugins[0].target_path)).expect("checksum"),
        );
    }

    #[test]
    fn doctor_reports_healthy_install() {
        let temp = TempDir::new().expect("temp dir");
        let project_dir = temp.path().join("project");
        let plugins_dir = temp.path().join("plugins");
        fs::create_dir_all(&project_dir).expect("create project dir");
        fs::create_dir_all(&plugins_dir).expect("create plugins dir");

        let plugin_file = temp.path().join("doctor-plugin.rbxm");
        fs::write(&plugin_file, b"rbxm-doctor").expect("write plugin file");

        let service = PyanpmService::new(&project_dir, Some(plugins_dir));
        service.init(false).expect("init");
        service
            .add(
                &format!("file:{}", plugin_file.display()),
                Some("1.0.0".to_owned()),
                GitSourceOptions::default(),
            )
            .expect("add plugin");

        let report = service.doctor().expect("doctor").report;
        assert!(report.healthy);
        assert!(report.findings.iter().any(|finding| finding.code == "manifest"));
    }

    #[test]
    fn parses_git_manifest_roundtrip() {
        let temp = TempDir::new().expect("temp dir");
        let manifest = Manifest {
            plugins: [(
                "example".to_owned(),
                crate::config::ManifestPluginSpec {
                    source: crate::plugin::PluginSourceKind::Git,
                    path: None,
                    url: Some("https://github.com/org/example.git".to_owned()),
                    version: Some("1.2.3".to_owned()),
                    git_ref_kind: Some(GitRefKind::Tag),
                    git_ref: Some("v1.2.3".to_owned()),
                    subdir: Some("packages/example".to_owned()),
                },
            )]
            .into_iter()
            .collect(),
        };
        manifest.save(temp.path()).expect("save manifest");

        let loaded = Manifest::load(temp.path()).expect("load manifest");
        let plugin = loaded.plugins.get("example").expect("git plugin");
        assert_eq!(plugin.url.as_deref(), Some("https://github.com/org/example.git"));
        assert_eq!(plugin.git_ref_kind, Some(GitRefKind::Tag));
        assert_eq!(plugin.git_ref.as_deref(), Some("v1.2.3"));
        assert_eq!(plugin.subdir.as_deref(), Some("packages/example"));
    }

    #[test]
    fn lockfile_roundtrip_preserves_git_fields() {
        let temp = TempDir::new().expect("temp dir");
        let mut lockfile = Lockfile {
            version: 1,
            generated_at: None,
            plugins: vec![crate::config::LockedPlugin {
                name: "example".to_owned(),
                requested_version: Some("1.2.3".to_owned()),
                resolved_version: Some("1.2.3".to_owned()),
                source: crate::plugin::PluginSourceKind::Git,
                source_path: "C:/cache/git/example".to_owned(),
                source_url: Some("https://github.com/org/example.git".to_owned()),
                source_subdir: Some("packages/example".to_owned()),
                git_ref_kind: Some(GitRefKind::Commit),
                git_requested_ref: Some("0123abcd".to_owned()),
                git_resolved_commit: Some("0123abcd0123abcd".to_owned()),
                checksum: "checksum".to_owned(),
                artifact_kind: crate::plugin::ArtifactKind::File,
                target_path: "C:/Plugins/example.rbxm".to_owned(),
                installed_at: chrono::Utc::now(),
            }],
        };
        lockfile.save(temp.path()).expect("save lockfile");

        let loaded = Lockfile::load(temp.path()).expect("load lockfile");
        let plugin = &loaded.plugins[0];
        assert_eq!(plugin.source_url.as_deref(), Some("https://github.com/org/example.git"));
        assert_eq!(plugin.source_subdir.as_deref(), Some("packages/example"));
        assert_eq!(plugin.git_ref_kind, Some(GitRefKind::Commit));
        assert_eq!(plugin.git_requested_ref.as_deref(), Some("0123abcd"));
        assert_eq!(plugin.git_resolved_commit.as_deref(), Some("0123abcd0123abcd"));
    }

    #[test]
    fn installs_git_source_and_doctor_reports_git_health() {
        let temp = TempDir::new().expect("temp dir");
        let project_dir = temp.path().join("project");
        let plugins_dir = temp.path().join("plugins");
        let repo_dir = temp.path().join("repo");
        fs::create_dir_all(&project_dir).expect("create project dir");
        fs::create_dir_all(&plugins_dir).expect("create plugins dir");
        init_git_repo(&repo_dir);
        write_plugin_package(&repo_dir, "git-plugin", "1.0.0", "plugin.rbxm", b"git-plugin-v1");
        let commit = commit_all(&repo_dir, "initial");

        let service = PyanpmService::new(&project_dir, Some(plugins_dir.clone()));
        service.init(false).expect("init");
        service
            .add(&format!("git:{}", file_git_url(&repo_dir)), None, GitSourceOptions::default())
            .expect("add git plugin");

        let installed_path = plugins_dir.join("git-plugin.rbxm");
        assert!(installed_path.exists());

        let lockfile = Lockfile::load(&project_dir).expect("load lockfile");
        let plugin = &lockfile.plugins[0];
        assert_eq!(plugin.source, PluginSourceKind::Git);
        assert_eq!(plugin.source_url.as_deref(), Some(file_git_url(&repo_dir).as_str()));
        assert_eq!(plugin.git_resolved_commit.as_deref(), Some(commit.as_str()));

        let report = service.doctor().expect("doctor").report;
        assert!(report.healthy);
        assert!(report.findings.iter().any(|finding| finding.code == "git.access"));
        assert!(report.findings.iter().any(|finding| finding.code == "source.git.git-plugin"));
    }

    #[test]
    fn installs_git_tag_source() {
        let temp = TempDir::new().expect("temp dir");
        let project_dir = temp.path().join("project");
        let plugins_dir = temp.path().join("plugins");
        let repo_dir = temp.path().join("repo");
        fs::create_dir_all(&project_dir).expect("create project dir");
        fs::create_dir_all(&plugins_dir).expect("create plugins dir");
        init_git_repo(&repo_dir);
        write_plugin_package(&repo_dir, "tagged-plugin", "1.0.0", "plugin.rbxm", b"git-tag");
        let tagged_commit = commit_all(&repo_dir, "tagged");
        run_git(&repo_dir, &["tag", "v1.0.0"]);

        let service = PyanpmService::new(&project_dir, Some(plugins_dir));
        service.init(false).expect("init");
        service
            .add(
                &format!("git:{}", file_git_url(&repo_dir)),
                None,
                GitSourceOptions {
                    git_ref_kind: Some(GitRefKind::Tag),
                    git_ref: Some("v1.0.0".to_owned()),
                    git_subdir: None,
                },
            )
            .expect("add git tag plugin");

        let lockfile = Lockfile::load(&project_dir).expect("load lockfile");
        let plugin = &lockfile.plugins[0];
        assert_eq!(plugin.git_ref_kind, Some(GitRefKind::Tag));
        assert_eq!(plugin.git_requested_ref.as_deref(), Some("v1.0.0"));
        assert_eq!(plugin.git_resolved_commit.as_deref(), Some(tagged_commit.as_str()));
    }

    #[test]
    fn installs_git_commit_source() {
        let temp = TempDir::new().expect("temp dir");
        let project_dir = temp.path().join("project");
        let plugins_dir = temp.path().join("plugins");
        let repo_dir = temp.path().join("repo");
        fs::create_dir_all(&project_dir).expect("create project dir");
        fs::create_dir_all(&plugins_dir).expect("create plugins dir");
        init_git_repo(&repo_dir);
        write_plugin_package(&repo_dir, "commit-plugin", "1.0.0", "plugin.rbxm", b"commit-v1");
        let _first_commit = commit_all(&repo_dir, "first");
        write_plugin_package(&repo_dir, "commit-plugin", "1.1.0", "plugin.rbxm", b"commit-v2");
        let second_commit = commit_all(&repo_dir, "second");

        let service = PyanpmService::new(&project_dir, Some(plugins_dir));
        service.init(false).expect("init");
        service
            .add(
                &format!("git:{}", file_git_url(&repo_dir)),
                None,
                GitSourceOptions {
                    git_ref_kind: Some(GitRefKind::Commit),
                    git_ref: Some(second_commit.clone()),
                    git_subdir: None,
                },
            )
            .expect("add git commit plugin");

        let lockfile = Lockfile::load(&project_dir).expect("load lockfile");
        let plugin = &lockfile.plugins[0];
        assert_eq!(plugin.git_ref_kind, Some(GitRefKind::Commit));
        assert_eq!(plugin.git_requested_ref.as_deref(), Some(second_commit.as_str()));
        assert_eq!(plugin.git_resolved_commit.as_deref(), Some(second_commit.as_str()));
        assert_eq!(plugin.resolved_version.as_deref(), Some("1.1.0"));
    }

    #[test]
    fn installs_git_subdir_source() {
        let temp = TempDir::new().expect("temp dir");
        let project_dir = temp.path().join("project");
        let plugins_dir = temp.path().join("plugins");
        let repo_dir = temp.path().join("repo");
        let package_dir = repo_dir.join("packages").join("subdir-plugin");
        fs::create_dir_all(&project_dir).expect("create project dir");
        fs::create_dir_all(&plugins_dir).expect("create plugins dir");
        init_git_repo(&repo_dir);
        write_plugin_package(&package_dir, "subdir-plugin", "1.0.0", "plugin.rbxm", b"subdir-v1");
        commit_all(&repo_dir, "initial");

        let service = PyanpmService::new(&project_dir, Some(plugins_dir.clone()));
        service.init(false).expect("init");
        service
            .add(
                &format!("git:{}", file_git_url(&repo_dir)),
                None,
                GitSourceOptions {
                    git_ref_kind: None,
                    git_ref: None,
                    git_subdir: Some("packages/subdir-plugin".to_owned()),
                },
            )
            .expect("add git subdir plugin");

        let installed_path = plugins_dir.join("subdir-plugin.rbxm");
        assert!(installed_path.exists());

        let lockfile = Lockfile::load(&project_dir).expect("load lockfile");
        let plugin = &lockfile.plugins[0];
        assert_eq!(plugin.name, "subdir-plugin");
        assert_eq!(plugin.source_subdir.as_deref(), Some("packages/subdir-plugin"));
    }

    #[test]
    fn update_detects_and_installs_new_git_commit() {
        let temp = TempDir::new().expect("temp dir");
        let project_dir = temp.path().join("project");
        let plugins_dir = temp.path().join("plugins");
        let repo_dir = temp.path().join("repo");
        fs::create_dir_all(&project_dir).expect("create project dir");
        fs::create_dir_all(&plugins_dir).expect("create plugins dir");
        init_git_repo(&repo_dir);
        write_plugin_package(&repo_dir, "update-plugin", "1.0.0", "plugin.rbxm", b"update-v1");
        let first_commit = commit_all(&repo_dir, "first");

        let service = PyanpmService::new(&project_dir, Some(plugins_dir.clone()));
        service.init(false).expect("init");
        service
            .add(&format!("git:{}", file_git_url(&repo_dir)), None, GitSourceOptions::default())
            .expect("add git plugin");

        write_plugin_package(&repo_dir, "update-plugin", "2.0.0", "plugin.rbxm", b"update-v2");
        let second_commit = commit_all(&repo_dir, "second");
        assert_ne!(first_commit, second_commit);

        let preview = service
            .update(Some("update-plugin"), false, true)
            .expect("preview update");
        assert_eq!(preview.candidates.len(), 1);
        assert!(preview.candidates[0].will_change);
        assert_eq!(preview.candidates[0].candidate_version.as_deref(), Some("2.0.0"));

        let result = service
            .update(Some("update-plugin"), false, false)
            .expect("apply update");
        assert_eq!(result.updated.len(), 1);

        let lockfile = Lockfile::load(&project_dir).expect("load lockfile");
        let plugin = &lockfile.plugins[0];
        assert_eq!(plugin.git_resolved_commit.as_deref(), Some(second_commit.as_str()));
        assert_eq!(plugin.resolved_version.as_deref(), Some("2.0.0"));
        assert_eq!(
            plugin.checksum,
            checksum_path(Path::new(&plugin.target_path)).expect("checksum"),
        );
    }

    fn init_git_repo(repo_dir: &Path) {
        assert!(git_available(), "git must be available to run git source tests");
        fs::create_dir_all(repo_dir).expect("create repo dir");
        run_git(repo_dir, &["init"]);
        run_git(repo_dir, &["config", "user.email", "tests@pyanpm.local"]);
        run_git(repo_dir, &["config", "user.name", "pyanpm tests"]);
    }

    fn write_plugin_package(package_dir: &Path, name: &str, version: &str, entry: &str, bytes: &[u8]) {
        fs::create_dir_all(package_dir).expect("create package dir");
        fs::write(
            package_dir.join("pyanpm.plugin.toml"),
            format!("name = \"{name}\"\nversion = \"{version}\"\nentry = \"{entry}\"\n"),
        )
        .expect("write metadata");
        fs::write(package_dir.join(entry), bytes).expect("write artifact");
    }

    fn commit_all(repo_dir: &Path, message: &str) -> String {
        run_git(repo_dir, &["add", "."]);
        run_git(repo_dir, &["commit", "--quiet", "-m", message]);
        run_git(repo_dir, &["rev-parse", "HEAD"])
    }

    fn file_git_url(repo_dir: &Path) -> String {
        let canonical = repo_dir.canonicalize().expect("canonical repo path");
        let mut path = canonical.display().to_string();
        if let Some(stripped) = path.strip_prefix(r"\\?\") {
            path = stripped.to_owned();
        }
        format!("file:///{}", path.replace('\\', "/"))
    }

    fn git_available() -> bool {
        Command::new("git")
            .arg("--version")
            .output()
            .is_ok_and(|output| output.status.success())
    }

    fn run_git(repo_dir: &Path, args: &[&str]) -> String {
        let output = Command::new("git")
            .args(args)
            .current_dir(repo_dir)
            .output()
            .expect("run git");
        assert!(
            output.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).trim().to_owned()
    }
}
