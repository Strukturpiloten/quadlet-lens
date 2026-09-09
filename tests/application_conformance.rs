//! Deterministic, independently asserted real-application Quadlet contracts.

#[path = "support/application.rs"]
mod application;

use std::path::PathBuf;

use quadlet_lens::model::{
    AuthoredContainerEnvironmentDirective, ContainerKey, EntryKind, EnvironmentReferenceState, NamedQuadletDocument,
    NativeCommandDirective, NativeMountDirective, NativeMountOption, NativePortProtocol, NativePortPublication,
    QuadletDocument, QuadletDocumentSet, QuadletUnitType, ReferenceResolution, SystemdUnitKey, UnitReferenceKind,
};
use quadlet_lens::source::SourceId;

use application::{
    CopiedUnit, FORGEJO_ID, FORGEJO_UNITS, NEXTCLOUD_ID, NEXTCLOUD_UNITS, application_contract_target, read_fixture,
    verify_generated_application, verify_known_application_fixture,
};

struct Application {
    root: PathBuf,
    set: QuadletDocumentSet,
}

impl Application {
    fn document(&self, name: &str) -> Result<&QuadletDocument, String> {
        self.set
            .document(name)
            .map(NamedQuadletDocument::document)
            .ok_or_else(|| format!("application fixture omitted `{name}`"))
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "One application-sized contract keeps the complete Nextcloud native meaning auditable."
)]
fn nextcloud_units_preserve_independent_native_meaning_and_relationships() -> Result<(), String> {
    let application = load_application(NEXTCLOUD_ID, NEXTCLOUD_UNITS, 20_000)?;
    assert_eq!(application.set.documents().len(), 13);
    assert_graph(
        &application,
        &[
            ("app.container", "nextcloud-data.volume", UnitReferenceKind::Volume),
            ("app.container", "backend.network", UnitReferenceKind::Network),
            ("cache.container", "redis-data.volume", UnitReferenceKind::Volume),
            ("cache.container", "backend.network", UnitReferenceKind::Network),
            ("cron.container", "nextcloud-data.volume", UnitReferenceKind::Volume),
            ("cron.container", "backend.network", UnitReferenceKind::Network),
            ("db.container", "db-data.volume", UnitReferenceKind::Volume),
            ("db.container", "backend.network", UnitReferenceKind::Network),
            ("edge-proxy.container", "edge.network", UnitReferenceKind::Network),
            ("frontend.container", "nextcloud-data.volume", UnitReferenceKind::Volume),
            ("frontend.container", "backend.network", UnitReferenceKind::Network),
            ("frontend.container", "edge.network", UnitReferenceKind::Network),
            ("init.container", "nextcloud-data.volume", UnitReferenceKind::Volume),
            ("init.container", "backend.network", UnitReferenceKind::Network),
            ("second-app.container", "edge.network", UnitReferenceKind::Network),
        ],
    );

    let environment_count = application
        .set
        .documents()
        .iter()
        .map(|named| named.document().container_environment().directives().len())
        .sum::<usize>();
    assert_eq!(environment_count, 19);
    let app_environment = application.document("app.container")?.container_environment();
    assert_eq!(
        app_environment
            .directives()
            .iter()
            .filter_map(AuthoredContainerEnvironmentDirective::name)
            .collect::<Vec<_>>(),
        ["POSTGRES_HOST", "REDIS_HOST", "POSTGRES_PASSWORD", "REDIS_PASSWORD",]
    );
    assert!(app_environment.diagnostics().is_empty());
    assert_literal_environment(
        application.document("app.container")?,
        &[
            ("POSTGRES_HOST", "db"),
            ("REDIS_HOST", "cache"),
            ("POSTGRES_PASSWORD", "boxferry-test-db-password-not-secret"),
            ("REDIS_PASSWORD", "boxferry-test-cache-password-not-secret"),
        ],
    );
    assert_literal_environment(
        application.document("cache.container")?,
        &[(
            "BOXFERRY_REDIS_TEST_PASSWORD",
            "boxferry-test-cache-password-not-secret",
        )],
    );
    assert_literal_environment(
        application.document("cron.container")?,
        &[
            ("POSTGRES_HOST", "db"),
            ("REDIS_HOST", "cache"),
            ("POSTGRES_PASSWORD", "boxferry-test-db-password-not-secret"),
            ("REDIS_PASSWORD", "boxferry-test-cache-password-not-secret"),
        ],
    );
    assert_literal_environment(
        application.document("db.container")?,
        &[
            ("POSTGRES_DB", "nextcloud_test"),
            ("POSTGRES_USER", "nextcloud_test"),
            ("POSTGRES_PASSWORD", "boxferry-test-db-password-not-secret"),
        ],
    );
    assert_literal_environment(
        application.document("edge-proxy.container")?,
        &[("BOXFERRY_EDGE_TEST_TOKEN", "boxferry-test-edge-token-not-secret")],
    );
    assert_literal_environment(application.document("frontend.container")?, &[]);
    assert_literal_environment(
        application.document("init.container")?,
        &[
            ("POSTGRES_HOST", "db"),
            ("REDIS_HOST", "cache"),
            ("NEXTCLOUD_ADMIN_USER", "boxferry-test-admin"),
            ("NEXTCLOUD_ADMIN_PASSWORD", "boxferry-test-admin-password-not-secret"),
            ("POSTGRES_PASSWORD", "boxferry-test-db-password-not-secret"),
            ("REDIS_PASSWORD", "boxferry-test-cache-password-not-secret"),
        ],
    );
    assert_literal_environment(application.document("second-app.container")?, &[]);

    let mount_count = application
        .set
        .documents()
        .iter()
        .map(|named| named.document().container_mounts().directives().len())
        .sum::<usize>();
    assert_eq!(mount_count, 9);
    let frontend_mounts = application.document("frontend.container")?.container_mounts();
    assert_eq!(frontend_mounts.directives().len(), 2);
    let NativeMountDirective::Mount(bind) = &frontend_mounts.directives()[1] else {
        return Err("frontend second mount was not decoded".to_owned());
    };
    assert_eq!(
        bind.source(),
        Some("/srv/boxferry-fixture/nextcloud-application/frontend.conf")
    );
    assert_eq!(bind.destination(), Some("/etc/nginx/conf.d/nextcloud.conf"));
    assert_eq!(
        bind.options(),
        [
            NativeMountOption::Flag("ro".to_owned()),
            NativeMountOption::Flag("Z".to_owned()),
        ]
    );

    let ports = application.document("edge-proxy.container")?.container_ports();
    assert_eq!(ports.directives().len(), 1);
    let NativePortPublication::Publication(port) = &ports.directives()[0] else {
        return Err("edge proxy port was not decoded".to_owned());
    };
    assert_eq!(port.host(), Some("127.0.0.1"));
    assert_eq!(port.published().ok_or("missing host port")?.start(), 18_443);
    assert_eq!(port.container().start(), 8_443);
    assert_eq!(port.protocol(), NativePortProtocol::Tcp);

    assert_command(application.document("cron.container")?, &["/cron.sh"])?;
    assert_command(
        application.document("init.container")?,
        &["/opt/boxferry/init-nextcloud", "--non-interactive"],
    )?;
    let health_count = application
        .set
        .documents()
        .iter()
        .map(|named| typed_values(named.document(), EntryKind::Container(ContainerKey::HealthCmd)).len())
        .sum::<usize>();
    assert_eq!(health_count, 6);

    for (unit, dependency) in [
        ("app.container", "init.service"),
        ("cron.container", "app.service"),
        ("frontend.container", "app.service"),
        ("init.container", "db.service cache.service"),
        ("edge-proxy.container", "frontend.service second-app.service"),
    ] {
        for key in [SystemdUnitKey::Requires, SystemdUnitKey::After] {
            assert_eq!(
                typed_values(application.document(unit)?, EntryKind::SystemdUnit(key)),
                [dependency]
            );
        }
    }
    assert_redacted(
        &application,
        &[
            "boxferry-test-db-password-not-secret",
            "boxferry-test-cache-password-not-secret",
            "boxferry-test-admin-password-not-secret",
            "boxferry-test-edge-token-not-secret",
        ],
    )?;
    assert_eq!(
        application_contract_target(&application.root)?,
        tracked_current_generator_target()?
    );
    Ok(())
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "One application-sized contract keeps complete Forgejo native meaning auditable."
)]
fn forgejo_units_preserve_ports_identity_external_network_and_storage() -> Result<(), String> {
    let application = load_application(FORGEJO_ID, FORGEJO_UNITS, 21_000)?;
    assert_eq!(application.set.documents().len(), 5);
    assert_graph(
        &application,
        &[
            ("db.container", "db-data.volume", UnitReferenceKind::Volume),
            ("db.container", "backend.network", UnitReferenceKind::Network),
            ("forgejo.container", "forgejo-data.volume", UnitReferenceKind::Volume),
            ("forgejo.container", "backend.network", UnitReferenceKind::Network),
        ],
    );
    assert!(
        !application
            .set
            .graph()
            .references()
            .iter()
            .any(|reference| reference.target_name() == "edge")
    );

    let environment_count = application
        .set
        .documents()
        .iter()
        .map(|named| named.document().container_environment().directives().len())
        .sum::<usize>();
    assert_eq!(environment_count, 15);
    assert_literal_environment(
        application.document("db.container")?,
        &[
            ("POSTGRES_DB", "forgejo"),
            ("POSTGRES_USER", "forgejo"),
            ("POSTGRES_PASSWORD", "boxferry-public-db-password-canary"),
        ],
    );
    assert_literal_environment(
        application.document("forgejo.container")?,
        &[
            ("FORGEJO__database__DB_TYPE", "postgres"),
            ("FORGEJO__database__HOST", "db:5432"),
            ("FORGEJO__database__NAME", "forgejo"),
            ("FORGEJO__database__USER", "forgejo"),
            ("FORGEJO__database__PASSWD", "boxferry-public-db-password-canary"),
            ("FORGEJO__database__SSL_MODE", "disable"),
            ("FORGEJO__security__INSTALL_LOCK", "true"),
            ("FORGEJO__security__SECRET_KEY", "boxferry-public-forgejo-secret-canary"),
            ("FORGEJO__server__DOMAIN", "127.0.0.1"),
            ("FORGEJO__server__ROOT_URL", "http://127.0.0.1:13000/"),
            ("FORGEJO__server__SSH_DOMAIN", "127.0.0.1"),
            ("FORGEJO__server__SSH_PORT", "12222"),
        ],
    );
    let mount_count = application
        .set
        .documents()
        .iter()
        .map(|named| named.document().container_mounts().directives().len())
        .sum::<usize>();
    assert_eq!(mount_count, 2);

    let forgejo = application.document("forgejo.container")?;
    let ports = forgejo.container_ports();
    assert_eq!(ports.directives().len(), 2);
    let decoded = ports
        .directives()
        .iter()
        .map(|directive| match directive {
            NativePortPublication::Publication(port) => Ok((
                port.host().map(ToOwned::to_owned),
                port.published().ok_or("missing Forgejo host port")?.start(),
                port.container().start(),
            )),
            _ => Err("Forgejo port was not decoded"),
        })
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(
        decoded,
        [
            (Some("127.0.0.1".to_owned()), 13_000, 3_000),
            (Some("127.0.0.1".to_owned()), 12_222, 2_222),
        ]
    );
    assert_eq!(
        typed_values(forgejo, EntryKind::Container(ContainerKey::User)),
        ["1000"]
    );
    assert_eq!(
        typed_values(forgejo, EntryKind::Container(ContainerKey::Group)),
        ["1000"]
    );
    assert_eq!(
        typed_values(forgejo, EntryKind::Container(ContainerKey::Network)),
        ["backend.network", "edge"]
    );
    assert!(typed_values(forgejo, EntryKind::SystemdUnit(SystemdUnitKey::Requires)).is_empty());
    assert!(forgejo.container_commands().directives().is_empty());
    assert_redacted(
        &application,
        &[
            "boxferry-public-db-password-canary",
            "boxferry-public-forgejo-secret-canary",
        ],
    )?;
    assert_eq!(
        application_contract_target(&application.root)?,
        tracked_current_generator_target()?
    );
    Ok(())
}

