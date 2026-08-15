//! Native typed documents, conservative value forms, and model diagnostics.

use quadlet_lens::diagnostic::Severity;
use quadlet_lens::model::{
    ArtifactKey, AuthoredContainerEnvironmentDirective, AuthoredContainerEnvironmentValue, BuildKey, ContainerKey,
    EntryKind, ImageKey, KubeKey, NetworkKey, PodKey, QuadletDocument, QuadletKey, QuadletUnitType, SectionKind,
    SystemdUnitKey, TypedEntry, UnitReferenceKind, ValueKind, VolumeKey,
};
use quadlet_lens::path::PathForm;
use quadlet_lens::source::SourceId;

const CONTAINER: &str = include_str!("../fixtures/typed-model/minimum-native-set/app.container");
const POD: &str = include_str!("../fixtures/typed-model/minimum-native-set/application.pod");
const NETWORK: &str = include_str!("../fixtures/typed-model/minimum-native-set/frontend.network");
const VOLUME: &str = include_str!("../fixtures/typed-model/minimum-native-set/cache.volume");
const BUILD: &str = include_str!("../fixtures/typed-model/build-core/application.build");

const BUILD_TARGET_DUPLICATES: &str = "[Build]\nTarget=builder\nTarget=final\n";
const BUILD_PLATFORM_DUPLICATES: &str = "[Build]\nArch=\nArch=arm64\nVariant=\nVariant=v8\n";
const BUILD_PODMAN_ARGS: &str =
    "[Build]\nPodmanArgs=--build-context extra=container-image://alpine:3.15\nPodmanArgs=--layers\n";
const CONTAINER_ENVIRONMENT_RESETS: &str = concat!(
    "[Container]\n",
    "Image=example.invalid/environment-reset\n",
    "Environment=PRE_ONE=one\n",
    "Environment=PRE_TWO=two\n",
    "Environment=\n",
    "Environment=POST_ONE=one\n",
    "Environment=POST_TWO=two\n",
);

#[test]
fn authored_container_environment_view_preserves_order_resets_empty_values_and_continuations() -> Result<(), String> {
    let parsed = QuadletDocument::parse(
        QuadletUnitType::Container,
        SourceId::new(9_500),
        concat!(
            "[Container]\n",
            "Image=example.invalid/application\n",
            "Environment=FIRST=one \"SPACED=two words\" ESCAPED=three\\x20words EMPTY=\n",
            "Environment=FIRST=later \\\n",
            "  \"CONTINUED=joined value\"\n",
            "Environment=\n",
            "Environment=AFTER=final BARE\n",
        ),
    )
    .map_err(|error| error.to_string())?;
    let environment = parsed.document().container_environment();
    assert!(environment.is_complete());
    assert!(environment.diagnostics().is_empty());
    assert_eq!(environment.directives().len(), 9);
    assert!(matches!(
        environment.directives(),
        [
            AuthoredContainerEnvironmentDirective::Assignment { name, value, .. },
            AuthoredContainerEnvironmentDirective::Assignment { .. },
            AuthoredContainerEnvironmentDirective::Assignment { .. },
            AuthoredContainerEnvironmentDirective::Assignment { .. },
            AuthoredContainerEnvironmentDirective::Assignment { .. },
            AuthoredContainerEnvironmentDirective::Assignment { .. },
            AuthoredContainerEnvironmentDirective::Reset { .. },
            AuthoredContainerEnvironmentDirective::Assignment { .. },
            AuthoredContainerEnvironmentDirective::BareName { .. },
        ] if name == "FIRST" && value == "one"
    ));
    assert_eq!(environment.get("FIRST"), AuthoredContainerEnvironmentValue::Absent);
    assert_eq!(environment.get("SPACED"), AuthoredContainerEnvironmentValue::Absent);
    assert_eq!(environment.get("EMPTY"), AuthoredContainerEnvironmentValue::Absent);
    assert_eq!(environment.get("CONTINUED"), AuthoredContainerEnvironmentValue::Absent);
    assert_eq!(
        environment.get("AFTER"),
        AuthoredContainerEnvironmentValue::Literal("final")
    );
    assert_eq!(environment.get("BARE"), AuthoredContainerEnvironmentValue::Deferred);
    assert_eq!(environment.directives()[1].literal_value(), Some("two words"));
    assert_eq!(environment.directives()[2].literal_value(), Some("three words"));
    assert_eq!(environment.directives()[3].literal_value(), Some(""));
    Ok(())
}

#[test]
fn authored_container_environment_view_keeps_specifiers_and_malformed_tokens_recoverable_and_redacted()
-> Result<(), String> {
    let parsed = QuadletDocument::parse(
        QuadletUnitType::Container,
        SourceId::new(9_501),
        concat!(
            "[Container]\n",
            "Image=example.invalid/application\n",
            "Environment=DEFERRED=%h BAD-NAME=leak-me\n",
            "Environment=\"BROKEN=leak-me\n",
            "Environment=ESCAPE=leak\\q\n",
        ),
    )
    .map_err(|error| error.to_string())?;
    let environment = parsed.document().container_environment();
    assert!(!environment.is_complete());
    assert_eq!(environment.get("DEFERRED"), AuthoredContainerEnvironmentValue::Deferred);
    assert!(matches!(
        environment.directives()[0],
        AuthoredContainerEnvironmentDirective::Deferred { ref name, .. } if name == "DEFERRED"
    ));
    assert_eq!(
        environment
            .diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        ["QLM0024", "QLM0023", "QLM0023", "QLM0023"]
    );
    let debug = format!("{environment:?}");
    for secret in ["%h", "leak-me", "BROKEN", "leak\\q"] {
        assert!(!debug.contains(secret), "debug leaked {secret:?}");
    }
    Ok(())
}

#[test]
fn authored_container_environment_view_is_empty_and_complete_for_non_container_documents() -> Result<(), String> {
    let parsed = QuadletDocument::parse(
        QuadletUnitType::Build,
        SourceId::new(9_502),
        "[Build]\nEnvironment=BUILD_SECRET=must-not-be-interpreted-here\n",
    )
    .map_err(|error| error.to_string())?;
    let environment = parsed.document().container_environment();
    assert!(environment.directives().is_empty());
    assert!(environment.diagnostics().is_empty());
    assert!(environment.is_complete());
    Ok(())
}

#[test]
fn recognized_environment_values_preserve_explicit_access_and_rendering_but_redact_debug() -> Result<(), String> {
    for (unit_type, source, kind, secret) in [
        (
            QuadletUnitType::Container,
            "[Container]\nImage=example.invalid/application\nEnvironment=CONTAINER_SECRET=seeded-container-secret\n",
            EntryKind::Container(ContainerKey::Environment),
            "seeded-container-secret",
        ),
        (
            QuadletUnitType::Build,
            "[Build]\nEnvironment=BUILD_SECRET=seeded-build-secret\n",
            EntryKind::Build(BuildKey::Environment),
            "seeded-build-secret",
        ),
    ] {
        let parsed =
            QuadletDocument::parse(unit_type, SourceId::new(9_503), source).map_err(|error| error.to_string())?;
        let entry = parsed
            .document()
            .entries()
            .find(|entry| entry.kind() == kind)
            .ok_or("missing recognized environment entry")?;
        assert!(entry.is_sensitive());
        assert!(entry.value().primary().text().contains(secret));
        assert_eq!(parsed.syntax().document().render_preserved(), source);
        assert!(!format!("{:#?}", parsed.document()).contains(secret));
        assert!(!format!("{parsed:?}").contains(secret));
    }
    Ok(())
}

#[test]
fn container_environment_reset_remains_a_blank_ordered_source_entry() -> Result<(), String> {
    let source_id = SourceId::new(9_204);
    let result = QuadletDocument::parse(QuadletUnitType::Container, source_id, CONTAINER_ENVIRONMENT_RESETS)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
    assert_eq!(
        result.syntax().document().render_preserved(),
        CONTAINER_ENVIRONMENT_RESETS
    );
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::Environment))
            .map(|entry| {
                let primary = entry.value().primary();
                (primary.text(), primary.span().source_id(), primary.span().len())
            })
            .collect::<Vec<_>>(),
        [
            ("PRE_ONE=one", source_id, "PRE_ONE=one".len()),
            ("PRE_TWO=two", source_id, "PRE_TWO=two".len()),
            ("", source_id, 0),
            ("POST_ONE=one", source_id, "POST_ONE=one".len()),
            ("POST_TWO=two", source_id, "POST_TWO=two".len()),
        ]
    );
    Ok(())
}

#[test]
fn systemd_unit_relationships_are_typed_repeatable_and_lossless() -> Result<(), String> {
    let source = concat!(
        "[Unit]\n",
        "Requires=alpha.container beta.pod\n",
        "Wants=\"quoted.network\"\n",
        "After=continued.volume \\\n",
        "  continued.build\n",
        "Requisite=requisite.image\n",
        "BindsTo=binds.container\n",
        "PartOf=part.pod\n",
        "Upholds=upholds.kube\n",
        "Conflicts=conflicts.artifact\n",
        "Before=malformed\"\n",
        "Description=generic systemd text\n",
        "requires=case-sensitive\n",
        "[Service]\n",
        "Requires=still generic outside Unit\n",
        "[Container]\n",
        "Image=example.invalid/application\n",
    );
    let parsed = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(9_100), source)
        .map_err(|error| error.to_string())?;
    assert!(parsed.is_valid(), "{:#?}", parsed.model_diagnostics());
    assert_eq!(parsed.syntax().document().render_preserved(), source);

    let typed = parsed
        .document()
        .entries()
        .filter_map(|entry| match entry.kind() {
            EntryKind::SystemdUnit(key) => Some((key, entry.value().primary().text(), entry.value().is_continued())),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        typed,
        [
            (SystemdUnitKey::Requires, "alpha.container beta.pod", false),
            (SystemdUnitKey::Wants, "\"quoted.network\"", false),
            (SystemdUnitKey::After, "continued.volume \\", true),
            (SystemdUnitKey::Requisite, "requisite.image", false),
            (SystemdUnitKey::BindsTo, "binds.container", false),
            (SystemdUnitKey::PartOf, "part.pod", false),
            (SystemdUnitKey::Upholds, "upholds.kube", false),
            (SystemdUnitKey::Conflicts, "conflicts.artifact", false),
            (SystemdUnitKey::Before, "malformed\"", false),
        ]
    );
    assert!(
        typed
            .iter()
            .all(|(key, _, _)| EntryKind::SystemdUnit(*key).is_repeatable())
    );
    assert!(
        parsed
            .document()
            .entries()
            .filter(|entry| matches!(entry.kind(), EntryKind::SystemdUnit(_)))
            .all(|entry| entry.value_kind() == ValueKind::Opaque)
    );
    assert_eq!(
        parsed
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::GenericSystemd)
            .map(|entry| entry.key().text())
            .collect::<Vec<_>>(),
        ["Description", "requires", "Requires"]
    );
    Ok(())
}

#[test]
fn pod_exit_policy_is_an_opaque_singleton_without_effective_value_selection() -> Result<(), String> {
    let source = "[Pod]\nExitPolicy=continue\nExitPolicy=\"stop %i\"\nExitPolicy=continued \\\n+value\nExitPolicy=malformed\"\nExitPolicy=\n";
    let result =
        QuadletDocument::parse(QuadletUnitType::Pod, SourceId::new(472), source).map_err(|error| error.to_string())?;
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Pod(PodKey::ExitPolicy))
            .map(|entry| (
                entry.value().primary().text(),
                entry.value().is_continued(),
                entry.value_kind()
            ))
            .collect::<Vec<_>>(),
        [
            ("continue", false, ValueKind::Opaque),
            ("\"stop %i\"", false, ValueKind::Opaque),
            ("continued \\", true, ValueKind::Opaque),
            ("malformed\"", false, ValueKind::Opaque),
            ("", false, ValueKind::Opaque),
        ]
    );
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        ["QLM0004", "QLM0004", "QLM0004", "QLM0004"]
    );
    Ok(())
}

#[test]
fn pod_stop_timeout_is_an_opaque_singleton_without_timeout_interpretation() -> Result<(), String> {
    let source = "[Pod]\nStopTimeout=37\nStopTimeout=\"0 %i\"\nStopTimeout=continued \\\n+value\nStopTimeout=malformed\"\nStopTimeout=\n";
    let result =
        QuadletDocument::parse(QuadletUnitType::Pod, SourceId::new(475), source).map_err(|error| error.to_string())?;
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Pod(PodKey::StopTimeout))
            .map(|entry| (
                entry.value().primary().text(),
                entry.value().is_continued(),
                entry.value_kind()
            ))
            .collect::<Vec<_>>(),
        [
            ("37", false, ValueKind::Opaque),
            ("\"0 %i\"", false, ValueKind::Opaque),
            ("continued \\", true, ValueKind::Opaque),
            ("malformed\"", false, ValueKind::Opaque),
            ("", false, ValueKind::Opaque),
        ]
    );
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        ["QLM0004", "QLM0004", "QLM0004", "QLM0004"]
    );
    Ok(())
}

#[test]
fn pod_service_name_is_an_opaque_singleton_without_identity_interpretation() -> Result<(), String> {
    let source = "[Pod]\nServiceName=ordinary\nServiceName=\"quoted %i\"\nServiceName=continued \\\n+value\nServiceName=malformed\"\nServiceName=\n";
    let result =
        QuadletDocument::parse(QuadletUnitType::Pod, SourceId::new(478), source).map_err(|error| error.to_string())?;
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Pod(PodKey::ServiceName))
            .map(|entry| (
                entry.value().primary().text(),
                entry.value().is_continued(),
                entry.value_kind()
            ))
            .collect::<Vec<_>>(),
        [
            ("ordinary", false, ValueKind::Opaque),
            ("\"quoted %i\"", false, ValueKind::Opaque),
            ("continued \\", true, ValueKind::Opaque),
            ("malformed\"", false, ValueKind::Opaque),
            ("", false, ValueKind::Opaque),
        ]
    );
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        ["QLM0004", "QLM0004", "QLM0004", "QLM0004"]
    );
    Ok(())
}

#[test]
fn container_reload_keys_are_opaque_singletons_and_conflict_without_losing_source() -> Result<(), String> {
    let source = "[Container]\nImage=example.invalid/app\nReloadCmd=first --flag=\"quoted %i\"\nReloadCmd=continued \\\n+value\nReloadSignal=SIGUSR1\nReloadSignal=malformed\"\n";
    let result = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(471), source)
        .map_err(|error| error.to_string())?;
    assert!(!result.is_valid());
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| {
                matches!(
                    entry.kind(),
                    EntryKind::Container(ContainerKey::ReloadCmd | ContainerKey::ReloadSignal)
                )
            })
            .map(|entry| (
                entry.kind(),
                entry.value().primary().text(),
                entry.value().is_continued(),
                entry.value_kind()
            ))
            .collect::<Vec<_>>(),
        [
            (
                EntryKind::Container(ContainerKey::ReloadCmd),
                "first --flag=\"quoted %i\"",
                false,
                ValueKind::Opaque
            ),
            (
                EntryKind::Container(ContainerKey::ReloadCmd),
                "continued \\",
                true,
                ValueKind::Opaque
            ),
            (
                EntryKind::Container(ContainerKey::ReloadSignal),
                "SIGUSR1",
                false,
                ValueKind::Opaque
            ),
            (
                EntryKind::Container(ContainerKey::ReloadSignal),
                "malformed\"",
                false,
                ValueKind::Opaque
            ),
        ]
    );
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        ["QLM0004", "QLM0004", "QLM0010"]
    );
    assert!(result.model_diagnostics().iter().all(|diagnostic| {
        !diagnostic.summary().contains("quoted")
            && !diagnostic
                .labels()
                .iter()
                .any(|label| label.message().contains("quoted"))
    }));
    Ok(())
}

#[test]
fn image_model_preserves_opaque_singletons_and_reports_missing_or_blank_sources() -> Result<(), String> {
    let source = "[Image]\nImage=registry.example/one:1\nImage= \nImage=\"quoted-%i\"\nImage=continued \\\n+ value\n";
    let result = QuadletDocument::parse(QuadletUnitType::Image, SourceId::new(440), source)
        .map_err(|error| error.to_string())?;
    assert!(!result.is_valid());
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Image(ImageKey::Image))
            .map(|entry| (entry.value().primary().text(), entry.value_kind()))
            .collect::<Vec<_>>(),
        [
            ("registry.example/one:1", ValueKind::Opaque),
            ("", ValueKind::Opaque),
            ("\"quoted-%i\"", ValueKind::Opaque),
            ("continued \\", ValueKind::Opaque),
        ]
    );
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        ["QLM0004", "QLM0004", "QLM0004", "QLM0009"]
    );

    let missing = QuadletDocument::parse(QuadletUnitType::Image, SourceId::new(441), "[Image]\n")
        .map_err(|error| error.to_string())?;
    assert_eq!(
        missing
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        ["QLM0008"]
    );
    assert_eq!(QuadletUnitType::from_extension("image"), Some(QuadletUnitType::Image));
    Ok(())
}

#[test]
fn image_tag_is_an_opaque_singleton_without_reference_or_source_validation_semantics() -> Result<(), String> {
    let source = "[Image]\nImage=example.invalid/source:1\nImageTag=first\nImageTag=\nImageTag=\"quoted-%i\"\nImageTag=unmatched\"\nImageTag=continued \\\n+ value\n";
    let result = QuadletDocument::parse(QuadletUnitType::Image, SourceId::new(447), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Image(ImageKey::ImageTag))
            .map(|entry| (
                entry.value().primary().text(),
                entry.value_kind(),
                entry.unit_reference_name()
            ))
            .collect::<Vec<_>>(),
        [
            ("first", ValueKind::Opaque, None),
            ("", ValueKind::Opaque, None),
            ("\"quoted-%i\"", ValueKind::Opaque, None),
            ("unmatched\"", ValueKind::Opaque, None),
            ("continued \\", ValueKind::Opaque, None),
        ]
    );
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        ["QLM0004", "QLM0004", "QLM0004", "QLM0004"]
    );
    Ok(())
}

#[test]
fn image_service_name_is_an_opaque_singleton_without_identity_or_reference_semantics() -> Result<(), String> {
    let source = "[Image]\nImage=example.invalid/source:1\nServiceName=first\nServiceName=\nServiceName=\"quoted-%i\"\nServiceName=unmatched\"\nServiceName=continued \\\n+ value\n";
    let result = QuadletDocument::parse(QuadletUnitType::Image, SourceId::new(451), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Image(ImageKey::ServiceName))
            .map(|entry| (
                entry.value().primary().text(),
                entry.value_kind(),
                entry.unit_reference_name()
            ))
            .collect::<Vec<_>>(),
        [
            ("first", ValueKind::Opaque, None),
            ("", ValueKind::Opaque, None),
            ("\"quoted-%i\"", ValueKind::Opaque, None),
            ("unmatched\"", ValueKind::Opaque, None),
            ("continued \\", ValueKind::Opaque, None),
        ]
    );
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        ["QLM0004", "QLM0004", "QLM0004", "QLM0004"]
    );
    Ok(())
}

#[test]
fn image_all_tags_is_an_opaque_singleton_without_boolean_or_reference_semantics() -> Result<(), String> {
    let source = "[Image]\nImage=example.invalid/source:1\nAllTags=true\nAllTags=\nAllTags= whitespace \nAllTags=\"quoted-%i\"\nAllTags=unmatched\"\nAllTags=continued \\\nvalue\n";
    let result = QuadletDocument::parse(QuadletUnitType::Image, SourceId::new(453), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Image(ImageKey::AllTags))
            .map(|entry| (
                entry.value().primary().text(),
                entry.value_kind(),
                entry.unit_reference_name()
            ))
            .collect::<Vec<_>>(),
        [
            ("true", ValueKind::Opaque, None),
            ("", ValueKind::Opaque, None),
            ("whitespace ", ValueKind::Opaque, None),
            ("\"quoted-%i\"", ValueKind::Opaque, None),
            ("unmatched\"", ValueKind::Opaque, None),
            (r"continued \", ValueKind::Opaque, None),
        ]
    );
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        ["QLM0004", "QLM0004", "QLM0004", "QLM0004", "QLM0004"]
    );
    Ok(())
}

#[test]
fn image_arch_is_an_opaque_singleton_without_platform_or_reference_semantics() -> Result<(), String> {
    let source = "[Image]\nImage=example.invalid/source:1\nArch=arm64\nArch=\nArch= whitespace \nArch=\"quoted-%i\"\nArch=unmatched\"\nArch=continued \\\n+value\n";
    let result = QuadletDocument::parse(QuadletUnitType::Image, SourceId::new(454), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Image(ImageKey::Arch))
            .map(|entry| (
                entry.value().primary().text(),
                entry.value_kind(),
                entry.unit_reference_name()
            ))
            .collect::<Vec<_>>(),
        [
            ("arm64", ValueKind::Opaque, None),
            ("", ValueKind::Opaque, None),
            ("whitespace ", ValueKind::Opaque, None),
            ("\"quoted-%i\"", ValueKind::Opaque, None),
            ("unmatched\"", ValueKind::Opaque, None),
            (r"continued \", ValueKind::Opaque, None),
        ]
    );
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        ["QLM0004", "QLM0004", "QLM0004", "QLM0004", "QLM0004"]
    );
    Ok(())
}

#[test]
fn image_auth_file_is_an_opaque_singleton_without_path_or_credential_semantics() -> Result<(), String> {
    let source = "[Image]\nImage=example.invalid/source:1\nAuthFile=/placeholder/quadlet-lens-auth.json\nAuthFile=\nAuthFile= whitespace \nAuthFile=\"quoted-%i\"\nAuthFile=unmatched\"\nAuthFile=continued \\\n+value\nAuthFile=%h/placeholder-auth.json\nAuthFile=arbitrary text\n";
    let result = QuadletDocument::parse(QuadletUnitType::Image, SourceId::new(455), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Image(ImageKey::AuthFile))
            .map(|entry| (
                entry.value().primary().text(),
                entry.value_kind(),
                entry.unit_reference_name()
            ))
            .collect::<Vec<_>>(),
        [
            ("/placeholder/quadlet-lens-auth.json", ValueKind::Opaque, None),
            ("", ValueKind::Opaque, None),
            ("whitespace ", ValueKind::Opaque, None),
            ("\"quoted-%i\"", ValueKind::Opaque, None),
            ("unmatched\"", ValueKind::Opaque, None),
            (r"continued \", ValueKind::Opaque, None),
            ("%h/placeholder-auth.json", ValueKind::Opaque, None),
            ("arbitrary text", ValueKind::Opaque, None),
        ]
    );
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        [
            "QLM0004", "QLM0004", "QLM0004", "QLM0004", "QLM0004", "QLM0004", "QLM0004"
        ]
    );
    Ok(())
}

#[test]
fn image_cert_dir_is_an_opaque_singleton_without_path_or_certificate_semantics() -> Result<(), String> {
    let source = "[Image]\nImage=example.invalid/source:1\nCertDir=/placeholder/quadlet-lens-certs\nCertDir=\nCertDir= whitespace \nCertDir=\"quoted-%i\"\nCertDir=unmatched\"\nCertDir=continued \\\n+value\nCertDir=%h/placeholder-certs\nCertDir=escaped\\ text\nCertDir=arbitrary path-looking text\n";
    let result = QuadletDocument::parse(QuadletUnitType::Image, SourceId::new(456), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Image(ImageKey::CertDir))
            .map(|entry| (
                entry.value().primary().text(),
                entry.value_kind(),
                entry.unit_reference_name()
            ))
            .collect::<Vec<_>>(),
        [
            ("/placeholder/quadlet-lens-certs", ValueKind::Opaque, None),
            ("", ValueKind::Opaque, None),
            ("whitespace ", ValueKind::Opaque, None),
            ("\"quoted-%i\"", ValueKind::Opaque, None),
            ("unmatched\"", ValueKind::Opaque, None),
            (r"continued \", ValueKind::Opaque, None),
            ("%h/placeholder-certs", ValueKind::Opaque, None),
            ("escaped\\ text", ValueKind::Opaque, None),
            ("arbitrary path-looking text", ValueKind::Opaque, None),
        ]
    );
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        [
            "QLM0004", "QLM0004", "QLM0004", "QLM0004", "QLM0004", "QLM0004", "QLM0004", "QLM0004"
        ]
    );
    Ok(())
}

