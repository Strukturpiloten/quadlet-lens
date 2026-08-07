//! Consumer-facing compile and behavior contract for the supported 0.1.x API.

use quadlet_lens::capability::{CapabilityCatalogue, PodmanTarget, PodmanVersion, SupportClassification};
use quadlet_lens::model::{
    ContainerKey, EntryKind, NamedQuadletDocument, NetworkKey, PodKey, QuadletDocument, QuadletDocumentSet,
    QuadletUnitType, ValueKind, VolumeKey,
};
use quadlet_lens::path::{PathForm, classify_path};
use quadlet_lens::render::{
    EntryValue, Memory, MemoryError, PidsLimit, PidsLimitError, QuadletDocumentBuilder, ShmSize, ShmSizeError,
    SystemdUnitKey,
};
use quadlet_lens::source::SourceId;

#[test]
fn growing_public_key_enums_preserve_published_discriminants() {
    assert_eq!(
        [
            ContainerKey::AddHost as isize,
            ContainerKey::Image as isize,
            ContainerKey::Exec as isize,
            ContainerKey::Environment as isize,
            ContainerKey::EnvironmentFile as isize,
            ContainerKey::PublishPort as isize,
            ContainerKey::Volume as isize,
            ContainerKey::Network as isize,
            ContainerKey::Pod as isize,
            ContainerKey::HealthCmd as isize,
            ContainerKey::PodmanArgs as isize,
            ContainerKey::HealthInterval as isize,
            ContainerKey::HealthRetries as isize,
            ContainerKey::HealthStartPeriod as isize,
            ContainerKey::HealthTimeout as isize,
            ContainerKey::Notify as isize,
            ContainerKey::User as isize,
            ContainerKey::Group as isize,
            ContainerKey::UserNS as isize,
            ContainerKey::GroupAdd as isize,
            ContainerKey::WorkingDir as isize,
            ContainerKey::ReadOnly as isize,
            ContainerKey::Secret as isize,
            ContainerKey::Label as isize,
            ContainerKey::Rootfs as isize,
            ContainerKey::ContainerName as isize,
            ContainerKey::Entrypoint as isize,
            ContainerKey::RunInit as isize,
            ContainerKey::StopSignal as isize,
            ContainerKey::StopTimeout as isize,
            ContainerKey::Pull as isize,
            ContainerKey::PidsLimit as isize,
            ContainerKey::HostName as isize,
            ContainerKey::ShmSize as isize,
            ContainerKey::DropCapability as isize,
            ContainerKey::AddCapability as isize,
            ContainerKey::Tmpfs as isize,
            ContainerKey::Sysctl as isize,
            ContainerKey::Ulimit as isize,
            ContainerKey::AddDevice as isize,
            ContainerKey::Memory as isize,
            ContainerKey::DNS as isize,
            ContainerKey::DNSOption as isize,
            ContainerKey::DNSSearch as isize,
            ContainerKey::ExposeHostPort as isize,
            ContainerKey::Annotation as isize,
            ContainerKey::AppArmor as isize,
            ContainerKey::NoNewPrivileges as isize,
            ContainerKey::SeccompProfile as isize,
            ContainerKey::SecurityLabelDisable as isize,
            ContainerKey::SecurityLabelFileType as isize,
            ContainerKey::SecurityLabelLevel as isize,
            ContainerKey::SecurityLabelNested as isize,
            ContainerKey::SecurityLabelType as isize,
            ContainerKey::Mask as isize,
            ContainerKey::Unmask as isize,
            ContainerKey::LogDriver as isize,
            ContainerKey::LogOpt as isize,
            ContainerKey::IP as isize,
            ContainerKey::IP6 as isize,
            ContainerKey::NetworkAlias as isize,
        ],
        [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28,
            29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55,
            56, 57, 58, 59, 60,
        ]
    );
    assert_eq!(
        [
            PodKey::AddHost as isize,
            PodKey::PodName as isize,
            PodKey::PublishPort as isize,
            PodKey::Network as isize,
            PodKey::Volume as isize,
            PodKey::UserNS as isize,
            PodKey::ShmSize as isize,
        ],
        [0, 1, 2, 3, 4, 5, 6]
    );
    assert_eq!(
        [
            NetworkKey::NetworkName as isize,
            NetworkKey::Driver as isize,
            NetworkKey::Options as isize,
            NetworkKey::Internal as isize,
            NetworkKey::IPv6 as isize,
            NetworkKey::IPAMDriver as isize,
            NetworkKey::Subnet as isize,
            NetworkKey::Gateway as isize,
            NetworkKey::IPRange as isize,
            NetworkKey::Label as isize,
        ],
        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
    );
}

#[test]
fn growing_volume_key_enum_preserves_published_discriminants() {
    assert_eq!(
        [
            VolumeKey::VolumeName as isize,
            VolumeKey::Driver as isize,
            VolumeKey::Options as isize,
            VolumeKey::Label as isize,
            VolumeKey::Device as isize,
            VolumeKey::Type as isize,
            VolumeKey::Copy as isize,
        ],
        [0, 1, 2, 3, 4, 5, 6]
    );
}

#[test]
fn rootfs_can_be_built_through_the_public_api() -> Result<(), Box<dyn std::error::Error>> {
    let mut generated = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    generated.push_container(ContainerKey::Rootfs, EntryValue::new("/var/lib/application-rootfs")?)?;
    assert_eq!(
        generated.build(SourceId::new(4))?.text(),
        "[Container]\nRootfs=/var/lib/application-rootfs\n"
    );
    Ok(())
}

#[test]
fn container_name_can_be_built_through_the_public_api() -> Result<(), Box<dyn std::error::Error>> {
    let mut generated = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    generated.push_container(ContainerKey::Image, EntryValue::new("example.invalid/application:1")?)?;
    generated.push_container(ContainerKey::ContainerName, EntryValue::new("application-web")?)?;
    assert_eq!(
        generated.build(SourceId::new(5))?.text(),
        "[Container]\nImage=example.invalid/application:1\nContainerName=application-web\n"
    );
    Ok(())
}

#[test]
fn run_init_false_can_be_built_and_recovered_through_the_public_api() -> Result<(), Box<dyn std::error::Error>> {
    let mut generated = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    generated.push_container(ContainerKey::Image, EntryValue::new("example.invalid/application:1")?)?;
    generated.push_container(ContainerKey::RunInit, EntryValue::new("false")?)?;
    let generated = generated.build(SourceId::new(7))?;
    assert_eq!(
        generated.text(),
        "[Container]\nImage=example.invalid/application:1\nRunInit=false\n"
    );
    assert!(generated.document().entries().any(|entry| {
        entry.kind() == EntryKind::Container(ContainerKey::RunInit) && entry.value().primary().text() == "false"
    }));
    Ok(())
}

