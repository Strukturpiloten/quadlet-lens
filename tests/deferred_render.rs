//! Typed native deferred words and explicitly provisioned service environment.
use quadlet_lens::{
    model::{ContainerKey, QuadletDocument, QuadletUnitType},
    render::{
        ContainerEnvironmentName, DeferredValueError, DeferredWord, DeferredWordPart, EntryValue,
        QuadletDocumentBuilder, RenderError, ServiceEnvironmentAssignment, ServiceEnvironmentFile,
        ServiceEnvironmentName,
    },
    source::SourceId,
    syntax::SyntaxDocument,
};
const CANARY: &str = "QUADLET_LENS_DEFERRED_CANARY";
fn name(value: &str) -> Result<ServiceEnvironmentName, DeferredValueError> {
    ServiceEnvironmentName::new(value)
}
fn literal(value: &str) -> Result<DeferredWord, DeferredValueError> {
    DeferredWord::new([DeferredWordPart::Literal(value.to_owned())])
}
fn builder() -> Result<QuadletDocumentBuilder, RenderError> {
    let mut builder = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    builder.push_container(ContainerKey::Image, EntryValue::new("example.invalid/app:1")?)?;
    Ok(builder)
}
fn protected(value: &impl std::fmt::Debug) {
    assert!(!format!("{value:#?}").contains(CANARY));
}

#[test]
fn typed_words_render_context_specific_dollars_percents_and_one_argument() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = builder()?;
    let parts = vec![
        DeferredWordPart::Literal("before $LITERAL ${LITERAL} %h \"quoted\" \\".to_owned()),
        DeferredWordPart::Reference(name("PROVIDER")?),
        DeferredWordPart::Literal(" after".to_owned()),
    ];
    let word = DeferredWord::new(parts.clone())?;
    assert_eq!(word.parts(), parts);
    builder.push_container_environment_deferred(name("COMBINED")?, word)?;
    builder.push_container_environment_deferred(name("EMPTY")?, DeferredWord::new([])?)?;
    builder.push_container_bind_source_deferred(
        DeferredWord::new([
            DeferredWordPart::Reference(name("ROOT")?),
            DeferredWordPart::Literal("/path with $dollar %percent.image".to_owned()),
        ])?,
        "/target $dollar %percent",
        vec!["ro".to_owned(), "Z".to_owned()],
    )?;
    builder.push_service_environment_literal(ServiceEnvironmentAssignment::new(
        name("PROVIDER")?,
        r#"$LITERAL ${LITERAL} %h "quoted" \"#,
    )?)?;
    let generated = builder.build(SourceId::new(163))?;
    assert!(generated.text().contains("Environment=\"EMPTY=\""));
    assert!(generated.text().contains("before $$LITERAL $${LITERAL} %%h"));
    assert!(generated.text().contains("${PROVIDER} after"));
    assert!(
        generated
            .text()
            .contains("Volume=${ROOT}/path with $$dollar %%percent.image:/target $$dollar %%percent:ro,Z")
    );
    assert!(
        generated
            .text()
            .contains("Environment=\"PROVIDER=$LITERAL ${LITERAL} %%h")
    );
    assert!(QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(164), generated.text())?.is_valid());
    Ok(())
}

#[test]
fn provider_files_are_absolute_literal_ordered_and_mandatory_by_default() -> Result<(), Box<dyn std::error::Error>> {
    let first = ServiceEnvironmentFile::new("/run/provider first$literal%value.env")?;
    assert!(!first.is_optional());
    assert_eq!(first.path(), "/run/provider first$literal%value.env");
    let second = ServiceEnvironmentFile::new("/run/provider-second.env")?.with_optional(true);
    assert!(second.is_optional());
    assert_eq!(second.as_str(), "\"-/run/provider-second.env\"");
    let assignment = ServiceEnvironmentAssignment::new(name("EMPTY")?, "")?;
    assert_eq!(assignment.value(), "");
    assert_eq!(assignment.name().as_str(), "EMPTY");
    let mut builder = builder()?;
    builder.push_service_environment_file(first)?;
    builder.push_service_environment_literal(assignment)?;
    builder.push_service_environment_file(second)?;
    let generated = builder.build(SourceId::new(165))?;
    let files: Vec<_> = generated
        .document()
        .entries()
        .filter(|entry| entry.key().text() == "EnvironmentFile")
        .map(|entry| entry.value().primary().text())
        .collect();
    assert_eq!(
        files,
        [
            "\"/run/provider first$literal%%value.env\"",
            "\"-/run/provider-second.env\""
        ]
    );
    assert!(generated.text().find("EnvironmentFile=") < generated.text().find("Environment=\"EMPTY=\""));
    Ok(())
}

