//! Native typed documents, conservative value forms, and model diagnostics.

use quadlet_lens::diagnostic::Severity;
use quadlet_lens::model::{
    ContainerKey, EntryKind, NetworkKey, PodKey, QuadletDocument, QuadletUnitType, SectionKind, UnitReferenceKind,
    ValueKind, VolumeKey,
};
use quadlet_lens::path::PathForm;
use quadlet_lens::source::SourceId;

const CONTAINER: &str = include_str!("../fixtures/typed-model/minimum-native-set/app.container");
const POD: &str = include_str!("../fixtures/typed-model/minimum-native-set/application.pod");
const NETWORK: &str = include_str!("../fixtures/typed-model/minimum-native-set/frontend.network");
const VOLUME: &str = include_str!("../fixtures/typed-model/minimum-native-set/cache.volume");

#[test]
fn container_model_retains_order_repetition_unknowns_and_generic_systemd() -> Result<(), String> {
    let result = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(31), CONTAINER)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
    assert_eq!(result.syntax().document().render_preserved(), CONTAINER);

    let sections: Vec<_> = result
        .document()
        .sections()
        .iter()
        .map(quadlet_lens::model::TypedSection::kind)
        .collect();
    assert_eq!(
        sections,
        [
            SectionKind::Unit,
            SectionKind::Container,
            SectionKind::Service,
            SectionKind::Install
        ]
    );

    let generic: Vec<_> = result
        .document()
        .entries()
        .filter(|entry| entry.kind() == EntryKind::GenericSystemd)
        .map(|entry| entry.key().text())
        .collect();
    assert_eq!(generic, ["Description", "After", "After", "Restart", "WantedBy"]);

    let known: Vec<_> = result
        .document()
        .entries()
        .filter_map(|entry| match entry.kind() {
            EntryKind::Container(key) => Some(key),
            _ => None,
        })
        .collect();
    assert_eq!(
        known,
        [
            ContainerKey::ContainerName,
            ContainerKey::AddHost,
            ContainerKey::AddHost,
            ContainerKey::Image,
            ContainerKey::Exec,
            ContainerKey::Environment,
            ContainerKey::Environment,
            ContainerKey::EnvironmentFile,
            ContainerKey::Label,
            ContainerKey::Label,
            ContainerKey::Label,
            ContainerKey::Label,
            ContainerKey::Secret,
            ContainerKey::Secret,
            ContainerKey::User,
            ContainerKey::Group,
            ContainerKey::UserNS,
            ContainerKey::GroupAdd,
            ContainerKey::GroupAdd,
            ContainerKey::WorkingDir,
            ContainerKey::ReadOnly,
            ContainerKey::PublishPort,
            ContainerKey::Volume,
            ContainerKey::Volume,
            ContainerKey::Network,
            ContainerKey::Pod,
            ContainerKey::HealthCmd,
            ContainerKey::Notify,
            ContainerKey::HealthInterval,
            ContainerKey::HealthRetries,
            ContainerKey::HealthStartPeriod,
            ContainerKey::HealthTimeout,
            ContainerKey::PodmanArgs,
        ]
    );
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Unknown)
            .map(|entry| entry.key().text())
            .collect::<Vec<_>>(),
        ["FutureContainerKey"]
    );

    let after_lines: Vec<_> = result
        .document()
        .entries()
        .filter(|entry| entry.key().text() == "After")
        .map(quadlet_lens::model::TypedEntry::source_line)
        .collect();
    assert_eq!(after_lines.len(), 2);
    assert!(after_lines[0] < after_lines[1]);
    Ok(())
}

