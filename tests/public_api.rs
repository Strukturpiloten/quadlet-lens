//! Consumer-facing compile and behavior contract for the supported 0.1.x API.

use quadlet_lens::capability::{CapabilityCatalogue, PodmanTarget, PodmanVersion, SupportClassification};
use quadlet_lens::model::{
    ContainerKey, EntryKind, NamedQuadletDocument, PodKey, QuadletDocument, QuadletDocumentSet, QuadletUnitType,
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
        ],
        [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28,
            29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40,
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