#[test]
fn invalid_inputs_are_value_free_and_cannot_inject_native_fields() -> Result<(), Box<dyn std::error::Error>> {
    for value in ["", "1NAME", "BAD-NAME", "é", "A\nInjected", "${NAME}"] {
        assert_eq!(name(value).err(), Some(DeferredValueError::InvalidName));
    }
    for value in [
        format!("{CANARY}\n"),
        format!("{CANARY}\r"),
        format!("{CANARY}\0"),
        format!("{CANARY}\t"),
    ] {
        let error = literal(&value).err().ok_or("control accepted")?;
        protected(&error);
        assert!(!error.to_string().contains(CANARY));
        assert!(ServiceEnvironmentAssignment::new(name("SAFE")?, &value).is_err());
    }
    for path in [
        "",
        "relative.env",
        "-/run/file",
        "/run/*.env",
        "/run/?.env",
        "/run/[ab].env",
        "/run/a\nfile",
    ] {
        assert!(ServiceEnvironmentFile::new(path).is_err());
    }
    for source in ["", "relative", "/host:bad", "/cache.volume", "/cache.artifact"] {
        assert!(
            builder()?
                .push_container_bind_source_deferred(literal(source)?, "/target", Vec::new())
                .is_err()
        );
    }
    for suffix in ["/cache.volume", "/cache.artifact"] {
        let source = DeferredWord::new([
            DeferredWordPart::Reference(name("ROOT")?),
            DeferredWordPart::Literal(suffix.to_owned()),
        ])?;
        assert!(
            builder()?
                .push_container_bind_source_deferred(source, "/target", Vec::new())
                .is_err()
        );
    }
    for target in ["", "relative", "/target:bad", "/target\nInjected"] {
        assert!(
            builder()?
                .push_container_bind_source_deferred(literal("/source")?, target, Vec::new())
                .is_err()
        );
    }
    for option in ["", "ro,Z", "ro:bad", "ro bad", "ro\nInjected"] {
        assert!(
            builder()?
                .push_container_bind_source_deferred(literal("/source")?, "/target", vec![option.to_owned()])
                .is_err()
        );
    }
    let mut wrong = QuadletDocumentBuilder::new(QuadletUnitType::Pod);
    assert!(matches!(
        wrong.push_container_environment_deferred(name("TARGET")?, literal("safe")?),
        Err(RenderError::WrongUnitType { .. })
    ));
    Ok(())
}

#[test]
fn protected_provider_debug_covers_fresh_valid_malformed_and_consumed_source() -> Result<(), Box<dyn std::error::Error>>
{
    let assignment = ServiceEnvironmentAssignment::new(name(CANARY)?, CANARY)?;
    let file = ServiceEnvironmentFile::new(format!("/run/{CANARY}.env"))?;
    let word = DeferredWord::new([
        DeferredWordPart::Literal(CANARY.to_owned()),
        DeferredWordPart::Reference(name(CANARY)?),
    ])?;
    protected(&assignment);
    protected(&file);
    protected(&word);
    protected(&word.parts());
    protected(&name(CANARY)?);
    assert!(assignment.as_str().contains(CANARY));
    let mut builder = builder()?;
    builder.push_container_environment_deferred(name("TARGET")?, word)?;
    builder.push_service_environment_literal(assignment)?;
    builder.push_service_environment_file(file)?;
    protected(&builder);
    let generated = builder.build(SourceId::new(166))?;
    protected(&generated);
    for header in ["[Service]\n", "[Service-Broken\n", "Label=continued\\\n[Service]\n"] {
        let text = format!(
            "[Container]\nImage=example.invalid/app:1\n{header}Environment={CANARY}={CANARY}\nEnvironmentFile=/run/{CANARY}.env\n"
        );
        let parsed = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(167), &text)?;
        protected(&parsed);
        protected(parsed.document());
        protected(parsed.syntax());
        protected(parsed.syntax().document());
        protected(parsed.syntax().document().source());
        for entry in parsed.document().entries() {
            protected(entry);
            protected(entry.value());
            protected(entry.value().primary());
            protected(&entry.value().continuations());
        }
        protected(&parsed.document().container_environment());
        protected(&SyntaxDocument::parse(SourceId::new(168), &text));
        assert_eq!(parsed.syntax().document().render_preserved(), text);
    }
    Ok(())
}

