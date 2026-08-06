//! Contract and opt-in execution tests for exact Podman Quadlet generators.

use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::str::FromStr;

use quadlet_lens::capability::PodmanVersion;
use serde::Deserialize;

const MATRIX: &str = include_str!("../tools/generator-matrix.toml");
const EXPECTED_IMAGE_VERSIONS: &[&str] = &[
    "5.4.0", "5.4.1", "5.4.2", "5.5.0", "5.5.1", "5.5.2", "5.6.0", "5.6.1", "5.6.2", "5.7.0", "5.7.1", "5.8.0",
    "5.8.1", "5.8.2",
];
const EXPECTED_SOURCE_VERSIONS: &[&str] = &["5.8.3", "5.8.4", "5.8.5", "6.0.0", "6.0.1", "6.0.2"];
const QUOTED_LABEL_LITERAL_SPACE: &str =
    r#"--label "io.github.strukturpiloten.quadlet-lens.metadata={\"channel\": \"stable\"}""#;
const QUOTED_LABEL_HEX_SPACE: &str =
    r#"--label "io.github.strukturpiloten.quadlet-lens.metadata={\"channel\":\x20\"stable\"}""#;
const ENTRYPOINT_SEPARATE_ARGUMENT: &str = r#"--entrypoint "[\"/usr/bin/env\",\"sh\"]""#;
const ENTRYPOINT_EQUALS_ARGUMENT: &str = r#""--entrypoint=[\"/usr/bin/env\",\"sh\"]""#;
const RUN_INIT_ARGUMENT: &str = "--init";
const RUN_INIT_FALSE_ARGUMENT: &str = "--init=false";
const NAMED_STOP_SIGNAL_ARGUMENT: &str = "--stop-signal SIGUSR1";
const NUMERIC_STOP_SIGNAL_ARGUMENT: &str = "--stop-signal 9";
const POSITIVE_STOP_TIMEOUT_ARGUMENT: &str = "--stop-timeout 37";
const ZERO_STOP_TIMEOUT_ARGUMENT: &str = "--stop-timeout 0";
const PULL_CASES: &[(&str, &str)] = &[
    ("pull-always.service", "--pull always"),
    ("pull-missing.service", "--pull missing"),
    ("pull-never.service", "--pull never"),
    ("pull-newer.service", "--pull newer"),
];
const PIDS_LIMIT_CASES: &[(&str, &str)] = &[
    ("pids-limit-finite.service", "--pids-limit 127"),
    ("pids-limit-unlimited.service", "--pids-limit -1"),
];
const HOSTNAME_SEPARATE_ARGUMENT: &str = "--hostname app.example";
const SHM_SIZE_CASES: &[(&str, &str)] = &[
    ("shm-size-container.service", "--shm-size 67108864b"),
    ("shm-size-zero.service", "--shm-size 0"),
    ("shm-size-pod.service", "--shm-size 32m"),
];
const CAP_DROP_ARGUMENTS: &[&str] = &[
    "--cap-drop cap_net_admin",
    "--cap-drop all",
    "--cap-drop cap_dac_override",
    "--cap-drop cap_ipc_owner",
];
const CAP_ADD_ARGUMENTS: &[&str] = &[
    "--cap-add cap_net_admin",
    "--cap-add all",
    "--cap-add cap_dac_override",
    "--cap-add cap_ipc_owner",
];
const CAP_DROP_ALL_ARGUMENT: &str = "--cap-drop all";
const CAP_ADD_NET_BIND_SERVICE_ARGUMENT: &str = "--cap-add cap_net_bind_service";
const TMPFS_ARGUMENT: &str = "--tmpfs /data:mode=755,uid=1009,gid=1009";
const TMPFS_PRE_RESET_PATHS: &[&str] = &["/earlier-one", "/earlier-two"];
const SYSCTL_ARGUMENT: &str = "--sysctl net.ipv4.ip_forward=1";
const SYSCTL_PRE_RESET_SETTINGS: &[&str] = &["net.ipv4.conf.all.rp_filter=2", "net.ipv4.tcp_syncookies=0"];
const ULIMIT_ARGUMENTS: &[&str] = &["--ulimit nproc=4096:8192", "--ulimit stack=-1:-1"];
const ULIMIT_PRE_RESET_LIMITS: &[&str] = &["core=0:0", "nofile=1024:2048"];
const ULIMIT_EMPTY_OR_ALTERNATE_FORMS: &[&str] = &["--ulimit=", "--ulimit \"\"", "--ulimit ''"];
const DEVICE_ARGUMENTS: &[&str] = &[
    "--device /dev/null:/dev/final-null:r",
    "--device /dev/zero:/dev/final-zero:w",
];
const DEVICE_PRE_RESET_MAPPINGS: &[&str] = &["/dev/null:/dev/pre-null:r", "/dev/zero:/dev/pre-zero:w"];
const DEVICE_EMPTY_OR_ALTERNATE_FORMS: &[&str] = &[
    "--device=",
    "--device \"\"",
    "--device ''",
    "--device=/dev/null:/dev/final-null:r",
    "--device=/dev/zero:/dev/final-zero:w",
];
const MEMORY_ARGUMENT: &str = "--memory 16777216b";
const MEMORY_EMPTY_OR_ALTERNATE_FORMS: &[&str] = &[
    "--memory=",
    "--memory \"\"",
    "--memory ''",
    "--memory \"16777216b\"",
    "--memory 32m",
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GeneratorMatrix {
    schema: u32,
    support_minimum: String,
    tracked_current: String,
    checked_on: String,
    official_image_maximum: String,
    source_repository: String,
    builder_reference: String,
    image: Vec<GeneratorImage>,
    source: Vec<GeneratorSource>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GeneratorImage {
    version: String,
    reference: String,
    #[serde(default)]
    smoke: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GeneratorSource {
    version: String,
    commit: String,
    #[serde(default)]
    smoke: bool,
}

#[test]
fn generator_matrix_is_exact_complete_and_digest_pinned() -> Result<(), String> {
    let matrix = parse_matrix()?;
    assert_eq!(matrix.schema, 1);
    assert_eq!(matrix.support_minimum, "5.4.0");
    assert_eq!(matrix.tracked_current, "6.0.2");
    assert_eq!(matrix.checked_on, "2026-08-06");
    assert_eq!(matrix.official_image_maximum, "5.8.2");

    assert_eq!(matrix.source_repository, "https://github.com/containers/podman.git");
    validate_digest_pinned_reference(&matrix.builder_reference, "docker.io/library/golang:")?;

    let image_versions: Vec<_> = matrix.image.iter().map(|image| image.version.as_str()).collect();
    assert_eq!(image_versions, EXPECTED_IMAGE_VERSIONS);
    assert_eq!(matrix.image.iter().filter(|image| image.smoke).count(), 2);
    assert!(matrix.image.first().is_some_and(|image| image.smoke));
    assert!(matrix.image.last().is_some_and(|image| image.smoke));

    let mut unique_references = BTreeSet::new();
    for image in &matrix.image {
        PodmanVersion::from_str(&image.version).map_err(|error| error.to_string())?;
        let prefix = format!("quay.io/podman/stable:v{}-immutable", image.version);
        validate_digest_pinned_reference(&image.reference, &prefix)?;
        if !unique_references.insert(&image.reference) {
            return Err(format!("duplicate generator image {}", image.reference));
        }
    }

    let source_versions: Vec<_> = matrix.source.iter().map(|source| source.version.as_str()).collect();
    assert_eq!(source_versions, EXPECTED_SOURCE_VERSIONS);
    assert_eq!(matrix.source.iter().filter(|source| source.smoke).count(), 1);
    assert!(matrix.source.last().is_some_and(|source| source.smoke));
    for source in &matrix.source {
        PodmanVersion::from_str(&source.version).map_err(|error| error.to_string())?;
        if source.commit.len() != 40
            || !source
                .commit
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(format!(
                "Podman {} source commit must be a full lowercase Git object ID",
                source.version
            ));
        }
    }

    assert_eq!(
        matrix.image.first().map(|image| image.version.as_str()),
        Some(matrix.support_minimum.as_str())
    );
    assert_eq!(
        matrix.image.last().map(|image| image.version.as_str()),
        Some(matrix.official_image_maximum.as_str())
    );
    assert_eq!(
        matrix.source.first().map(|source| source.version.as_str()),
        Some("5.8.3")
    );
    assert_eq!(
        matrix.source.last().map(|source| source.version.as_str()),
        Some(matrix.tracked_current.as_str())
    );
    let memory_versions = matrix
        .image
        .iter()
        .map(|image| image.version.as_str())
        .chain(matrix.source.iter().map(|source| source.version.as_str()))
        .filter(|version| PodmanVersion::from_str(version).is_ok_and(|version| version >= PodmanVersion::new(5, 5, 0)))
        .count();
    assert_eq!(memory_versions, 17);
    Ok(())
}

#[test]
#[ignore = "pulls or builds exact Podman releases and executes their Quadlet generators"]
fn supported_generators_match_the_first_conversion_fixture() -> Result<(), String> {
    let matrix = parse_matrix()?;
    let engine = env::var("QUADLET_LENS_CONTAINER_ENGINE").unwrap_or_else(|_| "podman".to_owned());
    let lane = env::var("QUADLET_LENS_GENERATOR_LANE").unwrap_or_else(|_| "smoke".to_owned());
    if lane != "smoke" && lane != "full" {
        return Err(format!("unknown generator lane `{lane}`; expected `smoke` or `full`"));
    }
    let version_filter = env::var("QUADLET_LENS_GENERATOR_VERSION").ok();
    let selected_images: Vec<_> = matrix
        .image
        .iter()
        .filter(|image| selected(&image.version, image.smoke, &lane, version_filter.as_deref()))
        .collect();
    let selected_sources: Vec<_> = matrix
        .source
        .iter()
        .filter(|source| selected(&source.version, source.smoke, &lane, version_filter.as_deref()))
        .collect();
    if selected_images.is_empty() && selected_sources.is_empty() {
        return Err(format!("generator selection is empty for lane `{lane}`"));
    }

    let fixture = fixture_directory()?;
    let expected = expected_fragments(&fixture)?;
    let memory_fixture = memory_fixture_directory()?;
    let memory_expected = expected_fragments(&memory_fixture)?;
    for image in selected_images {
        eprintln!("testing Podman {} with {}", image.version, image.reference);
        verify_image_version(&engine, image)?;
        let output = run_generator(&engine, image, &fixture)?;
        verify_generator_output(&image.version, &expected, &output)?;
        let memory_output = run_generator_raw(&engine, image, &memory_fixture)?;
        verify_memory_generator_output(&image.version, &memory_expected, &memory_output)?;
    }
    for source in selected_sources {
        eprintln!("testing Podman {} source at {}", source.version, source.commit);
        let generator = build_source_generator(&engine, &matrix, source)?;
        verify_source_version(&engine, &matrix.builder_reference, source, &generator)?;
        let output = run_source_generator(&engine, &matrix.builder_reference, source, &generator, &fixture)?;
        verify_generator_output(&source.version, &expected, &output)?;
        let memory_output =
            run_source_generator(&engine, &matrix.builder_reference, source, &generator, &memory_fixture)?;
        verify_memory_generator_output(&source.version, &memory_expected, &memory_output)?;
    }
    Ok(())
}

fn validate_digest_pinned_reference(reference: &str, expected_prefix: &str) -> Result<(), String> {
    let suffix = reference
        .strip_prefix(expected_prefix)
        .ok_or_else(|| format!("container image `{reference}` must use an exact tag and sha256 digest"))?;
    let (tag, digest) = suffix
        .split_once("@sha256:")
        .ok_or_else(|| format!("container image `{reference}` must use an exact tag and sha256 digest"))?;
    let tag_is_valid = if expected_prefix.ends_with(':') {
        !tag.is_empty()
    } else {
        tag.is_empty()
    };
    if !tag_is_valid || tag.contains('@') {
        return Err(format!("container image `{reference}` has an invalid tag"));
    }
    if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("container image `{reference}` has an invalid digest"));
    }
    Ok(())
}

