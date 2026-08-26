//! Caller-authorized external container environment resolution.

use std::{collections::BTreeMap, error::Error, fmt};

use crate::{
    diagnostic::{Diagnostic, DiagnosticCode, Label, Severity},
    source::SourceSpan,
};

use super::{
    AuthoredContainerEnvironment, ContainerKey, EntryKind, QuadletDocument, is_authored_environment_name,
    logical_authored_value, systemd_environment_tokens,
};

const MALFORMED_ENVIRONMENT_FILE_REFERENCE: DiagnosticCode = DiagnosticCode::new("QLM0025");
const DEFERRED_ENVIRONMENT_FILE_REFERENCE: DiagnosticCode = DiagnosticCode::new("QLM0026");
const MALFORMED_ENVIRONMENT_SECRET_REFERENCE: DiagnosticCode = DiagnosticCode::new("QLM0027");
const DEFERRED_ENVIRONMENT_SECRET_REFERENCE: DiagnosticCode = DiagnosticCode::new("QLM0028");

/// How much `QuadletLens` can know about one external environment reference without host access.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum EnvironmentReferenceState {
    /// The reference is a complete literal name or path.
    Literal,
    /// The reference contains a systemd specifier requiring target-manager context.
    Deferred,
    /// The reference could not be interpreted by the bounded native model.
    Unmodeled,
}

/// One source-located `[Container] EnvironmentFile=` reference.
#[derive(Clone, Eq, PartialEq)]
pub struct EnvironmentFileReference {
    path: Option<String>,
    optional: bool,
    state: EnvironmentReferenceState,
    span: SourceSpan,
}

impl EnvironmentFileReference {
    /// Returns the decoded reference path when its syntax was modeled.
    #[must_use]
    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }

    /// Reports whether a leading `-` permits the native source to be absent.
    #[must_use]
    pub const fn optional(&self) -> bool {
        self.optional
    }

    /// Returns the reference classification without resolving systemd specifiers.
    #[must_use]
    pub const fn state(&self) -> EnvironmentReferenceState {
        self.state
    }

    /// Returns the span of the complete authored native value.
    #[must_use]
    pub const fn span(&self) -> SourceSpan {
        self.span
    }
}

impl fmt::Debug for EnvironmentFileReference {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EnvironmentFileReference")
            .field("path", &"<redacted environment-file path>")
            .field("optional", &self.optional)
            .field("state", &self.state)
            .field("span", &self.span)
            .finish()
    }
}

/// One source-located `Secret=...,type=env` reference.
#[derive(Clone, Eq, PartialEq)]
pub struct EnvironmentSecretReference {
    secret: Option<String>,
    target: Option<String>,
    state: EnvironmentReferenceState,
    span: SourceSpan,
}

impl EnvironmentSecretReference {
    /// Returns the Podman secret name when its syntax was modeled.
    #[must_use]
    pub fn secret(&self) -> Option<&str> {
        self.secret.as_deref()
    }

    /// Returns the target environment name when its syntax was modeled.
    #[must_use]
    pub fn target(&self) -> Option<&str> {
        self.target.as_deref()
    }

    /// Returns the reference classification without reading a secret store.
    #[must_use]
    pub const fn state(&self) -> EnvironmentReferenceState {
        self.state
    }

    /// Returns the span of the complete authored native value.
    #[must_use]
    pub const fn span(&self) -> SourceSpan {
        self.span
    }
}

impl fmt::Debug for EnvironmentSecretReference {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EnvironmentSecretReference")
            .field("secret", &self.secret)
            .field("target", &self.target)
            .field("state", &self.state)
            .field("span", &self.span)
            .finish()
    }
}

/// Source-preserving container environment view, including unresolved external references.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContainerEnvironmentSources {
    inline: AuthoredContainerEnvironment,
    files: Vec<EnvironmentFileReference>,
    secrets: Vec<EnvironmentSecretReference>,
    diagnostics: Vec<Diagnostic>,
}