#[test]
fn authored_continuations_cannot_consume_provider_or_deferred_boundaries() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = builder()?;
    builder.push_container(ContainerKey::Label, EntryValue::new("continued\\")?)?;
    builder.push_service_environment_file(ServiceEnvironmentFile::new("/run/external.env")?)?;
    assert_eq!(
        builder.build(SourceId::new(169)).err(),
        Some(RenderError::DeferredValue(DeferredValueError::UnsafeContinuation))
    );
    Ok(())
}

#[test]
fn endpoint_fixture_matches_the_supported_construction_api() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    builder.push_container(ContainerKey::Image, EntryValue::new("example.invalid/deferred:1")?)?;
    builder.push_container_environment_deferred(
        name("COMBINED")?,
        DeferredWord::new([
            DeferredWordPart::Literal("prefix $LITERAL %literal ".to_owned()),
            DeferredWordPart::Reference(name("PROVIDER")?),
            DeferredWordPart::Literal(" suffix".to_owned()),
        ])?,
    )?;
    builder.push_container_environment_deferred(name("EMPTY")?, literal("")?)?;
    builder.push_container_environment_deferred(
        ContainerEnvironmentName::new("NC_davstorage.request_timeout")?,
        literal("30")?,
    )?;
    builder.push_container_environment_deferred(
        ContainerEnvironmentName::new("literal$KEY%key")?,
        literal("literal $value %value")?,
    )?;
    builder.push_container_bind_source_deferred(
        DeferredWord::new([
            DeferredWordPart::Reference(name("ROOT")?),
            DeferredWordPart::Literal("/path with $LITERAL %literal \"quote\" \\backslash.image".to_owned()),
        ])?,
        "/target with $LITERAL %literal",
        vec!["ro".to_owned(), "Z".to_owned()],
    )?;
    builder.push_service_environment_literal(ServiceEnvironmentAssignment::new(
        name("PROVIDER")?,
        "literal $DOLLAR %percent",
    )?)?;
    builder.push_service_environment_literal(ServiceEnvironmentAssignment::new(name("EMPTY_PROVIDER")?, "")?)?;
    builder.push_service_environment_file(ServiceEnvironmentFile::new("/run/quadlet-lens-first.env")?)?;
    builder.push_service_environment_file(ServiceEnvironmentFile::new("/run/quadlet-lens-second.env")?)?;
    assert_eq!(
        builder.build(SourceId::new(170))?.text(),
        include_str!("../fixtures/generators/deferred-service-environment/deferred.container")
    );
    Ok(())
}

#[test]
fn standalone_deferred_helpers_keep_source_and_derived_debug_protected() -> Result<(), Box<dyn std::error::Error>> {
    for bind in [false, true] {
        let mut builder = builder()?;
        if bind {
            builder.push_container_bind_source_deferred(literal(&format!("/{CANARY}"))?, "/data", Vec::new())?;
        } else {
            builder.push_container_environment_deferred(name("TARGET")?, literal(CANARY)?)?;
        }
        protected(&builder);
        let generated = builder.build(SourceId::new(171))?;
        protected(&generated);
        protected(generated.document());
        protected(generated.parse_result().syntax().document().source());
        let parsed = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(172), generated.text())?;
        protected(&parsed);
        protected(parsed.document());
        protected(parsed.syntax().document().source());
        protected(&parsed.document().container_environment());
        protected(&parsed.document().container_mounts());
        for entry in parsed.document().entries() {
            protected(entry.value());
        }
        assert!(parsed.syntax().document().render_preserved().contains(CANARY));
    }
    Ok(())
}

