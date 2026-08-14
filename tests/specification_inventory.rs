//! Offline policy checks for the versioned upstream Quadlet-manual inventory.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use quadlet_lens::model::{EntryKind, QuadletDocument, QuadletUnitType, TypedEntry};
use quadlet_lens::source::SourceId;
use toml::{Table, Value};

const INVENTORY: &str = include_str!("../fixtures/specification-drift/quadlet-manual-current.toml");
const PINNED_MANUAL_EVIDENCE: &str = "fixtures/specification-drift/podman-systemd.unit.5-v6.0.2.md.gz.b64";
const PINNED_MANUAL_LICENSE: &str = "fixtures/specification-drift/podman-v6.0.2-LICENSE";
const SECTION_ORDER: &[&str] = &[
    "Container",
    "Pod",
    "Network",
    "Volume",
    "Build",
    "Image",
    "Kube",
    "Artifact",
    "Quadlet",
];
const EXPECTED_COUNTS: &[(&str, usize)] = &[
    ("Container", 89),
    ("Pod", 25),
    ("Network", 18),
    ("Volume", 16),
    ("Build", 28),
    ("Image", 18),
    ("Kube", 14),
    ("Artifact", 13),
    ("Quadlet", 1),
];

#[derive(Debug)]
struct InventoryRow {
    section: String,
    key: String,
    classification: String,
}

#[test]
fn inventory_is_complete_ordered_and_matches_the_public_native_parser() -> Result<(), String> {
    let rows = validate_inventory(INVENTORY)?;
    assert_eq!(rows.len(), 222);

    let mut counts = BTreeMap::new();
    for row in rows {
        *counts.entry(row.section.clone()).or_insert(0_usize) += 1;
        let kind = parsed_kind(&row.section, &row.key)?;
        match row.classification.as_str() {
            "typed" => assert!(
                !matches!(kind, EntryKind::Unknown | EntryKind::GenericSystemd),
                "{} {} must be classified by the public parser",
                row.section,
                row.key
            ),
            "preserved-only" | "intentionally-unsupported" => assert!(
                matches!(kind, EntryKind::Unknown),
                "{} {} must remain untyped",
                row.section,
                row.key
            ),
            other => return Err(format!("unexpected validated classification `{other}`")),
        }
    }

    assert_eq!(counts.len(), EXPECTED_COUNTS.len());
    for (section, expected_count) in EXPECTED_COUNTS {
        assert_eq!(
            counts.get(*section),
            Some(expected_count),
            "unexpected count for {section}"
        );
    }
    Ok(())
}

#[test]
fn non_typed_inventory_rows_require_a_rationale_and_remain_losslessly_preserved() -> Result<(), String> {
    let header = INVENTORY
        .split_once("keys = [")
        .ok_or_else(|| "inventory has no keys field".to_owned())?
        .0;
    let non_typed_inventory = format!(
        "{header}keys = [[\"Container\", \"FutureManualKey\", \"preserved-only\", \"awaiting a typed contract\"]]\n"
    );
    let rows = validate_inventory(&non_typed_inventory)?;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].classification, "preserved-only");

    let source = "[Container]\nFutureManualKey=opaque value\n";
    let parsed = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(91_001), source)
        .map_err(|error| error.to_string())?;
    let entry = parsed
        .document()
        .entries()
        .next()
        .ok_or_else(|| "test document has no entry".to_owned())?;
    assert!(matches!(entry.kind(), EntryKind::Unknown));
    assert_eq!(entry.key().text(), "FutureManualKey");
    assert_eq!(parsed.syntax().document().render_preserved(), source);
    let missing_rationale = non_typed_inventory.replace(", \"awaiting a typed contract\"", "");
    let missing_rationale_error = validate_inventory(&missing_rationale)
        .err()
        .ok_or_else(|| "non-typed rows must require a rationale".to_owned())?;
    assert!(missing_rationale_error.contains("invalid rationale shape"));
    Ok(())
}

