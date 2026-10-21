//! Fail-closed Release-native generator workflow contract.

use std::{fs, path::PathBuf, process::Command};

#[test]
fn hosted_podman_prerequisite_and_launcher_controls() -> Result<(), String> {
    let output = Command::new("python3")
        .args(["-B", "scripts/test-hosted-generator-podman.py"])
        .current_dir(repository_root())
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    Ok(())
}

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn release_reuses_complete_native_generator_validation_fail_closed() -> Result<(), String> {
    let root = repository_root();
    let release = fs::read_to_string(root.join(".github/workflows/release.yml")).map_err(|error| error.to_string())?;
    let generators =
        fs::read_to_string(root.join(".github/workflows/generator-matrix.yml")).map_err(|error| error.to_string())?;

    assert_release_validation_contract(&release)?;
    assert_generator_validation_contract(&generators)?;
    for guidance_path in ["AGENTS.md", "docs/testing.md"] {
        let guidance = fs::read_to_string(root.join(guidance_path)).map_err(|error| error.to_string())?;
        for required in [
            "Every Release automatically",
            "fail-closed",
            "full pinned generator matrix",
            "tracked-current",
            "validation_only",
            "dry-run",
        ] {
            if !guidance.contains(required) {
                return Err(format!("{guidance_path} is missing Release guidance `{required}`"));
            }
        }
    }
    Ok(())
}

fn assert_release_validation_contract(release: &str) -> Result<(), String> {
    for required in [
        "uses: ./.github/workflows/ci.yml",
        "uses: ./.github/workflows/generator-matrix.yml",
        "candidate_sha: ${{ github.sha }}",
        "task: release-native-generator",
        "name: Exact-SHA native generator evidence",
        "name: Fail-closed release validation gate",
        "!inputs.validation_only",
        "artifact-metadata: write",
        "attestations: write",
        "contents: write",
        "id-token: write",
    ] {
        if !release.contains(required) {
            return Err(format!(
                "release workflow missing native validation contract `{required}`"
            ));
        }
    }
    for forbidden in ["contents: write\n    uses:", "id-token: write\n    uses:"] {
        if release.contains(forbidden) {
            return Err("reusable validation jobs must not receive release credentials".to_owned());
        }
    }
    let (validation_jobs, publish_job) = release
        .split_once("\n  release:\n")
        .ok_or_else(|| "release workflow must have one final publish job".to_owned())?;
    if validation_jobs.contains(": write") || !publish_job.contains("id-token: write") {
        return Err("only the final publish job may have release write permissions".to_owned());
    }
    for forbidden in [
        "cargo fmt",
        "cargo ci-check",
        "cargo ci-catalogue",
        "cargo ci-model",
        "cargo ci-application",
        "cargo ci-policy",
        "cargo ci-clippy",
        "cargo ci-test",
        "cargo ci-generators",
        "cargo ci-application-generators",
        "cargo ci-doctest",
        "cargo ci-doc",
        "cargo llvm-cov",
        "cargo deny",
        "cargo-semver-checks",
    ] {
        if publish_job.contains(forbidden) {
            return Err(format!(
                "credentialed publish job must not repeat validation `{forbidden}`"
            ));
        }
    }
    Ok(())
}

fn assert_generator_validation_contract(generators: &str) -> Result<(), String> {
    for required in [
        "workflow_call:",
        "ref: ${{ inputs.candidate_sha || github.sha }}",
        "run: cargo ci-generators",
        "run: cargo ci-application-generators",
        "QUADLET_LENS_CONTAINER_ENGINE: ${{ github.workspace }}/scripts/hosted-generator-podman.py",
        "QUADLET_LENS_PODMAN_PRIVILEGED_GENERATORS: \"true\"",
        "timeout-minutes: 33",
        "QUADLET_LENS_ENGINE_RUN_TIMEOUT_SECONDS: \"240\"",
        "run: python3 scripts/hosted-generator-podman.py prepare target/release-evidence/outer-podman.json",
        "--slurpfile outer_runtime target/release-evidence/outer-podman.json",
        "quadlet-generator-evidence-${GITHUB_RUN_ID}-${EVIDENCE_TASK}",
        "overwrite: true",
        "retention-days: 90",
    ] {
        if !generators.contains(required) {
            return Err(format!(
                "generator workflow missing reusable evidence contract `{required}`"
            ));
        }
    }
    for forbidden in [
        "QUADLET_LENS_CONTAINER_ENGINE: docker",
        "QUADLET_LENS_DOCKER_PRIVILEGED_GENERATORS",
    ] {
        if generators.contains(forbidden) {
            return Err(format!("generator workflow retains Docker dependency `{forbidden}`"));
        }
    }
    Ok(())
}
