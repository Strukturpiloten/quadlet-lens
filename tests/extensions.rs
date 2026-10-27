//! Protected inert X-section generation and fresh-source privacy boundaries.
use quadlet_lens::{
    model::{ContainerKey, QuadletDocument, QuadletUnitType},
    render::{EntryValue, ExtensionValue, QuadletDocumentBuilder, RenderError, XSectionName},
    source::SourceId,
    syntax::SyntaxDocument,
};

const CANARY: &str = "QUADLET_LENS_EXTENSION_PAYLOAD_CANARY";

#[test]
fn extension_entries_group_sections_and_preserve_repetition_and_exact_values() -> Result<(), Box<dyn std::error::Error>>
{
    let mut builder = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    builder.push_container(ContainerKey::Image, EntryValue::new("example.invalid/app:1")?)?;
    let section = XSectionName::new("X-Vendor.test_1")?;
    assert_eq!(section.as_str(), "X-Vendor.test_1");
    let value = ExtensionValue::new(format!("{{\"payload\":\"{CANARY}\"}}"))?;
    assert!(value.as_str().contains(CANARY));
    assert!(!format!("{value:?}").contains(CANARY));
    builder.push_extension_entry(section.clone(), "Payload", value.clone())?;
    builder.push_extension_entry(XSectionName::new("X-A")?, "Other", ExtensionValue::new("first")?)?;
    builder.push_extension_entry(section, "Payload", value)?;
    builder.push_extension_entry(XSectionName::new("X-A")?, "Other", ExtensionValue::new("")?)?;
    assert!(!format!("{builder:?}").contains(CANARY));
    let generated = builder.build(SourceId::new(162))?;
    assert_eq!(generated.text().matches("[X-Vendor.test_1]").count(), 1);
    assert!(generated.text().find("[X-A]") < generated.text().find("[X-Vendor.test_1]"));
    assert!(generated.text().contains("[X-A]\nOther=first\nOther=\n"));
    let entries: Vec<_> = generated
        .document()
        .entries()
        .filter(|entry| entry.key().text() == "Payload")
        .collect();
    assert_eq!(entries.len(), 2);
    assert!(entries.iter().all(|entry| entry.is_sensitive()));
    assert_eq!(entries[0].value().primary().text(), entries[1].value().primary().text());
    assert!(!format!("{generated:?}").contains(CANARY));
    let parsed = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(163), generated.text())?;
    assert!(parsed.is_valid());
    assert_eq!(parsed.syntax().document().render_preserved(), generated.text());
    assert!(!format!("{parsed:?}").contains(CANARY));
    Ok(())
}

#[test]
fn extension_validation_errors_never_retain_attempted_input() -> Result<(), Box<dyn std::error::Error>> {
    for name in [
        "",
        "X-",
        "Unit",
        "Service",
        "x-vendor",
        "X-é",
        "X-_name",
        "X-name]",
        "X-name\n[Service]",
        "X-name\0",
    ] {
        let error = XSectionName::new(name).err().ok_or("unsafe section accepted")?;
        assert_eq!(error, RenderError::InvalidExtensionSection);
    }
    for value in [
        format!("{CANARY}\nInjected=yes"),
        format!("{CANARY}\r"),
        format!("{CANARY}\0"),
        format!("{CANARY}\\"),
        format!("{CANARY}\\  "),
        format!("{CANARY}\\\\"),
    ] {
        let error = ExtensionValue::new(value).err().ok_or("unsafe value accepted")?;
        assert_eq!(error, RenderError::InvalidExtensionValue);
        assert!(!format!("{error:?} {error}").contains(CANARY));
    }
    for key in [
        String::new(),
        format!("{CANARY}=bad"),
        format!("{CANARY}\nOther"),
        format!("{CANARY}]"),
        "Not_ASCII".to_owned(),
        "é".to_owned(),
    ] {
        let mut builder = QuadletDocumentBuilder::new(QuadletUnitType::Container);
        let error = builder
            .push_extension_entry(XSectionName::new("X-Test")?, key, ExtensionValue::new(CANARY)?)
            .err()
            .ok_or("unsafe key accepted")?;
        assert_eq!(error, RenderError::InvalidExtensionKey);
        assert!(!format!("{error:?} {error}").contains(CANARY));
    }
    let error = XSectionName::new(format!("X-{CANARY}\n"))
        .err()
        .ok_or("unsafe name accepted")?;
    assert!(!format!("{error:?} {error}").contains(CANARY));
    for value in [
        "",
        "café",
        "[Service]",
        "key=value",
        "\\ escaped",
        "{\"json\":true}",
        "%h $value # ;",
    ] {
        assert_eq!(ExtensionValue::new(value)?.as_str(), value);
    }
    Ok(())
}

