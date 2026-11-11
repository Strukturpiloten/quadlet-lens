//! Executable repository and fixture-contract checks.

mod support;

use std::{
    collections::BTreeSet,
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
fn ci_runs_once_per_pull_request_update_and_on_main_pushes() -> Result<(), String> {
    let workflow_path = repository_root().join(".github/workflows/ci.yml");
    let workflow = fs::read_to_string(&workflow_path)
        .map_err(|error| format!("failed to read {}: {error}", workflow_path.display()))?;
    validate_ci_trigger_contract(&workflow)
}

fn validate_ci_trigger_contract(workflow: &str) -> Result<(), String> {
    let expected = "on:\n  push:\n    branches:\n      - main\n  pull_request:\n  workflow_dispatch:\n";
    if !workflow.contains(expected) {
        return Err(
            "CI must run for main pushes, pull requests, and manual dispatch without duplicate feature-branch push runs"
                .to_owned(),
        );
    }

    if workflow.lines().filter(|line| *line == "on:").count() != 1 {
        return Err("CI must declare one event boundary".to_owned());
    }
    for event in [
        "push",
        "pull_request",
        "workflow_dispatch",
        "workflow_call",
        "merge_group",
    ] {
        let declaration = format!("  {event}:");
        if workflow.lines().filter(|line| *line == declaration.as_str()).count() != 1 {
            return Err(format!("CI must retain exactly one {event} event"));
        }
    }
    Ok(())
}

#[test]
fn repository_supply_chain_has_single_sources_and_immutable_pins() -> Result<(), String> {
    support::validate_repository_supply_chain(&repository_root())
}

#[test]
fn rust_minimum_matches_clippy_and_public_claims() -> Result<(), String> {
    let parse = |path| {
        toml::from_str::<toml::Value>(&read_repository_file(path)?)
            .map_err(|error| format!("invalid TOML in {path}: {error}"))
    };
    let manifest = parse("Cargo.toml")?;
    let minimum = manifest["workspace"]["package"]["rust-version"]
        .as_str()
        .ok_or("workspace package must declare rust-version")?;
    if manifest["package"]["rust-version"]["workspace"].as_bool() != Some(true) {
        return Err("published package must inherit the workspace Rust minimum".to_owned());
    }
    if exact_rust_release(minimum)? < [1, 98, 1] {
        return Err("Rust minimum must retain the approved 1.98.1 baseline".to_owned());
    }
    let clippy = parse("clippy.toml")?;
    if clippy["msrv"].as_str() != Some(minimum) {
        return Err("Clippy MSRV must match the workspace Rust minimum".to_owned());
    }
    let toolchain = parse("rust-toolchain.toml")?;
    let channel = toolchain["toolchain"]["channel"]
        .as_str()
        .ok_or("normal Rust toolchain must declare its channel")?;
    if exact_rust_release(channel)? < exact_rust_release(minimum)? {
        return Err("normal Rust toolchain must support the workspace Rust minimum".to_owned());
    }
    for path in ["README.md", "docs/api-stability.md", "docs/public/index.md"] {
        if !read_repository_file(path)?.contains(&format!("Rust {minimum} or newer")) {
            return Err(format!("{path} must document the workspace Rust minimum"));
        }
    }
    let guidance = read_repository_file("AGENTS.md")?;
    for alias in ["ci-check", "ci-policy"] {
        if !guidance.contains(&format!("cargo +{minimum} {alias}")) {
            return Err(format!("AGENTS.md must use the workspace Rust minimum for {alias}"));
        }
    }
    Ok(())
}

#[test]
fn ci_trigger_contract_rejects_removed_events_and_feature_branch_pushes() -> Result<(), String> {
    let workflow = read_repository_file(".github/workflows/ci.yml")?;
    validate_ci_trigger_contract(&workflow)?;
    for event in ["pull_request", "workflow_dispatch", "workflow_call", "merge_group"] {
        let changed = workflow.replacen(&format!("  {event}:\n"), "", 1);
        assert_ne!(changed, workflow);
        assert!(
            validate_ci_trigger_contract(&changed).is_err(),
            "accepted missing {event}"
        );
    }
    let extra_push = workflow.replacen("      - main\n", "      - main\n      - feature\n", 1);
    assert!(validate_ci_trigger_contract(&extra_push).is_err());
    assert!(validate_ci_trigger_contract(&(workflow + "\non:\n")).is_err());
    Ok(())
}

fn exact_rust_release(version: &str) -> Result<[u32; 3], String> {
    version
        .split('.')
        .map(str::parse::<u32>)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Rust release {version:?} must be exact: {error}"))?
        .try_into()
        .map_err(|_| format!("Rust release {version:?} must contain major.minor.patch"))
}

#[test]
fn rust_release_comparison_preserves_patch_and_minor_boundaries() -> Result<(), String> {
    let baseline = exact_rust_release("1.98.1")?;
    assert!(exact_rust_release("1.98.0")? < baseline);
    assert!(exact_rust_release("1.98.2")? > baseline);
    assert!(exact_rust_release("1.99.0")? > baseline);
    assert!(exact_rust_release("1.100.0")? > baseline);
    for unsupported in ["stable", "1.98", "1.98.1.0", "1.98.1-nightly"] {
        assert!(exact_rust_release(unsupported).is_err());
    }
    Ok(())
}

#[test]
fn public_api_compatibility_runs_in_reusable_ci() -> Result<(), String> {
    let workflow = read_repository_file(".github/workflows/ci.yml")?;
    let manifest: serde_json::Value =
        serde_json::from_str(&read_repository_file(".github/automation.json")?).map_err(|error| error.to_string())?;
    let revision = manifest["revision"].as_str().ok_or("missing automation revision")?;
    let reference = format!("shared-rust-semver.yml@{revision} # automation-v1 interface v1");
    if workflow.matches(&reference).count() != 1 {
        return Err("API must have one manifest-bound shared SemVer owner".to_owned());
    }
    for required in [
        "package: quadlet-lens",
        "api_purpose: ${{ inputs.api_purpose || 'development' }}",
        "PUBLISHED_API_CANDIDATE_SHA: ${{ needs.api.outputs.published_api_candidate_sha }}",
        "test \"${PUBLISHED_API_CANDIDATE_SHA}\" = \"${GITHUB_SHA}\"",
    ] {
        if !workflow.contains(required) {
            return Err(format!("shared API contract is missing `{required}`"));
        }
    }
    if workflow.contains("release_type:") || workflow.contains("initial_api_contract:") {
        return Err("published QuadletLens must derive API baselines without initial-publication bypasses".to_owned());
    }
    Ok(())
}

#[test]
fn coverage_ratchet_runs_in_reusable_ci() -> Result<(), String> {
    let workflow = read_repository_file(".github/workflows/ci.yml")?;
    let revision: serde_json::Value =
        serde_json::from_str(&read_repository_file(".github/automation.json")?).map_err(|error| error.to_string())?;
    let revision = revision["revision"].as_str().ok_or("missing automation revision")?;
    let coverage = workflow
        .split("\n  coverage:\n")
        .nth(1)
        .and_then(|job| job.split("\n  msrv:\n").next())
        .ok_or("missing coverage job")?;
    for required in [
        format!("shared-rust-ci.yml@{revision} # automation-v1 interface v1"),
        format!("automation_sha: {revision}"),
        "task: coverage".to_owned(),
        "require_configured_task: true".to_owned(),
        "candidate_sha: ${{ github.sha }}".to_owned(),
        "policy_sha: ${{ github.event.pull_request.base.sha || github.event.merge_group.base_sha || github.sha }}"
            .to_owned(),
        "config_path: scripts/validation-policy.json".to_owned(),
    ] {
        if coverage.matches(&required).count() != 1 {
            return Err(format!("coverage must retain one shared task binding `{required}`"));
        }
    }
    if coverage.contains("steps:")
        || coverage.contains("runs-on:")
        || workflow.contains("cargo install --locked --version")
    {
        return Err("coverage must delegate execution without a duplicate workflow tool declaration".to_owned());
    }
    let policy: serde_json::Value = serde_json::from_str(&read_repository_file("scripts/validation-policy.json")?)
        .map_err(|error| error.to_string())?;
    if policy["tasks"]["coverage"]
        != serde_json::json!({
            "commands": [
                ["cargo", "llvm-cov", "clean", "--locked"],
                ["cargo", "llvm-cov", "--locked", "--no-clean", "--workspace", "--all-features", "--all-targets", "--summary-only", "--fail-under-regions", "91", "--fail-under-functions", "92", "--fail-under-lines", "92"]
            ],
            "timeout_seconds": 1800
        })
    {
        return Err("coverage task must preserve exact cleanup, measurement, ratchets and budget".to_owned());
    }
    pinned_cargo_llvm_cov_version(
        &read_repository_file(".devcontainer/Dockerfile")?,
        ".devcontainer/Dockerfile",
    )?;
    Ok(())
}

fn pinned_cargo_llvm_cov_version(document: &str, source: &str) -> Result<String, String> {
    const DEVCONTAINER_PREFIX: &str = "ARG CARGO_LLVM_COV_VERSION=";

    let versions = document
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            line.strip_prefix(DEVCONTAINER_PREFIX)
        })
        .collect::<Vec<_>>();

    if versions.len() != 1 {
        return Err(format!(
            "{source} must contain exactly one cargo-llvm-cov version pin, found {}",
            versions.len()
        ));
    }

    let version = versions[0];
    let components = version.split('.').collect::<Vec<_>>();
    if components.len() != 3
        || components
            .iter()
            .any(|component| component.is_empty() || !component.bytes().all(|byte| byte.is_ascii_digit()))
    {
        return Err(format!(
            "{source} must pin cargo-llvm-cov to an exact major.minor.patch version, found `{version}`"
        ));
    }

    Ok(version.to_owned())
}