#[test]
fn image_containers_conf_module_preserves_every_opaque_physical_value_without_module_semantics() -> Result<(), String> {
    let source = concat!(
        "[Image]\nImage=example.invalid/source:1\n",
        "ContainersConfModule=pre-one\nContainersConfModule=\n",
        "ContainersConfModule= post one \nContainersConfModule=post-two\n",
        "ContainersConfModule=post-two\nContainersConfModule=\"quoted %h module\"\n",
        "ContainersConfModule=module\\x20text\nContainersConfModule=-leading-dash\n",
        "ContainersConfModule=continued \\\n+module\n",
    );
    let result = QuadletDocument::parse(QuadletUnitType::Image, SourceId::new(457), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Image(ImageKey::ContainersConfModule))
            .map(|entry| (entry.value().primary().text(), entry.value_kind()))
            .collect::<Vec<_>>(),
        [
            ("pre-one", ValueKind::Opaque),
            ("", ValueKind::Opaque),
            ("post one ", ValueKind::Opaque),
            ("post-two", ValueKind::Opaque),
            ("post-two", ValueKind::Opaque),
            ("\"quoted %h module\"", ValueKind::Opaque),
            (r"module\x20text", ValueKind::Opaque),
            ("-leading-dash", ValueKind::Opaque),
            (r"continued \", ValueKind::Opaque),
        ]
    );
    assert!(
        result
            .document()
            .entries()
            .any(|entry| entry.kind() == EntryKind::Image(ImageKey::ContainersConfModule)
                && entry.value().is_continued())
    );
    assert!(result.model_diagnostics().is_empty());
    Ok(())
}

#[test]
fn image_creds_preserves_opaque_physical_values_and_redacts_only_credential_debug_output() -> Result<(), String> {
    const PLACEHOLDER: &str = "quadlet-lens-creds-debug-placeholder-7e9c:opaque-password";
    let source = concat!(
        "[Image]\nImage=example.invalid/source:1\n",
        "Creds=quadlet-lens-creds-debug-placeholder-7e9c:opaque-password\n",
        "Creds=\"quadlet-lens-creds-debug-placeholder-7e9c:opaque-password\"\n",
        "Creds=%h/quadlet-lens-creds-debug-placeholder-7e9c\n",
        "Creds=continued \\\n+quadlet-lens-creds-debug-placeholder-7e9c:opaque-password\n",
    );
    let result = QuadletDocument::parse(QuadletUnitType::Image, SourceId::new(459), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
    assert_eq!(result.syntax().document().render_preserved(), source);
    let entries: Vec<_> = result
        .document()
        .entries()
        .filter(|entry| entry.kind() == EntryKind::Image(ImageKey::Creds))
        .collect();
    assert_eq!(entries.len(), 4);
    assert!(entries.iter().all(|entry| entry.is_sensitive()));
    assert_eq!(entries[0].value().primary().text(), PLACEHOLDER);
    assert_eq!(entries[1].value().primary().text(), format!("\"{PLACEHOLDER}\""));
    assert_eq!(
        entries[2].value().primary().text(),
        "%h/quadlet-lens-creds-debug-placeholder-7e9c"
    );
    assert_eq!(entries[3].value().continuations()[0].text(), format!("+{PLACEHOLDER}"));
    assert!(entries[3].value().is_continued());
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        ["QLM0004", "QLM0004", "QLM0004"]
    );
    for debug in [
        format!("{:#?}", entries[0]),
        format!("{:#?}", entries[0].value()),
        format!("{:#?}", entries[0].value().primary()),
        format!("{:#?}", entries[3].value().continuations()),
        format!("{:#?}", result.document()),
        format!("{result:#?}"),
        format!("{:#?}", result.model_diagnostics()),
    ] {
        assert!(
            !debug.contains(PLACEHOLDER),
            "credential leaked in debug output: {debug}"
        );
    }
    let ordinary = QuadletDocument::parse(
        QuadletUnitType::Image,
        SourceId::new(460),
        "[Image]\nImage=ordinary-debug-value\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(ordinary.document().entries().all(|entry| !entry.is_sensitive()));
    assert!(format!("{:#?}", ordinary.document()).contains("ordinary-debug-value"));
    assert!(format!("{ordinary:#?}").contains("ordinary-debug-value"));
    Ok(())
}

#[test]
fn image_decryption_key_preserves_opaque_physical_values_and_redacts_key_debug_output() -> Result<(), String> {
    const PLACEHOLDER: &str = "quadlet-lens-decryption-key-debug-placeholder-7e9c";
    let source = concat!(
        "[Image]\nImage=example.invalid/source:1\n",
        "DecryptionKey=quadlet-lens-decryption-key-debug-placeholder-7e9c\n",
        "DecryptionKey=\"quadlet-lens-decryption-key-debug-placeholder-7e9c\"\n",
        "DecryptionKey=%h/quadlet-lens-decryption-key-debug-placeholder-7e9c\n",
        "DecryptionKey=continued \\\n+quadlet-lens-decryption-key-debug-placeholder-7e9c\n",
    );
    let result = QuadletDocument::parse(QuadletUnitType::Image, SourceId::new(461), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
    assert_eq!(result.syntax().document().render_preserved(), source);
    let entries: Vec<_> = result
        .document()
        .entries()
        .filter(|entry| entry.kind() == EntryKind::Image(ImageKey::DecryptionKey))
        .collect();
    assert_eq!(entries.len(), 4);
    assert!(entries.iter().all(|entry| entry.is_sensitive()));
    assert_eq!(entries[0].value().primary().text(), PLACEHOLDER);
    assert_eq!(entries[1].value().primary().text(), format!("\"{PLACEHOLDER}\""));
    assert_eq!(
        entries[2].value().primary().text(),
        "%h/quadlet-lens-decryption-key-debug-placeholder-7e9c"
    );
    assert_eq!(entries[3].value().continuations()[0].text(), format!("+{PLACEHOLDER}"));
    assert!(entries[3].value().is_continued());
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        ["QLM0004", "QLM0004", "QLM0004"]
    );
    for debug in [
        format!("{:#?}", entries[0]),
        format!("{:#?}", entries[0].value()),
        format!("{:#?}", entries[0].value().primary()),
        format!("{:#?}", entries[3].value().continuations()),
        format!("{:#?}", result.document()),
        format!("{result:#?}"),
        format!("{:#?}", result.syntax().diagnostics()),
        format!("{:#?}", result.model_diagnostics()),
    ] {
        assert!(
            !debug.contains(PLACEHOLDER),
            "decryption key leaked in debug output: {debug}"
        );
    }
    Ok(())
}

#[test]
fn image_global_args_remain_repeatable_opaque_physical_lines() -> Result<(), String> {
    let source = concat!(
        "[Image]\nImage=example.invalid/source:1\n",
        "GlobalArgs=pre-one\nGlobalArgs=pre-one\nGlobalArgs=\n",
        "GlobalArgs= --log-level=debug \nGlobalArgs=\"--events-backend=none\"\n",
        "GlobalArgs=--events-backend\\x3dfile\\x20value\n",
        "GlobalArgs=continued \\\ntext\nGlobalArgs=malformed \\ value\nGlobalArgs=%h/bin\n",
    );
    let result = QuadletDocument::parse(QuadletUnitType::Image, SourceId::new(462), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Image(ImageKey::GlobalArgs))
            .map(|entry| (entry.value().primary().text(), entry.value_kind(), entry.is_sensitive()))
            .collect::<Vec<_>>(),
        [
            ("pre-one", ValueKind::Opaque, false),
            ("pre-one", ValueKind::Opaque, false),
            ("", ValueKind::Opaque, false),
            ("--log-level=debug ", ValueKind::Opaque, false),
            ("\"--events-backend=none\"", ValueKind::Opaque, false),
            ("--events-backend\\x3dfile\\x20value", ValueKind::Opaque, false),
            ("continued \\", ValueKind::Opaque, false),
            ("malformed \\ value", ValueKind::Opaque, false),
            ("%h/bin", ValueKind::Opaque, false),
        ]
    );
    assert!(result.document().entries().any(|entry| {
        entry.kind() == EntryKind::Image(ImageKey::GlobalArgs)
            && entry.value().is_continued()
            && entry.value().continuations()[0].text() == "text"
    }));
    assert!(result.model_diagnostics().is_empty());
    Ok(())
}

#[test]
fn image_os_is_an_opaque_singleton_without_platform_or_effective_value_semantics() -> Result<(), String> {
    let source = "[Image]\nImage=example.invalid/source:1\nOS=windows\nOS=\nOS=\"quoted-%i\"\nOS=unmatched\"\nOS=continued \\\nvalue\nOS=%h/os\n";
    let result = QuadletDocument::parse(QuadletUnitType::Image, SourceId::new(463), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Image(ImageKey::OS))
            .map(|entry| (entry.value().primary().text(), entry.value_kind(), entry.is_sensitive()))
            .collect::<Vec<_>>(),
        [
            ("windows", ValueKind::Opaque, false),
            ("", ValueKind::Opaque, false),
            ("\"quoted-%i\"", ValueKind::Opaque, false),
            ("unmatched\"", ValueKind::Opaque, false),
            ("continued \\", ValueKind::Opaque, false),
            ("%h/os", ValueKind::Opaque, false),
        ]
    );
    assert!(result.document().entries().any(|entry| {
        entry.kind() == EntryKind::Image(ImageKey::OS)
            && entry.value().is_continued()
            && entry.value().continuations()[0].text() == "value"
    }));
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        ["QLM0004", "QLM0004", "QLM0004", "QLM0004", "QLM0004"]
    );
    Ok(())
}

#[test]
fn build_podman_args_remain_repeatable_opaque_physical_lines() -> Result<(), String> {
    let result = QuadletDocument::parse(QuadletUnitType::Build, SourceId::new(189), BUILD_PODMAN_ARGS)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
    let entries: Vec<_> = result
        .document()
        .entries()
        .filter(|entry| entry.kind() == EntryKind::Build(BuildKey::PodmanArgs))
        .map(|entry| (entry.value().primary().text(), entry.value_kind()))
        .collect();
    assert_eq!(
        entries,
        [
            ("--build-context extra=container-image://alpine:3.15", ValueKind::Opaque),
            ("--layers", ValueKind::Opaque),
        ]
    );
    Ok(())
}

#[test]
fn build_model_retains_repeatable_image_tags_files_and_opaque_working_directory() -> Result<(), String> {
    let result =
        QuadletDocument::parse(QuadletUnitType::Build, SourceId::new(180), BUILD).map_err(|error| error.to_string())?;
    assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
    assert_eq!(result.syntax().document().render_preserved(), BUILD);
    assert_eq!(
        result
            .document()
            .sections()
            .iter()
            .map(quadlet_lens::model::TypedSection::kind)
            .collect::<Vec<_>>(),
        [SectionKind::Unit, SectionKind::Build, SectionKind::Service]
    );
    let entries: Vec<_> = result
        .document()
        .entries()
        .filter_map(|entry| match entry.kind() {
            EntryKind::Build(key) => Some((key, entry.value().primary().text(), entry.value_kind())),
            _ => None,
        })
        .collect();
    assert_eq!(
        entries,
        [
            (BuildKey::ImageTag, "localhost/example:primary", ValueKind::Opaque),
            (BuildKey::ImageTag, "localhost/example:secondary", ValueKind::Opaque),
            (BuildKey::Network, "host", ValueKind::Opaque),
            (BuildKey::Network, "none", ValueKind::Opaque),
            (
                BuildKey::Network,
                "frontend.network",
                ValueKind::UnitReference(UnitReferenceKind::Network)
            ),
            (BuildKey::Label, "build.label=one", ValueKind::Opaque),
            (BuildKey::Label, "empty=", ValueKind::Opaque),
            (BuildKey::BuildArg, "KEY=one", ValueKind::Opaque),
            (BuildKey::BuildArg, "EMPTY=", ValueKind::Opaque),
            (BuildKey::BuildArg, "bare text stays opaque", ValueKind::Opaque),
            (
                BuildKey::Secret,
                "id=quadlet-lens-one,src=/run/quadlet-lens-placeholder-one",
                ValueKind::Opaque
            ),
            (
                BuildKey::Secret,
                "id=quadlet-lens-two,src=/run/quadlet-lens-placeholder-two",
                ValueKind::Opaque
            ),
            (BuildKey::File, "Containerfile.first", ValueKind::Opaque),
            (BuildKey::File, "", ValueKind::Opaque),
            (
                BuildKey::File,
                "https://example.invalid/Containerfile?ref=main",
                ValueKind::Opaque
            ),
            (BuildKey::SetWorkingDirectory, "unit", ValueKind::Opaque),
            (BuildKey::Pull, "", ValueKind::Opaque),
            (BuildKey::Pull, "always", ValueKind::Opaque),
        ]
    );
    assert!(
        result
            .document()
            .entries()
            .any(|entry| { entry.kind() == EntryKind::Unknown && entry.key().text() == "FutureBuildKey" })
    );
    Ok(())
}

#[test]
fn build_pull_remains_an_opaque_singleton_with_physical_duplicates() -> Result<(), String> {
    let result = QuadletDocument::parse(
        QuadletUnitType::Build,
        SourceId::new(188),
        "[Build]\nPull=\nPull=always\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(result.is_valid());
    let pulls: Vec<_> = result
        .document()
        .entries()
        .filter(|entry| entry.kind() == EntryKind::Build(BuildKey::Pull))
        .map(|entry| entry.value().primary().text())
        .collect();
    assert_eq!(pulls, ["", "always"]);
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .filter(|diagnostic| diagnostic.code().as_str() == "QLM0004")
            .count(),
        1
    );
    Ok(())
}

#[test]
fn build_retry_tls_verify_and_force_rm_values_remain_opaque_singletons_with_physical_duplicates() -> Result<(), String>
{
    let result = QuadletDocument::parse(
        QuadletUnitType::Build,
        SourceId::new(190),
        "[Build]\nRetry=\nRetry=4\nRetryDelay=\nRetryDelay=7s\nTLSVerify=\nTLSVerify=true\nForceRM=\nForceRM=true\nAuthFile=/run/quadlet-lens/first.json\nAuthFile=\nAuthFile=/run/quadlet-lens/final.json\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(result.is_valid());
    let retries: Vec<_> = result
        .document()
        .entries()
        .filter_map(|entry| match entry.kind() {
            EntryKind::Build(
                BuildKey::Retry | BuildKey::RetryDelay | BuildKey::TLSVerify | BuildKey::ForceRM | BuildKey::AuthFile,
            ) => Some((entry.kind(), entry.value().primary().text(), entry.value_kind())),
            _ => None,
        })
        .collect();
    assert_eq!(
        retries,
        [
            (EntryKind::Build(BuildKey::Retry), "", ValueKind::Opaque),
            (EntryKind::Build(BuildKey::Retry), "4", ValueKind::Opaque),
            (EntryKind::Build(BuildKey::RetryDelay), "", ValueKind::Opaque),
            (EntryKind::Build(BuildKey::RetryDelay), "7s", ValueKind::Opaque),
            (EntryKind::Build(BuildKey::TLSVerify), "", ValueKind::Opaque),
            (EntryKind::Build(BuildKey::TLSVerify), "true", ValueKind::Opaque),
            (EntryKind::Build(BuildKey::ForceRM), "", ValueKind::Opaque),
            (EntryKind::Build(BuildKey::ForceRM), "true", ValueKind::Opaque),
            (
                EntryKind::Build(BuildKey::AuthFile),
                "/run/quadlet-lens/first.json",
                ValueKind::Opaque
            ),
            (EntryKind::Build(BuildKey::AuthFile), "", ValueKind::Opaque),
            (
                EntryKind::Build(BuildKey::AuthFile),
                "/run/quadlet-lens/final.json",
                ValueKind::Opaque
            ),
        ]
    );
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .filter(|diagnostic| diagnostic.code().as_str() == "QLM0004")
            .count(),
        6
    );
    Ok(())
}

#[test]
fn build_ignore_file_preserves_singleton_physical_lines_without_path_interpretation() -> Result<(), String> {
    let source = concat!(
        "[Build]\n",
        "IgnoreFile=./first.ignore\n",
        "IgnoreFile=\n",
        "IgnoreFile=%h/final.ignore\n",
    );
    let result = QuadletDocument::parse(QuadletUnitType::Build, SourceId::new(195), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid());
    assert_eq!(result.syntax().document().render_preserved(), source);
    let values: Vec<_> = result
        .document()
        .entries()
        .filter(|entry| entry.kind() == EntryKind::Build(BuildKey::IgnoreFile))
        .map(|entry| (entry.value().primary().text(), entry.value_kind()))
        .collect();
    assert_eq!(
        values,
        [
            ("./first.ignore", ValueKind::Opaque),
            ("", ValueKind::Opaque),
            ("%h/final.ignore", ValueKind::Opaque),
        ]
    );
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        ["QLM0004", "QLM0004"]
    );
    Ok(())
}

#[test]
fn build_ignore_file_is_unknown_and_preserved_outside_build() -> Result<(), String> {
    let source = "[Container]\nImage=example.invalid/app\nIgnoreFile=./container.ignore\n";
    let result = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(196), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid());
    let entry = result.document().entries().nth(1).ok_or("missing entry")?;
    assert_eq!(entry.kind(), EntryKind::Unknown);
    assert_eq!(entry.value().primary().text(), "./container.ignore");
    assert!(result.model_diagnostics().is_empty());
    Ok(())
}

#[test]
fn build_service_name_preserves_opaque_singleton_physical_lines_and_scope() -> Result<(), String> {
    let source = concat!(
        "[Build]\n",
        "ServiceName=first.service\n",
        "ServiceName=\n",
        "ServiceName= second service \n",
        "ServiceName=\"quoted-%i\"\n",
        "ServiceName=%h/service\n",
        "ServiceName=continuation\\-looking\n",
    );
    let result = QuadletDocument::parse(QuadletUnitType::Build, SourceId::new(196), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid());
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Build(BuildKey::ServiceName))
            .map(|entry| (entry.value().primary().text(), entry.value_kind()))
            .collect::<Vec<_>>(),
        [
            ("first.service", ValueKind::Opaque),
            ("", ValueKind::Opaque),
            ("second service ", ValueKind::Opaque),
            ("\"quoted-%i\"", ValueKind::Opaque),
            ("%h/service", ValueKind::Opaque),
            ("continuation\\-looking", ValueKind::Opaque),
        ]
    );
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .filter(|diagnostic| diagnostic.code().as_str() == "QLM0004")
            .count(),
        5
    );

    let wrong_section = QuadletDocument::parse(
        QuadletUnitType::Container,
        SourceId::new(197),
        "[Container]\nImage=example.invalid/app\nServiceName=container.service\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(
        wrong_section
            .document()
            .entries()
            .any(|entry| entry.kind() == EntryKind::Container(ContainerKey::ServiceName))
    );
    Ok(())
}

#[test]
fn build_volume_classifies_only_the_source_prefix_and_preserves_raw_values() -> Result<(), String> {
    let source = concat!(
        "[Build]\n",
        "Volume=cache.volume:/var/cache:Z\n",
        "Volume=.:/workspace\n",
        "Volume=/host/data:/data:ro\n",
        "Volume=destination-only\n",
        "Volume=\n",
        "Volume=\"quoted.volume\":/quoted\n",
        "Volume=%h/data:/home\n",
        "Volume=relative/path:/relative\n",
        "Volume=cache.volume:/var/cache:Z\n",
    );
    let result = QuadletDocument::parse(QuadletUnitType::Build, SourceId::new(218), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid());
    assert_eq!(result.syntax().document().render_preserved(), source);
    let volumes: Vec<_> = result
        .document()
        .entries()
        .filter(|entry| entry.kind() == EntryKind::Build(BuildKey::Volume))
        .map(|entry| {
            (
                entry.value().primary().text(),
                entry.value_kind(),
                entry.unit_reference_name(),
            )
        })
        .collect();
    assert_eq!(
        volumes,
        [
            (
                "cache.volume:/var/cache:Z",
                ValueKind::UnitReference(UnitReferenceKind::Volume),
                Some("cache.volume")
            ),
            (".:/workspace", ValueKind::Path(PathForm::UnitRelativeLiteral), None),
            ("/host/data:/data:ro", ValueKind::Path(PathForm::AbsoluteLiteral), None),
            ("destination-only", ValueKind::Path(PathForm::RelativeLiteral), None),
            ("", ValueKind::Path(PathForm::RelativeLiteral), None),
            (
                "\"quoted.volume\":/quoted",
                ValueKind::Path(PathForm::RelativeLiteral),
                None
            ),
            ("%h/data:/home", ValueKind::Path(PathForm::SystemdSpecifier), None),
            (
                "relative/path:/relative",
                ValueKind::Path(PathForm::RelativeLiteral),
                None
            ),
            (
                "cache.volume:/var/cache:Z",
                ValueKind::UnitReference(UnitReferenceKind::Volume),
                Some("cache.volume")
            ),
        ]
    );
    Ok(())
}

#[test]
fn build_annotation_preserves_every_opaque_physical_line_in_source_order() -> Result<(), String> {
    let source = concat!(
        "[Build]\n",
        "Annotation=org.example.pre=one\n",
        "Annotation=\n",
        "Annotation=org.example.name=first\n",
        "Annotation=org.example.name=final\n",
        "Annotation=\"org.example.quoted=Authored Value\"\n",
        "Annotation=org.example.specifier=%i\n",
        "Annotation=org.example.escape=literal\\x20text\n",
        "Annotation=key-only\n",
        "Annotation= malformed = value \n",
    );
    let result = QuadletDocument::parse(QuadletUnitType::Build, SourceId::new(197), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid());
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Build(BuildKey::Annotation))
            .map(|entry| (entry.value().primary().text(), entry.value_kind()))
            .collect::<Vec<_>>(),
        [
            ("org.example.pre=one", ValueKind::Opaque),
            ("", ValueKind::Opaque),
            ("org.example.name=first", ValueKind::Opaque),
            ("org.example.name=final", ValueKind::Opaque),
            ("\"org.example.quoted=Authored Value\"", ValueKind::Opaque),
            ("org.example.specifier=%i", ValueKind::Opaque),
            ("org.example.escape=literal\\x20text", ValueKind::Opaque),
            ("key-only", ValueKind::Opaque),
            ("malformed = value ", ValueKind::Opaque),
        ]
    );
    assert!(result.model_diagnostics().is_empty());
    Ok(())
}

#[test]
fn build_annotation_is_unknown_and_preserved_outside_build() -> Result<(), String> {
    let source = "[Pod]\nPodName=example\nAnnotation=org.example.pod=one\n";
    let result =
        QuadletDocument::parse(QuadletUnitType::Pod, SourceId::new(198), source).map_err(|error| error.to_string())?;
    assert!(result.is_valid());
    let entry = result.document().entries().nth(1).ok_or("missing entry")?;
    assert_eq!(entry.kind(), EntryKind::Unknown);
    assert_eq!(entry.value().primary().text(), "org.example.pod=one");
    assert!(result.model_diagnostics().is_empty());
    Ok(())
}

#[test]
fn build_environment_preserves_opaque_physical_lines_and_wrong_section() -> Result<(), String> {
    let source = concat!(
        "[Build]\nEnvironment=PRE=one\nEnvironment=\nEnvironment=NAME=first\n",
        "Environment=NAME=final\nEnvironment=bare\nEnvironment=\"QUOTED=Authored Value\"\n",
        "Environment=ESCAPED=literal\\x20text\nEnvironment=embedded=a=b\n",
    );
    let result = QuadletDocument::parse(QuadletUnitType::Build, SourceId::new(199), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid());
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Build(BuildKey::Environment))
            .map(|entry| entry.value().primary().text())
            .collect::<Vec<_>>(),
        [
            "PRE=one",
            "",
            "NAME=first",
            "NAME=final",
            "bare",
            "\"QUOTED=Authored Value\"",
            "ESCAPED=literal\\x20text",
            "embedded=a=b"
        ]
    );
    let wrong = QuadletDocument::parse(
        QuadletUnitType::Pod,
        SourceId::new(200),
        "[Pod]\nEnvironment=NOPE=one\n",
    )
    .map_err(|error| error.to_string())?;
    assert_eq!(
        wrong.document().entries().next().ok_or("missing")?.kind(),
        EntryKind::Unknown
    );
    Ok(())
}

#[test]
fn build_containers_conf_module_preserves_opaque_physical_lines_and_wrong_section() -> Result<(), String> {
    let source = concat!(
        "[Build]\nContainersConfModule=pre-one\nContainersConfModule=\n",
        "ContainersConfModule= post one \nContainersConfModule=post-two\n",
        "ContainersConfModule=post-two\nContainersConfModule=\"quoted module\"\n",
        "ContainersConfModule=module\\x20text\nContainersConfModule=continuation\\-looking\n",
        "ContainersConfModule=%i.conf\n",
    );
    let result = QuadletDocument::parse(QuadletUnitType::Build, SourceId::new(205), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid());
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Build(BuildKey::ContainersConfModule))
            .map(|entry| entry.value().primary().text())
            .collect::<Vec<_>>(),
        [
            "pre-one",
            "",
            "post one ",
            "post-two",
            "post-two",
            "\"quoted module\"",
            "module\\x20text",
            "continuation\\-looking",
            "%i.conf",
        ]
    );
    let wrong = QuadletDocument::parse(
        QuadletUnitType::Pod,
        SourceId::new(206),
        "[Pod]\nContainersConfModule=pod.conf\n",
    )
    .map_err(|error| error.to_string())?;
    assert_eq!(
        wrong.document().entries().next().ok_or("missing")?.kind(),
        EntryKind::Pod(PodKey::ContainersConfModule)
    );
    Ok(())
}

