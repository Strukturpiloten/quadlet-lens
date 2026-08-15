//! Named Quadlet document sets and exact native dependency resolution.

use quadlet_lens::model::{
    DocumentSetError, NamedQuadletDocument, QuadletDocument, QuadletDocumentSet, QuadletUnitType, ReferenceResolution,
    SystemdUnitKey, UnitFileName, UnitReferenceKind,
};
use quadlet_lens::source::SourceId;

const APP: &str = include_str!("../fixtures/typed-model/document-set-resolution/app.container");
const PROXY: &str = include_str!("../fixtures/typed-model/document-set-resolution/proxy.container");
const POD: &str = include_str!("../fixtures/typed-model/document-set-resolution/application.pod");
const NETWORK: &str = include_str!("../fixtures/typed-model/document-set-resolution/frontend.network");
const VOLUME: &str = include_str!("../fixtures/typed-model/document-set-resolution/cache.volume");

#[test]
#[allow(clippy::too_many_lines)] // One table keeps all relationship keys and native suffixes auditable together.
fn document_set_resolves_effective_systemd_relationship_lists_with_key_identity() -> Result<(), String> {
    let source = concat!(
        "[Unit]\n",
        "Requisite=child.container pod.pod\n",
        "BindsTo=network.network\n",
        "PartOf=volume.volume\n",
        "Upholds=build.build\n",
        "Conflicts=image.image\n",
        "Before=kube.kube artifact.artifact ordinary.service ordinary.target\n",
        "Requires=discarded.container\n",
        "Requires=\n",
        "Requires=child.container\n",
        "Wants=\"pod.pod\" \\\n",
        "  network.network\n",
        "After='volume.volume'\n",
        "Before=\"malformed.container\n",
        "[Container]\n",
        "Image=example.invalid/application\n",
    );
    let set = QuadletDocumentSet::new([
        named("root.container", QuadletUnitType::Container, 9_200, source)?,
        named(
            "child.container",
            QuadletUnitType::Container,
            9_201,
            "[Container]\nImage=example.invalid/child\n",
        )?,
        named("pod.pod", QuadletUnitType::Pod, 9_202, "[Pod]\n")?,
        named("network.network", QuadletUnitType::Network, 9_203, "[Network]\n")?,
        named("volume.volume", QuadletUnitType::Volume, 9_204, "[Volume]\n")?,
        named(
            "build.build",
            QuadletUnitType::Build,
            9_205,
            "[Build]\nImageTag=example.invalid/build\n",
        )?,
        named(
            "image.image",
            QuadletUnitType::Image,
            9_206,
            "[Image]\nImage=example.invalid/image\n",
        )?,
        named(
            "kube.kube",
            QuadletUnitType::Kube,
            9_207,
            "[Kube]\nYaml=placeholder.yaml\n",
        )?,
        named(
            "artifact.artifact",
            QuadletUnitType::Artifact,
            9_208,
            "[Artifact]\nArtifact=example.invalid/artifact\n",
        )?,
    ])
    .map_err(|error| error.to_string())?;
    assert!(set.is_valid(), "{:#?}", set.diagnostics());

    let relationships = set
        .graph()
        .references()
        .iter()
        .map(|reference| (reference.target_name(), reference.kind(), reference.systemd_unit_key()))
        .collect::<Vec<_>>();
    assert_eq!(
        relationships,
        [
            (
                "child.container",
                UnitReferenceKind::Container,
                Some(SystemdUnitKey::Requisite)
            ),
            ("pod.pod", UnitReferenceKind::Pod, Some(SystemdUnitKey::Requisite)),
            (
                "network.network",
                UnitReferenceKind::Network,
                Some(SystemdUnitKey::BindsTo)
            ),
            ("volume.volume", UnitReferenceKind::Volume, Some(SystemdUnitKey::PartOf)),
            ("build.build", UnitReferenceKind::Build, Some(SystemdUnitKey::Upholds)),
            ("image.image", UnitReferenceKind::Image, Some(SystemdUnitKey::Conflicts)),
            ("kube.kube", UnitReferenceKind::Kube, Some(SystemdUnitKey::Before)),
            (
                "artifact.artifact",
                UnitReferenceKind::Artifact,
                Some(SystemdUnitKey::Before)
            ),
            (
                "child.container",
                UnitReferenceKind::Container,
                Some(SystemdUnitKey::Requires)
            ),
            ("pod.pod", UnitReferenceKind::Pod, Some(SystemdUnitKey::Wants)),
            (
                "network.network",
                UnitReferenceKind::Network,
                Some(SystemdUnitKey::Wants)
            ),
            ("volume.volume", UnitReferenceKind::Volume, Some(SystemdUnitKey::After)),
        ]
    );
    assert!(!relationships.iter().any(|(name, _, _)| *name == "discarded.container"));
    assert_eq!(set.graph().edges().len(), relationships.len());
    assert!(
        set.graph()
            .edges()
            .iter()
            .zip(set.graph().references())
            .all(|(edge, reference)| edge.systemd_unit_key() == reference.systemd_unit_key())
    );
    Ok(())
}

