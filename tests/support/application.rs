//! Strict, test-only support for copied application unit sets.
#![allow(dead_code)] // This helper is included by two focused integration-test crates.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde::Deserialize;
use toml::{Table, Value};

pub(crate) struct CopiedUnit<'a> {
    pub(crate) path: &'a str,
    pub(crate) hash_key: &'a str,
    pub(crate) sha256: &'a str,
}

pub(crate) const NEXTCLOUD_ID: &str = "boxferry-nextcloud-application";
pub(crate) const FORGEJO_ID: &str = "boxferry-forgejo-application";

const NEXTCLOUD_REVISION: &str = "18716257362932e7520d5aed68412ac7c8492e6d";
const FORGEJO_REVISION: &str = "68e08b9a86bcb9ced34e8d831563e732176275a1";
const NEXTCLOUD_EXPECTATIONS_SHA256: &str = "24fa67a440a23b1de9b200ae354741237f030d203fe0ecca31c4d5f3d5397d2b";
const FORGEJO_EXPECTATIONS_SHA256: &str = "07317c5e2361e1e51b93c17175148b30b1f3ede2fdfa6e82f31f03a46f18d1fa";
const PODMAN_RUN_FLAGS: &[&str] = &["--replace", "--rm", "-d"];
const PODMAN_RUN_VALUE_OPTIONS: &[&str] = &[
    "--name",
    "--cgroups",
    "--network",
    "--sdnotify",
    "-v",
    "--volume",
    "-p",
    "--publish",
    "-e",
    "--env",
    "--health-cmd",
    "--health-interval",
    "--health-retries",
    "--health-timeout",
    "--user",
];

pub(crate) const NEXTCLOUD_UNITS: &[CopiedUnit<'static>] = &[
    CopiedUnit {
        path: "app.container",
        hash_key: "app-container-sha256",
        sha256: "7cd2f0a4f7c49fca3380fc3001c927c6660e25556ca83245d977e1a270e11bc5",
    },
    CopiedUnit {
        path: "backend.network",
        hash_key: "backend-network-sha256",
        sha256: "7be93bb54584d2477fd57fedcb00e123d1d00794ff143034f9930290430b480a",
    },
    CopiedUnit {
        path: "cache.container",
        hash_key: "cache-container-sha256",
        sha256: "f21a58fb97f1f6874500a047b0b945d282bbe8b94fcf6b8cd1f03c008c84cece",
    },
    CopiedUnit {
        path: "cron.container",
        hash_key: "cron-container-sha256",
        sha256: "378e6f5d7e95c6ecc45f48143154e5055c312e042d618448eaa0e3f479419046",
    },
    CopiedUnit {
        path: "db-data.volume",
        hash_key: "db-data-volume-sha256",
        sha256: "e3e5d6e859dd60294fa3a971fb52111a346a3f6cd42faca1fc46388f3c2348b2",
    },
    CopiedUnit {
        path: "db.container",
        hash_key: "db-container-sha256",
        sha256: "45e26e7b16a997615cfec93ec56d369a032abc6181b05a8b54d69122688dfca3",
    },
    CopiedUnit {
        path: "edge-proxy.container",
        hash_key: "edge-proxy-container-sha256",
        sha256: "3e487e00c12a414c42669aa0e84112cafd59fd08ae8a350b653f8b6603a98cef",
    },
    CopiedUnit {
        path: "edge.network",
        hash_key: "edge-network-sha256",
        sha256: "15c4fdbc7b62313d7d0d5bcf7689e03da9bb44376fbcecb149baf296a8bcd9bf",
    },
    CopiedUnit {
        path: "frontend.container",
        hash_key: "frontend-container-sha256",
        sha256: "0e4603434a43675e6a118b3bd99659344b2230eedcce778b576d770acdc9ec7e",
    },
    CopiedUnit {
        path: "init.container",
        hash_key: "init-container-sha256",
        sha256: "4fa8b5c2e142f56ef1547cadf50bf1e2702adc3b9aaf226dfd17bb4828673514",
    },
    CopiedUnit {
        path: "nextcloud-data.volume",
        hash_key: "nextcloud-data-volume-sha256",
        sha256: "e3e5d6e859dd60294fa3a971fb52111a346a3f6cd42faca1fc46388f3c2348b2",
    },
    CopiedUnit {
        path: "redis-data.volume",
        hash_key: "redis-data-volume-sha256",
        sha256: "e3e5d6e859dd60294fa3a971fb52111a346a3f6cd42faca1fc46388f3c2348b2",
    },
    CopiedUnit {
        path: "second-app.container",
        hash_key: "second-app-container-sha256",
        sha256: "519276edbda2848f3bfe5a0ad0af42ace597b6661ad71a2fccc26a1dfed289f9",
    },
];

