//! Executable repository and fixture-contract checks.

mod support;

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

const FIXTURE_SUITES: &[&str] = &[
    "syntax",
    "typed-model",
    "roundtrip",
    "capabilities",
    "version-boundaries",
    "generators",
    "real-world",
];

#[test]
fn github_actions_are_immutable_and_versioned() -> Result<(), String> {
    support::validate_action_pins(&repository_root())
}

#[test]
fn repository_supply_chain_has_single_sources_and_immutable_pins() -> Result<(), String> {
    support::validate_repository_supply_chain(&repository_root())
}

#[test]
fn public_api_compatibility_runs_in_ci_and_release() -> Result<(), String> {
    const ACTION: &str = "obi1kenobi/cargo-semver-checks-action@6b69fcf40e9b5fb17adeb57e4b6ecd020649a239 # v2.9";
    const CONFIGURATION: &str = "package: quadlet-lens";

    for workflow_name in ["ci.yml", "release.yml"] {
        let workflow_path = repository_root().join(".github/workflows").join(workflow_name);
        let workflow = fs::read_to_string(&workflow_path)
            .map_err(|error| format!("failed to read {}: {error}", workflow_path.display()))?;

        let configured_action = format!("uses: {ACTION}\n        with:\n          {CONFIGURATION}");
        if workflow.matches(ACTION).count() != 1
            || workflow.matches(&configured_action).count() != 1
            || workflow.contains("release-type:")
        {
            return Err(format!(
                "{workflow_name} must contain one version-derived cargo-semver-checks action for quadlet-lens"
            ));
        }
    }

    Ok(())
}

#[test]
fn coverage_ratchet_runs_in_ci_and_release() -> Result<(), String> {
    const INSTALL: &str = "cargo install --locked --version 0.8.7 cargo-llvm-cov";
    const CLEAN: &str = "cargo llvm-cov clean --locked";
    const COMMAND: &str = "cargo llvm-cov --locked --no-clean --workspace --all-features --all-targets --summary-only\n          --fail-under-regions 91 --fail-under-functions 92 --fail-under-lines 92";

    for workflow_name in ["ci.yml", "release.yml"] {
        let workflow_path = repository_root().join(".github/workflows").join(workflow_name);
        let workflow = fs::read_to_string(&workflow_path)
            .map_err(|error| format!("failed to read {}: {error}", workflow_path.display()))?;

        for required in ["rustup component add llvm-tools-preview", INSTALL, CLEAN, COMMAND] {
            if workflow.matches(required).count() != 1 {
                return Err(format!(
                    "{workflow_name} must contain one pinned QuadletLens coverage guard `{required}`"
                ));
            }
        }
    }

    Ok(())
}

#[test]
fn ci_workflow_enforces_portability_and_an_actionable_pr_gate() -> Result<(), String> {
    let workflow = read_repository_file(".github/workflows/ci.yml")?;

    for required in [
        "  portability:\n    name: Portability (macOS)",
        "runs-on: macos-14",
        "run: cargo ci-check",
        "run: cargo ci-test",
        "  pr-gate:\n    name: PR gate\n    if: always()",
        "needs: [rust, msrv, dependencies, api, documentation, coverage, portability]",
    ] {
        if !workflow.contains(required) {
            return Err(format!("CI workflow is missing contract `{required}`"));
        }
    }

    for (job_name, result_variable, needs_job) in [
        ("Rust quality", "RUST_RESULT", "rust"),
        ("MSRV", "MSRV_RESULT", "msrv"),
        ("Dependency and license policy", "DEPENDENCIES_RESULT", "dependencies"),
        ("Public API compatibility", "API_RESULT", "api"),
        ("Documentation", "DOCUMENTATION_RESULT", "documentation"),
        ("Coverage ratchet", "COVERAGE_RESULT", "coverage"),
        ("macOS portability", "PORTABILITY_RESULT", "portability"),
    ] {
        let required = format!("{result_variable}: ${{{{ needs.{needs_job}.result }}}}");
        if !workflow.contains(&required) {
            return Err(format!("PR gate does not expose a result variable for `{job_name}`"));
        }
    }

    for required in [
        "printf '| Job | Result |\\n'",
        "printf \"| %s | \\`%s\\` |\\n\" \"${name}\" \"${result}\" >> \"${GITHUB_STEP_SUMMARY}\"",
        "::error title=Required PR job did not succeed::${name} concluded ${result}.",
        "Required PR job did not succeed: ${name} concluded ${result}.",
        "if (( failures != 0 )); then",
        "One or more required PR jobs did not succeed; see the result table and annotations above.",
    ] {
        if !workflow.contains(required) {
            return Err(format!("PR gate is missing actionable failure diagnostic `{required}`"));
        }
    }
    if workflow.contains("test \"${{ needs.") {
        return Err("PR gate must not use opaque success test predicates".to_owned());
    }
    if workflow.contains("windows-") {
        return Err("CI must not claim unsupported native Windows portability".to_owned());
    }

    Ok(())
}

