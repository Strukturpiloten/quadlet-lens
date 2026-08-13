//! Positive and negative public capability-schema boundary tests.

use quadlet_lens::capability::{
    CapabilityCatalogue, CatalogueError, PodmanTarget, PodmanVersion, SupportClassification, VerificationLevel,
    VersionRange,
};

#[test]
fn podman_versions_ranges_and_targets_accept_inclusive_numeric_boundaries() -> Result<(), String> {
    let abbreviated: PodmanVersion = "5.4"
        .parse()
        .map_err(|error: quadlet_lens::capability::VersionParseError| error.to_string())?;
    let exact: PodmanVersion = "6.0.2"
        .parse()
        .map_err(|error: quadlet_lens::capability::VersionParseError| error.to_string())?;

    assert_eq!(abbreviated, PodmanVersion::new(5, 4, 0));
    assert_eq!((exact.major(), exact.minor(), exact.patch()), (6, 0, 2));
    assert_eq!(abbreviated.to_string(), "5.4.0");

    let full = VersionRange::new(abbreviated, exact, "supported").map_err(|error| error.to_string())?;
    let patch = VersionRange::new(PodmanVersion::new(5, 4, 1), PodmanVersion::new(5, 4, 2), "patch")
        .map_err(|error| error.to_string())?;
    let later = VersionRange::new(PodmanVersion::new(6, 0, 2), PodmanVersion::new(6, 1, 0), "later")
        .map_err(|error| error.to_string())?;
    assert!(full.covers(patch));
    assert!(full.overlaps(later));
    assert!(!patch.covers(full));

    let target = PodmanTarget::new(abbreviated, Some(exact)).map_err(|error| error.to_string())?;
    assert_eq!(target.minimum(), abbreviated);
    assert_eq!(target.maximum(), Some(exact));
    assert_eq!(
        PodmanTarget::new(exact, None)
            .map_err(|error| error.to_string())?
            .maximum(),
        None
    );
    Ok(())
}

#[test]
fn podman_versions_ranges_and_targets_reject_ambiguous_or_inverted_input() {
    for spelling in [
        "",
        "5",
        "5.4.0.1",
        ".4",
        "5.",
        "05.4",
        "5.04",
        "5.4.00",
        "v5.4",
        "5.x",
        "18446744073709551616.0",
    ] {
        let error = spelling.parse::<PodmanVersion>().err();
        assert_eq!(
            error.as_ref().map(quadlet_lens::capability::VersionParseError::value),
            Some(spelling)
        );
        assert!(
            error.is_some_and(|error| error.to_string().contains(spelling)),
            "error should identify rejected spelling {spelling:?}"
        );
    }

    assert!(matches!(
        VersionRange::new(PodmanVersion::new(6, 0, 2), PodmanVersion::new(5, 4, 0), "coverage"),
        Err(CatalogueError::InvalidRange(field)) if field == "coverage"
    ));
    assert!(matches!(
        PodmanTarget::new(PodmanVersion::new(6, 0, 2), Some(PodmanVersion::new(5, 4, 0))),
        Err(CatalogueError::InvalidRange(field)) if field == "target"
    ));
}

#[test]
fn catalogue_exposes_evidence_support_and_evaluation_details() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::parse(CATALOGUE).map_err(|error| error.to_string())?;
    assert_eq!(catalogue.schema(), 1);
    assert_eq!(catalogue.id(), "catalogue-contract");
    assert_eq!(catalogue.coverage().minimum(), PodmanVersion::new(5, 4, 0));
    assert_eq!(catalogue.coverage().maximum(), PodmanVersion::new(5, 4, 2));

    let documentation = catalogue
        .evidence()
        .iter()
        .find(|evidence| evidence.id() == "documentation")
        .ok_or_else(|| "documentation evidence missing".to_owned())?;
    assert_eq!(documentation.level(), VerificationLevel::Documentation);
    assert_eq!(documentation.url(), "https://example.invalid/documentation");
    assert_eq!(documentation.claim(), "Documents the synthetic key.");
    assert_eq!(documentation.test(), "capability_boundaries::catalogue_contract");
    assert_eq!(
        documentation.gap(),
        Some("Generator behavior remains separately evidenced.")
    );

    let capability = catalogue
        .capability("quadlet.example.key")
        .ok_or_else(|| "synthetic capability missing".to_owned())?;
    assert_eq!(capability.description(), "A capability with bounded outcomes.");
    assert_eq!(capability.unit_types(), ["container"]);
    assert_eq!(capability.sections(), ["Container"]);
    assert!(capability.is_required());
    assert!(!capability.is_repeatable());
    assert_eq!(capability.value_forms(), ["opaque-one-line"]);
    assert_eq!(capability.deprecated_from(), None);
    assert_eq!(capability.removed_from(), None);
    assert_eq!(capability.evidence(), ["documentation", "generator"]);
    assert_eq!(capability.fallbacks()[0].kind(), "podman-argument");
    assert_eq!(
        capability.fallbacks()[0].semantic_difference(),
        "Uses a lower-level argument."
    );
    assert_eq!(capability.fallbacks()[0].evidence(), ["generator"]);
    assert_eq!(capability.known_bugs()[0].summary(), "The middle patch is broken.");
    assert_eq!(capability.known_bugs()[0].evidence(), ["generator"]);
    assert_eq!(
        capability.unsupported_ranges()[0].summary(),
        "The first patch has no representation."
    );

    let first = evaluate_patch(&catalogue, 0)?;
    assert_eq!(first.classification(), SupportClassification::Unsupported);
    assert_eq!(first.capability(), "quadlet.example.key");
    assert_eq!(first.evidence(), ["generator"]);
    assert_eq!(first.note(), Some("The first patch has no representation."));

    let middle = evaluate_patch(&catalogue, 1)?;
    assert_eq!(middle.classification(), SupportClassification::Broken);
    assert_eq!(middle.note(), Some("The middle patch is broken."));

    let final_patch = evaluate_patch(&catalogue, 2)?;
    assert_eq!(final_patch.classification(), SupportClassification::Native);
    assert_eq!(final_patch.selected_fallback(), None);

    let unknown = catalogue.evaluate(
        "quadlet.example.missing",
        PodmanTarget::new(PodmanVersion::new(5, 4, 0), None).map_err(|error| error.to_string())?,
    );
    assert_eq!(unknown.classification(), SupportClassification::Unknown);
    assert!(unknown.assumes_later_versions());
    assert_eq!(unknown.evaluated_range().maximum(), PodmanVersion::new(5, 4, 2));
    Ok(())
}

