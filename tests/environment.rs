//! External environment authorization and generated ordering contracts.

use quadlet_lens::{
    model::{
        AuthorizedContainerEnvironment, AuthorizedEnvironmentAssignment, ContainerKey, EnvironmentReferenceState,
        EnvironmentValueError, QuadletDocument, QuadletUnitType, SensitiveEnvironmentValue,
    },
    render::{
        ContainerEnvironmentDirective, ContainerEnvironmentPlan, EntryValue, EnvironmentAssignment,
        EnvironmentAssignments, QuadletDocumentBuilder,
    },
    source::SourceId,
};

fn protected(value: &str) -> Result<SensitiveEnvironmentValue, EnvironmentValueError> {
    SensitiveEnvironmentValue::new(value)
}

#[test]
fn external_environment_discovery_preserves_unresolved_source_and_never_reads_ambient_state()
-> Result<(), Box<dyn std::error::Error>> {
    let source = concat!(
        "[Container]\n",
        "Image=example.invalid/application:1\n",
        "Environment=\"QUOTED=hello world\" EMPTY= ESCAPED=one\\x20two\n",
        "EnvironmentFile=./application.env\n",
        "EnvironmentFile=-%h/deferred.env\n",
        "EnvironmentFile=\n",
        "Secret=application-token,type=env,target=APPLICATION_TOKEN\n",
        "Secret=mounted-only\n",
        "Secret=application-%i,type=env,target=APPLICATION_%i\n",
        "Secret=invalid,type=env,target=INVALID-NAME\n",
    );
    let parsed = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(9_301), source)?;
    let sources = parsed.document().container_environment_sources();

    assert_eq!(sources.inline().get("QUOTED").literal(), Some("hello world"));
    assert_eq!(sources.inline().get("EMPTY").literal(), Some(""));
    assert_eq!(sources.inline().get("ESCAPED").literal(), Some("one two"));
    assert_eq!(sources.environment_files().len(), 3);
    assert_eq!(sources.environment_files()[0].path(), Some("./application.env"));
    assert!(!sources.environment_files()[0].optional());
    assert_eq!(
        sources.environment_files()[0].state(),
        EnvironmentReferenceState::Literal
    );
    assert_eq!(sources.environment_files()[1].path(), Some("%h/deferred.env"));
    assert!(sources.environment_files()[1].optional());
    assert_eq!(
        sources.environment_files()[1].state(),
        EnvironmentReferenceState::Deferred
    );
    assert_eq!(sources.environment_files()[2].path(), None);
    assert_eq!(
        sources.environment_files()[2].state(),
        EnvironmentReferenceState::Unmodeled
    );
    assert_eq!(
        &source[sources.environment_files()[0].span().start()..sources.environment_files()[0].span().end()],
        "./application.env"
    );

    assert_eq!(sources.environment_secrets().len(), 3);
    assert_eq!(sources.environment_secrets()[0].secret(), Some("application-token"));
    assert_eq!(sources.environment_secrets()[0].target(), Some("APPLICATION_TOKEN"));
    assert_eq!(
        sources.environment_secrets()[0].state(),
        EnvironmentReferenceState::Literal
    );
    assert_eq!(
        sources.environment_secrets()[1].state(),
        EnvironmentReferenceState::Deferred
    );
    assert_eq!(
        sources.environment_secrets()[2].state(),
        EnvironmentReferenceState::Unmodeled
    );
    assert_eq!(
        &source[sources.environment_secrets()[0].span().start()..sources.environment_secrets()[0].span().end()],
        "application-token,type=env,target=APPLICATION_TOKEN"
    );

    let codes = sources
        .diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.code().as_str())
        .collect::<Vec<_>>();
    assert_eq!(codes, ["QLM0026", "QLM0025", "QLM0028", "QLM0027"]);
    Ok(())
}

trait AuthoredLiteral<'a> {
    fn literal(self) -> Option<&'a str>;
}

impl<'a> AuthoredLiteral<'a> for quadlet_lens::model::AuthoredContainerEnvironmentValue<'a> {
    fn literal(self) -> Option<&'a str> {
        match self {
            Self::Literal(value) => Some(value),
            _ => None,
        }
    }
}

