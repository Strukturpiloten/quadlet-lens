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
            EntryKind::Container(key) if is_extended_opaque_container_key(key) => None,
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
            ContainerKey::Entrypoint,
            ContainerKey::RunInit,
            ContainerKey::StopSignal,
            ContainerKey::StopTimeout,
            ContainerKey::Pull,
            ContainerKey::PidsLimit,
            ContainerKey::HostName,
            ContainerKey::ShmSize,
            ContainerKey::Memory,
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
            ("FuturePodKey", "future-value")
        ]
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

fn is_extended_opaque_container_key(key: ContainerKey) -> bool {
    matches!(
        key,
        ContainerKey::DropCapability
            | ContainerKey::AddCapability
            | ContainerKey::Tmpfs
            | ContainerKey::Sysctl
            | ContainerKey::Ulimit
            | ContainerKey::AddDevice
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