#[test]
fn inventory_rejects_schema_spelling_order_and_classification_errors() -> Result<(), String> {
    for (name, replacement, expected) in [
        ("unknown root field", "schema = 1", "unknown root field `unexpected`"),
        (
            "duplicate row",
            "[\"Container\", \"AddCapability\", \"typed\"],",
            "duplicate inventory row `Container.AddCapability`",
        ),
        (
            "bad classification",
            "[\"Container\", \"AddCapability\", \"typed\"],",
            "unsupported classification `not-a-classification`",
        ),
        (
            "bad spelling",
            "[\"Container\", \"AddCapability\", \"typed\"],",
            "is marked typed but the public parser retains it as unknown",
        ),
        (
            "bad order",
            "[\"Container\", \"AddCapability\", \"typed\"],\n  [\"Container\", \"AddDevice\", \"typed\"],",
            "inventory rows must be ordered by section and key",
        ),
        (
            "bad section",
            "[\"Container\", \"AddCapability\", \"typed\"],",
            "unsupported Quadlet section `Systemd`",
        ),
    ] {
        let invalid = match name {
            "unknown root field" => INVENTORY.replacen(replacement, "schema = 1\nunexpected = true", 1),
            "duplicate row" => INVENTORY.replacen(replacement, &format!("{replacement}\n  {replacement}"), 1),
            "bad classification" => INVENTORY.replacen(
                replacement,
                "[\"Container\", \"AddCapability\", \"not-a-classification\"],",
                1,
            ),
            "bad spelling" => INVENTORY.replacen(replacement, "[\"Container\", \"AddCapabilty\", \"typed\"],", 1),
            "bad order" => INVENTORY.replacen(
                replacement,
                "[\"Container\", \"AddDevice\", \"typed\"],\n  [\"Container\", \"AddCapability\", \"typed\"],",
                1,
            ),
            "bad section" => INVENTORY.replacen(replacement, "[\"Systemd\", \"AddCapability\", \"typed\"],", 1),
            _ => unreachable!(),
        };
        let error = validate_inventory(&invalid)
            .err()
            .ok_or_else(|| format!("{name}: accepted an invalid inventory"))?;
        assert!(error.contains(expected), "{name}: {error}");
    }
    Ok(())
}

