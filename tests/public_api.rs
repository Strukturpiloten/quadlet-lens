//! Consumer-facing compile and behavior contract for the supported 0.1.x API.

use quadlet_lens::capability::{CapabilityCatalogue, PodmanTarget, PodmanVersion, SupportClassification};
use quadlet_lens::model::{
    ContainerKey, NamedQuadletDocument, PodKey, QuadletDocument, QuadletDocumentSet, QuadletUnitType,
};
use quadlet_lens::path::{PathForm, classify_path};
use quadlet_lens::render::{EntryValue, QuadletDocumentBuilder, SystemdUnitKey};
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
        ],
        [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
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
        ],
        [0, 1, 2, 3, 4, 5]
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
    let support = catalogue.evaluate("quadlet.container.image", target);
    assert_eq!(support.classification(), SupportClassification::Native);
    let rootfs_support = catalogue.evaluate("quadlet.container.rootfs", target);
    assert_eq!(rootfs_support.classification(), SupportClassification::Native);
    let host_support = catalogue.evaluate("quadlet.container.add-host", target);
    assert_eq!(host_support.classification(), SupportClassification::Native);
    let health_support = catalogue.evaluate("quadlet.container.health-timeout", target);
    assert_eq!(health_support.classification(), SupportClassification::Native);
    let readiness_support = catalogue.evaluate("quadlet.container.notify-healthy", target);
    assert_eq!(readiness_support.classification(), SupportClassification::Native);
    for capability in [
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
