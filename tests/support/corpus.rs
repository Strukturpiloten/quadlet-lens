//! Validation for the pinned real-world Quadlet catalogue.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path};

use toml::{Table, Value};

const ROOT_FIELDS: &[&str] = &["projects", "reviewed", "schema"];
const PROJECT_FIELDS: &[&str] = &[
    "evidence_class",
    "files",
    "goals",
    "id",
    "license",
    "repository",
    "revision",
    "tier",
];
const FILE_FIELDS: &[&str] = &["blob_sha", "path", "required_keys", "required_sections"];
const EVIDENCE_CLASSES: &[&str] = &[
    "community-deployment",
    "distribution-project",
    "organization-example",
    "platform-project",
    "upstream-project",
    "vendor-example",
    "vendor-project",
];
const LICENSES: &[&str] = &["Apache-2.0", "BSD-3-Clause", "GPL-2.0-only", "GPL-3.0-only"];
const UNIT_EXTENSIONS: &[&str] = &["container", "network", "pod", "volume"];

#[derive(Default)]
struct ValidationState {
    ids: BTreeSet<String>,
    sources: BTreeSet<(String, String)>,
    tiers: BTreeSet<String>,
    extensions: BTreeSet<String>,
    previous_order: Option<(u8, String)>,
    errors: Vec<String>,
}

pub(crate) fn validate_real_world_quadlet_catalog(repository_root: &Path) -> Result<(), String> {
    let path = repository_root.join("fixtures/real-world/corpus.toml");
    let text = fs::read_to_string(&path).map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let table = text
        .parse::<Table>()
        .map_err(|error| format!("{}: invalid TOML: {error}", path.display()))?;
    let name = path.display().to_string();
    let mut state = ValidationState::default();

    validate_root(&name, &table, &mut state.errors);
    let Some(projects) = table.get("projects").and_then(Value::as_array) else {
        state
            .errors
            .push(format!("{name}: `projects` must be a non-empty array of tables"));
        return finish(&state.errors);
    };
    if projects.is_empty() {
        state.errors.push(format!("{name}: `projects` must not be empty"));
    }

    for (index, value) in projects.iter().enumerate() {
        let Some(project) = value.as_table() else {
            state.errors.push(format!("{name}: projects[{index}] must be a table"));
            continue;
        };
        validate_project(&name, index, project, &mut state);
    }

    for tier in ["baseline", "application", "stress"] {
        if !state.tiers.contains(tier) {
            state
                .errors
                .push(format!("{name}: corpus must contain the `{tier}` tier"));
        }
    }
    for extension in UNIT_EXTENSIONS {
        if !state.extensions.contains(*extension) {
            state
                .errors
                .push(format!("{name}: corpus must contain a `.{extension}` unit"));
        }
    }
    finish(&state.errors)
}

fn validate_root(name: &str, table: &Table, errors: &mut Vec<String>) {
    reject_unknown_fields(name, table, ROOT_FIELDS, errors);
    if table.get("schema").and_then(Value::as_integer) != Some(1) {
        errors.push(format!("{name}: `schema` must be integer 1"));
    }
    match string_field(table, "reviewed") {
        Some(reviewed) if valid_date(reviewed) => {}
        _ => errors.push(format!("{name}: `reviewed` must use YYYY-MM-DD")),
    }
}

fn validate_project(name: &str, index: usize, project: &Table, state: &mut ValidationState) {
    let subject = format!("{name}: projects[{index}]");
    reject_unknown_fields(&subject, project, PROJECT_FIELDS, &mut state.errors);

    let id = required_string(&subject, project, "id", &mut state.errors).map(ToOwned::to_owned);
    if let Some(id) = &id {
        if !valid_slug(id) {
            state
                .errors
                .push(format!("{subject}: `id` must be a lowercase ASCII slug"));
        }
        if !state.ids.insert(id.clone()) {
            state.errors.push(format!("{subject}: duplicate id `{id}`"));
        }
    }

    let tier = required_string(&subject, project, "tier", &mut state.errors).map(ToOwned::to_owned);
    let tier_rank = match tier.as_deref() {
        Some("baseline") => Some(0),
        Some("application") => Some(1),
        Some("stress") => Some(2),
        Some(value) => {
            state.errors.push(format!("{subject}: unsupported tier `{value}`"));
            None
        }
        None => None,
    };
    if let Some(tier) = &tier {
        state.tiers.insert(tier.clone());
    }
    if let (Some(rank), Some(id)) = (tier_rank, id) {
        let order = (rank, id);
        if state.previous_order.as_ref().is_some_and(|previous| previous >= &order) {
            state
                .errors
                .push(format!("{subject}: projects must be ordered by tier and id"));
        }
        state.previous_order = Some(order);
    }

    match required_string(&subject, project, "evidence_class", &mut state.errors) {
        Some(class) if EVIDENCE_CLASSES.contains(&class) => {}
        Some(class) => state
            .errors
            .push(format!("{subject}: unsupported evidence class `{class}`")),
        None => {}
    }
    match required_string(&subject, project, "license", &mut state.errors) {
        Some(license) if LICENSES.contains(&license) => {}
        Some(license) => state
            .errors
            .push(format!("{subject}: unreviewed SPDX license `{license}`")),
        None => {}
    }
    let repository = required_string(&subject, project, "repository", &mut state.errors).map(ToOwned::to_owned);
    if repository.as_deref().is_some_and(|value| !valid_repository(value)) {
        state
            .errors
            .push(format!("{subject}: `repository` must use owner/name form"));
    }
    if required_string(&subject, project, "revision", &mut state.errors).is_some_and(|value| !valid_git_sha(value)) {
        state
            .errors
            .push(format!("{subject}: `revision` must be a full lowercase Git SHA"));
    }
    validate_string_array(&subject, project, "goals", false, &mut state.errors);
    validate_files(&subject, project, repository.as_deref(), state);
}

