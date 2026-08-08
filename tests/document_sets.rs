//! Named Quadlet document sets and exact native dependency resolution.

use quadlet_lens::model::{
    DocumentSetError, NamedQuadletDocument, QuadletDocument, QuadletDocumentSet, QuadletUnitType, ReferenceResolution,
    UnitFileName, UnitReferenceKind,
};
use quadlet_lens::source::SourceId;

const APP: &str = include_str!("../fixtures/typed-model/document-set-resolution/app.container");
const PROXY: &str = include_str!("../fixtures/typed-model/document-set-resolution/proxy.container");
const POD: &str = include_str!("../fixtures/typed-model/document-set-resolution/application.pod");
const NETWORK: &str = include_str!("../fixtures/typed-model/document-set-resolution/frontend.network");
const VOLUME: &str = include_str!("../fixtures/typed-model/document-set-resolution/cache.volume");

#[test]
fn document_set_resolves_container_pod_network_and_volume_dependencies() -> Result<(), String> {
    let set = QuadletDocumentSet::new([
        named("app.container", QuadletUnitType::Container, 51, APP)?,
        named("proxy.container", QuadletUnitType::Container, 52, PROXY)?,
        named("application.pod", QuadletUnitType::Pod, 53, POD)?,
        named("frontend.network", QuadletUnitType::Network, 54, NETWORK)?,
        named("cache.volume", QuadletUnitType::Volume, 55, VOLUME)?,
    ])
    .map_err(|error| error.to_string())?;

    assert!(set.is_valid(), "{:#?}", set.diagnostics());
    assert!(set.graph().is_complete());
    assert_eq!(set.graph().references().len(), 5);
    assert_eq!(set.graph().edges().len(), 5);
    assert_eq!(
        set.document("application.pod")
            .map(|document| document.document().unit_type()),
        Some(QuadletUnitType::Pod)
    );

    let edges: Vec<_> = set
        .graph()
        .edges()
        .iter()
        .map(|edge| {
            (
                set.documents()[edge.source_document()].name().as_str(),
                set.documents()[edge.target_document()].name().as_str(),
                edge.kind(),
            )
        })
        .collect();
    assert_eq!(
        edges,
        [
            ("app.container", "cache.volume", UnitReferenceKind::Volume),
            ("app.container", "application.pod", UnitReferenceKind::Pod),
            ("proxy.container", "frontend.network", UnitReferenceKind::Network),
            ("application.pod", "frontend.network", UnitReferenceKind::Network),
            ("application.pod", "cache.volume", UnitReferenceKind::Volume),
        ]
    );
    Ok(())
}

#[test]
fn document_set_resolves_a_container_image_to_an_exact_build_unit() -> Result<(), String> {
    let set = QuadletDocumentSet::new([
        named(
            "application.container",
            QuadletUnitType::Container,
            181,
            "[Container]\nImage=application.build\n",
        )?,
        named(
            "application.build",
            QuadletUnitType::Build,
            182,
            "[Build]\nImageTag=localhost/application:latest\nTarget=build-stage\nSetWorkingDirectory=unit\n",
        )?,
    ])
    .map_err(|error| error.to_string())?;
    assert!(set.is_valid(), "{:#?}", set.diagnostics());
    assert_eq!(set.graph().references().len(), 1);
    assert_eq!(set.graph().edges().len(), 1);
    assert_eq!(set.graph().references()[0].kind(), UnitReferenceKind::Build);
    assert_eq!(
        set.graph().references()[0].resolution(),
        ReferenceResolution::Resolved { document_index: 1 }
    );
    assert_eq!(set.graph().edges()[0].target_document(), 1);
    assert_eq!(
        set.document("application.build")
            .map(|document| document.document().unit_type()),
        Some(QuadletUnitType::Build)
    );
    Ok(())
}

