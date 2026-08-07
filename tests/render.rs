//! Programmatic Quadlet construction and parse-back validation.

use quadlet_lens::{
    model::{
        ContainerKey, EntryKind, NamedQuadletDocument, NetworkKey, PodKey, QuadletDocumentSet, QuadletUnitType,
        VolumeKey,
    },
    render::{
        EntryValue, Memory, MemoryError, PidsLimit, PidsLimitError, QuadletDocumentBuilder, RenderError, ShmSize,
        ShmSizeError, SystemdSection, SystemdUnitKey,
    },
    source::SourceId,
};

#[test]
fn builds_a_deterministic_first_conversion_document_set() -> Result<(), Box<dyn std::error::Error>> {
    let mut network = QuadletDocumentBuilder::new(QuadletUnitType::Network);
    network.push_network(NetworkKey::NetworkName, value("example-network")?)?;
    let network = network.build(SourceId::new(81))?;
    assert_eq!(network.text(), "[Network]\nNetworkName=example-network\n");

    let mut volume = QuadletDocumentBuilder::new(QuadletUnitType::Volume);
    volume.push_volume(VolumeKey::VolumeName, value("example-data")?)?;
    let volume = volume.build(SourceId::new(82))?;
    assert_eq!(volume.text(), "[Volume]\nVolumeName=example-data\n");

    let mut container = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    container.push_systemd(SystemdSection::Unit, "Description", value("Generated application")?)?;
    container.push_systemd_unit(SystemdUnitKey::Requires, value("database.service")?)?;
    container.push_systemd_unit(SystemdUnitKey::After, value("database.service")?)?;
    container.push_container(ContainerKey::AddHost, value("host.docker.internal:host-gateway")?)?;
    container.push_container(ContainerKey::Image, value("example.invalid/app:1@sha256:abcd")?)?;
    container.push_container(ContainerKey::Entrypoint, value(r#"["/usr/bin/env","php"]"#)?)?;
    container.push_container(ContainerKey::RunInit, value("true")?)?;
    container.push_container(ContainerKey::StopSignal, value("SIGUSR1")?)?;
    container.push_container(ContainerKey::StopTimeout, value("37")?)?;
    container.push_container(ContainerKey::Pull, value("newer")?)?;
    container.push_container(ContainerKey::PidsLimit, PidsLimit::finite("127")?.into())?;
    container.push_container(ContainerKey::Exec, value("php -v")?)?;
    container.push_container(ContainerKey::Environment, value("APP_ENV=production")?)?;
    container.push_container(ContainerKey::Label, value("org.example.application=example")?)?;
    container.push_container(ContainerKey::Label, value("org.example.environment=production")?)?;
    container.push_container(
        ContainerKey::Secret,
        value("database-password,target=password,uid=1001,gid=1002,mode=0440")?,
    )?;
    container.push_container(ContainerKey::Secret, value("api-token,type=env,target=API_TOKEN")?)?;
    container.push_container(ContainerKey::User, value("1001")?)?;
    container.push_container(ContainerKey::Group, value("1002")?)?;
    container.push_container(ContainerKey::UserNS, value("keep-id")?)?;
    container.push_container(ContainerKey::GroupAdd, value("audio")?)?;
    container.push_container(ContainerKey::GroupAdd, value("44")?)?;
    container.push_container(ContainerKey::WorkingDir, value("/srv/app")?)?;
    container.push_container(ContainerKey::ReadOnly, value("true")?)?;
    container.push_container(ContainerKey::HealthCmd, value("/usr/bin/true")?)?;
    container.push_container(ContainerKey::Notify, value("healthy")?)?;
    container.push_container(ContainerKey::HealthInterval, value("30s")?)?;
    container.push_container(ContainerKey::HealthRetries, value("3")?)?;
    container.push_container(ContainerKey::HealthStartPeriod, value("10s")?)?;
    container.push_container(ContainerKey::HealthTimeout, value("5s")?)?;
    container.push_container(ContainerKey::PublishPort, value("127.0.0.1:8080:80/tcp")?)?;
    container.push_container(ContainerKey::Volume, value("data.volume:/srv/data:ro,Z")?)?;
    container.push_container(ContainerKey::Network, value("frontend.network")?)?;
    container.push_systemd(SystemdSection::Service, "Restart", value("on-failure")?)?;
    let container = container.build(SourceId::new(83))?;

    assert_eq!(
        container.text(),
        concat!(
            "[Unit]\n",
            "Description=Generated application\n",
            "Requires=database.service\n",
            "After=database.service\n",
            "\n",
            "[Container]\n",
            "AddHost=host.docker.internal:host-gateway\n",
            "Image=example.invalid/app:1@sha256:abcd\n",
            "Entrypoint=[\"/usr/bin/env\",\"php\"]\n",
            "RunInit=true\n",
            "StopSignal=SIGUSR1\n",
            "StopTimeout=37\n",
            "Pull=newer\n",
            "PidsLimit=127\n",
            "Exec=php -v\n",
            "Environment=APP_ENV=production\n",
            "Label=org.example.application=example\n",
            "Label=org.example.environment=production\n",
            "Secret=database-password,target=password,uid=1001,gid=1002,mode=0440\n",
            "Secret=api-token,type=env,target=API_TOKEN\n",
            "User=1001\n",
            "Group=1002\n",
            "UserNS=keep-id\n",
            "GroupAdd=audio\n",
            "GroupAdd=44\n",
            "WorkingDir=/srv/app\n",
            "ReadOnly=true\n",
            "HealthCmd=/usr/bin/true\n",
            "Notify=healthy\n",
            "HealthInterval=30s\n",
            "HealthRetries=3\n",
            "HealthStartPeriod=10s\n",
            "HealthTimeout=5s\n",
            "PublishPort=127.0.0.1:8080:80/tcp\n",
            "Volume=data.volume:/srv/data:ro,Z\n",
            "Network=frontend.network\n",
            "\n",
            "[Service]\n",
            "Restart=on-failure\n",
        )
    );

    let documents = QuadletDocumentSet::new([
        NamedQuadletDocument::new("frontend.network", network.document().clone())?,
        NamedQuadletDocument::new("data.volume", volume.document().clone())?,
        NamedQuadletDocument::new("app.container", container.document().clone())?,
    ])?;
    assert!(documents.is_valid(), "{:#?}", documents.diagnostics());
    assert!(documents.graph().is_complete());
    assert_eq!(documents.graph().edges().len(), 2);
    Ok(())
}

#[test]
fn preserves_repeated_native_and_generic_entries_in_order() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    builder.push_systemd(SystemdSection::Unit, "After", value("network-online.target")?)?;
    builder.push_systemd(SystemdSection::Unit, "After", value("database.container")?)?;
    builder.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
    builder.push_container(ContainerKey::AddHost, value("first:192.0.2.10")?)?;
    builder.push_container(ContainerKey::AddHost, value("second:[::1]")?)?;
    builder.push_container(ContainerKey::Environment, value("FIRST=1")?)?;
    builder.push_container(ContainerKey::Environment, value("SECOND=2")?)?;
    builder.push_container(ContainerKey::Label, value("org.example.first=1")?)?;
    builder.push_container(ContainerKey::Label, value("org.example.second=2")?)?;
    builder.push_container(ContainerKey::Label, value("org.example.empty=")?)?;
    builder.push_container(
        ContainerKey::Label,
        value(r#""org.example.metadata={\"channel\": \"stable\"}""#)?,
    )?;
    builder.push_container(ContainerKey::Secret, value("first-secret")?)?;
    builder.push_container(
        ContainerKey::Secret,
        value("second-secret,type=env,target=SECOND_SECRET")?,
    )?;
    builder.push_container(ContainerKey::GroupAdd, value("audio")?)?;
    builder.push_container(ContainerKey::GroupAdd, value("44")?)?;
    builder.push_container(ContainerKey::DropCapability, value("CAP_NET_ADMIN")?)?;
    builder.push_container(ContainerKey::DropCapability, value("ALL")?)?;
    builder.push_container(ContainerKey::DropCapability, value("CAP_DAC_OVERRIDE CAP_IPC_OWNER")?)?;
    builder.push_container(ContainerKey::AddCapability, value("CAP_NET_ADMIN")?)?;
    builder.push_container(ContainerKey::AddCapability, value("ALL")?)?;
    builder.push_container(ContainerKey::AddCapability, value("CAP_DAC_OVERRIDE CAP_IPC_OWNER")?)?;
    let generated = builder.build(SourceId::new(84))?;

    assert_eq!(
        generated.text(),
        concat!(
            "[Unit]\n",
            "After=network-online.target\n",
            "After=database.container\n",
            "\n",
            "[Container]\n",
            "Image=example.invalid/app\n",
            "AddHost=first:192.0.2.10\n",
            "AddHost=second:[::1]\n",
            "Environment=FIRST=1\n",
            "Environment=SECOND=2\n",
            "Label=org.example.first=1\n",
            "Label=org.example.second=2\n",
            "Label=org.example.empty=\n",
            "Label=\"org.example.metadata={\\\"channel\\\": \\\"stable\\\"}\"\n",
            "Secret=first-secret\n",
            "Secret=second-secret,type=env,target=SECOND_SECRET\n",
            "GroupAdd=audio\n",
            "GroupAdd=44\n",
            "DropCapability=CAP_NET_ADMIN\n",
            "DropCapability=ALL\n",
            "DropCapability=CAP_DAC_OVERRIDE CAP_IPC_OWNER\n",
            "AddCapability=CAP_NET_ADMIN\n",
            "AddCapability=ALL\n",
            "AddCapability=CAP_DAC_OVERRIDE CAP_IPC_OWNER\n",
        )
    );
    Ok(())
}

#[test]
fn builds_a_singleton_pod_user_namespace() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = QuadletDocumentBuilder::new(QuadletUnitType::Pod);
    builder.push_pod(PodKey::PodName, value("example-pod")?)?;
    builder.push_pod(PodKey::UserNS, value("auto:size=8192")?)?;
    builder.push_pod(PodKey::ShmSize, ShmSize::new("64m")?.into())?;
    assert!(matches!(
        builder.push_pod(PodKey::UserNS, value("keep-id")?),
        Err(RenderError::DuplicateSingleton(key)) if key == "UserNS"
    ));
    assert!(matches!(
        builder.push_pod(PodKey::ShmSize, ShmSize::unlimited().into()),
        Err(RenderError::DuplicateSingleton(key)) if key == "ShmSize"
    ));
    let generated = builder.build(SourceId::new(88))?;
    assert_eq!(
        generated.text(),
        "[Pod]\nPodName=example-pod\nUserNS=auto:size=8192\nShmSize=64m\n"
    );
    Ok(())
}

#[test]
fn run_init_omission_true_false_and_raw_text_render_distinctly() -> Result<(), Box<dyn std::error::Error>> {
    for (source_id, authored) in [
        (SourceId::new(89), None),
        (SourceId::new(90), Some("true")),
        (SourceId::new(91), Some("false")),
        (SourceId::new(92), Some("vendor-defined-value")),
    ] {
        let mut builder = QuadletDocumentBuilder::new(QuadletUnitType::Container);
        builder.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
        if let Some(authored) = authored {
            builder.push_container(ContainerKey::RunInit, value(authored)?)?;
        }
        let generated = builder.build(source_id)?;
        let expected = authored.map_or_else(
            || "[Container]\nImage=example.invalid/app\n".to_owned(),
            |value| format!("[Container]\nImage=example.invalid/app\nRunInit={value}\n"),
        );
        assert_eq!(generated.text(), expected);
        assert_eq!(
            generated.document().entries().find_map(|entry| match entry.kind() {
                EntryKind::Container(ContainerKey::RunInit) => Some(entry.value().primary().text()),
                _ => None,
            }),
            authored
        );
    }
    Ok(())
}

#[test]
fn pull_omission_supported_forms_and_raw_text_render_distinctly() -> Result<(), Box<dyn std::error::Error>> {
    for (source_id, authored) in [
        (SourceId::new(93), None),
        (SourceId::new(94), Some("always")),
        (SourceId::new(95), Some("missing")),
        (SourceId::new(96), Some("never")),
        (SourceId::new(97), Some("newer")),
        (SourceId::new(98), Some("vendor-defined-policy")),
    ] {
        let mut builder = QuadletDocumentBuilder::new(QuadletUnitType::Container);
        builder.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
        if let Some(authored) = authored {
            builder.push_container(ContainerKey::Pull, value(authored)?)?;
        }
        let generated = builder.build(source_id)?;
        let expected = authored.map_or_else(
            || "[Container]\nImage=example.invalid/app\n".to_owned(),
            |value| format!("[Container]\nImage=example.invalid/app\nPull={value}\n"),
        );
        assert_eq!(generated.text(), expected);
        assert_eq!(
            generated.document().entries().find_map(|entry| match entry.kind() {
                EntryKind::Container(ContainerKey::Pull) => Some(entry.value().primary().text()),
                _ => None,
            }),
            authored
        );
    }
    Ok(())
}

#[test]
fn typed_pids_limits_render_only_unlimited_or_nonzero_ascii_decimals() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(PidsLimit::unlimited().as_str(), "-1");
    assert_eq!(PidsLimit::finite("1")?.as_str(), "1");
    assert_eq!(PidsLimit::finite("00047")?.as_str(), "00047");
    assert_eq!(
        PidsLimit::finite("999999999999999999999999999999999999")?.as_str(),
        "999999999999999999999999999999999999"
    );
    assert_eq!(PidsLimit::finite(""), Err(PidsLimitError::Empty));
    assert_eq!(PidsLimit::finite("0"), Err(PidsLimitError::Zero));
    assert_eq!(PidsLimit::finite("000"), Err(PidsLimitError::Zero));
    for non_decimal in ["-1", "+1", "1_000", "1.5", " 1", "１"] {
        assert_eq!(PidsLimit::finite(non_decimal), Err(PidsLimitError::NonDecimal));
    }

    for (source_id, limit, expected) in [
        (SourceId::new(99), PidsLimit::finite("47")?, "47"),
        (SourceId::new(100), PidsLimit::unlimited(), "-1"),
    ] {
        let mut builder = QuadletDocumentBuilder::new(QuadletUnitType::Container);
        builder.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
        builder.push_container(ContainerKey::PidsLimit, limit.into())?;
        assert_eq!(
            builder.build(source_id)?.text(),
            format!("[Container]\nImage=example.invalid/app\nPidsLimit={expected}\n")
        );
    }
    Ok(())
}