#[test]
fn container_stop_lifecycle_can_be_built_through_the_public_api() -> Result<(), Box<dyn std::error::Error>> {
    let mut generated = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    generated.push_container(ContainerKey::Image, EntryValue::new("example.invalid/application:1")?)?;
    generated.push_container(ContainerKey::StopSignal, EntryValue::new("SIGUSR1")?)?;
    generated.push_container(ContainerKey::StopTimeout, EntryValue::new("0")?)?;
    let generated = generated.build(SourceId::new(6))?;
    assert_eq!(
        generated.text(),
        "[Container]\nImage=example.invalid/application:1\nStopSignal=SIGUSR1\nStopTimeout=0\n"
    );
    let recovered: Vec<_> = generated
        .document()
        .entries()
        .filter_map(|entry| match entry.kind() {
            EntryKind::Container(key @ (ContainerKey::StopSignal | ContainerKey::StopTimeout)) => {
                Some((key, entry.value().primary().text()))
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        recovered,
        [(ContainerKey::StopSignal, "SIGUSR1"), (ContainerKey::StopTimeout, "0")]
    );
    Ok(())
}

#[test]
fn container_pull_can_be_built_and_recovered_through_the_public_api() -> Result<(), Box<dyn std::error::Error>> {
    let mut generated = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    generated.push_container(ContainerKey::Image, EntryValue::new("example.invalid/application:1")?)?;
    generated.push_container(ContainerKey::Pull, EntryValue::new("newer")?)?;
    let generated = generated.build(SourceId::new(8))?;
    assert_eq!(
        generated.text(),
        "[Container]\nImage=example.invalid/application:1\nPull=newer\n"
    );
    assert!(generated.document().entries().any(|entry| {
        entry.kind() == EntryKind::Container(ContainerKey::Pull) && entry.value().primary().text() == "newer"
    }));
    Ok(())
}

#[test]
fn container_pids_limit_has_safe_typed_and_raw_public_construction() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(PidsLimit::finite("0"), Err(PidsLimitError::Zero));
    assert_eq!(PidsLimit::finite("1.5"), Err(PidsLimitError::NonDecimal));
    assert_eq!(
        PidsLimit::finite("999999999999999999999999999999999999")?.as_str(),
        "999999999999999999999999999999999999"
    );

    let mut generated = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    generated.push_container(ContainerKey::Image, EntryValue::new("example.invalid/application:1")?)?;
    generated.push_container(ContainerKey::PidsLimit, PidsLimit::unlimited().into())?;
    let generated = generated.build(SourceId::new(9))?;
    assert_eq!(
        generated.text(),
        "[Container]\nImage=example.invalid/application:1\nPidsLimit=-1\n"
    );
    assert!(generated.document().entries().any(|entry| {
        entry.kind() == EntryKind::Container(ContainerKey::PidsLimit) && entry.value().primary().text() == "-1"
    }));

    let raw_zero = EntryValue::new("0")?;
    assert_eq!(raw_zero.as_str(), "0");

    let catalogue = CapabilityCatalogue::supported_range()?;
    let target = PodmanTarget::new(PodmanVersion::new(5, 4, 0), Some(PodmanVersion::new(6, 0, 2)))?;
    assert_eq!(
        catalogue
            .evaluate("quadlet.container.pids-limit", target)
            .classification(),
        SupportClassification::Native
    );
    Ok(())
}

#[test]
fn container_hostname_can_be_built_and_recovered_through_the_public_api() -> Result<(), Box<dyn std::error::Error>> {
    let mut generated = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    generated.push_container(ContainerKey::Image, EntryValue::new("example.invalid/application:1")?)?;
    generated.push_container(ContainerKey::HostName, EntryValue::new("app.example")?)?;
    let generated = generated.build(SourceId::new(10))?;
    assert_eq!(
        generated.text(),
        "[Container]\nImage=example.invalid/application:1\nHostName=app.example\n"
    );
    assert!(generated.document().entries().any(|entry| {
        entry.kind() == EntryKind::Container(ContainerKey::HostName) && entry.value().primary().text() == "app.example"
    }));
    Ok(())
}

#[test]
fn container_and_pod_shm_size_have_safe_typed_public_construction() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(ShmSize::new("64mb"), Err(ShmSizeError::InvalidFormat));
    assert_eq!(ShmSize::new("00064m")?.as_str(), "00064m");
    assert!(ShmSize::unlimited().is_unlimited());

    let mut container = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    container.push_container(ContainerKey::Image, EntryValue::new("example.invalid/application:1")?)?;
    container.push_container(ContainerKey::ShmSize, ShmSize::new("00064m")?.into())?;
    assert_eq!(
        container.build(SourceId::new(11))?.text(),
        "[Container]\nImage=example.invalid/application:1\nShmSize=00064m\n"
    );

    let mut pod = QuadletDocumentBuilder::new(QuadletUnitType::Pod);
    pod.push_pod(PodKey::ShmSize, ShmSize::unlimited().into())?;
    assert_eq!(pod.build(SourceId::new(12))?.text(), "[Pod]\nShmSize=0\n");
    Ok(())
}

#[test]
fn container_memory_has_safe_typed_public_construction() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(Memory::new("0"), Err(MemoryError::Zero));
    assert_eq!(Memory::new("64mb"), Err(MemoryError::InvalidFormat));
    assert_eq!(Memory::new("00016777216b")?.as_str(), "00016777216b");

    let mut container = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    container.push_container(ContainerKey::Image, EntryValue::new("example.invalid/application:1")?)?;
    container.push_container(ContainerKey::Memory, Memory::new("00016777216b")?.into())?;
    assert_eq!(
        container.build(SourceId::new(19))?.text(),
        "[Container]\nImage=example.invalid/application:1\nMemory=00016777216b\n"
    );

    let catalogue = CapabilityCatalogue::supported_range()?;
    for (target, expected) in [
        (PodmanVersion::new(5, 4, 2), SupportClassification::Unknown),
        (PodmanVersion::new(5, 5, 0), SupportClassification::Native),
        (PodmanVersion::new(6, 0, 2), SupportClassification::Native),
    ] {
        let target = PodmanTarget::new(target, Some(target))?;
        assert_eq!(
            catalogue.evaluate("quadlet.container.memory", target).classification(),
            expected
        );
    }
    Ok(())
}

#[test]
fn container_drop_capability_uses_repeatable_raw_public_values() -> Result<(), Box<dyn std::error::Error>> {
    let mut generated = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    generated.push_container(ContainerKey::Image, EntryValue::new("example.invalid/application:1")?)?;
    generated.push_container(ContainerKey::DropCapability, EntryValue::new("CAP_NET_ADMIN")?)?;
    generated.push_container(ContainerKey::DropCapability, EntryValue::new("ALL")?)?;
    generated.push_container(
        ContainerKey::DropCapability,
        EntryValue::new("CAP_DAC_OVERRIDE CAP_IPC_OWNER")?,
    )?;
    let generated = generated.build(SourceId::new(13))?;
    assert_eq!(
        generated.text(),
        concat!(
            "[Container]\n",
            "Image=example.invalid/application:1\n",
            "DropCapability=CAP_NET_ADMIN\n",
            "DropCapability=ALL\n",
            "DropCapability=CAP_DAC_OVERRIDE CAP_IPC_OWNER\n",
        )
    );
    assert_eq!(
        generated
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::DropCapability))
            .map(|entry| entry.value().primary().text())
            .collect::<Vec<_>>(),
        ["CAP_NET_ADMIN", "ALL", "CAP_DAC_OVERRIDE CAP_IPC_OWNER"]
    );

    let catalogue = CapabilityCatalogue::supported_range()?;
    let target = PodmanTarget::new(PodmanVersion::new(5, 4, 0), Some(PodmanVersion::new(6, 0, 2)))?;
    assert_eq!(
        catalogue
            .evaluate("quadlet.container.drop-capability", target)
            .classification(),
        SupportClassification::Native
    );
    Ok(())
}