#[test]
fn built_in_application_contracts_target_the_generator_matrix_tracked_current() -> Result<(), String> {
    let tracked_current = tracked_current_generator_target()?;
    for id in [NEXTCLOUD_ID, FORGEJO_ID] {
        let root = verify_known_application_fixture(id)?;
        assert_eq!(application_contract_target(&root)?, tracked_current, "{id}");
    }
    Ok(())
}

#[test]
fn authored_environment_boundary_keeps_resets_files_escaping_and_specifiers_separate() -> Result<(), String> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generators/first-conversion-supported-range");
    let app_source = std::fs::read_to_string(root.join("app.container")).map_err(|error| error.to_string())?;
    let app = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(22_000), app_source)
        .map_err(|error| error.to_string())?;
    let environment = app.document().container_environment();
    assert_eq!(environment.directives().len(), 10);
    assert!(matches!(
        environment.directives()[0],
        AuthoredContainerEnvironmentDirective::Deferred { .. }
    ));
    assert_eq!(environment.directives()[1].literal_value(), Some("hello world"));
    assert_eq!(environment.directives()[2].literal_value(), Some("hello \"quoted\""));
    assert_eq!(environment.directives()[8].literal_value(), Some("hello world"));
    assert_eq!(environment.directives()[9].name(), Some("QUADLET_LENS_GROUP_EQUALS"));

    let worker_source = std::fs::read_to_string(root.join("worker.container")).map_err(|error| error.to_string())?;
    let worker = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(22_001), worker_source)
        .map_err(|error| error.to_string())?;
    let sources = worker.document().container_environment_sources();
    assert_eq!(sources.environment_files().len(), 2);
    assert_eq!(sources.environment_files()[0].path(), Some("./worker.env"));
    assert_eq!(
        sources.environment_files()[1].path(),
        Some("/etc/quadlet-lens/worker.env")
    );
    assert!(
        sources
            .environment_files()
            .iter()
            .all(|reference| reference.state() == EnvironmentReferenceState::Literal)
    );

    let reset_source = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures/generators/container-environment-reset-supported-range/environment-reset.container"),
    )
    .map_err(|error| error.to_string())?;
    let reset = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(22_002), reset_source)
        .map_err(|error| error.to_string())?;
    let directives = reset.document().container_environment();
    assert_eq!(directives.directives().len(), 5);
    assert!(matches!(
        directives.directives()[2],
        AuthoredContainerEnvironmentDirective::Reset { .. }
    ));
    assert_eq!(
        directives.directives()[3].name(),
        Some("QUADLET_LENS_ENV_RESET_POST_ONE")
    );
    Ok(())
}

