//! Source-aware native Quadlet value view contracts.

use quadlet_lens::capability::{CapabilityCatalogue, PodmanTarget, PodmanVersion, SupportClassification};
use quadlet_lens::model::{
    AuthoredContainerEnvironmentDirective, NativeCommandDirective, NativeCommandKind, NativeCommandSyntax,
    NativeMountDirective, NativeMountOption, NativePortProtocol, NativePortPublication, QuadletDocument,
    QuadletUnitType,
};
use quadlet_lens::source::SourceId;

fn document(unit_type: QuadletUnitType, source: &str) -> Result<QuadletDocument, String> {
    QuadletDocument::parse(unit_type, SourceId::new(9_700), source)
        .map(|parsed| parsed.document().clone())
        .map_err(|error| error.to_string())
}

#[test]
fn native_port_views_preserve_valid_forms_reset_and_recoverable_states() -> Result<(), String> {
    let container = document(
        QuadletUnitType::Container,
        concat!(
            "[Container]\nImage=example.invalid/application\n",
            "PublishPort=127.0.0.1::9090\n",
            "PublishPort=[::1]:18080-18081:8080-8081/tcp\n",
            "PublishPort=53:53/udp\nPublishPort=\n",
            "PublishPort=18080-18081:8080\nPublishPort=%i:8080\n",
        ),
    )?;
    let ports = container.container_ports();
    assert_eq!(ports.directives().len(), 6);
    assert!(ports.diagnostics().iter().any(|item| item.code().as_str() == "QLM0029"));
    assert!(ports.diagnostics().iter().any(|item| item.code().as_str() == "QLM0030"));

    match &ports.directives()[0] {
        NativePortPublication::Publication(port) => {
            assert_eq!(port.host(), Some("127.0.0.1"));
            assert_eq!(port.published(), None);
            assert_eq!(port.container().start(), 9090);
        }
        other => return Err(format!("expected native port, got {other:?}")),
    }
    match &ports.directives()[1] {
        NativePortPublication::Publication(port) => {
            assert_eq!(port.host(), Some("[::1]"));
            assert_eq!(port.published().ok_or("expected published range")?.len(), 2);
            assert_eq!(port.container().end(), 8081);
            assert_eq!(port.protocol(), NativePortProtocol::Tcp);
        }
        other => return Err(format!("expected IPv6 native port, got {other:?}")),
    }
    assert!(matches!(ports.directives()[3], NativePortPublication::Reset { .. }));
    assert!(matches!(ports.directives()[4], NativePortPublication::Unmodeled { .. }));
    assert!(matches!(ports.directives()[5], NativePortPublication::Deferred { .. }));

    let pod = document(
        QuadletUnitType::Pod,
        "[Pod]\nPublishPort=8443:443/sctp\nVolume=cache.volume:/var/cache:z\n",
    )?;
    match &pod.pod_ports().directives()[0] {
        NativePortPublication::Publication(port) => {
            assert_eq!(port.protocol(), NativePortProtocol::Sctp);
            assert_eq!(port.published().ok_or("expected published port")?.start(), 8443);
        }
        other => return Err(format!("expected Pod native port, got {other:?}")),
    }
    Ok(())
}

#[test]
fn native_mount_views_retain_opaque_native_evidence() -> Result<(), String> {
    let container = document(
        QuadletUnitType::Container,
        concat!(
            "[Container]\nImage=example.invalid/application\n",
            "Volume=cache.volume:/var/cache:z,Z\n",
            "Volume=./relative:/work\n",
            "Mount=type=tmpfs,destination=/run/cache,tmpfs-size=64m,unknown-option=still-native\n",
            "Mount=type=bind,source=%h/data,target=/data\n",
            "Mount=source=/missing-type,target=/bad\n",
            "Volume=too:many:fields:here\n",
        ),
    )?;
    let mounts = container.container_mounts();
    assert_eq!(mounts.directives().len(), 6);
    assert!(
        mounts
            .diagnostics()
            .iter()
            .any(|item| item.code().as_str() == "QLM0031")
    );
    assert!(
        mounts
            .diagnostics()
            .iter()
            .any(|item| item.code().as_str() == "QLM0032")
    );
    match &mounts.directives()[0] {
        NativeMountDirective::Mount(mount) => {
            assert!(!mount.is_long_form());
            assert_eq!(mount.source(), Some("cache.volume"));
            assert_eq!(mount.destination(), Some("/var/cache"));
            assert!(
                matches!(mount.options(), [NativeMountOption::Flag(first), NativeMountOption::Flag(second)] if first == "z" && second == "Z")
            );
        }
        other => return Err(format!("expected short native mount, got {other:?}")),
    }
    match &mounts.directives()[2] {
        NativeMountDirective::Mount(mount) => {
            assert!(mount.is_long_form());
            assert_eq!(mount.source(), None);
            assert_eq!(mount.destination(), Some("/run/cache"));
            assert!(
                matches!(mount.options().last(), Some(NativeMountOption::Assignment { name, value }) if name == "unknown-option" && value == "still-native")
            );
        }
        other => return Err(format!("expected long native mount, got {other:?}")),
    }
    assert!(matches!(mounts.directives()[3], NativeMountDirective::Deferred { .. }));
    assert!(matches!(mounts.directives()[4], NativeMountDirective::Unmodeled { .. }));
    assert!(matches!(mounts.directives()[5], NativeMountDirective::Unmodeled { .. }));
    assert!(!format!("{mounts:?}").contains("still-native"));
    Ok(())
}