#[test]
fn freshly_parsed_extension_payloads_are_protected_through_public_debug_subobjects()
-> Result<(), Box<dyn std::error::Error>> {
    let text = format!("[Container]\nImage=example.invalid/app:1\n[X-Unknown]\nUnknown={CANARY}\\\n  {CANARY}\n");
    let parsed = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(164), &text)?;
    assert!(parsed.is_valid());
    let syntax = parsed.syntax();
    for debug in [
        format!("{parsed:?}"),
        format!("{:?}", parsed.document()),
        format!("{syntax:?}"),
        format!("{:?}", syntax.document()),
        format!("{:?}", syntax.document().source()),
        format!("{:?}", syntax.diagnostics()),
        format!("{:?}", parsed.model_diagnostics()),
    ] {
        assert!(!debug.contains(CANARY), "{debug}");
    }
    let entry = parsed
        .document()
        .entries()
        .find(|entry| entry.key().text() == "Unknown")
        .ok_or("missing extension entry")?;
    for debug in [
        format!("{entry:?}"),
        format!("{:?}", entry.value()),
        format!("{:?}", entry.value().primary()),
        format!("{:?}", entry.value().continuations()),
    ] {
        assert!(!debug.contains(CANARY), "{debug}");
    }
    assert!(entry.is_sensitive());
    assert!(entry.value().primary().text().contains(CANARY));
    assert!(entry.value().continuations()[0].text().contains(CANARY));
    assert_eq!(syntax.document().source().text(), text);
    assert_eq!(syntax.document().render_preserved(), text);
    let fresh = SyntaxDocument::parse(SourceId::new(165), &text);
    assert!(!format!("{fresh:?}").contains(CANARY));
    let invalid = SyntaxDocument::parse(SourceId::new(166), format!("[X-Test]\n{CANARY}\nPayload={CANARY}\\"));
    assert!(!invalid.is_valid());
    assert!(!format!("{invalid:?}").contains(CANARY));
    assert!(!format!("{:?}", invalid.diagnostics()).contains(CANARY));
    Ok(())
}

#[test]
fn invalid_generated_document_diagnostics_do_not_expose_extension_values() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    builder.push_extension_entry(XSectionName::new("X-Test")?, "Payload", ExtensionValue::new(CANARY)?)?;
    let error = builder
        .build(SourceId::new(167))
        .err()
        .ok_or("missing image accepted")?;
    assert!(!format!("{error:?} {error}").contains(CANARY));
    assert!(!format!("{:?}", error.diagnostics()).contains(CANARY));
    Ok(())
}

#[test]
fn native_continuations_cannot_absorb_extension_sections() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    builder.push_container(ContainerKey::Image, EntryValue::new("example.invalid/app:1\\")?)?;
    builder.push_extension_entry(XSectionName::new("X-Test")?, "ExecStart", ExtensionValue::new(CANARY)?)?;
    let error = builder
        .build(SourceId::new(168))
        .err()
        .ok_or("unsafe continuation accepted")?;
    assert_eq!(error, RenderError::InvalidExtensionContext);
    assert!(!format!("{error:?} {error}").contains(CANARY));
    let mut builder = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    builder.push_container(ContainerKey::Image, EntryValue::new("example.invalid/app:1")?)?;
    builder.push_systemd(
        quadlet_lens::render::SystemdSection::Install,
        "WantedBy",
        EntryValue::new("default.target\\")?,
    )?;
    builder.push_extension_entry(XSectionName::new("X-Test")?, "Payload", ExtensionValue::new(CANARY)?)?;
    assert_eq!(
        builder.build(SourceId::new(169)).err(),
        Some(RenderError::InvalidExtensionContext)
    );
    Ok(())
}