#[test]
fn only_exact_caller_authorizations_resolve_and_payloads_remain_redacted() -> Result<(), Box<dyn std::error::Error>> {
    let parsed = QuadletDocument::parse(
        QuadletUnitType::Container,
        SourceId::new(9_302),
        concat!(
            "[Container]\n",
            "Image=example.invalid/application:1\n",
            "EnvironmentFile=./application.env\n",
            "EnvironmentFile=%h/deferred.env\n",
            "Secret=application-token,type=env,target=APPLICATION_TOKEN\n",
            "Secret=missing-token,type=env,target=MISSING_TOKEN\n",
        ),
    )?;
    let sources = parsed.document().container_environment_sources();
    let mut authorized = AuthorizedContainerEnvironment::new();
    authorized.authorize_environment_file(
        "./application.env",
        [
            AuthorizedEnvironmentAssignment::new("ALPHA", protected("first canary")?)?,
            AuthorizedEnvironmentAssignment::new("EMPTY", protected("")?)?,
            AuthorizedEnvironmentAssignment::new("ALPHA", protected("last canary")?)?,
        ],
    )?;
    authorized.authorize_environment_file(
        "%h/deferred.env",
        [AuthorizedEnvironmentAssignment::new(
            "IGNORED",
            protected("deferred canary")?,
        )?],
    )?;
    authorized.authorize_secret("application-token", protected("secret canary")?)?;

    let resolved = sources.resolve(&authorized);
    let Some(assignments) = resolved.environment_files()[0].assignments() else {
        return Err("exact literal path was not authorized".into());
    };
    assert_eq!(
        assignments
            .iter()
            .map(AuthorizedEnvironmentAssignment::name)
            .collect::<Vec<_>>(),
        ["ALPHA", "EMPTY", "ALPHA"]
    );
    assert_eq!(assignments[0].value().expose_secret(), "first canary");
    assert_eq!(assignments[1].value().expose_secret(), "");
    assert_eq!(assignments[2].value().expose_secret(), "last canary");
    assert!(resolved.environment_files()[1].assignments().is_none());
    let Some(secret) = resolved.environment_secrets()[0].value() else {
        return Err("exact secret was not authorized".into());
    };
    assert_eq!(secret.expose_secret(), "secret canary");
    assert!(resolved.environment_secrets()[1].value().is_none());

    let debug = format!("{authorized:?} {resolved:?}");
    for protected in ["first canary", "last canary", "deferred canary", "secret canary"] {
        assert!(!debug.contains(protected));
    }
    assert_eq!(protected("contains\0nul"), Err(EnvironmentValueError::Nul));
    assert_eq!(
        AuthorizedEnvironmentAssignment::new("INVALID-NAME", protected("value")?),
        Err(EnvironmentValueError::InvalidName)
    );
    Ok(())
}

#[test]
fn generated_literal_environment_sorting_is_stable_and_reset_bounded() -> Result<(), Box<dyn std::error::Error>> {
    let authored = concat!(
        "[Container]\n",
        "Image=example.invalid/application:1\n",
        "Environment=ZETA=authored-first\n",
        "Environment=ALPHA=authored-second\n",
    );
    let parsed = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(9_304), authored)?;
    assert_eq!(parsed.syntax().render_canonical()?, authored);

    let mut plan = ContainerEnvironmentPlan::new();
    plan.push_assignment(EnvironmentAssignment::new("ZETA", "first")?);
    plan.push_assignments(EnvironmentAssignments::new([
        EnvironmentAssignment::new("ALPHA", "before reset")?,
        EnvironmentAssignment::new("ZETA", "last")?,
    ])?);
    plan.push_reset();
    plan.push_assignments(EnvironmentAssignments::new([
        EnvironmentAssignment::new("BETA", "first")?,
        EnvironmentAssignment::new("ALPHA", "after reset")?,
        EnvironmentAssignment::new("BETA", "last")?,
    ])?);

    let sorted = plan.sorted_by_name();
    assert!(matches!(
        plan.directives(),
        [
            ContainerEnvironmentDirective::Assignment(_),
            ContainerEnvironmentDirective::Assignments(_),
            ContainerEnvironmentDirective::Reset(_),
            ContainerEnvironmentDirective::Assignments(_),
        ]
    ));
    let names = sorted
        .directives()
        .iter()
        .map(|directive| match directive {
            ContainerEnvironmentDirective::Assignment(assignment) => assignment.name(),
            ContainerEnvironmentDirective::Reset(_) => "<reset>",
            ContainerEnvironmentDirective::Assignments(_) => "<unexpected group>",
            _ => "<unknown>",
        })
        .collect::<Vec<_>>();
    assert_eq!(names, ["ALPHA", "ZETA", "ZETA", "<reset>", "ALPHA", "BETA", "BETA"]);
    assert_eq!(sorted.get("ALPHA"), plan.get("ALPHA"));
    assert_eq!(sorted.get("BETA"), plan.get("BETA"));
    assert_eq!(sorted.get("ZETA"), plan.get("ZETA"));

    let mut builder = QuadletDocumentBuilder::new(QuadletUnitType::Container);
    builder.push_container(ContainerKey::Image, EntryValue::new("example.invalid/application:1")?)?;
    builder.push_container_environment_plan(&sorted)?;
    assert_eq!(
        builder.build(SourceId::new(9_303))?.text(),
        concat!(
            "[Container]\n",
            "Image=example.invalid/application:1\n",
            "Environment=\"ALPHA=before reset\"\n",
            "Environment=\"ZETA=first\"\n",
            "Environment=\"ZETA=last\"\n",
            "Environment=\n",
            "Environment=\"ALPHA=after reset\"\n",
            "Environment=\"BETA=first\"\n",
            "Environment=\"BETA=last\"\n",
        )
    );
    Ok(())
}