#[test]
fn raw_pids_limit_omission_zero_and_noncanonical_text_render_distinctly() -> Result<(), Box<dyn std::error::Error>> {
    for (source_id, authored) in [
        (SourceId::new(101), None),
        (SourceId::new(102), Some("0")),
        (SourceId::new(103), Some("vendor-defined-limit")),
        (SourceId::new(104), Some("999999999999999999999999999999999999")),
    ] {
        let mut builder = QuadletDocumentBuilder::new(QuadletUnitType::Container);
        builder.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
        if let Some(authored) = authored {
            builder.push_container(ContainerKey::PidsLimit, value(authored)?)?;
        }
        let expected = authored.map_or_else(
            || "[Container]\nImage=example.invalid/app\n".to_owned(),
            |limit| format!("[Container]\nImage=example.invalid/app\nPidsLimit={limit}\n"),
        );
        assert_eq!(builder.build(source_id)?.text(), expected);
    }
    Ok(())
}

#[test]
fn hostname_omission_and_raw_text_render_distinctly() -> Result<(), Box<dyn std::error::Error>> {
    for (source_id, authored) in [
        (SourceId::new(105), None),
        (SourceId::new(106), Some("app.example")),
        (SourceId::new(107), Some("Authored_Native_Value")),
    ] {
        let mut builder = QuadletDocumentBuilder::new(QuadletUnitType::Container);
        builder.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
        if let Some(authored) = authored {
            builder.push_container(ContainerKey::HostName, value(authored)?)?;
        }
        let generated = builder.build(source_id)?;
        let expected = authored.map_or_else(
            || "[Container]\nImage=example.invalid/app\n".to_owned(),
            |value| format!("[Container]\nImage=example.invalid/app\nHostName={value}\n"),
        );
        assert_eq!(generated.text(), expected);
        assert_eq!(
            generated.document().entries().find_map(|entry| match entry.kind() {
                EntryKind::Container(ContainerKey::HostName) => Some(entry.value().primary().text()),
                _ => None,
            }),
            authored
        );
    }
    Ok(())
}