fn selected(version: &str, smoke: bool, lane: &str, version_filter: Option<&str>) -> bool {
    version_filter.map_or_else(|| lane == "full" || smoke, |filter| version == filter)
}

fn parse_matrix() -> Result<GeneratorMatrix, String> {
    toml::from_str(MATRIX).map_err(|error| format!("invalid generator matrix: {error}"))
}

fn fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/first-conversion-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn memory_fixture_directory() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/memory-supported-range");
    path.canonicalize()
        .map_err(|error| format!("cannot resolve generator fixture {}: {error}", path.display()))
}

fn expected_fragments(fixture: &Path) -> Result<Vec<String>, String> {
    let path = fixture.join("expected-fragments.txt");
    let text = fs::read_to_string(&path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    let fragments: Vec<_> = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect();
    if fragments.is_empty() {
        return Err(format!("{} must contain expected fragments", path.display()));
    }
    Ok(fragments)
}

fn verify_image_version(engine: &str, image: &GeneratorImage) -> Result<(), String> {
    let output = Command::new(engine)
        .args([
            "run",
            "--rm",
            "--pull=missing",
            "--entrypoint",
            "/usr/bin/podman",
            &image.reference,
            "version",
            "--format",
            "{{.Client.Version}}",
        ])
        .output()
        .map_err(|error| format!("cannot execute `{engine}`: {error}"))?;
    ensure_success(&image.version, "version probe", &output)?;
    let actual = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if actual != image.version {
        return Err(format!("generator image for {} reports Podman {actual}", image.version));
    }
    Ok(())
}

fn run_generator(engine: &str, image: &GeneratorImage, fixture: &Path) -> Result<Output, String> {
    let output = run_generator_raw(engine, image, fixture)?;
    ensure_success(&image.version, "generator", &output)?;
    Ok(output)
}

fn run_generator_raw(engine: &str, image: &GeneratorImage, fixture: &Path) -> Result<Output, String> {
    let mount = format!("type=bind,src={},dst=/fixtures,ro", fixture.display());
    let output = Command::new(engine)
        .args([
            "run",
            "--rm",
            "--pull=missing",
            "--security-opt",
            "label=disable",
            "--mount",
            &mount,
            "-e",
            "QUADLET_UNIT_DIRS=/fixtures",
            "--entrypoint",
            "/usr/lib/systemd/system-generators/podman-system-generator",
            &image.reference,
            "-dryrun",
            "-no-kmsg-log",
        ])
        .output()
        .map_err(|error| format!("cannot execute `{engine}`: {error}"))?;
    Ok(output)
}

fn build_source_generator(engine: &str, matrix: &GeneratorMatrix, source: &GeneratorSource) -> Result<PathBuf, String> {
    let matrix_directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/generator-matrix");
    let source_directory = matrix_directory.join("source");
    let output_directory = matrix_directory.join("out").join(&source.version);
    let module_cache = matrix_directory.join("cache/go-mod");
    let build_cache = matrix_directory.join("cache/go-build");
    for directory in [&source_directory, &output_directory, &module_cache, &build_cache] {
        fs::create_dir_all(directory).map_err(|error| format!("cannot create {}: {error}", directory.display()))?;
    }

    checkout_source(&source_directory, &matrix.source_repository, source)?;
    let source_mount = bind_mount(&source_directory, "/src", true)?;
    let output_mount = bind_mount(&output_directory, "/out", false)?;
    let module_cache_mount = bind_mount(&module_cache, "/cache/mod", false)?;
    let build_cache_mount = bind_mount(&build_cache, "/cache/build", false)?;
    let user = container_user(engine)?;
    let output = Command::new(engine)
        .args([
            "run",
            "--rm",
            "--pull=missing",
            "--security-opt",
            "label=disable",
            "--user",
            &user,
            "--mount",
            &source_mount,
            "--mount",
            &output_mount,
            "--mount",
            &module_cache_mount,
            "--mount",
            &build_cache_mount,
            "-e",
            "CGO_ENABLED=0",
            "-e",
            "HOME=/tmp",
            "-e",
            "GOMODCACHE=/cache/mod",
            "-e",
            "GOCACHE=/cache/build",
            "-w",
            "/src",
            "--entrypoint",
            "/usr/local/go/bin/go",
            &matrix.builder_reference,
            "build",
            "-trimpath",
            "-o",
            "/out/quadlet",
            "./cmd/quadlet",
        ])
        .output()
        .map_err(|error| format!("cannot execute `{engine}`: {error}"))?;
    ensure_success(&source.version, "source generator build", &output)?;

    let generator = output_directory.join("quadlet");
    if !generator.is_file() {
        return Err(format!(
            "Podman {} build did not create {}",
            source.version,
            generator.display()
        ));
    }
    Ok(generator)
}

fn checkout_source(directory: &Path, repository: &str, source: &GeneratorSource) -> Result<(), String> {
    if !directory.join(".git").is_dir() {
        let output = Command::new("git")
            .args(["init", "--quiet"])
            .arg(directory)
            .output()
            .map_err(|error| format!("cannot execute `git`: {error}"))?;
        ensure_success(&source.version, "source checkout initialization", &output)?;
    }

    let output = Command::new("git")
        .arg("-C")
        .arg(directory)
        .args([
            "fetch",
            "--quiet",
            "--force",
            "--depth",
            "1",
            repository,
            &source.commit,
        ])
        .output()
        .map_err(|error| format!("cannot execute `git`: {error}"))?;
    ensure_success(&source.version, "source fetch", &output)?;

    let output = Command::new("git")
        .arg("-C")
        .arg(directory)
        .args(["checkout", "--quiet", "--force", "--detach", "FETCH_HEAD"])
        .output()
        .map_err(|error| format!("cannot execute `git`: {error}"))?;
    ensure_success(&source.version, "source checkout", &output)?;

    let output = Command::new("git")
        .arg("-C")
        .arg(directory)
        .args(["rev-parse", "HEAD"])
        .output()
        .map_err(|error| format!("cannot execute `git`: {error}"))?;
    ensure_success(&source.version, "source commit probe", &output)?;
    let actual = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if actual != source.commit {
        return Err(format!(
            "Podman {} checkout resolved to {actual}, expected {}",
            source.version, source.commit
        ));
    }
    Ok(())
}

fn container_user(engine: &str) -> Result<String, String> {
    if Path::new(engine).file_name().and_then(|name| name.to_str()) == Some("podman") {
        return Ok("0:0".to_owned());
    }
    let uid = command_text("id", &["-u"])?;
    let gid = command_text("id", &["-g"])?;
    Ok(format!("{uid}:{gid}"))
}

fn command_text(program: &str, arguments: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(arguments)
        .output()
        .map_err(|error| format!("cannot execute `{program}`: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "`{program}` failed with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn bind_mount(source: &Path, destination: &str, read_only: bool) -> Result<String, String> {
    let source = source
        .canonicalize()
        .map_err(|error| format!("cannot resolve {}: {error}", source.display()))?;
    let mode = if read_only { ",ro" } else { "" };
    Ok(format!("type=bind,src={},dst={destination}{mode}", source.display()))
}

fn verify_source_version(
    engine: &str,
    builder_reference: &str,
    source: &GeneratorSource,
    generator: &Path,
) -> Result<(), String> {
    let output_directory = generator
        .parent()
        .ok_or_else(|| format!("generator {} has no parent directory", generator.display()))?;
    let output_mount = bind_mount(output_directory, "/out", true)?;
    let output = Command::new(engine)
        .args([
            "run",
            "--rm",
            "--pull=missing",
            "--security-opt",
            "label=disable",
            "--mount",
            &output_mount,
            "--entrypoint",
            "/out/quadlet",
            builder_reference,
            "-version",
        ])
        .output()
        .map_err(|error| format!("cannot execute `{engine}`: {error}"))?;
    ensure_success(&source.version, "source generator version probe", &output)?;
    let actual = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if actual != source.version {
        return Err(format!(
            "source generator for Podman {} reports `{actual}`",
            source.version
        ));
    }
    Ok(())
}

fn run_source_generator(
    engine: &str,
    builder_reference: &str,
    source: &GeneratorSource,
    generator: &Path,
    fixture: &Path,
) -> Result<Output, String> {
    let output_directory = generator
        .parent()
        .ok_or_else(|| format!("generator {} has no parent directory", generator.display()))?;
    let output_mount = bind_mount(output_directory, "/out", true)?;
    let fixture_mount = bind_mount(fixture, "/fixtures", true)?;
    let output = Command::new(engine)
        .args([
            "run",
            "--rm",
            "--pull=missing",
            "--security-opt",
            "label=disable",
            "--mount",
            &output_mount,
            "--mount",
            &fixture_mount,
            "-e",
            "QUADLET_UNIT_DIRS=/fixtures",
            "--entrypoint",
            "/out/quadlet",
            builder_reference,
            "-dryrun",
            "-no-kmsg-log",
        ])
        .output()
        .map_err(|error| format!("cannot execute `{engine}`: {error}"))?;
    ensure_success(&source.version, "source generator", &output)?;
    Ok(output)
}

fn verify_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    let generated = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("{version} generator emitted non-UTF-8 output: {error}"))?;
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    verify_entrypoint_encoding(version, &generated, output)?;
    verify_run_init_argument(version, &generated, output)?;
    verify_stop_lifecycle_arguments(version, &generated, output)?;
    verify_pull_arguments(version, &generated, output)?;
    verify_pids_limit_arguments(version, &generated, output)?;
    verify_hostname_argument(version, &generated, output)?;
    verify_shm_size_arguments(version, &generated, output)?;
    verify_cap_drop_arguments(version, &generated, output)?;
    verify_cap_add_arguments(version, &generated, output)?;
    verify_cap_drop_all_add_one_arguments(version, &generated, output)?;
    verify_tmpfs_argument(version, &generated, output)?;
    verify_sysctl_argument(version, &generated, output)?;
    verify_ulimit_arguments(version, &generated, output)?;
    verify_device_arguments(version, &generated, output)?;
    verify_quoted_label_encoding(version, &generated, output)?;
    Ok(())
}

fn verify_memory_generator_output(version: &str, expected: &[String], output: &Output) -> Result<(), String> {
    let parsed = PodmanVersion::from_str(version).map_err(|error| error.to_string())?;
    let generated = String::from_utf8(output.stdout.clone())
        .map_err(|error| format!("{version} generator emitted non-UTF-8 output: {error}"))?;
    let diagnostics = String::from_utf8_lossy(&output.stderr);

    if parsed < PodmanVersion::new(5, 5, 0) {
        let memory_argument_count = generated.matches("--memory").count();
        let rejected_or_excluded =
            !output.status.success() || !generated.contains("---memory.service---") || diagnostics.contains("Memory");
        if memory_argument_count != 0 || !rejected_or_excluded {
            return Err(format!(
                "Podman {version} predates native Memory support and must reject or exclude the fixture without emitting --memory; found memory-arguments={memory_argument_count}, status={}\nstdout:\n{generated}\nstderr:\n{diagnostics}",
                output.status
            ));
        }
        eprintln!("Podman {version} Memory: unsupported key is rejected or excluded with no --memory argument");
        return Ok(());
    }

    ensure_success(version, "memory generator", output)?;
    for fragment in expected {
        if !generated.contains(fragment) {
            return Err(format!(
                "Podman {version} memory generator output is missing fragment `{fragment}`\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            ));
        }
    }
    let generated_unit = generated_unit(version, &generated, "memory.service", output)?;
    let podman_run = generated_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman run "))
        .ok_or_else(|| {
            format!(
                "Podman {version} generator output for memory.service is missing its Podman run command\nstdout:\n{generated}\nstderr:\n{diagnostics}"
            )
        })?;
    let expected_count = podman_run.matches(MEMORY_ARGUMENT).count();
    let all_memory_count = podman_run.matches("--memory").count();
    let empty_or_alternate_forms: Vec<_> = MEMORY_EMPTY_OR_ALTERNATE_FORMS
        .iter()
        .copied()
        .filter(|form| podman_run.contains(form))
        .collect();
    if expected_count != 1 || all_memory_count != 1 || !empty_or_alternate_forms.is_empty() {
        return Err(format!(
            "Podman {version} generator output for memory.service must contain exactly one final `{MEMORY_ARGUMENT}` and no duplicate, equals, empty, quoted, or alternate form; found expected={expected_count}, all-memory={all_memory_count}, empty-or-alternate={empty_or_alternate_forms:?}\nstdout:\n{generated}\nstderr:\n{diagnostics}"
        ));
    }
    eprintln!("Podman {version} Memory: last effective assignment emits exactly one --memory 16777216b argument");
    Ok(())
}

fn verify_entrypoint_encoding(version: &str, generated: &str, output: &Output) -> Result<(), String> {
    let parsed = PodmanVersion::from_str(version).map_err(|error| error.to_string())?;
    let separate_count = generated.matches(ENTRYPOINT_SEPARATE_ARGUMENT).count();
    let equals_count = generated.matches(ENTRYPOINT_EQUALS_ARGUMENT).count();
    let (expected_name, expected_count, unexpected_count) = if parsed < PodmanVersion::new(5, 8, 2) {
        ("separate-argument", separate_count, equals_count)
    } else {
        ("equals-argument", equals_count, separate_count)
    };
    if expected_count != 1 || unexpected_count != 0 {
        return Err(format!(
            "Podman {version} generator output must contain exactly one {expected_name} JSON-array entrypoint encoding and no other supported encoding; found separate-argument={separate_count}, equals-argument={equals_count}\nstdout:\n{generated}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    eprintln!("Podman {version} JSON-array entrypoint encoding: {expected_name}");
    Ok(())
}

fn verify_run_init_argument(version: &str, generated: &str, output: &Output) -> Result<(), String> {
    let true_unit = generated_unit(version, generated, "app.service", output)?;
    let true_count = true_unit.matches(RUN_INIT_ARGUMENT).count();
    if true_count != 1 {
        return Err(format!(
            "Podman {version} generator output for authored `RunInit=true` must contain exactly one {RUN_INIT_ARGUMENT} argument; found {true_count}\nstdout:\n{generated}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let false_unit = generated_unit(version, generated, "run-init-false.service", output)?;
    let false_count = false_unit.matches(RUN_INIT_FALSE_ARGUMENT).count();
    let false_form_count = false_unit.matches(RUN_INIT_ARGUMENT).count();
    if false_count != 1 || false_form_count != 1 {
        return Err(format!(
            "Podman {version} generator output for authored `RunInit=false` must contain exactly one {RUN_INIT_FALSE_ARGUMENT} argument and no other {RUN_INIT_ARGUMENT} form; found explicit-false={false_count}, all-forms={false_form_count}\nstdout:\n{generated}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    eprintln!("Podman {version} RunInit: true emits one --init; false emits one --init=false");
    Ok(())
}

fn generated_unit<'a>(version: &str, generated: &'a str, unit: &str, output: &Output) -> Result<&'a str, String> {
    let marker = format!("---{unit}---");
    let (_, remainder) = generated.split_once(&marker).ok_or_else(|| {
        format!(
            "Podman {version} generator output is missing unit marker `{marker}`\nstdout:\n{generated}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        )
    })?;
    Ok(remainder.split("\n---").next().unwrap_or(remainder))
}

fn verify_stop_lifecycle_arguments(version: &str, generated: &str, output: &Output) -> Result<(), String> {
    for argument in [
        NAMED_STOP_SIGNAL_ARGUMENT,
        NUMERIC_STOP_SIGNAL_ARGUMENT,
        POSITIVE_STOP_TIMEOUT_ARGUMENT,
        ZERO_STOP_TIMEOUT_ARGUMENT,
    ] {
        let count = generated.matches(argument).count();
        if count != 1 {
            return Err(format!(
                "Podman {version} generator output must contain exactly one `{argument}` observation; found {count}\nstdout:\n{generated}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    eprintln!(
        "Podman {version} container stop lifecycle: named and numeric signals, positive timeout, and zero timeout preserved"
    );
    Ok(())
}

fn verify_pull_arguments(version: &str, generated: &str, output: &Output) -> Result<(), String> {
    for &(unit, argument) in PULL_CASES {
        let generated_unit = generated_unit(version, generated, unit, output)?;
        let expected_count = generated_unit.matches(argument).count();
        let all_pull_count = generated_unit.matches("--pull").count();
        if expected_count != 1 || all_pull_count != 1 {
            return Err(format!(
                "Podman {version} generator output for {unit} must contain exactly one `{argument}` and no other --pull form; found expected={expected_count}, all-pull={all_pull_count}\nstdout:\n{generated}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    eprintln!("Podman {version} Pull: always, missing, never, and newer each emit their matching --pull argument");
    Ok(())
}

fn verify_pids_limit_arguments(version: &str, generated: &str, output: &Output) -> Result<(), String> {
    for &(unit, argument) in PIDS_LIMIT_CASES {
        let generated_unit = generated_unit(version, generated, unit, output)?;
        let expected_count = generated_unit.matches(argument).count();
        let all_limit_count = generated_unit.matches("--pids-limit").count();
        if expected_count != 1 || all_limit_count != 1 {
            return Err(format!(
                "Podman {version} generator output for {unit} must contain exactly one `{argument}` and no other --pids-limit form; found expected={expected_count}, all-pids-limit={all_limit_count}\nstdout:\n{generated}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    eprintln!("Podman {version} PidsLimit: finite 127 and unlimited -1 each emit one matching --pids-limit argument");
    Ok(())
}

fn verify_hostname_argument(version: &str, generated: &str, output: &Output) -> Result<(), String> {
    let generated_unit = generated_unit(version, generated, "hostname.service", output)?;
    let separate_count = generated_unit.matches(HOSTNAME_SEPARATE_ARGUMENT).count();
    let all_hostname_count = generated_unit.matches("--hostname").count();
    if separate_count != 1 || all_hostname_count != 1 {
        return Err(format!(
            "Podman {version} generator output for hostname.service must contain exactly one `--hostname app.example` argument and no duplicate hostname form; found expected={separate_count}, all-hostname={all_hostname_count}\nstdout:\n{generated}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    eprintln!("Podman {version} HostName: app.example emits exactly one --hostname argument");
    Ok(())
}

fn verify_shm_size_arguments(version: &str, generated: &str, output: &Output) -> Result<(), String> {
    for &(unit, argument) in SHM_SIZE_CASES {
        let generated_unit = generated_unit(version, generated, unit, output)?;
        let expected_count = generated_unit.matches(argument).count();
        let all_shm_size_count = generated_unit.matches("--shm-size").count();
        if expected_count != 1 || all_shm_size_count != 1 {
            return Err(format!(
                "Podman {version} generator output for {unit} must contain exactly one `{argument}` and no duplicate --shm-size form; found expected={expected_count}, all-shm-size={all_shm_size_count}\nstdout:\n{generated}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    let member_unit = generated_unit(version, generated, "shm-size-pod-member.service", output)?;
    let member_count = member_unit.matches("--shm-size").count();
    if member_count != 0 {
        return Err(format!(
            "Podman {version} generator output for the container joining shm-size.pod must not duplicate the pod-owned --shm-size argument; found {member_count}\nstdout:\n{generated}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    eprintln!(
        "Podman {version} ShmSize: positive container, zero container, and pod-owned values each emit exactly one matching --shm-size argument"
    );
    Ok(())
}

fn verify_cap_drop_arguments(version: &str, generated: &str, output: &Output) -> Result<(), String> {
    let generated_unit = generated_unit(version, generated, "cap-drop.service", output)?;
    let all_drop_count = generated_unit.matches("--cap-drop").count();
    let add_count = generated_unit.matches("--cap-add").count();
    let mut positions = Vec::with_capacity(CAP_DROP_ARGUMENTS.len());
    for argument in CAP_DROP_ARGUMENTS {
        let matches: Vec<_> = generated_unit
            .match_indices(argument)
            .map(|(position, _)| position)
            .collect();
        if matches.len() != 1 {
            return Err(format!(
                "Podman {version} generator output for cap-drop.service must contain exactly one `{argument}`; found {}\nstdout:\n{generated}\nstderr:\n{}",
                matches.len(),
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        positions.push(matches[0]);
    }
    if all_drop_count != CAP_DROP_ARGUMENTS.len()
        || add_count != 0
        || !positions.windows(2).all(|pair| pair[0] < pair[1])
    {
        return Err(format!(
            "Podman {version} generator output for cap-drop.service must contain exactly four ordered lowercase separate-argument --cap-drop forms and no --cap-add form; found cap-drop={all_drop_count}, cap-add={add_count}, positions={positions:?}\nstdout:\n{generated}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    eprintln!(
        "Podman {version} DropCapability: repeated and space-separated values emit four ordered lowercase --cap-drop arguments and no --cap-add"
    );
    Ok(())
}

fn verify_cap_add_arguments(version: &str, generated: &str, output: &Output) -> Result<(), String> {
    let generated_unit = generated_unit(version, generated, "cap-add.service", output)?;
    let add_count = generated_unit.matches("--cap-add").count();
    let drop_count = generated_unit.matches("--cap-drop").count();
    let mut positions = Vec::with_capacity(CAP_ADD_ARGUMENTS.len());
    for argument in CAP_ADD_ARGUMENTS {
        let matches: Vec<_> = generated_unit
            .match_indices(argument)
            .map(|(position, _)| position)
            .collect();
        if matches.len() != 1 {
            return Err(format!(
                "Podman {version} generator output for cap-add.service must contain exactly one `{argument}`; found {}\nstdout:\n{generated}\nstderr:\n{}",
                matches.len(),
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        positions.push(matches[0]);
    }
    if add_count != CAP_ADD_ARGUMENTS.len() || drop_count != 0 || !positions.windows(2).all(|pair| pair[0] < pair[1]) {
        return Err(format!(
            "Podman {version} generator output for cap-add.service must contain exactly four ordered lowercase separate-argument --cap-add forms and no --cap-drop form; found cap-add={add_count}, cap-drop={drop_count}, positions={positions:?}\nstdout:\n{generated}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    eprintln!(
        "Podman {version} AddCapability: repeated and space-separated values emit four ordered lowercase --cap-add arguments and no --cap-drop"
    );
    Ok(())
}

fn verify_cap_drop_all_add_one_arguments(version: &str, generated: &str, output: &Output) -> Result<(), String> {
    let generated_unit = generated_unit(version, generated, "cap-drop-all-add-one.service", output)?;
    let all_add_count = generated_unit.matches("--cap-add").count();
    let all_drop_count = generated_unit.matches("--cap-drop").count();
    let drop_positions: Vec<_> = generated_unit
        .match_indices(CAP_DROP_ALL_ARGUMENT)
        .map(|(position, _)| position)
        .collect();
    let add_positions: Vec<_> = generated_unit
        .match_indices(CAP_ADD_NET_BIND_SERVICE_ARGUMENT)
        .map(|(position, _)| position)
        .collect();
    if all_drop_count != 1
        || all_add_count != 1
        || drop_positions.len() != 1
        || add_positions.len() != 1
        || drop_positions[0] >= add_positions[0]
    {
        return Err(format!(
            "Podman {version} generator output for cap-drop-all-add-one.service must contain exactly one `{CAP_DROP_ALL_ARGUMENT}` followed by exactly one `{CAP_ADD_NET_BIND_SERVICE_ARGUMENT}` and no other capability arguments; found cap-drop={all_drop_count}, cap-add={all_add_count}, drop-positions={drop_positions:?}, add-positions={add_positions:?}\nstdout:\n{generated}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    eprintln!("Podman {version} capability ordering: one --cap-drop all precedes one --cap-add cap_net_bind_service");
    Ok(())
}

fn verify_tmpfs_argument(version: &str, generated: &str, output: &Output) -> Result<(), String> {
    let generated_unit = generated_unit(version, generated, "tmpfs.service", output)?;
    let podman_run = generated_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman run "))
        .ok_or_else(|| {
            format!(
                "Podman {version} generator output for tmpfs.service is missing its Podman run command\nstdout:\n{generated}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            )
        })?;
    let expected_count = podman_run.matches(TMPFS_ARGUMENT).count();
    let all_tmpfs_count = podman_run.matches("--tmpfs").count();
    let pre_reset_paths: Vec<_> = TMPFS_PRE_RESET_PATHS
        .iter()
        .copied()
        .filter(|path| podman_run.contains(path))
        .collect();
    if expected_count != 1 || all_tmpfs_count != 1 || !pre_reset_paths.is_empty() {
        return Err(format!(
            "Podman {version} generator output for tmpfs.service must contain exactly one post-reset `{TMPFS_ARGUMENT}`, no other --tmpfs form, and no pre-reset path; found expected={expected_count}, all-tmpfs={all_tmpfs_count}, pre-reset={pre_reset_paths:?}\nstdout:\n{generated}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    eprintln!(
        "Podman {version} Tmpfs: LookupAll reset leaves exactly one --tmpfs /data:mode=755,uid=1009,gid=1009 argument"
    );
    Ok(())
}

fn verify_sysctl_argument(version: &str, generated: &str, output: &Output) -> Result<(), String> {
    let generated_unit = generated_unit(version, generated, "sysctl.service", output)?;
    let podman_run = generated_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman run "))
        .ok_or_else(|| {
            format!(
                "Podman {version} generator output for sysctl.service is missing its Podman run command\nstdout:\n{generated}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            )
        })?;
    let expected_count = podman_run.matches(SYSCTL_ARGUMENT).count();
    let all_sysctl_count = podman_run.matches("--sysctl").count();
    let pre_reset_settings: Vec<_> = SYSCTL_PRE_RESET_SETTINGS
        .iter()
        .copied()
        .filter(|setting| podman_run.contains(setting))
        .collect();
    if expected_count != 1 || all_sysctl_count != 1 || !pre_reset_settings.is_empty() {
        return Err(format!(
            "Podman {version} generator output for sysctl.service must contain exactly one post-reset `{SYSCTL_ARGUMENT}`, no other --sysctl form, and neither pre-reset setting; found expected={expected_count}, all-sysctl={all_sysctl_count}, pre-reset={pre_reset_settings:?}\nstdout:\n{generated}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    eprintln!(
        "Podman {version} Sysctl: LookupAllStrv reset leaves exactly one --sysctl net.ipv4.ip_forward=1 argument"
    );
    Ok(())
}

fn verify_ulimit_arguments(version: &str, generated: &str, output: &Output) -> Result<(), String> {
    let generated_unit = generated_unit(version, generated, "ulimit.service", output)?;
    let podman_run = generated_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman run "))
        .ok_or_else(|| {
            format!(
                "Podman {version} generator output for ulimit.service is missing its Podman run command\nstdout:\n{generated}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            )
        })?;

    let mut positions = Vec::with_capacity(ULIMIT_ARGUMENTS.len());
    for argument in ULIMIT_ARGUMENTS {
        let matches: Vec<_> = podman_run
            .match_indices(argument)
            .map(|(position, _)| position)
            .collect();
        if matches.len() != 1 {
            return Err(format!(
                "Podman {version} generator output for ulimit.service must contain `{argument}` exactly once; found {}\nstdout:\n{generated}\nstderr:\n{}",
                matches.len(),
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        positions.push(matches[0]);
    }

    let all_ulimit_count = podman_run.matches("--ulimit").count();
    let pre_reset_limits: Vec<_> = ULIMIT_PRE_RESET_LIMITS
        .iter()
        .copied()
        .filter(|limit| podman_run.contains(limit))
        .collect();
    let empty_or_alternate_forms: Vec<_> = ULIMIT_EMPTY_OR_ALTERNATE_FORMS
        .iter()
        .copied()
        .filter(|form| podman_run.contains(form))
        .collect();
    if all_ulimit_count != ULIMIT_ARGUMENTS.len()
        || !positions.windows(2).all(|pair| pair[0] < pair[1])
        || !pre_reset_limits.is_empty()
        || !empty_or_alternate_forms.is_empty()
    {
        return Err(format!(
            "Podman {version} generator output for ulimit.service must contain exactly two ordered post-reset Ulimit arguments, no duplicates, no pre-reset limit, and no empty/alternate form; found all-ulimit={all_ulimit_count}, positions={positions:?}, pre-reset={pre_reset_limits:?}, empty-or-alternate={empty_or_alternate_forms:?}\nstdout:\n{generated}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    eprintln!(
        "Podman {version} Ulimit: LookupAll reset leaves exactly two ordered --ulimit nproc=4096:8192 and --ulimit stack=-1:-1 arguments"
    );
    Ok(())
}

fn verify_device_arguments(version: &str, generated: &str, output: &Output) -> Result<(), String> {
    let generated_unit = generated_unit(version, generated, "device.service", output)?;
    let podman_run = generated_unit
        .lines()
        .find(|line| line.starts_with("ExecStart=/usr/bin/podman run "))
        .ok_or_else(|| {
            format!(
                "Podman {version} generator output for device.service is missing its Podman run command\nstdout:\n{generated}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            )
        })?;

    let mut positions = Vec::with_capacity(DEVICE_ARGUMENTS.len());
    for argument in DEVICE_ARGUMENTS {
        let matches: Vec<_> = podman_run
            .match_indices(argument)
            .map(|(position, _)| position)
            .collect();
        if matches.len() != 1 {
            return Err(format!(
                "Podman {version} generator output for device.service must contain `{argument}` exactly once; found {}\nstdout:\n{generated}\nstderr:\n{}",
                matches.len(),
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        positions.push(matches[0]);
    }

    let all_device_count = podman_run.matches("--device").count();
    let pre_reset_mappings: Vec<_> = DEVICE_PRE_RESET_MAPPINGS
        .iter()
        .copied()
        .filter(|mapping| podman_run.contains(mapping))
        .collect();
    let empty_or_alternate_forms: Vec<_> = DEVICE_EMPTY_OR_ALTERNATE_FORMS
        .iter()
        .copied()
        .filter(|form| podman_run.contains(form))
        .collect();
    if all_device_count != DEVICE_ARGUMENTS.len()
        || !positions.windows(2).all(|pair| pair[0] < pair[1])
        || !pre_reset_mappings.is_empty()
        || !empty_or_alternate_forms.is_empty()
    {
        return Err(format!(
            "Podman {version} generator output for device.service must contain exactly two ordered post-reset AddDevice arguments, no duplicates, no pre-reset mapping, and no empty/alternate form; found all-device={all_device_count}, positions={positions:?}, pre-reset={pre_reset_mappings:?}, empty-or-alternate={empty_or_alternate_forms:?}\nstdout:\n{generated}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    eprintln!("Podman {version} AddDevice: LookupAllStrv reset leaves exactly two ordered final --device arguments");
    Ok(())
}

fn verify_quoted_label_encoding(version: &str, generated: &str, output: &Output) -> Result<(), String> {
    let parsed = PodmanVersion::from_str(version).map_err(|error| error.to_string())?;
    let literal_count = generated.matches(QUOTED_LABEL_LITERAL_SPACE).count();
    let hex_count = generated.matches(QUOTED_LABEL_HEX_SPACE).count();
    let (expected_name, expected_count, unexpected_count) = if parsed.major() == 5 && parsed.minor() == 4 {
        ("literal-space", literal_count, hex_count)
    } else {
        ("hex-space", hex_count, literal_count)
    };
    if expected_count != 1 || unexpected_count != 0 {
        return Err(format!(
            "Podman {version} generator output must contain exactly one {expected_name} quoted-label encoding and no other supported encoding; found literal-space={literal_count}, hex-space={hex_count}\nstdout:\n{generated}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    eprintln!("Podman {version} quoted-label encoding: {expected_name}");
    Ok(())
}

fn ensure_success(version: &str, operation: &str, output: &Output) -> Result<(), String> {
    if output.status.success() {
        return Ok(());
    }
    Err(format!(
        "Podman {version} {operation} failed with {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    ))
}