pub(crate) const FORGEJO_UNITS: &[CopiedUnit<'static>] = &[
    CopiedUnit {
        path: "backend.network",
        hash_key: "backend-network-sha256",
        sha256: "7be93bb54584d2477fd57fedcb00e123d1d00794ff143034f9930290430b480a",
    },
    CopiedUnit {
        path: "db-data.volume",
        hash_key: "db-data-volume-sha256",
        sha256: "e3e5d6e859dd60294fa3a971fb52111a346a3f6cd42faca1fc46388f3c2348b2",
    },
    CopiedUnit {
        path: "db.container",
        hash_key: "db-container-sha256",
        sha256: "0efc85997ad786c5d301402088339f837be98ac4c7943fa98e6cfcda2335d40e",
    },
    CopiedUnit {
        path: "forgejo-data.volume",
        hash_key: "forgejo-data-volume-sha256",
        sha256: "e3e5d6e859dd60294fa3a971fb52111a346a3f6cd42faca1fc46388f3c2348b2",
    },
    CopiedUnit {
        path: "forgejo.container",
        hash_key: "forgejo-container-sha256",
        sha256: "be810fcf9f32ef81fff1ca8a473de5c4fe8368dafa529ae4624dd17e4a2f962d",
    },
];

pub(crate) fn verify_known_application_fixture(id: &str) -> Result<PathBuf, String> {
    match id {
        NEXTCLOUD_ID => {
            verify_application_fixture(id, NEXTCLOUD_REVISION, NEXTCLOUD_UNITS, NEXTCLOUD_EXPECTATIONS_SHA256)
        }
        FORGEJO_ID => verify_application_fixture(id, FORGEJO_REVISION, FORGEJO_UNITS, FORGEJO_EXPECTATIONS_SHA256),
        _ => Err(format!("unknown application contract `{id}`")),
    }
}

pub(crate) fn known_application_units(id: &str) -> Result<&'static [CopiedUnit<'static>], String> {
    match id {
        NEXTCLOUD_ID => Ok(NEXTCLOUD_UNITS),
        FORGEJO_ID => Ok(FORGEJO_UNITS),
        _ => Err(format!("unknown application contract `{id}`")),
    }
}

pub(crate) fn verify_application_fixture(
    id: &str,
    revision: &str,
    units: &[CopiedUnit<'_>],
    expected_systemd_sha256: &str,
) -> Result<PathBuf, String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/generators")
        .join(id);
    let manifest_path = root.join("fixture.toml");
    let manifest_text = fs::read_to_string(&manifest_path)
        .map_err(|error| format!("failed to read {}: {error}", manifest_path.display()))?;
    let manifest = manifest_text
        .parse::<Table>()
        .map_err(|error| format!("invalid {}: {error}", manifest_path.display()))?;

    require_integer(&manifest, "schema", 1, id)?;
    require_string(&manifest, "id", id, id)?;
    require_string(&manifest, "suite", "generators", id)?;
    if manifest.get("secrets_reviewed").and_then(Value::as_bool) != Some(true) {
        return Err(format!("{id}: secrets_reviewed must be true"));
    }

    let provenance = require_table(&manifest, "provenance", id)?;
    require_string(provenance, "source", "external", id)?;
    require_string(provenance, "revision", revision, id)?;
    require_string(provenance, "license", "MPL-2.0", id)?;
    require_string(provenance, "redistribution", "allowed", id)?;
    let url = required_string(provenance, "url", id)?;
    if !url.starts_with("https://github.com/Strukturpiloten/boxferry/tree/") || !url.contains(revision) {
        return Err(format!("{id}: provenance URL must contain the pinned revision"));
    }
    let modifications = required_string(provenance, "modifications", id)?;
    if !modifications.contains("copied byte-for-byte")
        || !modifications.contains("independent QuadletLens-authored oracle")
    {
        return Err(format!(
            "{id}: modifications must distinguish copied units from the independent oracle"
        ));
    }

    let extensions = require_table(&manifest, "extensions", id)?;
    let application = require_table(extensions, "application", id)?;
    require_string(
        application,
        "source-repository",
        "https://github.com/Strukturpiloten/boxferry",
        id,
    )?;
    require_string(
        application,
        "source-license-sha256",
        "3f3d9e0024b1921b067d6f7f88deb4a60cbe7a78e76c64e3f1d7fc3b779b9d04",
        id,
    )?;

    let mut expected_files = BTreeSet::from(["expected-systemd.toml"]);
    for unit in units {
        if !is_safe_relative(unit.path) {
            return Err(format!("{id}: unsafe unit path `{}`", unit.path));
        }
        if !expected_files.insert(unit.path) {
            return Err(format!("{id}: duplicate unit path `{}`", unit.path));
        }
        require_string(application, unit.hash_key, unit.sha256, id)?;
        let actual = sha256_file(&root.join(unit.path))?;
        if actual != unit.sha256 {
            return Err(format!(
                "{id}: {} SHA-256 is {actual}, expected {}",
                unit.path, unit.sha256
            ));
        }
    }

    let expected_actual = sha256_file(&root.join("expected-systemd.toml"))?;
    if expected_actual != expected_systemd_sha256 {
        return Err(format!(
            "{id}: expected-systemd.toml SHA-256 is {expected_actual}, expected {expected_systemd_sha256}"
        ));
    }

    let declared = string_set(&manifest, "files", id)?;
    if declared != expected_files {
        return Err(format!("{id}: fixture files do not match the application contract"));
    }
    let mut expected_directory_files = expected_files;
    expected_directory_files.insert("fixture.toml");
    verify_directory_members(&root, &expected_directory_files)?;
    Ok(root)
}

