//! Programmatic Quadlet construction and parse-back validation.

use quadlet_lens::{
    model::{ContainerKey, NamedQuadletDocument, NetworkKey, QuadletDocumentSet, QuadletUnitType, VolumeKey},
    render::{EntryValue, QuadletDocumentBuilder, RenderError, SystemdSection},
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
    container.push_container(ContainerKey::AddHost, value("host.docker.internal:host-gateway")?)?;
    container.push_container(ContainerKey::Image, value("example.invalid/app:1@sha256:abcd")?)?;
    container.push_container(ContainerKey::Exec, value("php -v")?)?;
    container.push_container(ContainerKey::Environment, value("APP_ENV=production")?)?;
    container.push_container(ContainerKey::HealthCmd, value("/usr/bin/true")?)?;
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
            "\n",
            "[Container]\n",
            "AddHost=host.docker.internal:host-gateway\n",
            "Image=example.invalid/app:1@sha256:abcd\n",
            "Exec=php -v\n",
            "Environment=APP_ENV=production\n",
            "HealthCmd=/usr/bin/true\n",
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
    assert!(matches!(
        container.push_systemd(SystemdSection::Unit, "Invalid-Key", value("value")?),
        Err(RenderError::InvalidKey(key)) if key == "Invalid-Key"
    ));
    Ok(())
}

#[test]
fn refuses_a_generated_container_without_an_image() {
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