#[test]
fn container_add_capability_uses_repeatable_raw_public_values() -> Result<(), Box<dyn std::error::Error>> {
    let mut generated = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    generated.push_container(ContainerKey::Image, EntryValue::new("example.invalid/application:1")?)?;
    for authored in [
        "CAP_NET_ADMIN",
        "",
        "CAP_NET_ADMIN",
        "ALL",
        "CAP_DAC_OVERRIDE CAP_IPC_OWNER",
    ] {
        generated.push_container(ContainerKey::AddCapability, EntryValue::new(authored)?)?;
    }
    let generated = generated.build(SourceId::new(14))?;
    assert_eq!(
        generated
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::AddCapability))
            .map(|entry| entry.value().primary().text())
            .collect::<Vec<_>>(),
        [
            "CAP_NET_ADMIN",
            "",
            "CAP_NET_ADMIN",
            "ALL",
            "CAP_DAC_OVERRIDE CAP_IPC_OWNER",
        ]
    );

    let catalogue = CapabilityCatalogue::supported_range()?;
    let target = PodmanTarget::new(PodmanVersion::new(5, 4, 0), Some(PodmanVersion::new(6, 0, 2)))?;
    assert_eq!(
        catalogue
            .evaluate("quadlet.container.add-capability", target)
            .classification(),
        SupportClassification::Native
    );
    Ok(())
}

#[test]
fn container_tmpfs_uses_repeatable_raw_public_values() -> Result<(), Box<dyn std::error::Error>> {
    let mut generated = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    generated.push_container(ContainerKey::Image, EntryValue::new("example.invalid/application:1")?)?;
    for authored in [
        "/cache:RW,NoExec",
        "",
        "/data:mode=755,uid=1009,gid=1009",
        "/data:mode=755,uid=1009,gid=1009",
    ] {
        generated.push_container(ContainerKey::Tmpfs, EntryValue::new(authored)?)?;
    }
    let generated = generated.build(SourceId::new(15))?;
    assert_eq!(
        generated
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::Tmpfs))
            .map(|entry| entry.value().primary().text())
            .collect::<Vec<_>>(),
        [
            "/cache:RW,NoExec",
            "",
            "/data:mode=755,uid=1009,gid=1009",
            "/data:mode=755,uid=1009,gid=1009",
        ]
    );

    let catalogue = CapabilityCatalogue::supported_range()?;
    let target = PodmanTarget::new(PodmanVersion::new(5, 4, 0), Some(PodmanVersion::new(6, 0, 2)))?;
    assert_eq!(
        catalogue.evaluate("quadlet.container.tmpfs", target).classification(),
        SupportClassification::Native
    );
    Ok(())
}

#[test]
fn container_sysctl_uses_repeatable_raw_public_values() -> Result<(), Box<dyn std::error::Error>> {
    let mut generated = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    generated.push_container(ContainerKey::Image, EntryValue::new("example.invalid/application:1")?)?;
    for authored in [
        "net.ipv4.conf.all.rp_filter=2 net.ipv4.ip_forward=0",
        r#"kernel.domainname="Authored Value""#,
        "net.ipv4.conf.%i.forwarding=%n",
        "",
        "net.ipv4.ip_forward=1",
        "net.ipv4.ip_forward=1",
    ] {
        generated.push_container(ContainerKey::Sysctl, EntryValue::new(authored)?)?;
    }
    let generated = generated.build(SourceId::new(16))?;
    assert_eq!(
        generated
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::Sysctl))
            .map(|entry| entry.value().primary().text())
            .collect::<Vec<_>>(),
        [
            "net.ipv4.conf.all.rp_filter=2 net.ipv4.ip_forward=0",
            r#"kernel.domainname="Authored Value""#,
            "net.ipv4.conf.%i.forwarding=%n",
            "",
            "net.ipv4.ip_forward=1",
            "net.ipv4.ip_forward=1",
        ]
    );

    let catalogue = CapabilityCatalogue::supported_range()?;
    let target = PodmanTarget::new(PodmanVersion::new(5, 4, 0), Some(PodmanVersion::new(6, 0, 2)))?;
    assert_eq!(
        catalogue.evaluate("quadlet.container.sysctl", target).classification(),
        SupportClassification::Native
    );
    Ok(())
}

#[test]
fn container_ulimit_uses_repeatable_raw_public_values() -> Result<(), Box<dyn std::error::Error>> {
    let mut generated = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    generated.push_container(ContainerKey::Image, EntryValue::new("example.invalid/application:1")?)?;
    for authored in [
        "Core=0:0",
        r#"nofile="1024:2048""#,
        "stack=%h:%n",
        "",
        "nproc=4096:8192",
        "nproc=4096:8192",
    ] {
        generated.push_container(ContainerKey::Ulimit, EntryValue::new(authored)?)?;
    }
    let generated = generated.build(SourceId::new(17))?;
    assert_eq!(
        generated
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

    let catalogue = CapabilityCatalogue::supported_range()?;
    let target = PodmanTarget::new(PodmanVersion::new(5, 4, 0), Some(PodmanVersion::new(6, 0, 2)))?;
    assert_eq!(
        catalogue.evaluate("quadlet.container.ulimit", target).classification(),
        SupportClassification::Native
    );
    Ok(())
}

#[test]
fn container_add_device_uses_repeatable_raw_public_values() -> Result<(), Box<dyn std::error::Error>> {
    let mut generated = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    generated.push_container(ContainerKey::Image, EntryValue::new("example.invalid/application:1")?)?;
    for authored in [
        "/dev/null:/dev/pre-null:r /dev/zero:/dev/pre-zero:w",
        "",
        r#""/dev/null:/dev/final null:r" %h/device:r"#,
        "-/dev/optional:/dev/optional:r",
        r#""/dev/null:/dev/final null:r" %h/device:r"#,
    ] {
        generated.push_container(ContainerKey::AddDevice, EntryValue::new(authored)?)?;
    }
    let generated = generated.build(SourceId::new(18))?;
    assert_eq!(
        generated
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::AddDevice))
            .map(|entry| entry.value().primary().text())
            .collect::<Vec<_>>(),
        [
            "/dev/null:/dev/pre-null:r /dev/zero:/dev/pre-zero:w",
            "",
            r#""/dev/null:/dev/final null:r" %h/device:r"#,
            "-/dev/optional:/dev/optional:r",
            r#""/dev/null:/dev/final null:r" %h/device:r"#,
        ]
    );

    let catalogue = CapabilityCatalogue::supported_range()?;
    let target = PodmanTarget::new(PodmanVersion::new(5, 4, 0), Some(PodmanVersion::new(6, 0, 2)))?;
    assert_eq!(
        catalogue
            .evaluate("quadlet.container.add-device", target)
            .classification(),
        SupportClassification::Native
    );
    Ok(())
}