impl ContainerEnvironmentSources {
    fn from_document(document: &QuadletDocument) -> Self {
        let inline = document.container_environment();
        let mut files = Vec::new();
        let mut secrets = Vec::new();
        let mut diagnostics = Vec::new();

        for entry in document.entries() {
            match entry.kind() {
                EntryKind::Container(ContainerKey::EnvironmentFile) => {
                    let span = entry.value().primary().span();
                    let Some(logical) = logical_authored_value(entry) else {
                        files.push(unmodeled_file(span));
                        diagnostics.push(reference_diagnostic(
                            MALFORMED_ENVIRONMENT_FILE_REFERENCE,
                            span,
                            "container EnvironmentFile reference could not be decoded",
                            "preserve the source and supply one physical, quoted systemd path",
                        ));
                        continue;
                    };
                    let Some(tokens) = systemd_environment_tokens(&logical) else {
                        files.push(unmodeled_file(span));
                        diagnostics.push(reference_diagnostic(
                            MALFORMED_ENVIRONMENT_FILE_REFERENCE,
                            span,
                            "container EnvironmentFile reference has malformed quoting or escaping",
                            "use one valid systemd word for the environment-file path",
                        ));
                        continue;
                    };
                    if tokens.len() != 1 || tokens[0].is_empty() {
                        files.push(unmodeled_file(span));
                        diagnostics.push(reference_diagnostic(
                            MALFORMED_ENVIRONMENT_FILE_REFERENCE,
                            span,
                            "container EnvironmentFile reference does not contain exactly one path",
                            "use one non-empty path; a leading - may mark it optional",
                        ));
                        continue;
                    }
                    let (optional, path) = tokens[0]
                        .strip_prefix('-')
                        .map_or((false, tokens[0].as_str()), |path| (true, path));
                    if path.is_empty() {
                        files.push(unmodeled_file(span));
                        diagnostics.push(reference_diagnostic(
                            MALFORMED_ENVIRONMENT_FILE_REFERENCE,
                            span,
                            "container EnvironmentFile reference has an empty path",
                            "add a path after the optional leading - marker",
                        ));
                        continue;
                    }
                    let state = if path.contains('%') {
                        diagnostics.push(reference_diagnostic(
                            DEFERRED_ENVIRONMENT_FILE_REFERENCE,
                            span,
                            "container EnvironmentFile path contains an unexpanded systemd specifier",
                            "resolve the path in the target manager context before authorizing file values",
                        ));
                        EnvironmentReferenceState::Deferred
                    } else {
                        EnvironmentReferenceState::Literal
                    };
                    files.push(EnvironmentFileReference {
                        path: Some(path.to_owned()),
                        optional,
                        state,
                        span,
                    });
                }
                EntryKind::Container(ContainerKey::Secret) => {
                    parse_environment_secret(entry, &mut secrets, &mut diagnostics);
                }
                _ => {}
            }
        }

        Self {
            inline,
            files,
            secrets,
            diagnostics,
        }
    }

    /// Returns the existing ordered inline `Environment=` semantic view.
    #[must_use]
    pub const fn inline(&self) -> &AuthoredContainerEnvironment {
        &self.inline
    }

    /// Returns environment-file references in authored entry order.
    #[must_use]
    pub fn environment_files(&self) -> &[EnvironmentFileReference] {
        &self.files
    }

    /// Returns environment-exposing secret references in authored entry order.
    #[must_use]
    pub fn environment_secrets(&self) -> &[EnvironmentSecretReference] {
        &self.secrets
    }