#[test]
fn reverse_environment_projection_decodes_literal_pairs_and_defers_braced_references()
-> Result<(), Box<dyn std::error::Error>> {
    use quadlet_lens::model::AuthoredContainerEnvironmentDirective;
    for (value, expected) in [
        ("${PROVIDER}", None),
        ("prefix${PROVIDER}suffix", None),
        ("${UNTERMINATED", None),
        ("${PROVIDER:-default}", None),
        ("$$${PROVIDER}", None),
        ("%h", None),
        ("$PROVIDER", Some("$PROVIDER")),
        ("$${PROVIDER}", Some("${PROVIDER}")),
        ("$$$${PROVIDER}", Some("$${PROVIDER}")),
        ("%%literal", Some("%literal")),
        ("$$", Some("$")),
        ("$", Some("$")),
        ("", Some("")),
    ] {
        let text = format!("[Container]\nImage=example.invalid/app:1\nEnvironment=\"TARGET={value}\"\n");
        let parsed = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(173), &text)?;
        let environment = parsed.document().container_environment();
        match (environment.directives().first(), expected) {
            (Some(AuthoredContainerEnvironmentDirective::Assignment { value, .. }), Some(expected)) => {
                assert_eq!(value, expected);
            }
            (Some(AuthoredContainerEnvironmentDirective::Deferred { .. }), None) => {
                assert!(!environment.diagnostics().is_empty());
            }
            _ => return Err("incorrect reverse environment literal/deferred classification".into()),
        }
        assert_eq!(parsed.syntax().document().render_preserved(), text);
    }
    // Service providers retain their literal declaration spelling without exec decoding.
    let assignment = ServiceEnvironmentAssignment::new(name("PROVIDER")?, "$${OTHER} %%")?;
    assert_eq!(assignment.value(), "$${OTHER} %%");
    assert_eq!(assignment.as_str(), "\"PROVIDER=$${OTHER} %%%%\"");
    Ok(())
}

#[test]
fn reverse_mount_projection_distinguishes_whole_and_embedded_dollars() -> Result<(), Box<dyn std::error::Error>> {
    use quadlet_lens::model::NativeMountDirective;
    for (key, value, expected) in [
        ("Volume", "${ROOT}/data:/target", None),
        ("Volume", "$ROOT:/target", None),
        ("Volume", "$?INVALID:/target", None),
        ("Volume", "$$${ROOT}:/target", None),
        ("Volume", "$${ROOT}/data:/target", Some("${ROOT}/data")),
        ("Volume", "/prefix/$ROOT:/target", Some("/prefix/$ROOT")),
        ("Volume", "/prefix/%%literal:/target", Some("/prefix/%literal")),
        ("Mount", "type=bind,source=${ROOT},target=/target", None),
        ("Mount", "type=bind,source=$ROOT,target=/target", Some("$ROOT")),
        ("Mount", "type=bind,source=$${ROOT},target=/target", Some("${ROOT}")),
    ] {
        let directive = if key == "Volume" {
            value.to_owned()
        } else {
            format!("\"{value}\"")
        };
        let text = format!("[Container]\nImage=example.invalid/app:1\n{key}={directive}\n");
        let parsed = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(174), &text)?;
        let mounts = parsed.document().container_mounts();
        match (mounts.directives().first(), expected) {
            (Some(NativeMountDirective::Mount(mount)), Some(source)) => assert_eq!(mount.source(), Some(source)),
            (Some(NativeMountDirective::Deferred { .. }), None) => assert!(!mounts.diagnostics().is_empty()),
            _ => return Err("incorrect reverse mount literal/deferred classification".into()),
        }
        assert_eq!(parsed.syntax().document().render_preserved(), text);
    }
    let pod = QuadletDocument::parse(
        QuadletUnitType::Pod,
        SourceId::new(175),
        "[Pod]\nVolume=$ROOT:/target\n",
    )?;
    assert!(matches!(
        pod.document().pod_mounts().directives(),
        [NativeMountDirective::Deferred { .. }]
    ));
    Ok(())
}

#[test]
fn bind_sources_reject_unevidenced_reference_positions() -> Result<(), Box<dyn std::error::Error>> {
    for parts in [
        vec![
            DeferredWordPart::Literal(format!("/{CANARY}/")),
            DeferredWordPart::Reference(name("ROOT")?),
        ],
        vec![
            DeferredWordPart::Reference(name("ROOT")?),
            DeferredWordPart::Literal("/middle/".to_owned()),
            DeferredWordPart::Reference(name("SECOND")?),
        ],
        vec![
            DeferredWordPart::Literal(String::new()),
            DeferredWordPart::Reference(name("ROOT")?),
        ],
        vec![
            DeferredWordPart::Reference(name("ROOT")?),
            DeferredWordPart::Reference(name("ROOT")?),
        ],
    ] {
        let error = builder()?
            .push_container_bind_source_deferred(DeferredWord::new(parts)?, "/target", Vec::new())
            .err()
            .ok_or("unsupported bind shape accepted")?;
        assert_eq!(error, RenderError::DeferredValue(DeferredValueError::InvalidBindSource));
        protected(&error);
    }
    for parts in [
        vec![
            DeferredWordPart::Literal("/absolute".to_owned()),
            DeferredWordPart::Literal("/literal".to_owned()),
        ],
        vec![
            DeferredWordPart::Reference(name("ROOT")?),
            DeferredWordPart::Literal("/safe/suffix".to_owned()),
        ],
    ] {
        let mut builder = builder()?;
        builder.push_container_bind_source_deferred(DeferredWord::new(parts)?, "/target", Vec::new())?;
        assert!(builder.build(SourceId::new(176))?.text().contains("Volume="));
    }
    Ok(())
}

