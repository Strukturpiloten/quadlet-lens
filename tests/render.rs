//! Programmatic Quadlet construction and parse-back validation.

use quadlet_lens::{
    model::{
        ArtifactKey, BuildKey, ContainerKey, EntryKind, ImageKey, KubeKey, NamedQuadletDocument, NetworkKey, PodKey,
        QuadletDocument, QuadletDocumentSet, QuadletKey, QuadletUnitType, SystemdUnitKey, ValueKind, VolumeKey,
    },
    render::{
        ContainerEnvironmentDirective, ContainerEnvironmentPlan, EntryValue, EnvironmentAssignment,
        EnvironmentAssignmentError, EnvironmentAssignments, EnvironmentAssignmentsError, EnvironmentReset, Memory,
        MemoryError, PidsLimit, PidsLimitError, QuadletDocumentBuilder, RenderError, ShmSize, ShmSizeError,
        SystemdSection,
    },
    source::SourceId,
};

#[test]
fn environment_assignment_encodes_one_literal_assignment_and_preserves_it_after_parse_back()
-> Result<(), Box<dyn std::error::Error>> {
    let assignment = EnvironmentAssignment::new("LITERAL_VALUE", "unicode café = \"quoted\" \\ $dollar")?;
    assert_eq!(assignment.name(), "LITERAL_VALUE");
    assert_eq!(assignment.value(), "unicode café = \"quoted\" \\ $dollar");
    assert_eq!(
        assignment.as_str(),
        r#""LITERAL_VALUE=unicode café = \"quoted\" \\ $dollar""#
    );

    let mut builder = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    builder.push_container(ContainerKey::Image, value("example.invalid/application")?)?;
    builder.push_container_environment(assignment)?;
    builder.push_container_environment(EnvironmentAssignment::new("EMPTY", "")?)?;
    let generated = builder.build(SourceId::new(9_200))?;
    assert_eq!(
        generated.text(),
        concat!(
            "[Container]\n",
            "Image=example.invalid/application\n",
            "Environment=\"LITERAL_VALUE=unicode café = \\\"quoted\\\" \\\\ $dollar\"\n",
            "Environment=\"EMPTY=\"\n",
        )
    );
    assert_eq!(
        generated
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::Environment))
            .map(|entry| entry.value().primary().text())
            .collect::<Vec<_>>(),
        [r#""LITERAL_VALUE=unicode café = \"quoted\" \\ $dollar""#, r#""EMPTY=""#]
    );
    Ok(())
}

#[test]
fn environment_assignment_rejects_the_bounded_name_and_value_categories() {
    for name in ["", "1NAME", "NAME-DASH", "NÄME", "NAME SPACE"] {
        assert_eq!(
            EnvironmentAssignment::new(name, "value"),
            Err(EnvironmentAssignmentError::InvalidName),
            "{name:?}"
        );
    }
    for (value, expected) in [
        ("nul\0byte", EnvironmentAssignmentError::Nul),
        ("carriage\rreturn", EnvironmentAssignmentError::CarriageReturn),
        ("line\nfeed", EnvironmentAssignmentError::LineFeed),
        ("tab\tvalue", EnvironmentAssignmentError::ControlCharacter),
        ("delete\u{7f}value", EnvironmentAssignmentError::ControlCharacter),
        ("specifier%h", EnvironmentAssignmentError::Specifier),
    ] {
        assert_eq!(
            EnvironmentAssignment::new("VALID_NAME", value),
            Err(expected),
            "{value:?}"
        );
    }
}

#[test]
fn environment_assignment_groups_render_one_physical_entry_and_preserve_group_order()
-> Result<(), Box<dyn std::error::Error>> {
    let first_group = EnvironmentAssignments::new([
        EnvironmentAssignment::new("FIRST", "space = \"quote\" \\ $dollar café")?,
        EnvironmentAssignment::new("SECOND", "")?,
    ])?;
    assert_eq!(first_group.assignments().len(), 2);
    assert_eq!(
        first_group.iter().map(EnvironmentAssignment::name).collect::<Vec<_>>(),
        ["FIRST", "SECOND"]
    );
    assert_eq!(
        first_group.as_str(),
        r#""FIRST=space = \"quote\" \\ $dollar café" "SECOND=""#
    );
    let raw: EntryValue = first_group.clone().into();
    assert_eq!(raw.as_str(), first_group.as_str());

    let second_group = EnvironmentAssignments::new([EnvironmentAssignment::new("THIRD", "last")?])?;
    let mut builder = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    builder.push_container(ContainerKey::Image, value("example.invalid/application")?)?;
    builder.push_container_environment_assignments(first_group)?;
    builder.push_container_environment(EnvironmentAssignment::new("BETWEEN", "single")?)?;
    builder.push_container_environment_assignments(second_group)?;
    let generated = builder.build(SourceId::new(9_202))?;

    assert_eq!(
        generated.text(),
        concat!(
            "[Container]\n",
            "Image=example.invalid/application\n",
            "Environment=\"FIRST=space = \\\"quote\\\" \\\\ $dollar café\" \"SECOND=\"\n",
            "Environment=\"BETWEEN=single\"\n",
            "Environment=\"THIRD=last\"\n",
        )
    );
    assert_eq!(
        generated
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::Environment))
            .map(|entry| entry.value().primary().text())
            .collect::<Vec<_>>(),
        [
            r#""FIRST=space = \"quote\" \\ $dollar café" "SECOND=""#,
            r#""BETWEEN=single""#,
            r#""THIRD=last""#,
        ]
    );
    Ok(())
}

#[test]
fn environment_assignment_groups_reject_empty_input_and_inherit_assignment_validation() {
    assert_eq!(
        EnvironmentAssignments::new(Vec::<EnvironmentAssignment>::new()),
        Err(EnvironmentAssignmentsError::Empty)
    );
    assert_eq!(
        EnvironmentAssignment::new("INVALID-NAME", "value"),
        Err(EnvironmentAssignmentError::InvalidName)
    );
    assert_eq!(
        EnvironmentAssignment::new("VALID_NAME", "specifier%h"),
        Err(EnvironmentAssignmentError::Specifier)
    );
}

#[test]
fn environment_reset_preserves_its_physical_position_and_parse_back_value() -> Result<(), Box<dyn std::error::Error>> {
    let reset = EnvironmentReset::new();
    assert_eq!(reset, EnvironmentReset::default());
    assert_eq!(reset.as_str(), "");
    let raw: EntryValue = reset.into();
    assert_eq!(raw.as_str(), "");

    let mut builder = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    builder.push_container(ContainerKey::Image, value("example.invalid/application")?)?;
    builder.push_container_environment(EnvironmentAssignment::new("BEFORE", "one")?)?;
    builder.push_container_environment_reset()?;
    builder.push_container_environment_assignments(EnvironmentAssignments::new([
        EnvironmentAssignment::new("AFTER_ONE", "one")?,
        EnvironmentAssignment::new("AFTER_TWO", "two")?,
    ])?)?;
    let generated = builder.build(SourceId::new(9_203))?;

    assert_eq!(
        generated.text(),
        concat!(
            "[Container]\n",
            "Image=example.invalid/application\n",
            "Environment=\"BEFORE=one\"\n",
            "Environment=\n",
            "Environment=\"AFTER_ONE=one\" \"AFTER_TWO=two\"\n",
        )
    );
    assert_eq!(
        generated
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Container(ContainerKey::Environment))
            .map(|entry| entry.value().primary().text())
            .collect::<Vec<_>>(),
        [r#""BEFORE=one""#, "", r#""AFTER_ONE=one" "AFTER_TWO=two""#]
    );

    let mut pod = QuadletDocumentBuilder::new(QuadletUnitType::Pod);
    assert_eq!(
        pod.push_container_environment_reset(),
        Err(RenderError::WrongUnitType {
            document: QuadletUnitType::Pod,
            entry: QuadletUnitType::Container,
        })
    );
    Ok(())
}