    /// Returns value-free diagnostics for malformed or deferred external references.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Associates only explicitly caller-authorized external values with these references.
    ///
    /// This operation performs no filesystem, process-environment, Podman, or secret-store
    /// access. Deferred and unmodeled references remain unresolved even if an authorization
    /// happens to use matching text.
    #[must_use]
    pub fn resolve(&self, authorized: &AuthorizedContainerEnvironment) -> ContainerEnvironmentResolution {
        let files = self
            .files
            .iter()
            .cloned()
            .map(|reference| {
                let assignments = (reference.state == EnvironmentReferenceState::Literal)
                    .then(|| reference.path.as_deref().and_then(|path| authorized.files.get(path)))
                    .flatten()
                    .cloned();
                EnvironmentFileResolution { reference, assignments }
            })
            .collect();
        let secrets = self
            .secrets
            .iter()
            .cloned()
            .map(|reference| {
                let value = (reference.state == EnvironmentReferenceState::Literal)
                    .then(|| {
                        reference
                            .secret
                            .as_deref()
                            .and_then(|secret| authorized.secrets.get(secret))
                    })
                    .flatten()
                    .cloned();
                EnvironmentSecretResolution { reference, value }
            })
            .collect();
        ContainerEnvironmentResolution {
            inline: self.inline.clone(),
            files,
            secrets,
        }
    }
}

impl QuadletDocument {
    /// Returns inline environment syntax and unresolved external environment references.
    ///
    /// Discovery is source-only. Call [`ContainerEnvironmentSources::resolve`] with explicit
    /// caller-owned values when external content is authorized.
    #[must_use]
    pub fn container_environment_sources(&self) -> ContainerEnvironmentSources {
        ContainerEnvironmentSources::from_document(self)
    }
}

fn unmodeled_file(span: SourceSpan) -> EnvironmentFileReference {
    EnvironmentFileReference {
        path: None,
        optional: false,
        state: EnvironmentReferenceState::Unmodeled,
        span,
    }
}

fn parse_environment_secret(
    entry: &super::TypedEntry,
    secrets: &mut Vec<EnvironmentSecretReference>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let span = entry.value().primary().span();
    let Some(logical) = logical_authored_value(entry) else {
        return;
    };
    let Some(tokens) = systemd_environment_tokens(&logical) else {
        return;
    };
    if tokens.len() != 1 {
        return;
    }
    let fields: Vec<_> = tokens[0].split(',').collect();
    let Some(secret) = fields.first().copied().filter(|secret| !secret.is_empty()) else {
        return;
    };
    let mut kind = None;
    let mut target = None;
    let mut malformed = false;
    for field in fields.iter().skip(1) {
        let Some((name, value)) = field.split_once('=') else {
            malformed = true;
            break;
        };
        match name {
            "type" if kind.replace(value).is_some() => malformed = true,
            "target" if target.replace(value).is_some() => malformed = true,
            _ => {}
        }
    }
    if kind != Some("env") {
        return;
    }
    let target = target.unwrap_or(secret);
    if malformed {
        secrets.push(EnvironmentSecretReference {
            secret: None,
            target: None,
            state: EnvironmentReferenceState::Unmodeled,
            span,
        });
        diagnostics.push(reference_diagnostic(
            MALFORMED_ENVIRONMENT_SECRET_REFERENCE,
            span,
            "container environment secret reference is malformed",
            "use Secret=NAME,type=env,target=ASCII_ENVIRONMENT_NAME",
        ));
        return;
    }
    let state = if secret.contains('%') || target.contains('%') {
        diagnostics.push(reference_diagnostic(
            DEFERRED_ENVIRONMENT_SECRET_REFERENCE,
            span,
            "container environment secret reference contains an unexpanded systemd specifier",
            "resolve the reference in target context before authorizing a secret payload",
        ));
        EnvironmentReferenceState::Deferred
    } else if !is_authored_environment_name(target) {
        secrets.push(EnvironmentSecretReference {
            secret: None,
            target: None,
            state: EnvironmentReferenceState::Unmodeled,
            span,
        });
        diagnostics.push(reference_diagnostic(
            MALFORMED_ENVIRONMENT_SECRET_REFERENCE,
            span,
            "container environment secret target is not a portable environment name",
            "use Secret=NAME,type=env,target=ASCII_ENVIRONMENT_NAME",
        ));
        return;
    } else {
        EnvironmentReferenceState::Literal
    };
    secrets.push(EnvironmentSecretReference {
        secret: Some(secret.to_owned()),
        target: Some(target.to_owned()),
        state,
        span,
    });
}