#[test]
fn ci_workflow_uses_trusted_change_plan_and_fail_closed_gate() -> Result<(), String> {
    let workflow = read_repository_file(".github/workflows/ci.yml")?;
    for required in [
        "  validation-plan:\n    name: Validation plan",
        "python -m boxferry_ci plan",
        "python -m boxferry_ci gate",
        "--policy-sha",
        "--tested-sha",
        "--automation-sha",
        "--plan-json",
        "--needs-json",
        "if: needs.validation-plan.outputs.select_rust == 'true'",
        "if: needs.validation-plan.outputs.select_documentation == 'true'",
        "  pr-gate:\n    name: Required CI\n    if: always()",
        "uses: ./_automation/.github/actions/shared-source",
        "uses: ./_automation/.github/actions/shared-python",
        "shared-dependency-wait.yml@",
        "shared-devcontainer-validation.yml@",
        "  devcontainer-validation:",
        "if: needs.validation-plan.outputs.select_devcontainer_validation != 'false'",
        "select_devcontainer_validation: ${{ steps.plan.outputs.select_devcontainer_validation }}",
        "--devcontainer-validation",
    ] {
        if !workflow.contains(required) {
            return Err(format!("CI shared trusted-candidate contract is missing `{required}`"));
        }
    }
    if workflow.matches("    name: Required CI\n").count() != 1
        || workflow.contains("continue-on-error: true")
        || workflow.contains("runs-on: macos-")
        || workflow.contains("runs-on: windows-")
        || workflow.contains("scripts/validation-plan.py")
    {
        return Err("CI must retain one fail-closed shared aggregate on evidenced Linux runners".to_owned());
    }
    let policy: serde_json::Value = serde_json::from_str(&read_repository_file("scripts/validation-policy.json")?)
        .map_err(|error| error.to_string())?;
    if policy["jobs"]
        != serde_json::json!([
            "rust",
            "coverage",
            "msrv",
            "dependencies",
            "api",
            "documentation",
            "lockfile-release-age",
            "devcontainer-validation"
        ])
    {
        return Err("native image qualification must preserve every original validation job".to_owned());
    }
    Ok(())
}