#[test]
fn build_global_args_preserves_exact_repeatable_physical_lines_and_wrong_section() -> Result<(), String> {
    let source = concat!(
        "[Build]\nGlobalArgs=--events-backend=none\nGlobalArgs=--events-backend=none\n",
        "GlobalArgs=\nGlobalArgs= --log-level=debug \nGlobalArgs=\"--transient\"\n",
        "GlobalArgs=--events-backend\\x3dfile\nGlobalArgs=continuation\\-looking\n",
        "GlobalArgs= malformed = value \nGlobalArgs=\\\n",
    );
    let result = QuadletDocument::parse(QuadletUnitType::Build, SourceId::new(209), source)
        .map_err(|error| error.to_string())?;
    assert!(
        !result.is_valid(),
        "the dangling continuation-looking physical line remains source-preserved with a syntax diagnostic"
    );
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Build(BuildKey::GlobalArgs))
            .map(|entry| (entry.value().primary().text(), entry.value_kind()))
            .collect::<Vec<_>>(),
        [
            ("--events-backend=none", ValueKind::Opaque),
            ("--events-backend=none", ValueKind::Opaque),
            ("", ValueKind::Opaque),
            ("--log-level=debug ", ValueKind::Opaque),
            ("\"--transient\"", ValueKind::Opaque),
            ("--events-backend\\x3dfile", ValueKind::Opaque),
            ("continuation\\-looking", ValueKind::Opaque),
            ("malformed = value ", ValueKind::Opaque),
            ("\\", ValueKind::Opaque),
        ]
    );
    let wrong = QuadletDocument::parse(
        QuadletUnitType::Pod,
        SourceId::new(210),
        "[Pod]\nGlobalArgs=--log-level=debug\n",
    )
    .map_err(|error| error.to_string())?;
    assert_eq!(
        wrong.document().entries().next().ok_or("missing")?.kind(),
        EntryKind::Pod(PodKey::GlobalArgs)
    );
    Ok(())
}

#[test]
fn build_group_add_values_remain_repeatable_opaque_physical_lines_in_source_order() -> Result<(), String> {
    let result = QuadletDocument::parse(
        QuadletUnitType::Build,
        SourceId::new(191),
        "[Build]\nGroupAdd=1234\nGroupAdd=5678\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(result.is_valid());
    let groups: Vec<_> = result
        .document()
        .entries()
        .filter(|entry| entry.kind() == EntryKind::Build(BuildKey::GroupAdd))
        .map(|entry| (entry.value().primary().text(), entry.value_kind()))
        .collect();
    assert_eq!(groups, [("1234", ValueKind::Opaque), ("5678", ValueKind::Opaque)]);
    assert!(result.model_diagnostics().is_empty());
    Ok(())
}

#[test]
fn build_dns_values_remain_repeatable_opaque_physical_lines_in_source_order() -> Result<(), String> {
    let result = QuadletDocument::parse(
        QuadletUnitType::Build,
        SourceId::new(192),
        "[Build]\nDNS=9.9.9.9\nDNS=2001:4860:4860::8888\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(result.is_valid());
    let servers: Vec<_> = result
        .document()
        .entries()
        .filter(|entry| entry.kind() == EntryKind::Build(BuildKey::DNS))
        .map(|entry| (entry.value().primary().text(), entry.value_kind()))
        .collect();
    assert_eq!(
        servers,
        [
            ("9.9.9.9", ValueKind::Opaque),
            ("2001:4860:4860::8888", ValueKind::Opaque)
        ]
    );
    assert!(result.model_diagnostics().is_empty());
    Ok(())
}

#[test]
fn build_dns_option_values_preserve_empty_entries_and_source_order() -> Result<(), String> {
    let result = QuadletDocument::parse(
        QuadletUnitType::Build,
        SourceId::new(193),
        "[Build]\nDNSOption=rotate\nDNSOption=\nDNSOption=ndots:1\nDNSOption=use-vc\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(result.is_valid());
    let options: Vec<_> = result
        .document()
        .entries()
        .filter(|entry| entry.kind() == EntryKind::Build(BuildKey::DNSOption))
        .map(|entry| (entry.value().primary().text(), entry.value_kind()))
        .collect();
    assert_eq!(
        options,
        [
            ("rotate", ValueKind::Opaque),
            ("", ValueKind::Opaque),
            ("ndots:1", ValueKind::Opaque),
            ("use-vc", ValueKind::Opaque)
        ]
    );
    assert!(result.model_diagnostics().is_empty());
    Ok(())
}

#[test]
fn build_dns_search_values_remain_repeatable_opaque_physical_lines() -> Result<(), String> {
    let result = QuadletDocument::parse(
        QuadletUnitType::Build,
        SourceId::new(194),
        "[Build]\nDNSSearch=old.example\nDNSSearch=\nDNSSearch=corp.example\nDNSSearch=.\n",
    )
    .map_err(|error| error.to_string())?;
    let values: Vec<_> = result
        .document()
        .entries()
        .filter(|entry| entry.kind() == EntryKind::Build(BuildKey::DNSSearch))
        .map(|entry| entry.value().primary().text())
        .collect();
    assert_eq!(values, ["old.example", "", "corp.example", "."]);
    assert!(result.model_diagnostics().is_empty());
    Ok(())
}

#[test]
fn build_args_remain_repeatable_opaque_physical_values() -> Result<(), String> {
    let source = concat!(
        "[Build]\n",
        "BuildArg=KEY=one\n",
        "BuildArg=EMPTY=\n",
        "BuildArg=bare text stays opaque\n",
        "BuildArg=\"QUOTED=%h value\"\n",
    );
    let result = QuadletDocument::parse(QuadletUnitType::Build, SourceId::new(186), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
    assert_eq!(result.syntax().document().render_preserved(), source);
    let build_args: Vec<_> = result
        .document()
        .entries()
        .filter(|entry| entry.kind() == EntryKind::Build(BuildKey::BuildArg))
        .map(|entry| {
            assert_eq!(entry.value_kind(), ValueKind::Opaque);
            entry.value().primary().text()
        })
        .collect();
    assert_eq!(
        build_args,
        ["KEY=one", "EMPTY=", "bare text stays opaque", "\"QUOTED=%h value\""]
    );
    Ok(())
}

#[test]
fn build_secrets_remain_repeatable_opaque_physical_values() -> Result<(), String> {
    let source = concat!(
        "[Build]\n",
        "Secret=id=quadlet-lens-one,src=/run/quadlet-lens-placeholder-one\n",
        "Secret=id=quadlet-lens-two,src=/run/quadlet-lens-placeholder-two\n",
    );
    let result = QuadletDocument::parse(QuadletUnitType::Build, SourceId::new(187), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
    assert_eq!(result.syntax().document().render_preserved(), source);
    let secrets: Vec<_> = result
        .document()
        .entries()
        .filter(|entry| entry.kind() == EntryKind::Build(BuildKey::Secret))
        .map(|entry| {
            assert_eq!(entry.value_kind(), ValueKind::Opaque);
            entry.value().primary().text()
        })
        .collect();
    assert_eq!(
        secrets,
        [
            "id=quadlet-lens-one,src=/run/quadlet-lens-placeholder-one",
            "id=quadlet-lens-two,src=/run/quadlet-lens-placeholder-two",
        ]
    );
    Ok(())
}

#[test]
fn build_labels_preserve_physical_lines_without_label_interpretation() -> Result<(), String> {
    let source = concat!(
        "[Build]\n",
        "Label=build.label=one\n",
        "Label=bare-label\n",
        "Label=build.label=one\n",
        "Label=empty=\n",
        "Label=embedded=a=b\n",
        "Label=\"quoted=%h value\"\n",
    );
    let result = QuadletDocument::parse(QuadletUnitType::Build, SourceId::new(185), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
    assert_eq!(result.syntax().document().render_preserved(), source);
    let labels: Vec<_> = result
        .document()
        .entries()
        .filter(|entry| entry.kind() == EntryKind::Build(BuildKey::Label))
        .map(|entry| {
            assert_eq!(entry.value_kind(), ValueKind::Opaque);
            entry.value().primary().text()
        })
        .collect();
    assert_eq!(
        labels,
        [
            "build.label=one",
            "bare-label",
            "build.label=one",
            "empty=",
            "embedded=a=b",
            "\"quoted=%h value\"",
        ]
    );
    Ok(())
}

#[test]
fn build_target_preserves_duplicate_physical_lines_with_singleton_diagnostics() -> Result<(), String> {
    let result = QuadletDocument::parse(QuadletUnitType::Build, SourceId::new(181), BUILD_TARGET_DUPLICATES)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid());
    assert_eq!(result.syntax().document().render_preserved(), BUILD_TARGET_DUPLICATES);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Build(BuildKey::Target))
            .map(|entry| entry.value().primary().text())
            .collect::<Vec<_>>(),
        ["builder", "final"]
    );
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        ["QLM0004"]
    );
    Ok(())
}

#[test]
fn build_platform_preserves_blank_and_duplicate_singleton_physical_lines() -> Result<(), String> {
    let result = QuadletDocument::parse(QuadletUnitType::Build, SourceId::new(182), BUILD_PLATFORM_DUPLICATES)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid());
    assert_eq!(result.syntax().document().render_preserved(), BUILD_PLATFORM_DUPLICATES);
    for (key, values) in [(BuildKey::Arch, ["", "arm64"]), (BuildKey::Variant, ["", "v8"])] {
        let entries: Vec<_> = result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Build(key))
            .collect();
        assert_eq!(
            entries
                .iter()
                .map(|entry| entry.value().primary().text())
                .collect::<Vec<_>>(),
            values
        );
        assert!(entries.iter().all(|entry| entry.value_kind() == ValueKind::Opaque));
    }
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        ["QLM0004", "QLM0004"]
    );
    Ok(())
}

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
    assert_eq!(generic, ["Description", "Restart", "WantedBy"]);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::SystemdUnit(SystemdUnitKey::After))
            .map(|entry| entry.value().primary().text())
            .collect::<Vec<_>>(),
        ["network-online.target", "frontend.network"]
    );

    let known: Vec<_> = result
        .document()
        .entries()
        .filter_map(|entry| match entry.kind() {
            EntryKind::Container(key) if is_extended_opaque_container_key(key) => None,
            EntryKind::Container(key) => Some(key),
            _ => None,
        })
        .collect();
    assert_eq!(known, expected_fixture_core_container_keys());
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
        .map(TypedEntry::source_line)
        .collect();
    assert!(after_lines.len() == 2 && after_lines[0] < after_lines[1]);
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
    assert_eq!(
        container_entry(&result, ContainerKey::StopSignal, 0)?
            .value()
            .primary()
            .text(),
        "SIGUSR1"
    );
    assert_eq!(
        container_entry(&result, ContainerKey::StopTimeout, 0)?
            .value()
            .primary()
            .text(),
        "0"
    );
    assert_eq!(
        container_entry(&result, ContainerKey::Pull, 0)?
            .value()
            .primary()
            .text(),
        "newer"
    );
    assert_eq!(
        container_entry(&result, ContainerKey::PidsLimit, 0)?
            .value()
            .primary()
            .text(),
        "0"
    );
    assert_eq!(
        container_entry(&result, ContainerKey::HostName, 0)?
            .value()
            .primary()
            .text(),
        "app.example"
    );
    assert_eq!(
        container_entry(&result, ContainerKey::ShmSize, 0)?
            .value()
            .primary()
            .text(),
        "0"
    );
    assert_fixture_memory(&result)?;
    assert_fixture_ulimits(&result);
    assert_fixture_add_devices(&result);
    assert_fixture_networking_values(&result);
    assert_fixture_security_singletons(&result)?;
    Ok(())
}

#[test]
fn build_networks_preserve_order_and_classify_only_exact_network_units() -> Result<(), String> {
    let source = concat!(
        "[Build]\n",
        "Network=host\n",
        "Network=none\n",
        "Network=frontend.network\n",
        "Network=frontend.network:ip=192.0.2.10\n",
        "Network=frontend.container\n",
    );
    let result = QuadletDocument::parse(QuadletUnitType::Build, SourceId::new(181), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
    assert_eq!(result.syntax().document().render_preserved(), source);

    let observed: Vec<_> = result
        .document()
        .entries()
        .filter(|entry| entry.kind() == EntryKind::Build(BuildKey::Network))
        .map(|entry| (entry.value().primary().text(), entry.value_kind()))
        .collect();
    assert_eq!(
        observed,
        [
            ("host", ValueKind::Opaque),
            ("none", ValueKind::Opaque),
            ("frontend.network", ValueKind::UnitReference(UnitReferenceKind::Network)),
            ("frontend.network:ip=192.0.2.10", ValueKind::Opaque),
            ("frontend.container", ValueKind::Opaque),
        ]
    );
    Ok(())
}

#[test]
fn drop_capability_omission_repetition_order_and_raw_values_remain_distinct() -> Result<(), String> {
    for (source_id, authored) in [
        (SourceId::new(83), &[][..]),
        (SourceId::new(84), &["CAP_NET_ADMIN"][..]),
        (SourceId::new(88), &["CAP_NET_ADMIN", "CAP_NET_ADMIN"][..]),
        (
            SourceId::new(85),
            &["CAP_NET_ADMIN", "ALL", "CAP_DAC_OVERRIDE CAP_IPC_OWNER"][..],
        ),
        (SourceId::new(86), &["Vendor_Defined Capability Text"][..]),
    ] {
        let mut source = "[Container]\nImage=example.invalid/app\n".to_owned();
        for value in authored {
            source.push_str("DropCapability=");
            source.push_str(value);
            source.push('\n');
        }
        let result = QuadletDocument::parse(QuadletUnitType::Container, source_id, source.clone())
            .map_err(|error| error.to_string())?;
        assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
        assert_eq!(result.syntax().document().render_preserved(), source);
        let observed: Vec<_> = result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::DropCapability))
            .map(|entry| {
                assert_eq!(entry.value_kind(), ValueKind::Opaque);
                entry.value().primary().text()
            })
            .collect();
        assert_eq!(observed, authored);
    }

    let wrong_case = QuadletDocument::parse(
        QuadletUnitType::Container,
        SourceId::new(87),
        "[Container]\nImage=example.invalid/app\nDropcapability=ALL\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(
        wrong_case
            .document()
            .entries()
            .any(|entry| { entry.kind() == EntryKind::Unknown && entry.key().text() == "Dropcapability" })
    );
    Ok(())
}

#[test]
fn add_capability_omission_reset_duplicates_order_case_and_raw_values_remain_distinct() -> Result<(), String> {
    for (source_id, authored) in [
        (SourceId::new(89), &[][..]),
        (SourceId::new(90), &["CAP_NET_ADMIN"][..]),
        (SourceId::new(91), &["CAP_NET_ADMIN", "", "CAP_NET_ADMIN"][..]),
        (
            SourceId::new(92),
            &["CAP_NET_ADMIN", "ALL", "CAP_DAC_OVERRIDE CAP_IPC_OWNER"][..],
        ),
        (SourceId::new(93), &["Vendor_Defined Capability Text"][..]),
    ] {
        let mut source = "[Container]\nImage=example.invalid/app\n".to_owned();
        for value in authored {
            source.push_str("AddCapability=");
            source.push_str(value);
            source.push('\n');
        }
        let result = QuadletDocument::parse(QuadletUnitType::Container, source_id, source.clone())
            .map_err(|error| error.to_string())?;
        assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
        assert_eq!(result.syntax().document().render_preserved(), source);
        let observed: Vec<_> = result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::AddCapability))
            .map(|entry| {
                assert_eq!(entry.value_kind(), ValueKind::Opaque);
                entry.value().primary().text()
            })
            .collect();
        assert_eq!(observed, authored);
    }

    let wrong_case = QuadletDocument::parse(
        QuadletUnitType::Container,
        SourceId::new(94),
        "[Container]\nImage=example.invalid/app\nAddcapability=ALL\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(
        wrong_case
            .document()
            .entries()
            .any(|entry| { entry.kind() == EntryKind::Unknown && entry.key().text() == "Addcapability" })
    );
    Ok(())
}

#[test]
fn tmpfs_omission_reset_duplicates_order_case_options_and_raw_values_remain_distinct() -> Result<(), String> {
    for (source_id, authored) in [
        (SourceId::new(95), &[][..]),
        (SourceId::new(96), &["/cache"][..]),
        (
            SourceId::new(97),
            &[
                "/Before:RW,NoExec",
                "/before-two:size=64M",
                "",
                "/data:mode=755,uid=1009,gid=1009",
                "/data:mode=755,uid=1009,gid=1009",
            ][..],
        ),
        (SourceId::new(98), &["Vendor_Defined Tmpfs Options"][..]),
    ] {
        let mut source = "[Container]\nImage=example.invalid/app\n".to_owned();
        for value in authored {
            source.push_str("Tmpfs=");
            source.push_str(value);
            source.push('\n');
        }
        let result = QuadletDocument::parse(QuadletUnitType::Container, source_id, source.clone())
            .map_err(|error| error.to_string())?;
        assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
        assert_eq!(result.syntax().document().render_preserved(), source);
        let observed: Vec<_> = result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::Tmpfs))
            .map(|entry| {
                assert_eq!(entry.value_kind(), ValueKind::Opaque);
                entry.value().primary().text()
            })
            .collect();
        assert_eq!(observed, authored);
    }

    let wrong_case = QuadletDocument::parse(
        QuadletUnitType::Container,
        SourceId::new(99),
        "[Container]\nImage=example.invalid/app\nTmpFs=/data\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(
        wrong_case
            .document()
            .entries()
            .any(|entry| { entry.kind() == EntryKind::Unknown && entry.key().text() == "TmpFs" })
    );

    let pod = QuadletDocument::parse(QuadletUnitType::Pod, SourceId::new(100), "[Pod]\nTmpfs=/data\n")
        .map_err(|error| error.to_string())?;
    assert!(pod.document().entries().any(|entry| {
        entry.kind() == EntryKind::Unknown && entry.key().text() == "Tmpfs" && entry.value().primary().text() == "/data"
    }));
    Ok(())
}

#[test]
fn sysctl_omission_reset_duplicates_order_case_quoting_specifiers_and_raw_values_remain_distinct() -> Result<(), String>
{
    for (source_id, authored) in [
        (SourceId::new(101), &[][..]),
        (SourceId::new(102), &["net.ipv4.ip_forward=1"][..]),
        (
            SourceId::new(103),
            &[
                "net.ipv4.conf.all.rp_filter=2 net.ipv4.ip_forward=0",
                r#"kernel.domainname="Authored Value""#,
                "net.ipv4.conf.%i.forwarding=%n",
                "",
                "net.ipv4.ip_forward=1",
                "net.ipv4.ip_forward=1",
                "Vendor_Defined=MixedCase",
            ][..],
        ),
    ] {
        let mut source = "[Container]\nImage=example.invalid/app\n".to_owned();
        for value in authored {
            source.push_str("Sysctl=");
            source.push_str(value);
            source.push('\n');
        }
        let result = QuadletDocument::parse(QuadletUnitType::Container, source_id, source.clone())
            .map_err(|error| error.to_string())?;
        assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
        assert_eq!(result.syntax().document().render_preserved(), source);
        let observed: Vec<_> = result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::Sysctl))
            .map(|entry| {
                assert_eq!(entry.value_kind(), ValueKind::Opaque);
                entry.value().primary().text()
            })
            .collect();
        assert_eq!(observed, authored);
    }

    let wrong_case = QuadletDocument::parse(
        QuadletUnitType::Container,
        SourceId::new(104),
        "[Container]\nImage=example.invalid/app\nSysCtl=net.ipv4.ip_forward=1\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(
        wrong_case
            .document()
            .entries()
            .any(|entry| { entry.kind() == EntryKind::Unknown && entry.key().text() == "SysCtl" })
    );

    let pod = QuadletDocument::parse(
        QuadletUnitType::Pod,
        SourceId::new(105),
        "[Pod]\nSysctl=net.ipv4.ip_forward=1\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(pod.document().entries().any(|entry| {
        entry.kind() == EntryKind::Unknown
            && entry.key().text() == "Sysctl"
            && entry.value().primary().text() == "net.ipv4.ip_forward=1"
    }));
    Ok(())
}

#[test]
fn ulimit_omission_reset_duplicates_order_case_quoting_specifiers_and_raw_values_remain_distinct() -> Result<(), String>
{
    for (source_id, authored) in [
        (SourceId::new(106), &[][..]),
        (SourceId::new(107), &["core=0:0"][..]),
        (
            SourceId::new(108),
            &[
                "Core=0:0",
                r#"nofile="1024:2048""#,
                "stack=%h:%n",
                "",
                "nproc=4096:8192",
                "nproc=4096:8192",
                "Vendor_Defined=Soft:Hard",
            ][..],
        ),
    ] {
        let mut source = "[Container]\nImage=example.invalid/app\n".to_owned();
        for value in authored {
            source.push_str("Ulimit=");
            source.push_str(value);
            source.push('\n');
        }
        let result = QuadletDocument::parse(QuadletUnitType::Container, source_id, source.clone())
            .map_err(|error| error.to_string())?;
        assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
        assert_eq!(result.syntax().document().render_preserved(), source);
        let observed: Vec<_> = result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::Ulimit))
            .map(|entry| {
                assert_eq!(entry.value_kind(), ValueKind::Opaque);
                entry.value().primary().text()
            })
            .collect();
        assert_eq!(observed, authored);
    }

    let wrong_case = QuadletDocument::parse(
        QuadletUnitType::Container,
        SourceId::new(109),
        "[Container]\nImage=example.invalid/app\nULimit=core=0:0\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(
        wrong_case
            .document()
            .entries()
            .any(|entry| { entry.kind() == EntryKind::Unknown && entry.key().text() == "ULimit" })
    );

    let pod = QuadletDocument::parse(QuadletUnitType::Pod, SourceId::new(110), "[Pod]\nUlimit=core=0:0\n")
        .map_err(|error| error.to_string())?;
    assert!(pod.document().entries().any(|entry| {
        entry.kind() == EntryKind::Unknown
            && entry.key().text() == "Ulimit"
            && entry.value().primary().text() == "core=0:0"
    }));
    Ok(())
}

#[test]
fn add_device_omission_reset_duplicates_order_case_quotes_specifiers_whitespace_and_leading_dash_remain_distinct()
-> Result<(), String> {
    for (source_id, authored) in [
        (SourceId::new(111), &[][..]),
        (SourceId::new(112), &["/dev/null:/dev/null:r"][..]),
        (
            SourceId::new(113),
            &[
                "/dev/null:/dev/pre-null:r /dev/zero:/dev/pre-zero:w",
                "",
                r#""/dev/null:/dev/final null:r" /dev/zero:/dev/final-zero:w"#,
                "%h/Device:/dev/MixedCase:rwm",
                "-/dev/optional:/dev/optional:r",
                r#""/dev/null:/dev/final null:r" /dev/zero:/dev/final-zero:w"#,
                "Vendor_Defined Device Text",
            ][..],
        ),
    ] {
        let mut source = "[Container]\nImage=example.invalid/app\n".to_owned();
        for value in authored {
            source.push_str("AddDevice=");
            source.push_str(value);
            source.push('\n');
        }
        let result = QuadletDocument::parse(QuadletUnitType::Container, source_id, source.clone())
            .map_err(|error| error.to_string())?;
        assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
        assert_eq!(result.syntax().document().render_preserved(), source);
        let observed: Vec<_> = result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::AddDevice))
            .map(|entry| {
                assert_eq!(entry.value_kind(), ValueKind::Opaque);
                entry.value().primary().text()
            })
            .collect();
        assert_eq!(observed, authored);
    }

    let wrong_case = QuadletDocument::parse(
        QuadletUnitType::Container,
        SourceId::new(114),
        "[Container]\nImage=example.invalid/app\nAdddevice=/dev/null\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(
        wrong_case
            .document()
            .entries()
            .any(|entry| { entry.kind() == EntryKind::Unknown && entry.key().text() == "Adddevice" })
    );

    let pod = QuadletDocument::parse(
        QuadletUnitType::Pod,
        SourceId::new(115),
        "[Pod]\nAddDevice=/dev/null:/dev/null:r\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(pod.document().entries().any(|entry| {
        entry.kind() == EntryKind::Unknown
            && entry.key().text() == "AddDevice"
            && entry.value().primary().text() == "/dev/null:/dev/null:r"
    }));
    Ok(())
}