#[test]
fn container_model_classifies_native_references_paths_and_continuations() -> Result<(), String> {
    let result = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(32), CONTAINER)
        .map_err(|error| error.to_string())?;

    assert_value_kind(
        &result,
        ContainerKey::EnvironmentFile,
        0,
        ValueKind::Path(PathForm::SystemdSpecifier),
    )?;
    assert_value_kind(
        &result,
        ContainerKey::Volume,
        0,
        ValueKind::UnitReference(UnitReferenceKind::Volume),
    )?;
    assert_value_kind(
        &result,
        ContainerKey::Volume,
        1,
        ValueKind::Path(PathForm::UnitRelativeLiteral),
    )?;
    assert_value_kind(
        &result,
        ContainerKey::Network,
        0,
        ValueKind::UnitReference(UnitReferenceKind::Network),
    )?;
    assert_value_kind(
        &result,
        ContainerKey::Pod,
        0,
        ValueKind::UnitReference(UnitReferenceKind::Pod),
    )?;

    let podman_args = container_entry(&result, ContainerKey::PodmanArgs, 0)?;
    assert!(podman_args.value().is_continued());
    assert_eq!(podman_args.value().continuations().len(), 1);
    assert_eq!(podman_args.value().continuations()[0].text(), "--label second=value");
    assert!(podman_args.value().primary().text().ends_with('\\'));

    assert_eq!(
        container_entry(&result, ContainerKey::Label, 2)?
            .value()
            .primary()
            .text(),
        "org.example.empty="
    );
    assert_eq!(
        container_entry(&result, ContainerKey::Label, 3)?
            .value()
            .primary()
            .text(),
        r#""org.example.metadata={\"channel\": \"stable\"}""#
    );
    Ok(())
}

#[test]
fn network_and_volume_models_retain_known_and_future_fields() -> Result<(), String> {
    let pod =
        QuadletDocument::parse(QuadletUnitType::Pod, SourceId::new(40), POD).map_err(|error| error.to_string())?;
    let network = QuadletDocument::parse(QuadletUnitType::Network, SourceId::new(33), NETWORK)
        .map_err(|error| error.to_string())?;
    let volume = QuadletDocument::parse(QuadletUnitType::Volume, SourceId::new(34), VOLUME)
        .map_err(|error| error.to_string())?;
    assert!(pod.is_valid());
    assert!(network.is_valid());
    assert!(volume.is_valid());
    assert_eq!(
        pod.document()
            .entries()
            .filter_map(|entry| match entry.kind() {
                EntryKind::Pod(key) => Some(key),
                _ => None,
            })
            .collect::<Vec<_>>(),
        [
            PodKey::AddHost,
            PodKey::PodName,
            PodKey::PublishPort,
            PodKey::Network,
            PodKey::Volume,
            PodKey::UserNS
        ]
    );
    assert!(pod.document().entries().any(|entry| {
        entry.kind() == EntryKind::Pod(PodKey::Network)
            && entry.value_kind() == ValueKind::UnitReference(UnitReferenceKind::Network)
    }));
    assert!(pod.document().entries().any(|entry| {
        entry.kind() == EntryKind::Pod(PodKey::Volume)
            && entry.value_kind() == ValueKind::UnitReference(UnitReferenceKind::Volume)
    }));
    assert!(network.document().entries().any(|entry| {
        entry.kind() == EntryKind::Network(NetworkKey::NetworkName)
            && entry.value().primary().text() == "example-frontend"
    }));
    assert!(volume.document().entries().any(|entry| {
        entry.kind() == EntryKind::Volume(VolumeKey::VolumeName) && entry.value().primary().text() == "example-cache"
    }));
    assert_eq!(
        pod.document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Unknown)
            .count(),
        1
    );
    assert_eq!(
        network
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Unknown)
            .count(),
        1
    );
    assert_eq!(
        volume
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Unknown)
            .count(),
        1
    );
    Ok(())
}