#[test]
fn release_workflow_rechecks_the_msrv() -> Result<(), String> {
    let workflow = read_repository_file(".github/workflows/release.yml")?;
    for required in [
        "- name: Read the workspace MSRV",
        "rustup toolchain install \"${RUST_MSRV}\" --profile minimal",
        "cargo \"+${RUST_MSRV}\" ci-check",
        "cargo \"+${RUST_MSRV}\" ci-policy",
    ] {
        if !workflow.contains(required) {
            return Err(format!("release workflow is missing MSRV guard `{required}`"));
        }
    }
    Ok(())
}

#[test]
fn local_developer_workflow_covers_deterministic_release_checks() -> Result<(), String> {
    let script = read_repository_file("scripts/check-all.sh")?;

    for required in [
        "list_existing_files",
        "cargo fmt --all",
        "bash scripts/check-files.sh --fix",
        "git --no-pager diff --check",
        "actionlint",
        "zizmor .github/workflows",
        "cargo ci-check",
        "cargo ci-catalogue",
        "cargo ci-model",
        "cargo ci-policy",
        "cargo ci-clippy",
        "cargo ci-test",
        "cargo ci-doctest",
        "cargo ci-doc",
        "cargo package --locked --allow-dirty",
        "cargo llvm-cov clean --locked",
        "cargo llvm-cov --locked --no-clean --workspace --all-features",
        "cargo \"+${msrv}\" ci-check",
        "cargo \"+${msrv}\" ci-policy",
        "cargo deny --all-features check",
        "lychee --config lychee.toml --root-dir . --offline",
        "validation_storage_root",
        "coverage_target_dir",
        "semver_cargo_home",
        "semver_target_dir",
        "${CARGO_TARGET_DIR:-${repository_root}/target}/check-all/quadlet-lens",
        "${validation_storage_root}/coverage",
        "${validation_storage_root}/cargo-home",
        "${validation_storage_root}/cargo-semver-checks-target",
        "env CARGO_TARGET_DIR=\"${coverage_target_dir}\"",
        "env CARGO_HOME=\"${semver_cargo_home}\"",
        "CARGO_TARGET_DIR=\"${semver_target_dir}\"",
        "cargo semver-checks check-release",
        "--package quadlet-lens",
    ] {
        if !script.contains(required) {
            return Err(format!("local validation runner missing `{required}`"));
        }
    }

    if script.contains("semver_cargo_home=\"${CARGO_HOME:-}\"") {
        return Err("local SemVer checks must not reuse ambient CARGO_HOME".to_owned());
    }

    if script.contains("--release-type") {
        return Err("local SemVer checks must derive the release type from Cargo versions".to_owned());
    }

    for opt_in in ["cargo ci-generators", "cargo ci-real-world-quadlet"] {
        if script.contains(opt_in) {
            return Err(format!(
                "local validation runner must not invoke opt-in tier `{opt_in}`"
            ));
        }
    }

    for (path, required) in [
        (
            ".vscode/settings.json",
            &["rust-analyzer.check.command", "editor.formatOnSave"][..],
        ),
        (
            ".vscode/extensions.json",
            &[
                "DavidAnson.vscode-markdownlint",
                "esbenp.prettier-vscode",
                "mkhl.shfmt",
                "tombi-toml.tombi",
                "timonwong.shellcheck",
            ][..],
        ),
        (
            ".vscode/tasks.json",
            &[
                "QuadletLens: Format, lint, and test all",
                "scripts/check-all.sh",
                "QuadletLens: Required Rust checks",
                "QuadletLens: Opt-in generator smoke lane",
                "QuadletLens: Package",
            ][..],
        ),
    ] {
        let contents = read_repository_file(path)?;
        for value in required {
            if !contents.contains(value) {
                return Err(format!("{path} is missing `{value}`"));
            }
        }
    }

    Ok(())
}