#[test]
fn container_logging_preserves_opaque_physical_values_cardinality_and_scope() -> Result<(), String> {
    let source = concat!(
        "[Container]\n",
        "Image=example.invalid/app\n",
        "LogDriver=k8s-file\n",
        "LogDriver=\n",
        "LogDriver=\"Vendor-%n Driver\"\n",
        "LogOpt=path=/var/log/pre.log\n",
        "LogOpt=\n",
        "LogOpt=tag=final-%n\n",
        "LogOpt=\"path=/var/log/Authored Value.log\"\n",
        "LogOpt=tag=final-%n\n",
    );
    let result = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(300), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        ["QLM0004", "QLM0004"]
    );
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::LogDriver))
            .map(|entry| {
                assert_eq!(entry.value_kind(), ValueKind::Opaque);
                entry.value().primary().text()
            })
            .collect::<Vec<_>>(),
        ["k8s-file", "", r#""Vendor-%n Driver""#]
    );
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::LogOpt))
            .map(|entry| {
                assert_eq!(entry.value_kind(), ValueKind::Opaque);
                entry.value().primary().text()
            })
            .collect::<Vec<_>>(),
        [
            "path=/var/log/pre.log",
            "",
            "tag=final-%n",
            r#""path=/var/log/Authored Value.log""#,
            "tag=final-%n",
        ]
    );

    let wrong_case = QuadletDocument::parse(
        QuadletUnitType::Container,
        SourceId::new(301),
        "[Container]\nImage=example.invalid/app\nLogdriver=k8s-file\nLogopt=tag=value\n",
    )
    .map_err(|error| error.to_string())?;
    assert_eq!(
        wrong_case
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Unknown)
            .map(|entry| entry.key().text())
            .collect::<Vec<_>>(),
        ["Logdriver", "Logopt"]
    );

    let pod = QuadletDocument::parse(
        QuadletUnitType::Pod,
        SourceId::new(302),
        "[Pod]\nLogDriver=k8s-file\nLogOpt=tag=value\n",
    )
    .map_err(|error| error.to_string())?;
    assert_eq!(
        pod.document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Unknown)
            .map(|entry| entry.key().text())
            .collect::<Vec<_>>(),
        ["LogDriver", "LogOpt"]
    );
    Ok(())
}

#[test]
fn container_network_identity_preserves_opaque_values_cardinality_continuations_and_scope() -> Result<(), String> {
    let source = concat!(
        "[Container]\n",
        "Image=example.invalid/app\n",
        "IP=192.0.2.10\n",
        "IP=\"192.0.2.%n\" \\\n",
        "  continued-ip\n",
        "IP6=2001:db8::10\n",
        "IP6=\"2001:db8::%n\"\n",
        "NetworkAlias=pre.example\n",
        "NetworkAlias=\n",
        "NetworkAlias=\"final %n\"\n",
        "NetworkAlias=alias-%i \\\n",
        "  continued-alias\n",
        "NetworkAlias=\"final %n\"\n",
    );
    let result = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(303), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        ["QLM0004", "QLM0004"]
    );

    let ipv4: Vec<_> = result
        .document()
        .entries()
        .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::IP))
        .collect();
    assert_eq!(ipv4.len(), 2);
    assert_eq!(ipv4[0].value_kind(), ValueKind::Opaque);
    assert_eq!(ipv4[0].value().primary().text(), "192.0.2.10");
    assert_eq!(ipv4[1].value().primary().text(), r#""192.0.2.%n" \"#);
    assert_eq!(ipv4[1].value().continuations()[0].text(), "continued-ip");

    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::IP6))
            .map(|entry| {
                assert_eq!(entry.value_kind(), ValueKind::Opaque);
                entry.value().primary().text()
            })
            .collect::<Vec<_>>(),
        ["2001:db8::10", r#""2001:db8::%n""#]
    );

    let aliases: Vec<_> = result
        .document()
        .entries()
        .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::NetworkAlias))
        .collect();
    assert_eq!(
        aliases
            .iter()
            .map(|entry| entry.value().primary().text())
            .collect::<Vec<_>>(),
        ["pre.example", "", r#""final %n""#, "alias-%i \\", r#""final %n""#]
    );
    assert!(aliases.iter().all(|entry| entry.value_kind() == ValueKind::Opaque));
    assert_eq!(aliases[3].value().continuations()[0].text(), "continued-alias");

    let wrong_case = QuadletDocument::parse(
        QuadletUnitType::Container,
        SourceId::new(304),
        "[Container]\nImage=example.invalid/app\nIp=192.0.2.1\nIp6=2001:db8::1\nNetworkalias=app\n",
    )
    .map_err(|error| error.to_string())?;
    assert_eq!(
        wrong_case
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Unknown)
            .map(|entry| entry.key().text())
            .collect::<Vec<_>>(),
        ["Ip", "Ip6", "Networkalias"]
    );

    let pod = QuadletDocument::parse(
        QuadletUnitType::Pod,
        SourceId::new(305),
        "[Pod]\nIP=192.0.2.1\nIP6=2001:db8::1\nNetworkAlias=app\n",
    )
    .map_err(|error| error.to_string())?;
    assert_eq!(
        pod.document()
            .entries()
            .filter_map(|entry| match entry.kind() {
                EntryKind::Pod(key) => Some(key),
                _ => None,
            })
            .collect::<Vec<_>>(),
        [PodKey::IP, PodKey::IP6, PodKey::NetworkAlias]
    );
    Ok(())
}

#[test]
fn dns_omission_reset_duplicates_order_case_quoting_specifiers_and_raw_values_remain_distinct() -> Result<(), String> {
    for (source_id, authored) in [
        (SourceId::new(126), &[][..]),
        (SourceId::new(127), &["1.1.1.1"][..]),
        (
            SourceId::new(128),
            &[
                "1.1.1.1",
                "1.1.1.1",
                "",
                "9.9.9.9",
                "2001:4860:4860::8888",
                r#""Authored Resolver""#,
                "%h",
                "Vendor_Defined_DNS",
            ][..],
        ),
    ] {
        let mut source = "[Container]\nImage=example.invalid/app\n".to_owned();
        for value in authored {
            source.push_str("DNS=");
            source.push_str(value);
            source.push('\n');
        }
        let result = QuadletDocument::parse(QuadletUnitType::Container, source_id, source.clone())
            .map_err(|error| error.to_string())?;
        assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
        assert_eq!(result.syntax().document().render_preserved(), source);
        let observed: Vec<_> = result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::DNS))
            .map(|entry| {
                assert_eq!(entry.value_kind(), ValueKind::Opaque);
                entry.value().primary().text()
            })
            .collect();
        assert_eq!(observed, authored);
    }

    let wrong_case = QuadletDocument::parse(
        QuadletUnitType::Container,
        SourceId::new(129),
        "[Container]\nImage=example.invalid/app\nDns=1.1.1.1\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(
        wrong_case
            .document()
            .entries()
            .any(|entry| entry.kind() == EntryKind::Unknown && entry.key().text() == "Dns")
    );

    let network = QuadletDocument::parse(QuadletUnitType::Network, SourceId::new(131), "[Network]\nDNS=1.1.1.1\n")
        .map_err(|error| error.to_string())?;
    assert!(
        network
            .document()
            .entries()
            .any(|entry| entry.kind() == EntryKind::Network(NetworkKey::DNS))
    );
    let image = QuadletDocument::parse(
        QuadletUnitType::Image,
        SourceId::new(132),
        "[Image]\nImage=example.invalid/app\nDNS=1.1.1.1\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(image.document().entries().any(|entry| {
        entry.kind() == EntryKind::Unknown && entry.key().text() == "DNS" && entry.value().primary().text() == "1.1.1.1"
    }));
    Ok(())
}

#[test]
fn dns_option_omission_reset_duplicates_order_quoting_specifiers_whitespace_and_raw_values_remain_distinct()
-> Result<(), String> {
    for (source_id, authored) in [
        (SourceId::new(132), &[][..]),
        (SourceId::new(133), &["rotate"][..]),
        (
            SourceId::new(134),
            &[
                "rotate",
                "rotate",
                "",
                "ndots:1",
                "use-vc",
                r#""Authored Option""#,
                "%h",
                "Vendor Defined Option",
            ][..],
        ),
    ] {
        let mut source = "[Container]\nImage=example.invalid/app\n".to_owned();
        for value in authored {
            source.push_str("DNSOption=");
            source.push_str(value);
            source.push('\n');
        }
        let result = QuadletDocument::parse(QuadletUnitType::Container, source_id, source.clone())
            .map_err(|error| error.to_string())?;
        assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
        assert_eq!(result.syntax().document().render_preserved(), source);
        let observed: Vec<_> = result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::DNSOption))
            .map(|entry| {
                assert_eq!(entry.value_kind(), ValueKind::Opaque);
                entry.value().primary().text()
            })
            .collect();
        assert_eq!(observed, authored);
    }

    let wrong_case = QuadletDocument::parse(
        QuadletUnitType::Container,
        SourceId::new(135),
        "[Container]\nImage=example.invalid/app\nDnsOption=rotate\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(
        wrong_case
            .document()
            .entries()
            .any(|entry| { entry.kind() == EntryKind::Unknown && entry.key().text() == "DnsOption" })
    );

    for (unit_type, source_id, source) in [
        (
            QuadletUnitType::Network,
            SourceId::new(137),
            "[Network]\nDNSOption=rotate\n",
        ),
        (
            QuadletUnitType::Image,
            SourceId::new(138),
            "[Image]\nImage=example.invalid/app\nDNSOption=rotate\n",
        ),
    ] {
        let result = QuadletDocument::parse(unit_type, source_id, source).map_err(|error| error.to_string())?;
        assert!(result.document().entries().any(|entry| {
            entry.kind() == EntryKind::Unknown
                && entry.key().text() == "DNSOption"
                && entry.value().primary().text() == "rotate"
        }));
    }
    assert_eq!(QuadletUnitType::from_extension("build"), Some(QuadletUnitType::Build));
    Ok(())
}

#[test]
fn dns_search_omission_reset_duplicates_order_quoting_specifiers_and_raw_values_remain_distinct() -> Result<(), String>
{
    for (source_id, authored) in [
        (SourceId::new(138), &[][..]),
        (SourceId::new(139), &["example.com"][..]),
        (
            SourceId::new(140),
            &[
                "pre.example.com",
                "pre.example.com",
                "",
                "dc1.example.com",
                ".",
                r#""Authored Search""#,
                "%h",
                "Vendor Defined Search",
            ][..],
        ),
    ] {
        let mut source = "[Container]\nImage=example.invalid/app\n".to_owned();
        for value in authored {
            source.push_str("DNSSearch=");
            source.push_str(value);
            source.push('\n');
        }
        let result = QuadletDocument::parse(QuadletUnitType::Container, source_id, source.clone())
            .map_err(|error| error.to_string())?;
        assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
        assert_eq!(result.syntax().document().render_preserved(), source);
        let observed: Vec<_> = result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::DNSSearch))
            .map(|entry| {
                assert_eq!(entry.value_kind(), ValueKind::Opaque);
                entry.value().primary().text()
            })
            .collect();
        assert_eq!(observed, authored);
    }

    let wrong_case = QuadletDocument::parse(
        QuadletUnitType::Container,
        SourceId::new(141),
        "[Container]\nImage=example.invalid/app\nDnsSearch=example.com\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(
        wrong_case
            .document()
            .entries()
            .any(|entry| { entry.kind() == EntryKind::Unknown && entry.key().text() == "DnsSearch" })
    );

    let pod = QuadletDocument::parse(
        QuadletUnitType::Pod,
        SourceId::new(142),
        "[Pod]\nDNSSearch=example.com\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(
        pod.document()
            .entries()
            .any(|entry| entry.kind() == EntryKind::Pod(PodKey::DNSSearch))
    );
    assert_eq!(QuadletUnitType::from_extension("build"), Some(QuadletUnitType::Build));
    Ok(())
}

#[test]
fn expose_host_port_omission_reset_duplicates_order_quotes_specifiers_invalid_and_sctp_remain_distinct()
-> Result<(), String> {
    for (source_id, authored) in [
        (SourceId::new(143), &[][..]),
        (SourceId::new(144), &["8080"][..]),
        (
            SourceId::new(145),
            &[
                "1000",
                "1000",
                "",
                "3000",
                "8080-8085",
                "9090/tcp",
                "5353/udp",
                "5353/sctp",
                r#""Authored Port""#,
                "%i",
                "not-a-port",
            ][..],
        ),
    ] {
        let mut source = "[Container]\nImage=example.invalid/app\n".to_owned();
        for value in authored {
            source.push_str("ExposeHostPort=");
            source.push_str(value);
            source.push('\n');
        }
        let result = QuadletDocument::parse(QuadletUnitType::Container, source_id, source.clone())
            .map_err(|error| error.to_string())?;
        assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
        assert_eq!(result.syntax().document().render_preserved(), source);
        let observed: Vec<_> = result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::ExposeHostPort))
            .map(|entry| {
                assert_eq!(entry.value_kind(), ValueKind::Opaque);
                entry.value().primary().text()
            })
            .collect();
        assert_eq!(observed, authored);
    }

    let wrong_case = QuadletDocument::parse(
        QuadletUnitType::Container,
        SourceId::new(146),
        "[Container]\nImage=example.invalid/app\nExposehostport=8080\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(
        wrong_case
            .document()
            .entries()
            .any(|entry| { entry.kind() == EntryKind::Unknown && entry.key().text() == "Exposehostport" })
    );

    let pod = QuadletDocument::parse(QuadletUnitType::Pod, SourceId::new(147), "[Pod]\nExposeHostPort=8080\n")
        .map_err(|error| error.to_string())?;
    assert!(pod.document().entries().any(|entry| {
        entry.kind() == EntryKind::Unknown
            && entry.key().text() == "ExposeHostPort"
            && entry.value().primary().text() == "8080"
    }));
    Ok(())
}

#[test]
fn annotation_is_container_and_build_repeatable_opaque_and_preserves_every_physical_value() -> Result<(), String> {
    for (source_id, authored) in [
        (SourceId::new(148), &[][..]),
        (SourceId::new(149), &["org.example.name=one"][..]),
        (
            SourceId::new(150),
            &[
                "org.example.name=first",
                "org.example.name=first",
                "",
                "org.example.name=final",
                r#""org.example.quoted=Authored Value""#,
                "org.example.specifier=%i",
                "key-only",
                "malformed = value ",
            ][..],
        ),
    ] {
        let mut source = "[Container]\nImage=example.invalid/app\n".to_owned();
        for value in authored {
            source.push_str("Annotation=");
            source.push_str(value);
            source.push('\n');
        }
        let result = QuadletDocument::parse(QuadletUnitType::Container, source_id, source.clone())
            .map_err(|error| error.to_string())?;
        assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
        assert_eq!(result.syntax().document().render_preserved(), source);
        let observed: Vec<_> = result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::Annotation))
            .map(|entry| {
                assert_eq!(entry.value_kind(), ValueKind::Opaque);
                entry.value().primary().text()
            })
            .collect();
        assert_eq!(observed, authored);
    }

    let other_sections = QuadletDocument::parse(
        QuadletUnitType::Container,
        SourceId::new(151),
        "[Container]\nImage=example.invalid/app\n[Build]\nAnnotation=org.example.build=value\n[Service]\nAnnotation=org.example.service=value\n",
    )
    .map_err(|error| error.to_string())?;
    assert_eq!(
        other_sections
            .document()
            .entries()
            .filter(|entry| entry.key().text() == "Annotation")
            .map(TypedEntry::kind)
            .collect::<Vec<_>>(),
        [EntryKind::Build(BuildKey::Annotation), EntryKind::GenericSystemd]
    );
    Ok(())
}