pub(crate) fn read_fixture(root: &Path, path: &str) -> Result<String, String> {
    if !is_safe_relative(path) {
        return Err(format!("unsafe fixture path `{path}`"));
    }
    fs::read_to_string(root.join(path)).map_err(|error| format!("failed to read {path}: {error}"))
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct GeneratedApplicationContract {
    schema: u32,
    #[serde(rename = "target-version")]
    target_version: String,
    application: String,
    units: Vec<String>,
    #[serde(rename = "unit")]
    unit_rules: Vec<GeneratedUnitRule>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct GeneratedUnitRule {
    name: String,
    source: String,
    #[serde(rename = "exec-required", default)]
    exec_required: Vec<String>,
    #[serde(rename = "exec-forbidden", default)]
    exec_forbidden: Vec<String>,
    #[serde(default)]
    requires: Vec<String>,
    #[serde(default)]
    after: Vec<String>,
    #[serde(default)]
    environment: Vec<String>,
    #[serde(default)]
    publish: Vec<String>,
    #[serde(default)]
    network: Vec<String>,
    #[serde(default)]
    mount: Vec<String>,
    #[serde(default)]
    internal: Option<bool>,
    #[serde(default)]
    exec: Vec<String>,
    #[serde(default)]
    image: Option<String>,
    #[serde(default)]
    command: Vec<String>,
    #[serde(default)]
    ordered: Vec<OrderedFragments>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct OrderedFragments {
    fragments: Vec<String>,
}

pub(crate) fn application_contract_target(root: &Path) -> Result<String, String> {
    Ok(load_generated_contract(root)?.target_version)
}

#[expect(
    clippy::too_many_lines,
    reason = "One ordered verifier keeps the complete independent application contract auditable."
)]
pub(crate) fn verify_generated_application(
    id: &str,
    root: &Path,
    version: &str,
    generated: &str,
) -> Result<(), String> {
    let contract = load_generated_contract(root)?;
    if contract.schema != 1 {
        return Err(format!("{id}: expected-systemd schema must be 1"));
    }
    let expected_application = id
        .strip_prefix("boxferry-")
        .and_then(|value| value.strip_suffix("-application"))
        .ok_or_else(|| format!("invalid application contract id `{id}`"))?;
    if contract.application != expected_application {
        return Err(format!(
            "{id}: expectation application `{}`, expected `{expected_application}`",
            contract.application
        ));
    }
    if contract.target_version != version {
        return Err(format!(
            "{id}: generator version `{version}` is unsupported; contract requires exact {}",
            contract.target_version
        ));
    }

    let expected_units = unique_strings(&contract.units, "units")?;
    let actual_units = generated_unit_names(generated)?;
    if actual_units != expected_units {
        return Err(format!(
            "{id}: generated unit inventory differs; actual={actual_units:?}, expected={expected_units:?}"
        ));
    }

    let source_units = known_application_units(id)?
        .iter()
        .map(|unit| unit.path)
        .collect::<BTreeSet<_>>();
    let rule_names = contract
        .unit_rules
        .iter()
        .map(|rule| rule.name.as_str())
        .collect::<Vec<_>>();
    if unique_strings(&rule_names, "unit rule names")? != expected_units {
        return Err(format!("{id}: every generated unit must have exactly one rule"));
    }

    for rule in &contract.unit_rules {
        if !source_units.contains(rule.source.as_str()) {
            return Err(format!(
                "{id}: rule {} names unreviewed source {}",
                rule.name, rule.source
            ));
        }
        let unit = generated_unit(generated, &rule.name)?;
        let source_path = format!("SourcePath=/fixtures/{}", rule.source);
        if unit.matches(&source_path).count() != 1 {
            return Err(format!(
                "{id}: {} must contain exactly one fixture-relative `{source_path}`",
                rule.name
            ));
        }
        verify_exact_values(
            id,
            &rule.name,
            "Requires",
            &rule.requires,
            &dependency_values(id, &rule.name, unit, "Requires")?,
        )?;
        verify_exact_values(
            id,
            &rule.name,
            "After",
            &rule.after,
            &dependency_values(id, &rule.name, unit, "After")?,
        )?;

        let exec_start = podman_exec_start(id, &rule.name, unit)?;
        if !rule.exec.is_empty() {
            let actual = exec_start.split_whitespace().map(str::to_owned).collect::<Vec<_>>();
            verify_exact_values(id, &rule.name, "Podman ExecStart", &rule.exec, &actual)?;
        }
        verify_run_payload(id, &rule.name, exec_start, rule.image.as_deref(), &rule.command)?;
        verify_exact_values(
            id,
            &rule.name,
            "--env",
            &rule.environment,
            &podman_option_values(id, &rule.name, exec_start, &["-e", "--env"], rule.image.as_deref())?,
        )?;
        verify_exact_values(
            id,
            &rule.name,
            "--publish",
            &rule.publish,
            &podman_option_values(id, &rule.name, exec_start, &["-p", "--publish"], rule.image.as_deref())?,
        )?;
        verify_exact_values(
            id,
            &rule.name,
            "--network",
            &rule.network,
            &podman_option_values(id, &rule.name, exec_start, &["--network"], rule.image.as_deref())?,
        )?;
        verify_exact_values(
            id,
            &rule.name,
            "mount",
            &rule.mount,
            &podman_option_values(id, &rule.name, exec_start, &["-v", "--volume"], rule.image.as_deref())?,
        )?;
        let actual_internal = podman_internal(id, &rule.name, exec_start, rule.image.as_deref())?;
        if actual_internal != rule.internal {
            return Err(format!(
                "{id}: {} internal differs; actual={actual_internal:?}, expected={:?}",
                rule.name, rule.internal
            ));
        }
        for fragment in &rule.exec_required {
            let count = exec_start.matches(fragment).count();
            if count != 1 {
                return Err(format!(
                    "{id}: {} ExecStart requires exactly one `{fragment}`, found {count}",
                    rule.name
                ));
            }
        }
        for fragment in &rule.exec_forbidden {
            if exec_start.contains(fragment) {
                return Err(format!("{id}: {} ExecStart contains forbidden `{fragment}`", rule.name));
            }
        }
        for ordered in &rule.ordered {
            verify_ordered_fragments(id, &rule.name, unit, &ordered.fragments)?;
        }
    }

    for specifier in ["%t/containers", "%n", "%N", "systemd-%N"] {
        if !generated.contains(specifier) {
            return Err(format!(
                "{id}: generated application omitted retained systemd specifier `{specifier}`"
            ));
        }
    }
    Ok(())
}

pub(crate) fn verify_supplied_application_directory(id: &str, supplied: &Path) -> Result<PathBuf, String> {
    if !supplied.is_absolute() {
        return Err("QUADLET_LENS_APPLICATION_UNIT_DIR must be absolute".to_owned());
    }
    let supplied_metadata =
        fs::symlink_metadata(supplied).map_err(|error| format!("cannot inspect {}: {error}", supplied.display()))?;
    if supplied_metadata.file_type().is_symlink() {
        return Err(format!(
            "application unit directory may not be a symlink: {}",
            supplied.display()
        ));
    }
    let root = supplied
        .canonicalize()
        .map_err(|error| format!("cannot canonicalize {}: {error}", supplied.display()))?;
    if !root.is_dir() {
        return Err(format!("{} is not a directory", root.display()));
    }
    let root_text = root
        .to_str()
        .ok_or_else(|| "application unit directory must be UTF-8".to_owned())?;
    if root_text.contains(',') || root_text.chars().any(char::is_control) {
        return Err("application unit directory may not contain commas or control characters".to_owned());
    }
    let units = known_application_units(id)?;
    let expected = units.iter().map(|unit| unit.path).collect::<BTreeSet<_>>();
    verify_directory_members(&root, &expected)?;
    for unit in units {
        let actual = sha256_file(&root.join(unit.path))?;
        if actual != unit.sha256 {
            return Err(format!(
                "{id}: supplied {} SHA-256 {actual}, expected {}",
                unit.path, unit.sha256
            ));
        }
    }
    Ok(root)
}

fn load_generated_contract(root: &Path) -> Result<GeneratedApplicationContract, String> {
    let path = root.join("expected-systemd.toml");
    let text = fs::read_to_string(&path).map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    toml::from_str(&text).map_err(|error| format!("invalid {}: {error}", path.display()))
}

fn unique_strings<'a, T>(values: &'a [T], subject: &str) -> Result<BTreeSet<&'a str>, String>
where
    T: AsRef<str>,
{
    let mut unique = BTreeSet::new();
    for value in values {
        let value = value.as_ref();
        if value.is_empty() || !unique.insert(value) {
            return Err(format!("{subject} must contain unique non-empty strings"));
        }
    }
    Ok(unique)
}

