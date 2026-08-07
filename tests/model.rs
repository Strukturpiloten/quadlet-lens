//! Native typed documents, conservative value forms, and model diagnostics.

use quadlet_lens::diagnostic::Severity;
use quadlet_lens::model::{
    ContainerKey, EntryKind, NetworkKey, PodKey, QuadletDocument, QuadletUnitType, SectionKind, TypedEntry,
    UnitReferenceKind, ValueKind, VolumeKey,
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
            .filter(|entry| entry.kind() == EntryKind::Unknown)
            .map(|entry| entry.key().text())
            .collect::<Vec<_>>(),
        ["IP", "IP6", "NetworkAlias"]
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

    for (unit_type, source_id, source) in [
        (QuadletUnitType::Pod, SourceId::new(130), "[Pod]\nDNS=1.1.1.1\n"),
        (QuadletUnitType::Network, SourceId::new(131), "[Network]\nDNS=1.1.1.1\n"),
    ] {
        let result = QuadletDocument::parse(unit_type, source_id, source).map_err(|error| error.to_string())?;
        assert!(result.document().entries().any(|entry| {
            entry.kind() == EntryKind::Unknown
                && entry.key().text() == "DNS"
                && entry.value().primary().text() == "1.1.1.1"
        }));
    }
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
        (QuadletUnitType::Pod, SourceId::new(136), "[Pod]\nDNSOption=rotate\n"),
        (
            QuadletUnitType::Network,
            SourceId::new(137),
            "[Network]\nDNSOption=rotate\n",
        ),
    ] {
        let result = QuadletDocument::parse(unit_type, source_id, source).map_err(|error| error.to_string())?;
        assert!(result.document().entries().any(|entry| {
            entry.kind() == EntryKind::Unknown
                && entry.key().text() == "DNSOption"
                && entry.value().primary().text() == "rotate"
        }));
    }
    assert_eq!(QuadletUnitType::from_extension("build"), None);
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
    assert!(pod.document().entries().any(|entry| {
        entry.kind() == EntryKind::Unknown
            && entry.key().text() == "DNSSearch"
            && entry.value().primary().text() == "example.com"
    }));
    assert_eq!(QuadletUnitType::from_extension("build"), None);
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
fn annotation_is_container_only_repeatable_opaque_and_preserves_every_physical_value() -> Result<(), String> {
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
        [EntryKind::Unknown, EntryKind::GenericSystemd]
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
            PodKey::ShmSize
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
            ("DNS", "1.1.1.1"),
            ("DNSOption", "rotate"),
            ("DNSSearch", "example.com"),
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
        ["QLM0004", "QLM0004", "QLM0004", "QLM0004"]
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
        ["QLM0004", "QLM0004", "QLM0004", "QLM0004", "QLM0004", "QLM0004"]
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
        ["QLM0004", "QLM0004", "QLM0004", "QLM0004", "QLM0004", "QLM0004"]
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
        ["QLM0004", "QLM0004", "QLM0004", "QLM0004", "QLM0004", "QLM0004"]
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
        ["QLM0004", "QLM0004", "QLM0004", "QLM0004", "QLM0004", "QLM0004"]
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
        ["QLM0004", "QLM0004", "QLM0004", "QLM0004", "QLM0004", "QLM0004"]
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
        ["QLM0004", "QLM0004", "QLM0004", "QLM0004", "QLM0004", "QLM0004"]
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
        ["QLM0004", "QLM0004", "QLM0004", "QLM0004", "QLM0004", "QLM0004"]
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
    assert!(result.model_diagnostics().is_empty());
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
    assert!(result.model_diagnostics().is_empty());
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