#[test]
fn container_logging_uses_singleton_driver_and_repeatable_raw_options() -> Result<(), Box<dyn std::error::Error>> {
    let mut generated = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    generated.push_container(ContainerKey::Image, EntryValue::new("example.invalid/application:1")?)?;
    generated.push_container(ContainerKey::LogDriver, EntryValue::new(r#""Vendor-%n Driver""#)?)?;
    for authored in [
        "path=/var/log/pre.log",
        "",
        "tag=final-%n",
        r#""path=/var/log/Authored Value.log""#,
        "tag=final-%n",
    ] {
        generated.push_container(ContainerKey::LogOpt, EntryValue::new(authored)?)?;
    }
    let generated = generated.build(SourceId::new(30))?;
    assert_eq!(
        generated
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::LogOpt))
            .map(|entry| entry.value().primary().text())
            .collect::<Vec<_>>(),
        [
            "path=/var/log/pre.log",
            "",
            "tag=final-%n",
            r#""path=/var/log/Authored Value.log""#,
            "tag=final-%n",
        ]
    );

    let catalogue = CapabilityCatalogue::supported_range()?;
    let target = PodmanTarget::new(PodmanVersion::new(5, 4, 0), Some(PodmanVersion::new(6, 0, 2)))?;
    for capability in ["quadlet.container.log-driver", "quadlet.container.log-opt"] {
        assert_eq!(
            catalogue.evaluate(capability, target).classification(),
            SupportClassification::Native
        );
    }
    Ok(())
}

#[test]
fn container_network_identity_uses_singletons_and_repeatable_raw_aliases() -> Result<(), Box<dyn std::error::Error>> {
    let mut generated = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    generated.push_container(ContainerKey::Image, EntryValue::new("example.invalid/application:1")?)?;
    generated.push_container(ContainerKey::IP, EntryValue::new("192.0.2.%n")?)?;
    generated.push_container(ContainerKey::IP6, EntryValue::new("2001:db8::%i")?)?;
    for authored in ["pre.example", "", r#""final %n""#, "final-%i"] {
        generated.push_container(ContainerKey::NetworkAlias, EntryValue::new(authored)?)?;
    }
    let generated = generated.build(SourceId::new(31))?;
    assert_eq!(
        generated
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::NetworkAlias))
            .map(|entry| entry.value().primary().text())
            .collect::<Vec<_>>(),
        ["pre.example", "", r#""final %n""#, "final-%i"]
    );

    let catalogue = CapabilityCatalogue::supported_range()?;
    let target = PodmanTarget::new(PodmanVersion::new(5, 4, 0), Some(PodmanVersion::new(6, 0, 2)))?;
    for capability in [
        "quadlet.container.ip",
        "quadlet.container.ip6",
        "quadlet.container.network-alias",
    ] {
        assert_eq!(
            catalogue.evaluate(capability, target).classification(),
            SupportClassification::Native
        );
    }
    Ok(())
}

#[test]
fn network_driver_and_options_use_public_singleton_and_repeatable_keys() -> Result<(), Box<dyn std::error::Error>> {
    let mut generated = QuadletDocumentBuilder::new(QuadletUnitType::Network);
    generated.push_network(NetworkKey::NetworkName, EntryValue::new("frontend")?)?;
    generated.push_network(NetworkKey::Driver, EntryValue::new("bridge")?)?;
    for authored in ["pre=one", "", "zeta=last", "alpha=first", "alpha=final", "bare-token"] {
        generated.push_network(NetworkKey::Options, EntryValue::new(authored)?)?;
    }
    let generated = generated.build(SourceId::new(32))?;
    assert_eq!(
        generated
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Network(NetworkKey::Options))
            .map(|entry| entry.value().primary().text())
            .collect::<Vec<_>>(),
        ["pre=one", "", "zeta=last", "alpha=first", "alpha=final", "bare-token"]
    );

    let catalogue = CapabilityCatalogue::supported_range()?;
    let target = PodmanTarget::new(PodmanVersion::new(5, 4, 0), Some(PodmanVersion::new(6, 0, 2)))?;
    for capability in ["quadlet.network.driver", "quadlet.network.options"] {
        assert_eq!(
            catalogue.evaluate(capability, target).classification(),
            SupportClassification::Native
        );
    }
    Ok(())
}

#[test]
fn volume_driver_options_device_type_and_copy_use_public_singleton_keys() -> Result<(), Box<dyn std::error::Error>> {
    let mut generated = QuadletDocumentBuilder::new(QuadletUnitType::Volume);
    generated.push_volume(VolumeKey::VolumeName, EntryValue::new("cache")?)?;
    generated.push_volume(VolumeKey::Driver, EntryValue::new("local")?)?;
    generated.push_volume(VolumeKey::Options, EntryValue::new("mode=1777")?)?;
    generated.push_volume(VolumeKey::Device, EntryValue::new("tmpfs")?)?;
    generated.push_volume(VolumeKey::Type, EntryValue::new("bind")?)?;
    generated.push_volume(VolumeKey::Copy, EntryValue::new("TrUe")?)?;
    let generated = generated.build(SourceId::new(33))?;
    assert_eq!(
        generated.text(),
        "[Volume]\nVolumeName=cache\nDriver=local\nOptions=mode=1777\nDevice=tmpfs\nType=bind\nCopy=TrUe\n"
    );
    assert!(generated.document().entries().any(|entry| {
        entry.kind() == EntryKind::Volume(VolumeKey::Options) && entry.value().primary().text() == "mode=1777"
    }));
    let catalogue = CapabilityCatalogue::supported_range()?;
    let target = PodmanTarget::new(PodmanVersion::new(5, 4, 0), Some(PodmanVersion::new(6, 0, 2)))?;
    for capability in [
        "quadlet.volume.driver",
        "quadlet.volume.options",
        "quadlet.volume.device",
        "quadlet.volume.type",
        "quadlet.volume.copy",
    ] {
        assert_eq!(
            catalogue.evaluate(capability, target).classification(),
            SupportClassification::Native
        );
    }
    Ok(())
}

#[test]
fn volume_labels_use_the_public_repeatable_opaque_key() -> Result<(), Box<dyn std::error::Error>> {
    let mut generated = QuadletDocumentBuilder::new(QuadletUnitType::Volume);
    for authored in [
        "pre=discard",
        "",
        "alpha=first",
        "alpha=final",
        "empty=",
        "embedded=a=b",
        "bare-token",
        r#""quoted=%h value""#,
    ] {
        generated.push_volume(VolumeKey::Label, EntryValue::new(authored)?)?;
    }
    let generated = generated.build(SourceId::new(36))?;
    assert_eq!(
        generated
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Volume(VolumeKey::Label))
            .map(|entry| entry.value().primary().text())
            .collect::<Vec<_>>(),
        [
            "pre=discard",
            "",
            "alpha=first",
            "alpha=final",
            "empty=",
            "embedded=a=b",
            "bare-token",
            r#""quoted=%h value""#,
        ]
    );

    let catalogue = CapabilityCatalogue::supported_range()?;
    let target = PodmanTarget::new(PodmanVersion::new(5, 4, 0), Some(PodmanVersion::new(6, 0, 2)))?;
    assert_eq!(
        catalogue.evaluate("quadlet.volume.label", target).classification(),
        SupportClassification::Native
    );
    Ok(())
}