#[test]
fn hostname_omission_and_raw_values_remain_distinct_without_validation() -> Result<(), String> {
    for (source_id, authored) in [
        (SourceId::new(67), None),
        (SourceId::new(68), Some("app.example")),
        (SourceId::new(69), Some("Authored_Native_Value")),
    ] {
        let source = authored.map_or_else(
            || "[Container]\nImage=example.invalid/app\n".to_owned(),
            |value| format!("[Container]\nImage=example.invalid/app\nHostName={value}\n"),
        );
        let result = QuadletDocument::parse(QuadletUnitType::Container, source_id, source.clone())
            .map_err(|error| error.to_string())?;
        assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
        assert_eq!(result.syntax().document().render_preserved(), source);

        let hostname = result
            .document()
            .entries()
            .find(|entry| entry.kind() == EntryKind::Container(ContainerKey::HostName));
        assert_eq!(hostname.map(|entry| entry.value().primary().text()), authored);
        if let Some(entry) = hostname {
            assert_eq!(entry.value_kind(), ValueKind::Opaque);
        }
    }

    let wrong_case = QuadletDocument::parse(
        QuadletUnitType::Container,
        SourceId::new(71),
        "[Container]\nImage=example.invalid/app\nHostname=app.example\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(wrong_case.is_valid());
    assert!(
        wrong_case
            .document()
            .entries()
            .any(|entry| { entry.kind() == EntryKind::Unknown && entry.key().text() == "Hostname" })
    );
    Ok(())
}

#[test]
fn hostname_is_a_singleton_in_authored_documents() -> Result<(), String> {
    let result = QuadletDocument::parse(
        QuadletUnitType::Container,
        SourceId::new(70),
        "[Container]\nImage=example.invalid/app\nHostName=first.example\nHostName=second.example\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(result.is_valid());
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        ["QLM0004"]
    );
    assert_eq!(
        container_entry(&result, ContainerKey::HostName, 1)?
            .value()
            .primary()
            .text(),
        "second.example"
    );
    Ok(())
}

#[test]
fn container_and_pod_shm_size_omission_and_raw_values_remain_distinct() -> Result<(), String> {
    for (source_id, unit_type, section, expected_kind) in [
        (
            72,
            QuadletUnitType::Container,
            "Container",
            EntryKind::Container(ContainerKey::ShmSize),
        ),
        (73, QuadletUnitType::Pod, "Pod", EntryKind::Pod(PodKey::ShmSize)),
    ] {
        for authored in [None, Some("0"), Some("00064m"), Some("vendor-defined-size")] {
            let workload = if unit_type == QuadletUnitType::Container {
                "Image=example.invalid/app\n"
            } else {
                ""
            };
            let entry = authored.map_or_else(String::new, |value| format!("ShmSize={value}\n"));
            let source = format!("[{section}]\n{workload}{entry}");
            let result = QuadletDocument::parse(unit_type, SourceId::new(source_id), source.clone())
                .map_err(|error| error.to_string())?;
            assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
            assert_eq!(result.syntax().document().render_preserved(), source);
            let shm_size = result.document().entries().find(|entry| entry.kind() == expected_kind);
            assert_eq!(shm_size.map(|entry| entry.value().primary().text()), authored);
            if let Some(entry) = shm_size {
                assert_eq!(entry.value_kind(), ValueKind::Opaque);
            }
        }
    }

    let wrong_case = QuadletDocument::parse(
        QuadletUnitType::Container,
        SourceId::new(80),
        "[Container]\nImage=example.invalid/app\nShmsize=64m\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(
        wrong_case
            .document()
            .entries()
            .any(|entry| { entry.kind() == EntryKind::Unknown && entry.key().text() == "Shmsize" })
    );
    Ok(())
}

#[test]
fn container_and_pod_shm_size_are_singletons_in_authored_documents() -> Result<(), String> {
    for (source_id, unit_type, source, expected_kind) in [
        (
            81,
            QuadletUnitType::Container,
            "[Container]\nImage=example.invalid/app\nShmSize=64m\nShmSize=0\n",
            EntryKind::Container(ContainerKey::ShmSize),
        ),
        (
            82,
            QuadletUnitType::Pod,
            "[Pod]\nShmSize=64m\nShmSize=0\n",
            EntryKind::Pod(PodKey::ShmSize),
        ),
    ] {
        let result =
            QuadletDocument::parse(unit_type, SourceId::new(source_id), source).map_err(|error| error.to_string())?;
        assert!(result.is_valid());
        assert_eq!(
            result
                .model_diagnostics()
                .iter()
                .map(|diagnostic| diagnostic.code().as_str())
                .collect::<Vec<_>>(),
            ["QLM0004"]
        );
        assert_eq!(
            result
                .document()
                .entries()
                .filter(|entry| entry.kind() == expected_kind)
                .nth(1)
                .map(|entry| entry.value().primary().text()),
            Some("0")
        );
    }
    Ok(())
}

#[test]
fn container_memory_omission_and_raw_values_remain_distinct() -> Result<(), String> {
    for (source_id, authored) in [
        (SourceId::new(116), None),
        (SourceId::new(117), Some("")),
        (SourceId::new(118), Some("0")),
        (SourceId::new(119), Some("00016777216b")),
        (SourceId::new(120), Some(r#""64m""#)),
        (SourceId::new(121), Some("%h")),
        (SourceId::new(122), Some("vendor-defined-memory")),
    ] {
        let source = authored.map_or_else(
            || "[Container]\nImage=example.invalid/app\n".to_owned(),
            |value| format!("[Container]\nImage=example.invalid/app\nMemory={value}\n"),
        );
        let result = QuadletDocument::parse(QuadletUnitType::Container, source_id, source.clone())
            .map_err(|error| error.to_string())?;
        assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
        assert_eq!(result.syntax().document().render_preserved(), source);
        let memory = result
            .document()
            .entries()
            .find(|entry| entry.kind() == EntryKind::Container(ContainerKey::Memory));
        assert_eq!(memory.map(|entry| entry.value().primary().text()), authored);
        if let Some(entry) = memory {
            assert_eq!(entry.value_kind(), ValueKind::Opaque);
        }
    }

    let wrong_case = QuadletDocument::parse(
        QuadletUnitType::Container,
        SourceId::new(123),
        "[Container]\nImage=example.invalid/app\nmemory=16m\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(
        wrong_case
            .document()
            .entries()
            .any(|entry| { entry.kind() == EntryKind::Unknown && entry.key().text() == "memory" })
    );

    let pod = QuadletDocument::parse(QuadletUnitType::Pod, SourceId::new(124), "[Pod]\nMemory=16m\n")
        .map_err(|error| error.to_string())?;
    assert!(pod.document().entries().any(|entry| {
        entry.kind() == EntryKind::Unknown && entry.key().text() == "Memory" && entry.value().primary().text() == "16m"
    }));
    Ok(())
}

#[test]
fn container_memory_is_a_singleton_in_authored_documents() -> Result<(), String> {
    let result = QuadletDocument::parse(
        QuadletUnitType::Container,
        SourceId::new(125),
        "[Container]\nImage=example.invalid/app\nMemory=\nMemory=\"64m\"\nMemory=16777216b\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(result.is_valid());
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        ["QLM0004", "QLM0004"]
    );
    assert_eq!(
        container_entry(&result, ContainerKey::Memory, 2)?
            .value()
            .primary()
            .text(),
        "16777216b"
    );
    Ok(())
}

#[test]
fn pids_limit_omission_and_raw_values_remain_distinct_without_validation() -> Result<(), String> {
    for (source_id, authored) in [
        (SourceId::new(60), None),
        (SourceId::new(61), Some("-1")),
        (SourceId::new(62), Some("47")),
        (SourceId::new(63), Some("0")),
        (SourceId::new(64), Some("vendor-defined-limit")),
        (SourceId::new(65), Some("999999999999999999999999999999999999")),
    ] {
        let source = authored.map_or_else(
            || "[Container]\nImage=example.invalid/app\n".to_owned(),
            |value| format!("[Container]\nImage=example.invalid/app\nPidsLimit={value}\n"),
        );
        let result = QuadletDocument::parse(QuadletUnitType::Container, source_id, source.clone())
            .map_err(|error| error.to_string())?;
        assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
        assert_eq!(result.syntax().document().render_preserved(), source);

        let pids_limit = result
            .document()
            .entries()
            .find(|entry| entry.kind() == EntryKind::Container(ContainerKey::PidsLimit));
        assert_eq!(pids_limit.map(|entry| entry.value().primary().text()), authored);
        if let Some(entry) = pids_limit {
            assert_eq!(entry.value_kind(), ValueKind::Opaque);
        }
    }
    Ok(())
}

#[test]
fn pids_limit_is_a_singleton_in_authored_documents() -> Result<(), String> {
    let result = QuadletDocument::parse(
        QuadletUnitType::Container,
        SourceId::new(66),
        "[Container]\nImage=example.invalid/app\nPidsLimit=47\nPidsLimit=-1\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(result.is_valid());
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        ["QLM0004"]
    );
    assert_eq!(
        container_entry(&result, ContainerKey::PidsLimit, 1)?
            .value()
            .primary()
            .text(),
        "-1"
    );
    Ok(())
}

#[test]
fn pull_omission_supported_forms_and_raw_text_remain_distinct() -> Result<(), String> {
    for (source_id, authored) in [
        (SourceId::new(53), None),
        (SourceId::new(54), Some("always")),
        (SourceId::new(55), Some("missing")),
        (SourceId::new(56), Some("never")),
        (SourceId::new(57), Some("newer")),
        (SourceId::new(58), Some("vendor-defined-policy")),
    ] {
        let source = authored.map_or_else(
            || "[Container]\nImage=example.invalid/app\n".to_owned(),
            |value| format!("[Container]\nImage=example.invalid/app\nPull={value}\n"),
        );
        let result = QuadletDocument::parse(QuadletUnitType::Container, source_id, source.clone())
            .map_err(|error| error.to_string())?;
        assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
        assert_eq!(result.syntax().document().render_preserved(), source);

        let pull = result
            .document()
            .entries()
            .find(|entry| entry.kind() == EntryKind::Container(ContainerKey::Pull));
        assert_eq!(pull.map(|entry| entry.value().primary().text()), authored);
        if let Some(entry) = pull {
            assert_eq!(entry.value_kind(), ValueKind::Opaque);
        }
    }
    Ok(())
}

#[test]
fn pull_is_a_singleton_in_authored_documents() -> Result<(), String> {
    let result = QuadletDocument::parse(
        QuadletUnitType::Container,
        SourceId::new(59),
        "[Container]\nImage=example.invalid/app\nPull=missing\nPull=always\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(result.is_valid());
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        ["QLM0004"]
    );
    assert_eq!(
        container_entry(&result, ContainerKey::Pull, 1)?
            .value()
            .primary()
            .text(),
        "always"
    );
    Ok(())
}

#[test]
fn run_init_omission_and_authored_values_remain_distinct() -> Result<(), String> {
    for (source_id, authored) in [
        (SourceId::new(48), None),
        (SourceId::new(49), Some("true")),
        (SourceId::new(50), Some("false")),
        (SourceId::new(51), Some("vendor-defined-value")),
    ] {
        let source = authored.map_or_else(
            || "[Container]\nImage=example.invalid/app\n".to_owned(),
            |value| format!("[Container]\nImage=example.invalid/app\nRunInit={value}\n"),
        );
        let result = QuadletDocument::parse(QuadletUnitType::Container, source_id, source.clone())
            .map_err(|error| error.to_string())?;
        assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
        assert_eq!(result.syntax().document().render_preserved(), source);

        let run_init = result
            .document()
            .entries()
            .find(|entry| entry.kind() == EntryKind::Container(ContainerKey::RunInit));
        assert_eq!(run_init.map(|entry| entry.value().primary().text()), authored);
        if let Some(entry) = run_init {
            assert_eq!(entry.value_kind(), ValueKind::Opaque);
        }
    }
    Ok(())
}

#[test]
fn run_init_is_a_singleton_in_authored_documents() -> Result<(), String> {
    let result = QuadletDocument::parse(
        QuadletUnitType::Container,
        SourceId::new(52),
        "[Container]\nImage=example.invalid/app\nRunInit=true\nRunInit=false\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(result.is_valid());
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        ["QLM0004"]
    );
    assert_eq!(
        container_entry(&result, ContainerKey::RunInit, 1)?
            .value()
            .primary()
            .text(),
        "false"
    );
    Ok(())
}

#[test]
fn lifecycle_keys_are_singletons_in_authored_documents() -> Result<(), String> {
    let result = QuadletDocument::parse(
        QuadletUnitType::Container,
        SourceId::new(44),
        concat!(
            "[Container]\n",
            "Image=example.invalid/app\n",
            "StopSignal=SIGTERM\n",
            "StopSignal=9\n",
            "StopTimeout=30\n",
            "StopTimeout=0\n",
        ),
    )
    .map_err(|error| error.to_string())?;
    assert!(result.is_valid());
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        ["QLM0004", "QLM0004"]
    );
    assert_eq!(
        container_entry(&result, ContainerKey::StopSignal, 1)?
            .value()
            .primary()
            .text(),
        "9"
    );
    assert_eq!(
        container_entry(&result, ContainerKey::StopTimeout, 1)?
            .value()
            .primary()
            .text(),
        "0"
    );
    Ok(())
}

#[test]
fn lifecycle_recognition_preserves_one_line_values_without_semantic_validation() -> Result<(), String> {
    for (source_id, timeout) in [
        (SourceId::new(45), "-1"),
        (SourceId::new(46), "1.5"),
        (SourceId::new(47), "999999999999999999999999999999999999"),
    ] {
        let source = format!(
            "[Container]\nImage=example.invalid/app\nStopSignal=vendor-defined-signal\nStopTimeout={timeout}\n"
        );
        let result =
            QuadletDocument::parse(QuadletUnitType::Container, source_id, source).map_err(|error| error.to_string())?;
        assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
        let signal = container_entry(&result, ContainerKey::StopSignal, 0)?;
        assert_eq!(signal.value_kind(), ValueKind::Opaque);
        assert_eq!(signal.value().primary().text(), "vendor-defined-signal");
        let timeout_entry = container_entry(&result, ContainerKey::StopTimeout, 0)?;
        assert_eq!(timeout_entry.value_kind(), ValueKind::Opaque);
        assert_eq!(timeout_entry.value().primary().text(), timeout);
    }
    Ok(())
}

#[test]
fn network_and_image_completion_keys_are_typed_opaque_and_preserve_repeatability() -> Result<(), String> {
    let network = QuadletDocument::parse(
        QuadletUnitType::Network,
        SourceId::new(906),
        concat!(
            "[Network]\nNetworkName=completion\nContainersConfModule=one.conf\n",
            "ContainersConfModule=two.conf\nDNS=9.9.9.9\nDNS=2001:4860:4860::8888\n",
            "GlobalArgs=--log-level=debug\nPodmanArgs=--internal\nPodmanArgs=--opt isolate=true\n",
            "DisableDNS=true\nDisableDNS=false\nInterfaceName=quadlet0\n",
            "NetworkDeleteOnStop=true\nServiceName=completion-network\n"
        ),
    )
    .map_err(|error| error.to_string())?;
    assert!(network.is_valid());
    assert_eq!(
        network
            .document()
            .entries()
            .filter_map(|entry| match entry.kind() {
                EntryKind::Network(key) => Some(key),
                _ => None,
            })
            .collect::<Vec<_>>(),
        [
            NetworkKey::NetworkName,
            NetworkKey::ContainersConfModule,
            NetworkKey::ContainersConfModule,
            NetworkKey::DNS,
            NetworkKey::DNS,
            NetworkKey::GlobalArgs,
            NetworkKey::PodmanArgs,
            NetworkKey::PodmanArgs,
            NetworkKey::DisableDNS,
            NetworkKey::DisableDNS,
            NetworkKey::InterfaceName,
            NetworkKey::NetworkDeleteOnStop,
            NetworkKey::ServiceName,
        ]
    );
    assert_eq!(
        network
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        ["QLM0004"]
    );

    let image = QuadletDocument::parse(
        QuadletUnitType::Image,
        SourceId::new(907),
        concat!(
            "[Image]\nImage=example.invalid/application:1\nPodmanArgs=--quiet\n",
            "PodmanArgs=--all-tags\nPolicy=newer\nPolicy=missing\nRetry=4\n",
            "RetryDelay=7s\nTLSVerify=false\nVariant=v8\n"
        ),
    )
    .map_err(|error| error.to_string())?;
    assert!(image.is_valid());
    assert_eq!(
        image
            .document()
            .entries()
            .filter_map(|entry| match entry.kind() {
                EntryKind::Image(key) => Some(key),
                _ => None,
            })
            .collect::<Vec<_>>(),
        [
            ImageKey::Image,
            ImageKey::PodmanArgs,
            ImageKey::PodmanArgs,
            ImageKey::Policy,
            ImageKey::Policy,
            ImageKey::Retry,
            ImageKey::RetryDelay,
            ImageKey::TLSVerify,
            ImageKey::Variant,
        ]
    );
    assert_eq!(
        image
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        ["QLM0004"]
    );
    assert!(
        image
            .document()
            .entries()
            .filter(|entry| entry.kind() != EntryKind::Image(ImageKey::Image))
            .all(|entry| entry.value_kind() == ValueKind::Opaque)
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
            PodKey::UserNS,
            PodKey::ShmSize,
            PodKey::DNS,
            PodKey::DNSOption,
            PodKey::DNSSearch
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
            .map(|entry| (entry.key().text(), entry.value().primary().text()))
            .collect::<Vec<_>>(),
        [
            ("Ulimit", "core=0:0"),
            ("AddDevice", "/dev/null:/dev/null:r"),
            ("Memory", "16m"),
            ("ExposeHostPort", "8080"),
            ("AppArmor", "unconfined"),
            ("SeccompProfile", "unconfined"),
            ("FuturePodKey", "future-value")
        ]
    );
    assert_eq!(
        network
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Unknown)
            .count(),
        2
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
fn network_driver_and_options_preserve_physical_values_cardinality_and_scope() -> Result<(), String> {
    let source = concat!(
        "[Network]\n",
        "NetworkName=frontend\n",
        "Driver=bridge\n",
        "Driver=Vendor-%n-Driver\n",
        "Options=pre=one\n",
        "Options=pre=two\n",
        "Options=\n",
        "Options=zeta=last\n",
        "Options=alpha=first\n",
        "Options=alpha=final\n",
        "Options=bare-token\n",
        "Options=\"quoted option=%n\"\n",
        "Options=continuation=one \\\n",
        "  two\n",
    );
    let result = QuadletDocument::parse(QuadletUnitType::Network, SourceId::new(306), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Network(NetworkKey::Driver))
            .map(|entry| entry.value().primary().text())
            .collect::<Vec<_>>(),
        ["bridge", "Vendor-%n-Driver"]
    );
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Network(NetworkKey::Options))
            .map(|entry| entry.value().primary().text())
            .collect::<Vec<_>>(),
        [
            "pre=one",
            "pre=two",
            "",
            "zeta=last",
            "alpha=first",
            "alpha=final",
            "bare-token",
            r#""quoted option=%n""#,
            "continuation=one \\",
        ]
    );
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        ["QLM0004"]
    );

    let container = QuadletDocument::parse(
        QuadletUnitType::Container,
        SourceId::new(307),
        "[Container]\nImage=example.invalid/app\nDriver=bridge\nOptions=mtu=1500\n",
    )
    .map_err(|error| error.to_string())?;
    assert_eq!(
        container
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Unknown)
            .map(|entry| entry.key().text())
            .collect::<Vec<_>>(),
        ["Driver", "Options"]
    );
    Ok(())
}

#[test]
fn volume_driver_options_device_and_type_preserve_physical_values_cardinality_and_scope() -> Result<(), String> {
    let source = concat!(
        "[Volume]\n",
        "VolumeName=cache\n",
        "Driver=local\n",
        "Driver=\"Vendor-%n Driver\"\n",
        "Options=pre=discard\n",
        "Options=bare-token\n",
        "Options=\"matched option=%h\"\n",
        "Options=\"unmatched option=%h\n",
        "Options=continued=one \\\n",
        "  two\n",
        "Options=\n",
        "Device=/srv/pre\n",
        "Device=\"/srv/matched %h\"\n",
        "Device=\"/srv/unmatched %h\n",
        "Device=/srv/continued \\\n",
        "  two\n",
        "Device=\n",
        "Type=tmpfs\n",
        "Type=\"bind %h\"\n",
        "Type=\"unmatched %h\n",
        "Type=bind \\\n",
        "  continued\n",
    );
    let result = QuadletDocument::parse(QuadletUnitType::Volume, SourceId::new(308), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Volume(VolumeKey::Driver))
            .map(|entry| entry.value().primary().text())
            .collect::<Vec<_>>(),
        ["local", r#""Vendor-%n Driver""#]
    );
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Volume(VolumeKey::Options))
            .map(|entry| entry.value().primary().text())
            .collect::<Vec<_>>(),
        [
            "pre=discard",
            "bare-token",
            r#""matched option=%h""#,
            r#""unmatched option=%h"#,
            "continued=one \\",
            "",
        ]
    );
    assert_volume_device_and_type_physical_values(result.document())?;
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        [
            "QLM0004", "QLM0004", "QLM0004", "QLM0004", "QLM0004", "QLM0004", "QLM0004", "QLM0004", "QLM0004",
            "QLM0004", "QLM0004", "QLM0004", "QLM0004",
        ]
    );

    let container = QuadletDocument::parse(
        QuadletUnitType::Container,
        SourceId::new(309),
        "[Container]\nImage=example.invalid/app\nDriver=local\nOptions=o=discard\nDevice=/srv/data\nType=bind\n",
    )
    .map_err(|error| error.to_string())?;
    assert_eq!(
        container
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Unknown)
            .map(|entry| entry.key().text())
            .collect::<Vec<_>>(),
        ["Driver", "Options", "Device", "Type"]
    );
    Ok(())
}

#[test]
fn volume_copy_is_an_opaque_singleton_with_physical_source_fidelity() -> Result<(), String> {
    let source = concat!(
        "[Volume]\n",
        "Copy=pre=discard\n",
        "Copy=TrUe\n",
        "Copy=\"matched true\"\n",
        "Copy=\"unmatched true\n",
        "Copy=\n",
        "Copy=%h\n",
        "Copy=tr\\\n",
        "  ue\n",
    );
    let result = QuadletDocument::parse(QuadletUnitType::Volume, SourceId::new(311), source)
        .map_err(|error| error.to_string())?;
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Volume(VolumeKey::Copy))
            .map(|entry| entry.value().primary().text())
            .collect::<Vec<_>>(),
        [
            "pre=discard",
            "TrUe",
            r#""matched true""#,
            r#""unmatched true"#,
            "",
            "%h",
            "tr\\",
        ]
    );
    let continued = result
        .document()
        .entries()
        .find(|entry| entry.kind() == EntryKind::Volume(VolumeKey::Copy) && entry.value().is_continued())
        .ok_or_else(|| "continued Copy value must be retained".to_owned())?;
    assert_eq!(continued.value().continuations()[0].text(), "ue");
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        ["QLM0004", "QLM0004", "QLM0004", "QLM0004", "QLM0004", "QLM0004"]
    );

    let container = QuadletDocument::parse(
        QuadletUnitType::Container,
        SourceId::new(312),
        "[Container]\nImage=example.invalid/app\nCopy=true\n",
    )
    .map_err(|error| error.to_string())?;
    assert_eq!(
        container
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Unknown)
            .map(|entry| entry.key().text())
            .collect::<Vec<_>>(),
        ["Copy"]
    );
    Ok(())
}

#[test]
fn volume_containers_conf_module_preserves_every_opaque_physical_value_and_scope() -> Result<(), String> {
    let source = concat!(
        "[Volume]\n",
        "ContainersConfModule=pre-one\n",
        "ContainersConfModule=\n",
        "ContainersConfModule= post one \n",
        "ContainersConfModule=post-two\n",
        "ContainersConfModule=post-two\n",
        "ContainersConfModule=\"quoted %h module\"\n",
        "ContainersConfModule=module\\x20text\n",
        "ContainersConfModule=continuation\\-looking\n",
        "ContainersConfModule=continued \\\n",
        " module\n",
    );
    let result = QuadletDocument::parse(QuadletUnitType::Volume, SourceId::new(313), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Volume(VolumeKey::ContainersConfModule))
            .map(|entry| (entry.value().primary().text(), entry.value_kind()))
            .collect::<Vec<_>>(),
        [
            ("pre-one", ValueKind::Opaque),
            ("", ValueKind::Opaque),
            ("post one ", ValueKind::Opaque),
            ("post-two", ValueKind::Opaque),
            ("post-two", ValueKind::Opaque),
            ("\"quoted %h module\"", ValueKind::Opaque),
            ("module\\x20text", ValueKind::Opaque),
            ("continuation\\-looking", ValueKind::Opaque),
            ("continued \\", ValueKind::Opaque),
        ]
    );
    let continued = result
        .document()
        .entries()
        .find(|entry| {
            entry.kind() == EntryKind::Volume(VolumeKey::ContainersConfModule) && entry.value().is_continued()
        })
        .ok_or("continued ContainersConfModule must be retained")?;
    assert_eq!(continued.value().continuations()[0].text(), "module");

    let wrong_section = QuadletDocument::parse(
        QuadletUnitType::Build,
        SourceId::new(314),
        "[Build]\nImageTag=example.invalid/app\nContainersConfModule=build.conf\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(
        wrong_section
            .document()
            .entries()
            .any(|entry| { entry.kind() == EntryKind::Build(BuildKey::ContainersConfModule) })
    );
    let container = QuadletDocument::parse(
        QuadletUnitType::Container,
        SourceId::new(315),
        "[Container]\nImage=example.invalid/app\nContainersConfModule=container.conf\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(
        container
            .document()
            .entries()
            .any(|entry| entry.kind() == EntryKind::Container(ContainerKey::ContainersConfModule))
    );
    Ok(())
}

#[test]
fn volume_global_args_preserves_every_opaque_physical_value_and_scope() -> Result<(), String> {
    let source = concat!(
        "[Volume]\n",
        "GlobalArgs=pre-one\n",
        "GlobalArgs=pre-two\n",
        "GlobalArgs=\n",
        "GlobalArgs=--log-level=debug\n",
        "GlobalArgs=\"--events-backend=none\"\n",
        "GlobalArgs=--events-backend=file\\x20value\n",
        "GlobalArgs= malformed \\ value\n",
        "GlobalArgs=continued \\\n",
        " logical\n",
        "globalargs=wrong-case\n",
    );
    let result = QuadletDocument::parse(QuadletUnitType::Volume, SourceId::new(317), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Volume(VolumeKey::GlobalArgs))
            .map(|entry| (entry.value().primary().text(), entry.value_kind()))
            .collect::<Vec<_>>(),
        [
            ("pre-one", ValueKind::Opaque),
            ("pre-two", ValueKind::Opaque),
            ("", ValueKind::Opaque),
            ("--log-level=debug", ValueKind::Opaque),
            ("\"--events-backend=none\"", ValueKind::Opaque),
            ("--events-backend=file\\x20value", ValueKind::Opaque),
            ("malformed \\ value", ValueKind::Opaque),
            ("continued \\", ValueKind::Opaque),
        ]
    );
    let continued = result
        .document()
        .entries()
        .find(|entry| entry.kind() == EntryKind::Volume(VolumeKey::GlobalArgs) && entry.value().is_continued())
        .ok_or("continued GlobalArgs must be retained")?;
    assert_eq!(continued.value().continuations()[0].text(), "logical");
    assert!(
        result
            .document()
            .entries()
            .any(|entry| entry.kind() == EntryKind::Unknown && entry.key().text() == "globalargs")
    );

    let wrong_section = QuadletDocument::parse(
        QuadletUnitType::Container,
        SourceId::new(318),
        "[Container]\nImage=example.invalid/app\nGlobalArgs=--log-level=debug\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(
        wrong_section
            .document()
            .entries()
            .any(|entry| entry.kind() == EntryKind::Container(ContainerKey::GlobalArgs))
    );
    Ok(())
}

#[test]
fn volume_podman_args_preserves_every_opaque_physical_value_and_scope() -> Result<(), String> {
    let source = concat!(
        "[Volume]\n",
        "PodmanArgs=pre-one\n",
        "PodmanArgs=\n",
        "PodmanArgs=--label=post-one\n",
        "PodmanArgs=\"--label=quoted value\"\n",
        "PodmanArgs=--label\\x3descaped\n",
        "PodmanArgs= malformed \\ value\n",
        "PodmanArgs=continued \\\n",
        " logical\n",
        "podmanargs=wrong-case\n",
    );
    let result = QuadletDocument::parse(QuadletUnitType::Volume, SourceId::new(319), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Volume(VolumeKey::PodmanArgs))
            .map(|entry| (entry.value().primary().text(), entry.value_kind()))
            .collect::<Vec<_>>(),
        [
            ("pre-one", ValueKind::Opaque),
            ("", ValueKind::Opaque),
            ("--label=post-one", ValueKind::Opaque),
            ("\"--label=quoted value\"", ValueKind::Opaque),
            ("--label\\x3descaped", ValueKind::Opaque),
            ("malformed \\ value", ValueKind::Opaque),
            ("continued \\", ValueKind::Opaque),
        ]
    );
    let continued = result
        .document()
        .entries()
        .find(|entry| entry.kind() == EntryKind::Volume(VolumeKey::PodmanArgs) && entry.value().is_continued())
        .ok_or("continued PodmanArgs must be retained")?;
    assert_eq!(continued.value().continuations()[0].text(), "logical");
    assert!(
        result
            .document()
            .entries()
            .any(|entry| entry.kind() == EntryKind::Unknown && entry.key().text() == "podmanargs")
    );

    Ok(())
}

#[test]
fn volume_user_preserves_opaque_singleton_physical_lines_and_scope() -> Result<(), String> {
    let source = concat!(
        "[Volume]\n",
        "User=123\n",
        "User=007\n",
        "User=alice\n",
        "User=\n",
        "User= user name \n",
        "User=\"quoted-%i\"\n",
        "User=%h/user\n",
        "User=continued \\\n",
        " text\n",
        "user=wrong-case\n",
    );
    let result = QuadletDocument::parse(QuadletUnitType::Volume, SourceId::new(321), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Volume(VolumeKey::User))
            .map(|entry| (entry.value().primary().text(), entry.value_kind()))
            .collect::<Vec<_>>(),
        [
            ("123", ValueKind::Opaque),
            ("007", ValueKind::Opaque),
            ("alice", ValueKind::Opaque),
            ("", ValueKind::Opaque),
            ("user name ", ValueKind::Opaque),
            ("\"quoted-%i\"", ValueKind::Opaque),
            ("%h/user", ValueKind::Opaque),
            ("continued \\", ValueKind::Opaque),
        ]
    );
    let continued = result
        .document()
        .entries()
        .find(|entry| entry.kind() == EntryKind::Volume(VolumeKey::User) && entry.value().is_continued())
        .ok_or("continued User must be retained")?;
    assert_eq!(continued.value().continuations()[0].text(), "text");
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .filter(|diagnostic| diagnostic.code().as_str() == "QLM0004")
            .count(),
        7
    );
    assert!(
        result
            .document()
            .entries()
            .any(|entry| entry.kind() == EntryKind::Unknown && entry.key().text() == "user")
    );

    let wrong_section = QuadletDocument::parse(
        QuadletUnitType::Network,
        SourceId::new(322),
        "[Network]\nNetworkName=example\nUser=123\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(
        wrong_section
            .document()
            .entries()
            .any(|entry| entry.kind() == EntryKind::Unknown && entry.key().text() == "User")
    );
    Ok(())
}

#[test]
fn volume_group_preserves_opaque_singleton_physical_lines_and_scope() -> Result<(), String> {
    let source = concat!(
        "[Volume]\n",
        "Group=456\n",
        "Group=00456\n",
        "Group=operators\n",
        "Group=\n",
        "Group= group name \n",
        "Group=\"quoted-%i\"\n",
        "Group=%h/group\n",
        "Group=continued \\\n",
        " text\n",
        "group=wrong-case\n",
    );
    let result = QuadletDocument::parse(QuadletUnitType::Volume, SourceId::new(323), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Volume(VolumeKey::Group))
            .map(|entry| (entry.value().primary().text(), entry.value_kind()))
            .collect::<Vec<_>>(),
        [
            ("456", ValueKind::Opaque),
            ("00456", ValueKind::Opaque),
            ("operators", ValueKind::Opaque),
            ("", ValueKind::Opaque),
            ("group name ", ValueKind::Opaque),
            ("\"quoted-%i\"", ValueKind::Opaque),
            ("%h/group", ValueKind::Opaque),
            ("continued \\", ValueKind::Opaque),
        ]
    );
    assert!(
        result
            .document()
            .entries()
            .any(|entry| { entry.kind() == EntryKind::Volume(VolumeKey::Group) && entry.value().is_continued() })
    );
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .filter(|diagnostic| diagnostic.code().as_str() == "QLM0004")
            .count(),
        7
    );
    assert!(
        result
            .document()
            .entries()
            .any(|entry| entry.kind() == EntryKind::Unknown && entry.key().text() == "group")
    );
    Ok(())
}