fn generated_unit_names(generated: &str) -> Result<BTreeSet<&str>, String> {
    let mut names = BTreeSet::new();
    for line in generated.lines() {
        let Some(name) = line.strip_prefix("---").and_then(|value| value.strip_suffix("---")) else {
            continue;
        };
        if name.is_empty() || !names.insert(name) {
            return Err("generator output contains an empty or duplicate unit marker".to_owned());
        }
    }
    if names.is_empty() {
        return Err("generator output contains no unit markers".to_owned());
    }
    Ok(names)
}

fn generated_unit<'a>(generated: &'a str, name: &str) -> Result<&'a str, String> {
    let marker = format!("---{name}---");
    let (_, remainder) = generated
        .split_once(&marker)
        .ok_or_else(|| format!("generator output omitted `{marker}`"))?;
    Ok(remainder.split("\n---").next().unwrap_or(remainder))
}

fn dependency_values(id: &str, unit_name: &str, unit: &str, directive: &str) -> Result<Vec<String>, String> {
    let prefix = format!("{directive}=");
    let mut in_unit_section = false;
    let mut values = Vec::new();
    for line in unit.lines() {
        if line.starts_with('[') && line.ends_with(']') {
            in_unit_section = line == "[Unit]";
            continue;
        }
        if !in_unit_section {
            continue;
        }
        let Some(value) = line.strip_prefix(&prefix) else {
            continue;
        };
        if value.trim().is_empty() {
            return Err(format!(
                "{id}: {unit_name} contains an empty [Unit] {directive} assignment"
            ));
        }
        values.extend(value.split_whitespace().map(str::to_owned));
    }
    Ok(values)
}