#[test]
fn systemd_relationship_graph_reports_missing_and_ambiguous_native_targets_only() -> Result<(), String> {
    let set = QuadletDocumentSet::new([
        named(
            "root.container",
            QuadletUnitType::Container,
            9_210,
            "[Unit]\nBefore=missing.container duplicate.network ordinary.service ordinary.target\n[Container]\nImage=x\n",
        )?,
        named("duplicate.network", QuadletUnitType::Network, 9_211, "[Network]\n")?,
        named("duplicate.network", QuadletUnitType::Network, 9_212, "[Network]\n")?,
    ])
    .map_err(|error| error.to_string())?;
    assert!(!set.is_valid());
    assert_eq!(
        set.graph()
            .references()
            .iter()
            .map(|reference| (
                reference.target_name(),
                reference.resolution(),
                reference.systemd_unit_key()
            ))
            .collect::<Vec<_>>(),
        [
            (
                "missing.container",
                ReferenceResolution::Missing,
                Some(SystemdUnitKey::Before)
            ),
            (
                "duplicate.network",
                ReferenceResolution::Ambiguous { candidates: 2 },
                Some(SystemdUnitKey::Before),
            ),
        ]
    );
    assert_eq!(diagnostic_codes(&set), ["QLG0003", "QLG0001", "QLG0002"]);
    Ok(())
}

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
fn document_set_resolves_a_volume_image_to_an_exact_build_unit() -> Result<(), String> {
    let set = QuadletDocumentSet::new([
        named(
            "cache.volume",
            QuadletUnitType::Volume,
            183,
            "[Volume]\nImage=application.build\n",
        )?,
        named(
            "application.build",
            QuadletUnitType::Build,
            184,
            "[Build]\nImageTag=localhost/application:latest\nTarget=build-stage\nSetWorkingDirectory=unit\n",
        )?,
    ])
    .map_err(|error| error.to_string())?;
    assert!(set.is_valid(), "{:#?}", set.diagnostics());
    assert_eq!(set.graph().references().len(), 1);
    assert_eq!(set.graph().edges().len(), 1);
    assert_eq!(set.graph().references()[0].kind(), UnitReferenceKind::Build);
    assert_eq!(set.graph().references()[0].target_name(), "application.build");
    Ok(())
}

#[test]
fn document_set_resolves_exact_container_and_volume_image_references_to_an_image_unit() -> Result<(), String> {
    let set = QuadletDocumentSet::new([
        named(
            "application.container",
            QuadletUnitType::Container,
            185,
            "[Container]\nImage=application.image\n",
        )?,
        named(
            "cache.volume",
            QuadletUnitType::Volume,
            186,
            "[Volume]\nImage=application.image\n",
        )?,
        named(
            "application.image",
            QuadletUnitType::Image,
            187,
            "[Image]\nImage=example.invalid/application:latest\n",
        )?,
    ])
    .map_err(|error| error.to_string())?;
    assert!(set.is_valid(), "{:#?}", set.diagnostics());
    assert_eq!(set.graph().references().len(), 2);
    assert_eq!(set.graph().edges().len(), 2);
    assert!(set.graph().references().iter().all(|reference| {
        reference.kind() == UnitReferenceKind::Image
            && reference.target_name() == "application.image"
            && reference.resolution() == ReferenceResolution::Resolved { document_index: 2 }
    }));
    assert_eq!(
        set.document("application.image")
            .map(|document| document.document().unit_type()),
        Some(QuadletUnitType::Image)
    );
    Ok(())
}

#[test]
fn image_tag_does_not_create_or_mutate_document_set_edges() -> Result<(), String> {
    let set = QuadletDocumentSet::new([
        named(
            "application.container",
            QuadletUnitType::Container,
            449,
            "[Container]\nImage=application.image\n",
        )?,
        named(
            "application.image",
            QuadletUnitType::Image,
            450,
            "[Image]\nImage=example.invalid/application:latest\nImageTag=other.image\nServiceName=other.service\nAllTags=true\nArch=arm64\nAuthFile=/placeholder/quadlet-lens-auth.json\nCertDir=/placeholder/quadlet-lens-certs\nContainersConfModule=one.conf\nContainersConfModule=two.conf\n",
        )?,
    ])
    .map_err(|error| error.to_string())?;
    assert!(set.is_valid(), "{:#?}", set.diagnostics());
    assert_eq!(set.graph().references().len(), 1);
    assert_eq!(set.graph().edges().len(), 1);
    assert_eq!(set.graph().references()[0].target_name(), "application.image");
    assert_eq!(set.graph().references()[0].kind(), UnitReferenceKind::Image);
    Ok(())
}