fn reference_diagnostic(
    code: DiagnosticCode,
    span: SourceSpan,
    summary: &'static str,
    help: &'static str,
) -> Diagnostic {
    Diagnostic::new(code, Severity::Warning, summary, Label::new(span, help))
}

/// Secret or external environment value whose formatting never reveals its payload.
#[derive(Clone, Eq, PartialEq)]
pub struct SensitiveEnvironmentValue(String);

impl SensitiveEnvironmentValue {
    /// Creates a caller-owned value. Empty values are distinct from missing authorization.
    ///
    /// # Errors
    ///
    /// Returns [`EnvironmentValueError::Nul`] because process environment values cannot contain
    /// a NUL byte.
    pub fn new(value: impl Into<String>) -> Result<Self, EnvironmentValueError> {
        let value = value.into();
        if value.contains('\0') {
            return Err(EnvironmentValueError::Nul);
        }
        Ok(Self(value))
    }

    /// Explicitly exposes the protected payload to an authorized caller.
    #[must_use]
    pub fn expose_secret(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SensitiveEnvironmentValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("<redacted environment value>")
    }
}

/// One decoded assignment supplied by the caller for an authorized environment file.
#[derive(Clone, Eq, PartialEq)]
pub struct AuthorizedEnvironmentAssignment {
    name: String,
    value: SensitiveEnvironmentValue,
}

impl AuthorizedEnvironmentAssignment {
    /// Creates one decoded external assignment.
    ///
    /// # Errors
    ///
    /// Returns [`EnvironmentValueError::InvalidName`] for a non-portable environment name.
    pub fn new(name: impl Into<String>, value: SensitiveEnvironmentValue) -> Result<Self, EnvironmentValueError> {
        let name = name.into();
        if !is_authored_environment_name(&name) {
            return Err(EnvironmentValueError::InvalidName);
        }
        Ok(Self { name, value })
    }

    /// Returns the validated assignment name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the protected value wrapper; payload access remains explicit.
    #[must_use]
    pub const fn value(&self) -> &SensitiveEnvironmentValue {
        &self.value
    }
}

impl fmt::Debug for AuthorizedEnvironmentAssignment {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AuthorizedEnvironmentAssignment")
            .field("name", &self.name)
            .field("value", &self.value)
            .finish()
    }
}

/// Explicit caller authorization for already-decoded external environment values.
#[derive(Clone, Default, Eq, PartialEq)]
pub struct AuthorizedContainerEnvironment {
    files: BTreeMap<String, Vec<AuthorizedEnvironmentAssignment>>,
    secrets: BTreeMap<String, SensitiveEnvironmentValue>,
}

impl AuthorizedContainerEnvironment {
    /// Creates empty authorization; all external references remain unresolved.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            files: BTreeMap::new(),
            secrets: BTreeMap::new(),
        }
    }

    /// Authorizes decoded assignments for one exact literal `EnvironmentFile=` path.
    ///
    /// # Errors
    ///
    /// Returns [`EnvironmentValueError::InvalidReference`] for an empty path or a path containing
    /// a NUL byte.
    pub fn authorize_environment_file(
        &mut self,
        path: impl Into<String>,
        assignments: impl IntoIterator<Item = AuthorizedEnvironmentAssignment>,
    ) -> Result<(), EnvironmentValueError> {
        let path = path.into();
        if path.is_empty() || path.contains('\0') {
            return Err(EnvironmentValueError::InvalidReference);
        }
        self.files.insert(path, assignments.into_iter().collect());
        Ok(())
    }

    /// Authorizes one payload for one exact literal Podman secret name.
    ///
    /// # Errors
    ///
    /// Returns [`EnvironmentValueError::InvalidReference`] for an empty name or a name containing
    /// a NUL byte.
    pub fn authorize_secret(
        &mut self,
        secret: impl Into<String>,
        value: SensitiveEnvironmentValue,
    ) -> Result<(), EnvironmentValueError> {
        let secret = secret.into();
        if secret.is_empty() || secret.contains('\0') {
            return Err(EnvironmentValueError::InvalidReference);
        }
        self.secrets.insert(secret, value);
        Ok(())
    }
}