fn podman_exec_start<'a>(id: &str, unit_name: &str, unit: &'a str) -> Result<&'a str, String> {
    let mut commands = unit
        .lines()
        .filter_map(|line| line.strip_prefix("ExecStart=/usr/bin/podman "));
    let command = commands
        .next()
        .ok_or_else(|| format!("{id}: {unit_name} omitted its Podman ExecStart"))?;
    if commands.next().is_some() {
        return Err(format!("{id}: {unit_name} contains more than one Podman ExecStart"));
    }
    Ok(command)
}

fn podman_image_index(id: &str, unit_name: &str, tokens: &[&str], image: &str) -> Result<usize, String> {
    if tokens.first().copied() != Some("run") {
        return Err(format!("{id}: {unit_name} container ExecStart must use `podman run`"));
    }
    let positions = tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| (*token == image).then_some(index))
        .collect::<Vec<_>>();
    let image_index = match positions.as_slice() {
        [index] => *index,
        [] => return Err(format!("{id}: {unit_name} omitted exact container image `{image}`")),
        _ => {
            return Err(format!(
                "{id}: {unit_name} contains the container image `{image}` more than once"
            ));
        }
    };

    let mut index = 1;
    while index < image_index {
        let token = tokens[index];
        if token == "--" {
            if index + 1 != image_index {
                return Err(format!(
                    "{id}: {unit_name} has an unexpected positional argument before its image"
                ));
            }
            break;
        }
        if PODMAN_RUN_FLAGS.contains(&token) {
            index += 1;
            continue;
        }
        if PODMAN_RUN_VALUE_OPTIONS.contains(&token) {
            if index + 1 >= image_index {
                return Err(format!(
                    "{id}: {unit_name} Podman option `{token}` is missing its value before the image"
                ));
            }
            index += 2;
            continue;
        }
        if let Some((option, value)) = token.split_once('=') {
            if PODMAN_RUN_VALUE_OPTIONS.contains(&option) && !value.is_empty() {
                index += 1;
                continue;
            }
        }
        return Err(format!(
            "{id}: {unit_name} has unreviewed Podman argument `{token}` before its image"
        ));
    }
    Ok(image_index)
}

fn verify_run_payload(
    id: &str,
    unit_name: &str,
    command: &str,
    image: Option<&str>,
    expected_command: &[String],
) -> Result<(), String> {
    let Some(image) = image else {
        if !expected_command.is_empty() {
            return Err(format!("{id}: {unit_name} command contract requires an image"));
        }
        return Ok(());
    };
    let tokens = command.split_whitespace().collect::<Vec<_>>();
    let image_index = podman_image_index(id, unit_name, &tokens, image)?;
    let actual_command = tokens[image_index + 1..]
        .iter()
        .map(|value| (*value).to_owned())
        .collect::<Vec<_>>();
    verify_exact_values(id, unit_name, "container command", expected_command, &actual_command)
}

fn podman_option_values(
    id: &str,
    unit_name: &str,
    command: &str,
    options: &[&str],
    image: Option<&str>,
) -> Result<Vec<String>, String> {
    let tokens = command.split_whitespace().collect::<Vec<_>>();
    let boundary = match image {
        Some(image) => podman_image_index(id, unit_name, &tokens, image)?,
        None => tokens.len(),
    };
    let mut values = Vec::new();
    let mut index = 0;
    while index < boundary {
        let token = tokens[index];
        if token == "--" {
            break;
        }
        if options.contains(&token) {
            let value = tokens
                .get(index + 1)
                .filter(|value| **value != "--")
                .ok_or_else(|| format!("{id}: {unit_name} Podman option `{token}` is missing its value"))?;
            values.push((*value).to_owned());
            index += 2;
            continue;
        } else if let Some(value) = options
            .iter()
            .find_map(|option| token.strip_prefix(option).and_then(|suffix| suffix.strip_prefix('=')))
        {
            if value.is_empty() {
                return Err(format!(
                    "{id}: {unit_name} Podman option `{token}` is missing its value"
                ));
            }
            values.push(value.to_owned());
        }
        index += 1;
    }
    Ok(values)
}

fn podman_internal(id: &str, unit_name: &str, command: &str, image: Option<&str>) -> Result<Option<bool>, String> {
    let tokens = command.split_whitespace().collect::<Vec<_>>();
    let boundary = match image {
        Some(image) => podman_image_index(id, unit_name, &tokens, image)?,
        None => tokens.len(),
    };
    let mut values = command
        .split_whitespace()
        .take(boundary)
        .take_while(|token| *token != "--")
        .filter_map(|token| match token {
            "--internal" | "--internal=true" => Some(Ok(true)),
            "--internal=false" => Some(Ok(false)),
            value if value.starts_with("--internal=") => {
                Some(Err(format!("{id}: {unit_name} has invalid internal option `{value}`")))
            }
            _ => None,
        });
    let value = values.next().transpose()?;
    if values.next().is_some() {
        return Err(format!("{id}: {unit_name} contains more than one internal option"));
    }
    Ok(value)
}