#[test]
fn document_set_resolves_build_volume_source_prefix_without_mutating_identities() -> Result<(), String> {
    let set = QuadletDocumentSet::new([
        named(
            "application.build",
            QuadletUnitType::Build,
            220,
            "[Build]\nImageTag=localhost/application\nVolume=cache.volume:/var/cache:Z\n",
        )?,
        named("cache.volume", QuadletUnitType::Volume, 221, "[Volume]\n")?,
    ])
    .map_err(|error| error.to_string())?;
    assert!(set.is_valid(), "{:#?}", set.diagnostics());
    assert_eq!(set.graph().references().len(), 1);
    assert_eq!(set.graph().references()[0].target_name(), "cache.volume");
    assert_eq!(set.graph().references()[0].kind(), UnitReferenceKind::Volume);
    assert_eq!(set.graph().edges().len(), 1);
    assert_eq!(
        set.documents()[set.graph().edges()[0].target_document()]
            .name()
            .as_str(),
        "cache.volume"
    );
    Ok(())
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "The table-driven integration case keeps all accepted Artifact reference spellings and outcomes auditable together."
)]
fn document_set_resolves_artifact_volume_and_mount_references_without_guessing_suffixes() -> Result<(), String> {
    let set = QuadletDocumentSet::new([
        named(
            "application.container",
            QuadletUnitType::Container,
            2_101,
            concat!(
                "[Container]\nImage=example.invalid/application:1\n",
                "Volume=shared.artifact:/volume:ro\n",
                "Mount=type=artifact,readonly,source=source.artifact,destination=/source\n",
                "Mount=type=artifact,readonly,src=src.artifact,destination=/src\n",
            ),
        )?,
        named(
            "application.pod",
            QuadletUnitType::Pod,
            2_102,
            "[Pod]\nVolume=shared.artifact:/volume:ro\n",
        )?,
        named(
            "application.build",
            QuadletUnitType::Build,
            2_103,
            "[Build]\nImageTag=localhost/application:latest\nVolume=shared.artifact:/volume:ro\n",
        )?,
        named(
            "shared.artifact",
            QuadletUnitType::Artifact,
            2_104,
            "[Artifact]\nArtifact=registry.invalid/shared:1\n",
        )?,
        named(
            "source.artifact",
            QuadletUnitType::Artifact,
            2_105,
            "[Artifact]\nArtifact=registry.invalid/source:1\n",
        )?,
        named(
            "src.artifact",
            QuadletUnitType::Artifact,
            2_106,
            "[Artifact]\nArtifact=registry.invalid/src:1\n",
        )?,
    ])
    .map_err(|error| error.to_string())?;
    assert!(set.is_valid(), "{:#?}", set.diagnostics());
    assert_eq!(set.graph().references().len(), 5);
    assert_eq!(set.graph().edges().len(), 5);
    assert!(set.graph().references().iter().all(|reference| {
        reference.kind() == UnitReferenceKind::Artifact
            && matches!(reference.resolution(), ReferenceResolution::Resolved { .. })
    }));
    assert_eq!(
        set.graph()
            .references()
            .iter()
            .map(quadlet_lens::model::UnitReference::target_name)
            .collect::<Vec<_>>(),
        [
            "shared.artifact",
            "source.artifact",
            "src.artifact",
            "shared.artifact",
            "shared.artifact"
        ]
    );

    let missing = QuadletDocumentSet::new([named(
        "missing.container",
        QuadletUnitType::Container,
        2_107,
        "[Container]\nImage=example.invalid/application:1\nMount=type=artifact,readonly,source=missing.artifact,destination=/missing\n",
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
            2_108,
            "[Container]\nImage=example.invalid/application:1\nMount=type=artifact,readonly,src=ambiguous.artifact,destination=/ambiguous\n",
        )?,
        named(
            "ambiguous.artifact",
            QuadletUnitType::Artifact,
            2_109,
            "[Artifact]\nArtifact=registry.invalid/one:1\n",
        )?,
        named(
            "ambiguous.artifact",
            QuadletUnitType::Artifact,
            2_110,
            "[Artifact]\nArtifact=registry.invalid/two:1\n",
        )?,
    ])
    .map_err(|error| error.to_string())?;
    assert!(!ambiguous.is_valid());
    assert_eq!(diagnostic_codes(&ambiguous), ["QLG0003", "QLG0002"]);
    assert_eq!(
        ambiguous.graph().references()[0].resolution(),
        ReferenceResolution::Ambiguous { candidates: 2 }
    );

    let wrong_suffix = QuadletDocumentSet::new([
        named(
            "wrong-suffix.container",
            QuadletUnitType::Container,
            2_111,
            "[Container]\nImage=example.invalid/application:1\nMount=type=artifact,source=not-an-artifact.volume,destination=/wrong\n",
        )?,
        named("not-an-artifact.volume", QuadletUnitType::Volume, 2_112, "[Volume]\n")?,
    ])
    .map_err(|error| error.to_string())?;
    assert!(wrong_suffix.is_valid(), "{:#?}", wrong_suffix.diagnostics());
    assert!(wrong_suffix.graph().references().is_empty());
    let mount = wrong_suffix.documents()[0]
        .document()
        .entries()
        .find(|entry| entry.key().text() == "Mount")
        .ok_or("missing Mount entry")?;
    assert_eq!(mount.value_kind(), quadlet_lens::model::ValueKind::Opaque);
    assert_eq!(mount.unit_reference_name(), None);
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
fn document_set_resolves_an_exact_kube_network_reference_without_loading_yaml() -> Result<(), String> {
    let set = QuadletDocumentSet::new([
        named(
            "application.kube",
            QuadletUnitType::Kube,
            1_100,
            "[Kube]\nYaml=./placeholder.yaml\nNetwork=frontend.network\n",
        )?,
        named("frontend.network", QuadletUnitType::Network, 1_101, "[Network]\n")?,
    ])
    .map_err(|error| error.to_string())?;
    assert!(set.is_valid(), "{:#?}", set.diagnostics());
    assert_eq!(set.graph().references().len(), 1);
    assert_eq!(set.graph().edges().len(), 1);
    assert_eq!(set.graph().references()[0].kind(), UnitReferenceKind::Network);
    assert_eq!(set.graph().references()[0].target_name(), "frontend.network");
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
    for invalid in [
        "",
        "application",
        ".container",
        "application.",
        "nested/app.container",
        "nested\\app.container",
    ] {
        assert!(
            matches!(
                UnitFileName::new(invalid),
                Err(DocumentSetError::InvalidUnitFileName(_))
            ),
            "unexpectedly accepted {invalid:?}"
        );
    }
    assert!(matches!(
        UnitFileName::new("application.service"),
        Err(DocumentSetError::UnsupportedUnitFileExtension(_))
    ));

    let image = UnitFileName::new("application.image").map_err(|error| error.to_string())?;
    assert_eq!(image.as_str(), "application.image");
    assert_eq!(image.to_string(), "application.image");
    assert_eq!(image.unit_type(), QuadletUnitType::Image);

    assert_eq!(
        UnitFileName::new("application.build")
            .map_err(|error| error.to_string())?
            .unit_type(),
        QuadletUnitType::Build
    );

    let network = named_document(QuadletUnitType::Network, 61, "[Network]\n")?;
    let mismatch = NamedQuadletDocument::new("wrong.volume", network);
    assert!(matches!(mismatch, Err(DocumentSetError::UnitTypeMismatch { .. })));
    assert_eq!(
        mismatch.err().map(|error| error.to_string()),
        Some("Quadlet filename `wrong.volume` implies Volume, but the document is Network".to_owned())
    );
    Ok(())
}

#[test]
fn document_set_accessors_preserve_names_sources_and_reference_spans() -> Result<(), String> {
    let source = "[Container]\nImage=example.invalid/app\nNetwork=frontend.network\n";
    let set = QuadletDocumentSet::new([
        named("app.container", QuadletUnitType::Container, 62, source)?,
        named("frontend.network", QuadletUnitType::Network, 63, "[Network]\n")?,
    ])
    .map_err(|error| error.to_string())?;

    let reference = &set.graph().references()[0];
    let edge = set.graph().edges()[0];
    let expected_start = source
        .find("frontend.network")
        .ok_or_else(|| "test source must contain its reference".to_owned())?;
    assert_eq!(reference.source_document(), 0);
    assert_eq!(reference.span().source_id(), SourceId::new(62));
    assert_eq!(
        (reference.span().start(), reference.span().end()),
        (expected_start, source.len() - 1)
    );
    assert_eq!(edge.source_document(), 0);
    assert_eq!(edge.target_document(), 1);
    assert_eq!(edge.kind(), UnitReferenceKind::Network);
    assert_eq!(edge.span(), reference.span());

    let decomposed = named("standalone.network", QuadletUnitType::Network, 64, "[Network]\n")?;
    let (name, document) = decomposed.into_parts();
    assert_eq!(name.as_str(), "standalone.network");
    assert_eq!(document.source_id(), SourceId::new(64));
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