#[test]
fn non_rust_file_quality_is_locked_and_required() -> Result<(), String> {
    let script = read_repository_file("scripts/check-files.sh")?;
    for required in [
        "git ls-files --cached --others --exclude-standard",
        "list_existing_files",
        "markdownlint-cli2 --fix",
        "prettier --write",
        "prettier --check",
        "tombi format --check --offline",
        "tombi lint --error-on-warnings --offline",
        "shfmt -w",
        "shellcheck --",
        "hadolint",
    ] {
        if !script.contains(required) {
            return Err(format!("non-Rust file runner missing `{required}`"));
        }
    }

    let tombi = read_repository_file("tombi.toml")?;
    for required in [
        "dotted-keys-out-of-order = \"error\"",
        "key-empty = \"error\"",
        "tables-out-of-order = \"error\"",
        "enabled = false",
        "catalogue/**/*.toml",
        "fixtures/**/*.toml",
        "tools/**/*.toml",
    ] {
        if !tombi.contains(required) {
            return Err(format!("tombi.toml is missing `{required}`"));
        }
    }

    let lock = read_repository_file("package-lock.json")?;
    for package in ["markdownlint-cli2", "prettier"] {
        if !lock.contains(&format!("\"{package}\"")) {
            return Err(format!("package-lock.json must lock `{package}`"));
        }
    }

    for workflow_name in ["ci.yml", "release.yml"] {
        let workflow = read_repository_file(&format!(".github/workflows/{workflow_name}"))?;
        for required in [
            "npm ci --ignore-scripts",
            "bash scripts/install-file-tools.sh /usr/local/bin",
            "bash scripts/check-files.sh --check",
        ] {
            if !workflow.contains(required) {
                return Err(format!("{workflow_name} is missing `{required}`"));
            }
        }
    }

    Ok(())
}

#[test]
fn routine_link_checks_are_offline_and_external_checks_are_scheduled() -> Result<(), String> {
    let ci = read_repository_file(".github/workflows/ci.yml")?;
    for required in ["--config lychee.toml", "--offline"] {
        if !ci.contains(required) {
            return Err(format!("CI local-link check is missing `{required}`"));
        }
    }

    let external = read_repository_file(".github/workflows/documentation-links.yml")?;
    for required in ["schedule:", "workflow_dispatch:", "path: .lycheecache", "--cache"] {
        if !external.contains(required) {
            return Err(format!("scheduled external-link workflow is missing `{required}`"));
        }
    }

    Ok(())
}

#[test]
fn release_workflow_uses_the_create_response_as_its_draft_identity() -> Result<(), String> {
    let workflow_path = repository_root().join(".github/workflows/release.yml");
    let workflow = fs::read_to_string(&workflow_path)
        .map_err(|error| format!("failed to read {}: {error}", workflow_path.display()))?;

    if workflow.contains("/releases/tags/") {
        return Err("release workflow must not use the published-release-by-tag endpoint for drafts".to_owned());
    }
    if workflow.contains("databaseId") {
        return Err("release workflow must use stable REST release fields instead of CLI JSON fields".to_owned());
    }
    if workflow.contains("gh release create") || workflow.contains("gh release list") {
        return Err(
            "release workflow must not rediscover a newly created draft through high-level CLI commands".to_owned(),
        );
    }

    for required in [
        "RELEASE_GITHUB_API_VERSION: \"2026-03-10\"",
        "repos/${GITHUB_REPOSITORY}/releases?per_page=100",
        "gh api --method POST",
        "target_commitish: $target",
        "'.upload_url | sub(",
        "steps.release.outputs.upload_url",
        "--data-binary \"@${asset_path}\"",
        "steps.release.outputs.release_id",
    ] {
        if !workflow.contains(required) {
            return Err(format!(
                "release workflow is missing the draft release-ID guard `{required}`"
            ));
        }
    }

    let release_list_endpoint = "repos/${GITHUB_REPOSITORY}/releases?per_page=100";
    if workflow.matches(release_list_endpoint).count() != 1 {
        return Err(
            "release workflow must list releases only before creation and never rediscover the new draft".to_owned(),
        );
    }

    Ok(())
}