#[test]
fn network_labels_use_the_public_repeatable_opaque_key() -> Result<(), Box<dyn std::error::Error>> {
    let mut generated = QuadletDocumentBuilder::new(QuadletUnitType::Network);
    for authored in [
        "pre=discard",
        "",
        "alpha=first",
        "alpha=final",
        "empty=",
        "embedded=a=b",
        "bare-token",
        r#""quoted=%h value""#,
    ] {
        generated.push_network(NetworkKey::Label, EntryValue::new(authored)?)?;
    }
    let generated = generated.build(SourceId::new(35))?;
    assert_eq!(
        generated
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Network(NetworkKey::Label))
            .map(|entry| entry.value().primary().text())
            .collect::<Vec<_>>(),
        [
            "pre=discard",
            "",
            "alpha=first",
            "alpha=final",
            "empty=",
            "embedded=a=b",
            "bare-token",
            r#""quoted=%h value""#,
        ]
    );

    let catalogue = CapabilityCatalogue::supported_range()?;
    let target = PodmanTarget::new(PodmanVersion::new(5, 4, 0), Some(PodmanVersion::new(6, 0, 2)))?;
    assert_eq!(
        catalogue.evaluate("quadlet.network.label", target).classification(),
        SupportClassification::Native
    );
    Ok(())
}

#[test]
fn network_internal_and_ipv6_use_public_opaque_singleton_keys() -> Result<(), Box<dyn std::error::Error>> {
    let mut generated = QuadletDocumentBuilder::new(QuadletUnitType::Network);
    generated.push_network(NetworkKey::NetworkName, EntryValue::new("frontend")?)?;
    generated.push_network(NetworkKey::Internal, EntryValue::new("false")?)?;
    generated.push_network(NetworkKey::IPv6, EntryValue::new("vendor-defined-%n")?)?;
    let generated = generated.build(SourceId::new(33))?;
    assert_eq!(
        generated.text(),
        "[Network]\nNetworkName=frontend\nInternal=false\nIPv6=vendor-defined-%n\n"
    );

    let catalogue = CapabilityCatalogue::supported_range()?;
    let target = PodmanTarget::new(PodmanVersion::new(5, 4, 0), Some(PodmanVersion::new(6, 0, 2)))?;
    for capability in ["quadlet.network.internal", "quadlet.network.ipv6"] {
        assert_eq!(
            catalogue.evaluate(capability, target).classification(),
            SupportClassification::Native
        );
    }
    Ok(())
}

#[test]
fn network_ipam_keys_use_public_opaque_singleton_and_repeatable_forms() -> Result<(), Box<dyn std::error::Error>> {
    let mut generated = QuadletDocumentBuilder::new(QuadletUnitType::Network);
    generated.push_network(NetworkKey::IPAMDriver, EntryValue::new("")?)?;
    for (key, values) in [
        (NetworkKey::Subnet, ["pre-subnet", "", "10.88.0.0/24", "10.89.0.0/24"]),
        (NetworkKey::Gateway, ["pre-gateway", "", "10.88.0.1", "10.89.0.1"]),
        (NetworkKey::IPRange, ["pre-range", "", "10.88.0.64/26", "10.89.0.64/26"]),
    ] {
        for value in values {
            generated.push_network(key, EntryValue::new(value)?)?;
        }
    }
    let generated = generated.build(SourceId::new(34))?;
    for key in [NetworkKey::Subnet, NetworkKey::Gateway, NetworkKey::IPRange] {
        assert_eq!(
            generated
                .document()
                .entries()
                .filter(|entry| entry.kind() == EntryKind::Network(key))
                .map(|entry| entry.value().primary().text())
                .collect::<Vec<_>>(),
            match key {
                NetworkKey::Subnet => vec!["pre-subnet", "", "10.88.0.0/24", "10.89.0.0/24"],
                NetworkKey::Gateway => vec!["pre-gateway", "", "10.88.0.1", "10.89.0.1"],
                NetworkKey::IPRange => vec!["pre-range", "", "10.88.0.64/26", "10.89.0.64/26"],
                _ => unreachable!(),
            }
        );
    }
    let catalogue = CapabilityCatalogue::supported_range()?;
    let target = PodmanTarget::new(PodmanVersion::new(5, 4, 0), Some(PodmanVersion::new(6, 0, 2)))?;
    for capability in [
        "quadlet.network.ipam-driver",
        "quadlet.network.subnet",
        "quadlet.network.gateway",
        "quadlet.network.ip-range",
    ] {
        assert_eq!(
            catalogue.evaluate(capability, target).classification(),
            SupportClassification::Native
        );
    }
    Ok(())
}

#[test]
fn container_dns_uses_repeatable_raw_public_values() -> Result<(), Box<dyn std::error::Error>> {
    let mut generated = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    generated.push_container(ContainerKey::Image, EntryValue::new("example.invalid/application:1")?)?;
    for authored in [
        "1.1.1.1",
        "1.1.1.1",
        "",
        "9.9.9.9",
        "2001:4860:4860::8888",
        r#""Authored Resolver""#,
        "%h",
    ] {
        generated.push_container(ContainerKey::DNS, EntryValue::new(authored)?)?;
    }
    let generated = generated.build(SourceId::new(19))?;
    assert_eq!(
        generated
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::DNS))
            .map(|entry| entry.value().primary().text())
            .collect::<Vec<_>>(),
        [
            "1.1.1.1",
            "1.1.1.1",
            "",
            "9.9.9.9",
            "2001:4860:4860::8888",
            r#""Authored Resolver""#,
            "%h",
        ]
    );

    let catalogue = CapabilityCatalogue::supported_range()?;
    let target = PodmanTarget::new(PodmanVersion::new(5, 4, 0), Some(PodmanVersion::new(6, 0, 2)))?;
    assert_eq!(
        catalogue.evaluate("quadlet.container.dns", target).classification(),
        SupportClassification::Native
    );
    Ok(())
}

#[test]
fn container_dns_option_uses_repeatable_raw_public_values() -> Result<(), Box<dyn std::error::Error>> {
    let mut generated = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    generated.push_container(ContainerKey::Image, EntryValue::new("example.invalid/application:1")?)?;
    for authored in [
        "rotate",
        "rotate",
        "",
        "ndots:1",
        "use-vc",
        r#""Authored Option""#,
        "%h",
    ] {
        generated.push_container(ContainerKey::DNSOption, EntryValue::new(authored)?)?;
    }
    let generated = generated.build(SourceId::new(20))?;
    assert_eq!(
        generated
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::DNSOption))
            .map(|entry| entry.value().primary().text())
            .collect::<Vec<_>>(),
        [
            "rotate",
            "rotate",
            "",
            "ndots:1",
            "use-vc",
            r#""Authored Option""#,
            "%h"
        ]
    );

    let catalogue = CapabilityCatalogue::supported_range()?;
    let target = PodmanTarget::new(PodmanVersion::new(5, 4, 0), Some(PodmanVersion::new(6, 0, 2)))?;
    assert_eq!(
        catalogue
            .evaluate("quadlet.container.dns-option", target)
            .classification(),
        SupportClassification::Native
    );
    Ok(())
}