#[test]
fn application_generator_contract_rejects_untracked_target_boundaries_before_output() -> Result<(), String> {
    for id in [NEXTCLOUD_ID, FORGEJO_ID] {
        let root = verify_known_application_fixture(id)?;
        for target in ["5.3.3", "6.1.1", "not-a-version"] {
            let error = match verify_generated_application(id, &root, target, "") {
                Ok(()) => return Err(format!("unsupported target `{target}` was accepted")),
                Err(error) => error,
            };
            assert!(error.contains("unsupported"), "{error}");
        }
    }
    Ok(())
}

fn load_application(id: &str, units: &[CopiedUnit<'_>], first_source_id: u32) -> Result<Application, String> {
    let root = verify_known_application_fixture(id)?;
    let mut documents = Vec::with_capacity(units.len());
    for (index, unit) in units.iter().enumerate() {
        let source = read_fixture(&root, unit.path)?;
        let extension = unit
            .path
            .rsplit_once('.')
            .map(|(_, extension)| extension)
            .ok_or_else(|| format!("{} has no extension", unit.path))?;
        let unit_type = QuadletUnitType::from_extension(extension)
            .ok_or_else(|| format!("{} has unsupported extension", unit.path))?;
        let offset = u32::try_from(index).map_err(|error| error.to_string())?;
        let parsed = QuadletDocument::parse(unit_type, SourceId::new(first_source_id + offset), source.clone())
            .map_err(|error| error.to_string())?;
        if !parsed.is_valid() {
            return Err(format!("{} did not parse as valid Quadlet", unit.path));
        }
        if parsed.syntax().document().render_preserved() != source {
            return Err(format!("{} did not preserve source bytes", unit.path));
        }
        let canonical = parsed.syntax().render_canonical().map_err(|error| error.to_string())?;
        let canonical_parsed =
            QuadletDocument::parse(unit_type, SourceId::new(first_source_id + 1_000 + offset), canonical)
                .map_err(|error| error.to_string())?;
        if !canonical_parsed.is_valid() {
            return Err(format!("{} canonical rendering did not parse", unit.path));
        }
        let (_, document, _) = parsed.into_parts();
        documents.push(NamedQuadletDocument::new(unit.path, document).map_err(|error| error.to_string())?);
    }
    let set = QuadletDocumentSet::new(documents).map_err(|error| error.to_string())?;
    if !set.is_valid() || !set.graph().is_complete() {
        return Err(format!("{id}: native application document graph is incomplete"));
    }
    Ok(Application { root, set })
}

fn assert_graph(application: &Application, expected: &[(&str, &str, UnitReferenceKind)]) {
    let references = application.set.graph().references();
    assert!(
        references
            .iter()
            .all(|reference| matches!(reference.resolution(), ReferenceResolution::Resolved { .. }))
    );
    let actual = application
        .set
        .graph()
        .edges()
        .iter()
        .map(|edge| {
            (
                application.set.documents()[edge.source_document()].name().as_str(),
                application.set.documents()[edge.target_document()].name().as_str(),
                edge.kind(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
    assert_eq!(actual.len(), references.len());
}

fn typed_values(document: &QuadletDocument, kind: EntryKind) -> Vec<&str> {
    document
        .entries()
        .filter(|entry| entry.kind() == kind)
        .map(|entry| entry.value().primary().text())
        .collect()
}

fn assert_literal_environment(document: &QuadletDocument, expected: &[(&str, &str)]) {
    let environment = document.container_environment();
    assert_eq!(environment.directives().len(), expected.len());
    for (directive, (name, value)) in environment.directives().iter().zip(expected) {
        assert_eq!(directive.name(), Some(*name));
        assert_eq!(directive.literal_value(), Some(*value));
    }
    assert!(environment.diagnostics().is_empty());
}

fn assert_command(document: &QuadletDocument, expected: &[&str]) -> Result<(), String> {
    let commands = document.container_commands();
    assert_eq!(commands.directives().len(), 1);
    let NativeCommandDirective::Command { command, .. } = &commands.directives()[0] else {
        return Err("application Exec was not decoded".to_owned());
    };
    assert_eq!(
        command.arguments(),
        expected.iter().map(|value| (*value).to_owned()).collect::<Vec<_>>()
    );
    Ok(())
}

fn assert_redacted(application: &Application, canaries: &[&str]) -> Result<(), String> {
    for document in application.set.documents() {
        let debug = format!("{:?}", document.document().container_environment());
        for canary in canaries {
            if debug.contains(canary) {
                return Err(format!(
                    "environment debug output exposed canary for {}",
                    document.name()
                ));
            }
        }
    }
    Ok(())
}

fn tracked_current_generator_target() -> Result<String, String> {
    let matrix = include_str!("../tools/generator-matrix.toml")
        .parse::<toml::Table>()
        .map_err(|error| format!("invalid tools/generator-matrix.toml: {error}"))?;
    matrix
        .get("tracked_current")
        .and_then(toml::Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| "tools/generator-matrix.toml tracked_current must be a string".to_owned())
}