#[test]
fn release_workflow_uses_only_trusted_publishing() -> Result<(), String> {
    let workflow_path = repository_root().join(".github/workflows/release.yml");
    let workflow = fs::read_to_string(&workflow_path)
        .map_err(|error| format!("failed to read {}: {error}", workflow_path.display()))?;

    for forbidden in [
        "CRATES_IO_API_TOKEN",
        "CRATES_IO_BOOTSTRAP_TOKEN",
        "cargo login",
        "--token",
        "secrets.",
    ] {
        if workflow.contains(forbidden) {
            return Err(format!(
                "release workflow contains the forbidden long-lived credential path `{forbidden}`"
            ));
        }
    }

    for required in [
        "id-token: write",
        "rust-lang/crates-io-auth-action@",
        "CARGO_REGISTRY_TOKEN: ${{ steps.crates-auth.outputs.token }}",
        "cargo publish --locked",
    ] {
        if !workflow.contains(required) {
            return Err(format!(
                "release workflow is missing the trusted-publishing guard `{required}`"
            ));
        }
    }

    if workflow.matches("cargo publish --locked").count() != 1 {
        return Err("release workflow must contain exactly one publication command".to_owned());
    }

    Ok(())
}

#[test]
fn release_plz_prepares_only_guarded_releases() -> Result<(), String> {
    validate_release_plz_contract("Strukturpiloten/quadlet-lens")
}

#[test]
fn release_note_extraction_is_strict_and_bounded() -> Result<(), String> {
    validate_release_note_extraction("quadlet-lens")
}

fn validate_release_plz_contract(repository: &str) -> Result<(), String> {
    if repository_root().join("docs/releases").exists() {
        return Err("CHANGELOG.md must remain the only release-history source".to_owned());
    }
    let config_text = read_repository_file("release-plz.toml")?;
    let config = toml::from_str::<toml::Value>(&config_text)
        .map_err(|error| format!("failed to parse release-plz.toml: {error}"))?;
    let workspace = config["workspace"]
        .as_table()
        .ok_or_else(|| "release-plz.toml must contain [workspace]".to_owned())?;
    for (name, expected) in [
        ("allow_dirty", false),
        ("changelog_update", true),
        ("dependencies_update", false),
        ("git_release_enable", false),
        ("git_tag_enable", false),
        ("publish", false),
        ("release_always", false),
        ("semver_check", true),
    ] {
        if workspace.get(name).and_then(toml::Value::as_bool) != Some(expected) {
            return Err(format!("release-plz workspace setting {name} must be {expected}"));
        }
    }
    if workspace.get("changelog_path").and_then(toml::Value::as_str) != Some("CHANGELOG.md")
        || workspace.get("pr_branch_prefix").and_then(toml::Value::as_str) != Some("release-plz-")
    {
        return Err("release-plz must use the root changelog and guarded branch prefix".to_owned());
    }

    let workflow = read_repository_file(".github/workflows/release-plz.yml")?;
    for required in [
        repository,
        "secrets.RELEASE_PLZ_APP_ID",
        "secrets.RELEASE_PLZ_APP_PRIVATE_KEY",
        "permission-contents: write",
        "permission-pull-requests: write",
        "command: release-pr",
        "version: \"0.3.160\"",
        "release-plz/action@2eb1d8bcb770b4c48ccfaad919734b38b51958c9 # v0.5.131",
        "actions/create-github-app-token@bcd2ba49218906704ab6c1aa796996da409d3eb1 # v3.2.0",
        "(.head.ref | startswith(\"release-plz-\"))",
        "actions/workflows/release.yml/dispatches",
        "actions: write",
        "No release was dispatched.",
    ] {
        if !workflow.contains(required) {
            return Err(format!("release-plz workflow is missing `{required}`"));
        }
    }
    for forbidden in ["command: release\n", "cargo publish", "git tag", "gh release create"] {
        if workflow.contains(forbidden) {
            return Err(format!("release-plz workflow must not contain `{forbidden}`"));
        }
    }

    let release = read_repository_file(".github/workflows/release.yml")?;
    if release.contains("docs/releases/${version}.md") || !release.contains("bash scripts/extract-release-notes.sh") {
        return Err("protected publication must derive release notes from CHANGELOG.md".to_owned());
    }
    Ok(())
}

