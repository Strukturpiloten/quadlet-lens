//! Catalogue schema, rolling supported range, and version-boundary behavior.

use std::collections::BTreeSet;

use quadlet_lens::capability::{
    CapabilityCatalogue, PodmanTarget, PodmanVersion, SupportClassification, VerificationLevel,
};

const EXPECTED_CAPABILITIES: &str =
    include_str!("../fixtures/capabilities/podman-supported-range/expected-capabilities.txt");
const EXPECTED_BOUNDARIES: &str = include_str!("../fixtures/version-boundaries/podman-5-4-floor/expected.txt");

#[test]
fn supported_range_has_the_reviewed_first_conversion_surface() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    assert_eq!(catalogue.schema(), 1);
    assert_eq!(catalogue.id(), "podman-supported-range");
    assert_eq!(catalogue.coverage().minimum(), version(5, 4, 0));
    assert_eq!(catalogue.coverage().maximum(), version(6, 0, 2));

    let actual: BTreeSet<_> = catalogue
        .capabilities()
        .iter()
        .map(quadlet_lens::capability::CapabilityRecord::id)
        .collect();
    let expected: BTreeSet<_> = EXPECTED_CAPABILITIES.lines().filter(|line| !line.is_empty()).collect();
    assert_eq!(actual, expected);

    let documentation: Vec<_> = catalogue
        .evidence()
        .iter()
        .filter(|evidence| evidence.level() == VerificationLevel::Documentation)
        .collect();
    assert!(!documentation.is_empty());
    assert!(documentation.iter().all(|evidence| evidence.gap().is_some()));
    let generator = catalogue
        .evidence()
        .iter()
        .find(|evidence| evidence.level() == VerificationLevel::Generator)
        .ok_or_else(|| "supported range must have generator evidence".to_owned())?;
    assert_eq!(generator.versions().minimum(), version(5, 4, 0));
    assert_eq!(generator.versions().maximum(), version(6, 0, 2));
    assert_eq!(generator.gap(), None);
    Ok(())
}

#[test]
fn supported_range_records_paths_references_and_repetition() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let volume = catalogue
        .capability("quadlet.container.volume")
        .ok_or_else(|| "container volume capability must exist".to_owned())?;
    assert!(volume.is_repeatable());
    assert!(volume.value_forms().iter().any(|form| form == "unit-relative-path"));
    assert!(volume.value_forms().iter().any(|form| form == "volume-unit-reference"));

    let secret = catalogue
        .capability("quadlet.container.secret")
        .ok_or_else(|| "container secret capability must exist".to_owned())?;
    assert!(secret.is_repeatable());
    assert!(
        secret
            .value_forms()
            .iter()
            .any(|form| form == "podman-secret-mount-options")
    );

    let label = catalogue
        .capability("quadlet.container.label")
        .ok_or_else(|| "container label capability must exist".to_owned())?;
    assert!(label.is_repeatable());
    assert!(label.value_forms().iter().any(|form| form == "oci-label-assignment"));
    assert!(
        secret
            .value_forms()
            .iter()
            .any(|form| form == "podman-secret-environment-options")
    );

    let image = catalogue
        .capability("quadlet.container.image")
        .ok_or_else(|| "container image capability must exist".to_owned())?;
    let rootfs = catalogue
        .capability("quadlet.container.rootfs")
        .ok_or_else(|| "container rootfs capability must exist".to_owned())?;
    assert!(!image.is_required());
    assert!(!rootfs.is_required());
    assert!(rootfs.value_forms().iter().any(|form| form == "podman-rootfs"));

    let restart = catalogue
        .capability("systemd.service.restart")
        .ok_or_else(|| "generic systemd restart capability must exist".to_owned())?;
    assert_eq!(restart.sections(), ["Service"]);

    for capability in ["systemd.unit.requires", "systemd.unit.wants", "systemd.unit.after"] {
        let record = catalogue
            .capability(capability)
            .ok_or_else(|| format!("{capability} capability must exist"))?;
        assert_eq!(record.sections(), ["Unit"]);
        assert!(record.is_repeatable());
    }

    let current = PodmanTarget::new(version(6, 0, 2), Some(version(6, 0, 2))).map_err(|error| error.to_string())?;
    for capability in [
        "quadlet.unit-type.pod",
        "quadlet.container.add-host",
        "quadlet.container.container-name",
        "quadlet.container.environment-file",
        "quadlet.container.user",
        "quadlet.container.group",
        "quadlet.container.userns",
        "quadlet.container.group-add",
        "quadlet.container.working-dir",
        "quadlet.container.read-only",
        "quadlet.container.rootfs",
        "quadlet.container.secret",
        "quadlet.container.label",
        "quadlet.container.pod",
        "quadlet.container.health-command",
        "quadlet.container.health-interval",
        "quadlet.container.health-retries",
        "quadlet.container.health-start-period",
        "quadlet.container.health-timeout",
        "quadlet.container.notify-healthy",
        "systemd.unit.requires",
        "systemd.unit.wants",
        "systemd.unit.after",
        "quadlet.pod.add-host",
        "quadlet.pod.name",
        "quadlet.pod.publish-port",
        "quadlet.pod.network",
        "quadlet.pod.volume",
        "quadlet.pod.userns",
    ] {
        assert_eq!(
            catalogue.evaluate(capability, current).classification(),
            SupportClassification::Native
        );
    }
    Ok(())
}