#[test]
fn volume_uid_preserves_opaque_singleton_physical_lines() -> Result<(), String> {
    let source =
        "[Volume]\nUID=1234\nUID=001234\nUID=name\nUID=\nUID=\"quoted-%i\"\nUID=%h/uid\nUID=continued \\\n text\n";
    let result = QuadletDocument::parse(QuadletUnitType::Volume, SourceId::new(324), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Volume(VolumeKey::UID))
            .map(|entry| (entry.value().primary().text(), entry.value_kind()))
            .collect::<Vec<_>>(),
        [
            ("1234", ValueKind::Opaque),
            ("001234", ValueKind::Opaque),
            ("name", ValueKind::Opaque),
            ("", ValueKind::Opaque),
            ("\"quoted-%i\"", ValueKind::Opaque),
            ("%h/uid", ValueKind::Opaque),
            ("continued \\", ValueKind::Opaque)
        ]
    );
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .filter(|diagnostic| diagnostic.code().as_str() == "QLM0004")
            .count(),
        6
    );
    Ok(())
}

#[test]
fn volume_gid_preserves_opaque_singleton_physical_lines() -> Result<(), String> {
    let source =
        "[Volume]\nGID=5678\nGID=005678\nGID=group\nGID=\nGID=\"quoted-%i\"\nGID=%h/gid\nGID=continued \\\n+ text\n";
    let result = QuadletDocument::parse(QuadletUnitType::Volume, SourceId::new(325), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
    assert!(!EntryKind::Volume(VolumeKey::GID).is_repeatable());
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Volume(VolumeKey::GID))
            .map(|entry| (entry.value().primary().text(), entry.value_kind()))
            .collect::<Vec<_>>(),
        [
            ("5678", ValueKind::Opaque),
            ("005678", ValueKind::Opaque),
            ("group", ValueKind::Opaque),
            ("", ValueKind::Opaque),
            ("\"quoted-%i\"", ValueKind::Opaque),
            ("%h/gid", ValueKind::Opaque),
            ("continued \\", ValueKind::Opaque)
        ]
    );
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .filter(|diagnostic| diagnostic.code().as_str() == "QLM0004")
            .count(),
        6
    );
    Ok(())
}

#[test]
fn volume_service_name_preserves_opaque_singleton_physical_lines() -> Result<(), String> {
    let source = "[Volume]\nServiceName=ordinary\nServiceName=\nServiceName= whitespace \nServiceName=\"quoted-%i\"\nServiceName=explicit.service\nServiceName=%i\nServiceName=escape\\x20text\nServiceName=continued \\\n+ text\nServiceName=\"unmatched\n";
    let result = QuadletDocument::parse(QuadletUnitType::Volume, SourceId::new(326), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
    assert!(!EntryKind::Volume(VolumeKey::ServiceName).is_repeatable());
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Volume(VolumeKey::ServiceName))
            .map(|entry| (entry.value().primary().text(), entry.value_kind()))
            .collect::<Vec<_>>(),
        [
            ("ordinary", ValueKind::Opaque),
            ("", ValueKind::Opaque),
            ("whitespace ", ValueKind::Opaque),
            ("\"quoted-%i\"", ValueKind::Opaque),
            ("explicit.service", ValueKind::Opaque),
            ("%i", ValueKind::Opaque),
            ("escape\\x20text", ValueKind::Opaque),
            ("continued \\", ValueKind::Opaque),
            ("\"unmatched", ValueKind::Opaque),
        ]
    );
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .filter(|diagnostic| diagnostic.code().as_str() == "QLM0004")
            .count(),
        8
    );
    Ok(())
}

#[test]
fn volume_image_preserves_raw_singletons_and_classifies_only_exact_references() -> Result<(), String> {
    let source = "[Volume]\nImage=literal.example/image:1\nImage=unit.image\nImage=unit.build\nImage=lookalike.IMAGE\nImage=quoted.image\nImage=\n";
    let result = QuadletDocument::parse(QuadletUnitType::Volume, SourceId::new(327), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
    assert!(!EntryKind::Volume(VolumeKey::Image).is_repeatable());
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Volume(VolumeKey::Image))
            .map(|entry| (
                entry.value().primary().text(),
                entry.value_kind(),
                entry.unit_reference_name()
            ))
            .collect::<Vec<_>>(),
        [
            ("literal.example/image:1", ValueKind::Opaque, None),
            (
                "unit.image",
                ValueKind::UnitReference(UnitReferenceKind::Image),
                Some("unit.image")
            ),
            (
                "unit.build",
                ValueKind::UnitReference(UnitReferenceKind::Build),
                Some("unit.build")
            ),
            ("lookalike.IMAGE", ValueKind::Opaque, None),
            (
                "quoted.image",
                ValueKind::UnitReference(UnitReferenceKind::Image),
                Some("quoted.image")
            ),
            ("", ValueKind::Opaque, None),
        ]
    );
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .filter(|diagnostic| diagnostic.code().as_str() == "QLM0004")
            .count(),
        5
    );
    Ok(())
}

fn assert_volume_device_and_type_physical_values(document: &QuadletDocument) -> Result<(), String> {
    for (key, expected) in [
        (
            VolumeKey::Device,
            vec![
                "/srv/pre",
                r#""/srv/matched %h""#,
                r#""/srv/unmatched %h"#,
                concat!("/srv/continued ", "\\"),
                "",
            ],
        ),
        (
            VolumeKey::Type,
            vec!["tmpfs", r#""bind %h""#, r#""unmatched %h"#, concat!("bind ", "\\")],
        ),
    ] {
        assert_eq!(
            document
                .entries()
                .filter(|entry| entry.kind() == EntryKind::Volume(key))
                .map(|entry| entry.value().primary().text())
                .collect::<Vec<_>>(),
            expected
        );
    }
    for (key, continuation) in [(VolumeKey::Device, "two"), (VolumeKey::Type, "continued")] {
        let entry = document
            .entries()
            .find(|entry| entry.kind() == EntryKind::Volume(key) && entry.value().is_continued())
            .ok_or_else(|| format!("continued volume {key:?} must be retained"))?;
        assert_eq!(entry.value().continuations()[0].text(), continuation);
    }
    Ok(())
}

#[test]
fn volume_labels_preserve_physical_values_cardinality_and_scope() -> Result<(), String> {
    let source = concat!(
        "[Volume]\n",
        "Label=pre=one\n",
        "Label=pre=two\n",
        "Label=\n",
        "Label=zeta=last\n",
        "Label=alpha=first\n",
        "Label=alpha=final\n",
        "Label=empty=\n",
        "Label=embedded=a=b\n",
        "Label=bare-token\n",
        "Label=\"quoted=%h value\"\n",
        "Label=continued=one \\\n",
        "  two\n",
    );
    let result = QuadletDocument::parse(QuadletUnitType::Volume, SourceId::new(310), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Volume(VolumeKey::Label))
            .map(|entry| entry.value().primary().text())
            .collect::<Vec<_>>(),
        [
            "pre=one",
            "pre=two",
            "",
            "zeta=last",
            "alpha=first",
            "alpha=final",
            "empty=",
            "embedded=a=b",
            "bare-token",
            r#""quoted=%h value""#,
            "continued=one \\",
        ]
    );
    let continued = result
        .document()
        .entries()
        .find(|entry| entry.kind() == EntryKind::Volume(VolumeKey::Label) && entry.value().is_continued())
        .ok_or_else(|| "continued volume label must be retained".to_owned())?;
    assert_eq!(continued.value().continuations()[0].text(), "two");

    let network = QuadletDocument::parse(
        QuadletUnitType::Network,
        SourceId::new(311),
        "[Network]\nLabel=network=value\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(
        network
            .document()
            .entries()
            .any(|entry| entry.kind() == EntryKind::Network(NetworkKey::Label))
    );
    Ok(())
}

#[test]
fn network_labels_preserve_physical_values_cardinality_and_scope() -> Result<(), String> {
    let source = concat!(
        "[Network]\n",
        "Label=pre=one\n",
        "Label=pre=two\n",
        "Label=\n",
        "Label=zeta=last\n",
        "Label=alpha=first\n",
        "Label=alpha=final\n",
        "Label=empty=\n",
        "Label=embedded=a=b\n",
        "Label=bare-token\n",
        "Label=\"quoted=%h value\"\n",
        "Label=continued=one \\\n",
        "  two\n",
    );
    let result = QuadletDocument::parse(QuadletUnitType::Network, SourceId::new(308), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Network(NetworkKey::Label))
            .map(|entry| entry.value().primary().text())
            .collect::<Vec<_>>(),
        [
            "pre=one",
            "pre=two",
            "",
            "zeta=last",
            "alpha=first",
            "alpha=final",
            "empty=",
            "embedded=a=b",
            "bare-token",
            r#""quoted=%h value""#,
            "continued=one \\",
        ]
    );
    let continued = result
        .document()
        .entries()
        .find(|entry| entry.kind() == EntryKind::Network(NetworkKey::Label) && entry.value().is_continued())
        .ok_or_else(|| "continued network label must be retained".to_owned())?;
    assert_eq!(continued.value().continuations()[0].text(), "two");

    let container = QuadletDocument::parse(
        QuadletUnitType::Container,
        SourceId::new(309),
        "[Container]\nImage=example.invalid/app\nLabel=container=value\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(
        container
            .document()
            .entries()
            .any(|entry| entry.kind() == EntryKind::Container(ContainerKey::Label))
    );
    Ok(())
}

#[test]
fn network_internal_and_ipv6_preserve_opaque_values_cardinality_and_scope() -> Result<(), String> {
    let source = concat!(
        "[Network]\n",
        "Internal=true\n",
        "Internal=false\n",
        "Internal=\"Vendor-%n Internal\" \\\n",
        "  continued-internal\n",
        "IPv6=true\n",
        "IPv6=false\n",
        "IPv6=\"Vendor-%n IPv6\"\n",
    );
    let result = QuadletDocument::parse(QuadletUnitType::Network, SourceId::new(308), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        ["QLM0004", "QLM0004", "QLM0004", "QLM0004"]
    );
    let internal: Vec<_> = result
        .document()
        .entries()
        .filter(|entry| entry.kind() == EntryKind::Network(NetworkKey::Internal))
        .collect();
    assert_eq!(internal.len(), 3);
    assert!(internal.iter().all(|entry| entry.value_kind() == ValueKind::Opaque));
    assert_eq!(internal[0].value().primary().text(), "true");
    assert_eq!(internal[1].value().primary().text(), "false");
    assert_eq!(internal[2].value().primary().text(), "\"Vendor-%n Internal\" \\");
    assert_eq!(internal[2].value().continuations()[0].text(), "continued-internal");
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Network(NetworkKey::IPv6))
            .map(|entry| entry.value().primary().text())
            .collect::<Vec<_>>(),
        ["true", "false", r#""Vendor-%n IPv6""#]
    );

    let container = QuadletDocument::parse(
        QuadletUnitType::Container,
        SourceId::new(309),
        "[Container]\nImage=example.invalid/app\nInternal=true\nIPv6=false\n",
    )
    .map_err(|error| error.to_string())?;
    assert_eq!(
        container
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Unknown)
            .map(|entry| entry.key().text())
            .collect::<Vec<_>>(),
        ["Internal", "IPv6"]
    );
    Ok(())
}

#[test]
fn network_ipam_values_preserve_physical_columns_resets_and_scope() -> Result<(), String> {
    let source = concat!(
        "[Network]\n",
        "IPAMDriver=host-local\n",
        "IPAMDriver=\n",
        "Subnet=10.88.0.0/24\n",
        "Subnet=10.89.0.0/24\n",
        "Subnet=\n",
        "Subnet=\"10.90.0.0/24\"\n",
        "Subnet=10.91.0.0/24 \\\n",
        "  continued-subnet\n",
        "Gateway=10.88.0.1\n",
        "Gateway=10.89.0.1\n",
        "Gateway=\n",
        "Gateway=\"10.90.0.1\"\n",
        "Gateway=10.91.0.1\n",
        "IPRange=10.88.0.64/26\n",
        "IPRange=10.89.0.64/26\n",
        "IPRange=\n",
        "IPRange=\"10.90.0.64/26\"\n",
        "IPRange=10.91.0.64/26\n",
    );
    let result = QuadletDocument::parse(QuadletUnitType::Network, SourceId::new(309), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Network(NetworkKey::IPAMDriver))
            .map(|entry| entry.value().primary().text())
            .collect::<Vec<_>>(),
        ["host-local", ""]
    );
    assert_network_ipam_columns(&result)?;
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        ["QLM0004"]
    );

    let container = QuadletDocument::parse(
        QuadletUnitType::Container,
        SourceId::new(310),
        "[Container]\nImage=example.invalid/app\nIPAMDriver=host-local\nSubnet=10.88.0.0/24\nGateway=10.88.0.1\nIPRange=10.88.0.64/26\n",
    )
    .map_err(|error| error.to_string())?;
    assert_eq!(
        container
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Unknown)
            .map(|entry| entry.key().text())
            .collect::<Vec<_>>(),
        ["IPAMDriver", "Subnet", "Gateway", "IPRange"]
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
fn apparmor_is_a_container_only_opaque_singleton_with_recoverable_duplicates() -> Result<(), String> {
    let source = concat!(
        "[Container]\n",
        "Image=example.invalid/app\n",
        "AppArmor=unconfined\n",
        "AppArmor=\n",
        "AppArmor=\"Authored Profile\"\n",
        "AppArmor= profile:with %i \n",
        "AppArmor=malformed:value:extra\n",
        "apparmor=case-sensitive-unknown\n",
        "[Build]\n",
        "AppArmor=build-unknown\n",
    );
    let result = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(140), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid());
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::AppArmor))
            .map(|entry| {
                assert_eq!(entry.value_kind(), ValueKind::Opaque);
                entry.value().primary().text()
            })
            .collect::<Vec<_>>(),
        [
            "unconfined",
            "",
            r#""Authored Profile""#,
            "profile:with %i ",
            "malformed:value:extra",
        ]
    );
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        ["QLM0003", "QLM0004", "QLM0004", "QLM0004", "QLM0004"]
    );
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Unknown)
            .map(|entry| (entry.key().text(), entry.value().primary().text()))
            .collect::<Vec<_>>(),
        [("apparmor", "case-sensitive-unknown"), ("AppArmor", "build-unknown")]
    );
    Ok(())
}

#[test]
fn no_new_privileges_is_a_container_only_opaque_singleton_with_recoverable_duplicates() -> Result<(), String> {
    let source = concat!(
        "[Container]\n",
        "Image=example.invalid/app\n",
        "NoNewPrivileges=true\n",
        "NoNewPrivileges=yes\n",
        "NoNewPrivileges=false\n",
        "NoNewPrivileges=\n",
        "NoNewPrivileges=\"true\"\n",
        "NoNewPrivileges= %i \n",
        "NoNewPrivileges=not-a-boolean\n",
        "nonewprivileges=case-sensitive-unknown\n",
        "[Build]\n",
        "NoNewPrivileges=build-unknown\n",
    );
    let result = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(149), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid());
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::NoNewPrivileges))
            .map(|entry| {
                assert_eq!(entry.value_kind(), ValueKind::Opaque);
                entry.value().primary().text()
            })
            .collect::<Vec<_>>(),
        ["true", "yes", "false", "", r#""true""#, "%i ", "not-a-boolean"]
    );
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        [
            "QLM0003", "QLM0004", "QLM0004", "QLM0004", "QLM0004", "QLM0004", "QLM0004"
        ]
    );
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Unknown)
            .map(|entry| (entry.key().text(), entry.value().primary().text()))
            .collect::<Vec<_>>(),
        [
            ("nonewprivileges", "case-sensitive-unknown"),
            ("NoNewPrivileges", "build-unknown")
        ]
    );

    let pod = QuadletDocument::parse(
        QuadletUnitType::Pod,
        SourceId::new(150),
        "[Pod]\nNoNewPrivileges=true\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(pod.document().entries().any(|entry| {
        entry.kind() == EntryKind::Unknown
            && entry.key().text() == "NoNewPrivileges"
            && entry.value().primary().text() == "true"
    }));
    Ok(())
}

#[test]
fn seccomp_profile_is_a_container_only_opaque_singleton_with_recoverable_duplicates() -> Result<(), String> {
    let source = concat!(
        "[Container]\n",
        "Image=example.invalid/app\n",
        "SeccompProfile=unconfined\n",
        "SeccompProfile=/tmp/profile.json\n",
        "SeccompProfile=\n",
        "SeccompProfile=\"\"\n",
        "SeccompProfile= \"/tmp/Authored Profile.json\" \n",
        "SeccompProfile=%h/profiles/%i.json\n",
        "SeccompProfile=malformed:value\n",
        "seccompprofile=case-sensitive-unknown\n",
        "[Build]\n",
        "SeccompProfile=build-unknown\n",
        "[Service]\n",
        "SeccompProfile=service-unknown\n",
    );
    let result = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(151), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid());
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::SeccompProfile))
            .map(|entry| {
                assert_eq!(entry.value_kind(), ValueKind::Opaque);
                entry.value().primary().text()
            })
            .collect::<Vec<_>>(),
        [
            "unconfined",
            "/tmp/profile.json",
            "",
            "\"\"",
            r#""/tmp/Authored Profile.json" "#,
            "%h/profiles/%i.json",
            "malformed:value",
        ]
    );
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        [
            "QLM0003", "QLM0004", "QLM0004", "QLM0004", "QLM0004", "QLM0004", "QLM0004"
        ]
    );
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Unknown)
            .map(|entry| (entry.key().text(), entry.value().primary().text()))
            .collect::<Vec<_>>(),
        [
            ("seccompprofile", "case-sensitive-unknown"),
            ("SeccompProfile", "build-unknown"),
        ]
    );
    assert!(result.document().entries().any(|entry| {
        entry.kind() == EntryKind::GenericSystemd
            && entry.key().text() == "SeccompProfile"
            && entry.value().primary().text() == "service-unknown"
    }));

    let pod = QuadletDocument::parse(
        QuadletUnitType::Pod,
        SourceId::new(152),
        "[Pod]\nSeccompProfile=unconfined\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(pod.document().entries().any(|entry| {
        entry.kind() == EntryKind::Unknown
            && entry.key().text() == "SeccompProfile"
            && entry.value().primary().text() == "unconfined"
    }));
    Ok(())
}

#[test]
fn security_label_disable_is_a_container_only_opaque_singleton_with_recoverable_duplicates() -> Result<(), String> {
    let source = concat!(
        "[Container]\n",
        "Image=example.invalid/app\n",
        "SecurityLabelDisable=true\n",
        "SecurityLabelDisable=false\n",
        "SecurityLabelDisable=\n",
        "SecurityLabelDisable=\"true\"\n",
        "SecurityLabelDisable= \" false \" \n",
        "SecurityLabelDisable=%i\n",
        "SecurityLabelDisable=not-a-boolean\n",
        "securitylabeldisable=case-sensitive-unknown\n",
        "[Build]\n",
        "SecurityLabelDisable=build-unknown\n",
        "[Service]\n",
        "SecurityLabelDisable=service-unknown\n",
    );
    let result = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(153), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid());
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::SecurityLabelDisable))
            .map(|entry| {
                assert_eq!(entry.value_kind(), ValueKind::Opaque);
                entry.value().primary().text()
            })
            .collect::<Vec<_>>(),
        ["true", "false", "", r#""true""#, r#"" false " "#, "%i", "not-a-boolean"]
    );
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        [
            "QLM0003", "QLM0004", "QLM0004", "QLM0004", "QLM0004", "QLM0004", "QLM0004"
        ]
    );
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Unknown)
            .map(|entry| (entry.key().text(), entry.value().primary().text()))
            .collect::<Vec<_>>(),
        [
            ("securitylabeldisable", "case-sensitive-unknown"),
            ("SecurityLabelDisable", "build-unknown"),
        ]
    );
    assert!(result.document().entries().any(|entry| {
        entry.kind() == EntryKind::GenericSystemd
            && entry.key().text() == "SecurityLabelDisable"
            && entry.value().primary().text() == "service-unknown"
    }));

    let pod = QuadletDocument::parse(
        QuadletUnitType::Pod,
        SourceId::new(154),
        "[Pod]\nSecurityLabelDisable=true\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(pod.document().entries().any(|entry| {
        entry.kind() == EntryKind::Unknown
            && entry.key().text() == "SecurityLabelDisable"
            && entry.value().primary().text() == "true"
    }));
    Ok(())
}

#[test]
fn security_label_file_type_is_a_container_only_opaque_singleton_with_recoverable_duplicates() -> Result<(), String> {
    let source = concat!(
        "[Container]\n",
        "Image=example.invalid/app\n",
        "SecurityLabelFileType=container_file_t\n",
        "SecurityLabelFileType=custom_file_t\n",
        "SecurityLabelFileType=\n",
        "SecurityLabelFileType=\"container_file_t\"\n",
        "SecurityLabelFileType= custom file type \n",
        "SecurityLabelFileType=%i_file_t\n",
        "SecurityLabelFileType=malformed:type\n",
        "securitylabelfiletype=case-sensitive-unknown\n",
        "[Build]\n",
        "SecurityLabelFileType=build-unknown\n",
        "[Service]\n",
        "SecurityLabelFileType=service-unknown\n",
    );
    let result = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(155), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid());
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::SecurityLabelFileType))
            .map(|entry| {
                assert_eq!(entry.value_kind(), ValueKind::Opaque);
                entry.value().primary().text()
            })
            .collect::<Vec<_>>(),
        [
            "container_file_t",
            "custom_file_t",
            "",
            r#""container_file_t""#,
            "custom file type ",
            "%i_file_t",
            "malformed:type",
        ]
    );
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        [
            "QLM0003", "QLM0004", "QLM0004", "QLM0004", "QLM0004", "QLM0004", "QLM0004"
        ]
    );
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Unknown)
            .map(|entry| (entry.key().text(), entry.value().primary().text()))
            .collect::<Vec<_>>(),
        [
            ("securitylabelfiletype", "case-sensitive-unknown"),
            ("SecurityLabelFileType", "build-unknown"),
        ]
    );
    assert!(result.document().entries().any(|entry| {
        entry.kind() == EntryKind::GenericSystemd
            && entry.key().text() == "SecurityLabelFileType"
            && entry.value().primary().text() == "service-unknown"
    }));

    let pod = QuadletDocument::parse(
        QuadletUnitType::Pod,
        SourceId::new(156),
        "[Pod]\nSecurityLabelFileType=container_file_t\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(pod.document().entries().any(|entry| {
        entry.kind() == EntryKind::Unknown
            && entry.key().text() == "SecurityLabelFileType"
            && entry.value().primary().text() == "container_file_t"
    }));
    Ok(())
}

#[test]
fn security_label_level_is_a_container_only_opaque_singleton_with_recoverable_duplicates() -> Result<(), String> {
    let source = concat!(
        "[Container]\n",
        "Image=example.invalid/app\n",
        "SecurityLabelLevel=s0:c1,c2\n",
        "SecurityLabelLevel=s0:c3,c4\n",
        "SecurityLabelLevel=\n",
        "SecurityLabelLevel=\"s0:c5,c6\"\n",
        "SecurityLabelLevel= s0 : c7,c8 \n",
        "SecurityLabelLevel=%i:c9,c10\n",
        "SecurityLabelLevel=malformed level\n",
        "securitylabellevel=case-sensitive-unknown\n",
        "[Build]\n",
        "SecurityLabelLevel=build-unknown\n",
        "[Service]\n",
        "SecurityLabelLevel=service-unknown\n",
    );
    let result = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(157), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid());
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::SecurityLabelLevel))
            .map(|entry| {
                assert_eq!(entry.value_kind(), ValueKind::Opaque);
                entry.value().primary().text()
            })
            .collect::<Vec<_>>(),
        [
            "s0:c1,c2",
            "s0:c3,c4",
            "",
            r#""s0:c5,c6""#,
            "s0 : c7,c8 ",
            "%i:c9,c10",
            "malformed level",
        ]
    );
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        [
            "QLM0003", "QLM0004", "QLM0004", "QLM0004", "QLM0004", "QLM0004", "QLM0004"
        ]
    );
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Unknown)
            .map(|entry| (entry.key().text(), entry.value().primary().text()))
            .collect::<Vec<_>>(),
        [
            ("securitylabellevel", "case-sensitive-unknown"),
            ("SecurityLabelLevel", "build-unknown"),
        ]
    );
    assert!(result.document().entries().any(|entry| {
        entry.kind() == EntryKind::GenericSystemd
            && entry.key().text() == "SecurityLabelLevel"
            && entry.value().primary().text() == "service-unknown"
    }));

    let pod = QuadletDocument::parse(
        QuadletUnitType::Pod,
        SourceId::new(158),
        "[Pod]\nSecurityLabelLevel=s0:c1,c2\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(pod.document().entries().any(|entry| {
        entry.kind() == EntryKind::Unknown
            && entry.key().text() == "SecurityLabelLevel"
            && entry.value().primary().text() == "s0:c1,c2"
    }));
    Ok(())
}