#[test]
fn malformed_or_consumed_x_headers_protect_every_returned_typed_value() -> Result<(), Box<dyn std::error::Error>> {
    for text in [
        format!("[Container]\nImage=example.invalid/app:1\n[X-Broken\nUnknown={CANARY}\n"),
        format!("[Container]\nImage=example.invalid/app:1\\\n[X-Consumed]\nUnknown={CANARY}\n"),
        format!("[Container]\nImage=example.invalid/app:1\\\n[X-Consumed]\\\n{CANARY}\n"),
    ] {
        let parsed = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(170), &text)?;
        assert!(!format!("{parsed:?}").contains(CANARY));
        assert!(!format!("{:?}", parsed.document()).contains(CANARY));
        assert!(!format!("{:?}", parsed.model_diagnostics()).contains(CANARY));
        for entry in parsed.document().entries() {
            assert!(entry.is_sensitive());
            for debug in [
                format!("{entry:?}"),
                format!("{:?}", entry.value()),
                format!("{:?}", entry.value().primary()),
                format!("{:?}", entry.value().continuations()),
            ] {
                assert!(!debug.contains(CANARY), "{debug}");
            }
        }
        assert_eq!(parsed.syntax().document().render_preserved(), text);
    }
    let text = format!("[Container]\nImage=example.invalid/app:1\nUnknown={CANARY}\n");
    let ordinary = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(171), &text)?;
    assert!(ordinary.is_valid());
    assert!(ordinary.document().entries().all(|entry| !entry.is_sensitive()));
    assert!(format!("{ordinary:?}").contains(CANARY));
    assert_eq!(ordinary.syntax().document().render_preserved(), text);
    Ok(())
}

fn assert_protected_debug(value: &impl std::fmt::Debug) {
    assert!(!format!("{value:?}").contains(CANARY));
    assert!(!format!("{value:#?}").contains(CANARY));
}