#[test]
fn container_dns_search_uses_repeatable_raw_public_values() -> Result<(), Box<dyn std::error::Error>> {
    let mut generated = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    generated.push_container(ContainerKey::Image, EntryValue::new("example.invalid/application:1")?)?;
    for authored in [
        "pre.example.com",
        "pre.example.com",
        "",
        "dc1.example.com",
        ".",
        r#""Authored Search""#,
        "%h",
    ] {
        generated.push_container(ContainerKey::DNSSearch, EntryValue::new(authored)?)?;
    }
    let generated = generated.build(SourceId::new(21))?;
    assert_eq!(
        generated
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::DNSSearch))
            .map(|entry| entry.value().primary().text())
            .collect::<Vec<_>>(),
        [
            "pre.example.com",
            "pre.example.com",
            "",
            "dc1.example.com",
            ".",
            r#""Authored Search""#,
            "%h"
        ]
    );

    let catalogue = CapabilityCatalogue::supported_range()?;
    let target = PodmanTarget::new(PodmanVersion::new(5, 4, 0), Some(PodmanVersion::new(6, 0, 2)))?;
    assert_eq!(
        catalogue
            .evaluate("quadlet.container.dns-search", target)
            .classification(),
        SupportClassification::Native
    );
    Ok(())
}

#[test]
fn container_expose_host_port_uses_repeatable_raw_public_values() -> Result<(), Box<dyn std::error::Error>> {
    let mut generated = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    generated.push_container(ContainerKey::Image, EntryValue::new("example.invalid/application:1")?)?;
    for authored in [
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
    ] {
        generated.push_container(ContainerKey::ExposeHostPort, EntryValue::new(authored)?)?;
    }
    let generated = generated.build(SourceId::new(22))?;
    assert_eq!(
        generated
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
            "5353/sctp",
            r#""Authored Port""#,
            "%i",
            "not-a-port",
        ]
    );

    let catalogue = CapabilityCatalogue::supported_range()?;
    let target = PodmanTarget::new(PodmanVersion::new(5, 4, 0), Some(PodmanVersion::new(6, 0, 2)))?;
    assert_eq!(
        catalogue
            .evaluate("quadlet.container.expose-host-port", target)
            .classification(),
        SupportClassification::Native
    );
    Ok(())
}

#[test]
fn container_annotation_uses_repeatable_raw_public_values() -> Result<(), Box<dyn std::error::Error>> {
    let mut generated = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    generated.push_container(ContainerKey::Image, EntryValue::new("example.invalid/application:1")?)?;
    for authored in [
        "org.example.name=first",
        "org.example.name=first",
        "",
        "org.example.name=final",
        r#""org.example.quoted=Authored Value""#,
        "org.example.specifier=%i",
        "key-only",
        "malformed = value ",
    ] {
        generated.push_container(ContainerKey::Annotation, EntryValue::new(authored)?)?;
    }
    let generated = generated.build(SourceId::new(23))?;
    assert_eq!(
        generated
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
            "malformed = value ",
        ]
    );

    let catalogue = CapabilityCatalogue::supported_range()?;
    let target = PodmanTarget::new(PodmanVersion::new(5, 4, 0), Some(PodmanVersion::new(6, 0, 2)))?;
    assert_eq!(
        catalogue
            .evaluate("quadlet.container.annotation", target)
            .classification(),
        SupportClassification::Native
    );
    Ok(())
}