#[test]
fn security_label_nested_is_a_container_only_opaque_singleton_with_recoverable_duplicates() -> Result<(), String> {
    let source = concat!(
        "[Container]\n",
        "Image=example.invalid/app\n",
        "SecurityLabelNested=true\n",
        "SecurityLabelNested=false\n",
        "SecurityLabelNested=\n",
        "SecurityLabelNested=\"true\"\n",
        "SecurityLabelNested= false \n",
        "SecurityLabelNested=%i\n",
        "SecurityLabelNested=not-a-boolean\n",
        "securitylabelnested=case-sensitive-unknown\n",
        "[Build]\n",
        "SecurityLabelNested=build-unknown\n",
        "[Service]\n",
        "SecurityLabelNested=service-unknown\n",
    );
    let result = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(159), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid());
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::SecurityLabelNested))
            .map(|entry| {
                assert_eq!(entry.value_kind(), ValueKind::Opaque);
                entry.value().primary().text()
            })
            .collect::<Vec<_>>(),
        ["true", "false", "", r#""true""#, "false ", "%i", "not-a-boolean"]
    );
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        [
            "QLM0003", "QLM0004", "QLM0004", "QLM0004", "QLM0004", "QLM0004", "QLM0004"
        ]
    );
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Unknown)
            .map(|entry| (entry.key().text(), entry.value().primary().text()))
            .collect::<Vec<_>>(),
        [
            ("securitylabelnested", "case-sensitive-unknown"),
            ("SecurityLabelNested", "build-unknown"),
        ]
    );
    assert!(result.document().entries().any(|entry| {
        entry.kind() == EntryKind::GenericSystemd
            && entry.key().text() == "SecurityLabelNested"
            && entry.value().primary().text() == "service-unknown"
    }));

    let pod = QuadletDocument::parse(
        QuadletUnitType::Pod,
        SourceId::new(160),
        "[Pod]\nSecurityLabelNested=true\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(pod.document().entries().any(|entry| {
        entry.kind() == EntryKind::Unknown
            && entry.key().text() == "SecurityLabelNested"
            && entry.value().primary().text() == "true"
    }));
    Ok(())
}

#[test]
fn security_label_type_is_a_container_only_opaque_singleton_with_recoverable_duplicates() -> Result<(), String> {
    let source = concat!(
        "[Container]\n",
        "Image=example.invalid/app\n",
        "SecurityLabelType=container_t\n",
        "SecurityLabelType=custom_t\n",
        "SecurityLabelType=\n",
        "SecurityLabelType=\"container_t\"\n",
        "SecurityLabelType= custom type \n",
        "SecurityLabelType=%i_t\n",
        "SecurityLabelType=malformed:type\n",
        "securitylabeltype=case-sensitive-unknown\n",
        "[Build]\n",
        "SecurityLabelType=build-unknown\n",
        "[Service]\n",
        "SecurityLabelType=service-unknown\n",
    );
    let result = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(161), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid());
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::SecurityLabelType))
            .map(|entry| {
                assert_eq!(entry.value_kind(), ValueKind::Opaque);
                entry.value().primary().text()
            })
            .collect::<Vec<_>>(),
        [
            "container_t",
            "custom_t",
            "",
            r#""container_t""#,
            "custom type ",
            "%i_t",
            "malformed:type",
        ]
    );
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        [
            "QLM0003", "QLM0004", "QLM0004", "QLM0004", "QLM0004", "QLM0004", "QLM0004"
        ]
    );
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Unknown)
            .map(|entry| (entry.key().text(), entry.value().primary().text()))
            .collect::<Vec<_>>(),
        [
            ("securitylabeltype", "case-sensitive-unknown"),
            ("SecurityLabelType", "build-unknown"),
        ]
    );
    assert!(result.document().entries().any(|entry| {
        entry.kind() == EntryKind::GenericSystemd
            && entry.key().text() == "SecurityLabelType"
            && entry.value().primary().text() == "service-unknown"
    }));

    let pod = QuadletDocument::parse(
        QuadletUnitType::Pod,
        SourceId::new(162),
        "[Pod]\nSecurityLabelType=container_t\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(pod.document().entries().any(|entry| {
        entry.kind() == EntryKind::Unknown
            && entry.key().text() == "SecurityLabelType"
            && entry.value().primary().text() == "container_t"
    }));
    Ok(())
}

#[test]
fn mask_is_container_only_repeatable_and_preserves_every_opaque_physical_value() -> Result<(), String> {
    let authored = [
        "/pre/one:/pre/two",
        "/pre/one:/pre/two",
        "",
        r#""/quoted/path:/quoted/other""#,
        "%h/private:%t/shared",
        "relative path:other path",
        "/malformed::path:",
        "/proc/acpi:/sys/firmware",
    ];
    let mut source = "[Container]\nImage=example.invalid/app\n".to_owned();
    for value in authored {
        source.push_str("Mask=");
        source.push_str(value);
        source.push('\n');
    }
    source.push_str(concat!(
        "mask=case-sensitive-unknown\n",
        "[Build]\n",
        "Mask=build-unknown\n",
        "[Service]\n",
        "Mask=service-unknown\n",
    ));

    let result = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(163), source.clone())
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::Mask))
            .map(|entry| {
                assert_eq!(entry.value_kind(), ValueKind::Opaque);
                entry.value().primary().text()
            })
            .collect::<Vec<_>>(),
        authored
    );
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        ["QLM0003"]
    );
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Unknown)
            .map(|entry| (entry.key().text(), entry.value().primary().text()))
            .collect::<Vec<_>>(),
        [("mask", "case-sensitive-unknown"), ("Mask", "build-unknown")]
    );
    assert!(result.document().entries().any(|entry| {
        entry.kind() == EntryKind::GenericSystemd
            && entry.key().text() == "Mask"
            && entry.value().primary().text() == "service-unknown"
    }));

    let pod = QuadletDocument::parse(
        QuadletUnitType::Pod,
        SourceId::new(164),
        "[Pod]\nMask=/proc/acpi:/sys/firmware\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(pod.document().entries().any(|entry| {
        entry.kind() == EntryKind::Unknown
            && entry.key().text() == "Mask"
            && entry.value().primary().text() == "/proc/acpi:/sys/firmware"
    }));
    Ok(())
}

#[test]
fn unmask_is_container_only_repeatable_and_preserves_every_opaque_physical_value() -> Result<(), String> {
    let authored = [
        "/pre/one:/pre/two",
        "/pre/one:/pre/two",
        "",
        "ALL",
        "/proc/acpi:/sys/firmware",
        r#""/quoted/%h/*:/sys/*""#,
        "%h/private:/proc/*",
        "/proc/acpi : /sys/firmware ",
        "malformed::path:",
    ];
    let mut source = "[Container]\nImage=example.invalid/app\n".to_owned();
    for value in authored {
        source.push_str("Unmask=");
        source.push_str(value);
        source.push('\n');
    }
    source.push_str(concat!(
        "unmask=case-sensitive-unknown\n",
        "[Build]\n",
        "Unmask=build-unknown\n",
        "[Service]\n",
        "Unmask=service-unknown\n",
    ));

    let result = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(165), source.clone())
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::Unmask))
            .map(|entry| {
                assert_eq!(entry.value_kind(), ValueKind::Opaque);
                entry.value().primary().text()
            })
            .collect::<Vec<_>>(),
        authored
    );
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        ["QLM0003"]
    );
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Unknown)
            .map(|entry| (entry.key().text(), entry.value().primary().text()))
            .collect::<Vec<_>>(),
        [("unmask", "case-sensitive-unknown"), ("Unmask", "build-unknown")]
    );
    assert!(result.document().entries().any(|entry| {
        entry.kind() == EntryKind::GenericSystemd
            && entry.key().text() == "Unmask"
            && entry.value().primary().text() == "service-unknown"
    }));

    let pod = QuadletDocument::parse(QuadletUnitType::Pod, SourceId::new(166), "[Pod]\nUnmask=ALL\n")
        .map_err(|error| error.to_string())?;
    assert!(pod.document().entries().any(|entry| {
        entry.kind() == EntryKind::Unknown && entry.key().text() == "Unmask" && entry.value().primary().text() == "ALL"
    }));
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

fn expected_fixture_core_container_keys() -> &'static [ContainerKey] {
    &[
        ContainerKey::ContainerName,
        ContainerKey::AddHost,
        ContainerKey::AddHost,
        ContainerKey::Image,
        ContainerKey::Entrypoint,
        ContainerKey::RunInit,
        ContainerKey::StopSignal,
        ContainerKey::StopTimeout,
        ContainerKey::Pull,
        ContainerKey::PidsLimit,
        ContainerKey::HostName,
        ContainerKey::ShmSize,
        ContainerKey::Memory,
        ContainerKey::AppArmor,
        ContainerKey::NoNewPrivileges,
        ContainerKey::SeccompProfile,
        ContainerKey::SecurityLabelDisable,
        ContainerKey::SecurityLabelFileType,
        ContainerKey::SecurityLabelLevel,
        ContainerKey::SecurityLabelNested,
        ContainerKey::SecurityLabelType,
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
}

fn is_extended_opaque_container_key(key: ContainerKey) -> bool {
    matches!(
        key,
        ContainerKey::DropCapability
            | ContainerKey::AddCapability
            | ContainerKey::Tmpfs
            | ContainerKey::Sysctl
            | ContainerKey::Ulimit
            | ContainerKey::AddDevice
            | ContainerKey::DNS
            | ContainerKey::DNSOption
            | ContainerKey::DNSSearch
            | ContainerKey::ExposeHostPort
            | ContainerKey::Annotation
            | ContainerKey::Mask
            | ContainerKey::Unmask
    )
}

fn assert_fixture_ulimits(result: &quadlet_lens::model::QuadletParseResult) {
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::Ulimit))
            .map(|entry| entry.value().primary().text())
            .collect::<Vec<_>>(),
        [
            "Core=0:0",
            r#"nofile="1024:2048""#,
            "stack=%h:%n",
            "",
            "nproc=4096:8192",
            "nproc=4096:8192",
        ]
    );
}

fn assert_fixture_add_devices(result: &quadlet_lens::model::QuadletParseResult) {
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::AddDevice))
            .map(|entry| entry.value().primary().text())
            .collect::<Vec<_>>(),
        [
            "/dev/null:/dev/pre-null:r /dev/zero:/dev/pre-zero:w",
            "",
            r#""/dev/null:/dev/final null:r" /dev/zero:/dev/final-zero:w"#,
            "%h/Device:/dev/MixedCase:rwm",
            "-/dev/optional:/dev/optional:r",
            r#""/dev/null:/dev/final null:r" /dev/zero:/dev/final-zero:w"#,
        ]
    );
}

fn assert_fixture_memory(result: &quadlet_lens::model::QuadletParseResult) -> Result<(), String> {
    assert_eq!(
        container_entry(result, ContainerKey::Memory, 0)?
            .value()
            .primary()
            .text(),
        "16777216b"
    );
    Ok(())
}

fn assert_network_ipam_columns(result: &quadlet_lens::model::QuadletParseResult) -> Result<(), String> {
    for (key, expected) in [
        (
            NetworkKey::Subnet,
            vec![
                "10.88.0.0/24",
                "10.89.0.0/24",
                "",
                r#""10.90.0.0/24""#,
                "10.91.0.0/24 \\",
            ],
        ),
        (
            NetworkKey::Gateway,
            vec!["10.88.0.1", "10.89.0.1", "", r#""10.90.0.1""#, "10.91.0.1"],
        ),
        (
            NetworkKey::IPRange,
            vec![
                "10.88.0.64/26",
                "10.89.0.64/26",
                "",
                r#""10.90.0.64/26""#,
                "10.91.0.64/26",
            ],
        ),
    ] {
        let entries: Vec<_> = result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Network(key))
            .collect();
        assert_eq!(
            entries
                .iter()
                .map(|entry| entry.value().primary().text())
                .collect::<Vec<_>>(),
            expected
        );
        assert!(entries.iter().all(|entry| entry.value_kind() == ValueKind::Opaque));
    }
    let subnet = result
        .document()
        .entries()
        .find(|entry| entry.kind() == EntryKind::Network(NetworkKey::Subnet) && entry.value().is_continued())
        .ok_or_else(|| "continued subnet must be retained".to_owned())?;
    assert_eq!(subnet.value().continuations()[0].text(), "continued-subnet");
    Ok(())
}

fn assert_fixture_apparmor(result: &quadlet_lens::model::QuadletParseResult) -> Result<(), String> {
    assert_eq!(
        container_entry(result, ContainerKey::AppArmor, 0)?
            .value()
            .primary()
            .text(),
        "unconfined"
    );
    Ok(())
}

fn assert_fixture_no_new_privileges(result: &quadlet_lens::model::QuadletParseResult) -> Result<(), String> {
    assert_eq!(
        container_entry(result, ContainerKey::NoNewPrivileges, 0)?
            .value()
            .primary()
            .text(),
        "true"
    );
    Ok(())
}

fn assert_fixture_seccomp_profile(result: &quadlet_lens::model::QuadletParseResult) -> Result<(), String> {
    assert_eq!(
        container_entry(result, ContainerKey::SeccompProfile, 0)?
            .value()
            .primary()
            .text(),
        "unconfined"
    );
    Ok(())
}

fn assert_fixture_security_label_disable(result: &quadlet_lens::model::QuadletParseResult) -> Result<(), String> {
    assert_eq!(
        container_entry(result, ContainerKey::SecurityLabelDisable, 0)?
            .value()
            .primary()
            .text(),
        "true"
    );
    Ok(())
}

fn assert_fixture_security_label_file_type(result: &quadlet_lens::model::QuadletParseResult) -> Result<(), String> {
    assert_eq!(
        container_entry(result, ContainerKey::SecurityLabelFileType, 0)?
            .value()
            .primary()
            .text(),
        "container_file_t"
    );
    Ok(())
}

fn assert_fixture_security_label_level(result: &quadlet_lens::model::QuadletParseResult) -> Result<(), String> {
    assert_eq!(
        container_entry(result, ContainerKey::SecurityLabelLevel, 0)?
            .value()
            .primary()
            .text(),
        "s0:c1,c2"
    );
    Ok(())
}

fn assert_fixture_security_label_nested(result: &quadlet_lens::model::QuadletParseResult) -> Result<(), String> {
    assert_eq!(
        container_entry(result, ContainerKey::SecurityLabelNested, 0)?
            .value()
            .primary()
            .text(),
        "true"
    );
    Ok(())
}

fn assert_fixture_security_label_type(result: &quadlet_lens::model::QuadletParseResult) -> Result<(), String> {
    assert_eq!(
        container_entry(result, ContainerKey::SecurityLabelType, 0)?
            .value()
            .primary()
            .text(),
        "container_t"
    );
    Ok(())
}

fn assert_fixture_masks(result: &quadlet_lens::model::QuadletParseResult) {
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::Mask))
            .map(|entry| entry.value().primary().text())
            .collect::<Vec<_>>(),
        ["/pre/mask", "", "/proc/acpi:/sys/firmware"]
    );
}

fn assert_fixture_unmasks(result: &quadlet_lens::model::QuadletParseResult) {
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::Unmask))
            .map(|entry| entry.value().primary().text())
            .collect::<Vec<_>>(),
        ["/pre/unmask", "", "ALL", "/proc/acpi:/sys/firmware"]
    );
}

fn assert_fixture_security_singletons(result: &quadlet_lens::model::QuadletParseResult) -> Result<(), String> {
    assert_fixture_apparmor(result)?;
    assert_fixture_no_new_privileges(result)?;
    assert_fixture_seccomp_profile(result)?;
    assert_fixture_security_label_disable(result)?;
    assert_fixture_security_label_file_type(result)?;
    assert_fixture_security_label_level(result)?;
    assert_fixture_security_label_nested(result)?;
    assert_fixture_security_label_type(result)?;
    assert_fixture_masks(result);
    assert_fixture_unmasks(result);
    Ok(())
}

fn assert_fixture_dns(result: &quadlet_lens::model::QuadletParseResult) {
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::DNS))
            .map(|entry| entry.value().primary().text())
            .collect::<Vec<_>>(),
        ["1.1.1.1", "1.1.1.1", "", "9.9.9.9", "2001:4860:4860::8888"]
    );
}

fn assert_fixture_networking_values(result: &quadlet_lens::model::QuadletParseResult) {
    assert_fixture_dns(result);
    assert_fixture_dns_options(result);
    assert_fixture_dns_searches_and_exposed_host_ports(result);
    assert_fixture_annotations(result);
}

fn assert_fixture_dns_options(result: &quadlet_lens::model::QuadletParseResult) {
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::DNSOption))
            .map(|entry| entry.value().primary().text())
            .collect::<Vec<_>>(),
        ["rotate", "rotate", "", "ndots:1", "use-vc"]
    );
}

fn assert_fixture_dns_searches_and_exposed_host_ports(result: &quadlet_lens::model::QuadletParseResult) {
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::DNSSearch))
            .map(|entry| entry.value().primary().text())
            .collect::<Vec<_>>(),
        ["pre.example.com", "pre.example.com", "", "dc1.example.com", "."]
    );
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::ExposeHostPort))
            .map(|entry| entry.value().primary().text())
            .collect::<Vec<_>>(),
        [
            "1000",
            "1000",
            "",
            "3000",
            "8080-8085",
            "9090/tcp",
            "5353/udp",
            "5353/sctp"
        ]
    );
}

fn assert_fixture_annotations(result: &quadlet_lens::model::QuadletParseResult) {
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::Annotation))
            .map(|entry| entry.value().primary().text())
            .collect::<Vec<_>>(),
        [
            "org.example.name=first",
            "org.example.name=first",
            "",
            "org.example.name=final",
            r#""org.example.quoted=Authored Value""#,
            "org.example.specifier=%i",
            "key-only",
            "malformed = value",
        ]
    );
}

fn container_entry(
    result: &quadlet_lens::model::QuadletParseResult,
    key: ContainerKey,
    occurrence: usize,
) -> Result<&TypedEntry, String> {
    result
        .document()
        .entries()
        .filter(|entry| entry.kind() == EntryKind::Container(key))
        .nth(occurrence)
        .ok_or_else(|| format!("fixture has no {key:?} occurrence {occurrence}"))
}

#[test]
fn container_batch_keys_preserve_opaque_values_repetition_and_relationship_diagnostics() -> Result<(), String> {
    let source = concat!(
        "[Container]\n",
        "Image=example.invalid/app:1\n",
        "AutoUpdate=registry\n",
        "CgroupsMode=split\n",
        "EnvironmentHost=false\n",
        "GIDMap=0:100000:65536\n",
        "GIDMap=1:200000:1\n",
        "HttpProxy=true\n",
        "Mount=type=volume,src=data.volume,dst=/data\n",
        "Mount=type=image,src=assets.image,dst=/assets\n",
        "ReadOnlyTmpfs=true\n",
        "Retry=4\n",
        "RetryDelay=7s\n",
        "StartWithPod=false\n",
        "SubGIDMap=keep-id\n",
        "SubUIDMap=keep-id\n",
        "Timezone=Europe/Berlin\n",
        "UIDMap=0:100000:65536\n",
        "HealthOnFailure=kill\n",
    );
    let result = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(980), source)
        .map_err(|error| error.to_string())?;
    assert_eq!(result.syntax().document().render_preserved(), source);
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::GIDMap))
            .map(|entry| entry.value().primary().text())
            .collect::<Vec<_>>(),
        ["0:100000:65536", "1:200000:1"]
    );
    for key in [
        ContainerKey::AutoUpdate,
        ContainerKey::CgroupsMode,
        ContainerKey::EnvironmentHost,
        ContainerKey::HttpProxy,
        ContainerKey::Mount,
        ContainerKey::ReadOnlyTmpfs,
        ContainerKey::Retry,
        ContainerKey::RetryDelay,
        ContainerKey::StartWithPod,
        ContainerKey::SubGIDMap,
        ContainerKey::SubUIDMap,
        ContainerKey::Timezone,
        ContainerKey::UIDMap,
        ContainerKey::HealthOnFailure,
    ] {
        assert!(
            result
                .document()
                .entries()
                .any(|entry| { entry.kind() == EntryKind::Container(key) && entry.value_kind() == ValueKind::Opaque })
        );
    }
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        ["QLM0012", "QLM0014", "QLM0015"]
    );
    Ok(())
}

#[test]
fn container_mapping_conflicts_have_stable_source_spanned_diagnostics() -> Result<(), String> {
    let source = concat!(
        "[Container]\nImage=example.invalid/app:1\nPod=app.pod\nUserNS=keep-id\n",
        "UIDMap=0:100000:65536\nGIDMap=0:100000:65536\n",
    );
    let result = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(981), source)
        .map_err(|error| error.to_string())?;
    assert_eq!(
        result
            .model_diagnostics()
            .iter()
            .map(|diagnostic| (diagnostic.code().as_str(), diagnostic.labels()[0].span().start()))
            .collect::<Vec<_>>(),
        [("QLM0013", 52), ("QLM0016", 40)]
    );
    Ok(())
}

#[test]
fn container_relationship_diagnostics_use_only_effective_active_values() -> Result<(), String> {
    let cases = [
        ("Pod=\nStartWithPod=true\n", vec!["QLM0011"]),
        ("StartWithPod=false\n", vec![]),
        ("Pod=\nStartWithPod=\"YES\"\n", vec!["QLM0011"]),
        ("Pod=\nStartWithPod=\"off\"\n", vec![]),
        ("Pod=\nStartWithPod=\\\"true\\\"\n", vec![]),
        ("Pod=\nStartWithPod=tr\\x75e\n", vec![]),
        ("StartWithPod=opaque\n", vec![]),
        ("Pod=app.pod\nStartWithPod=true\n", vec![]),
        ("ReadOnlyTmpfs=true\n", vec!["QLM0012"]),
        ("ReadOnly=false\nReadOnlyTmpfs=true\n", vec!["QLM0012"]),
        ("ReadOnly=true\nReadOnlyTmpfs=true\n", vec![]),
        ("ReadOnly=\"On\"\nReadOnlyTmpfs=\"true\"\n", vec![]),
        ("ReadOnly=\"off\"\nReadOnlyTmpfs=\"YES\"\n", vec!["QLM0012"]),
        ("ReadOnlyTmpfs=\\\"true\\\"\n", vec![]),
        ("ReadOnlyTmpfs=tr\\x75e\n", vec![]),
        ("ReadOnlyTmpfs=false\n", vec![]),
        ("ReadOnlyTmpfs=opaque\n", vec![]),
        ("UserNS=\nUIDMap=\n", vec![]),
        ("UIDMap=0:1:2\nUIDMap=\nSubUIDMap=keep-id\n", vec![]),
        ("Pod=\nUIDMap=0:1:2\n", vec![]),
    ];
    for (source_id, (entries, expected)) in (990_u32..).zip(cases) {
        let source = format!("[Container]\nImage=example.invalid/app:1\n{entries}");
        let result = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(source_id), source)
            .map_err(|error| error.to_string())?;
        assert_eq!(
            result
                .model_diagnostics()
                .iter()
                .map(|diagnostic| diagnostic.code().as_str())
                .collect::<Vec<_>>(),
            expected
        );
    }

    let quoted_source = concat!(
        "[Container]\nImage=example.invalid/app:1\nStartWithPod=\"YES\"\n",
        "ReadOnly=\"On\"\nReadOnlyTmpfs=\"true\"\n"
    );
    let quoted = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(1_070), quoted_source)
        .map_err(|error| error.to_string())?;
    assert_eq!(quoted.syntax().document().render_preserved(), quoted_source);
    assert_eq!(
        quoted
            .document()
            .entries()
            .filter(|entry| {
                matches!(
                    entry.kind(),
                    EntryKind::Container(
                        ContainerKey::StartWithPod | ContainerKey::ReadOnly | ContainerKey::ReadOnlyTmpfs
                    )
                )
            })
            .map(|entry| entry.value().primary().text())
            .collect::<Vec<_>>(),
        ["\"YES\"", "\"On\"", "\"true\""]
    );
    Ok(())
}

#[test]
fn start_with_pod_relationship_diagnostic_recognizes_effective_boolean_forms() -> Result<(), String> {
    for (source_id, spelling) in (1_010_u32..).zip(["1", "yes", "true", "on", "YES", "On"]) {
        let source = format!("[Container]\nImage=example.invalid/app:1\nPod=\nStartWithPod={spelling}\n");
        let result = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(source_id), source)
            .map_err(|error| error.to_string())?;
        assert_eq!(
            result
                .model_diagnostics()
                .iter()
                .map(|diagnostic| diagnostic.code().as_str())
                .collect::<Vec<_>>(),
            ["QLM0011"],
            "StartWithPod={spelling} must be recognized as true"
        );
    }
    for (source_id, spelling) in (1_020_u32..).zip(["0", "no", "false", "off", "NO", "Off"]) {
        let source = format!("[Container]\nImage=example.invalid/app:1\nPod=\nStartWithPod={spelling}\n");
        let result = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(source_id), source)
            .map_err(|error| error.to_string())?;
        assert!(
            result.model_diagnostics().is_empty(),
            "StartWithPod={spelling} must be recognized as false"
        );
    }
    Ok(())
}