#[test]
fn decoded_semantic_views_keep_protected_text_out_of_debug() -> Result<(), Box<dyn std::error::Error>> {
    use quadlet_lens::model::{
        AuthoredContainerEnvironmentValue, NativeCommandDirective, NativeMountDirective, NativeMountOption,
        NativePortPublication,
    };
    for candidate in [
        format!("[X-Valid]\nPayload={CANARY}\n[Container]\n"),
        "[X-Broken\n".to_owned(),
        "Label=continued\\\n[X-Consumed]\n".to_owned(),
    ] {
        let text = format!(
            "[Container]\nImage=example.invalid/app:1\n{candidate}PublishPort={CANARY}:18080:80\nMount=type=bind,src=/{CANARY},dst=/{CANARY},{CANARY},{CANARY}=opaque\nVolume={CANARY}.volume:/data:{CANARY}\nExec={CANARY}\nEntrypoint=[\"{CANARY}\"]\nEnvironment={CANARY}={CANARY}\nEnvironment={CANARY}\nEnvironment={CANARY}=%h\nEnvironment=SAFE_EXTENSION_LITERAL={CANARY}\nEnvironmentFile=/{CANARY}\nSecret={CANARY},type=env,target={CANARY}\n"
        );
        let parsed = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(172), &text)?;
        let document = parsed.document();
        let ports = document.container_ports();
        assert_protected_debug(&ports);
        assert_protected_debug(&ports.directives());
        match &ports.directives()[0] {
            NativePortPublication::Publication(port) => {
                assert_protected_debug(port);
                assert_eq!(port.host(), Some(CANARY));
            }
            _ => return Err("expected decoded port".into()),
        }
        let mounts = document.container_mounts();
        assert_protected_debug(&mounts);
        for directive in mounts.directives() {
            assert_protected_debug(directive);
            if let NativeMountDirective::Mount(mount) = directive {
                assert_protected_debug(mount);
                for option in mount.options() {
                    assert_protected_debug(option);
                }
            } else {
                return Err("expected decoded mount".into());
            }
        }
        match &mounts.directives()[0] {
            NativeMountDirective::Mount(mount) => {
                assert_eq!(mount.source(), Some(format!("/{CANARY}").as_str()));
                assert!(
                    mount
                        .options()
                        .iter()
                        .any(|option| matches!(option, NativeMountOption::Flag(name) if name == CANARY))
                );
                assert!(
                    mount
                        .options()
                        .iter()
                        .any(|option| matches!(option, NativeMountOption::Assignment { name, .. } if name == CANARY))
                );
            }
            _ => return Err("expected decoded mount".into()),
        }
        let commands = document.container_commands();
        assert_protected_debug(&commands);
        for directive in commands.directives() {
            assert_protected_debug(directive);
            if let NativeCommandDirective::Command { command, .. } = directive {
                assert_protected_debug(command);
                assert_eq!(command.arguments(), [CANARY]);
            } else {
                return Err("expected decoded command".into());
            }
        }
        let environment = document.container_environment();
        assert_protected_debug(&environment);
        for directive in environment.directives() {
            assert_protected_debug(directive);
            assert!(matches!(directive.name(), Some(name) if name == CANARY || name == "SAFE_EXTENSION_LITERAL"));
        }
        assert_protected_debug(&environment.get(CANARY));
        assert_protected_debug(&environment.get("SAFE_EXTENSION_LITERAL"));
        assert!(
            matches!(environment.get("SAFE_EXTENSION_LITERAL"), AuthoredContainerEnvironmentValue::Literal(value) if value == CANARY)
        );
        // The first assignment still exposes its literal only through explicit access.
        assert_eq!(environment.directives()[0].literal_value(), Some(CANARY));
        let literal = AuthoredContainerEnvironmentValue::Literal(CANARY);
        assert_protected_debug(&literal);
        let sources = document.container_environment_sources();
        assert_protected_debug(&sources);
        for reference in sources.environment_files() {
            assert_protected_debug(reference);
        }
        for reference in sources.environment_secrets() {
            assert_protected_debug(reference);
            assert_eq!(reference.secret(), Some(CANARY));
            assert_eq!(reference.target(), Some(CANARY));
        }
        let resolved = sources.resolve(&quadlet_lens::model::AuthorizedContainerEnvironment::new());
        assert_protected_debug(&resolved);
        for reference in resolved.environment_secrets() {
            assert_protected_debug(reference);
        }
        assert_eq!(parsed.syntax().document().render_preserved(), text);
    }
    Ok(())
}

#[test]
fn native_and_systemd_unit_reference_debug_propagates_source_protection() -> Result<(), Box<dyn std::error::Error>> {
    use quadlet_lens::model::{NamedQuadletDocument, QuadletDocumentSet};
    for candidate in [
        "[X-Valid]\nPayload=opaque\n[Container]\n",
        "[X-Broken\n",
        "Label=continued\\\n[X-Consumed]\n",
        "",
    ] {
        let text = format!(
            "[Container]\nImage=example.invalid/app:1\n{candidate}Network={CANARY}.network\n[Unit]\nRequires={CANARY}.container\n"
        );
        let parsed = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(175), &text)?;
        let set = QuadletDocumentSet::new([NamedQuadletDocument::new(
            "source.container",
            parsed.document().clone(),
        )?])?;
        assert_eq!(set.graph().references().len(), 2);
        for reference in set.graph().references() {
            assert!(reference.target_name().contains(CANARY));
            if candidate.is_empty() {
                assert!(format!("{reference:?}").contains(CANARY));
            } else {
                assert_protected_debug(reference);
            }
        }
        if !candidate.is_empty() {
            assert_protected_debug(&set);
            assert_protected_debug(set.graph());
            assert_protected_debug(&set.diagnostics());
        }
        assert_eq!(parsed.syntax().document().render_preserved(), text);
    }
    Ok(())
}