#[test]
fn document_set_resolves_an_exact_build_network_reference() -> Result<(), String> {
    let set = QuadletDocumentSet::new([
        named(
            "application.build",
            QuadletUnitType::Build,
            183,
            "[Build]\nNetwork=frontend.network\nNetwork=frontend.network:ip=192.0.2.10\nNetwork=frontend.container\n",
        )?,
        named("frontend.network", QuadletUnitType::Network, 184, "[Network]\n")?,
    ])
    .map_err(|error| error.to_string())?;
    assert!(set.is_valid(), "{:#?}", set.diagnostics());
    assert_eq!(set.graph().references().len(), 1);
    assert_eq!(set.graph().edges().len(), 1);
    assert_eq!(set.graph().references()[0].kind(), UnitReferenceKind::Network);
    assert_eq!(set.graph().references()[0].target_name(), "frontend.network");
    assert_eq!(
        set.graph().references()[0].resolution(),
        ReferenceResolution::Resolved { document_index: 1 }
    );
    assert_eq!(set.graph().edges()[0].target_document(), 1);
    Ok(())
}

#[test]
fn document_set_reports_missing_ambiguous_and_duplicate_identities() -> Result<(), String> {
    let missing = QuadletDocumentSet::new([named(
        "missing.container",
        QuadletUnitType::Container,
        56,
        "[Container]\nImage=example.invalid/app\nNetwork=absent.network\n",
    )?])
    .map_err(|error| error.to_string())?;
    assert!(!missing.is_valid());
    assert_eq!(diagnostic_codes(&missing), ["QLG0001"]);
    assert_eq!(
        missing.graph().references()[0].resolution(),
        ReferenceResolution::Missing
    );

    let ambiguous = QuadletDocumentSet::new([
        named(
            "ambiguous.container",
            QuadletUnitType::Container,
            57,
            "[Container]\nImage=example.invalid/app\nNetwork=duplicate.network\n",
        )?,
        named("duplicate.network", QuadletUnitType::Network, 58, "[Network]\n")?,
        named("duplicate.network", QuadletUnitType::Network, 59, "[Network]\n")?,
    ])
    .map_err(|error| error.to_string())?;
    assert!(!ambiguous.is_valid());
    assert_eq!(diagnostic_codes(&ambiguous), ["QLG0003", "QLG0002"]);
    assert_eq!(
        ambiguous.graph().references()[0].resolution(),
        ReferenceResolution::Ambiguous { candidates: 2 }
    );
    assert!(ambiguous.graph().edges().is_empty());
    assert!(ambiguous.document("duplicate.network").is_none());

    let duplicate_source = QuadletDocumentSet::new([
        named("one.network", QuadletUnitType::Network, 60, "[Network]\n")?,
        named("two.network", QuadletUnitType::Network, 60, "[Network]\n")?,
    ]);
    assert!(matches!(duplicate_source, Err(DocumentSetError::DuplicateSourceId(id)) if id == SourceId::new(60)));
    Ok(())
}

#[test]
fn unit_file_names_are_basenames_with_matching_supported_suffixes() -> Result<(), String> {
    assert!(matches!(
        UnitFileName::new("nested/app.container"),
        Err(DocumentSetError::InvalidUnitFileName(_))
    ));
    assert!(matches!(
        UnitFileName::new("application.image"),
        Err(DocumentSetError::UnsupportedUnitFileExtension(_))
    ));

    assert_eq!(
        UnitFileName::new("application.build")
            .map_err(|error| error.to_string())?
            .unit_type(),
        QuadletUnitType::Build
    );

    let network = named_document(QuadletUnitType::Network, 61, "[Network]\n")?;
    assert!(matches!(
        NamedQuadletDocument::new("wrong.volume", network),
        Err(DocumentSetError::UnitTypeMismatch { .. })
    ));
    Ok(())
}

fn named(name: &str, unit_type: QuadletUnitType, source_id: u32, source: &str) -> Result<NamedQuadletDocument, String> {
    let document = named_document(unit_type, source_id, source)?;
    NamedQuadletDocument::new(name, document).map_err(|error| error.to_string())
}

fn named_document(unit_type: QuadletUnitType, source_id: u32, source: &str) -> Result<QuadletDocument, String> {
    let result =
        QuadletDocument::parse(unit_type, SourceId::new(source_id), source).map_err(|error| error.to_string())?;
    if !result.is_valid() {
        return Err(format!(
            "test source has diagnostics: {:#?}",
            result.model_diagnostics()
        ));
    }
    let (_, document, _) = result.into_parts();
    Ok(document)
}

fn diagnostic_codes(set: &QuadletDocumentSet) -> Vec<&str> {
    set.diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.code().as_str())
        .collect()
}
