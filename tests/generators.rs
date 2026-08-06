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
    for image in selected_images {
        eprintln!("testing Podman {} with {}", image.version, image.reference);
        verify_image_version(&engine, image)?;
        let output = run_generator(&engine, image, &fixture)?;
        verify_generator_output(&image.version, &expected, &output)?;
    }
    for source in selected_sources {
        eprintln!("testing Podman {} source at {}", source.version, source.commit);
        let generator = build_source_generator(&engine, &matrix, source)?;
        verify_source_version(&engine, &matrix.builder_reference, source, &generator)?;
        let output = run_source_generator(&engine, &matrix.builder_reference, source, &generator, &fixture)?;
        verify_generator_output(&source.version, &expected, &output)?;
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
    ensure_success(&image.version, "generator", &output)?;
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
    verify_quoted_label_encoding(version, &generated, output)?;
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
    let count = generated.matches(RUN_INIT_ARGUMENT).count();
    if count != 1 {
        return Err(format!(
            "Podman {version} generator output must contain exactly one {RUN_INIT_ARGUMENT} argument; found {count}\nstdout:\n{generated}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
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