#[test]
fn typed_shm_sizes_preserve_native_spelling_and_distinguish_zero_unlimited() -> Result<(), Box<dyn std::error::Error>> {
    for accepted in [
        "0",
        "00",
        "0b",
        "1",
        "00064",
        "1b",
        "2k",
        "3m",
        "4g",
        "999999999999999999999999999999999999g",
    ] {
        let size = ShmSize::new(accepted)?;
        assert_eq!(size.as_str(), accepted);
        assert_eq!(
            size.is_unlimited(),
            accepted.starts_with('0')
                && size
                    .as_str()
                    .trim_end_matches(['b', 'k', 'm', 'g'])
                    .bytes()
                    .all(|byte| byte == b'0')
        );
    }
    assert_eq!(ShmSize::unlimited().as_str(), "0");
    assert!(ShmSize::unlimited().is_unlimited());
    assert_eq!(ShmSize::new(""), Err(ShmSizeError::Empty));
    for invalid in [
        "+1", "-1", "1.5", "1e3", "1kb", "1mb", "1gb", "1K", "1M", "1G", "1KiB", " 1m", "1m ", "１m",
    ] {
        assert_eq!(ShmSize::new(invalid), Err(ShmSizeError::InvalidFormat));
    }

    for (source_id, unit_type, expected) in [
        (
            SourceId::new(108),
            QuadletUnitType::Container,
            "[Container]\nImage=example.invalid/app\nShmSize=00064m\n",
        ),
        (SourceId::new(109), QuadletUnitType::Pod, "[Pod]\nShmSize=0\n"),
    ] {
        let mut builder = QuadletDocumentBuilder::new(unit_type);
        if unit_type == QuadletUnitType::Container {
            builder.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
            builder.push_container(ContainerKey::ShmSize, ShmSize::new("00064m")?.into())?;
        } else {
            builder.push_pod(PodKey::ShmSize, ShmSize::unlimited().into())?;
        }
        assert_eq!(builder.build(source_id)?.text(), expected);
    }
    Ok(())
}

#[test]
fn raw_shm_size_omission_and_noncanonical_values_remain_distinct() -> Result<(), Box<dyn std::error::Error>> {
    for (source_id, authored) in [
        (SourceId::new(110), None),
        (SourceId::new(111), Some("0")),
        (SourceId::new(112), Some("vendor-defined-size")),
    ] {
        let mut builder = QuadletDocumentBuilder::new(QuadletUnitType::Container);
        builder.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
        if let Some(authored) = authored {
            builder.push_container(ContainerKey::ShmSize, value(authored)?)?;
        }
        let expected = authored.map_or_else(
            || "[Container]\nImage=example.invalid/app\n".to_owned(),
            |size| format!("[Container]\nImage=example.invalid/app\nShmSize={size}\n"),
        );
        assert_eq!(builder.build(source_id)?.text(), expected);
    }
    Ok(())
}

#[test]
fn typed_memory_limits_preserve_positive_native_spelling() -> Result<(), Box<dyn std::error::Error>> {
    for accepted in [
        "1",
        "00001",
        "1b",
        "2k",
        "3m",
        "4g",
        "999999999999999999999999999999999999b",
    ] {
        assert_eq!(Memory::new(accepted)?.as_str(), accepted);
    }
    assert_eq!(Memory::new(""), Err(MemoryError::Empty));
    for zero in ["0", "00", "0b", "000m"] {
        assert_eq!(Memory::new(zero), Err(MemoryError::Zero));
    }
    for invalid in [
        "+1", "-1", "1.5", "1e3", "1kb", "1mb", "1gb", "1K", "1M", "1G", "1KiB", " 1m", "1m ", "１m",
    ] {
        assert_eq!(Memory::new(invalid), Err(MemoryError::InvalidFormat));
    }

    let mut builder = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    builder.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
    builder.push_container(ContainerKey::Memory, Memory::new("00016777216b")?.into())?;
    assert_eq!(
        builder.build(SourceId::new(119))?.text(),
        "[Container]\nImage=example.invalid/app\nMemory=00016777216b\n"
    );
    Ok(())
}