#[test]
fn container_environment_plan_preserves_directives_and_projects_effective_literal_values()
-> Result<(), Box<dyn std::error::Error>> {
    let first = EnvironmentAssignment::new("DUPLICATE", "before reset")?;
    let mut plan = ContainerEnvironmentPlan::new();
    assert!(plan.is_empty());
    assert_eq!(plan.len(), 0);
    plan.push_assignment(first.clone());
    plan.push_assignments(EnvironmentAssignments::new([
        EnvironmentAssignment::new("EMPTY", "")?,
        EnvironmentAssignment::new("DUPLICATE", "later in group")?,
    ])?);
    assert_eq!(plan.get("DUPLICATE"), Some("later in group"));
    assert_eq!(plan.get("EMPTY"), Some(""));
    assert!(plan.contains("EMPTY"));
    assert!(!plan.contains("ABSENT"));
    assert_eq!(plan.len(), 2);
    assert!(!plan.is_empty());

    plan.push_reset();
    assert_eq!(plan.get("DUPLICATE"), None);
    assert_eq!(plan.get("EMPTY"), None);
    assert!(plan.is_empty());
    assert_eq!(plan.len(), 0);
    plan.push_assignments(EnvironmentAssignments::new([
        EnvironmentAssignment::new("FINAL", "first after reset")?,
        EnvironmentAssignment::new("FINAL", "last after reset")?,
        EnvironmentAssignment::new("EMPTY_AFTER_RESET", "")?,
    ])?);
    assert_eq!(plan.get("FINAL"), Some("last after reset"));
    assert_eq!(plan.get("EMPTY_AFTER_RESET"), Some(""));
    assert!(plan.contains("EMPTY_AFTER_RESET"));
    assert_eq!(plan.len(), 2);
    assert!(!plan.is_empty());
    assert_eq!(plan.directives().len(), 4);
    assert!(matches!(
        plan.directives(),
        [
            ContainerEnvironmentDirective::Assignment(_),
            ContainerEnvironmentDirective::Assignments(_),
            ContainerEnvironmentDirective::Reset(_),
            ContainerEnvironmentDirective::Assignments(_),
        ]
    ));

    let debug = format!("{plan:?} {first:?}");
    for secret in [
        "before reset",
        "later in group",
        "first after reset",
        "last after reset",
    ] {
        assert!(!debug.contains(secret));
    }

    let mut builder = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    builder.push_container(ContainerKey::Image, value("example.invalid/application")?)?;
    builder.push_container_environment_plan(&plan)?;
    let generated = builder.build(SourceId::new(9_204))?;
    assert_eq!(
        generated.text(),
        concat!(
            "[Container]\n",
            "Image=example.invalid/application\n",
            "Environment=\"DUPLICATE=before reset\"\n",
            "Environment=\"EMPTY=\" \"DUPLICATE=later in group\"\n",
            "Environment=\n",
            "Environment=\"FINAL=first after reset\" \"FINAL=last after reset\" \"EMPTY_AFTER_RESET=\"\n",
        )
    );

    let mut pod = QuadletDocumentBuilder::new(QuadletUnitType::Pod);
    assert_eq!(
        pod.push_container_environment_plan(&plan),
        Err(RenderError::WrongUnitType {
            document: QuadletUnitType::Pod,
            entry: QuadletUnitType::Container,
        })
    );
    Ok(())
}

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
fn build_builder_preserves_repeated_tags_and_files_and_rejects_duplicate_working_directory()
-> Result<(), Box<dyn std::error::Error>> {
    let mut build = QuadletDocumentBuilder::new(QuadletUnitType::Build);
    build.push_build(BuildKey::ImageTag, value("localhost/example:primary")?)?;
    build.push_build(BuildKey::ImageTag, value("localhost/example:secondary")?)?;
    build.push_build(BuildKey::Network, value("host")?)?;
    build.push_build(BuildKey::Network, value("none")?)?;
    build.push_build(BuildKey::Network, value("frontend.network")?)?;
    build.push_build(BuildKey::Label, value("build.label=one")?)?;
    build.push_build(BuildKey::Label, value("empty=")?)?;
    build.push_build(BuildKey::BuildArg, value("KEY=one")?)?;
    build.push_build(BuildKey::BuildArg, value("EMPTY=")?)?;
    build.push_build(BuildKey::BuildArg, value("bare text stays opaque")?)?;
    build.push_build(
        BuildKey::Secret,
        value("id=quadlet-lens-one,src=/run/quadlet-lens-placeholder-one")?,
    )?;
    build.push_build(
        BuildKey::Secret,
        value("id=quadlet-lens-two,src=/run/quadlet-lens-placeholder-two")?,
    )?;
    build.push_build(BuildKey::File, value("Containerfile.first")?)?;
    build.push_build(BuildKey::File, value("")?)?;
    build.push_build(BuildKey::File, value("https://example.invalid/Containerfile?ref=main")?)?;
    build.push_build(BuildKey::Target, value("build-stage")?)?;
    build.push_build(BuildKey::SetWorkingDirectory, value("unit")?)?;
    build.push_build(BuildKey::Arch, value("arm64")?)?;
    build.push_build(BuildKey::Variant, value("v8")?)?;
    build.push_build(BuildKey::Pull, value("always")?)?;
    build.push_build(BuildKey::Retry, value("4")?)?;
    build.push_build(BuildKey::RetryDelay, value("7s")?)?;
    build.push_build(BuildKey::TLSVerify, value("true")?)?;
    build.push_build(BuildKey::ForceRM, value("true")?)?;
    build.push_build(BuildKey::GroupAdd, value("1234")?)?;
    build.push_build(BuildKey::GroupAdd, value("5678")?)?;
    build.push_build(BuildKey::DNS, value("9.9.9.9")?)?;
    build.push_build(BuildKey::DNS, value("2001:4860:4860::8888")?)?;
    build.push_build(BuildKey::DNSOption, value("")?)?;
    build.push_build(BuildKey::DNSOption, value("ndots:1")?)?;
    build.push_build(BuildKey::DNSOption, value("use-vc")?)?;
    build.push_build(BuildKey::DNSSearch, value("")?)?;
    build.push_build(BuildKey::DNSSearch, value("corp.example")?)?;
    build.push_build(BuildKey::DNSSearch, value(".")?)?;
    build.push_build(BuildKey::AuthFile, value("/run/quadlet-lens/auth.json")?)?;
    build.push_build(BuildKey::IgnoreFile, value("./ignored-input")?)?;
    build.push_build(BuildKey::Annotation, value("org.example.build=one")?)?;
    build.push_build(BuildKey::Annotation, value("")?)?;
    build.push_build(BuildKey::Annotation, value("org.example.build=final")?)?;
    build.push_build(
        BuildKey::PodmanArgs,
        value("--build-context extra=container-image://alpine:3.15")?,
    )?;
    build.push_build(BuildKey::PodmanArgs, value("--layers")?)?;
    assert_duplicate_build_singletons(&mut build)?;
    assert_eq!(
        build.build(SourceId::new(180))?.text(),
        concat!(
            "[Build]\n",
            "ImageTag=localhost/example:primary\n",
            "ImageTag=localhost/example:secondary\n",
            "Network=host\n",
            "Network=none\n",
            "Network=frontend.network\n",
            "Label=build.label=one\n",
            "Label=empty=\n",
            "BuildArg=KEY=one\n",
            "BuildArg=EMPTY=\n",
            "BuildArg=bare text stays opaque\n",
            "Secret=id=quadlet-lens-one,src=/run/quadlet-lens-placeholder-one\n",
            "Secret=id=quadlet-lens-two,src=/run/quadlet-lens-placeholder-two\n",
            "File=Containerfile.first\n",
            "File=\n",
            "File=https://example.invalid/Containerfile?ref=main\n",
            "Target=build-stage\n",
            "SetWorkingDirectory=unit\n",
            "Arch=arm64\n",
            "Variant=v8\n",
            "Pull=always\n",
            "Retry=4\n",
            "RetryDelay=7s\n",
            "TLSVerify=true\n",
            "ForceRM=true\n",
            "GroupAdd=1234\n",
            "GroupAdd=5678\n",
            "DNS=9.9.9.9\n",
            "DNS=2001:4860:4860::8888\n",
            "DNSOption=\n",
            "DNSOption=ndots:1\n",
            "DNSOption=use-vc\n",
            "DNSSearch=\n",
            "DNSSearch=corp.example\n",
            "DNSSearch=.\n",
            "AuthFile=/run/quadlet-lens/auth.json\n",
            "IgnoreFile=./ignored-input\n",
            "Annotation=org.example.build=one\n",
            "Annotation=\n",
            "Annotation=org.example.build=final\n",
            "PodmanArgs=--build-context extra=container-image://alpine:3.15\n",
            "PodmanArgs=--layers\n",
        )
    );
    Ok(())
}

fn assert_duplicate_build_singletons(build: &mut QuadletDocumentBuilder) -> Result<(), Box<dyn std::error::Error>> {
    for (key, value_text, expected_key) in [
        (BuildKey::SetWorkingDirectory, "/tmp/other", "SetWorkingDirectory"),
        (BuildKey::Target, "other-stage", "Target"),
        (BuildKey::Arch, "amd64", "Arch"),
        (BuildKey::Variant, "v7", "Variant"),
        (BuildKey::Pull, "never", "Pull"),
        (BuildKey::Retry, "5", "Retry"),
        (BuildKey::RetryDelay, "8s", "RetryDelay"),
        (BuildKey::TLSVerify, "false", "TLSVerify"),
        (BuildKey::ForceRM, "false", "ForceRM"),
        (BuildKey::AuthFile, "/run/quadlet-lens/other-auth.json", "AuthFile"),
        (BuildKey::IgnoreFile, "./other-ignored-input", "IgnoreFile"),
    ] {
        assert!(matches!(
            build.push_build(key, value(value_text)?),
            Err(RenderError::DuplicateSingleton(actual)) if actual == expected_key
        ));
    }
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
fn builds_an_opaque_singleton_pod_exit_policy() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = QuadletDocumentBuilder::new(QuadletUnitType::Pod);
    builder.push_pod(PodKey::ExitPolicy, value("continue")?)?;
    assert!(matches!(
        builder.push_pod(PodKey::ExitPolicy, value("stop")?),
        Err(RenderError::DuplicateSingleton(key)) if key == "ExitPolicy"
    ));
    assert_eq!(
        builder.build(SourceId::new(473))?.text(),
        "[Pod]\nExitPolicy=continue\n"
    );
    Ok(())
}

#[test]
fn builds_an_opaque_singleton_pod_stop_timeout() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = QuadletDocumentBuilder::new(QuadletUnitType::Pod);
    builder.push_pod(PodKey::StopTimeout, value("37")?)?;
    assert!(matches!(
        builder.push_pod(PodKey::StopTimeout, value("0")?),
        Err(RenderError::DuplicateSingleton(key)) if key == "StopTimeout"
    ));
    assert_eq!(builder.build(SourceId::new(476))?.text(), "[Pod]\nStopTimeout=37\n");
    Ok(())
}