#[test]
fn read_only_tmpfs_relationship_diagnostic_recognizes_effective_boolean_forms() -> Result<(), String> {
    for (source_id, spelling) in (1_030_u32..).zip(["1", "yes", "true", "on", "YES", "On"]) {
        let source = format!("[Container]\nImage=example.invalid/app:1\nReadOnly={spelling}\nReadOnlyTmpfs=On\n");
        let result = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(source_id), source)
            .map_err(|error| error.to_string())?;
        assert!(
            result.model_diagnostics().is_empty(),
            "ReadOnly={spelling} must satisfy ReadOnlyTmpfs=On"
        );
    }
    for (source_id, spelling) in (1_040_u32..).zip(["0", "no", "false", "off", "NO", "Off"]) {
        let source = format!("[Container]\nImage=example.invalid/app:1\nReadOnly={spelling}\nReadOnlyTmpfs=YES\n");
        let result = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(source_id), source)
            .map_err(|error| error.to_string())?;
        assert_eq!(
            result
                .model_diagnostics()
                .iter()
                .map(|diagnostic| diagnostic.code().as_str())
                .collect::<Vec<_>>(),
            ["QLM0012"],
            "ReadOnly={spelling} must not satisfy ReadOnlyTmpfs=YES"
        );
    }
    for (source_id, spelling) in (1_060_u32..).zip(["1", "yes", "true", "on", "YES", "On"]) {
        let source = format!("[Container]\nImage=example.invalid/app:1\nReadOnlyTmpfs={spelling}\n");
        let result = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(source_id), source)
            .map_err(|error| error.to_string())?;
        assert_eq!(
            result
                .model_diagnostics()
                .iter()
                .map(|diagnostic| diagnostic.code().as_str())
                .collect::<Vec<_>>(),
            ["QLM0012"],
            "ReadOnlyTmpfs={spelling} must be recognized as true"
        );
    }
    for (source_id, spelling) in (1_050_u32..).zip(["0", "no", "false", "off", "NO", "Off"]) {
        let source = format!("[Container]\nImage=example.invalid/app:1\nReadOnlyTmpfs={spelling}\n");
        let result = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(source_id), source)
            .map_err(|error| error.to_string())?;
        assert!(
            result.model_diagnostics().is_empty(),
            "ReadOnlyTmpfs={spelling} must be recognized as false"
        );
    }
    Ok(())
}

#[test]
fn remaining_container_keys_preserve_opaque_values_duplicates_resets_and_order() -> Result<(), String> {
    let source = concat!(
        "[Container]\nImage=example.invalid/app:1\n",
        "ContainersConfModule=pre.conf\nContainersConfModule=\n",
        "ContainersConfModule=post-one.conf\nContainersConfModule=post-two.conf\n",
        "GlobalArgs=--log-level=info\nGlobalArgs=\nGlobalArgs=--log-level=debug\n",
        "ImageVolume=/assets\nImageVolume=src.image:/opt/assets\n",
        "HealthLogDestination=local\nHealthMaxLogCount=5\nHealthMaxLogSize=10m\n",
        "HealthStartupCmd=CMD-SHELL echo ready\nHealthStartupInterval=2s\n",
        "HealthStartupRetries=4\nHealthStartupSuccess=2\nHealthStartupTimeout=1s\n",
        "ServiceName=chosen.service\n",
    );
    let result = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(1_080), source)
        .map_err(|error| error.to_string())?;
    assert_eq!(result.syntax().document().render_preserved(), source);
    for key in [
        ContainerKey::ContainersConfModule,
        ContainerKey::GlobalArgs,
        ContainerKey::ImageVolume,
        ContainerKey::HealthLogDestination,
        ContainerKey::HealthMaxLogCount,
        ContainerKey::HealthMaxLogSize,
        ContainerKey::HealthStartupCmd,
        ContainerKey::HealthStartupInterval,
        ContainerKey::HealthStartupRetries,
        ContainerKey::HealthStartupSuccess,
        ContainerKey::HealthStartupTimeout,
        ContainerKey::ServiceName,
    ] {
        assert!(
            result
                .document()
                .entries()
                .any(|entry| entry.kind() == EntryKind::Container(key))
        );
    }
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::ContainersConfModule))
            .map(|entry| entry.value().primary().text())
            .collect::<Vec<_>>(),
        ["pre.conf", "", "post-one.conf", "post-two.conf"]
    );
    assert!(result.model_diagnostics().is_empty());
    Ok(())
}

#[test]
fn remaining_pod_keys_preserve_opaque_values_cardinality_and_mapping_diagnostics() -> Result<(), String> {
    let source = concat!(
        "[Pod]\nPodName=example\n",
        "ContainersConfModule=pre.conf\nContainersConfModule=\nContainersConfModule=post.conf\n",
        "DNS=9.9.9.9\nDNSOption=ndots:1\nDNSSearch=example.invalid\n",
        "GIDMap=0:200000:1\nGlobalArgs=--log-level=debug\nHostName=pod-host\n",
        "IP=10.88.0.2\nIP6=fd00::2\nLabel=org.example.pod=yes\n",
        "NetworkAlias=api\nPodmanArgs=--replace\nUIDMap=0:100000:1\n",
    );
    let result = QuadletDocument::parse(QuadletUnitType::Pod, SourceId::new(1_090), source)
        .map_err(|error| error.to_string())?;
    assert_eq!(result.syntax().document().render_preserved(), source);
    for key in [
        PodKey::ContainersConfModule,
        PodKey::DNS,
        PodKey::DNSOption,
        PodKey::DNSSearch,
        PodKey::GIDMap,
        PodKey::GlobalArgs,
        PodKey::HostName,
        PodKey::IP,
        PodKey::IP6,
        PodKey::Label,
        PodKey::NetworkAlias,
        PodKey::PodmanArgs,
        PodKey::UIDMap,
    ] {
        assert!(
            result
                .document()
                .entries()
                .any(|entry| entry.kind() == EntryKind::Pod(key))
        );
    }
    assert!(result.model_diagnostics().is_empty());
    let conflicts = QuadletDocument::parse(
        QuadletUnitType::Pod,
        SourceId::new(1_091),
        "[Pod]\nUserNS=keep-id\nUIDMap=0:100000:1\nSubUIDMap=keep-id\nGIDMap=0:200000:1\nSubGIDMap=keep-id\n",
    )
    .map_err(|error| error.to_string())?;
    assert_eq!(
        conflicts
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        ["QLM0013", "QLM0014", "QLM0015"]
    );
    for entries in [
        "UserNS=keep-id\nUserNS=\nUIDMap=0:100000:1\n",
        "UserNS=keep-id\nUIDMap=0:100000:1\nUIDMap=\n",
        "UIDMap=0:100000:1\nUIDMap=\nSubUIDMap=keep-id\n",
        "UIDMap=0:100000:1\nSubUIDMap=keep-id\nSubUIDMap=\n",
        "GIDMap=0:200000:1\nGIDMap=\nSubGIDMap=keep-id\n",
        "GIDMap=0:200000:1\nSubGIDMap=keep-id\nSubGIDMap=\n",
    ] {
        let result = QuadletDocument::parse(QuadletUnitType::Pod, SourceId::new(1_092), format!("[Pod]\n{entries}"))
            .map_err(|error| error.to_string())?;
        assert!(
            result
                .model_diagnostics()
                .iter()
                .all(|diagnostic| !matches!(diagnostic.code().as_str(), "QLM0013" | "QLM0014" | "QLM0015")),
            "blank reset must clear the effective Pod mapping conflict for {entries:?}"
        );
    }
    Ok(())
}

#[test]
fn kube_keys_preserve_source_fidelity_required_yaml_and_network_references() -> Result<(), String> {
    let source = concat!(
        "[Kube]\nYaml=./first.yaml\nYaml=/opt/quadlet-lens/second.yaml\n",
        "AutoUpdate=registry\nConfigMap=/run/quadlet-lens/config-one.yaml\n",
        "ConfigMap=/run/quadlet-lens/config-two.yaml\nContainersConfModule=pre.conf\n",
        "ContainersConfModule=\nContainersConfModule=post.conf\nExitCodePropagation=any\n",
        "GlobalArgs=--log-level=debug\nGlobalArgs=\nGlobalArgs=--events-backend=file\n",
        "KubeDownForce=true\nLogDriver=k8s-file\nNetwork=frontend.network\n",
        "PodmanArgs=--replace\nPodmanArgs=\nPodmanArgs=--userns=keep-id\n",
        "PublishPort=8080:80\nPublishPort=8443:443\nServiceName=quadlet-lens-kube.service\n",
        "SetWorkingDirectory=unit\nUserNS=keep-id\nLogOpt=path=/run/quadlet-lens/kube.log\n",
    );
    let result = QuadletDocument::parse(QuadletUnitType::Kube, SourceId::new(1_100), source)
        .map_err(|error| error.to_string())?;
    assert!(result.is_valid(), "{:#?}", result.model_diagnostics());
    assert_eq!(result.syntax().document().render_preserved(), source);
    for key in [
        KubeKey::AutoUpdate,
        KubeKey::ConfigMap,
        KubeKey::ContainersConfModule,
        KubeKey::ExitCodePropagation,
        KubeKey::GlobalArgs,
        KubeKey::KubeDownForce,
        KubeKey::LogDriver,
        KubeKey::Network,
        KubeKey::PodmanArgs,
        KubeKey::PublishPort,
        KubeKey::ServiceName,
        KubeKey::SetWorkingDirectory,
        KubeKey::UserNS,
        KubeKey::Yaml,
        KubeKey::LogOpt,
    ] {
        assert!(
            result
                .document()
                .entries()
                .any(|entry| entry.kind() == EntryKind::Kube(key))
        );
    }
    assert_eq!(
        result
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Kube(KubeKey::Yaml))
            .map(|entry| entry.value().primary().text())
            .collect::<Vec<_>>(),
        ["./first.yaml", "/opt/quadlet-lens/second.yaml"]
    );
    assert!(result.document().entries().any(|entry| {
        entry.kind() == EntryKind::Kube(KubeKey::Network)
            && entry.value_kind() == ValueKind::UnitReference(UnitReferenceKind::Network)
    }));
    assert!(result.document().entries().any(|entry| {
        entry.kind() == EntryKind::Kube(KubeKey::Yaml)
            && entry.value_kind() == ValueKind::Path(PathForm::UnitRelativeLiteral)
    }));

    let missing = QuadletDocument::parse(QuadletUnitType::Kube, SourceId::new(1_101), "[Kube]\n")
        .map_err(|error| error.to_string())?;
    assert_eq!(
        missing
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        ["QLM0017"]
    );
    let empty = QuadletDocument::parse(QuadletUnitType::Kube, SourceId::new(1_102), "[Kube]\nYaml=\n")
        .map_err(|error| error.to_string())?;
    assert_eq!(
        empty
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        ["QLM0018"]
    );
    let duplicate = QuadletDocument::parse(
        QuadletUnitType::Kube,
        SourceId::new(1_103),
        "[Kube]\nYaml=placeholder.yaml\nRemapUsers=true\nRemapUsers=false\n",
    )
    .map_err(|error| error.to_string())?;
    assert_eq!(
        duplicate
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        ["QLM0004"]
    );
    assert_kube_yaml_reset_cases()?;
    assert_kube_yaml_working_directory_cases()?;
    assert_kube_remap_cases()?;
    Ok(())
}

fn assert_kube_yaml_reset_cases() -> Result<(), String> {
    let reset_followed_by_source = QuadletDocument::parse(
        QuadletUnitType::Kube,
        SourceId::new(11_021),
        "[Kube]\nYaml=discarded.yaml\nYaml=\nYaml=effective.yaml\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(
        reset_followed_by_source.is_valid(),
        "{:#?}",
        reset_followed_by_source.model_diagnostics()
    );
    for (source_id, source) in [
        (11_022, "[Kube]\nYaml=discarded.yaml\nYaml=\n"),
        (11_023, "[Kube]\nYaml=\nYaml=\n"),
    ] {
        let parsed = QuadletDocument::parse(QuadletUnitType::Kube, SourceId::new(source_id), source)
            .map_err(|error| error.to_string())?;
        assert_eq!(
            parsed
                .model_diagnostics()
                .iter()
                .map(|diagnostic| diagnostic.code().as_str())
                .collect::<Vec<_>>(),
            ["QLM0018"]
        );
    }
    Ok(())
}

fn assert_kube_yaml_working_directory_cases() -> Result<(), String> {
    for (source_id, entries, expected) in [
        (
            1_104,
            "Yaml=one.yaml\nYaml=two.yaml\nSetWorkingDirectory=yaml\n",
            vec!["QLM0019"],
        ),
        (1_105, "Yaml=one.yaml\nSetWorkingDirectory=yaml\n", vec![]),
        (
            1_106,
            "Yaml=one.yaml\nYaml=two.yaml\nSetWorkingDirectory=unit\n",
            vec![],
        ),
        (
            1_107,
            "Yaml=one.yaml\nYaml=two.yaml\nYaml=\nSetWorkingDirectory=yaml\n",
            vec!["QLM0018"],
        ),
        (1_108, "Yaml=\"one source.yaml\"\nSetWorkingDirectory=yaml\n", vec![]),
        (1_109, "Yaml=one\\ source.yaml\nSetWorkingDirectory=yaml\n", vec![]),
        (
            1_110,
            "Yaml=one.yaml two.yaml\nSetWorkingDirectory=yaml\n",
            vec!["QLM0019"],
        ),
    ] {
        let parsed = QuadletDocument::parse(
            QuadletUnitType::Kube,
            SourceId::new(source_id),
            format!("[Kube]\n{entries}"),
        )
        .map_err(|error| error.to_string())?;
        assert_eq!(
            parsed
                .model_diagnostics()
                .iter()
                .map(|diagnostic| diagnostic.code().as_str())
                .collect::<Vec<_>>(),
            expected,
            "{entries}"
        );
        if expected == ["QLM0019"] {
            let diagnostic = parsed
                .model_diagnostics()
                .iter()
                .find(|diagnostic| diagnostic.code().as_str() == "QLM0019")
                .ok_or_else(|| "missing QLM0019".to_owned())?;
            assert_eq!(diagnostic.labels()[0].span().source_id(), SourceId::new(source_id));
            assert_eq!(
                diagnostic.labels()[0].span().start(),
                "[Kube]\n".len()
                    + entries.find("SetWorkingDirectory=").unwrap_or_default()
                    + "SetWorkingDirectory=".len(),
            );
        }
    }
    Ok(())
}

fn assert_kube_remap_cases() -> Result<(), String> {
    let remaps = QuadletDocument::parse(
        QuadletUnitType::Kube,
        SourceId::new(1_111),
        "[Kube]\nYaml=placeholder.yaml\nRemapGid=200000\nRemapGid=200001\nRemapUid=100000\nRemapUid=100001\nRemapUidSize=65536\nRemapUsers=auto\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(remaps.is_valid(), "{:#?}", remaps.model_diagnostics());
    let conflict = QuadletDocument::parse(
        QuadletUnitType::Kube,
        SourceId::new(1_112),
        "[Kube]\nYaml=placeholder.yaml\nUserNS=keep-id\nRemapUid=100000\n",
    )
    .map_err(|error| error.to_string())?;
    assert_eq!(
        conflict
            .model_diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        ["QLM0020"]
    );
    assert_eq!(conflict.model_diagnostics()[0].labels().len(), 2);
    let size_only = QuadletDocument::parse(
        QuadletUnitType::Kube,
        SourceId::new(1_113),
        "[Kube]\nYaml=placeholder.yaml\nUserNS=keep-id\nRemapUidSize=65536\n",
    )
    .map_err(|error| error.to_string())?;
    assert!(
        size_only
            .model_diagnostics()
            .iter()
            .all(|diagnostic| diagnostic.code().as_str() != "QLM0020")
    );
    for (entries, conflicts) in [
        ("UserNS=keep-id\nRemapUid=100000\nRemapUid=\n", false),
        ("UserNS=keep-id\nRemapUid=100000\nRemapUid=\nRemapGid=200000\n", true),
        ("UserNS=keep-id\nRemapUsers=auto\nRemapUid=100000\nRemapUid=\n", true),
        ("UserNS=keep-id\nUserNS=\nRemapUid=100000\n", false),
    ] {
        let parsed = QuadletDocument::parse(
            QuadletUnitType::Kube,
            SourceId::new(1_113),
            format!("[Kube]\nYaml=placeholder.yaml\n{entries}"),
        )
        .map_err(|error| error.to_string())?;
        assert_eq!(
            parsed
                .model_diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.code().as_str() == "QLM0020"),
            conflicts,
            "{entries}"
        );
    }
    Ok(())
}

#[test]
fn artifact_and_quadlet_keys_preserve_effective_source_boundaries_and_redact_seeded_values() -> Result<(), String> {
    let source = "[Quadlet]\nDefaultDependencies=not-a-boolean\n[Artifact]\nArtifact=pre.invalid/a\nArtifact=\nArtifact=registry.invalid/final\nCreds=SEEDED_ARTIFACT_CREDS\nDecryptionKey=SEEDED_ARTIFACT_KEY\nContainersConfModule=pre.conf\nContainersConfModule=\nContainersConfModule=post.conf\nGlobalArgs=--pre\nGlobalArgs=\nGlobalArgs=--post\nPodmanArgs=--pre\nPodmanArgs=\nPodmanArgs=--post\n";
    let parsed = QuadletDocument::parse(QuadletUnitType::Artifact, SourceId::new(9_001), source)
        .map_err(|error| error.to_string())?;
    assert!(parsed.is_valid(), "{:#?}", parsed.model_diagnostics());
    assert!(
        parsed
            .document()
            .entries()
            .any(|entry| entry.kind() == EntryKind::Quadlet(QuadletKey::DefaultDependencies))
    );
    assert!(
        parsed
            .document()
            .entries()
            .any(|entry| entry.kind() == EntryKind::Artifact(ArtifactKey::Artifact))
    );
    let debug = format!("{:?}", parsed.document());
    assert!(!debug.contains("SEEDED_ARTIFACT_CREDS") && !debug.contains("SEEDED_ARTIFACT_KEY"));
    for (source, code) in [
        ("[Artifact]\n", "QLM0021"),
        ("[Artifact]\nArtifact=ok\nArtifact=\n", "QLM0022"),
    ] {
        let parsed = QuadletDocument::parse(QuadletUnitType::Artifact, SourceId::new(9_002), source)
            .map_err(|error| error.to_string())?;
        assert!(
            parsed
                .model_diagnostics()
                .iter()
                .any(|item| item.code().as_str() == code)
        );
    }
    Ok(())
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "The table-driven parser case keeps the complete Artifact classification and provenance matrix auditable together."
)]
fn artifact_parser_classifies_every_key_preserves_cardinality_and_keeps_final_source_diagnostics() -> Result<(), String>
{
    const CREDS: &str = "quadlet-lens-artifact-parser-creds-canary";
    const KEY: &str = "quadlet-lens-artifact-parser-key-canary";
    let source = concat!(
        "[Quadlet]\nDefaultDependencies=true\nDefaultDependencies=false\n",
        "[Artifact]\nArtifact=registry.invalid/pre:1\nArtifact=registry.invalid/final:1\n",
        "AuthFile=/run/quadlet-lens/auth.json\nCertDir=/run/quadlet-lens/certs\n",
        "Creds=quadlet-lens-artifact-parser-creds-canary\n",
        "DecryptionKey=quadlet-lens-artifact-parser-key-canary\n",
        "Quiet=true\nRetry=4\nRetryDelay=7s\nServiceName=artifact-pull\nTLSVerify=false\n",
        "ContainersConfModule=pre.conf\nContainersConfModule=\nContainersConfModule=post.conf\n",
        "GlobalArgs=--pre\nGlobalArgs=\nGlobalArgs=--post\n",
        "PodmanArgs=--pre\nPodmanArgs=\nPodmanArgs=--post\n",
        "artifact=case-sensitive-unknown\n",
    );
    let parsed = QuadletDocument::parse(QuadletUnitType::Artifact, SourceId::new(9_010), source)
        .map_err(|error| error.to_string())?;
    assert_eq!(parsed.syntax().document().render_preserved(), source);
    assert_eq!(
        parsed
            .document()
            .entries()
            .filter(|entry| entry.kind() != EntryKind::Unknown)
            .map(|entry| (
                entry.kind(),
                entry.value().primary().text(),
                entry.value_kind(),
                entry.is_sensitive()
            ))
            .collect::<Vec<_>>(),
        [
            (
                EntryKind::Quadlet(QuadletKey::DefaultDependencies),
                "true",
                ValueKind::Opaque,
                false
            ),
            (
                EntryKind::Quadlet(QuadletKey::DefaultDependencies),
                "false",
                ValueKind::Opaque,
                false
            ),
            (
                EntryKind::Artifact(ArtifactKey::Artifact),
                "registry.invalid/pre:1",
                ValueKind::Opaque,
                false
            ),
            (
                EntryKind::Artifact(ArtifactKey::Artifact),
                "registry.invalid/final:1",
                ValueKind::Opaque,
                false
            ),
            (
                EntryKind::Artifact(ArtifactKey::AuthFile),
                "/run/quadlet-lens/auth.json",
                ValueKind::Opaque,
                false
            ),
            (
                EntryKind::Artifact(ArtifactKey::CertDir),
                "/run/quadlet-lens/certs",
                ValueKind::Opaque,
                false
            ),
            (EntryKind::Artifact(ArtifactKey::Creds), CREDS, ValueKind::Opaque, true),
            (
                EntryKind::Artifact(ArtifactKey::DecryptionKey),
                KEY,
                ValueKind::Opaque,
                true
            ),
            (
                EntryKind::Artifact(ArtifactKey::Quiet),
                "true",
                ValueKind::Opaque,
                false
            ),
            (EntryKind::Artifact(ArtifactKey::Retry), "4", ValueKind::Opaque, false),
            (
                EntryKind::Artifact(ArtifactKey::RetryDelay),
                "7s",
                ValueKind::Opaque,
                false
            ),
            (
                EntryKind::Artifact(ArtifactKey::ServiceName),
                "artifact-pull",
                ValueKind::Opaque,
                false
            ),
            (
                EntryKind::Artifact(ArtifactKey::TLSVerify),
                "false",
                ValueKind::Opaque,
                false
            ),
            (
                EntryKind::Artifact(ArtifactKey::ContainersConfModule),
                "pre.conf",
                ValueKind::Opaque,
                false
            ),
            (
                EntryKind::Artifact(ArtifactKey::ContainersConfModule),
                "",
                ValueKind::Opaque,
                false
            ),
            (
                EntryKind::Artifact(ArtifactKey::ContainersConfModule),
                "post.conf",
                ValueKind::Opaque,
                false
            ),
            (
                EntryKind::Artifact(ArtifactKey::GlobalArgs),
                "--pre",
                ValueKind::Opaque,
                false
            ),
            (
                EntryKind::Artifact(ArtifactKey::GlobalArgs),
                "",
                ValueKind::Opaque,
                false
            ),
            (
                EntryKind::Artifact(ArtifactKey::GlobalArgs),
                "--post",
                ValueKind::Opaque,
                false
            ),
            (
                EntryKind::Artifact(ArtifactKey::PodmanArgs),
                "--pre",
                ValueKind::Opaque,
                false
            ),
            (
                EntryKind::Artifact(ArtifactKey::PodmanArgs),
                "",
                ValueKind::Opaque,
                false
            ),
            (
                EntryKind::Artifact(ArtifactKey::PodmanArgs),
                "--post",
                ValueKind::Opaque,
                false
            ),
        ]
    );
    let diagnostics: Vec<_> = parsed
        .model_diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.code().as_str())
        .collect();
    assert_eq!(diagnostics, ["QLM0004", "QLM0004"]);
    let debug = format!("{:#?}", parsed.document());
    assert!(!debug.contains(CREDS) && !debug.contains(KEY));

    for (value, expected) in [("true", true), ("false", true), ("not-a-boolean", true), ("", true)] {
        let parsed = QuadletDocument::parse(
            QuadletUnitType::Artifact,
            SourceId::new(9_011),
            format!("[Quadlet]\nDefaultDependencies={value}\n[Artifact]\nArtifact=registry.invalid/final:1\n"),
        )
        .map_err(|error| error.to_string())?;
        assert_eq!(parsed.is_valid(), expected, "{value:?}");
    }
    Ok(())
}