#[test]
fn native_command_and_build_environment_views_are_bounded_and_redacted() -> Result<(), String> {
    let container = document(
        QuadletUnitType::Container,
        concat!(
            "[Container]\nImage=example.invalid/application\n",
            "Exec=/usr/bin/env sh -c \"echo hidden-command\"\n",
            "Entrypoint=[\"/bin/app\", \"--token=hidden-entrypoint\"]\n",
            "Entrypoint=[]\nEntrypoint=/usr/bin/plain-app\nExec=/usr/bin/app \"unterminated\nExec=%i\nExec=\n",
        ),
    )?;
    let commands = container.container_commands();
    assert_eq!(commands.directives().len(), 7);
    assert!(
        commands
            .diagnostics()
            .iter()
            .any(|item| item.code().as_str() == "QLM0033")
    );
    assert!(
        commands
            .diagnostics()
            .iter()
            .any(|item| item.code().as_str() == "QLM0034")
    );
    match &commands.directives()[0] {
        NativeCommandDirective::Command { kind, command } => {
            assert_eq!(*kind, NativeCommandKind::Exec);
            assert_eq!(command.arguments()[0], "/usr/bin/env");
            assert_eq!(command.arguments()[3], "echo hidden-command");
        }
        other => return Err(format!("expected Exec command, got {other:?}")),
    }
    assert!(matches!(
        commands.directives()[1],
        NativeCommandDirective::Command {
            kind: NativeCommandKind::Entrypoint,
            ..
        }
    ));
    assert!(matches!(
        commands.directives()[2],
        NativeCommandDirective::Command {
            kind: NativeCommandKind::Entrypoint,
            ..
        }
    ));
    assert!(matches!(
        commands.directives()[3],
        NativeCommandDirective::Command {
            kind: NativeCommandKind::Entrypoint,
            ..
        }
    ));
    assert!(matches!(
        commands.directives()[4],
        NativeCommandDirective::Unmodeled {
            kind: NativeCommandKind::Exec,
            ..
        }
    ));
    assert!(matches!(
        commands.directives()[5],
        NativeCommandDirective::Deferred {
            kind: NativeCommandKind::Exec,
            ..
        }
    ));
    assert!(matches!(
        commands.directives()[6],
        NativeCommandDirective::Reset {
            kind: NativeCommandKind::Exec,
            ..
        }
    ));
    assert!(!format!("{commands:?}").contains("hidden-command"));
    assert!(!format!("{commands:?}").contains("hidden-entrypoint"));

    let build = document(
        QuadletUnitType::Build,
        "[Build]\nEnvironment=BUILD_TOKEN=hidden-build\nEnvironment=\nEnvironment=AFTER=visible\nEnvironment=HOST=%h\n",
    )?;
    let environment = build.build_environment();
    assert_eq!(environment.directives().len(), 4);
    assert!(matches!(
        environment.directives()[0],
        AuthoredContainerEnvironmentDirective::Assignment { .. }
    ));
    assert!(
        environment
            .diagnostics()
            .iter()
            .any(|item| item.code().as_str() == "QLM0024")
    );
    assert!(!format!("{environment:?}").contains("hidden-build"));
    Ok(())
}

#[test]
fn native_overlay_volume_option_remains_native_evidence_without_portability_claim() -> Result<(), String> {
    let container = document(
        QuadletUnitType::Container,
        "[Container]\nImage=example.invalid/application\nVolume=cache.volume:/var/overlay:O\n",
    )?;
    let mounts = container.container_mounts();
    assert!(mounts.diagnostics().is_empty());
    assert!(matches!(
        mounts.directives(),
        [NativeMountDirective::Mount(mount)]
            if mount.source() == Some("cache.volume")
                && mount.destination() == Some("/var/overlay")
                && matches!(mount.options(), [NativeMountOption::Flag(option)] if option == "O")
    ));
    Ok(())
}