#[test]
fn raw_memory_omission_and_noncanonical_values_remain_distinct() -> Result<(), Box<dyn std::error::Error>> {
    for (source_id, authored) in [
        (SourceId::new(120), None),
        (SourceId::new(121), Some("0")),
        (SourceId::new(122), Some(r#""64m""#)),
        (SourceId::new(123), Some("%h")),
        (SourceId::new(124), Some("vendor-defined-memory")),
    ] {
        let mut builder = QuadletDocumentBuilder::new(QuadletUnitType::Container);
        builder.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
        if let Some(authored) = authored {
            builder.push_container(ContainerKey::Memory, value(authored)?)?;
        }
        let expected = authored.map_or_else(
            || "[Container]\nImage=example.invalid/app\n".to_owned(),
            |memory| format!("[Container]\nImage=example.invalid/app\nMemory={memory}\n"),
        );
        assert_eq!(builder.build(source_id)?.text(), expected);
    }
    Ok(())
}

#[test]
fn preserves_typed_optional_dependencies_and_rejects_duplicate_notify() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    builder.push_systemd_unit(SystemdUnitKey::Wants, value("cache.service")?)?;
    builder.push_systemd_unit(SystemdUnitKey::After, value("cache.service")?)?;
    builder.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
    builder.push_container(ContainerKey::Notify, value("healthy")?)?;
    assert!(matches!(
        builder.push_container(ContainerKey::Notify, value("true")?),
        Err(RenderError::DuplicateSingleton(key)) if key == "Notify"
    ));
    let generated = builder.build(SourceId::new(87))?;
    assert_eq!(
        generated.text(),
        concat!(
            "[Unit]\n",
            "Wants=cache.service\n",
            "After=cache.service\n",
            "\n",
            "[Container]\n",
            "Image=example.invalid/app\n",
            "Notify=healthy\n",
        )
    );
    Ok(())
}

#[test]
fn rejects_unsafe_values_wrong_units_and_duplicate_singletons() -> Result<(), Box<dyn std::error::Error>> {
    assert!(matches!(
        EntryValue::new("first\nsecond"),
        Err(RenderError::InvalidValue)
    ));

    let mut container = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    assert!(matches!(
        container.push_network(NetworkKey::NetworkName, value("wrong")?),
        Err(RenderError::WrongUnitType { .. })
    ));
    container.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
    assert!(matches!(
        container.push_container(ContainerKey::Image, value("example.invalid/other")?),
        Err(RenderError::DuplicateSingleton(key)) if key == "Image"
    ));
    container.push_container(ContainerKey::User, value("1001")?)?;
    assert!(matches!(
        container.push_container(ContainerKey::User, value("1002")?),
        Err(RenderError::DuplicateSingleton(key)) if key == "User"
    ));
    container.push_container(ContainerKey::Entrypoint, value("/usr/bin/env")?)?;
    assert!(matches!(
        container.push_container(ContainerKey::Entrypoint, value("/bin/sh")?),
        Err(RenderError::DuplicateSingleton(key)) if key == "Entrypoint"
    ));
    container.push_container(ContainerKey::RunInit, value("true")?)?;
    assert!(matches!(
        container.push_container(ContainerKey::RunInit, value("false")?),
        Err(RenderError::DuplicateSingleton(key)) if key == "RunInit"
    ));
    container.push_container(ContainerKey::StopSignal, value("SIGTERM")?)?;
    assert!(matches!(
        container.push_container(ContainerKey::StopSignal, value("9")?),
        Err(RenderError::DuplicateSingleton(key)) if key == "StopSignal"
    ));
    container.push_container(ContainerKey::StopTimeout, value("0")?)?;
    assert!(matches!(
        container.push_container(ContainerKey::StopTimeout, value("30")?),
        Err(RenderError::DuplicateSingleton(key)) if key == "StopTimeout"
    ));
    container.push_container(ContainerKey::Pull, value("missing")?)?;
    assert!(matches!(
        container.push_container(ContainerKey::Pull, value("always")?),
        Err(RenderError::DuplicateSingleton(key)) if key == "Pull"
    ));
    container.push_container(ContainerKey::PidsLimit, PidsLimit::finite("47")?.into())?;
    assert!(matches!(
        container.push_container(ContainerKey::PidsLimit, PidsLimit::unlimited().into()),
        Err(RenderError::DuplicateSingleton(key)) if key == "PidsLimit"
    ));
    container.push_container(ContainerKey::HostName, value("app.example")?)?;
    assert!(matches!(
        container.push_container(ContainerKey::HostName, value("other.example")?),
        Err(RenderError::DuplicateSingleton(key)) if key == "HostName"
    ));
    container.push_container(ContainerKey::ShmSize, ShmSize::new("64m")?.into())?;
    assert!(matches!(
        container.push_container(ContainerKey::ShmSize, ShmSize::unlimited().into()),
        Err(RenderError::DuplicateSingleton(key)) if key == "ShmSize"
    ));
    container.push_container(ContainerKey::Memory, Memory::new("16m")?.into())?;
    assert!(matches!(
        container.push_container(ContainerKey::Memory, Memory::new("32m")?.into()),
        Err(RenderError::DuplicateSingleton(key)) if key == "Memory"
    ));
    assert!(matches!(
        container.push_systemd(SystemdSection::Unit, "Invalid-Key", value("value")?),
        Err(RenderError::InvalidKey(key)) if key == "Invalid-Key"
    ));
    Ok(())
}

#[test]
fn entry_value_enforces_physical_line_safety_not_lifecycle_grammar() -> Result<(), Box<dyn std::error::Error>> {
    for unsafe_value in ["first\nsecond", "first\rsecond", "first\0second"] {
        assert!(matches!(EntryValue::new(unsafe_value), Err(RenderError::InvalidValue)));
    }

    let mut container = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    container.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
    container.push_container(ContainerKey::StopSignal, value("vendor-defined-signal")?)?;
    container.push_container(ContainerKey::StopTimeout, value("-1.5")?)?;
    assert_eq!(
        container.build(SourceId::new(88))?.text(),
        concat!(
            "[Container]\n",
            "Image=example.invalid/app\n",
            "StopSignal=vendor-defined-signal\n",
            "StopTimeout=-1.5\n",
        )
    );
    Ok(())
}

#[test]
fn drop_capability_builder_preserves_raw_values_without_native_normalization() -> Result<(), Box<dyn std::error::Error>>
{
    let mut container = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    container.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
    for authored in [
        "CAP_NET_ADMIN",
        "CAP_NET_ADMIN",
        "ALL",
        "CAP_DAC_OVERRIDE CAP_IPC_OWNER",
        "Vendor_Defined Capability Text",
    ] {
        container.push_container(ContainerKey::DropCapability, value(authored)?)?;
    }
    assert_eq!(
        container.build(SourceId::new(113))?.text(),
        concat!(
            "[Container]\n",
            "Image=example.invalid/app\n",
            "DropCapability=CAP_NET_ADMIN\n",
            "DropCapability=CAP_NET_ADMIN\n",
            "DropCapability=ALL\n",
            "DropCapability=CAP_DAC_OVERRIDE CAP_IPC_OWNER\n",
            "DropCapability=Vendor_Defined Capability Text\n",
        )
    );

    for unsafe_value in ["CAP_NET_ADMIN\nALL", "CAP_NET_ADMIN\rALL", "CAP_NET_ADMIN\0ALL"] {
        assert!(matches!(EntryValue::new(unsafe_value), Err(RenderError::InvalidValue)));
    }
    Ok(())
}

#[test]
fn add_capability_builder_preserves_resets_duplicates_and_raw_values_without_normalization()
-> Result<(), Box<dyn std::error::Error>> {
    let mut container = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    container.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
    for authored in [
        "CAP_NET_ADMIN",
        "",
        "CAP_NET_ADMIN",
        "ALL",
        "CAP_DAC_OVERRIDE CAP_IPC_OWNER",
        "Vendor_Defined Capability Text",
    ] {
        container.push_container(ContainerKey::AddCapability, value(authored)?)?;
    }
    assert_eq!(
        container.build(SourceId::new(114))?.text(),
        concat!(
            "[Container]\n",
            "Image=example.invalid/app\n",
            "AddCapability=CAP_NET_ADMIN\n",
            "AddCapability=\n",
            "AddCapability=CAP_NET_ADMIN\n",
            "AddCapability=ALL\n",
            "AddCapability=CAP_DAC_OVERRIDE CAP_IPC_OWNER\n",
            "AddCapability=Vendor_Defined Capability Text\n",
        )
    );

    for unsafe_value in ["CAP_NET_ADMIN\nALL", "CAP_NET_ADMIN\rALL", "CAP_NET_ADMIN\0ALL"] {
        assert!(matches!(EntryValue::new(unsafe_value), Err(RenderError::InvalidValue)));
    }
    Ok(())
}

#[test]
fn tmpfs_builder_preserves_resets_duplicates_case_options_and_raw_values_without_normalization()
-> Result<(), Box<dyn std::error::Error>> {
    let mut container = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    container.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
    for authored in [
        "/Before:RW,NoExec",
        "/before-two:size=64M",
        "",
        "/data:mode=755,uid=1009,gid=1009",
        "/data:mode=755,uid=1009,gid=1009",
        "Vendor_Defined Tmpfs Options",
    ] {
        container.push_container(ContainerKey::Tmpfs, value(authored)?)?;
    }
    assert_eq!(
        container.build(SourceId::new(115))?.text(),
        concat!(
            "[Container]\n",
            "Image=example.invalid/app\n",
            "Tmpfs=/Before:RW,NoExec\n",
            "Tmpfs=/before-two:size=64M\n",
            "Tmpfs=\n",
            "Tmpfs=/data:mode=755,uid=1009,gid=1009\n",
            "Tmpfs=/data:mode=755,uid=1009,gid=1009\n",
            "Tmpfs=Vendor_Defined Tmpfs Options\n",
        )
    );

    for unsafe_value in ["/data\n/cache", "/data\r/cache", "/data\0/cache"] {
        assert!(matches!(EntryValue::new(unsafe_value), Err(RenderError::InvalidValue)));
    }
    Ok(())
}

#[test]
fn sysctl_builder_preserves_resets_duplicates_case_whitespace_quoting_and_specifiers_without_interpretation()
-> Result<(), Box<dyn std::error::Error>> {
    let mut container = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    container.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
    for authored in [
        "net.ipv4.conf.all.rp_filter=2 net.ipv4.ip_forward=0",
        r#"kernel.domainname="Authored Value""#,
        " net.ipv4.ip_forward=0 ",
        "net.ipv4.conf.%i.forwarding=%n",
        "",
        "net.ipv4.ip_forward=1",
        "net.ipv4.ip_forward=1",
        "Vendor_Defined=MixedCase",
    ] {
        container.push_container(ContainerKey::Sysctl, value(authored)?)?;
    }
    assert_eq!(
        container.build(SourceId::new(116))?.text(),
        concat!(
            "[Container]\n",
            "Image=example.invalid/app\n",
            "Sysctl=net.ipv4.conf.all.rp_filter=2 net.ipv4.ip_forward=0\n",
            "Sysctl=kernel.domainname=\"Authored Value\"\n",
            "Sysctl= net.ipv4.ip_forward=0 \n",
            "Sysctl=net.ipv4.conf.%i.forwarding=%n\n",
            "Sysctl=\n",
            "Sysctl=net.ipv4.ip_forward=1\n",
            "Sysctl=net.ipv4.ip_forward=1\n",
            "Sysctl=Vendor_Defined=MixedCase\n",
        )
    );

    for unsafe_value in ["net.ipv4.ip_forward=1\nnext=2", "one=1\rtwo=2", "one=1\0two=2"] {
        assert!(matches!(EntryValue::new(unsafe_value), Err(RenderError::InvalidValue)));
    }
    Ok(())
}

#[test]
fn ulimit_builder_preserves_resets_duplicates_case_quoting_and_specifiers_without_interpretation()
-> Result<(), Box<dyn std::error::Error>> {
    let mut container = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    container.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
    for authored in [
        "Core=0:0",
        r#"nofile="1024:2048""#,
        "stack=%h:%n",
        "",
        "nproc=4096:8192",
        "nproc=4096:8192",
        "Vendor_Defined=Soft:Hard",
    ] {
        container.push_container(ContainerKey::Ulimit, value(authored)?)?;
    }
    assert_eq!(
        container.build(SourceId::new(117))?.text(),
        concat!(
            "[Container]\n",
            "Image=example.invalid/app\n",
            "Ulimit=Core=0:0\n",
            "Ulimit=nofile=\"1024:2048\"\n",
            "Ulimit=stack=%h:%n\n",
            "Ulimit=\n",
            "Ulimit=nproc=4096:8192\n",
            "Ulimit=nproc=4096:8192\n",
            "Ulimit=Vendor_Defined=Soft:Hard\n",
        )
    );

    for unsafe_value in ["core=0:0\nnofile=1:2", "core=0:0\rnproc=1:2", "core=0:0\0stack=1:2"] {
        assert!(matches!(EntryValue::new(unsafe_value), Err(RenderError::InvalidValue)));
    }
    Ok(())
}

#[test]
fn add_device_builder_preserves_resets_duplicates_case_quotes_specifiers_whitespace_and_leading_dash()
-> Result<(), Box<dyn std::error::Error>> {
    let mut container = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    container.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
    for authored in [
        "/dev/null:/dev/pre-null:r /dev/zero:/dev/pre-zero:w",
        "",
        r#""/dev/null:/dev/final null:r" /dev/zero:/dev/final-zero:w"#,
        "%h/Device:/dev/MixedCase:rwm",
        "-/dev/optional:/dev/optional:r",
        r#""/dev/null:/dev/final null:r" /dev/zero:/dev/final-zero:w"#,
        "Vendor_Defined Device Text",
    ] {
        container.push_container(ContainerKey::AddDevice, value(authored)?)?;
    }
    assert_eq!(
        container.build(SourceId::new(118))?.text(),
        concat!(
            "[Container]\n",
            "Image=example.invalid/app\n",
            "AddDevice=/dev/null:/dev/pre-null:r /dev/zero:/dev/pre-zero:w\n",
            "AddDevice=\n",
            "AddDevice=\"/dev/null:/dev/final null:r\" /dev/zero:/dev/final-zero:w\n",
            "AddDevice=%h/Device:/dev/MixedCase:rwm\n",
            "AddDevice=-/dev/optional:/dev/optional:r\n",
            "AddDevice=\"/dev/null:/dev/final null:r\" /dev/zero:/dev/final-zero:w\n",
            "AddDevice=Vendor_Defined Device Text\n",
        )
    );

    for unsafe_value in ["/dev/null\n/dev/zero", "/dev/null\r/dev/zero", "/dev/null\0/dev/zero"] {
        assert!(matches!(EntryValue::new(unsafe_value), Err(RenderError::InvalidValue)));
    }
    Ok(())
}

#[test]
fn dns_builder_preserves_resets_duplicates_order_quotes_and_specifiers_without_interpretation()
-> Result<(), Box<dyn std::error::Error>> {
    let mut container = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    container.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
    for authored in [
        "1.1.1.1",
        "1.1.1.1",
        "",
        "9.9.9.9",
        "2001:4860:4860::8888",
        r#""Authored Resolver""#,
        "%h",
        "Vendor_Defined_DNS",
    ] {
        container.push_container(ContainerKey::DNS, value(authored)?)?;
    }
    assert_eq!(
        container.build(SourceId::new(119))?.text(),
        concat!(
            "[Container]\n",
            "Image=example.invalid/app\n",
            "DNS=1.1.1.1\n",
            "DNS=1.1.1.1\n",
            "DNS=\n",
            "DNS=9.9.9.9\n",
            "DNS=2001:4860:4860::8888\n",
            "DNS=\"Authored Resolver\"\n",
            "DNS=%h\n",
            "DNS=Vendor_Defined_DNS\n",
        )
    );

    for unsafe_value in ["1.1.1.1\n9.9.9.9", "1.1.1.1\r9.9.9.9", "1.1.1.1\09.9.9.9"] {
        assert!(matches!(EntryValue::new(unsafe_value), Err(RenderError::InvalidValue)));
    }
    Ok(())
}

#[test]
fn dns_option_builder_preserves_resets_duplicates_order_quotes_whitespace_and_specifiers_without_interpretation()
-> Result<(), Box<dyn std::error::Error>> {
    let mut container = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    container.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
    for authored in [
        "rotate",
        "rotate",
        "",
        "ndots:1",
        "use-vc",
        r#""Authored Option""#,
        "%h",
        "Vendor Defined Option",
    ] {
        container.push_container(ContainerKey::DNSOption, value(authored)?)?;
    }
    assert_eq!(
        container.build(SourceId::new(120))?.text(),
        concat!(
            "[Container]\n",
            "Image=example.invalid/app\n",
            "DNSOption=rotate\n",
            "DNSOption=rotate\n",
            "DNSOption=\n",
            "DNSOption=ndots:1\n",
            "DNSOption=use-vc\n",
            "DNSOption=\"Authored Option\"\n",
            "DNSOption=%h\n",
            "DNSOption=Vendor Defined Option\n",
        )
    );

    for unsafe_value in ["rotate\nuse-vc", "rotate\ruse-vc", "rotate\0use-vc"] {
        assert!(matches!(EntryValue::new(unsafe_value), Err(RenderError::InvalidValue)));
    }
    Ok(())
}

#[test]
fn dns_search_builder_preserves_resets_duplicates_order_quotes_and_specifiers_without_interpretation()
-> Result<(), Box<dyn std::error::Error>> {
    let mut container = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    container.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
    for authored in [
        "pre.example.com",
        "pre.example.com",
        "",
        "dc1.example.com",
        ".",
        r#""Authored Search""#,
        "%h",
        "Vendor Defined Search",
    ] {
        container.push_container(ContainerKey::DNSSearch, value(authored)?)?;
    }
    assert_eq!(
        container.build(SourceId::new(121))?.text(),
        concat!(
            "[Container]\n",
            "Image=example.invalid/app\n",
            "DNSSearch=pre.example.com\n",
            "DNSSearch=pre.example.com\n",
            "DNSSearch=\n",
            "DNSSearch=dc1.example.com\n",
            "DNSSearch=.\n",
            "DNSSearch=\"Authored Search\"\n",
            "DNSSearch=%h\n",
            "DNSSearch=Vendor Defined Search\n",
        )
    );

    for unsafe_value in ["example.com\n.", "example.com\r.", "example.com\0."] {
        assert!(matches!(EntryValue::new(unsafe_value), Err(RenderError::InvalidValue)));
    }
    Ok(())
}

#[test]
fn expose_host_port_builder_preserves_resets_duplicates_order_quotes_specifiers_invalid_and_sctp_without_interpretation()
-> Result<(), Box<dyn std::error::Error>> {
    let mut container = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    container.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
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
        container.push_container(ContainerKey::ExposeHostPort, value(authored)?)?;
    }
    assert_eq!(
        container.build(SourceId::new(122))?.text(),
        concat!(
            "[Container]\n",
            "Image=example.invalid/app\n",
            "ExposeHostPort=1000\n",
            "ExposeHostPort=1000\n",
            "ExposeHostPort=\n",
            "ExposeHostPort=3000\n",
            "ExposeHostPort=8080-8085\n",
            "ExposeHostPort=9090/tcp\n",
            "ExposeHostPort=5353/udp\n",
            "ExposeHostPort=5353/sctp\n",
            "ExposeHostPort=\"Authored Port\"\n",
            "ExposeHostPort=%i\n",
            "ExposeHostPort=not-a-port\n",
        )
    );

    for unsafe_value in ["8080\n9090", "8080\r9090", "8080\09090"] {
        assert!(matches!(EntryValue::new(unsafe_value), Err(RenderError::InvalidValue)));
    }
    Ok(())
}