impl fmt::Debug for AuthorizedContainerEnvironment {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AuthorizedContainerEnvironment")
            .field("environment_file_count", &self.files.len())
            .field("secret_count", &self.secrets.len())
            .finish_non_exhaustive()
    }
}

/// Invalid explicitly authorized external environment input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum EnvironmentValueError {
    /// Assignment name does not match ASCII `[A-Za-z_][A-Za-z0-9_]*`.
    InvalidName,
    /// A value contains a NUL byte.
    Nul,
    /// A lookup reference is empty or contains a NUL byte.
    InvalidReference,
}

impl fmt::Display for EnvironmentValueError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidName => formatter.write_str("environment names must match ASCII [A-Za-z_][A-Za-z0-9_]*"),
            Self::Nul => formatter.write_str("environment values must not contain NUL bytes"),
            Self::InvalidReference => {
                formatter.write_str("environment references must be non-empty and contain no NUL bytes")
            }
        }
    }
}

impl Error for EnvironmentValueError {}

/// Caller-authorized result for one environment-file reference.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnvironmentFileResolution {
    reference: EnvironmentFileReference,
    assignments: Option<Vec<AuthorizedEnvironmentAssignment>>,
}

impl EnvironmentFileResolution {
    /// Returns source-located reference regardless of resolution outcome.
    #[must_use]
    pub const fn reference(&self) -> &EnvironmentFileReference {
        &self.reference
    }

    /// Returns caller-authorized decoded assignments, or `None` when unresolved.
    #[must_use]
    pub fn assignments(&self) -> Option<&[AuthorizedEnvironmentAssignment]> {
        self.assignments.as_deref()
    }
}

/// Caller-authorized result for one environment-secret reference.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnvironmentSecretResolution {
    reference: EnvironmentSecretReference,
    value: Option<SensitiveEnvironmentValue>,
}

impl EnvironmentSecretResolution {
    /// Returns source-located reference regardless of resolution outcome.
    #[must_use]
    pub const fn reference(&self) -> &EnvironmentSecretReference {
        &self.reference
    }

    /// Returns protected caller-authorized payload, or `None` when unresolved.
    #[must_use]
    pub const fn value(&self) -> Option<&SensitiveEnvironmentValue> {
        self.value.as_ref()
    }
}

/// Inline source view plus caller-authorized external environment resolutions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContainerEnvironmentResolution {
    inline: AuthoredContainerEnvironment,
    files: Vec<EnvironmentFileResolution>,
    secrets: Vec<EnvironmentSecretResolution>,
}

impl ContainerEnvironmentResolution {
    /// Returns ordered inline source semantics unchanged by external resolution.
    #[must_use]
    pub const fn inline(&self) -> &AuthoredContainerEnvironment {
        &self.inline
    }

    /// Returns environment-file outcomes in authored entry order.
    #[must_use]
    pub fn environment_files(&self) -> &[EnvironmentFileResolution] {
        &self.files
    }

    /// Returns environment-secret outcomes in authored entry order.
    #[must_use]
    pub fn environment_secrets(&self) -> &[EnvironmentSecretResolution] {
        &self.secrets
    }
}