fn validate_files(subject: &str, project: &Table, repository: Option<&str>, state: &mut ValidationState) {
    let Some(files) = project.get("files").and_then(Value::as_array) else {
        state
            .errors
            .push(format!("{subject}: `files` must be a non-empty array of tables"));
        return;
    };
    if files.is_empty() {
        state.errors.push(format!("{subject}: `files` must not be empty"));
    }

    let mut basenames = BTreeSet::new();
    for (index, value) in files.iter().enumerate() {
        let file_subject = format!("{subject}.files[{index}]");
        let Some(file) = value.as_table() else {
            state.errors.push(format!("{file_subject}: must be a table"));
            continue;
        };
        reject_unknown_fields(&file_subject, file, FILE_FIELDS, &mut state.errors);
        let path = required_string(&file_subject, file, "path", &mut state.errors);
        if let Some(path) = path {
            if !safe_relative_path(path) {
                state.errors.push(format!(
                    "{file_subject}: `path` must be a safe relative repository path"
                ));
            }
            let extension = Path::new(path).extension().and_then(|value| value.to_str());
            match extension {
                Some(extension) if UNIT_EXTENSIONS.contains(&extension) => {
                    state.extensions.insert(extension.to_owned());
                }
                _ => state
                    .errors
                    .push(format!("{file_subject}: `path` must name a supported Quadlet unit")),
            }
            if let Some(basename) = Path::new(path).file_name().and_then(|value| value.to_str()) {
                if !basenames.insert(basename.to_owned()) {
                    state
                        .errors
                        .push(format!("{file_subject}: duplicate unit-file basename `{basename}`"));
                }
            }
            if let Some(repository) = repository {
                let source = (repository.to_owned(), path.to_owned());
                if !state.sources.insert(source) {
                    state.errors.push(format!(
                        "{file_subject}: duplicate repository path `{repository}/{path}`"
                    ));
                }
            }
        }
        if required_string(&file_subject, file, "blob_sha", &mut state.errors)
            .is_some_and(|value| !valid_git_sha(value))
        {
            state
                .errors
                .push(format!("{file_subject}: `blob_sha` must be a full lowercase Git SHA"));
        }
        validate_string_array(&file_subject, file, "required_sections", false, &mut state.errors);
        validate_string_array(&file_subject, file, "required_keys", true, &mut state.errors);
    }
}

fn reject_unknown_fields(subject: &str, table: &Table, fields: &[&str], errors: &mut Vec<String>) {
    for key in table.keys() {
        if !fields.contains(&key.as_str()) {
            errors.push(format!("{subject}: unknown field `{key}`"));
        }
    }
}

fn validate_string_array(subject: &str, table: &Table, field: &str, allow_empty: bool, errors: &mut Vec<String>) {
    let Some(values) = table.get(field).and_then(Value::as_array) else {
        errors.push(format!("{subject}: `{field}` must be a string array"));
        return;
    };
    if values.is_empty() && !allow_empty {
        errors.push(format!("{subject}: `{field}` must not be empty"));
    }
    let mut unique = BTreeSet::new();
    for value in values {
        let Some(value) = value.as_str().filter(|value| !value.is_empty()) else {
            errors.push(format!("{subject}: `{field}` values must be non-empty strings"));
            continue;
        };
        if !unique.insert(value) {
            errors.push(format!("{subject}: duplicate `{field}` value `{value}`"));
        }
    }
}

fn required_string<'a>(subject: &str, table: &'a Table, field: &str, errors: &mut Vec<String>) -> Option<&'a str> {
    let value = string_field(table, field);
    if value.is_none() {
        errors.push(format!("{subject}: `{field}` must be a non-empty string"));
    }
    value
}

fn string_field<'a>(table: &'a Table, field: &str) -> Option<&'a str> {
    table
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
}

fn valid_slug(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with('-')
        && !value.ends_with('-')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn valid_repository(value: &str) -> bool {
    let mut components = value.split('/');
    let valid_component = |component: &str| {
        !component.is_empty()
            && component
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    };
    matches!(
        (components.next(), components.next(), components.next()),
        (Some(owner), Some(repository), None) if valid_component(owner) && valid_component(repository)
    )
}

fn safe_relative_path(value: &str) -> bool {
    let path = Path::new(value);
    !value.is_empty()
        && !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn valid_git_sha(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

fn valid_date(value: &str) -> bool {
    value.len() == 10
        && value.bytes().enumerate().all(|(index, byte)| {
            if matches!(index, 4 | 7) {
                byte == b'-'
            } else {
                byte.is_ascii_digit()
            }
        })
}

fn finish(errors: &[String]) -> Result<(), String> {
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}