fn verify_exact_values(
    id: &str,
    unit_name: &str,
    subject: &str,
    expected: &[String],
    actual: &[String],
) -> Result<(), String> {
    if actual != expected {
        return Err(format!(
            "{id}: {unit_name} {subject} values differ; actual={actual:?}, expected={expected:?}"
        ));
    }
    Ok(())
}

fn verify_ordered_fragments(id: &str, unit_name: &str, unit: &str, fragments: &[String]) -> Result<(), String> {
    if fragments.len() < 2 {
        return Err(format!(
            "{id}: {unit_name} ordered rule must contain at least two fragments"
        ));
    }
    let mut cursor = 0;
    for fragment in fragments {
        let relative = unit[cursor..]
            .find(fragment)
            .ok_or_else(|| format!("{id}: {unit_name} ordered fragment `{fragment}` is missing or out of order"))?;
        cursor += relative + fragment.len();
    }
    Ok(())
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "One mutation table keeps every structured application contract boundary auditable."
)]
fn generated_application_verifier_rejects_semantic_mutations() -> Result<(), String> {
    let id = NEXTCLOUD_ID;
    let root = verify_known_application_fixture(id)?;
    let contract = load_generated_contract(&root)?;
    let mut generated = String::new();
    for rule in &contract.unit_rules {
        generated.push_str("---");
        generated.push_str(&rule.name);
        generated.push_str("---\n[Unit]\nSourcePath=/fixtures/");
        generated.push_str(&rule.source);
        generated.push('\n');
        for value in &rule.requires {
            generated.push_str("Requires=");
            generated.push_str(value);
            generated.push('\n');
        }
        for value in &rule.after {
            generated.push_str("After=");
            generated.push_str(value);
            generated.push('\n');
        }
        generated.push_str("[Service]\nExecStart=/usr/bin/podman ");
        if rule.exec.is_empty() {
            generated.push_str("run");
        } else {
            generated.push_str(&rule.exec.join(" "));
        }
        if rule.exec.is_empty() {
            for fragment in &rule.exec_required {
                if fragment == "--health-cmd" {
                    generated.push_str(" --health-cmd synthetic-health-command");
                } else {
                    generated.push(' ');
                    generated.push_str(fragment);
                }
            }
            for value in &rule.environment {
                generated.push_str(" --env ");
                generated.push_str(value);
            }
            for value in &rule.publish {
                generated.push_str(" --publish ");
                generated.push_str(value);
            }
            for value in &rule.network {
                generated.push_str(" --network ");
                generated.push_str(value);
            }
            for value in &rule.mount {
                generated.push_str(" -v ");
                generated.push_str(value);
            }
            match rule.internal {
                Some(true) => generated.push_str(" --internal"),
                Some(false) => generated.push_str(" --internal=false"),
                None => {}
            }
        }
        if let Some(image) = &rule.image {
            generated.push(' ');
            generated.push_str(image);
            for argument in &rule.command {
                generated.push(' ');
                generated.push_str(argument);
            }
        }
        generated.push('\n');
    }
    generated.push_str("%t/containers\n%n\n%N\nsystemd-%N\n");
    verify_generated_application(id, &root, &contract.target_version, &generated)?;

    let equals_forms = generated
        .replacen(" --env POSTGRES_HOST=db", " --env=POSTGRES_HOST=db", 1)
        .replacen(
            " --publish 127.0.0.1:18443:8443/tcp",
            " --publish=127.0.0.1:18443:8443/tcp",
            1,
        )
        .replacen(" --network systemd-backend", " --network=systemd-backend", 1)
        .replacen(
            " -v systemd-nextcloud-data:/var/www/html",
            " --volume=systemd-nextcloud-data:/var/www/html",
            1,
        );
    verify_generated_application(id, &root, &contract.target_version, &equals_forms)?;
    let short_forms = generated
        .replacen(" --env POSTGRES_HOST=db", " -e POSTGRES_HOST=db", 1)
        .replacen(" --publish 127.0.0.1:18443:8443/tcp", " -p 127.0.0.1:18443:8443/tcp", 1);
    verify_generated_application(id, &root, &contract.target_version, &short_forms)?;

    let requires_in_service = generated.replacen("Requires=init.service\n", "", 1).replacen(
        "[Service]\nExecStart=/usr/bin/podman run",
        "[Service]\nRequires=init.service\nExecStart=/usr/bin/podman run",
        1,
    );
    let after_in_service = generated.replacen("After=init.service\n", "", 1).replacen(
        "[Service]\nExecStart=/usr/bin/podman run",
        "[Service]\nAfter=init.service\nExecStart=/usr/bin/podman run",
        1,
    );
    let nginx_image = "docker.io/library/nginx@sha256:42a516af16b852e33b7682d5ef8acbd5d13fe08fecadc7ed98605ba5e3b26ab8";
    let alpine_image =
        "docker.io/library/alpine@sha256:14358309a308569c32bdc37e2e0e9694be33a9d99e68afb0f5ff33cc1f695dce";
    let publish_after_image = generated
        .replacen(" --publish 127.0.0.1:18443:8443/tcp", "", 1)
        .replacen(
            nginx_image,
            &format!("{nginx_image} --publish 127.0.0.1:18443:8443/tcp"),
            1,
        );
    for (mutation_index, changed) in [
        generated.replacen("Requires=init.service", "Requires=changed.service", 1),
        requires_in_service,
        after_in_service,
        generated.replacen("Requires=init.service\n", "Requires=init.service\nRequires=\n", 1),
        generated.replacen("After=init.service\n", "After=init.service\nAfter=\n", 1),
        generated.replacen(
            "Requires=init.service",
            "Requires=unexpected.service\nRequires=init.service",
            1,
        ),
        generated.replacen("After=init.service", "After=unexpected.service\nAfter=init.service", 1),
        generated.replacen(
            " --env POSTGRES_HOST=db",
            " --env UNEXPECTED=value --env POSTGRES_HOST=db",
            1,
        ),
        generated.replacen(
            "--publish 127.0.0.1:18443:8443/tcp",
            "--publish 127.0.0.1:19443:8443/tcp",
            1,
        ),
        generated.replacen(
            "--publish 127.0.0.1:18443:8443/tcp",
            "-- --publish 127.0.0.1:18443:8443/tcp",
            1,
        ),
        publish_after_image,
        generated.replacen(
            " --publish 127.0.0.1:18443:8443/tcp",
            " -p unexpected:9443:9443 --publish 127.0.0.1:18443:8443/tcp",
            1,
        ),
        generated.replacen(
            " --publish 127.0.0.1:18443:8443/tcp",
            " --publish-all --publish 127.0.0.1:18443:8443/tcp",
            1,
        ),
        generated.replacen(
            &format!(" --network systemd-edge {alpine_image}"),
            &format!(" --network systemd-edge --publish {alpine_image}"),
            1,
        ),
        generated.replacen(
            " -v systemd-nextcloud-data:/var/www/html",
            " -v unexpected:/tmp -v systemd-nextcloud-data:/var/www/html",
            1,
        ),
        generated.replacen(
            " --network systemd-backend",
            " --network unexpected --network systemd-backend",
            1,
        ),
        generated.replacen("--internal systemd-backend", "--internal --internal systemd-backend", 1),
        generated.replacen(
            "network create --ignore --internal systemd-backend",
            "network create --ignore --internal wrong-name # systemd-backend",
            1,
        ),
        generated.replacen(
            "network create --ignore --internal systemd-backend",
            "network create --ignore --internal=false systemd-backend",
            1,
        ),
        generated.replacen(nginx_image, "docker.io/library/nginx:unexpected", 1),
        generated.replacen(nginx_image, &format!("wrong-image {nginx_image}"), 1),
        generated.replacen(" /cron.sh\n", " /cron.sh unexpected\n", 1),
        generated.replacen("---app.service---\n", "", 1),
        format!("{generated}---unexpected.service---\n"),
        generated.replacen(
            "--network systemd-backend --network systemd-edge",
            "--network systemd-edge --network systemd-backend",
            1,
        ),
        generated.replacen(
            "SourcePath=/fixtures/app.container",
            "SourcePath=/elsewhere/app.container",
            1,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        assert_ne!(
            changed, generated,
            "application generator mutation {mutation_index} was a no-op"
        );
        assert!(
            verify_generated_application(id, &root, &contract.target_version, &changed).is_err(),
            "application generator mutation {mutation_index} was accepted"
        );
    }
    Ok(())
}

fn verify_directory_members(root: &Path, expected: &BTreeSet<&str>) -> Result<(), String> {
    let mut actual = BTreeSet::new();
    for entry in fs::read_dir(root).map_err(|error| format!("{}: {error}", root.display()))? {
        let entry = entry.map_err(|error| format!("{}: {error}", root.display()))?;
        let file_type = entry
            .file_type()
            .map_err(|error| format!("{}: {error}", entry.path().display()))?;
        if !file_type.is_file() || file_type.is_symlink() {
            return Err(format!(
                "{}: application fixtures may contain only regular files",
                entry.path().display()
            ));
        }
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| format!("{}: non-UTF-8 filename", entry.path().display()))?;
        actual.insert(name);
    }
    let expected: BTreeSet<_> = expected.iter().map(|value| (*value).to_owned()).collect();
    if actual != expected {
        return Err(format!(
            "{}: directory members differ from the reviewed contract",
            root.display()
        ));
    }
    Ok(())
}