#[test]
fn raw_volume_preserves_field_bytes_and_rejects_unsafe_derived_dependencies() -> Result<(), Box<dyn std::error::Error>>
{
    use quadlet_lens::model::NativeMountDirective;
    for source in [
        "/path/$literal",
        "/path/with space",
        "/path/with\"quote",
        "/path/with'quote",
        "/path/with\\backslash",
    ] {
        assert_eq!(
            builder()?
                .push_container_bind_source_deferred(literal(source)?, "/target", Vec::new())
                .err(),
            Some(RenderError::DeferredValue(
                DeferredValueError::UnsupportedLiteralBindSource
            ))
        );
    }
    for target in ["/target ", "/target\\"] {
        assert_eq!(
            builder()?
                .push_container_bind_source_deferred(literal("/source")?, target, Vec::new())
                .err(),
            Some(RenderError::DeferredValue(DeferredValueError::InvalidBindTarget))
        );
    }
    let mut builder = builder()?;
    builder.push_container_bind_source_deferred(
        DeferredWord::new([
            DeferredWordPart::Reference(name("ROOT")?),
            DeferredWordPart::Literal("/space $literal %literal \"quote\" \\backslash".to_owned()),
        ])?,
        "/target with space",
        Vec::new(),
    )?;
    let generated = builder.build(SourceId::new(177))?;
    assert!(
        generated
            .text()
            .contains("Volume=${ROOT}/space $$literal %%literal \"quote\" \\backslash:/target with space\n")
    );
    for (value, source, target) in [
        (
            "/source with spaces:/target with spaces",
            "/source with spaces",
            "/target with spaces",
        ),
        ("\"/literal-quote:/target\"", "\"/literal-quote", "/target\""),
        ("/literal\\x20bytes:/target", "/literal\\x20bytes", "/target"),
    ] {
        let text = format!("[Container]\nImage=example.invalid/app:1\nVolume={value}\n");
        let parsed = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(178), &text)?;
        let mounts = parsed.document().container_mounts();
        let Some(NativeMountDirective::Mount(mount)) = mounts.directives().first() else {
            return Err("raw Volume was not decoded".into());
        };
        assert_eq!(mount.source(), Some(source));
        assert_eq!(mount.destination(), Some(target));
        assert_eq!(parsed.syntax().document().render_preserved(), text);
    }
    Ok(())
}

#[test]
fn literal_container_keys_are_distinct_from_strict_service_names() -> Result<(), Box<dyn std::error::Error>> {
    for key in [
        "NC_davstorage.request_timeout",
        "literal$KEY%key",
        "literal${KEY}",
        "dash-key",
        "café",
        "TRAILING ",
    ] {
        let container = ContainerEnvironmentName::new(key)?;
        assert_eq!(container.as_str(), key);
        let mut builder = builder()?;
        builder.push_container_environment_deferred(container, literal("value")?)?;
        let generated = builder.build(SourceId::new(179))?;
        assert!(
            generated
                .text()
                .contains(&format!("{}=value", key.replace('$', "$$").replace('%', "%%")))
        );
    }
    for key in ["", " leading", "\tleading", "A=B", "A\0B", "A\nB", "A\rB"] {
        assert_eq!(
            ContainerEnvironmentName::new(key).err(),
            Some(DeferredValueError::InvalidContainerName)
        );
    }
    assert!(name("NC_davstorage.request_timeout").is_err());
    protected(&ContainerEnvironmentName::new(CANARY)?);
    let mut builder = builder()?;
    builder.push_container_environment_deferred(ContainerEnvironmentName::new(CANARY)?, literal(CANARY)?)?;
    protected(&builder);
    protected(&builder.build(SourceId::new(180))?);
    let raw = "[Container]\nImage=example.invalid/app:1\nEnvironment=NC_davstorage.request_timeout=30\n";
    let parsed = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(181), raw)?;
    assert!(matches!(
        parsed.document().container_environment().directives(),
        [quadlet_lens::model::AuthoredContainerEnvironmentDirective::Unmodeled { .. }]
    ));
    assert!(!parsed.document().container_environment().diagnostics().is_empty());
    Ok(())
}