#[test]
fn release_workflow_rechecks_the_msrv() -> Result<(), String> {
    let workflow = read_repository_file(".github/workflows/release.yml")?;
    if !workflow.contains("uses: ./.github/workflows/ci.yml") {
        return Err("Release must reuse CI's MSRV validation".to_owned());
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
        "python3 scripts/shared-ci.py run --event local --task api-development",
        "--task api-development",
        "cargo-tools verify",
        "--repository-root",
        "--package cargo-semver-checks",
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
fn real_application_contracts_have_distinct_deterministic_and_generator_gates() -> Result<(), String> {
    let cargo_aliases = read_repository_file(".cargo/config.toml")?;
    for required in [
        "ci-application = \"test --locked --test application_conformance\"",
        "ci-application-generators = \"test --locked --test generators application_document_sets_match_pinned_generator_contracts -- --exact --ignored --nocapture\"",
        "ci-generators = \"test --locked --test generators supported_generators_match_the_first_conversion_fixture -- --exact --ignored --nocapture\"",
    ] {
        if cargo_aliases.matches(required).count() != 1 {
            return Err(format!("Cargo aliases must contain exactly one `{required}`"));
        }
    }

    let local = read_repository_file("scripts/check-all.sh")?;
    if local.matches("cargo ci-application\n").count() != 1 {
        return Err("local validation must run deterministic application contracts exactly once".to_owned());
    }
    let workflow = read_repository_file(".github/workflows/ci.yml")?;
    let automation: serde_json::Value =
        serde_json::from_str(&read_repository_file(".github/automation.json")?).map_err(|error| error.to_string())?;
    let revision = automation["revision"].as_str().ok_or("missing automation revision")?;
    let policy: serde_json::Value = serde_json::from_str(&read_repository_file("scripts/validation-policy.json")?)
        .map_err(|error| error.to_string())?;
    validate_hosted_application_contract(&workflow, revision, &policy)?;
    let path = ".github/workflows/generator-matrix.yml";
    let contents = read_repository_file(path)?;
    if contents.matches("cargo ci-application-generators").count() != 1 {
        return Err(format!("{path} must run pinned application generators exactly once"));
    }
    let release = read_repository_file(".github/workflows/release.yml")?;
    if !release.contains("uses: ./.github/workflows/ci.yml")
        || !release.contains("uses: ./.github/workflows/generator-matrix.yml")
    {
        return Err("Release must reuse deterministic and native generator validation".to_owned());
    }

    let manifest = read_repository_file("Cargo.toml")?;
    let lockfile = read_repository_file("Cargo.lock")?;
    if manifest.lines().any(|line| line.trim_start().starts_with("boxferry ="))
        || lockfile.contains("name = \"boxferry\"")
    {
        return Err("QuadletLens production dependencies must not include BoxFerry".to_owned());
    }
    Ok(())
}

fn validate_hosted_application_contract(
    workflow: &str,
    revision: &str,
    policy: &serde_json::Value,
) -> Result<(), String> {
    if workflow.matches("\n  rust:\n").count() != 1 || workflow.contains("cargo ci-application\n") {
        return Err(
            "CI must have one shared Rust quality owner without duplicate inline application checks".to_owned(),
        );
    }
    let rust = workflow
        .split("\n  rust:\n")
        .nth(1)
        .and_then(|job| job.split("\n  coverage:\n").next())
        .ok_or("missing Rust quality job")?;
    for required in [
        format!(
            "    uses: Strukturpiloten/boxferry/.github/workflows/shared-rust-ci.yml@{revision} # automation-v1 interface v1\n"
        ),
        format!("      automation_sha: {revision}\n"),
        "      config_path: scripts/validation-policy.json\n".to_owned(),
        "      task: rust-quality\n".to_owned(),
        "      require_configured_task: true\n".to_owned(),
    ] {
        if rust
            .lines()
            .filter(|line| *line == required.trim_end_matches('\n'))
            .count()
            != 1
        {
            return Err(format!(
                "application contracts require the exact configured Rust caller `{required}`"
            ));
        }
    }
    let commands = policy["tasks"]["rust-quality"]["commands"]
        .as_array()
        .ok_or("Rust quality must explicitly configure its application command")?;
    let application = serde_json::json!(["cargo", "ci-application"]);
    if commands.iter().filter(|command| *command == &application).count() != 1 {
        return Err("shared Rust quality must run the exact deterministic application command once".to_owned());
    }
    if commands.iter().any(|command| {
        command.as_array().is_some_and(|argv| {
            argv.iter()
                .any(|arg| matches!(arg.as_str(), Some("ci-generators" | "ci-application-generators")))
        })
    }) {
        return Err("native generator commands must remain outside deterministic Rust quality".to_owned());
    }
    Ok(())
}

#[test]
fn hosted_application_contract_rejects_missing_duplicate_and_native_commands() -> Result<(), String> {
    let workflow = read_repository_file(".github/workflows/ci.yml")?;
    let automation: serde_json::Value =
        serde_json::from_str(&read_repository_file(".github/automation.json")?).map_err(|error| error.to_string())?;
    let revision = automation["revision"].as_str().ok_or("missing automation revision")?;
    let policy: serde_json::Value = serde_json::from_str(&read_repository_file("scripts/validation-policy.json")?)
        .map_err(|error| error.to_string())?;
    validate_hosted_application_contract(&workflow, revision, &policy)?;
    for commands in [
        serde_json::json!([["cargo", "ci-test"]]),
        serde_json::json!([["cargo", "ci-application", "--ignored"]]),
        serde_json::json!([["cargo", "ci-application"], ["cargo", "ci-application"]]),
        serde_json::json!([["cargo", "ci-application"], ["cargo", "ci-generators"]]),
        serde_json::json!([["cargo", "ci-application"], ["cargo", "ci-application-generators"]]),
    ] {
        let mut changed = policy.clone();
        changed["tasks"]["rust-quality"]["commands"] = commands;
        assert!(validate_hosted_application_contract(&workflow, revision, &changed).is_err());
    }
    let mut missing = policy.clone();
    missing["tasks"]
        .as_object_mut()
        .ok_or("missing tasks")?
        .remove("rust-quality");
    assert!(validate_hosted_application_contract(&workflow, revision, &missing).is_err());
    for (before, after) in [
        ("task: rust-quality", "task: coverage"),
        ("require_configured_task: true", "require_configured_task: false"),
        ("shared-rust-ci.yml@", "unreviewed-rust-ci.yml@"),
        (
            "config_path: scripts/validation-policy.json",
            "config_path: candidate.json",
        ),
    ] {
        let changed = workflow.replacen(before, after, 1);
        assert_ne!(changed, workflow);
        assert!(validate_hosted_application_contract(&changed, revision, &policy).is_err());
    }
    assert!(validate_hosted_application_contract(&workflow, &"b".repeat(40), &policy).is_err());
    Ok(())
}

#[test]
fn issue_to_pr_workflow_requires_primary_ownership_and_the_complete_local_gate() -> Result<(), String> {
    for (path, required) in [
        (
            "AGENTS.md",
            &[
                "## GitHub issue-to-PR workflow",
                "Run `./scripts/check-all.sh`",
                "hard gate against commit, push",
                "primary agent owns Git and GitHub writes",
                "Worker subagents",
                "never execute the Git or GitHub",
                "remains the primary agent's responsibility",
            ][..],
        ),
        (
            "docs/development-environment.md",
            &[
                "## Issue-to-PR contribution workflow",
                "./scripts/check-all.sh",
                "All steps must pass before the change is committed, pushed, or submitted",
                "primary agent uses `gpt-6-sol` with `xhigh` reasoning",
                "Worker agents",
                "never perform Git or GitHub writes",
                "the primary agent's final responsibility",
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
fn standing_github_authorization_has_exact_workspace_and_merge_boundaries() -> Result<(), String> {
    let instructions = read_repository_file("AGENTS.md")?;
    let scope = instructions
        .split_once("## Workspace scope and standing GitHub authorization")
        .ok_or("AGENTS.md is missing standing authorization")?
        .1
        .split("\n## ")
        .next()
        .ok_or("AGENTS.md has no authorization section")?;
    let allowed_repositories: Vec<_> = scope.lines().filter(|line| line.starts_with("- ")).collect();
    assert_eq!(
        allowed_repositories,
        [
            "- `Strukturpiloten/boxferry`",
            "- `Strukturpiloten/compose-lens`",
            "- `Strukturpiloten/podman-lens`",
            "- `Strukturpiloten/quadlet-lens`",
            "- `Strukturpiloten/boxferry-website`",
            "- `Strukturpiloten/docker-lens`",
        ]
    );
    for required in [
        "task-related Git and GitHub work only in these",
        "Do not work on or modify any repository outside this explicit allowlist",
        "A newly discovered checkout is not implicitly",
        "For user-requested work within this scope",
        "merge verified task-related pull requests without asking for renewed",
        "independently reviewed, and has every required check successful",
        "never bypass branch protection or use an administrator",
        "does not authorize releases, publication, deployment operations",
        "merging release/publication/deployment pull requests",
        "Subagents remain within their assigned task and",
    ] {
        if !scope.contains(required) {
            return Err(format!("AGENTS.md authorization is missing `{required}`"));
        }
    }
    for stale in [
        "Merge only when the user explicitly authorizes",
        "Opening and reading back the ready pull request is the default stopping point",
    ] {
        if instructions.contains(stale) {
            return Err(format!("AGENTS.md retains obsolete approval text `{stale}`"));
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
        "check_yaml_document_markers",
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
        "docs/schemas/tombi-cargo-offline.schema.json",
        "include = [\"Cargo.toml\", \"**/Cargo.toml\"]",
        "enabled = false",
        "catalogue/**/*.toml",
        "fixtures/**/*.toml",
        "tools/**/*.toml",
    ] {
        if !tombi.contains(required) {
            return Err(format!("tombi.toml is missing `{required}`"));
        }
    }

    let cargo_schema = read_repository_file("docs/schemas/tombi-cargo-offline.schema.json")?;
    for required in [r#""type": "object""#, r#""additionalProperties": true"#] {
        if !cargo_schema.contains(required) {
            return Err(format!("offline Cargo schema must contain `{required}`"));
        }
    }

    let prettier_ignore = read_repository_file(".prettierignore")?;
    if prettier_ignore
        .lines()
        .filter(|line| !line.starts_with('#') && !line.is_empty())
        .collect::<Vec<_>>()
        != ["/CHANGELOG.md"]
    {
        return Err("only the release-plz-owned CHANGELOG.md may be excluded from Prettier".to_owned());
    }
    for required in [
        r#"prettier --write --ignore-path .prettierignore --ignore-unknown "${markdown_files[@]}""#,
        r#"markdownlint-cli2 --fix "${markdown_literals[@]}""#,
        r#"markdownlint-cli2 "${markdown_literals[@]}""#,
        r#"prettier --check --ignore-path .prettierignore --ignore-unknown "${markdown_files[@]}""#,
    ] {
        if !script.contains(required) {
            return Err(format!(
                "non-Rust file runner must preserve generated-changelog boundary `{required}`"
            ));
        }
    }

    let lock = read_repository_file("package-lock.json")?;
    for package in ["markdownlint-cli2", "prettier"] {
        if !lock.contains(&format!("\"{package}\"")) {
            return Err(format!("package-lock.json must lock `{package}`"));
        }
    }

    for workflow_name in ["ci.yml"] {
        let workflow = read_repository_file(&format!(".github/workflows/{workflow_name}"))?;
        for required in [
            "npm ci --ignore-scripts",
            r#"tool_dir="${RUNNER_TEMP}/quadlet-lens-tools""#,
            r#"bash scripts/install-file-tools.sh "${tool_dir}""#,
            r#"python3 scripts/shared-ci.py tools --destination "${tool_dir}" --tools actionlint lychee"#,
            r#"printf '%s\n' "${tool_dir}" >> "${GITHUB_PATH}""#,
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
fn complete_yaml_documents_use_explicit_start_markers() -> Result<(), String> {
    let root = repository_root();
    let output = Command::new("git")
        .args(["ls-files", "-z", "--", "*.yaml", "*.yml"])
        .current_dir(&root)
        .output()
        .map_err(|error| format!("failed to list YAML documents: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "git ls-files failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    for path in output.stdout.split(|byte| *byte == 0).filter(|path| !path.is_empty()) {
        let path = Path::new(std::str::from_utf8(path).map_err(|error| error.to_string())?);
        let contents = fs::read_to_string(root.join(path))
            .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
        if contents.lines().next() != Some("---") {
            return Err(format!("{} must start with `---`", path.display()));
        }
    }

    Ok(())
}

#[test]
fn offline_manual_evidence_uses_linux_and_macos_portable_tool_interfaces() -> Result<(), String> {
    let test_source = read_repository_file("tests/specification_inventory.rs")?;
    for required in [
        "base64 --decode | gzip -d -c",
        ".stdin(evidence_file)",
        "Command::new(\"shasum\")",
        ".args([\"-a\", \"256\"])",
    ] {
        if !test_source.contains(required) {
            return Err(format!(
                "offline manual evidence check is missing portable form `{required}`"
            ));
        }
    }
    for forbidden in ["base64 --decode --", "Command::new(\"sha256sum\")"] {
        if test_source.contains(forbidden) {
            return Err(format!(
                "offline manual evidence check retains GNU-only form `{forbidden}`"
            ));
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
    let workflow = read_repository_file(".github/workflows/release.yml")?;
    for forbidden in [
        "CRATES_IO_API_TOKEN",
        "CRATES_IO_BOOTSTRAP_TOKEN",
        "cargo login",
        "--token",
        "secrets.",
        "cargo publish --locked",
    ] {
        if workflow.contains(forbidden) {
            return Err(format!(
                "consumer Release must not own credential or publication implementation `{forbidden}`"
            ));
        }
    }
    for required in [
        "id-token: write",
        "environment: release",
        "uses: ./_automation/.github/actions/shared-crate-publish",
        "validation-sha: ${{ needs.release-gate.outputs.candidate_sha }}",
        "validation-result: ${{ needs.release-gate.result }}",
        "validation-only: \"true\"",
        "validation-only: \"false\"",
        "api_purpose: published",
    ] {
        if !workflow.contains(required) {
            return Err(format!("shared trusted publication boundary is missing `{required}`"));
        }
    }
    if workflow
        .matches("uses: ./_automation/.github/actions/shared-crate-publish")
        .count()
        != 2
    {
        return Err("Release needs one read-only preparation and one protected publication caller".to_owned());
    }
    Ok(())
}

#[test]
fn release_plz_prepares_only_guarded_releases() -> Result<(), String> {
    validate_release_plz_contract("Strukturpiloten/quadlet-lens")
}

#[test]
fn release_metadata_context_is_explicit_in_every_gate() -> Result<(), String> {
    let local = read_repository_file("scripts/check-all.sh")?;
    if !local.contains("bash scripts/check-release-metadata.sh --development") {
        return Err("complete local validation must use development release metadata".to_owned());
    }
    let ci = read_repository_file(".github/workflows/ci.yml")?;
    for required in [
        "run: bash scripts/test-release-metadata.sh",
        "RELEASE_METADATA_MODE: ${{ github.event_name == 'pull_request' && startsWith(github.head_ref, 'release-plz-') && '--release' || '--development' }}",
        "run: bash scripts/check-release-metadata.sh \"${RELEASE_METADATA_MODE}\"",
    ] {
        if !ci.contains(required) {
            return Err(format!("CI release metadata context is missing `{required}`"));
        }
    }
    let release = read_repository_file(".github/workflows/release.yml")?;
    let metadata_job = release
        .split("  release-metadata:")
        .nth(1)
        .and_then(|job| job.split("  native-generator-evidence:").next())
        .ok_or("Release must retain its independent metadata gate")?;
    if !metadata_job.contains("bash scripts/check-release-metadata.sh --release")
        || metadata_job.contains("validation_only")
    {
        return Err("Release, including validation_only, must use strict release metadata".to_owned());
    }
    let checker = read_repository_file("scripts/check-release-metadata.sh")?;
    if checker.contains("git describe") || checker.contains("latest_tag") {
        return Err("release metadata context must not be inferred from published tags".to_owned());
    }
    Ok(())
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
    if workspace.get("release_commits").and_then(toml::Value::as_str)
        != Some(r"^(feat|fix|perf|refactor|revert)(\([^)]+\))?!?:")
    {
        return Err("release-plz must prepare releases only for release-worthy code commits".to_owned());
    }

    validate_release_plz_changelog(&config)?;

    let workflow = read_repository_file(".github/workflows/release-plz.yml")?;
    let manifest: serde_json::Value =
        serde_json::from_str(&read_repository_file(".github/automation.json")?).map_err(|error| error.to_string())?;
    let revision = manifest["revision"]
        .as_str()
        .ok_or("missing shared automation revision")?;
    validate_shared_release_preparation(&workflow, repository, revision)?;
    for required in [
        "(.head.ref | startswith(\"release-plz-\"))",
        "actions/workflows/release.yml/dispatches",
        "actions: write",
        "for attempt in {1..8}; do",
        "Associated pull-request metadata is not available",
        "sleep 3",
        "No release was dispatched.",
    ] {
        if !workflow.contains(required) {
            return Err(format!("guarded release dispatcher missing `{required}`"));
        }
    }
    for forbidden in [
        "secrets.RELEASE_PLZ_APP_ID",
        "app-id:",
        "command: release\n",
        "cargo publish",
        "git tag",
        "gh release create",
        "renovate: datasource=crate depName=release-plz",
    ] {
        if workflow.contains(forbidden) {
            return Err(format!("release preparation must not retain `{forbidden}`"));
        }
    }
    let release = read_repository_file(".github/workflows/release.yml")?;
    if release.contains("docs/releases/${version}.md") || !release.contains("bash scripts/extract-release-notes.sh") {
        return Err("protected publication must derive release notes from CHANGELOG.md".to_owned());
    }
    Ok(())
}

/// Validate the preparation job against the full owner/repository identity.
fn validate_shared_release_preparation(workflow: &str, repository: &str, revision: &str) -> Result<(), String> {
    let (_, tail) = workflow
        .split_once("  prepare-release-pr:\n")
        .ok_or("missing release preparation job")?;
    let (prepare, _) = tail
        .split_once("  dispatch-release:\n")
        .ok_or("missing guarded release dispatcher")?;
    let reference = format!(
        "uses: Strukturpiloten/boxferry/.github/workflows/shared-release-prepare.yml@{revision} # automation-v1 interface v1"
    );
    if prepare.matches(reference.as_str()).count() != 1 {
        return Err("release preparation must select the manifest-bound shared workflow once".to_owned());
    }
    let repository_guard = format!("    if: github.repository == '{repository}'");
    if !prepare
        .lines()
        .filter(|line| line.starts_with("    if:"))
        .eq([repository_guard.as_str()])
    {
        return Err("release preparation must retain its exact job-level repository guard".to_owned());
    }
    for expected in [
        "permissions:\n      contents: read".to_owned(),
        "candidate_sha: ${{ github.sha }}".to_owned(),
        format!("automation_sha: {revision}"),
        "policy_sha: ${{ github.sha }}".to_owned(),
        "config_path: scripts/validation-policy.json".to_owned(),
        "app_client_id: ${{ vars.RELEASE_PLZ_APP_CLIENT_ID }}".to_owned(),
        "app_private_key: ${{ secrets.RELEASE_PLZ_APP_PRIVATE_KEY }}".to_owned(),
    ] {
        if !prepare.contains(expected.as_str()) {
            return Err(format!("shared release preparation omits `{expected}`"));
        }
    }
    for forbidden in [
        "runs-on:",
        "steps:",
        "release-plz/action@",
        "actions/create-github-app-token@",
        "version:",
        "secrets: inherit",
        "contents: write",
        "actions: write",
        "pull-requests: write",
        "id-token: write",
    ] {
        if prepare.contains(forbidden) {
            return Err(format!("thin release preparation must not contain `{forbidden}`"));
        }
    }
    Ok(())
}

#[test]
fn shared_release_preparation_rejects_source_scope_and_token_drift() -> Result<(), String> {
    let workflow = read_repository_file(".github/workflows/release-plz.yml")?;
    let manifest: serde_json::Value =
        serde_json::from_str(&read_repository_file(".github/automation.json")?).map_err(|error| error.to_string())?;
    let revision = manifest["revision"].as_str().ok_or("missing automation revision")?;
    validate_shared_release_preparation(&workflow, "Strukturpiloten/quadlet-lens", revision)?;
    for (before, after) in [
        (
            "if: github.repository == 'Strukturpiloten/quadlet-lens'".to_owned(),
            "if: github.repository == 'OtherOwner/quadlet-lens'".to_owned(),
        ),
        (
            "if: github.repository == 'Strukturpiloten/quadlet-lens'".to_owned(),
            String::new(),
        ),
        (
            "if: github.repository == 'Strukturpiloten/quadlet-lens'".to_owned(),
            "if: github.repository == 'Strukturpiloten/quadlet-lens' || true".to_owned(),
        ),
        (
            "if: github.repository == 'Strukturpiloten/quadlet-lens'".to_owned(),
            "if: github.repository == 'OtherOwner/quadlet-lens'\n    # if: github.repository == 'Strukturpiloten/quadlet-lens'".to_owned(),
        ),
        (
            "if: github.repository == 'Strukturpiloten/quadlet-lens'".to_owned(),
            "if: github.repository == 'OtherOwner/quadlet-lens'\n    env:\n      GUARD_TEXT: |\n        if: github.repository == 'Strukturpiloten/quadlet-lens'".to_owned(),
        ),
        (
            format!("shared-release-prepare.yml@{revision}"),
            format!("shared-release-prepare.yml@{}", "a".repeat(40)),
        ),
        (
            format!("automation_sha: {revision}"),
            format!("automation_sha: {}", "a".repeat(40)),
        ),
        (
            "candidate_sha: ${{ github.sha }}".to_owned(),
            "candidate_sha: main".to_owned(),
        ),
        (
            "policy_sha: ${{ github.sha }}".to_owned(),
            "policy_sha: main".to_owned(),
        ),
        (
            "app_client_id: ${{ vars.RELEASE_PLZ_APP_CLIENT_ID }}".to_owned(),
            "app_client_id: missing".to_owned(),
        ),
        (
            "app_private_key: ${{ secrets.RELEASE_PLZ_APP_PRIVATE_KEY }}".to_owned(),
            "app_private_key: missing".to_owned(),
        ),
        (
            "permissions:\n      contents: read".to_owned(),
            "permissions:\n      contents: write".to_owned(),
        ),
        (
            "    uses: Strukturpiloten/boxferry/".to_owned(),
            "    steps: []\n    uses: Strukturpiloten/boxferry/".to_owned(),
        ),
    ] {
        let changed = workflow.replacen(before.as_str(), after.as_str(), 1);
        assert_ne!(changed, workflow);
        assert!(
            validate_shared_release_preparation(&changed, "Strukturpiloten/quadlet-lens", revision).is_err(),
            "accepted drift: {before}"
        );
    }
    Ok(())
}

/// Check the action's immutable identity without copying its Renovate-owned revision.
///
/// Renovate updates both the SHA and annotated tag comment.  The repository policy owns
/// the security properties of that pair, not a second, stale copy of the revision.
fn validate_renovate_owned_action_pin(workflow: &str, expected_action: &str) -> Result<(), String> {
    let pins = workflow
        .lines()
        .filter_map(|line| line.trim().strip_prefix("uses:"))
        .map(str::trim)
        .filter_map(|reference| {
            let (action, pin) = reference.split_once('@')?;
            (action == expected_action).then_some(pin)
        })
        .collect::<Vec<_>>();
    let [pin] = pins.as_slice() else {
        return Err(format!("release-plz workflow must use {expected_action} exactly once"));
    };
    let (sha, comment) = pin
        .split_once('#')
        .ok_or_else(|| format!("{expected_action} pin must have an exact release-tag comment"))?;
    let sha = sha.trim();
    if sha.len() != 40
        || !sha
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(format!("{expected_action} must be pinned to a full immutable SHA"));
    }
    let tag = comment.trim();
    let version = tag
        .strip_prefix('v')
        .ok_or_else(|| format!("{expected_action} pin comment must be an exact vX.Y.Z tag"))?;
    if version.split('.').count() != 3
        || version
            .split('.')
            .any(|component| component.is_empty() || !component.bytes().all(|byte| byte.is_ascii_digit()))
    {
        return Err(format!("{expected_action} pin comment must be an exact vX.Y.Z tag"));
    }
    Ok(())
}

#[test]
fn release_plz_action_pin_accepts_renovate_owned_replacements_and_rejects_weakened_forms() {
    assert!(
        validate_renovate_owned_action_pin(
            "uses: release-plz/action@0123456789abcdef0123456789abcdef01234567 # v0.5.999\n",
            "release-plz/action",
        )
        .is_ok(),
        "a full SHA with an exact tag comment must remain valid after a Renovate update"
    );

    for invalid in [
        "uses: release-plz/action@0123456789abcdef0123456789abcdef01234567\n",
        "uses: release-plz/action@v0.5.999 # v0.5.999\n",
        "uses: release-plz/action@0123456789abcdef # v0.5.999\n",
        "uses: release-plz/action@0123456789abcdef0123456789abcdef0123456g # v0.5.999\n",
        "uses: release-plz/action@0123456789abcdef0123456789abcdef01234567 # latest\n",
        "uses: actions/checkout@0123456789abcdef0123456789abcdef01234567 # v4.2.2\n",
        "uses: release-plz/action@0123456789abcdef0123456789abcdef01234567\nuses: release-plz/action@fedcba9876543210fedcba9876543210fedcba98 # v0.5.999\n",
    ] {
        assert!(
            validate_renovate_owned_action_pin(invalid, "release-plz/action").is_err(),
            "weakened release-plz action contract unexpectedly accepted: {invalid}"
        );
    }
}

fn validate_release_plz_changelog(config: &toml::Value) -> Result<(), String> {
    let changelog = config["changelog"]
        .as_table()
        .ok_or_else(|| "release-plz.toml must contain [changelog]".to_owned())?;
    if changelog.get("protect_breaking_commits").and_then(toml::Value::as_bool) != Some(true) {
        return Err("release-plz must preserve breaking commits in generated changelogs".to_owned());
    }

    let parsers = changelog
        .get("commit_parsers")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| "release-plz must configure changelog commit parsers".to_owned())?;
    let expected = [
        ("^feat", Some("Added"), false),
        ("^fix", Some("Fixed"), false),
        ("^perf", Some("Performance"), false),
        ("^refactor", Some("Changed"), false),
        ("^revert", Some("Reverted"), false),
        ("^.*", None, true),
    ];
    if parsers.len() != expected.len() {
        return Err("release-plz must configure the exact code-only changelog parser set".to_owned());
    }
    for (parser, (message, group, skip)) in parsers.iter().zip(expected) {
        if parser["message"].as_str() != Some(message)
            || parser.get("group").and_then(toml::Value::as_str) != group
            || parser.get("skip").and_then(toml::Value::as_bool).unwrap_or(false) != skip
        {
            return Err(format!("release-plz changelog parser for {message} is invalid"));
        }
    }

    let releasing = read_repository_file("docs/releasing.md")?;
    for required in [
        "classification contract",
        "`feat`, `fix`, `perf`, `refactor`, or `revert`",
        "`docs`, `test`, `ci`, `build`, `style`, or `chore`",
    ] {
        if !releasing.contains(required) {
            return Err(format!("release documentation is missing `{required}`"));
        }
    }

    Ok(())
}

#[test]
fn shared_file_tool_bootstrap_preserves_pins_arguments_and_child_failures() -> Result<(), String> {
    let output = Command::new("python3").arg("-c").arg(r#"
import contextlib
import importlib.util
import io
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

source = Path(sys.argv[1])
config = json.loads((source / '.github/automation.json').read_text())
caller = (source / '.github/workflows/dependency-eligibility.yml').read_text()
revision = config['revision']
assert len(revision) == 40 and all(c in '0123456789abcdef' for c in revision)
assert caller.count('shared-dependency-eligibility.yml@' + revision + ' # automation-v1 interface v1') == 1
assert caller.count('automation-sha: ' + revision) == 1
with tempfile.TemporaryDirectory(prefix='quadlet-tool-policy-') as temporary:
    root = Path(temporary)
    (root / '.github').mkdir()
    (root / 'scripts').mkdir()
    (root / 'bin').mkdir()
    manifest = root / '.github/automation.json'
    manifest.write_text(json.dumps(config))
    for name in ['shared-ci.py', 'install-file-tools.sh']:
        shutil.copyfile(source / 'scripts' / name, root / 'scripts' / name)
    spec = importlib.util.spec_from_file_location('quadlet_tool_bootstrap_test', root / 'scripts/shared-ci.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    assert module.load_config(root) == config
    replacement = dict(config, revision='0123456789abcdef0123456789abcdef01234567')
    manifest.write_text(json.dumps(replacement))
    assert module.load_config(root) == replacement, 'valid immutable updates must remain accepted'
    for invalid in ['main', 'automation-v1', 'a' * 39, 'a' * 41, 'A' * 40, 'g' * 40, 'a' * 40 + '\n']:
        manifest.write_text(json.dumps(dict(config, revision=invalid)))
        try:
            module.load_config(root)
        except module.BootstrapError:
            pass
        else:
            raise AssertionError('accepted invalid revision: ' + repr(invalid))
    manifest.write_text(json.dumps(config))
    project = root / 'implementation/automation'
    bootstrap = project / 'src/boxferry_ci/bootstrap.py'
    bootstrap.parent.mkdir(parents=True)
    bootstrap.write_text('def run_project(root, project, args):\n'
                         '    assert root.name.startswith("quadlet-tool-policy-")\n'
                         '    assert project.name == "automation"\n'
                         '    assert args == ["tools", "--destination", "target dir", "--profile", "core"]\n'
                         '    return 37\n')
    def prepare(repository_root, pinned_revision):
        assert repository_root == root and pinned_revision == revision
        return project
    module.prepare_implementation = prepare
    arguments = ['tools', '--destination', 'target dir', '--profile', 'core']
    assert module.main(arguments) == 37, 'loader masked delegated failure'
    def unavailable(*args):
        raise module.BootstrapError('unavailable immutable implementation')
    module.prepare_implementation = unavailable
    error = io.StringIO()
    with contextlib.redirect_stderr(error):
        assert module.main(arguments) == 2
    assert 'unavailable immutable implementation' in error.getvalue()
    recorder = root / 'arguments'
    python = root / 'bin/python3'
    python.write_text('#!/bin/bash\nprintf "%s\\n" "$@" > "$ARGUMENT_RECORD"\nexit 37\n')
    python.chmod(0o755)
    environment = dict(os.environ, PATH=str(root / 'bin') + os.pathsep + os.defpath, ARGUMENT_RECORD=str(recorder))
    wrapper = root / 'scripts/install-file-tools.sh'
    for extra, destination in [([], '/usr/local/bin'), (['target dir'], 'target dir')]:
        result = subprocess.run(['bash', str(wrapper), *extra], env=environment, capture_output=True)
        assert result.returncode == 37, 'installer masked child failure'
        assert recorder.read_text().splitlines() == [str(root / 'scripts/shared-ci.py'), 'tools', '--destination', destination, '--profile', 'core']
    recorder.unlink()
    result = subprocess.run(['bash', str(wrapper), 'one', 'two'], env=environment, capture_output=True)
    assert result.returncode == 2 and b'Usage:' in result.stderr
    assert not recorder.exists(), 'invalid arguments reached tool acquisition'
"#).arg(repository_root()).output().map_err(|error| format!("failed to run shared file-tool contract: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "shared file-tool contract failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

#[test]
fn renovate_tracks_every_directly_pinned_development_tool() -> Result<(), String> {
    let configuration: serde_json::Value =
        serde_json::from_str(&read_repository_file(".github/renovate.json")?).map_err(|error| error.to_string())?;
    let manifest: serde_json::Value =
        serde_json::from_str(&read_repository_file(".github/automation.json")?).map_err(|error| error.to_string())?;
    let revision = manifest["revision"].as_str().ok_or("missing automation revision")?;
    let expected = serde_json::json!([
        format!("github>Strukturpiloten/boxferry//renovate-presets/common#{revision}"),
        format!("github>Strukturpiloten/boxferry//renovate-presets/rust#{revision}")
    ]);
    if configuration["extends"] != expected {
        return Err("common/Rust presets and executable source must form one immutable adoption".to_owned());
    }
    validate_podman_patch_discovery(&configuration, &read_repository_file("tools/generator-matrix.toml")?)?;
    let managers = configuration["customManagers"]
        .as_array()
        .ok_or("missing local managers")?;
    for required in [
        "Update versioned Dev Container tools",
        "Update directly pinned workflow tool versions",
        "Update the Dev Container base release and digest together",
        "Update the GitHub CLI installed in the Dev Container",
    ] {
        if managers
            .iter()
            .filter(|manager| manager["description"] == required)
            .count()
            != 1
        {
            return Err(format!("local pin must retain one owner `{required}`"));
        }
    }
    if managers
        .iter()
        .any(|manager| manager["depNameTemplate"] == "Strukturpiloten/boxferry")
    {
        return Err("shared preset must replace temporary duplicate bootstrap managers".to_owned());
    }
    let rules = configuration["packageRules"].as_array().ok_or("missing local rules")?;
    if rules.iter().any(|rule| {
        rule["dependencyDashboardApproval"] == true
            || rule["automerge"] == false
            || rule.get("minimumReleaseAge").is_some()
    }) {
        return Err("local overrides must not weaken shared automatic policy or third-party age".to_owned());
    }
    Ok(())
}

const UPSTREAM_DISCOVERY: &str = "Discover the newest stable Podman release across minor lines";
const MINOR_DISCOVERY: &str = "Discover the newest stable Podman patch independently for each maintained minor line";

fn discovery_release(value: &str) -> Result<[u32; 3], String> {
    let parts = value
        .split('.')
        .map(str::parse::<u32>)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| format!("discovery must use a stable numeric release: {value}"))?;
    let release: [u32; 3] = parts
        .try_into()
        .map_err(|_| format!("discovery must use three release components: {value}"))?;
    if value != format!("{}.{}.{}", release[0], release[1], release[2]) {
        return Err("discovery must use canonical decimal release components".to_owned());
    }
    Ok(release)
}

fn discovery_owner<'a>(
    configuration: &'a serde_json::Value,
    description: &str,
    pattern: &str,
) -> Result<&'a serde_json::Value, String> {
    let owners = configuration["customManagers"]
        .as_array()
        .ok_or("missing discovery managers")?
        .iter()
        .filter(|manager| manager["description"] == description)
        .collect::<Vec<_>>();
    let [owner] = owners.as_slice() else {
        return Err(format!("discovery must retain one owner for {description}"));
    };
    if owner["customType"] != "regex"
        || owner["enabled"] == false
        || owner["managerFilePatterns"] != serde_json::json!(["/^tools/generator-matrix\\.toml$/"])
        || owner["matchStrings"] != serde_json::json!([pattern])
        || owner["extractVersionTemplate"] != "^v?(?<version>.*)$"
    {
        return Err(format!("discovery owner path/extraction changed: {description}"));
    }
    Ok(owner)
}

fn validate_discovery_declaration(
    rules: &[serde_json::Value],
    repository: &str,
    key: &str,
    value: &toml::Value,
    lines: &[&str],
) -> Result<String, String> {
    let value = value.as_str().ok_or("discovery release must be a string")?;
    let release = discovery_release(value)?;
    let (dependency, allowed) = if key == "latest_upstream" {
        ("podman-upstream".to_owned(), r"/^[0-9]+\.[0-9]+\.[0-9]+$/".to_owned())
    } else {
        let suffix = key.strip_prefix("latest_").ok_or("missing discovery prefix")?;
        if suffix != format!("{}_{}", release[0], release[1]) {
            return Err("maintained discovery version must match its minor key".to_owned());
        }
        (
            format!("podman-{}.{}", release[0], release[1]),
            format!(r"/^{}\.{}\.[0-9]+$/", release[0], release[1]),
        )
    };
    let declaration = format!("{key} = \"{value}\"");
    let positions = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| (*line == declaration.as_str()).then_some(index))
        .collect::<Vec<_>>();
    let [position] = positions.as_slice() else {
        return Err("discovery must retain one canonical declaration".to_owned());
    };
    let marker = format!("# renovate: datasource=github-releases packageName={repository} depName={dependency}");
    if *position == 0 || lines[*position - 1] != marker.as_str() {
        return Err("discovery declaration must retain its exact owner marker".to_owned());
    }
    let filters = rules
        .iter()
        .filter(|rule| {
            rule["matchManagers"]
                .as_array()
                .is_some_and(|values| values.iter().any(|value| value.as_str() == Some("custom.regex")))
                && rule["matchDepNames"]
                    .as_array()
                    .is_some_and(|values| values.iter().any(|value| value.as_str() == Some(dependency.as_str())))
        })
        .collect::<Vec<_>>();
    let [filter] = filters.as_slice() else {
        return Err(format!("discovery must retain one candidate filter for {dependency}"));
    };
    if filter["matchManagers"] != serde_json::json!(["custom.regex"])
        || filter["matchDepNames"] != serde_json::json!([dependency])
        || filter["allowedVersions"].as_str() != Some(allowed.as_str())
        || filter["enabled"] == false
    {
        return Err("discovery candidate filter must preserve its stable minor boundary".to_owned());
    }
    Ok(dependency)
}

fn validate_podman_patch_discovery(configuration: &serde_json::Value, matrix: &str) -> Result<(), String> {
    // Rust/MSRV policy checks remain structural and Node-free. The mandatory MJS
    // check in check-files.sh independently exercises actual regex semantics.
    discovery_owner(
        configuration,
        UPSTREAM_DISCOVERY,
        r#"# renovate: datasource=(?<datasource>[^\s]+) packageName=(?<packageName>[^\s]+) depName=(?<depName>podman-upstream)\nlatest_upstream = \"(?<currentValue>[^\"]+)\""#,
    )?;
    discovery_owner(
        configuration,
        MINOR_DISCOVERY,
        r#"# renovate: datasource=(?<datasource>[^\s]+) packageName=(?<packageName>[^\s]+) depName=(?<depName>podman-[0-9]+\.[0-9]+)\nlatest_[0-9]+_[0-9]+ = \"(?<currentValue>[^\"]+)\""#,
    )?;
    let managers = configuration["customManagers"].as_array().ok_or("missing managers")?;
    let matrix_owners = managers
        .iter()
        .filter(|manager| {
            manager["customType"] == "regex"
                && manager["enabled"] != false
                && manager["managerFilePatterns"] == serde_json::json!(["/^tools/generator-matrix\\.toml$/"])
                && manager["matchStrings"].as_array().is_some_and(|patterns| {
                    patterns
                        .iter()
                        .any(|pattern| pattern.as_str().is_some_and(|value| value.contains("latest_")))
                })
        })
        .count();
    if matrix_owners != 2 {
        return Err("discovery must retain exactly two canonical matrix managers".to_owned());
    }
    let parsed = toml::from_str::<toml::Value>(matrix).map_err(|error| error.to_string())?;
    let table = parsed.as_table().ok_or("generator matrix must be a table")?;
    let repository = table
        .get("source_repository")
        .and_then(toml::Value::as_str)
        .and_then(|value| value.strip_prefix("https://github.com/"))
        .and_then(|value| value.strip_suffix(".git"))
        .ok_or("matrix must declare its public upstream repository")?;
    let rules = configuration["packageRules"]
        .as_array()
        .ok_or("missing discovery rules")?;
    let mut dependencies = BTreeSet::new();
    let lines = matrix.lines().collect::<Vec<_>>();
    for (key, value) in table.iter().filter(|(key, _)| key.starts_with("latest_")) {
        let dependency = validate_discovery_declaration(rules, repository, key, value, &lines)?;
        if !dependencies.insert(dependency) {
            return Err("duplicate discovery dependency".to_owned());
        }
    }
    if !dependencies.contains("podman-upstream") || dependencies.len() < 2 {
        return Err("newest and maintained discovery must both remain present".to_owned());
    }
    let groups = rules
        .iter()
        .filter(|rule| {
            rule["matchManagers"]
                .as_array()
                .is_some_and(|values| values.iter().any(|value| value.as_str() == Some("custom.regex")))
                && rule["matchPackageNames"]
                    .as_array()
                    .is_some_and(|values| values.iter().any(|value| value.as_str() == Some(repository)))
        })
        .collect::<Vec<_>>();
    let [group] = groups.as_slice() else {
        return Err("discovery must have one separately grouped package rule".to_owned());
    };
    if group["groupName"] != "Podman generator discovery"
        || group["matchFileNames"] != serde_json::json!(["tools/generator-matrix.toml"])
        || [
            "automerge",
            "dependencyDashboardApproval",
            "minimumReleaseAge",
            "enabled",
        ]
        .iter()
        .any(|field| group.get(*field).is_some())
    {
        return Err("discovery grouping must retain the shared automatic update policy".to_owned());
    }
    Ok(())
}

#[test]
fn podman_discovery_guard_accepts_future_values_and_rejects_owner_filter_drift() -> Result<(), String> {
    let configuration: serde_json::Value =
        serde_json::from_str(&read_repository_file(".github/renovate.json")?).map_err(|error| error.to_string())?;
    let matrix = read_repository_file("tools/generator-matrix.toml")?;
    validate_podman_patch_discovery(&configuration, &matrix)?;
    let parsed = toml::from_str::<toml::Value>(&matrix).map_err(|error| error.to_string())?;
    for (key, value) in parsed
        .as_table()
        .ok_or("missing matrix table")?
        .iter()
        .filter(|(key, _)| key.starts_with("latest_"))
    {
        let value = value.as_str().ok_or("missing discovery release")?;
        let mut release = discovery_release(value)?;
        if key == "latest_upstream" {
            release[1] = release[1].checked_add(1).ok_or("minor overflow")?;
            release[2] = 0;
        } else {
            release[2] = release[2].checked_add(1).ok_or("patch overflow")?;
        }
        let before = format!("{key} = \"{value}\"");
        let after = format!("{key} = \"{}.{}.{}\"", release[0], release[1], release[2]);
        let advanced = matrix.replacen(&before, &after, 1);
        assert_ne!(advanced, matrix);
        validate_podman_patch_discovery(&configuration, &advanced)?;
        assert_eq!(
            advanced.replacen(&after, &before, 1),
            matrix,
            "fixed generator evidence changed"
        );
    }
    let owners = configuration["customManagers"].as_array().ok_or("missing managers")?;
    let upstream = owners
        .iter()
        .position(|manager| manager["description"] == UPSTREAM_DISCOVERY)
        .ok_or("missing upstream owner")?;
    let mut duplicate = configuration.clone();
    duplicate["customManagers"]
        .as_array_mut()
        .ok_or("missing managers")?
        .push(owners[upstream].clone());
    assert!(validate_podman_patch_discovery(&duplicate, &matrix).is_err());
    let mut missing = configuration.clone();
    missing["customManagers"]
        .as_array_mut()
        .ok_or("missing managers")?
        .remove(upstream);
    assert!(validate_podman_patch_discovery(&missing, &matrix).is_err());
    let mut wrong_path = configuration.clone();
    wrong_path["customManagers"][upstream]["managerFilePatterns"] = serde_json::json!(["/^unrelated\\.toml$/"]);
    assert!(validate_podman_patch_discovery(&wrong_path, &matrix).is_err());
    let mut broad = configuration.clone();
    let filter = broad["packageRules"]
        .as_array_mut()
        .ok_or("missing rules")?
        .iter_mut()
        .find(|rule| {
            rule["matchDepNames"]
                .as_array()
                .is_some_and(|names| names.iter().any(|name| name == "podman-upstream"))
        })
        .ok_or("missing upstream filter")?;
    filter["allowedVersions"] = serde_json::json!("/.*/");
    assert!(validate_podman_patch_discovery(&broad, &matrix).is_err());
    Ok(())
}

#[test]
fn podman_maintained_filter_rejects_broad_missing_and_duplicate_rules() -> Result<(), String> {
    let configuration: serde_json::Value =
        serde_json::from_str(&read_repository_file(".github/renovate.json")?).map_err(|error| error.to_string())?;
    let matrix = read_repository_file("tools/generator-matrix.toml")?;
    let parsed = toml::from_str::<toml::Value>(&matrix).map_err(|error| error.to_string())?;
    let key = parsed
        .as_table()
        .ok_or("missing matrix table")?
        .keys()
        .find(|key| key.starts_with("latest_") && *key != "latest_upstream")
        .ok_or("missing maintained minor")?;
    let dependency = format!("podman-{}", key.trim_start_matches("latest_").replace('_', "."));
    let rules = configuration["packageRules"].as_array().ok_or("missing rules")?;
    let index = rules
        .iter()
        .position(|rule| {
            rule["matchDepNames"]
                .as_array()
                .is_some_and(|values| values.iter().any(|value| value.as_str() == Some(dependency.as_str())))
        })
        .ok_or("missing maintained filter")?;
    let mut broad = configuration.clone();
    broad["packageRules"][index]["allowedVersions"] = serde_json::json!(r"/^[0-9]+\.[0-9]+\.[0-9]+$/");
    assert!(validate_podman_patch_discovery(&broad, &matrix).is_err());
    let mut missing = configuration.clone();
    missing["packageRules"]
        .as_array_mut()
        .ok_or("missing rules")?
        .remove(index);
    assert!(validate_podman_patch_discovery(&missing, &matrix).is_err());
    let mut duplicate = configuration.clone();
    duplicate["packageRules"]
        .as_array_mut()
        .ok_or("missing rules")?
        .push(rules[index].clone());
    assert!(validate_podman_patch_discovery(&duplicate, &matrix).is_err());
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

#[test]
fn maintainer_documentation_is_task_oriented_and_bounded() -> Result<(), String> {
    const GUIDES: &[&str] = &[
        "README.md",
        "api-stability.md",
        "architecture.md",
        "capability-model.md",
        "dependency-policy.md",
        "development-environment.md",
        "environment-and-secrets.md",
        "fixture-format.md",
        "generation.md",
        "generator-maintenance.md",
        "generator-matrix.md",
        "real-world-quadlet-corpus.md",
        "releasing.md",
        "roadmap.md",
        "testing.md",
        "typed-model.md",
    ];

    let root = repository_root();
    let docs_root = root.join("docs");
    let actual = fs::read_dir(&docs_root)
        .map_err(|error| format!("failed to read {}: {error}", docs_root.display()))?
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            (path.is_file() && path.extension().is_some_and(|extension| extension == "md")).then_some(path)
        })
        .collect::<BTreeSet<_>>();
    let expected = GUIDES.iter().map(|path| docs_root.join(path)).collect::<BTreeSet<_>>();
    if actual != expected {
        return Err(format!(
            "docs must contain exactly the task-oriented guide inventory; expected {expected:#?}, found {actual:#?}"
        ));
    }

    for path in GUIDES {
        let repository_path = format!("docs/{path}");
        let document = read_repository_file(&repository_path)?;
        let mut inside_fence = false;
        let mut level_one_headings = 0;
        for line in document.lines() {
            if line.starts_with("```") {
                inside_fence = !inside_fence;
            } else if !inside_fence && line.starts_with("# ") {
                level_one_headings += 1;
            }
        }
        if level_one_headings != 1 {
            return Err(format!("{repository_path} must contain exactly one level-one heading"));
        }
        if document.lines().count() > 150 {
            return Err(format!("{repository_path} exceeds the 150-line maintainer-guide limit"));
        }
        for paragraph in document.split("\n\n") {
            let block = paragraph.trim_start();
            let is_table_or_code = block.starts_with('|') || block.starts_with("```");
            if !is_table_or_code && paragraph.split_whitespace().count() > 120 {
                return Err(format!(
                    "{repository_path} contains a prose paragraph longer than 120 words"
                ));
            }
        }
        let lowercase = document.to_ascii_lowercase();
        for placeholder in ["todo", "coming soon", "lorem ipsum"] {
            if lowercase.contains(placeholder) {
                return Err(format!("{repository_path} contains placeholder text `{placeholder}`"));
            }
        }
    }

    Ok(())
}

#[test]
fn documentation_uses_machine_sources_instead_of_stale_ledgers() -> Result<(), String> {
    let root = repository_root();
    for removed in [
        "docs/coverage.md",
        "docs/implementation-plan.md",
        "docs/project-structure.md",
        "docs/quality-plan.md",
        "docs/research/podlet-regressions-2026-08-01.md",
    ] {
        if root.join(removed).exists() {
            return Err(format!("obsolete documentation ledger still exists: {removed}"));
        }
    }

    let index = read_repository_file("docs/README.md")?;
    for source in [
        "fixtures/specification-drift/quadlet-manual-current.toml",
        "catalogue/v1/podman-supported-range.toml",
        "tools/generator-matrix.toml",
        "fixtures/real-world/corpus.toml",
        "CHANGELOG.md",
    ] {
        if !index.contains(source) {
            return Err(format!("documentation index does not route readers to `{source}`"));
        }
    }

    let roadmap = read_repository_file("docs/roadmap.md")?;
    for stale in ["## Phase 0", "## Additive 0.", "Maintainer-controlled 0.1.0"] {
        if roadmap.contains(stale) {
            return Err(format!("roadmap contains completed release ledger marker `{stale}`"));
        }
    }

    for path in [
        "docs/README.md",
        "docs/architecture.md",
        "docs/capability-model.md",
        "docs/generation.md",
        "docs/generator-matrix.md",
        "docs/roadmap.md",
        "docs/testing.md",
        "docs/typed-model.md",
    ] {
        let document = read_repository_file(path)?;
        for stale in ["222-key", "222-row", "20-patch", "all-20", "current 6.0.2"] {
            if document.contains(stale) {
                return Err(format!("{path} contains stale prose-ledger marker `{stale}`"));
            }
        }
    }

    let agents = read_repository_file("AGENTS.md")?;
    for required in ["docs/README.md", "Read only the guide", "only the ADRs relevant"] {
        if !agents.contains(required) {
            return Err(format!("AGENTS.md is missing task-based reading rule `{required}`"));
        }
    }

    Ok(())
}

#[test]
fn public_documentation_is_bounded_and_website_owned() -> Result<(), String> {
    const PAGES: &[(&str, &[&str])] = &[
        ("docs/public/index.md", &["directly", "Rust API"]),
        ("docs/public/model/index.md", &["QuadletDocumentSet", "side effects"]),
        (
            "docs/public/parsing-rendering/index.md",
            &["render_preserved", "QuadletDocumentBuilder", "Podman"],
        ),
        (
            "docs/public/diagnostics/index.md",
            &["machine-readable code", "source", "recovery"],
        ),
        (
            "docs/public/compatibility/index.md",
            &["PodmanTarget", "downloadable TOML catalogue", "Unknown"],
        ),
    ];

    let root = repository_root();
    let public_root = root.join("docs/public");
    let actual = walk_markdown_files(&public_root)?;
    let expected = PAGES.iter().map(|(path, _)| root.join(path)).collect::<BTreeSet<_>>();
    if actual != expected {
        return Err(format!(
            "docs/public must contain exactly the website-owned page inventory; expected {expected:#?}, found {actual:#?}"
        ));
    }

    for (path, topics) in PAGES {
        let document = read_repository_file(path)?;
        if document.lines().filter(|line| line.starts_with("# ")).count() != 1 {
            return Err(format!("{path} must contain exactly one level-one heading"));
        }
        if document.lines().count() > 90 {
            return Err(format!("{path} exceeds the 90-line public-page limit"));
        }
        for paragraph in document.split("\n\n") {
            if paragraph.lines().count() > 14 {
                return Err(format!("{path} contains a paragraph longer than 14 lines"));
            }
        }
        let lowercase = document.to_ascii_lowercase();
        for placeholder in ["todo", "coming soon", "lorem ipsum"] {
            if lowercase.contains(placeholder) {
                return Err(format!("{path} contains placeholder text `{placeholder}`"));
            }
        }
        for topic in *topics {
            if !document.contains(topic) {
                return Err(format!("{path} is missing required public topic `{topic}`"));
            }
        }
    }

    let catalogue = read_repository_file("catalogue/v1/podman-supported-range.toml")?;
    for required in ["schema = 1", "minimum = \"5.4.0\"", "maximum = \"6.1.2\""] {
        if !catalogue.contains(required) {
            return Err(format!("public capability catalogue is missing `{required}`"));
        }
    }

    Ok(())
}

fn walk_markdown_files(root: &Path) -> Result<BTreeSet<PathBuf>, String> {
    let mut files = BTreeSet::new();
    let mut directories = vec![root.to_path_buf()];
    while let Some(directory) = directories.pop() {
        for entry in
            fs::read_dir(&directory).map_err(|error| format!("failed to read {}: {error}", directory.display()))?
        {
            let path = entry.map_err(|error| error.to_string())?.path();
            if path.is_dir() {
                directories.push(path);
            } else if path.extension().is_some_and(|extension| extension == "md") {
                files.insert(path);
            }
        }
    }
    Ok(files)
}

#[test]
fn agent_roles_are_explicit() -> Result<(), Box<dyn std::error::Error>> {
    let root = repository_root();
    let config = fs::read_to_string(root.join(".codex/config.toml"))?;
    for required in [
        "model = \"gpt-6-sol\"",
        "model_reasoning_effort = \"xhigh\"",
        "max_concurrent_threads_per_session = 8",
        "default_subagent_model = \"gpt-6.1-sol\"",
        "default_subagent_reasoning_effort = \"medium\"",
        "This limit excludes the primary manager",
    ] {
        assert!(config.contains(required), "missing agent default: {required}");
    }
    for (role, model, effort, sandbox) in [
        ("implementation-worker", "gpt-6.1-sol", "high", "workspace-write"),
        ("specification-researcher", "gpt-6.1-sol", "high", "read-only"),
        ("reviewer", "gpt-6.1-sol", "high", "read-only"),
        ("presentation-worker", "gpt-6.1-terra", "medium", "workspace-write"),
        ("verifier", "gpt-6-luna", "high", "workspace-write"),
    ] {
        let text = fs::read_to_string(root.join(format!(".codex/agents/{role}.toml")))?;
        for (key, value) in [
            ("model", model),
            ("model_reasoning_effort", effort),
            ("sandbox_mode", sandbox),
        ] {
            assert!(
                text.contains(&format!("{key} = \"{value}\"")),
                "{role}: incorrect {key}"
            );
        }
    }
    let presentation = fs::read_to_string(root.join(".codex/agents/presentation-worker.toml"))?;
    for required in [
        "explicit file ownership",
        "Escalate architecture, native-format semantics, security, migrations",
        "Browser and application inspection is read-only",
        "Git or GitHub writes",
        "A Sol reviewer independently checks the original requirements",
    ] {
        assert!(
            presentation.contains(required),
            "missing presentation boundary: {required}"
        );
    }
    let reviewer = fs::read_to_string(root.join(".codex/agents/reviewer.toml"))?;
    assert!(reviewer.contains("original user requirements"));
    assert!(reviewer.contains("independent expected results"));
    let verifier = fs::read_to_string(root.join(".codex/agents/verifier.toml"))?;
    assert!(verifier.contains("./scripts/check-all.sh --check"));
    assert!(verifier.contains("never run the default formatting gate"));
    assert!(verifier.contains("Escalate complex failure diagnosis to the primary agent for a gpt-6.1-sol agent"));
    assert!(verifier.contains("remain check-only and do not edit tracked files"));
    let instructions = fs::read_to_string(root.join("AGENTS.md"))?
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    for required in [
        "The primary manager always uses",
        "`gpt-6-sol` with `xhigh` reasoning",
        "Unnamed subagents default to `gpt-6.1-sol` with `medium`",
        "Implementation, specification research, and independent review use `gpt-6.1-sol` with `high` reasoning",
        "check-only verification uses `gpt-6-luna` with `high`",
        "Task-specific subagents may use `gpt-6.1-sol`, `gpt-6.1-terra`, or `gpt-6-luna` with `medium`, `high`, or `xhigh` reasoning",
        "Use `gpt-6-astra` only with `xhigh` reasoning for particularly difficult architectural questions",
        "up to eight concurrent subagents plus the primary manager (nine agents in total)",
        "Eight is a ceiling, not a target or eight distinct roles",
        "Reuse existing subagents for related follow-up work",
        "Do not create nested agents to evade the limit",
        "`agents.max_concurrent_threads_per_session = 8` excludes the primary manager",
        "Never run two writers in one checkout",
        "Run at most one complete gate or heavy runtime suite at a time",
    ] {
        assert!(instructions.contains(required), "missing agent policy: {required}");
    }
    assert!(!instructions.contains("up to nine concurrent subagents"));
    assert!(!instructions.contains("high-effort primary"));
    Ok(())
}

// The full shell gate targets the Linux Dev Container; macOS has no CI validation lane.
// Keep configuration assertions above platform-independent.
#[cfg(target_os = "linux")]
#[test]
fn linux_gate_modes_and_failure_propagation_are_correct() -> Result<(), Box<dyn std::error::Error>> {
    let root = repository_root();
    let result = Command::new("bash")
        .arg("scripts/test-check-all.sh")
        .current_dir(root)
        .output()?;
    assert!(
        result.status.success(),
        "gate mode regression failed:\n{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(())
}