fn require_table<'a>(table: &'a Table, field: &str, subject: &str) -> Result<&'a Table, String> {
    table
        .get(field)
        .and_then(Value::as_table)
        .ok_or_else(|| format!("{subject}: {field} must be a table"))
}

fn required_string<'a>(table: &'a Table, field: &str, subject: &str) -> Result<&'a str, String> {
    table
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("{subject}: {field} must be a non-empty string"))
}

fn require_string(table: &Table, field: &str, expected: &str, subject: &str) -> Result<(), String> {
    let actual = required_string(table, field, subject)?;
    if actual != expected {
        return Err(format!("{subject}: {field} is `{actual}`, expected `{expected}`"));
    }
    Ok(())
}

fn require_integer(table: &Table, field: &str, expected: i64, subject: &str) -> Result<(), String> {
    let actual = table.get(field).and_then(Value::as_integer);
    if actual != Some(expected) {
        return Err(format!("{subject}: {field} must be {expected}"));
    }
    Ok(())
}

fn string_set<'a>(table: &'a Table, field: &str, subject: &str) -> Result<BTreeSet<&'a str>, String> {
    let values = table
        .get(field)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{subject}: {field} must be an array"))?;
    let mut result = BTreeSet::new();
    for value in values {
        let value = value
            .as_str()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| format!("{subject}: {field} entries must be non-empty strings"))?;
        if !result.insert(value) {
            return Err(format!("{subject}: duplicate {field} entry `{value}`"));
        }
    }
    Ok(result)
}