#[test]
fn podman_5_4_floor_is_fail_closed_outside_evidence_coverage() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    let capability = "quadlet.container.image";
    let cases = [
        ("5.3.0..=5.3.0 unknown", target(5, 3, Some((5, 3)))?),
        ("5.4.0..=5.4.0 native", target(5, 4, Some((5, 4)))?),
        ("5.4.0..=5.8.0 native", target(5, 4, Some((5, 8)))?),
        ("5.4.0..=6.0.0 native", target(5, 4, Some((6, 0)))?),
        (
            "6.0.2..=6.0.2 native",
            PodmanTarget::new(version(6, 0, 2), Some(version(6, 0, 2))).map_err(|error| error.to_string())?,
        ),
        ("5.4.0..=6.1.0 unknown", target(5, 4, Some((6, 1)))?),
    ];
    let mut observed = Vec::new();
    for (label, target) in cases {
        let evaluation = catalogue.evaluate(capability, target);
        let expected = label.rsplit_once(' ').map(|(_, value)| value).unwrap_or_default();
        assert_eq!(classification_name(evaluation.classification()), expected);
        observed.push(label);
    }
    assert_eq!(observed.join("\n") + "\n", EXPECTED_BOUNDARIES);

    let open_ended = catalogue.evaluate(capability, target(5, 4, None)?);
    assert_eq!(open_ended.classification(), SupportClassification::Native);
    assert!(open_ended.assumes_later_versions());
    assert!(open_ended.note().is_some());

    let future_open_ended = catalogue.evaluate(capability, target(5, 5, None)?);
    assert_eq!(future_open_ended.classification(), SupportClassification::Native);
    assert_eq!(future_open_ended.evaluated_range().minimum(), version(5, 5, 0));
    assert_eq!(future_open_ended.evaluated_range().maximum(), version(6, 0, 2));

    let unverified_form = catalogue.evaluate("quadlet.container.image-unit-reference", target(6, 0, Some((6, 0)))?);
    assert_eq!(unverified_form.classification(), SupportClassification::Unknown);
    Ok(())
}

#[test]
fn schema_evaluates_fallbacks_and_known_bugs_before_native_support() -> Result<(), String> {
    let catalogue = CapabilityCatalogue::parse(SYNTHETIC_CATALOGUE).map_err(|error| error.to_string())?;
    let fallback = catalogue.evaluate("quadlet.example.fallback", target(5, 4, Some((5, 4)))?);
    assert_eq!(fallback.classification(), SupportClassification::Fallback);
    assert_eq!(fallback.selected_fallback(), Some("podman-argument"));

    let broken = catalogue.evaluate(
        "quadlet.example.broken",
        PodmanTarget::new(version(5, 4, 1), Some(version(5, 4, 1))).map_err(|error| error.to_string())?,
    );
    assert_eq!(broken.classification(), SupportClassification::Broken);
    assert_eq!(broken.note(), Some("documented patch regression"));

    let deprecated = catalogue.evaluate(
        "quadlet.example.lifecycle",
        PodmanTarget::new(version(5, 4, 0), Some(version(5, 4, 0))).map_err(|error| error.to_string())?,
    );
    assert_eq!(deprecated.classification(), SupportClassification::Deprecated);

    let removed = catalogue.evaluate(
        "quadlet.example.lifecycle",
        PodmanTarget::new(version(5, 4, 1), Some(version(5, 4, 1))).map_err(|error| error.to_string())?,
    );
    assert_eq!(removed.classification(), SupportClassification::Removed);
    Ok(())
}

fn target(major: u64, minor: u64, maximum: Option<(u64, u64)>) -> Result<PodmanTarget, String> {
    PodmanTarget::new(
        version(major, minor, 0),
        maximum.map(|(major, minor)| version(major, minor, 0)),
    )
    .map_err(|error| error.to_string())
}

const fn version(major: u64, minor: u64, patch: u64) -> PodmanVersion {
    PodmanVersion::new(major, minor, patch)
}

const fn classification_name(classification: SupportClassification) -> &'static str {
    match classification {
        SupportClassification::Native => "native",
        SupportClassification::Fallback => "fallback",
        SupportClassification::Deprecated => "deprecated",
        SupportClassification::Removed => "removed",
        SupportClassification::Unsupported => "unsupported",
        SupportClassification::Unknown => "unknown",
        SupportClassification::Broken => "broken",
        _ => "future",
    }
}

const SYNTHETIC_CATALOGUE: &str = r#"
schema = 1
id = "synthetic-5-4"

[coverage]
minimum = "5.4.0"
maximum = "5.4.2"

[[evidence]]
id = "generator-5-4"
verification = "generator"
url = "https://example.invalid/generator-evidence"
target = "5.4.1"
claim = "Synthetic exact-version behavior."
test = "capabilities::schema_evaluates_fallbacks_and_known_bugs_before_native_support"

[[capability]]
id = "quadlet.example.fallback"
description = "Synthetic fallback coverage."
unit_types = ["container"]
sections = ["Container"]
evidence = ["generator-5-4"]

[[capability.fallback]]
kind = "podman-argument"
versions = { minimum = "5.4.0", maximum = "5.4.2" }
semantic_difference = "The native key spelling is unavailable."
evidence = ["generator-5-4"]

[[capability]]
id = "quadlet.example.broken"
description = "Synthetic known patch regression."
unit_types = ["container"]
sections = ["Container"]
native = { minimum = "5.4.0", maximum = "5.4.2" }
evidence = ["generator-5-4"]

[[capability.known_bug]]
versions = { minimum = "5.4.1", maximum = "5.4.1" }
summary = "documented patch regression"
evidence = ["generator-5-4"]

[[capability]]
id = "quadlet.example.lifecycle"
description = "Synthetic deprecation and removal boundaries."
unit_types = ["container"]
sections = ["Container"]
native = { minimum = "5.4.0", maximum = "5.4.0" }
deprecated_from = "5.4.0"
removed_from = "5.4.1"
evidence = ["generator-5-4"]
"#;