#[test]
fn builds_an_opaque_singleton_pod_service_name() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = QuadletDocumentBuilder::new(QuadletUnitType::Pod);
    builder.push_pod(PodKey::ServiceName, value("chosen-name.service")?)?;
    assert!(matches!(
        builder.push_pod(PodKey::ServiceName, value("other")?),
        Err(RenderError::DuplicateSingleton(key)) if key == "ServiceName"
    ));
    assert_eq!(
        builder.build(SourceId::new(479))?.text(),
        "[Pod]\nServiceName=chosen-name.service\n"
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
fn builds_every_typed_systemd_unit_relationship_and_rejects_only_unsafe_physical_values()
-> Result<(), Box<dyn std::error::Error>> {
    let relationships = [
        (SystemdUnitKey::Requires, "Requires"),
        (SystemdUnitKey::Wants, "Wants"),
        (SystemdUnitKey::After, "After"),
        (SystemdUnitKey::Requisite, "Requisite"),
        (SystemdUnitKey::BindsTo, "BindsTo"),
        (SystemdUnitKey::PartOf, "PartOf"),
        (SystemdUnitKey::Upholds, "Upholds"),
        (SystemdUnitKey::Conflicts, "Conflicts"),
        (SystemdUnitKey::Before, "Before"),
    ];
    let mut builder = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    for (key, _) in relationships {
        builder.push_systemd_unit(key, value("alpha.container beta.pod")?)?;
        builder.push_systemd_unit(key, value("")?)?;
    }
    builder.push_container(ContainerKey::Image, value("example.invalid/application")?)?;
    let generated = builder.build(SourceId::new(9_101))?;
    for (_, spelling) in relationships {
        assert_eq!(
            generated
                .text()
                .matches(&format!("{spelling}=alpha.container beta.pod\n"))
                .count(),
            1
        );
        assert_eq!(generated.text().matches(&format!("{spelling}=\n")).count(), 1);
    }
    assert!(matches!(
        EntryValue::new("unsafe\0value"),
        Err(RenderError::InvalidValue)
    ));
    assert!(matches!(
        EntryValue::new("unsafe\nvalue"),
        Err(RenderError::InvalidValue)
    ));
    assert!(matches!(
        EntryValue::new("unsafe\rvalue"),
        Err(RenderError::InvalidValue)
    ));
    for safe in ["", "quoted value", "value\twith-tab", "value%N", "value\\ with-escape"] {
        assert!(
            EntryValue::new(safe).is_ok(),
            "{safe:?} must remain a valid single-line value"
        );
    }
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
fn logging_builder_preserves_opaque_values_and_enforces_cardinality() -> Result<(), Box<dyn std::error::Error>> {
    let mut empty_driver = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    empty_driver.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
    empty_driver.push_container(ContainerKey::LogDriver, value("")?)?;
    assert_eq!(
        empty_driver.build(SourceId::new(299))?.text(),
        "[Container]\nImage=example.invalid/app\nLogDriver=\n"
    );

    let mut container = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    container.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
    container.push_container(ContainerKey::LogDriver, value(r#""Vendor-%n Driver""#)?)?;
    assert!(matches!(
        container.push_container(ContainerKey::LogDriver, value("journald")?),
        Err(RenderError::DuplicateSingleton(key)) if key == "LogDriver"
    ));
    for authored in [
        "path=/var/log/pre.log",
        "",
        "tag=final-%n",
        r#""path=/var/log/Authored Value.log""#,
        "tag=final-%n",
    ] {
        container.push_container(ContainerKey::LogOpt, value(authored)?)?;
    }
    assert_eq!(
        container.build(SourceId::new(300))?.text(),
        concat!(
            "[Container]\n",
            "Image=example.invalid/app\n",
            "LogDriver=\"Vendor-%n Driver\"\n",
            "LogOpt=path=/var/log/pre.log\n",
            "LogOpt=\n",
            "LogOpt=tag=final-%n\n",
            "LogOpt=\"path=/var/log/Authored Value.log\"\n",
            "LogOpt=tag=final-%n\n",
        )
    );

    for unsafe_value in ["tag=one\ntag=two", "tag=one\rtag=two", "tag=one\0tag=two"] {
        assert!(matches!(EntryValue::new(unsafe_value), Err(RenderError::InvalidValue)));
    }
    Ok(())
}

#[test]
fn network_driver_and_options_builder_preserve_raw_values_and_enforce_cardinality()
-> Result<(), Box<dyn std::error::Error>> {
    let mut network = QuadletDocumentBuilder::new(QuadletUnitType::Network);
    network.push_network(NetworkKey::NetworkName, value("example-network")?)?;
    network.push_network(NetworkKey::Driver, value(r#""Vendor-%n Driver""#)?)?;
    assert!(matches!(
        network.push_network(NetworkKey::Driver, value("bridge")?),
        Err(RenderError::DuplicateSingleton(key)) if key == "Driver"
    ));
    network.push_network(NetworkKey::Internal, value("false")?)?;
    assert!(matches!(
        network.push_network(NetworkKey::Internal, value("true")?),
        Err(RenderError::DuplicateSingleton(key)) if key == "Internal"
    ));
    network.push_network(NetworkKey::IPv6, value(r#""Vendor-%n IPv6""#)?)?;
    assert!(matches!(
        network.push_network(NetworkKey::IPv6, value("true")?),
        Err(RenderError::DuplicateSingleton(key)) if key == "IPv6"
    ));
    for authored in [
        "pre=one",
        "pre=two",
        "",
        "zeta=last",
        "alpha=first",
        "alpha=final",
        "bare-token",
        r#""quoted option=%n""#,
        "Vendor_Defined Option Text",
    ] {
        network.push_network(NetworkKey::Options, value(authored)?)?;
    }
    assert_eq!(
        network.build(SourceId::new(302))?.text(),
        concat!(
            "[Network]\n",
            "NetworkName=example-network\n",
            "Driver=\"Vendor-%n Driver\"\n",
            "Internal=false\n",
            "IPv6=\"Vendor-%n IPv6\"\n",
            "Options=pre=one\n",
            "Options=pre=two\n",
            "Options=\n",
            "Options=zeta=last\n",
            "Options=alpha=first\n",
            "Options=alpha=final\n",
            "Options=bare-token\n",
            "Options=\"quoted option=%n\"\n",
            "Options=Vendor_Defined Option Text\n",
        )
    );
    for unsafe_value in ["key=one\nkey=two", "key=one\rkey=two", "key=one\0key=two"] {
        assert!(matches!(EntryValue::new(unsafe_value), Err(RenderError::InvalidValue)));
    }
    Ok(())
}

#[test]
fn network_completion_builder_preserves_opaque_values_and_enforces_singletons() -> Result<(), Box<dyn std::error::Error>>
{
    let mut network = QuadletDocumentBuilder::new(QuadletUnitType::Network);
    network.push_network(NetworkKey::NetworkName, value("completion-network")?)?;
    for (key, authored) in [
        (NetworkKey::ContainersConfModule, "one.conf"),
        (NetworkKey::ContainersConfModule, "two.conf"),
        (NetworkKey::DNS, "9.9.9.9"),
        (NetworkKey::DNS, "2001:4860:4860::8888"),
        (NetworkKey::GlobalArgs, "--log-level=debug"),
        (NetworkKey::PodmanArgs, "--internal"),
        (NetworkKey::PodmanArgs, "--opt isolate=true"),
    ] {
        network.push_network(key, value(authored)?)?;
    }
    for (key, authored) in [
        (NetworkKey::DisableDNS, "true"),
        (NetworkKey::InterfaceName, "quadlet0"),
        (NetworkKey::NetworkDeleteOnStop, "false"),
        (NetworkKey::ServiceName, "completion-network-create"),
    ] {
        network.push_network(key, value(authored)?)?;
        assert!(matches!(
            network.push_network(key, value("other")?),
            Err(RenderError::DuplicateSingleton(_))
        ));
    }
    assert_eq!(
        network.build(SourceId::new(904))?.text(),
        concat!(
            "[Network]\nNetworkName=completion-network\nContainersConfModule=one.conf\n",
            "ContainersConfModule=two.conf\nDNS=9.9.9.9\nDNS=2001:4860:4860::8888\n",
            "GlobalArgs=--log-level=debug\nPodmanArgs=--internal\nPodmanArgs=--opt isolate=true\n",
            "DisableDNS=true\nInterfaceName=quadlet0\nNetworkDeleteOnStop=false\n",
            "ServiceName=completion-network-create\n"
        )
    );
    Ok(())
}

#[test]
fn image_completion_builder_preserves_opaque_values_and_enforces_singletons() -> Result<(), Box<dyn std::error::Error>>
{
    let mut image = QuadletDocumentBuilder::new(QuadletUnitType::Image);
    image.push_image(ImageKey::Image, value("example.invalid/application:1")?)?;
    image.push_image(ImageKey::PodmanArgs, value("--quiet")?)?;
    image.push_image(ImageKey::PodmanArgs, value("--all-tags")?)?;
    for (key, authored) in [
        (ImageKey::Policy, "newer"),
        (ImageKey::Retry, "4"),
        (ImageKey::RetryDelay, "7s"),
        (ImageKey::TLSVerify, "false"),
        (ImageKey::Variant, "v8"),
    ] {
        image.push_image(key, value(authored)?)?;
        assert!(matches!(
            image.push_image(key, value("other")?),
            Err(RenderError::DuplicateSingleton(_))
        ));
    }
    assert_eq!(
        image.build(SourceId::new(905))?.text(),
        concat!(
            "[Image]\nImage=example.invalid/application:1\nPodmanArgs=--quiet\n",
            "PodmanArgs=--all-tags\nPolicy=newer\nRetry=4\nRetryDelay=7s\n",
            "TLSVerify=false\nVariant=v8\n"
        )
    );
    Ok(())
}

#[test]
fn volume_driver_options_device_type_and_copy_builder_preserve_raw_values_and_enforce_cardinality()
-> Result<(), Box<dyn std::error::Error>> {
    let mut volume = QuadletDocumentBuilder::new(QuadletUnitType::Volume);
    volume.push_volume(VolumeKey::VolumeName, value("example-volume")?)?;
    volume.push_volume(VolumeKey::Driver, value(r#""Vendor-%n Driver""#)?)?;
    assert!(matches!(
        volume.push_volume(VolumeKey::Driver, value("local")?),
        Err(RenderError::DuplicateSingleton(key)) if key == "Driver"
    ));
    volume.push_volume(VolumeKey::Options, value(r#"bare,opt=one "matched=%h""#)?)?;
    assert!(matches!(
        volume.push_volume(VolumeKey::Options, value("o=second")?),
        Err(RenderError::DuplicateSingleton(key)) if key == "Options"
    ));
    volume.push_volume(VolumeKey::Device, value(r#""/srv/%h source""#)?)?;
    assert!(matches!(
        volume.push_volume(VolumeKey::Device, value("/srv/other")?),
        Err(RenderError::DuplicateSingleton(key)) if key == "Device"
    ));
    volume.push_volume(VolumeKey::Type, value(r#""bind %h""#)?)?;
    assert!(matches!(
        volume.push_volume(VolumeKey::Type, value("tmpfs")?),
        Err(RenderError::DuplicateSingleton(key)) if key == "Type"
    ));
    volume.push_volume(VolumeKey::Copy, value(r#""TrUe %h""#)?)?;
    assert!(matches!(
        volume.push_volume(VolumeKey::Copy, value("false")?),
        Err(RenderError::DuplicateSingleton(key)) if key == "Copy"
    ));
    assert_eq!(
        volume.build(SourceId::new(304))?.text(),
        concat!(
            "[Volume]\n",
            "VolumeName=example-volume\n",
            "Driver=\"Vendor-%n Driver\"\n",
            "Options=bare,opt=one \"matched=%h\"\n",
            "Device=\"/srv/%h source\"\n",
            "Type=\"bind %h\"\n",
            "Copy=\"TrUe %h\"\n",
        )
    );
    for unsafe_value in ["o=one\no=two", "o=one\ro=two", "o=one\0o=two"] {
        assert!(matches!(EntryValue::new(unsafe_value), Err(RenderError::InvalidValue)));
    }
    Ok(())
}

#[test]
fn volume_label_builder_preserves_raw_repeatable_values() -> Result<(), Box<dyn std::error::Error>> {
    let mut volume = QuadletDocumentBuilder::new(QuadletUnitType::Volume);
    for authored in [
        "pre=one",
        "",
        "alpha=first",
        "alpha=final",
        "empty=",
        "embedded=a=b",
        "bare-token",
        r#""quoted=%h value""#,
    ] {
        volume.push_volume(VolumeKey::Label, value(authored)?)?;
    }
    assert_eq!(
        volume.build(SourceId::new(305))?.text(),
        concat!(
            "[Volume]\n",
            "Label=pre=one\n",
            "Label=\n",
            "Label=alpha=first\n",
            "Label=alpha=final\n",
            "Label=empty=\n",
            "Label=embedded=a=b\n",
            "Label=bare-token\n",
            "Label=\"quoted=%h value\"\n",
        )
    );
    for unsafe_value in ["key=one\nkey=two", "key=one\rkey=two", "key=one\0key=two"] {
        assert!(matches!(EntryValue::new(unsafe_value), Err(RenderError::InvalidValue)));
    }
    Ok(())
}

#[test]
fn volume_containers_conf_module_builder_preserves_repeatable_opaque_values() -> Result<(), Box<dyn std::error::Error>>
{
    let mut volume = QuadletDocumentBuilder::new(QuadletUnitType::Volume);
    volume.push_volume(VolumeKey::VolumeName, value("quadlet-lens-module")?)?;
    for authored in [
        "pre-one",
        "",
        " post one ",
        "post-two",
        "post-two",
        r#""quoted %h module""#,
        r"module\x20text",
        r"continuation\-looking",
    ] {
        volume.push_volume(VolumeKey::ContainersConfModule, value(authored)?)?;
    }
    assert_eq!(
        volume.build(SourceId::new(316))?.text(),
        concat!(
            "[Volume]\n",
            "VolumeName=quadlet-lens-module\n",
            "ContainersConfModule=pre-one\n",
            "ContainersConfModule=\n",
            "ContainersConfModule= post one \n",
            "ContainersConfModule=post-two\n",
            "ContainersConfModule=post-two\n",
            "ContainersConfModule=\"quoted %h module\"\n",
            "ContainersConfModule=module\\x20text\n",
            "ContainersConfModule=continuation\\-looking\n",
        )
    );
    Ok(())
}

#[test]
fn volume_global_args_builder_preserves_repeatable_opaque_values() -> Result<(), Box<dyn std::error::Error>> {
    let mut volume = QuadletDocumentBuilder::new(QuadletUnitType::Volume);
    volume.push_volume(VolumeKey::VolumeName, value("quadlet-lens-global-args")?)?;
    for authored in [
        "pre-one",
        "",
        "--log-level=debug",
        r#""--events-backend=none""#,
        r"--events-backend=file\x20value",
        r"malformed \ value",
    ] {
        volume.push_volume(VolumeKey::GlobalArgs, value(authored)?)?;
    }
    assert_eq!(
        volume.build(SourceId::new(317))?.text(),
        concat!(
            "[Volume]\n",
            "VolumeName=quadlet-lens-global-args\n",
            "GlobalArgs=pre-one\n",
            "GlobalArgs=\n",
            "GlobalArgs=--log-level=debug\n",
            "GlobalArgs=\"--events-backend=none\"\n",
            "GlobalArgs=--events-backend=file\\x20value\n",
            "GlobalArgs=malformed \\ value\n",
        )
    );
    Ok(())
}

#[test]
fn volume_podman_args_builder_preserves_repeatable_opaque_values() -> Result<(), Box<dyn std::error::Error>> {
    let mut volume = QuadletDocumentBuilder::new(QuadletUnitType::Volume);
    volume.push_volume(VolumeKey::VolumeName, value("quadlet-lens-podman-args")?)?;
    for authored in [
        "pre-one",
        "",
        "--label=post-one",
        r#""--label=quoted value""#,
        r"--label\x3descaped",
        r"malformed \ value",
    ] {
        volume.push_volume(VolumeKey::PodmanArgs, value(authored)?)?;
    }
    assert_eq!(
        volume.build(SourceId::new(318))?.text(),
        concat!(
            "[Volume]\n",
            "VolumeName=quadlet-lens-podman-args\n",
            "PodmanArgs=pre-one\n",
            "PodmanArgs=\n",
            "PodmanArgs=--label=post-one\n",
            "PodmanArgs=\"--label=quoted value\"\n",
            "PodmanArgs=--label\\x3descaped\n",
            "PodmanArgs=malformed \\ value\n",
        )
    );
    Ok(())
}

#[test]
fn volume_user_builder_preserves_one_opaque_value_and_rejects_duplicates() -> Result<(), Box<dyn std::error::Error>> {
    let mut volume = QuadletDocumentBuilder::new(QuadletUnitType::Volume);
    volume.push_volume(VolumeKey::VolumeName, value("quadlet-lens-user")?)?;
    volume.push_volume(VolumeKey::User, value("007")?)?;
    assert_eq!(
        volume.build(SourceId::new(319))?.text(),
        "[Volume]\nVolumeName=quadlet-lens-user\nUser=007\n"
    );

    let mut duplicate = QuadletDocumentBuilder::new(QuadletUnitType::Volume);
    duplicate.push_volume(VolumeKey::User, value("")?)?;
    assert!(matches!(
        duplicate.push_volume(VolumeKey::User, value("alice")?),
        Err(RenderError::DuplicateSingleton { .. })
    ));
    Ok(())
}

#[test]
fn volume_group_builder_preserves_one_opaque_value_and_rejects_duplicates() -> Result<(), Box<dyn std::error::Error>> {
    let mut volume = QuadletDocumentBuilder::new(QuadletUnitType::Volume);
    volume.push_volume(VolumeKey::VolumeName, value("quadlet-lens-group")?)?;
    volume.push_volume(VolumeKey::Group, value("00456")?)?;
    assert_eq!(
        volume.build(SourceId::new(320))?.text(),
        "[Volume]\nVolumeName=quadlet-lens-group\nGroup=00456\n"
    );
    let mut duplicate = QuadletDocumentBuilder::new(QuadletUnitType::Volume);
    duplicate.push_volume(VolumeKey::Group, value("")?)?;
    assert!(matches!(
        duplicate.push_volume(VolumeKey::Group, value("operators")?),
        Err(RenderError::DuplicateSingleton { .. })
    ));
    Ok(())
}

#[test]
fn volume_uid_builder_preserves_one_opaque_value_and_rejects_duplicates() -> Result<(), Box<dyn std::error::Error>> {
    let mut volume = QuadletDocumentBuilder::new(QuadletUnitType::Volume);
    volume.push_volume(VolumeKey::UID, value("001234")?)?;
    assert_eq!(volume.build(SourceId::new(321))?.text(), "[Volume]\nUID=001234\n");
    let mut duplicate = QuadletDocumentBuilder::new(QuadletUnitType::Volume);
    duplicate.push_volume(VolumeKey::UID, value("")?)?;
    assert!(matches!(
        duplicate.push_volume(VolumeKey::UID, value("1234")?),
        Err(RenderError::DuplicateSingleton { .. })
    ));
    Ok(())
}

#[test]
fn volume_gid_builder_preserves_one_opaque_value_and_rejects_duplicates() -> Result<(), Box<dyn std::error::Error>> {
    for authored in ["", "005678", "group", "\"quoted-%i\"", "%h/gid", "continued \\ text"] {
        let mut volume = QuadletDocumentBuilder::new(QuadletUnitType::Volume);
        volume.push_volume(VolumeKey::GID, value(authored)?)?;
        assert_eq!(
            volume.build(SourceId::new(322))?.text(),
            format!("[Volume]\nGID={authored}\n")
        );
    }
    let mut duplicate = QuadletDocumentBuilder::new(QuadletUnitType::Volume);
    duplicate.push_volume(VolumeKey::GID, value("")?)?;
    assert!(matches!(
        duplicate.push_volume(VolumeKey::GID, value("5678")?),
        Err(RenderError::DuplicateSingleton { .. })
    ));
    Ok(())
}

#[test]
fn volume_service_name_builder_preserves_one_opaque_value_and_rejects_duplicates()
-> Result<(), Box<dyn std::error::Error>> {
    for authored in [
        "",
        "ordinary",
        " explicit ",
        "\"quoted-%i\"",
        "%i",
        "escape\\x20text",
        "continued \\ text",
    ] {
        let mut volume = QuadletDocumentBuilder::new(QuadletUnitType::Volume);
        volume.push_volume(VolumeKey::ServiceName, value(authored)?)?;
        assert_eq!(
            volume.build(SourceId::new(323))?.text(),
            format!("[Volume]\nServiceName={authored}\n")
        );
    }
    let mut duplicate = QuadletDocumentBuilder::new(QuadletUnitType::Volume);
    duplicate.push_volume(VolumeKey::ServiceName, value("first.service")?)?;
    assert!(matches!(
        duplicate.push_volume(VolumeKey::ServiceName, value("second.service")?),
        Err(RenderError::DuplicateSingleton { .. })
    ));
    Ok(())
}

#[test]
fn volume_image_builder_preserves_one_value_and_rejects_duplicates() -> Result<(), Box<dyn std::error::Error>> {
    for authored in [
        "",
        "literal.example/image:1",
        "unit.image",
        "unit.build",
        "continued \\ text",
    ] {
        let mut volume = QuadletDocumentBuilder::new(QuadletUnitType::Volume);
        volume.push_volume(VolumeKey::Image, value(authored)?)?;
        assert_eq!(
            volume.build(SourceId::new(324))?.text(),
            format!("[Volume]\nImage={authored}\n")
        );
    }
    let mut duplicate = QuadletDocumentBuilder::new(QuadletUnitType::Volume);
    duplicate.push_volume(VolumeKey::Image, value("first.image")?)?;
    assert!(matches!(
        duplicate.push_volume(VolumeKey::Image, value("second.build")?),
        Err(RenderError::DuplicateSingleton { .. })
    ));
    Ok(())
}

#[test]
fn image_builder_requires_one_nonblank_opaque_source() -> Result<(), Box<dyn std::error::Error>> {
    let mut image = QuadletDocumentBuilder::new(QuadletUnitType::Image);
    image.push_image(ImageKey::Image, value("example.invalid/application:1")?)?;
    assert_eq!(
        image.build(SourceId::new(442))?.text(),
        "[Image]\nImage=example.invalid/application:1\n"
    );
    assert!(matches!(
        image.push_image(ImageKey::Image, value("other")?),
        Err(RenderError::DuplicateSingleton(_))
    ));
    let image = QuadletDocumentBuilder::new(QuadletUnitType::Image);
    assert!(matches!(
        image.build(SourceId::new(443)),
        Err(RenderError::InvalidDocument(_))
    ));
    let mut blank = QuadletDocumentBuilder::new(QuadletUnitType::Image);
    blank.push_image(ImageKey::Image, value(" ")?)?;
    assert!(matches!(
        blank.build(SourceId::new(444)),
        Err(RenderError::InvalidDocument(_))
    ));
    Ok(())
}

#[test]
fn artifact_builder_requires_a_final_source_preserves_opaque_entries_and_redacts_sensitive_debug()
-> Result<(), Box<dyn std::error::Error>> {
    const CREDS: &str = "quadlet-lens-artifact-creds-canary:user-password";
    const KEY: &str = "quadlet-lens-artifact-key-canary";
    let mut artifact = QuadletDocumentBuilder::new(QuadletUnitType::Artifact);
    artifact.push_quadlet(QuadletKey::DefaultDependencies, value("false")?)?;
    artifact.push_artifact(ArtifactKey::Artifact, value("registry.invalid/example/artifact:1")?)?;
    artifact.push_artifact(ArtifactKey::AuthFile, value("/run/quadlet-lens/auth.json")?)?;
    artifact.push_artifact(ArtifactKey::CertDir, value("/run/quadlet-lens/certs")?)?;
    artifact.push_artifact(ArtifactKey::Creds, value(CREDS)?)?;
    artifact.push_artifact(ArtifactKey::DecryptionKey, value(KEY)?)?;
    artifact.push_artifact(ArtifactKey::Quiet, value("true")?)?;
    artifact.push_artifact(ArtifactKey::Retry, value("4")?)?;
    artifact.push_artifact(ArtifactKey::RetryDelay, value("7s")?)?;
    artifact.push_artifact(ArtifactKey::ServiceName, value("artifact-pull")?)?;
    artifact.push_artifact(ArtifactKey::TLSVerify, value("false")?)?;
    for module in ["pre.conf", "", "post-one.conf", "post-two.conf"] {
        artifact.push_artifact(ArtifactKey::ContainersConfModule, value(module)?)?;
    }
    for argument in ["--log-level=debug", "", "--events-backend=file"] {
        artifact.push_artifact(ArtifactKey::GlobalArgs, value(argument)?)?;
    }
    for argument in ["--pre", "", "--post"] {
        artifact.push_artifact(ArtifactKey::PodmanArgs, value(argument)?)?;
    }
    let generated = artifact.build(SourceId::new(4_601))?;
    assert_eq!(
        generated.text(),
        concat!(
            "[Quadlet]\nDefaultDependencies=false\n\n",
            "[Artifact]\nArtifact=registry.invalid/example/artifact:1\n",
            "AuthFile=/run/quadlet-lens/auth.json\nCertDir=/run/quadlet-lens/certs\n",
            "Creds=quadlet-lens-artifact-creds-canary:user-password\n",
            "DecryptionKey=quadlet-lens-artifact-key-canary\nQuiet=true\nRetry=4\n",
            "RetryDelay=7s\nServiceName=artifact-pull\nTLSVerify=false\n",
            "ContainersConfModule=pre.conf\nContainersConfModule=\n",
            "ContainersConfModule=post-one.conf\nContainersConfModule=post-two.conf\n",
            "GlobalArgs=--log-level=debug\nGlobalArgs=\nGlobalArgs=--events-backend=file\n",
            "PodmanArgs=--pre\nPodmanArgs=\nPodmanArgs=--post\n",
        )
    );
    assert!(
        generated
            .document()
            .entries()
            .any(quadlet_lens::model::TypedEntry::is_sensitive)
    );
    let debug = format!("{artifact:#?}");
    assert!(!debug.contains(CREDS) && !debug.contains(KEY));

    for key in [
        ArtifactKey::Artifact,
        ArtifactKey::AuthFile,
        ArtifactKey::CertDir,
        ArtifactKey::Creds,
        ArtifactKey::DecryptionKey,
        ArtifactKey::Quiet,
        ArtifactKey::Retry,
        ArtifactKey::RetryDelay,
        ArtifactKey::ServiceName,
        ArtifactKey::TLSVerify,
    ] {
        let mut duplicate = QuadletDocumentBuilder::new(QuadletUnitType::Artifact);
        duplicate.push_artifact(ArtifactKey::Artifact, value("registry.invalid/example/artifact:1")?)?;
        if key != ArtifactKey::Artifact {
            duplicate.push_artifact(key, value("first")?)?;
        }
        assert!(matches!(
            duplicate.push_artifact(key, value("second")?),
            Err(RenderError::DuplicateSingleton(_))
        ));
    }
    assert!(matches!(
        QuadletDocumentBuilder::new(QuadletUnitType::Artifact).build(SourceId::new(4_602)),
        Err(RenderError::InvalidDocument(_))
    ));
    let mut blank = QuadletDocumentBuilder::new(QuadletUnitType::Artifact);
    blank.push_artifact(ArtifactKey::Artifact, value(" ")?)?;
    assert!(matches!(
        blank.build(SourceId::new(4_603)),
        Err(RenderError::InvalidDocument(_))
    ));
    Ok(())
}

#[test]
fn quadlet_default_dependencies_builder_retains_raw_values_and_rejects_duplicates()
-> Result<(), Box<dyn std::error::Error>> {
    for value_text in ["true", "false", "not-a-boolean", ""] {
        let mut document = QuadletDocumentBuilder::new(QuadletUnitType::Artifact);
        document.push_quadlet(QuadletKey::DefaultDependencies, value(value_text)?)?;
        document.push_artifact(ArtifactKey::Artifact, value("registry.invalid/example/artifact:1")?)?;
        assert_eq!(
            document.build(SourceId::new(4_610))?.text(),
            format!(
                "[Quadlet]\nDefaultDependencies={value_text}\n\n[Artifact]\nArtifact=registry.invalid/example/artifact:1\n"
            )
        );
    }
    let mut document = QuadletDocumentBuilder::new(QuadletUnitType::Artifact);
    document.push_quadlet(QuadletKey::DefaultDependencies, value("true")?)?;
    assert!(matches!(
        document.push_quadlet(QuadletKey::DefaultDependencies, value("false")?),
        Err(RenderError::DuplicateSingleton(_))
    ));
    Ok(())
}

#[test]
fn image_tag_builder_preserves_one_opaque_value_including_blank_and_rejects_duplicates()
-> Result<(), Box<dyn std::error::Error>> {
    for authored in ["", "localhost/application:tag", "\"quoted-%i\"", "continued \\ text"] {
        let mut image = QuadletDocumentBuilder::new(QuadletUnitType::Image);
        image.push_image(ImageKey::Image, value("example.invalid/application:1")?)?;
        image.push_image(ImageKey::ImageTag, value(authored)?)?;
        assert_eq!(
            image.build(SourceId::new(448))?.text(),
            format!("[Image]\nImage=example.invalid/application:1\nImageTag={authored}\n")
        );
    }
    let mut duplicate = QuadletDocumentBuilder::new(QuadletUnitType::Image);
    duplicate.push_image(ImageKey::ImageTag, value("first")?)?;
    assert!(matches!(
        duplicate.push_image(ImageKey::ImageTag, value("second")?),
        Err(RenderError::DuplicateSingleton(_))
    ));
    Ok(())
}

#[test]
fn image_service_name_builder_preserves_one_opaque_value_including_blank_and_rejects_duplicates()
-> Result<(), Box<dyn std::error::Error>> {
    for authored in ["", "custom.service", "\"quoted-%i\"", "continued \\ text"] {
        let mut image = QuadletDocumentBuilder::new(QuadletUnitType::Image);
        image.push_image(ImageKey::Image, value("example.invalid/application:1")?)?;
        image.push_image(ImageKey::ServiceName, value(authored)?)?;
        assert_eq!(
            image.build(SourceId::new(452))?.text(),
            format!("[Image]\nImage=example.invalid/application:1\nServiceName={authored}\n")
        );
    }
    let mut duplicate = QuadletDocumentBuilder::new(QuadletUnitType::Image);
    duplicate.push_image(ImageKey::ServiceName, value("first")?)?;
    assert!(matches!(
        duplicate.push_image(ImageKey::ServiceName, value("second")?),
        Err(RenderError::DuplicateSingleton(_))
    ));
    Ok(())
}

#[test]
fn image_all_tags_builder_preserves_one_opaque_value_including_blank_and_rejects_duplicates()
-> Result<(), Box<dyn std::error::Error>> {
    for authored in ["", "true", "false", "\"quoted-%i\"", "continued \\ text"] {
        let mut image = QuadletDocumentBuilder::new(QuadletUnitType::Image);
        image.push_image(ImageKey::Image, value("example.invalid/application:1")?)?;
        image.push_image(ImageKey::AllTags, value(authored)?)?;
        assert_eq!(
            image.build(SourceId::new(453))?.text(),
            format!("[Image]\nImage=example.invalid/application:1\nAllTags={authored}\n")
        );
    }
    let mut duplicate = QuadletDocumentBuilder::new(QuadletUnitType::Image);
    duplicate.push_image(ImageKey::AllTags, value("true")?)?;
    assert!(matches!(
        duplicate.push_image(ImageKey::AllTags, value("false")?),
        Err(RenderError::DuplicateSingleton(_))
    ));
    Ok(())
}

#[test]
fn image_arch_builder_preserves_one_opaque_value_including_blank_and_rejects_duplicates()
-> Result<(), Box<dyn std::error::Error>> {
    for authored in ["", "arm64", "\"quoted-%i\"", "continued \\ text"] {
        let mut image = QuadletDocumentBuilder::new(QuadletUnitType::Image);
        image.push_image(ImageKey::Image, value("example.invalid/application:1")?)?;
        image.push_image(ImageKey::Arch, value(authored)?)?;
        assert_eq!(
            image.build(SourceId::new(454))?.text(),
            format!("[Image]\nImage=example.invalid/application:1\nArch={authored}\n")
        );
    }
    let mut duplicate = QuadletDocumentBuilder::new(QuadletUnitType::Image);
    duplicate.push_image(ImageKey::Arch, value("arm64")?)?;
    assert!(matches!(
        duplicate.push_image(ImageKey::Arch, value("amd64")?),
        Err(RenderError::DuplicateSingleton(_))
    ));
    Ok(())
}

#[test]
fn image_auth_file_builder_preserves_one_opaque_value_including_blank_and_rejects_duplicates()
-> Result<(), Box<dyn std::error::Error>> {
    for authored in [
        "",
        "/placeholder/quadlet-lens-auth.json",
        "\"quoted-%i\"",
        "unmatched\"",
        "continued \\ text",
    ] {
        let mut image = QuadletDocumentBuilder::new(QuadletUnitType::Image);
        image.push_image(ImageKey::Image, value("example.invalid/application:1")?)?;
        image.push_image(ImageKey::AuthFile, value(authored)?)?;
        assert_eq!(
            image.build(SourceId::new(455))?.text(),
            format!("[Image]\nImage=example.invalid/application:1\nAuthFile={authored}\n")
        );
    }
    let mut duplicate = QuadletDocumentBuilder::new(QuadletUnitType::Image);
    duplicate.push_image(ImageKey::AuthFile, value("/placeholder/first-auth.json")?)?;
    assert!(matches!(
        duplicate.push_image(ImageKey::AuthFile, value("/placeholder/second-auth.json")?),
        Err(RenderError::DuplicateSingleton(_))
    ));
    Ok(())
}

#[test]
fn image_cert_dir_builder_preserves_one_opaque_value_including_blank_and_rejects_duplicates()
-> Result<(), Box<dyn std::error::Error>> {
    for authored in [
        "",
        "/placeholder/quadlet-lens-certs",
        "\"quoted-%i\"",
        "unmatched\"",
        "continued \\ text",
    ] {
        let mut image = QuadletDocumentBuilder::new(QuadletUnitType::Image);
        image.push_image(ImageKey::Image, value("example.invalid/application:1")?)?;
        image.push_image(ImageKey::CertDir, value(authored)?)?;
        assert_eq!(
            image.build(SourceId::new(456))?.text(),
            format!("[Image]\nImage=example.invalid/application:1\nCertDir={authored}\n")
        );
    }
    let mut duplicate = QuadletDocumentBuilder::new(QuadletUnitType::Image);
    duplicate.push_image(ImageKey::CertDir, value("/placeholder/first-certs")?)?;
    assert!(matches!(
        duplicate.push_image(ImageKey::CertDir, value("/placeholder/second-certs")?),
        Err(RenderError::DuplicateSingleton(_))
    ));
    Ok(())
}

#[test]
fn image_containers_conf_module_builder_preserves_repeatable_raw_values_and_parse_back()
-> Result<(), Box<dyn std::error::Error>> {
    let mut image = QuadletDocumentBuilder::new(QuadletUnitType::Image);
    image.push_image(ImageKey::Image, value("example.invalid/application:1")?)?;
    for authored in [
        "pre-one",
        "",
        " post one ",
        "post-two",
        "post-two",
        "\"quoted %h module\"",
        r"module\x20text",
        "-leading-dash",
    ] {
        image.push_image(ImageKey::ContainersConfModule, value(authored)?)?;
    }
    let generated = image.build(SourceId::new(457))?;
    assert_eq!(
        generated.text(),
        concat!(
            "[Image]\nImage=example.invalid/application:1\n",
            "ContainersConfModule=pre-one\nContainersConfModule=\n",
            "ContainersConfModule= post one \nContainersConfModule=post-two\n",
            "ContainersConfModule=post-two\nContainersConfModule=\"quoted %h module\"\n",
            "ContainersConfModule=module\\x20text\nContainersConfModule=-leading-dash\n",
        )
    );
    let parsed = QuadletDocument::parse(QuadletUnitType::Image, SourceId::new(458), generated.text())?;
    assert_eq!(
        parsed
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Image(ImageKey::ContainersConfModule))
            .map(|entry| (entry.value().primary().text(), entry.value_kind()))
            .collect::<Vec<_>>(),
        [
            ("pre-one", ValueKind::Opaque),
            ("", ValueKind::Opaque),
            ("post one ", ValueKind::Opaque),
            ("post-two", ValueKind::Opaque),
            ("post-two", ValueKind::Opaque),
            ("\"quoted %h module\"", ValueKind::Opaque),
            (r"module\x20text", ValueKind::Opaque),
            ("-leading-dash", ValueKind::Opaque),
        ]
    );
    Ok(())
}

#[test]
fn image_creds_builder_preserves_one_opaque_value_and_redacts_builder_debug() -> Result<(), Box<dyn std::error::Error>>
{
    const PLACEHOLDER: &str = "quadlet-lens-creds-debug-placeholder-7e9c:opaque-password";
    for authored in ["", PLACEHOLDER, "\"quoted-%i\"", "%h/creds", "continued \\ text"] {
        let mut image = QuadletDocumentBuilder::new(QuadletUnitType::Image);
        image.push_image(ImageKey::Image, value("example.invalid/application:1")?)?;
        image.push_image(ImageKey::Creds, value(authored)?)?;
        assert_eq!(
            image.build(SourceId::new(459))?.text(),
            format!("[Image]\nImage=example.invalid/application:1\nCreds={authored}\n")
        );
        assert!(!format!("{image:#?}").contains(PLACEHOLDER));
    }
    let mut duplicate = QuadletDocumentBuilder::new(QuadletUnitType::Image);
    duplicate.push_image(ImageKey::Image, value("example.invalid/application:1")?)?;
    duplicate.push_image(ImageKey::Creds, value(PLACEHOLDER)?)?;
    assert!(matches!(
        duplicate.push_image(ImageKey::Creds, value("second")?),
        Err(RenderError::DuplicateSingleton(_))
    ));
    Ok(())
}

#[test]
fn image_decryption_key_builder_preserves_one_opaque_value_and_redacts_builder_debug()
-> Result<(), Box<dyn std::error::Error>> {
    const PLACEHOLDER: &str = "quadlet-lens-decryption-key-debug-placeholder-7e9c";
    for authored in [
        "",
        PLACEHOLDER,
        "\"quoted-%i\"",
        "%h/decryption-key",
        "continued \\ text",
    ] {
        let mut image = QuadletDocumentBuilder::new(QuadletUnitType::Image);
        image.push_image(ImageKey::Image, value("example.invalid/application:1")?)?;
        image.push_image(ImageKey::DecryptionKey, value(authored)?)?;
        assert_eq!(
            image.build(SourceId::new(460))?.text(),
            format!("[Image]\nImage=example.invalid/application:1\nDecryptionKey={authored}\n")
        );
        assert!(!format!("{image:#?}").contains(PLACEHOLDER));
    }
    let mut duplicate = QuadletDocumentBuilder::new(QuadletUnitType::Image);
    duplicate.push_image(ImageKey::Image, value("example.invalid/application:1")?)?;
    duplicate.push_image(ImageKey::DecryptionKey, value(PLACEHOLDER)?)?;
    assert!(matches!(
        duplicate.push_image(ImageKey::DecryptionKey, value("second")?),
        Err(RenderError::DuplicateSingleton(_))
    ));
    Ok(())
}

#[test]
fn image_global_args_builder_preserves_unlimited_raw_physical_values() -> Result<(), Box<dyn std::error::Error>> {
    let mut image = QuadletDocumentBuilder::new(QuadletUnitType::Image);
    image.push_image(ImageKey::Image, value("example.invalid/application:1")?)?;
    for authored in [
        "pre-one",
        "pre-one",
        "",
        " --log-level=debug ",
        "\"--events-backend=none\"",
        r"--events-backend\x3dfile\x20value",
        r"continuation\-looking",
        " malformed global argument ",
    ] {
        image.push_image(ImageKey::GlobalArgs, value(authored)?)?;
    }
    assert_eq!(
        image.build(SourceId::new(461))?.text(),
        concat!(
            "[Image]\nImage=example.invalid/application:1\n",
            "GlobalArgs=pre-one\nGlobalArgs=pre-one\nGlobalArgs=\n",
            "GlobalArgs= --log-level=debug \nGlobalArgs=\"--events-backend=none\"\n",
            "GlobalArgs=--events-backend\\x3dfile\\x20value\n",
            "GlobalArgs=continuation\\-looking\nGlobalArgs= malformed global argument \n",
        )
    );
    Ok(())
}

#[test]
fn image_os_builder_preserves_one_opaque_value_and_rejects_duplicates() -> Result<(), Box<dyn std::error::Error>> {
    for authored in [
        "",
        "windows",
        "\"quoted-%i\"",
        "unmatched\"",
        "continued \\ text",
        "%h/os",
    ] {
        let mut image = QuadletDocumentBuilder::new(QuadletUnitType::Image);
        image.push_image(ImageKey::Image, value("example.invalid/application:1")?)?;
        image.push_image(ImageKey::OS, value(authored)?)?;
        assert_eq!(
            image.build(SourceId::new(462))?.text(),
            format!("[Image]\nImage=example.invalid/application:1\nOS={authored}\n")
        );
    }
    let mut duplicate = QuadletDocumentBuilder::new(QuadletUnitType::Image);
    duplicate.push_image(ImageKey::Image, value("example.invalid/application:1")?)?;
    duplicate.push_image(ImageKey::OS, value("windows")?)?;
    assert!(matches!(
        duplicate.push_image(ImageKey::OS, value("linux")?),
        Err(RenderError::DuplicateSingleton(_))
    ));
    Ok(())
}

#[test]
fn network_label_builder_preserves_raw_repeatable_values() -> Result<(), Box<dyn std::error::Error>> {
    let mut network = QuadletDocumentBuilder::new(QuadletUnitType::Network);
    for authored in [
        "pre=one",
        "",
        "alpha=first",
        "alpha=final",
        "empty=",
        "embedded=a=b",
        "bare-token",
        r#""quoted=%h value""#,
    ] {
        network.push_network(NetworkKey::Label, value(authored)?)?;
    }
    assert_eq!(
        network.build(SourceId::new(304))?.text(),
        concat!(
            "[Network]\n",
            "Label=pre=one\n",
            "Label=\n",
            "Label=alpha=first\n",
            "Label=alpha=final\n",
            "Label=empty=\n",
            "Label=embedded=a=b\n",
            "Label=bare-token\n",
            "Label=\"quoted=%h value\"\n",
        )
    );
    for unsafe_value in ["key=one\nkey=two", "key=one\rkey=two", "key=one\0key=two"] {
        assert!(matches!(EntryValue::new(unsafe_value), Err(RenderError::InvalidValue)));
    }
    Ok(())
}

#[test]
fn network_identity_builder_preserves_opaque_values_and_enforces_cardinality() -> Result<(), Box<dyn std::error::Error>>
{
    let mut container = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    container.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
    container.push_container(ContainerKey::IP, value(r#""192.0.2.%n""#)?)?;
    assert!(matches!(
        container.push_container(ContainerKey::IP, value("192.0.2.10")?),
        Err(RenderError::DuplicateSingleton(key)) if key == "IP"
    ));
    container.push_container(ContainerKey::IP6, value("2001:db8::%i")?)?;
    assert!(matches!(
        container.push_container(ContainerKey::IP6, value("2001:db8::10")?),
        Err(RenderError::DuplicateSingleton(key)) if key == "IP6"
    ));
    for authored in ["pre.example", "", r#""final %n""#, "final-%i", r#""final %n""#] {
        container.push_container(ContainerKey::NetworkAlias, value(authored)?)?;
    }
    assert_eq!(
        container.build(SourceId::new(301))?.text(),
        concat!(
            "[Container]\n",
            "Image=example.invalid/app\n",
            "IP=\"192.0.2.%n\"\n",
            "IP6=2001:db8::%i\n",
            "NetworkAlias=pre.example\n",
            "NetworkAlias=\n",
            "NetworkAlias=\"final %n\"\n",
            "NetworkAlias=final-%i\n",
            "NetworkAlias=\"final %n\"\n",
        )
    );

    for unsafe_value in ["alias-one\nalias-two", "alias-one\ralias-two", "alias-one\0alias-two"] {
        assert!(matches!(EntryValue::new(unsafe_value), Err(RenderError::InvalidValue)));
    }
    Ok(())
}

#[test]
fn network_ipam_builder_preserves_raw_columns_and_rejects_duplicate_driver() -> Result<(), Box<dyn std::error::Error>> {
    let mut network = QuadletDocumentBuilder::new(QuadletUnitType::Network);
    network.push_network(NetworkKey::NetworkName, value("ipam-network")?)?;
    network.push_network(NetworkKey::IPAMDriver, value(r#""host-local-%n""#)?)?;
    assert!(matches!(
        network.push_network(NetworkKey::IPAMDriver, value("dhcp")?),
        Err(RenderError::DuplicateSingleton(key)) if key == "IPAMDriver"
    ));
    for (key, values) in [
        (
            NetworkKey::Subnet,
            ["pre-subnet", "", r#""10.88.0.0/24""#, "10.89.0.0/24"],
        ),
        (NetworkKey::Gateway, ["pre-gateway", "", r#""10.88.0.1""#, "10.89.0.1"]),
        (
            NetworkKey::IPRange,
            ["pre-range", "", r#""10.88.0.64/26""#, "10.89.0.64/26"],
        ),
    ] {
        for authored in values {
            network.push_network(key, value(authored)?)?;
        }
    }
    assert_eq!(
        network.build(SourceId::new(303))?.text(),
        concat!(
            "[Network]\n",
            "NetworkName=ipam-network\n",
            "IPAMDriver=\"host-local-%n\"\n",
            "Subnet=pre-subnet\n",
            "Subnet=\n",
            "Subnet=\"10.88.0.0/24\"\n",
            "Subnet=10.89.0.0/24\n",
            "Gateway=pre-gateway\n",
            "Gateway=\n",
            "Gateway=\"10.88.0.1\"\n",
            "Gateway=10.89.0.1\n",
            "IPRange=pre-range\n",
            "IPRange=\n",
            "IPRange=\"10.88.0.64/26\"\n",
            "IPRange=10.89.0.64/26\n",
        )
    );
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
fn build_environment_builder_preserves_repeatable_opaque_values_and_parse_back()
-> Result<(), Box<dyn std::error::Error>> {
    let mut build = QuadletDocumentBuilder::new(QuadletUnitType::Build);
    build.push_build(BuildKey::ImageTag, value("localhost/example:environment")?)?;
    for authored in [
        "PRE=one",
        "",
        "NAME=final",
        "bare",
        r#""QUOTED=Authored Value""#,
        r"ESCAPED=literal\x20text",
        "embedded=a=b",
    ] {
        build.push_build(BuildKey::Environment, value(authored)?)?;
    }
    assert_eq!(
        build.build(SourceId::new(204))?.text(),
        concat!(
            "[Build]\n",
            "ImageTag=localhost/example:environment\n",
            "Environment=PRE=one\n",
            "Environment=\n",
            "Environment=NAME=final\n",
            "Environment=bare\n",
            "Environment=\"QUOTED=Authored Value\"\n",
            "Environment=ESCAPED=literal\\x20text\n",
            "Environment=embedded=a=b\n",
        )
    );
    Ok(())
}

#[test]
fn build_containers_conf_module_builder_preserves_repeatable_opaque_values_and_parse_back()
-> Result<(), Box<dyn std::error::Error>> {
    let mut build = QuadletDocumentBuilder::new(QuadletUnitType::Build);
    build.push_build(BuildKey::ImageTag, value("localhost/example:module")?)?;
    for authored in [
        "pre-one",
        "",
        " post one ",
        "post-two",
        "post-two",
        r#""quoted module""#,
        r"module\x20text",
        r"continuation\-looking",
    ] {
        build.push_build(BuildKey::ContainersConfModule, value(authored)?)?;
    }
    let generated = build.build(SourceId::new(207))?;
    assert_eq!(
        generated.text(),
        concat!(
            "[Build]\n",
            "ImageTag=localhost/example:module\n",
            "ContainersConfModule=pre-one\n",
            "ContainersConfModule=\n",
            "ContainersConfModule= post one \n",
            "ContainersConfModule=post-two\n",
            "ContainersConfModule=post-two\n",
            "ContainersConfModule=\"quoted module\"\n",
            "ContainersConfModule=module\\x20text\n",
            "ContainersConfModule=continuation\\-looking\n",
        )
    );
    let parsed = QuadletDocument::parse(QuadletUnitType::Build, SourceId::new(208), generated.text())?;
    assert!(parsed.is_valid());
    assert_eq!(
        parsed
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Build(BuildKey::ContainersConfModule))
            .map(|entry| (entry.value().primary().text(), entry.value_kind()))
            .collect::<Vec<_>>(),
        [
            ("pre-one", ValueKind::Opaque),
            ("", ValueKind::Opaque),
            ("post one ", ValueKind::Opaque),
            ("post-two", ValueKind::Opaque),
            ("post-two", ValueKind::Opaque),
            ("\"quoted module\"", ValueKind::Opaque),
            ("module\\x20text", ValueKind::Opaque),
            ("continuation\\-looking", ValueKind::Opaque),
        ]
    );
    Ok(())
}

#[test]
fn build_global_args_builder_preserves_empty_duplicates_and_raw_physical_values()
-> Result<(), Box<dyn std::error::Error>> {
    let mut build = QuadletDocumentBuilder::new(QuadletUnitType::Build);
    build.push_build(BuildKey::ImageTag, value("localhost/example:global-args")?)?;
    for authored in [
        "--events-backend=none",
        "--events-backend=none",
        "",
        " --log-level=debug ",
        r#""--transient""#,
        r"--events-backend\x3dfile",
        r"continuation\-looking",
        " malformed global argument ",
    ] {
        build.push_build(BuildKey::GlobalArgs, value(authored)?)?;
    }
    let generated = build.build(SourceId::new(211))?;
    assert_eq!(
        generated.text(),
        concat!(
            "[Build]\n",
            "ImageTag=localhost/example:global-args\n",
            "GlobalArgs=--events-backend=none\n",
            "GlobalArgs=--events-backend=none\n",
            "GlobalArgs=\n",
            "GlobalArgs= --log-level=debug \n",
            "GlobalArgs=\"--transient\"\n",
            "GlobalArgs=--events-backend\\x3dfile\n",
            "GlobalArgs=continuation\\-looking\n",
            "GlobalArgs= malformed global argument \n",
        )
    );
    assert_eq!(
        QuadletDocument::parse(QuadletUnitType::Build, SourceId::new(212), generated.text())?
            .document()
            .entries()
            .filter(|entry| entry.kind() == EntryKind::Build(BuildKey::GlobalArgs))
            .map(|entry| entry.value().primary().text())
            .collect::<Vec<_>>(),
        [
            "--events-backend=none",
            "--events-backend=none",
            "",
            "--log-level=debug ",
            "\"--transient\"",
            "--events-backend\\x3dfile",
            "continuation\\-looking",
            "malformed global argument ",
        ]
    );
    Ok(())
}

#[test]
fn build_service_name_builder_preserves_one_opaque_value_and_rejects_duplicates()
-> Result<(), Box<dyn std::error::Error>> {
    for (source_id, authored) in [
        (SourceId::new(213), ""),
        (SourceId::new(214), "quadlet-lens.service"),
        (SourceId::new(215), r#"\"quoted-%i\""#),
        (SourceId::new(216), " service name "),
        (SourceId::new(217), r"continuation\-looking"),
    ] {
        let mut build = QuadletDocumentBuilder::new(QuadletUnitType::Build);
        build.push_build(BuildKey::ImageTag, value("localhost/example:service-name")?)?;
        build.push_build(BuildKey::ServiceName, value(authored)?)?;
        assert_eq!(
            build.build(source_id)?.text(),
            format!("[Build]\nImageTag=localhost/example:service-name\nServiceName={authored}\n")
        );
    }
    let mut duplicate = QuadletDocumentBuilder::new(QuadletUnitType::Build);
    duplicate.push_build(BuildKey::ImageTag, value("localhost/example:service-name")?)?;
    duplicate.push_build(BuildKey::ServiceName, value("first.service")?)?;
    assert!(
        duplicate
            .push_build(BuildKey::ServiceName, value("second.service")?)
            .is_err()
    );
    Ok(())
}

#[test]
fn build_volume_builder_preserves_repeatable_opaque_values() -> Result<(), Box<dyn std::error::Error>> {
    let mut build = QuadletDocumentBuilder::new(QuadletUnitType::Build);
    build.push_build(BuildKey::ImageTag, value("localhost/example:volume")?)?;
    for authored in [
        "cache.volume:/var/cache:Z",
        ".:/workspace",
        "destination-only",
        "",
        r#""quoted.volume":/quoted"#,
        "%h/data:/home",
        "cache.volume:/var/cache:Z",
    ] {
        build.push_build(BuildKey::Volume, value(authored)?)?;
    }
    let generated = build.build(SourceId::new(219))?;
    assert_eq!(
        generated.text(),
        concat!(
            "[Build]\n",
            "ImageTag=localhost/example:volume\n",
            "Volume=cache.volume:/var/cache:Z\n",
            "Volume=.:/workspace\n",
            "Volume=destination-only\n",
            "Volume=\n",
            "Volume=\"quoted.volume\":/quoted\n",
            "Volume=%h/data:/home\n",
            "Volume=cache.volume:/var/cache:Z\n",
        )
    );
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

#[test]
fn container_reload_keys_are_opaque_singletons_and_mutually_exclusive() -> Result<(), Box<dyn std::error::Error>> {
    let mut command = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    command.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
    command.push_container(ContainerKey::ReloadCmd, value("/usr/bin/reload --mode=\"safe %i\"")?)?;
    assert!(matches!(
        command.push_container(ContainerKey::ReloadCmd, value("second")?),
        Err(RenderError::DuplicateSingleton(key)) if key == "ReloadCmd"
    ));
    assert!(matches!(
        command.push_container(ContainerKey::ReloadSignal, value("SIGUSR1")?),
        Err(RenderError::ConflictingSingletons { existing, attempted })
            if existing == "ReloadCmd" && attempted == "ReloadSignal"
    ));
    assert_eq!(
        command.build(SourceId::new(181))?.text(),
        "[Container]\nImage=example.invalid/app\nReloadCmd=/usr/bin/reload --mode=\"safe %i\"\n"
    );

    let mut signal = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    signal.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
    signal.push_container(ContainerKey::ReloadSignal, value("vendor-defined-signal")?)?;
    assert!(matches!(
        signal.push_container(ContainerKey::ReloadCmd, value("opaque command")?),
        Err(RenderError::ConflictingSingletons { existing, attempted })
            if existing == "ReloadSignal" && attempted == "ReloadCmd"
    ));
    Ok(())
}

fn value(value: &str) -> Result<EntryValue, RenderError> {
    EntryValue::new(value)
}

#[test]
fn remaining_container_key_builders_preserve_repeatable_order_and_reject_duplicate_singletons()
-> Result<(), Box<dyn std::error::Error>> {
    let mut container = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    container.push_container(ContainerKey::Image, value("example.invalid/app")?)?;
    for authored in ["pre.conf", "", "post.conf"] {
        container.push_container(ContainerKey::ContainersConfModule, value(authored)?)?;
    }
    for authored in ["--log-level=debug", "--events-backend=none"] {
        container.push_container(ContainerKey::GlobalArgs, value(authored)?)?;
    }
    for authored in ["/assets", "src.image:/opt/assets"] {
        container.push_container(ContainerKey::ImageVolume, value(authored)?)?;
    }
    for (key, authored) in [
        (ContainerKey::HealthLogDestination, "local"),
        (ContainerKey::HealthMaxLogCount, "5"),
        (ContainerKey::HealthMaxLogSize, "10m"),
        (ContainerKey::HealthStartupCmd, "CMD-SHELL echo ready"),
        (ContainerKey::HealthStartupInterval, "2s"),
        (ContainerKey::HealthStartupRetries, "4"),
        (ContainerKey::HealthStartupSuccess, "2"),
        (ContainerKey::HealthStartupTimeout, "1s"),
        (ContainerKey::ServiceName, "chosen.service"),
    ] {
        container.push_container(key, value(authored)?)?;
        assert!(matches!(
            container.push_container(key, value("duplicate")?),
            Err(RenderError::DuplicateSingleton(_))
        ));
    }
    let text = container.build(SourceId::new(1_081))?.text().to_owned();
    assert!(text.contains("ContainersConfModule=pre.conf\nContainersConfModule=\nContainersConfModule=post.conf\n"));
    assert!(text.contains("GlobalArgs=--log-level=debug\nGlobalArgs=--events-backend=none\n"));
    assert!(text.contains("ImageVolume=/assets\nImageVolume=src.image:/opt/assets\n"));
    Ok(())
}

#[test]
fn remaining_pod_key_builders_preserve_order_and_reject_duplicate_singletons() -> Result<(), Box<dyn std::error::Error>>
{
    let mut pod = QuadletDocumentBuilder::new(QuadletUnitType::Pod);
    pod.push_pod(PodKey::PodName, value("example")?)?;
    for (key, authored) in [
        (PodKey::ContainersConfModule, "post.conf"),
        (PodKey::DNS, "9.9.9.9"),
        (PodKey::DNSOption, "ndots:1"),
        (PodKey::DNSSearch, "example.invalid"),
        (PodKey::GIDMap, "0:200000:1"),
        (PodKey::GlobalArgs, "--log-level=debug"),
        (PodKey::Label, "org.example.pod=yes"),
        (PodKey::NetworkAlias, "api"),
        (PodKey::PodmanArgs, "--replace"),
        (PodKey::UIDMap, "0:100000:1"),
    ] {
        pod.push_pod(key, value(authored)?)?;
    }
    for (key, authored) in [
        (PodKey::HostName, "pod-host"),
        (PodKey::IP, "10.88.0.2"),
        (PodKey::IP6, "fd00::2"),
    ] {
        pod.push_pod(key, value(authored)?)?;
        assert!(matches!(
            pod.push_pod(key, value("duplicate")?),
            Err(RenderError::DuplicateSingleton(_))
        ));
    }
    let text = pod.build(SourceId::new(1_092))?.text().to_owned();
    assert!(text.contains("DNS=9.9.9.9\nDNSOption=ndots:1\nDNSSearch=example.invalid\n"));
    Ok(())
}

#[test]
fn pod_mapping_builders_accept_independent_direct_and_subordinate_forms() -> Result<(), Box<dyn std::error::Error>> {
    let mut direct = QuadletDocumentBuilder::new(QuadletUnitType::Pod);
    for (key, authored) in [
        (PodKey::UIDMap, "0:100000:65536"),
        (PodKey::UIDMap, "1:200000:1"),
        (PodKey::GIDMap, "0:100000:65536"),
        (PodKey::GIDMap, "1:200000:1"),
    ] {
        direct.push_pod(key, value(authored)?)?;
    }
    assert_eq!(
        direct.build(SourceId::new(1_093))?.text(),
        concat!(
            "[Pod]\n",
            "UIDMap=0:100000:65536\n",
            "UIDMap=1:200000:1\n",
            "GIDMap=0:100000:65536\n",
            "GIDMap=1:200000:1\n",
        )
    );

    let mut subordinate = QuadletDocumentBuilder::new(QuadletUnitType::Pod);
    subordinate.push_pod(PodKey::SubUIDMap, value("keep-id")?)?;
    subordinate.push_pod(PodKey::SubGIDMap, value("keep-id")?)?;
    for key in [PodKey::SubUIDMap, PodKey::SubGIDMap] {
        assert!(matches!(
            subordinate.push_pod(key, value("duplicate")?),
            Err(RenderError::DuplicateSingleton(actual)) if actual == pod_key_name_for_test(key)
        ));
    }
    assert_eq!(
        subordinate.build(SourceId::new(1_094))?.text(),
        "[Pod]\nSubUIDMap=keep-id\nSubGIDMap=keep-id\n"
    );
    Ok(())
}

#[test]
fn pod_mapping_builder_conflicts_fail_parse_back_validation() -> Result<(), Box<dyn std::error::Error>> {
    let mut user = QuadletDocumentBuilder::new(QuadletUnitType::Pod);
    user.push_pod(PodKey::UserNS, value("keep-id")?)?;
    user.push_pod(PodKey::UIDMap, value("0:100000:1")?)?;
    assert_invalid_pod_mapping_builder(&user, SourceId::new(1_095), "QLM0013")?;

    let mut uid = QuadletDocumentBuilder::new(QuadletUnitType::Pod);
    uid.push_pod(PodKey::UIDMap, value("0:100000:1")?)?;
    uid.push_pod(PodKey::SubUIDMap, value("keep-id")?)?;
    assert_invalid_pod_mapping_builder(&uid, SourceId::new(1_096), "QLM0014")?;

    let mut gid = QuadletDocumentBuilder::new(QuadletUnitType::Pod);
    gid.push_pod(PodKey::GIDMap, value("0:200000:1")?)?;
    gid.push_pod(PodKey::SubGIDMap, value("keep-id")?)?;
    assert_invalid_pod_mapping_builder(&gid, SourceId::new(1_097), "QLM0015")?;
    Ok(())
}

#[test]
fn kube_builder_preserves_order_rejects_singleton_duplicates_and_requires_yaml()
-> Result<(), Box<dyn std::error::Error>> {
    let mut generated = QuadletDocumentBuilder::new(QuadletUnitType::Kube);
    for (key, text) in [
        (KubeKey::Yaml, "./first.yaml"),
        (KubeKey::Yaml, "/opt/quadlet-lens/second.yaml"),
        (KubeKey::ConfigMap, "/run/quadlet-lens/first-config.yaml"),
        (KubeKey::ConfigMap, "/run/quadlet-lens/second-config.yaml"),
        (KubeKey::ContainersConfModule, "pre.conf"),
        (KubeKey::ContainersConfModule, ""),
        (KubeKey::ContainersConfModule, "post.conf"),
        (KubeKey::GlobalArgs, "--log-level=debug"),
        (KubeKey::GlobalArgs, ""),
        (KubeKey::GlobalArgs, "--events-backend=file"),
        (KubeKey::Network, "frontend.network"),
        (KubeKey::PublishPort, "8080:80"),
        (KubeKey::PublishPort, "8443:443"),
        (KubeKey::PodmanArgs, "--replace"),
        (KubeKey::PodmanArgs, ""),
        (KubeKey::PodmanArgs, "--userns=keep-id"),
        (KubeKey::AutoUpdate, "registry"),
        (KubeKey::AutoUpdate, "local"),
        (KubeKey::ExitCodePropagation, "any"),
        (KubeKey::KubeDownForce, "true"),
        (KubeKey::LogDriver, "k8s-file"),
        (KubeKey::ServiceName, "quadlet-lens-kube.service"),
        (KubeKey::SetWorkingDirectory, "unit"),
        (KubeKey::UserNS, "keep-id"),
        (KubeKey::LogOpt, "path=/run/quadlet-lens/kube.log"),
        (KubeKey::LogOpt, "max-size=1m"),
    ] {
        generated.push_kube(key, value(text)?)?;
    }
    assert!(matches!(
        generated.push_kube(KubeKey::KubeDownForce, value("false")?),
        Err(RenderError::DuplicateSingleton(actual)) if actual == "KubeDownForce"
    ));
    let built = generated.build(SourceId::new(1_100))?;
    assert!(
        built
            .text()
            .starts_with("[Kube]\nYaml=./first.yaml\nYaml=/opt/quadlet-lens/second.yaml\n")
    );
    assert!(
        built
            .text()
            .contains("ContainersConfModule=pre.conf\nContainersConfModule=\nContainersConfModule=post.conf\n")
    );
    assert!(
        built
            .text()
            .contains("PodmanArgs=--replace\nPodmanArgs=\nPodmanArgs=--userns=keep-id\n")
    );
    let empty = QuadletDocumentBuilder::new(QuadletUnitType::Kube);
    assert!(matches!(
        empty.build(SourceId::new(1_101)),
        Err(RenderError::InvalidDocument(_))
    ));
    let mut wrong = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    assert!(matches!(
        wrong.push_kube(KubeKey::Yaml, value("placeholder.yaml")?),
        Err(RenderError::WrongUnitType { .. })
    ));
    Ok(())
}

fn assert_invalid_pod_mapping_builder(
    builder: &QuadletDocumentBuilder,
    source_id: SourceId,
    code: &str,
) -> Result<(), String> {
    let error = builder
        .build(source_id)
        .err()
        .ok_or_else(|| "conflicting Pod mapping builder entries must not generate a valid document".to_owned())?;
    assert!(matches!(error, RenderError::InvalidDocument(_)));
    assert_eq!(
        error
            .diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().as_str())
            .collect::<Vec<_>>(),
        [code]
    );
    Ok(())
}

fn pod_key_name_for_test(key: PodKey) -> &'static str {
    match key {
        PodKey::SubUIDMap => "SubUIDMap",
        PodKey::SubGIDMap => "SubGIDMap",
        _ => unreachable!("only subordinate mapping keys are passed to this helper"),
    }
}