fn is_safe_relative(path: &str) -> bool {
    !path.is_empty()
        && Path::new(path)
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
    Ok(sha256(&bytes))
}

// Kept dependency-free so this test-only provenance check works at the MSRV and on macOS.
#[allow(
    clippy::format_collect,
    clippy::many_single_char_names,
    clippy::too_many_lines,
    clippy::useless_conversion
)]
fn sha256(input: &[u8]) -> String {
    const INITIAL: [u32; 8] = [
        0x6a09_e667,
        0xbb67_ae85,
        0x3c6e_f372,
        0xa54f_f53a,
        0x510e_527f,
        0x9b05_688c,
        0x1f83_d9ab,
        0x5be0_cd19,
    ];
    const ROUND: [u32; 64] = [
        0x428a_2f98,
        0x7137_4491,
        0xb5c0_fbcf,
        0xe9b5_dba5,
        0x3956_c25b,
        0x59f1_11f1,
        0x923f_82a4,
        0xab1c_5ed5,
        0xd807_aa98,
        0x1283_5b01,
        0x2431_85be,
        0x550c_7dc3,
        0x72be_5d74,
        0x80de_b1fe,
        0x9bdc_06a7,
        0xc19b_f174,
        0xe49b_69c1,
        0xefbe_4786,
        0x0fc1_9dc6,
        0x240c_a1cc,
        0x2de9_2c6f,
        0x4a74_84aa,
        0x5cb0_a9dc,
        0x76f9_88da,
        0x983e_5152,
        0xa831_c66d,
        0xb003_27c8,
        0xbf59_7fc7,
        0xc6e0_0bf3,
        0xd5a7_9147,
        0x06ca_6351,
        0x1429_2967,
        0x27b7_0a85,
        0x2e1b_2138,
        0x4d2c_6dfc,
        0x5338_0d13,
        0x650a_7354,
        0x766a_0abb,
        0x81c2_c92e,
        0x9272_2c85,
        0xa2bf_e8a1,
        0xa81a_664b,
        0xc24b_8b70,
        0xc76c_51a3,
        0xd192_e819,
        0xd699_0624,
        0xf40e_3585,
        0x106a_a070,
        0x19a4_c116,
        0x1e37_6c08,
        0x2748_774c,
        0x34b0_bcb5,
        0x391c_0cb3,
        0x4ed8_aa4a,
        0x5b9c_ca4f,
        0x682e_6ff3,
        0x748f_82ee,
        0x78a5_636f,
        0x84c8_7814,
        0x8cc7_0208,
        0x90be_fffa,
        0xa450_6ceb,
        0xbef9_a3f7,
        0xc671_78f2,
    ];

    let bit_length = (input.len() as u64).wrapping_mul(8);
    let mut padded = input.to_vec();
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_length.to_be_bytes());

    let mut state = INITIAL;
    for block in padded.chunks_exact(64) {
        let mut words = [0_u32; 64];
        for (index, bytes) in block.chunks_exact(4).enumerate() {
            words[index] = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        }
        for index in 16..64 {
            let first =
                words[index - 15].rotate_right(7) ^ words[index - 15].rotate_right(18) ^ (words[index - 15] >> 3);
            let second =
                words[index - 2].rotate_right(17) ^ words[index - 2].rotate_right(19) ^ (words[index - 2] >> 10);
            words[index] = words[index - 16]
                .wrapping_add(first)
                .wrapping_add(words[index - 7])
                .wrapping_add(second);
        }

        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = state;
        for index in 0..64 {
            let sum_one = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let choose = (e & f) ^ (!e & g);
            let temporary_one = h
                .wrapping_add(sum_one)
                .wrapping_add(choose)
                .wrapping_add(ROUND[index])
                .wrapping_add(words[index]);
            let sum_zero = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let majority = (a & b) ^ (a & c) ^ (b & c);
            let temporary_two = sum_zero.wrapping_add(majority);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temporary_one);
            d = c;
            c = b;
            b = a;
            a = temporary_one.wrapping_add(temporary_two);
        }
        for (value, addition) in state.iter_mut().zip([a, b, c, d, e, f, g, h].into_iter()) {
            *value = value.wrapping_add(addition);
        }
    }

    state.iter().map(|value| format!("{value:08x}")).collect()
}

#[test]
fn sha256_matches_the_published_short_test_vector() {
    assert_eq!(
        sha256(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}