#[test]
fn nonprotected_private_views_retain_ordinary_debug() -> Result<(), Box<dyn std::error::Error>> {
    let text = format!(
        "[Container]\nImage=example.invalid/app:1\nPublishPort={CANARY}:18080:80\nSecret={CANARY},type=env,target={CANARY}\n"
    );
    let parsed = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(176), text)?;
    assert!(format!("{:?}", parsed.document().container_ports()).contains(CANARY));
    assert!(
        format!(
            "{:?}",
            parsed.document().container_environment_sources().environment_secrets()
        )
        .contains(CANARY)
    );
    Ok(())
}

#[test]
fn pod_and_build_views_protect_malformed_extension_source() -> Result<(), Box<dyn std::error::Error>> {
    let pod_text =
        format!("[Pod]\nPublishPort={CANARY}:18080:80\nMount=type=bind,src=/safe,dst=/data,{CANARY}\n[X-Broken\n");
    let pod = QuadletDocument::parse(QuadletUnitType::Pod, SourceId::new(173), pod_text)?;
    assert_protected_debug(&pod.document().pod_ports());
    assert_protected_debug(&pod.document().pod_mounts());
    let build_text = format!("[Build]\nImageTag=example.invalid/build:1\nEnvironment={CANARY}={CANARY}\n[X-Broken\n");
    let build = QuadletDocument::parse(QuadletUnitType::Build, SourceId::new(174), build_text)?;
    let environment = build.document().build_environment();
    assert_protected_debug(&environment);
    assert_protected_debug(&environment.directives());
    assert_protected_debug(&environment.get(CANARY));
    Ok(())
}

#[test]
fn sensitive_port_debug_redacts_numeric_ranges_and_protocol() -> Result<(), Box<dyn std::error::Error>> {
    use quadlet_lens::model::{NativePortProtocol, NativePortPublication};
    for unit_type in [QuadletUnitType::Container, QuadletUnitType::Pod] {
        let (prefix, section) = match unit_type {
            QuadletUnitType::Container => ("[Container]\nImage=example.invalid/app:1\n", "Container"),
            _ => ("[Pod]\n", "Pod"),
        };
        for candidate in [
            format!("[X-Valid]\nPayload=opaque\n[{section}]\n"),
            "[X-Broken\n".to_owned(),
            "Label=continued\\\n[X-Consumed]\n".to_owned(),
            String::new(),
        ] {
            let text = format!("{prefix}{candidate}PublishPort=54321:12345/udp\n");
            let parsed = QuadletDocument::parse(unit_type, SourceId::new(177), &text)?;
            let ports = match unit_type {
                QuadletUnitType::Container => parsed.document().container_ports(),
                _ => parsed.document().pod_ports(),
            };
            let NativePortPublication::Publication(port) = &ports.directives()[0] else {
                return Err("expected numeric publication".into());
            };
            assert_eq!(port.published().ok_or("expected published port")?.start(), 54321);
            assert_eq!(port.container().start(), 12345);
            assert_eq!(port.protocol(), NativePortProtocol::Udp);
            let entry = parsed
                .document()
                .entries()
                .find(|entry| entry.key().text() == "PublishPort")
                .ok_or("expected source entry")?;
            assert_eq!(port.span(), entry.value().primary().span());
            for debug in [
                format!("{ports:?}"),
                format!("{ports:#?}"),
                format!("{:?}", ports.directives()),
                format!("{:#?}", ports.directives()),
                format!("{:?}", ports.directives()[0]),
                format!("{:#?}", ports.directives()[0]),
                format!("{port:?}"),
                format!("{port:#?}"),
            ] {
                for payload in ["54321", "12345", "Udp"] {
                    assert_eq!(debug.contains(payload), candidate.is_empty());
                }
            }
            assert_eq!(parsed.syntax().document().render_preserved(), text);
        }
    }
    Ok(())
}