#[test]
fn annotation_builder_preserves_resets_duplicates_order_quotes_whitespace_specifiers_and_malformed_values()
-> Result<(), Box<dyn std::error::Error>> {
    let mut container = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    container.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
    for authored in [
        "org.example.name=first",
        "org.example.name=first",
        "",
        "org.example.name=final",
        r#""org.example.quoted=Authored Value""#,
        "org.example.specifier=%i",
        "key-only",
        " malformed = value ",
    ] {
        container.push_container(ContainerKey::Annotation, value(authored)?)?;
    }
    assert_eq!(
        container.build(SourceId::new(123))?.text(),
        concat!(
            "[Container]\n",
            "Image=example.invalid/app\n",
            "Annotation=org.example.name=first\n",
            "Annotation=org.example.name=first\n",
            "Annotation=\n",
            "Annotation=org.example.name=final\n",
            "Annotation=\"org.example.quoted=Authored Value\"\n",
            "Annotation=org.example.specifier=%i\n",
            "Annotation=key-only\n",
            "Annotation= malformed = value \n",
        )
    );

    for unsafe_value in [
        "key=value\nother=value",
        "key=value\rother=value",
        "key=value\0other=value",
    ] {
        assert!(matches!(EntryValue::new(unsafe_value), Err(RenderError::InvalidValue)));
    }
    Ok(())
}

