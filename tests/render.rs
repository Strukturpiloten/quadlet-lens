//! Programmatic Quadlet construction and parse-back validation.

use quadlet_lens::{
    model::{ContainerKey, NamedQuadletDocument, NetworkKey, PodKey, QuadletDocumentSet, QuadletUnitType, VolumeKey},
    render::{EntryValue, QuadletDocumentBuilder, RenderError, SystemdSection, SystemdUnitKey},
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
        )
    );
    Ok(())
}

#[test]
fn builds_a_singleton_pod_user_namespace() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = QuadletDocumentBuilder::new(QuadletUnitType::Pod);
    builder.push_pod(PodKey::PodName, value("example-pod")?)?;
    builder.push_pod(PodKey::UserNS, value("auto:size=8192")?)?;
    assert!(matches!(
        builder.push_pod(PodKey::UserNS, value("keep-id")?),
        Err(RenderError::DuplicateSingleton(key)) if key == "UserNS"
    ));
    let generated = builder.build(SourceId::new(88))?;
    assert_eq!(generated.text(), "[Pod]\nPodName=example-pod\nUserNS=auto:size=8192\n");
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
    assert!(matches!(
        container.push_systemd(SystemdSection::Unit, "Invalid-Key", value("value")?),
        Err(RenderError::InvalidKey(key)) if key == "Invalid-Key"
    ));
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