#[test]
fn native_views_preserve_exact_spans_and_bounded_lexical_command_arguments() -> Result<(), String> {
    let source_id = SourceId::new(9_701);
    let source = concat!(
        "[Container]\nImage=example.invalid/application\n",
        "PublishPort=\"8080:80\"\n",
        "Mount=\"type=tmpfs,target=/run/cache\\x20space,mode=0700,mode=0755\"\n",
        "Exec=:-/usr/bin/app before \"\" after\n",
        "Exec=/usr/bin/printf %%h\n",
        "Exec=/usr/bin/app ; retained\n",
    );
    let parsed =
        QuadletDocument::parse(QuadletUnitType::Container, source_id, source).map_err(|error| error.to_string())?;
    let document = parsed.document();

    let ports = document.container_ports();
    let port = match &ports.directives()[0] {
        NativePortPublication::Publication(port) => port,
        other => return Err(format!("expected native port, got {other:?}")),
    };
    assert_eq!(port.span().source_id(), source_id);
    assert_eq!(port.span().start(), source.find("\"8080:80\"").ok_or("port source")?);
    assert_eq!(port.span().len(), "\"8080:80\"".len());

    let mounts = document.container_mounts();
    let mount = match &mounts.directives()[0] {
        NativeMountDirective::Mount(mount) => mount,
        other => return Err(format!("expected native mount, got {other:?}")),
    };
    assert_eq!(mount.span().source_id(), source_id);
    assert_eq!(mount.span().start(), source.find("\"type=tmpfs").ok_or("mount source")?);
    assert!(matches!(
        mount.options(),
        [
            NativeMountOption::Assignment { name: first_name, value: first_value },
            NativeMountOption::Assignment { name: target_name, value: target_value },
            NativeMountOption::Assignment { name: first_mode, value: first_mode_value },
            NativeMountOption::Assignment { name: second_mode, value: second_mode_value },
        ] if first_name == "type" && first_value == "tmpfs"
            && target_name == "target" && target_value == "/run/cache space"
            && first_mode == "mode" && first_mode_value == "0700"
            && second_mode == "mode" && second_mode_value == "0755"
    ));

    let commands = document.container_commands();
    match &commands.directives()[0] {
        NativeCommandDirective::Command {
            kind: NativeCommandKind::Exec,
            command,
        } => {
            assert_eq!(command.arguments(), [":-/usr/bin/app", "before", "", "after"]);
            assert_eq!(command.span().source_id(), source_id);
            assert_eq!(
                command.span().start(),
                source.find(":-/usr/bin/app").ok_or("command source")?
            );
        }
        other => return Err(format!("expected lexical Exec argv, got {other:?}")),
    }
    match &commands.directives()[1] {
        NativeCommandDirective::Command { command, .. } => {
            assert_eq!(command.arguments(), ["/usr/bin/printf", "%h"]);
        }
        other => return Err(format!("expected literal-percent Exec argv, got {other:?}")),
    }
    match &commands.directives()[2] {
        NativeCommandDirective::Command { command, .. } => {
            assert_eq!(command.arguments(), ["/usr/bin/app", ";", "retained"]);
        }
        other => return Err(format!("expected lexical Exec separator, got {other:?}")),
    }
    Ok(())
}

#[test]
fn continued_native_values_keep_primary_spans_and_do_not_select_targets() -> Result<(), String> {
    let source_id = SourceId::new(9_702);
    let source = concat!(
        "[Container]\nImage=example.invalid/application\n",
        "Entrypoint=[\"/usr/bin/app\", \\\n",
        "  \"--serve\"]\n",
    );
    let parsed =
        QuadletDocument::parse(QuadletUnitType::Container, source_id, source).map_err(|error| error.to_string())?;
    let commands = parsed.document().container_commands();
    let NativeCommandDirective::Command { command, .. } = &commands.directives()[0] else {
        return Err("continued Entrypoint was not decoded".to_owned());
    };
    assert_eq!(command.arguments(), ["/usr/bin/app", "--serve"]);
    assert_eq!(command.span().source_id(), source_id);
    assert_eq!(
        &source[command.span().start()..command.span().end()],
        "[\"/usr/bin/app\", \\"
    );

    let catalogue = CapabilityCatalogue::supported_range().map_err(|error| error.to_string())?;
    for (version, expected) in [
        (PodmanVersion::new(5, 3, 3), SupportClassification::Unknown),
        (PodmanVersion::new(5, 4, 0), SupportClassification::Native),
    ] {
        let target = PodmanTarget::new(version, Some(version)).map_err(|error| error.to_string())?;
        assert_eq!(
            catalogue
                .evaluate("quadlet.container.entrypoint", target)
                .classification(),
            expected
        );
        assert_eq!(
            commands.directives().len(),
            1,
            "native decoding must not vary by target"
        );
    }
    Ok(())
}

#[test]
fn literal_entrypoint_is_distinct_native_evidence() -> Result<(), String> {
    let source = "[Container]\nImage=example.invalid/application\nEntrypoint=/usr/bin/app\n";
    let commands = document(QuadletUnitType::Container, source)?.container_commands();
    let [NativeCommandDirective::Command { kind, command }] = commands.directives() else {
        return Err(format!("expected one decoded literal Entrypoint, got {commands:?}"));
    };
    assert_eq!(*kind, NativeCommandKind::Entrypoint);
    assert_eq!(command.syntax(), NativeCommandSyntax::EntrypointLiteral);
    assert_eq!(command.arguments(), ["/usr/bin/app"]);
    assert_eq!(&source[command.span().start()..command.span().end()], "/usr/bin/app");
    assert!(commands.diagnostics().is_empty());
    Ok(())
}