#[test]
fn extractor_reads_the_offline_representative_manual() -> Result<(), String> {
    let output = Command::new("bash")
        .arg(repository_root().join("scripts/extract-quadlet-manual-keys.sh"))
        .arg(repository_root().join("fixtures/specification-drift/podman-systemd.unit.5.sample.md"))
        .output()
        .map_err(|error| format!("failed to run manual-key extractor: {error}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        include_str!("../fixtures/specification-drift/podman-systemd.unit.5.sample.keys")
    );
    Ok(())
}

#[test]
fn pinned_aggregate_manual_verifies_digest_and_exact_inventory_rows_offline() -> Result<(), String> {
    let manual = decode_pinned_manual()?;
    let inventory = INVENTORY
        .parse::<Table>()
        .map_err(|error| format!("invalid inventory TOML: {error}"))?;
    assert_eq!(
        required_string(&inventory, "source_sha256")?,
        sha256sum(&manual)?,
        "pinned source evidence must verify the inventory digest"
    );

    let manual_path = temporary_manual_path();
    fs::write(&manual_path, manual).map_err(|error| format!("failed to write {}: {error}", manual_path.display()))?;
    let extraction = Command::new("bash")
        .arg(repository_root().join("scripts/extract-quadlet-manual-keys.sh"))
        .arg(&manual_path)
        .output()
        .map_err(|error| format!("failed to extract pinned manual keys: {error}"))?;
    let _ = fs::remove_file(&manual_path);
    if !extraction.status.success() {
        return Err(String::from_utf8_lossy(&extraction.stderr).into_owned());
    }

    let mut extracted_rows = String::from_utf8_lossy(&extraction.stdout)
        .lines()
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    extracted_rows.sort_unstable();
    let mut inventory_rows = validate_inventory(INVENTORY)?
        .into_iter()
        .map(|row| format!("{}\t{}", row.section, row.key))
        .collect::<Vec<_>>();
    inventory_rows.sort_unstable();
    assert_eq!(extracted_rows, inventory_rows);
    Ok(())
}

#[test]
fn pinned_manual_evidence_includes_its_upstream_apache_license() -> Result<(), String> {
    let license_path = repository_root().join(PINNED_MANUAL_LICENSE);
    let license = fs::read_to_string(&license_path)
        .map_err(|error| format!("failed to read {}: {error}", license_path.display()))?;
    assert!(
        license.starts_with(
            "                                 Apache License\n                           Version 2.0, January 2004\n"
        ),
        "pinned manual evidence must retain its exact upstream Apache-2.0 license"
    );
    assert!(license.contains("   END OF TERMS AND CONDITIONS\n"));
    Ok(())
}

#[test]
fn historical_parser_keys_are_excluded_from_the_current_manual_inventory() -> Result<(), String> {
    let current_manual_keys = validate_inventory(INVENTORY)?
        .into_iter()
        .map(|row| (row.section, row.key))
        .collect::<BTreeSet<_>>();
    for (section, key) in [
        ("Container", "ImageVolume"),
        ("Kube", "LogOpt"),
        ("Kube", "RemapGid"),
        ("Kube", "RemapUid"),
        ("Kube", "RemapUidSize"),
        ("Kube", "RemapUsers"),
    ] {
        assert!(
            !current_manual_keys.contains(&(section.to_owned(), key.to_owned())),
            "{section}.{key} must stay outside the current-manual inventory"
        );
        assert!(
            !matches!(
                parsed_kind(section, key)?,
                EntryKind::Unknown | EntryKind::GenericSystemd
            ),
            "{section}.{key} must remain parser-recognized"
        );
    }
    Ok(())
}

#[test]
fn drift_workflow_is_scheduled_manual_and_report_only() -> Result<(), String> {
    let workflow = fs::read_to_string(repository_root().join(".github/workflows/specification-drift.yml"))
        .map_err(|error| format!("failed to read specification-drift workflow: {error}"))?;
    for required in [
        "schedule:",
        "workflow_dispatch:",
        "permissions:",
        "contents: read",
        "curl --fail --location",
        "scripts/check-specification-drift.sh",
        "raw.githubusercontent.com/podman-container-tools/podman/main",
    ] {
        assert!(workflow.contains(required), "workflow is missing `{required}`");
    }
    for forbidden in [
        "pull_request:",
        "push:",
        "issues: write",
        "gh issue",
        "git commit",
        "git push",
    ] {
        assert!(!workflow.contains(forbidden), "workflow must not contain `{forbidden}`");
    }
    Ok(())
}

#[allow(clippy::too_many_lines)] // The strict inventory schema is deliberately maintained in one exhaustive validator.
fn validate_inventory(text: &str) -> Result<Vec<InventoryRow>, String> {
    let table = text
        .parse::<Table>()
        .map_err(|error| format!("invalid inventory TOML: {error}"))?;
    let expected_fields = [
        "schema",
        "manual_version",
        "source_url",
        "retrieved_on",
        "source_sha256",
        "source_license",
        "keys",
    ];
    for field in table.keys() {
        if !expected_fields.contains(&field.as_str()) {
            return Err(format!("unknown root field `{field}`"));
        }
    }
    if table.get("schema").and_then(Value::as_integer) != Some(1) {
        return Err("`schema` must be integer 1".to_owned());
    }
    let version = required_string(&table, "manual_version")?;
    if !valid_version(version) {
        return Err("`manual_version` must be a dotted numeric version".to_owned());
    }
    let source_url = required_string(&table, "source_url")?;
    let expected_url = format!(
        "https://raw.githubusercontent.com/podman-container-tools/podman/v{version}/docs/source/markdown/podman-systemd.unit.5.md"
    );
    if source_url != expected_url {
        return Err("`source_url` must pin the official aggregate manual for `manual_version`".to_owned());
    }
    if !valid_date(required_string(&table, "retrieved_on")?) {
        return Err("`retrieved_on` must use YYYY-MM-DD".to_owned());
    }
    if !valid_sha256(required_string(&table, "source_sha256")?) {
        return Err("`source_sha256` must be a lowercase SHA-256 digest".to_owned());
    }
    if required_string(&table, "source_license")? != "Apache-2.0" {
        return Err("`source_license` must be `Apache-2.0`".to_owned());
    }

    let keys = table
        .get("keys")
        .and_then(Value::as_array)
        .ok_or_else(|| "`keys` must be an array".to_owned())?;
    let mut rows = Vec::with_capacity(keys.len());
    let mut seen = BTreeSet::new();
    let mut previous: Option<(usize, &str)> = None;
    for (index, row) in keys.iter().enumerate() {
        let row = row
            .as_array()
            .ok_or_else(|| format!("keys[{index}] must be an array"))?;
        if !(3..=4).contains(&row.len()) {
            return Err(format!("keys[{index}] must have three or four strings"));
        }
        let values = row
            .iter()
            .map(Value::as_str)
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| format!("keys[{index}] values must be strings"))?;
        let (section, key, classification) = (values[0], values[1], values[2]);
        let section_rank = SECTION_ORDER
            .iter()
            .position(|candidate| *candidate == section)
            .ok_or_else(|| format!("unsupported Quadlet section `{section}`"))?;
        if !valid_key(key) {
            return Err(format!("keys[{index}] has invalid key spelling `{key}`"));
        }
        if !matches!(classification, "typed" | "preserved-only" | "intentionally-unsupported") {
            return Err(format!("unsupported classification `{classification}`"));
        }
        if (classification == "typed" && row.len() != 3) || (classification != "typed" && row.len() != 4) {
            return Err(format!("keys[{index}] has an invalid rationale shape"));
        }
        if row.len() == 4 && values[3].trim().is_empty() {
            return Err(format!("keys[{index}] requires a non-empty rationale"));
        }
        if !seen.insert((section, key)) {
            return Err(format!("duplicate inventory row `{section}.{key}`"));
        }
        if previous.is_some_and(|(previous_rank, previous_key)| (section_rank, key) <= (previous_rank, previous_key)) {
            return Err("inventory rows must be ordered by section and key".to_owned());
        }
        let kind = parsed_kind(section, key)?;
        if classification == "typed" && matches!(kind, EntryKind::Unknown | EntryKind::GenericSystemd) {
            return Err(format!(
                "`{section}.{key}` is marked typed but the public parser retains it as unknown"
            ));
        }
        if classification != "typed" && !matches!(kind, EntryKind::Unknown) {
            return Err(format!(
                "`{section}.{key}` is non-typed but the public parser recognizes it"
            ));
        }
        previous = Some((section_rank, key));
        rows.push(InventoryRow {
            section: section.to_owned(),
            key: key.to_owned(),
            classification: classification.to_owned(),
        });
    }
    if rows.is_empty() {
        return Err("`keys` must not be empty".to_owned());
    }
    Ok(rows)
}