#[test]
fn apparmor_builder_preserves_one_exact_opaque_physical_value() -> Result<(), Box<dyn std::error::Error>> {
    for (source_id, authored) in [
        (SourceId::new(124), "unconfined"),
        (SourceId::new(125), ""),
        (SourceId::new(126), r#""Authored Profile""#),
        (SourceId::new(127), " profile:with %i "),
        (SourceId::new(128), "Vendor_Defined/Profile"),
    ] {
        let mut container = QuadletDocumentBuilder::new(QuadletUnitType::Container);
        container.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
        container.push_container(ContainerKey::AppArmor, value(authored)?)?;
        assert_eq!(
            container.build(source_id)?.text(),
            format!("[Container]\nImage=example.invalid/app\nAppArmor={authored}\n")
        );
    }

    let mut duplicate = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    duplicate.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
    duplicate.push_container(ContainerKey::AppArmor, value("first")?)?;
    assert!(matches!(
        duplicate.push_container(ContainerKey::AppArmor, value("second")?),
        Err(RenderError::DuplicateSingleton(key)) if key == "AppArmor"
    ));
    Ok(())
}

#[test]
fn no_new_privileges_builder_preserves_one_exact_opaque_physical_value() -> Result<(), Box<dyn std::error::Error>> {
    for (source_id, authored) in [
        (SourceId::new(129), "true"),
        (SourceId::new(130), "yes"),
        (SourceId::new(131), "false"),
        (SourceId::new(132), ""),
        (SourceId::new(133), r#""true""#),
        (SourceId::new(134), " %i "),
        (SourceId::new(135), "not-a-boolean"),
    ] {
        let mut container = QuadletDocumentBuilder::new(QuadletUnitType::Container);
        container.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
        container.push_container(ContainerKey::NoNewPrivileges, value(authored)?)?;
        assert_eq!(
            container.build(source_id)?.text(),
            format!("[Container]\nImage=example.invalid/app\nNoNewPrivileges={authored}\n")
        );
    }

    let mut duplicate = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    duplicate.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
    duplicate.push_container(ContainerKey::NoNewPrivileges, value("true")?)?;
    assert!(matches!(
        duplicate.push_container(ContainerKey::NoNewPrivileges, value("false")?),
        Err(RenderError::DuplicateSingleton(key)) if key == "NoNewPrivileges"
    ));
    Ok(())
}

#[test]
fn seccomp_profile_builder_preserves_one_exact_opaque_physical_value() -> Result<(), Box<dyn std::error::Error>> {
    for (source_id, authored) in [
        (SourceId::new(136), "unconfined"),
        (SourceId::new(137), "/tmp/profile.json"),
        (SourceId::new(138), ""),
        (SourceId::new(139), r#""/tmp/Authored Profile.json""#),
        (SourceId::new(140), " %h/profiles/%i.json "),
        (SourceId::new(141), "malformed:value"),
    ] {
        let mut container = QuadletDocumentBuilder::new(QuadletUnitType::Container);
        container.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
        container.push_container(ContainerKey::SeccompProfile, value(authored)?)?;
        assert_eq!(
            container.build(source_id)?.text(),
            format!("[Container]\nImage=example.invalid/app\nSeccompProfile={authored}\n")
        );
    }

    let mut duplicate = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    duplicate.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
    duplicate.push_container(ContainerKey::SeccompProfile, value("unconfined")?)?;
    assert!(matches!(
        duplicate.push_container(ContainerKey::SeccompProfile, value("/tmp/profile.json")?),
        Err(RenderError::DuplicateSingleton(key)) if key == "SeccompProfile"
    ));
    Ok(())
}

#[test]
fn security_label_disable_builder_preserves_one_exact_opaque_physical_value() -> Result<(), Box<dyn std::error::Error>>
{
    for (source_id, authored) in [
        (SourceId::new(142), "true"),
        (SourceId::new(143), "false"),
        (SourceId::new(144), ""),
        (SourceId::new(145), r#""true""#),
        (SourceId::new(146), " %i "),
        (SourceId::new(147), "not-a-boolean"),
    ] {
        let mut container = QuadletDocumentBuilder::new(QuadletUnitType::Container);
        container.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
        container.push_container(ContainerKey::SecurityLabelDisable, value(authored)?)?;
        assert_eq!(
            container.build(source_id)?.text(),
            format!("[Container]\nImage=example.invalid/app\nSecurityLabelDisable={authored}\n")
        );
    }

    let mut duplicate = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    duplicate.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
    duplicate.push_container(ContainerKey::SecurityLabelDisable, value("true")?)?;
    assert!(matches!(
        duplicate.push_container(ContainerKey::SecurityLabelDisable, value("false")?),
        Err(RenderError::DuplicateSingleton(key)) if key == "SecurityLabelDisable"
    ));
    Ok(())
}

#[test]
fn security_label_file_type_builder_preserves_one_exact_opaque_physical_value() -> Result<(), Box<dyn std::error::Error>>
{
    for (source_id, authored) in [
        (SourceId::new(148), "container_file_t"),
        (SourceId::new(149), ""),
        (SourceId::new(150), r#""custom_file_t""#),
        (SourceId::new(151), " custom file type "),
        (SourceId::new(152), "%i_file_t"),
        (SourceId::new(153), "malformed:type"),
    ] {
        let mut container = QuadletDocumentBuilder::new(QuadletUnitType::Container);
        container.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
        container.push_container(ContainerKey::SecurityLabelFileType, value(authored)?)?;
        assert_eq!(
            container.build(source_id)?.text(),
            format!("[Container]\nImage=example.invalid/app\nSecurityLabelFileType={authored}\n")
        );
    }

    let mut duplicate = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    duplicate.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
    duplicate.push_container(ContainerKey::SecurityLabelFileType, value("container_file_t")?)?;
    assert!(matches!(
        duplicate.push_container(ContainerKey::SecurityLabelFileType, value("custom_file_t")?),
        Err(RenderError::DuplicateSingleton(key)) if key == "SecurityLabelFileType"
    ));
    Ok(())
}

#[test]
fn security_label_level_builder_preserves_one_exact_opaque_physical_value() -> Result<(), Box<dyn std::error::Error>> {
    for (source_id, authored) in [
        (SourceId::new(154), "s0:c1,c2"),
        (SourceId::new(155), ""),
        (SourceId::new(156), r#""s0:c3,c4""#),
        (SourceId::new(157), " s0:c5,c6 "),
        (SourceId::new(158), "%i:c7,c8"),
        (SourceId::new(159), "malformed level"),
    ] {
        let mut container = QuadletDocumentBuilder::new(QuadletUnitType::Container);
        container.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
        container.push_container(ContainerKey::SecurityLabelLevel, value(authored)?)?;
        assert_eq!(
            container.build(source_id)?.text(),
            format!("[Container]\nImage=example.invalid/app\nSecurityLabelLevel={authored}\n")
        );
    }

    let mut duplicate = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    duplicate.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
    duplicate.push_container(ContainerKey::SecurityLabelLevel, value("s0:c1,c2")?)?;
    assert!(matches!(
        duplicate.push_container(ContainerKey::SecurityLabelLevel, value("s0:c3,c4")?),
        Err(RenderError::DuplicateSingleton(key)) if key == "SecurityLabelLevel"
    ));
    Ok(())
}

#[test]
fn security_label_nested_builder_preserves_one_exact_opaque_physical_value() -> Result<(), Box<dyn std::error::Error>> {
    for (source_id, authored) in [
        (SourceId::new(160), "true"),
        (SourceId::new(161), "false"),
        (SourceId::new(162), ""),
        (SourceId::new(163), r#""true""#),
        (SourceId::new(164), " false "),
        (SourceId::new(165), "%i"),
        (SourceId::new(166), "not-a-boolean"),
    ] {
        let mut container = QuadletDocumentBuilder::new(QuadletUnitType::Container);
        container.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
        container.push_container(ContainerKey::SecurityLabelNested, value(authored)?)?;
        assert_eq!(
            container.build(source_id)?.text(),
            format!("[Container]\nImage=example.invalid/app\nSecurityLabelNested={authored}\n")
        );
    }

    let mut duplicate = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    duplicate.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
    duplicate.push_container(ContainerKey::SecurityLabelNested, value("true")?)?;
    assert!(matches!(
        duplicate.push_container(ContainerKey::SecurityLabelNested, value("false")?),
        Err(RenderError::DuplicateSingleton(key)) if key == "SecurityLabelNested"
    ));
    Ok(())
}

#[test]
fn security_label_type_builder_preserves_one_exact_opaque_physical_value() -> Result<(), Box<dyn std::error::Error>> {
    for (source_id, authored) in [
        (SourceId::new(167), "container_t"),
        (SourceId::new(168), ""),
        (SourceId::new(169), r#""custom_t""#),
        (SourceId::new(170), " custom type "),
        (SourceId::new(171), "%i_t"),
        (SourceId::new(172), "malformed:type"),
    ] {
        let mut container = QuadletDocumentBuilder::new(QuadletUnitType::Container);
        container.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
        container.push_container(ContainerKey::SecurityLabelType, value(authored)?)?;
        assert_eq!(
            container.build(source_id)?.text(),
            format!("[Container]\nImage=example.invalid/app\nSecurityLabelType={authored}\n")
        );
    }

    let mut duplicate = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    duplicate.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
    duplicate.push_container(ContainerKey::SecurityLabelType, value("container_t")?)?;
    assert!(matches!(
        duplicate.push_container(ContainerKey::SecurityLabelType, value("custom_t")?),
        Err(RenderError::DuplicateSingleton(key)) if key == "SecurityLabelType"
    ));
    Ok(())
}

#[test]
fn mask_builder_preserves_repeated_exact_opaque_physical_values_in_order() -> Result<(), Box<dyn std::error::Error>> {
    let authored = [
        "/pre/one:/pre/two",
        "",
        r#""/quoted/path:%h/private""#,
        " malformed::path ",
        "/proc/acpi:/sys/firmware",
        "/proc/acpi:/sys/firmware",
    ];
    let mut container = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    container.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
    for entry in authored {
        container.push_container(ContainerKey::Mask, value(entry)?)?;
    }
    let generated = container.build(SourceId::new(173))?;
    assert_eq!(
        generated.text(),
        concat!(
            "[Container]\n",
            "Image=example.invalid/app\n",
            "Mask=/pre/one:/pre/two\n",
            "Mask=\n",
            "Mask=\"/quoted/path:%h/private\"\n",
            "Mask= malformed::path \n",
            "Mask=/proc/acpi:/sys/firmware\n",
            "Mask=/proc/acpi:/sys/firmware\n",
        )
    );
    Ok(())
}

#[test]
fn unmask_builder_preserves_repeated_exact_opaque_physical_values_in_order() -> Result<(), Box<dyn std::error::Error>> {
    let authored = [
        "/pre/one:/pre/two",
        "/pre/one:/pre/two",
        "",
        "ALL",
        "/proc/acpi:/sys/firmware",
        r#""/quoted/%h/*:/sys/*""#,
        "%h/private:/proc/*",
        " whitespace : value ",
        "malformed::path:",
    ];
    let mut container = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    container.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
    for entry in authored {
        container.push_container(ContainerKey::Unmask, value(entry)?)?;
    }
    let generated = container.build(SourceId::new(174))?;
    assert_eq!(
        generated.text(),
        concat!(
            "[Container]\n",
            "Image=example.invalid/app\n",
            "Unmask=/pre/one:/pre/two\n",
            "Unmask=/pre/one:/pre/two\n",
            "Unmask=\n",
            "Unmask=ALL\n",
            "Unmask=/proc/acpi:/sys/firmware\n",
            "Unmask=\"/quoted/%h/*:/sys/*\"\n",
            "Unmask=%h/private:/proc/*\n",
            "Unmask= whitespace : value \n",
            "Unmask=malformed::path:\n",
        )
    );
    Ok(())
}

#[test]
fn refuses_a_generated_container_without_a_workload_source() {
    let builder = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    let result = builder.build(SourceId::new(85));
    assert!(matches!(result, Err(RenderError::InvalidDocument(_))));
    let codes = match result {
        Ok(_) => Vec::new(),
        Err(error) => error
            .diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str().to_owned())
            .collect::<Vec<_>>(),
    };
    assert_eq!(codes, ["QLM0002"]);
}

fn value(value: &str) -> Result<EntryValue, RenderError> {
    EntryValue::new(value)
}