#[test]
fn container_apparmor_uses_a_singleton_raw_public_value() -> Result<(), Box<dyn std::error::Error>> {
    let mut generated = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    generated.push_container(ContainerKey::Image, EntryValue::new("example.invalid/application:1")?)?;
    generated.push_container(ContainerKey::AppArmor, EntryValue::new(r#""profile:with %i""#)?)?;
    assert!(matches!(
        generated.push_container(ContainerKey::AppArmor, EntryValue::new("unconfined")?),
        Err(quadlet_lens::render::RenderError::DuplicateSingleton(key)) if key == "AppArmor"
    ));
    let generated = generated.build(SourceId::new(24))?;
    assert!(generated.document().entries().any(|entry| {
        entry.kind() == EntryKind::Container(ContainerKey::AppArmor)
            && entry.value().primary().text() == r#""profile:with %i""#
    }));

    let catalogue = CapabilityCatalogue::supported_range()?;
    let target = PodmanTarget::new(PodmanVersion::new(5, 8, 0), Some(PodmanVersion::new(6, 0, 2)))?;
    assert_eq!(
        catalogue
            .evaluate("quadlet.container.apparmor", target)
            .classification(),
        SupportClassification::Native
    );
    Ok(())
}

#[test]
fn container_no_new_privileges_uses_a_singleton_raw_public_value() -> Result<(), Box<dyn std::error::Error>> {
    let mut generated = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    generated.push_container(ContainerKey::Image, EntryValue::new("example.invalid/application:1")?)?;
    generated.push_container(ContainerKey::NoNewPrivileges, EntryValue::new(r#""yes""#)?)?;
    assert!(matches!(
        generated.push_container(ContainerKey::NoNewPrivileges, EntryValue::new("false")?),
        Err(quadlet_lens::render::RenderError::DuplicateSingleton(key)) if key == "NoNewPrivileges"
    ));
    let generated = generated.build(SourceId::new(25))?;
    assert!(generated.document().entries().any(|entry| {
        entry.kind() == EntryKind::Container(ContainerKey::NoNewPrivileges)
            && entry.value().primary().text() == r#""yes""#
            && entry.value_kind() == ValueKind::Opaque
    }));

    let catalogue = CapabilityCatalogue::supported_range()?;
    let target = PodmanTarget::new(PodmanVersion::new(5, 4, 0), Some(PodmanVersion::new(6, 0, 2)))?;
    assert_eq!(
        catalogue
            .evaluate("quadlet.container.no-new-privileges", target)
            .classification(),
        SupportClassification::Native
    );
    Ok(())
}

#[test]
fn container_seccomp_profile_uses_a_singleton_raw_public_value() -> Result<(), Box<dyn std::error::Error>> {
    let mut generated = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    generated.push_container(ContainerKey::Image, EntryValue::new("example.invalid/application:1")?)?;
    generated.push_container(
        ContainerKey::SeccompProfile,
        EntryValue::new(r#""%h/profiles/profile.json""#)?,
    )?;
    assert!(matches!(
        generated.push_container(ContainerKey::SeccompProfile, EntryValue::new("unconfined")?),
        Err(quadlet_lens::render::RenderError::DuplicateSingleton(key)) if key == "SeccompProfile"
    ));
    let generated = generated.build(SourceId::new(26))?;
    assert!(generated.document().entries().any(|entry| {
        entry.kind() == EntryKind::Container(ContainerKey::SeccompProfile)
            && entry.value().primary().text() == r#""%h/profiles/profile.json""#
            && entry.value_kind() == ValueKind::Opaque
    }));

    let catalogue = CapabilityCatalogue::supported_range()?;
    let target = PodmanTarget::new(PodmanVersion::new(5, 4, 0), Some(PodmanVersion::new(6, 0, 2)))?;
    assert_eq!(
        catalogue
            .evaluate("quadlet.container.seccomp-profile", target)
            .classification(),
        SupportClassification::Native
    );
    Ok(())
}

#[test]
fn container_security_label_disable_uses_a_singleton_raw_public_value() -> Result<(), Box<dyn std::error::Error>> {
    let mut generated = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    generated.push_container(ContainerKey::Image, EntryValue::new("example.invalid/application:1")?)?;
    generated.push_container(ContainerKey::SecurityLabelDisable, EntryValue::new(r#""yes""#)?)?;
    assert!(matches!(
        generated.push_container(ContainerKey::SecurityLabelDisable, EntryValue::new("false")?),
        Err(quadlet_lens::render::RenderError::DuplicateSingleton(key)) if key == "SecurityLabelDisable"
    ));
    let generated = generated.build(SourceId::new(27))?;
    assert!(generated.document().entries().any(|entry| {
        entry.kind() == EntryKind::Container(ContainerKey::SecurityLabelDisable)
            && entry.value().primary().text() == r#""yes""#
            && entry.value_kind() == ValueKind::Opaque
    }));

    let catalogue = CapabilityCatalogue::supported_range()?;
    let target = PodmanTarget::new(PodmanVersion::new(5, 4, 0), Some(PodmanVersion::new(6, 0, 2)))?;
    assert_eq!(
        catalogue
            .evaluate("quadlet.container.security-label-disable", target)
            .classification(),
        SupportClassification::Native
    );
    Ok(())
}

#[test]
fn container_security_label_file_type_uses_a_singleton_raw_public_value() -> Result<(), Box<dyn std::error::Error>> {
    let mut generated = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    generated.push_container(ContainerKey::Image, EntryValue::new("example.invalid/application:1")?)?;
    generated.push_container(ContainerKey::SecurityLabelFileType, EntryValue::new(r#""%i_file_t""#)?)?;
    assert!(matches!(
        generated.push_container(ContainerKey::SecurityLabelFileType, EntryValue::new("container_file_t")?),
        Err(quadlet_lens::render::RenderError::DuplicateSingleton(key)) if key == "SecurityLabelFileType"
    ));
    let generated = generated.build(SourceId::new(28))?;
    assert!(generated.document().entries().any(|entry| {
        entry.kind() == EntryKind::Container(ContainerKey::SecurityLabelFileType)
            && entry.value().primary().text() == r#""%i_file_t""#
            && entry.value_kind() == ValueKind::Opaque
    }));

    let catalogue = CapabilityCatalogue::supported_range()?;
    let target = PodmanTarget::new(PodmanVersion::new(5, 4, 0), Some(PodmanVersion::new(6, 0, 2)))?;
    assert_eq!(
        catalogue
            .evaluate("quadlet.container.security-label-file-type", target)
            .classification(),
        SupportClassification::Native
    );
    Ok(())
}

#[test]
fn container_security_label_level_uses_a_singleton_raw_public_value() -> Result<(), Box<dyn std::error::Error>> {
    let mut generated = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    generated.push_container(ContainerKey::Image, EntryValue::new("example.invalid/application:1")?)?;
    generated.push_container(ContainerKey::SecurityLabelLevel, EntryValue::new(r#""%i:c1,c2""#)?)?;
    assert!(matches!(
        generated.push_container(ContainerKey::SecurityLabelLevel, EntryValue::new("s0:c1,c2")?),
        Err(quadlet_lens::render::RenderError::DuplicateSingleton(key)) if key == "SecurityLabelLevel"
    ));
    let generated = generated.build(SourceId::new(29))?;
    assert!(generated.document().entries().any(|entry| {
        entry.kind() == EntryKind::Container(ContainerKey::SecurityLabelLevel)
            && entry.value().primary().text() == r#""%i:c1,c2""#
            && entry.value_kind() == ValueKind::Opaque
    }));

    let catalogue = CapabilityCatalogue::supported_range()?;
    let target = PodmanTarget::new(PodmanVersion::new(5, 4, 0), Some(PodmanVersion::new(6, 0, 2)))?;
    assert_eq!(
        catalogue
            .evaluate("quadlet.container.security-label-level", target)
            .classification(),
        SupportClassification::Native
    );
    Ok(())
}

#[test]
fn container_security_label_nested_uses_a_singleton_raw_public_value() -> Result<(), Box<dyn std::error::Error>> {
    let mut generated = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    generated.push_container(ContainerKey::Image, EntryValue::new("example.invalid/application:1")?)?;
    generated.push_container(ContainerKey::SecurityLabelNested, EntryValue::new(r#""%i""#)?)?;
    assert!(matches!(
        generated.push_container(ContainerKey::SecurityLabelNested, EntryValue::new("true")?),
        Err(quadlet_lens::render::RenderError::DuplicateSingleton(key)) if key == "SecurityLabelNested"
    ));
    let generated = generated.build(SourceId::new(30))?;
    assert!(generated.document().entries().any(|entry| {
        entry.kind() == EntryKind::Container(ContainerKey::SecurityLabelNested)
            && entry.value().primary().text() == r#""%i""#
            && entry.value_kind() == ValueKind::Opaque
    }));

    let catalogue = CapabilityCatalogue::supported_range()?;
    let target = PodmanTarget::new(PodmanVersion::new(5, 4, 0), Some(PodmanVersion::new(6, 0, 2)))?;
    assert_eq!(
        catalogue
            .evaluate("quadlet.container.security-label-nested", target)
            .classification(),
        SupportClassification::Native
    );
    Ok(())
}

#[test]
fn container_security_label_type_uses_a_singleton_raw_public_value() -> Result<(), Box<dyn std::error::Error>> {
    let mut generated = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    generated.push_container(ContainerKey::Image, EntryValue::new("example.invalid/application:1")?)?;
    generated.push_container(ContainerKey::SecurityLabelType, EntryValue::new(r#""%i_t""#)?)?;
    assert!(matches!(
        generated.push_container(ContainerKey::SecurityLabelType, EntryValue::new("container_t")?),
        Err(quadlet_lens::render::RenderError::DuplicateSingleton(key)) if key == "SecurityLabelType"
    ));
    let generated = generated.build(SourceId::new(31))?;
    assert!(generated.document().entries().any(|entry| {
        entry.kind() == EntryKind::Container(ContainerKey::SecurityLabelType)
            && entry.value().primary().text() == r#""%i_t""#
            && entry.value_kind() == ValueKind::Opaque
    }));

    let catalogue = CapabilityCatalogue::supported_range()?;
    let target = PodmanTarget::new(PodmanVersion::new(5, 4, 0), Some(PodmanVersion::new(6, 0, 2)))?;
    assert_eq!(
        catalogue
            .evaluate("quadlet.container.security-label-type", target)
            .classification(),
        SupportClassification::Native
    );
    Ok(())
}

#[test]
fn container_mask_uses_repeatable_raw_public_values() -> Result<(), Box<dyn std::error::Error>> {
    let mut generated = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    generated.push_container(ContainerKey::Image, EntryValue::new("example.invalid/application:1")?)?;
    for authored in ["/pre/one", "", r#""%h/private:%t/shared""#, "/proc/acpi:/sys/firmware"] {
        generated.push_container(ContainerKey::Mask, EntryValue::new(authored)?)?;
    }
    let generated = generated.build(SourceId::new(32))?;
    assert_eq!(
        generated
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::Mask))
            .map(|entry| {
                assert_eq!(entry.value_kind(), ValueKind::Opaque);
                entry.value().primary().text()
            })
            .collect::<Vec<_>>(),
        ["/pre/one", "", r#""%h/private:%t/shared""#, "/proc/acpi:/sys/firmware"]
    );

    let catalogue = CapabilityCatalogue::supported_range()?;
    let target = PodmanTarget::new(PodmanVersion::new(5, 4, 0), Some(PodmanVersion::new(6, 0, 2)))?;
    assert_eq!(
        catalogue.evaluate("quadlet.container.mask", target).classification(),
        SupportClassification::Native
    );
    Ok(())
}

#[test]
fn container_unmask_uses_repeatable_raw_public_values() -> Result<(), Box<dyn std::error::Error>> {
    let authored = [
        "/pre/one",
        "/pre/one",
        "",
        "ALL",
        r#""%h/private:/proc/*""#,
        "/proc/acpi:/sys/firmware",
    ];
    let mut generated = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    generated.push_container(ContainerKey::Image, EntryValue::new("example.invalid/application:1")?)?;
    for value in authored {
        generated.push_container(ContainerKey::Unmask, EntryValue::new(value)?)?;
    }
    let generated = generated.build(SourceId::new(33))?;
    assert_eq!(
        generated
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

    let catalogue = CapabilityCatalogue::supported_range()?;
    let target = PodmanTarget::new(PodmanVersion::new(5, 4, 0), Some(PodmanVersion::new(6, 0, 2)))?;
    assert_eq!(
        catalogue.evaluate("quadlet.container.unmask", target).classification(),
        SupportClassification::Native
    );
    Ok(())
}

#[test]
fn supported_public_pipeline_compiles_and_keeps_stages_explicit() -> Result<(), Box<dyn std::error::Error>> {
    let source = "[Container]\nImage=example.invalid/app:1@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n";
    let parsed = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(1), source)?;

    assert!(parsed.is_valid());
    assert_eq!(parsed.syntax().document().render_preserved(), source);
    assert_eq!(parsed.syntax().render_canonical()?, source);

    let named = NamedQuadletDocument::new("app.container", parsed.document().clone())?;
    let documents = QuadletDocumentSet::new([named])?;
    assert!(documents.is_valid());
    assert!(documents.graph().is_complete());

    let catalogue = CapabilityCatalogue::supported_range()?;
    let target = PodmanTarget::new(PodmanVersion::new(5, 4, 0), Some(PodmanVersion::new(6, 0, 2)))?;
    for capability in [
        "quadlet.container.image",
        "quadlet.container.rootfs",
        "quadlet.container.container-name",
        "quadlet.container.entrypoint",
        "quadlet.container.run-init",
        "quadlet.container.pull",
        "quadlet.container.add-host",
        "quadlet.container.health-timeout",
        "quadlet.container.notify-healthy",
        "quadlet.container.user",
        "quadlet.container.group",
        "quadlet.container.userns",
        "quadlet.container.group-add",
        "quadlet.container.working-dir",
        "quadlet.container.read-only",
        "quadlet.container.secret",
        "quadlet.container.label",
        "quadlet.pod.userns",
    ] {
        assert_eq!(
            catalogue.evaluate(capability, target).classification(),
            SupportClassification::Native
        );
    }
    assert_eq!(classify_path("%h/application.env"), PathForm::SystemdSpecifier);

    let mut generated = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    generated.push_systemd_unit(SystemdUnitKey::Requires, EntryValue::new("database.service")?)?;
    generated.push_systemd_unit(SystemdUnitKey::After, EntryValue::new("database.service")?)?;
    generated.push_container(
        ContainerKey::AddHost,
        EntryValue::new("host.docker.internal:host-gateway")?,
    )?;
    generated.push_container(ContainerKey::Image, EntryValue::new("example.invalid/generated:1")?)?;
    generated.push_container(ContainerKey::Entrypoint, EntryValue::new(r#"["/usr/bin/env","php"]"#)?)?;
    generated.push_container(ContainerKey::RunInit, EntryValue::new("true")?)?;
    generated.push_container(
        ContainerKey::Label,
        EntryValue::new("org.example.application=generated")?,
    )?;
    generated.push_container(ContainerKey::Label, EntryValue::new("org.example.stage=test")?)?;
    generated.push_container(
        ContainerKey::Secret,
        EntryValue::new("database-password,target=password,mode=0440")?,
    )?;
    generated.push_container(ContainerKey::User, EntryValue::new("1001")?)?;
    generated.push_container(ContainerKey::Group, EntryValue::new("1002")?)?;
    generated.push_container(ContainerKey::UserNS, EntryValue::new("keep-id")?)?;
    generated.push_container(ContainerKey::GroupAdd, EntryValue::new("audio")?)?;
    generated.push_container(ContainerKey::WorkingDir, EntryValue::new("/srv/app")?)?;
    generated.push_container(ContainerKey::ReadOnly, EntryValue::new("true")?)?;
    generated.push_container(ContainerKey::HealthCmd, EntryValue::new("/usr/bin/true")?)?;
    generated.push_container(ContainerKey::Notify, EntryValue::new("healthy")?)?;
    generated.push_container(ContainerKey::HealthTimeout, EntryValue::new("5s")?)?;
    let generated = generated.build(SourceId::new(2))?;
    assert_eq!(
        generated.text(),
        concat!(
            "[Unit]\n",
            "Requires=database.service\n",
            "After=database.service\n",
            "\n",
            "[Container]\n",
            "AddHost=host.docker.internal:host-gateway\n",
            "Image=example.invalid/generated:1\n",
            "Entrypoint=[\"/usr/bin/env\",\"php\"]\n",
            "RunInit=true\n",
            "Label=org.example.application=generated\n",
            "Label=org.example.stage=test\n",
            "Secret=database-password,target=password,mode=0440\n",
            "User=1001\n",
            "Group=1002\n",
            "UserNS=keep-id\n",
            "GroupAdd=audio\n",
            "WorkingDir=/srv/app\n",
            "ReadOnly=true\n",
            "HealthCmd=/usr/bin/true\n",
            "Notify=healthy\n",
            "HealthTimeout=5s\n",
        )
    );

    let mut generated_pod = QuadletDocumentBuilder::new(QuadletUnitType::Pod);
    generated_pod.push_pod(PodKey::PodName, EntryValue::new("generated-pod")?)?;
    generated_pod.push_pod(PodKey::UserNS, EntryValue::new("auto:size=8192")?)?;
    assert_eq!(
        generated_pod.build(SourceId::new(3))?.text(),
        "[Pod]\nPodName=generated-pod\nUserNS=auto:size=8192\n"
    );

    Ok(())
}