fn parsed_kind(section: &str, key: &str) -> Result<EntryKind, String> {
    let unit_type = match section {
        "Container" | "Quadlet" => QuadletUnitType::Container,
        "Pod" => QuadletUnitType::Pod,
        "Network" => QuadletUnitType::Network,
        "Volume" => QuadletUnitType::Volume,
        "Build" => QuadletUnitType::Build,
        "Image" => QuadletUnitType::Image,
        "Kube" => QuadletUnitType::Kube,
        "Artifact" => QuadletUnitType::Artifact,
        _ => return Err(format!("unsupported Quadlet section `{section}`")),
    };
    let source = format!("[{section}]\n{key}=value\n");
    let parsed = QuadletDocument::parse(unit_type, SourceId::new(92_000), source).map_err(|error| error.to_string())?;
    parsed
        .document()
        .entries()
        .next()
        .map(TypedEntry::kind)
        .ok_or_else(|| format!("{section}.{key} produced no parsed entry"))
}

fn required_string<'a>(table: &'a Table, field: &str) -> Result<&'a str, String> {
    table
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("`{field}` must be a non-empty string"))
}

fn valid_version(value: &str) -> bool {
    value
        .split('.')
        .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
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

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

fn valid_key(value: &str) -> bool {
    value.bytes().next().is_some_and(|byte| byte.is_ascii_uppercase())
        && value.bytes().all(|byte| byte.is_ascii_alphanumeric())
}

fn decode_pinned_manual() -> Result<Vec<u8>, String> {
    let evidence_path = repository_root().join(PINNED_MANUAL_EVIDENCE);
    let output = Command::new("bash")
        .arg("-c")
        .arg("base64 --decode -- \"$1\" | gzip --decompress")
        .arg("decode-pinned-manual")
        .arg(&evidence_path)
        .output()
        .map_err(|error| format!("failed to decode {}: {error}", evidence_path.display()))?;
    if output.status.success() {
        Ok(output.stdout)
    } else {
        Err(String::from_utf8_lossy(&output.stderr).into_owned())
    }
}

fn sha256sum(bytes: &[u8]) -> Result<String, String> {
    let temporary_path = temporary_manual_path();
    fs::write(&temporary_path, bytes)
        .map_err(|error| format!("failed to write {}: {error}", temporary_path.display()))?;
    let output = Command::new("sha256sum")
        .arg(&temporary_path)
        .output()
        .map_err(|error| format!("failed to hash {}: {error}", temporary_path.display()))?;
    let _ = fs::remove_file(&temporary_path);
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    let digest = String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .next()
        .ok_or_else(|| "sha256sum produced no digest".to_owned())?
        .to_owned();
    Ok(digest)
}

fn temporary_manual_path() -> PathBuf {
    std::env::temp_dir().join(format!("quadlet-lens-specification-manual-{}.md", std::process::id()))
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}