#[test]
fn catalogue_rejects_decode_schema_identifier_version_field_and_evidence_errors() {
    assert!(matches!(
        CapabilityCatalogue::parse("schema = ["),
        Err(CatalogueError::Decode(_))
    ));
    assert!(matches!(
        CapabilityCatalogue::parse(&CATALOGUE.replacen("schema = 1", "schema = 2", 1)),
        Err(CatalogueError::UnsupportedSchema(2))
    ));
    assert!(matches!(
        CapabilityCatalogue::parse(&CATALOGUE.replacen("id = \"catalogue-contract\"", "id = \"Invalid_ID\"", 1)),
        Err(CatalogueError::InvalidIdentifier(id)) if id == "Invalid_ID"
    ));
    assert!(matches!(
        CapabilityCatalogue::parse(&CATALOGUE.replacen("minimum = \"5.4.0\"", "minimum = \"05.4.0\"", 1)),
        Err(CatalogueError::InvalidVersion { field, value }) if field == "coverage.minimum" && value == "05.4.0"
    ));
    assert!(matches!(
        CapabilityCatalogue::parse(&CATALOGUE.replacen("url = \"https://example.invalid/documentation\"", "url = \"http://example.invalid/documentation\"", 1)),
        Err(CatalogueError::InvalidField(field)) if field == "evidence.documentation"
    ));
    assert!(matches!(
        CapabilityCatalogue::parse(&CATALOGUE.replace("evidence = [\"generator\"]", "evidence = [\"missing\"]")),
        Err(CatalogueError::MissingEvidence { evidence, .. }) if evidence == "missing"
    ));
    assert!(matches!(
        CapabilityCatalogue::parse(&CATALOGUE.replacen("description = \"A capability with bounded outcomes.\"", "description = \"\"", 1)),
        Err(CatalogueError::InvalidField(field)) if field == "quadlet.example.key"
    ));
}

fn evaluate_patch(
    catalogue: &CapabilityCatalogue,
    patch: u64,
) -> Result<quadlet_lens::capability::CapabilityEvaluation, String> {
    let version = PodmanVersion::new(5, 4, patch);
    let target = PodmanTarget::new(version, Some(version)).map_err(|error| error.to_string())?;
    Ok(catalogue.evaluate("quadlet.example.key", target))
}

const CATALOGUE: &str = r#"
schema = 1
id = "catalogue-contract"

[coverage]
minimum = "5.4.0"
maximum = "5.4.2"

[[evidence]]
id = "documentation"
verification = "documentation"
url = "https://example.invalid/documentation"
versions = { minimum = "5.4.0", maximum = "5.4.2" }
claim = "Documents the synthetic key."
test = "capability_boundaries::catalogue_contract"
gap = "Generator behavior remains separately evidenced."

[[evidence]]
id = "generator"
verification = "generator"
url = "https://example.invalid/generator"
target = "5.4.1"
claim = "Checks the middle patch."
test = "capability_boundaries::catalogue_contract"

[[capability]]
id = "quadlet.example.key"
description = "A capability with bounded outcomes."
unit_types = ["container"]
sections = ["Container"]
required = true
repeatable = false
value_forms = ["opaque-one-line"]
native = { minimum = "5.4.2", maximum = "5.4.2" }
evidence = ["documentation", "generator"]

[[capability.fallback]]
kind = "podman-argument"
versions = { minimum = "5.4.0", maximum = "5.4.1" }
semantic_difference = "Uses a lower-level argument."
evidence = ["generator"]

[[capability.known_bug]]
versions = { minimum = "5.4.1", maximum = "5.4.1" }
summary = "The middle patch is broken."
evidence = ["generator"]

[[capability.unsupported]]
versions = { minimum = "5.4.0", maximum = "5.4.0" }
summary = "The first patch has no representation."
evidence = ["generator"]
"#;