fn validate_release_note_extraction(repository: &str) -> Result<(), String> {
    let root = repository_root();
    let directory = std::env::temp_dir().join(format!("{repository}-release-notes-{}", std::process::id()));
    let changelog = directory.join("CHANGELOG.md");
    fs::create_dir_all(&directory).map_err(|error| format!("failed to create {}: {error}", directory.display()))?;
    fs::write(
        &changelog,
        "# Changelog\n\n## [Unreleased]\n\n## [1.2.3](https://example.invalid/v1.2.3) - 2026-08-17\n\n### Added\n\n- Useful change.\n\n## [1.2.2] - 2026-08-16\n\n- Older change.\n",
    )
    .map_err(|error| format!("failed to write {}: {error}", changelog.display()))?;
    let valid = run_release_notes_script(&root, "1.2.3", &changelog)?;
    let valid_stdout = String::from_utf8(valid.stdout).map_err(|error| error.to_string())?;
    if !valid.status.success() || !valid_stdout.contains("Useful change") || valid_stdout.contains("Older change") {
        return Err("valid release notes were not extracted as one bounded section".to_owned());
    }

    let missing = run_release_notes_script(&root, "9.9.9", &changelog)?;
    if missing.status.success() || !String::from_utf8_lossy(&missing.stderr).contains("no release section") {
        return Err("a missing release section must fail with an actionable diagnostic".to_owned());
    }
    let malformed_version = run_release_notes_script(&root, "v1.2.3", &changelog)?;
    if malformed_version.status.success()
        || !String::from_utf8_lossy(&malformed_version.stderr).contains("major.minor.patch")
    {
        return Err("a malformed release version must fail before extraction".to_owned());
    }

    fs::write(
        &changelog,
        "# Changelog\n\n## [1.2.3] - 2026-08-17\n\n## [1.2.2] - 2026-08-16\n",
    )
    .map_err(|error| format!("failed to write {}: {error}", changelog.display()))?;
    let empty = run_release_notes_script(&root, "1.2.3", &changelog)?;
    if empty.status.success() || !String::from_utf8_lossy(&empty.stderr).contains("is empty") {
        return Err("an empty release section must fail".to_owned());
    }

    fs::write(&changelog, "# Changelog\n\n## [1.2.3] - not-a-date\n\n- Change.\n")
        .map_err(|error| format!("failed to write {}: {error}", changelog.display()))?;
    let malformed_heading = run_release_notes_script(&root, "1.2.3", &changelog)?;
    if malformed_heading.status.success() || !String::from_utf8_lossy(&malformed_heading.stderr).contains("YYYY-MM-DD")
    {
        return Err("a malformed release heading must fail".to_owned());
    }

    fs::remove_dir_all(&directory).map_err(|error| format!("failed to remove {}: {error}", directory.display()))?;
    Ok(())
}

fn run_release_notes_script(root: &Path, version: &str, changelog: &Path) -> Result<Output, String> {
    Command::new("bash")
        .arg(root.join("scripts/extract-release-notes.sh"))
        .arg(version)
        .arg(changelog)
        .current_dir(root)
        .output()
        .map_err(|error| format!("failed to run release-note extractor: {error}"))
}

#[test]
fn fixture_manifests_follow_the_common_contract() -> Result<(), String> {
    support::validate_fixture_tree(&repository_root(), FIXTURE_SUITES)
}

#[test]
fn real_world_quadlet_catalog_is_immutable_and_reviewed() -> Result<(), String> {
    support::validate_real_world_quadlet_catalog(&repository_root())
}

#[test]
fn fixture_contract_accepts_authored_metadata() {
    let errors = support::validate_fixture_manifest_text(
        "valid fixture",
        r#"
schema = 1
id = "minimal-container"
suite = "syntax"
description = "Protects a minimal container unit."
secrets_reviewed = true
files = ["example.container"]

[provenance]
source = "authored"
license = "MPL-2.0"
redistribution = "allowed"
modifications = "none"

[environment]
description = "No generator environment is provided."

[expectations]
summary = "The container unit remains present."
"#,
        FIXTURE_SUITES,
    );

    assert!(errors.is_empty(), "{errors:#?}");
}

#[test]
fn fixture_contract_rejects_unsafe_external_metadata() {
    let errors = support::validate_fixture_manifest_text(
        "invalid fixture",
        r#"
schema = 1
id = "external-project"
suite = "real-world"
description = "An incomplete external fixture."
secrets_reviewed = false
files = ["../secret.env"]

[provenance]
source = "external"
license = "unknown"
redistribution = "allowed"
modifications = "none"

[environment]
description = "Unknown."

[expectations]
summary = "Must not be accepted."
"#,
        FIXTURE_SUITES,
    );

    assert!(
        errors.iter().any(|error| error.contains("secrets_reviewed")),
        "{errors:#?}"
    );
    assert!(
        errors.iter().any(|error| error.contains("unsafe fixture path")),
        "{errors:#?}"
    );
    assert!(errors.iter().any(|error| error.contains("`url`")), "{errors:#?}");
    assert!(errors.iter().any(|error| error.contains("`revision`")), "{errors:#?}");
}

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_repository_file(path: &str) -> Result<String, String> {
    let path = repository_root().join(path);
    fs::read_to_string(&path).map_err(|error| format!("failed to read {}: {error}", path.display()))
}