#[test]
fn model_diagnostics_are_source_aware_and_recoverable() -> Result<(), String> {
    let source = "[Container]\nImage=\nImage=example.invalid/app\n[Volume]\nVolumeName=wrong-kind\n";
    let result = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(35), source)
        .map_err(|error| error.to_string())?;
    assert!(!result.is_valid());
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        ["QLM0003", "QLM0004", "QLM0005"]
    );
    assert!(
        result.model_diagnostics().iter().any(|diagnostic| {
            diagnostic.code().as_str() == "QLM0003" && diagnostic.severity() == Severity::Warning
        })
    );
    assert!(
        result
            .model_diagnostics()
            .iter()
            .flat_map(quadlet_lens::diagnostic::Diagnostic::labels)
            .all(|label| result.syntax().document().source().slice(label.span()).is_some())
    );

    let missing = QuadletDocument::parse(
        QuadletUnitType::Container,
        SourceId::new(36),
        "[Unit]\nDescription=No native section\n",
    )
    .map_err(|error| error.to_string())?;
    assert_eq!(
        missing
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        ["QLM0001"]
    );
    Ok(())
}

#[test]
fn supported_extensions_are_explicit_and_fail_closed() {
    assert_eq!(
        QuadletUnitType::from_extension("container"),
        Some(QuadletUnitType::Container)
    );
    assert_eq!(
        QuadletUnitType::from_extension("network"),
        Some(QuadletUnitType::Network)
    );
    assert_eq!(QuadletUnitType::from_extension("volume"), Some(QuadletUnitType::Volume));
    assert_eq!(QuadletUnitType::from_extension("pod"), Some(QuadletUnitType::Pod));
    assert_eq!(QuadletUnitType::from_extension("Container"), None);
}

#[test]
fn image_unit_references_and_dangling_continuations_stay_explicit() -> Result<(), String> {
    for (source_id, image, expected) in [
        (37, "application.image", UnitReferenceKind::Image),
        (38, "application.build", UnitReferenceKind::Build),
    ] {
        let source = format!("[Container]\nImage={image}\n");
        let result = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(source_id), source)
            .map_err(|error| error.to_string())?;
        assert!(result.is_valid());
        assert_value_kind(&result, ContainerKey::Image, 0, ValueKind::UnitReference(expected))?;
    }

    let dangling = QuadletDocument::parse(
        QuadletUnitType::Container,
        SourceId::new(39),
        "[Container]\nImage=example.invalid/app \\",
    )
    .map_err(|error| error.to_string())?;
    assert!(!dangling.syntax().is_valid());
    let image = container_entry(&dangling, ContainerKey::Image, 0)?;
    assert!(image.value().is_continued());
    assert!(image.value().continuations().is_empty());
    Ok(())
}

#[test]
fn rootfs_is_a_typed_image_alternative_and_conflicts_remain_explicit() -> Result<(), String> {
    let rootfs = QuadletDocument::parse(
        QuadletUnitType::Container,
        SourceId::new(41),
        "[Container]\nRootfs=/var/lib/qm/rootfs\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(rootfs.is_valid(), "{:#?}", rootfs.model_diagnostics());
    assert_value_kind(
        &rootfs,
        ContainerKey::Rootfs,
        0,
        ValueKind::Path(PathForm::AbsoluteLiteral),
    )?;

    let conflicting = QuadletDocument::parse(
        QuadletUnitType::Container,
        SourceId::new(42),
        "[Container]\nImage=example.invalid/app\nRootfs=/var/lib/qm/rootfs\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(!conflicting.is_valid());
    assert_eq!(
        conflicting
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        ["QLM0006"]
    );

    let empty = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(43), "[Container]\nRootfs=\n")
        .map_err(|error| error.to_string())?;
    assert_eq!(
        empty
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        ["QLM0007"]
    );
    Ok(())
}

fn assert_value_kind(
    result: &quadlet_lens::model::QuadletParseResult,
    key: ContainerKey,
    occurrence: usize,
    expected: ValueKind,
) -> Result<(), String> {
    assert_eq!(container_entry(result, key, occurrence)?.value_kind(), expected);
    Ok(())
}

fn container_entry(
    result: &quadlet_lens::model::QuadletParseResult,
    key: ContainerKey,
    occurrence: usize,
) -> Result<&quadlet_lens::model::TypedEntry, String> {
    result
        .document()
        .entries()
        .filter(|entry| entry.kind() == EntryKind::Container(key))
        .nth(occurrence)
        .ok_or_else(|| format!("fixture has no {key:?} occurrence {occurrence}"))
}
