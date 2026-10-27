//! Protected native construction for deferred service-environment words.
use std::{error::Error, fmt};

use super::{GeneratedEntry, GeneratedValue, QuadletDocumentBuilder, RenderError};
use crate::model::{ContainerKey, EntryKind, QuadletUnitType, SectionKind};

/// Value-free failure at the focused deferred construction boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum DeferredValueError {
    /// A name is not ASCII `[A-Za-z_][A-Za-z0-9_]*`.
    InvalidName,
    /// A literal container key is empty, starts with whitespace, or contains equals/controls.
    InvalidContainerName,
    /// A literal contains control characters, including NUL or physical line endings.
    InvalidLiteral,
    /// A bind source is empty, non-absolute literal text, or contains native delimiters/references.
    InvalidBindSource,
    /// An absolute literal source cannot preserve native inferred mount dependencies.
    UnsupportedLiteralBindSource,
    /// A bind target is not an absolute literal path without native delimiters.
    InvalidBindTarget,
    /// A native bind option token contains a delimiter or unsupported byte.
    InvalidBindOption,
    /// A provider file is not an absolute literal path, or contains glob syntax.
    InvalidEnvironmentFile,
    /// An authored standard value could continue into a protected directive or section.
    UnsafeContinuation,
}
impl fmt::Display for DeferredValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidName => "invalid service environment name",
            Self::InvalidContainerName => "invalid literal container environment key",
            Self::InvalidLiteral => "invalid protected literal",
            Self::InvalidBindSource => "invalid deferred bind source",
            Self::UnsupportedLiteralBindSource => "absolute literal bind sources with dollars, whitespace, quotes, or backslashes cannot preserve inferred mount dependencies; use one leading service reference",
            Self::InvalidBindTarget => "invalid literal bind target",
            Self::InvalidBindOption => "invalid native bind option",
            Self::InvalidEnvironmentFile => "invalid literal service environment file",
            Self::UnsafeContinuation => "protected construction requires complete physical lines",
        })
    }
}
impl Error for DeferredValueError {}

/// Validated ASCII service environment name. Debug never exposes its spelling.
#[derive(Clone, Eq, PartialEq)]
pub struct ServiceEnvironmentName(String);
impl ServiceEnvironmentName {
    /// Creates an ASCII environment name.
    ///
    /// # Errors
    /// Returns [`DeferredValueError::InvalidName`] for an invalid name.
    pub fn new(name: impl Into<String>) -> Result<Self, DeferredValueError> {
        let name = name.into();
        if !super::is_environment_name(&name) {
            return Err(DeferredValueError::InvalidName);
        }
        Ok(Self(name))
    }
    /// Explicitly returns the validated name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl fmt::Debug for ServiceEnvironmentName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ServiceEnvironmentName(<redacted>)")
    }
}

/// Protected literal native container environment key, distinct from a service reference name.
#[derive(Clone, Eq, PartialEq)]
pub struct ContainerEnvironmentName(String);
impl ContainerEnvironmentName {
    /// Validates a nonempty literal key, preserving dots and other native key bytes.
    ///
    /// # Errors
    /// Returns [`DeferredValueError::InvalidContainerName`] for empty keys, leading whitespace,
    /// equals, NUL, or other control characters. This does not broaden reverse-name modeling.
    pub fn new(name: impl Into<String>) -> Result<Self, DeferredValueError> {
        let name = name.into();
        if name.is_empty()
            || name.contains('=')
            || name.chars().any(char::is_control)
            || name.chars().next().is_some_and(char::is_whitespace)
        {
            return Err(DeferredValueError::InvalidContainerName);
        }
        Ok(Self(name))
    }
    /// Explicitly returns the literal native key.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl From<ServiceEnvironmentName> for ContainerEnvironmentName {
    fn from(name: ServiceEnvironmentName) -> Self {
        Self(name.0)
    }
}
impl fmt::Debug for ContainerEnvironmentName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ContainerEnvironmentName(<redacted>)")
    }
}

/// One ordered literal or intentional service-environment reference.
#[derive(Clone, Eq, PartialEq)]
pub enum DeferredWordPart {
    /// Exact literal text, validated when the word is constructed.
    Literal(String),
    /// A validated reference, rendered canonically as `${NAME}`.
    Reference(ServiceEnvironmentName),
}
impl fmt::Debug for DeferredWordPart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Literal(_) => "Literal(<redacted>)",
            Self::Reference(_) => "Reference(<redacted>)",
        })
    }
}

/// Protected ordered native word. Empty words represent empty environment values.
#[derive(Clone, Eq, PartialEq)]
pub struct DeferredWord(Vec<DeferredWordPart>);
impl DeferredWord {
    /// Validates ordered word parts without expanding references.
    ///
    /// # Errors
    /// Returns [`DeferredValueError::InvalidLiteral`] for literal control characters.
    pub fn new(parts: impl IntoIterator<Item = DeferredWordPart>) -> Result<Self, DeferredValueError> {
        let parts: Vec<_> = parts.into_iter().collect();
        for part in &parts {
            if let DeferredWordPart::Literal(value) = part {
                validate_literal(value)?;
            }
        }
        Ok(Self(parts))
    }
    /// Explicitly returns ordered parts. Literal fields retain their original bytes.
    #[must_use]
    pub fn parts(&self) -> &[DeferredWordPart] {
        &self.0
    }
    fn exec_text(self) -> String {
        let mut text = String::new();
        for part in self.0 {
            match part {
                DeferredWordPart::Literal(value) => text.push_str(&exec_literal(&value)),
                DeferredWordPart::Reference(name) => {
                    text.push_str("${");
                    text.push_str(name.as_str());
                    text.push('}');
                }
            }
        }
        text
    }
}
impl fmt::Debug for DeferredWord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DeferredWord")
            .field("part_count", &self.0.len())
            .finish()
    }
}

/// Protected literal `[Service] Environment=` assignment; dollars are literal in this context.
#[derive(Clone, Eq, PartialEq)]
pub struct ServiceEnvironmentAssignment {
    name: ServiceEnvironmentName,
    value: String,
    rendered: String,
}
impl ServiceEnvironmentAssignment {
    /// Creates one literal service assignment, escaping literal percent characters.
    ///
    /// # Errors
    /// Returns [`DeferredValueError::InvalidLiteral`] for control characters.
    pub fn new(name: ServiceEnvironmentName, value: impl Into<String>) -> Result<Self, DeferredValueError> {
        let value = value.into();
        validate_literal(&value)?;
        let rendered = quote_word(&format!("{}={}", name.as_str(), value.replace('%', "%%")));
        Ok(Self { name, value, rendered })
    }
    /// Explicitly returns the assignment name.
    #[must_use]
    pub const fn name(&self) -> &ServiceEnvironmentName {
        &self.name
    }
    /// Explicitly returns the original literal value.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }
    /// Explicitly returns native service assignment syntax.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.rendered
    }
}
impl fmt::Debug for ServiceEnvironmentAssignment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ServiceEnvironmentAssignment(<redacted>)")
    }
}

/// Protected declaration of an externally provisioned absolute literal service environment file.
#[derive(Clone, Eq, PartialEq)]
pub struct ServiceEnvironmentFile {
    path: String,
    optional: bool,
    rendered: String,
}
impl ServiceEnvironmentFile {
    /// Creates a mandatory provider declaration without reading or creating its file.
    ///
    /// # Errors
    /// Returns [`DeferredValueError::InvalidEnvironmentFile`] for relative paths, controls, or glob syntax.
    pub fn new(path: impl Into<String>) -> Result<Self, DeferredValueError> {
        let path = path.into();
        if !path.starts_with('/') || validate_literal(&path).is_err() || path.contains(['*', '?', '[', ']']) {
            return Err(DeferredValueError::InvalidEnvironmentFile);
        }
        let rendered = quote_word(&path.replace('%', "%%"));
        Ok(Self {
            path,
            optional: false,
            rendered,
        })
    }
    /// Explicitly selects whether systemd may ignore a missing file. Default is mandatory.
    #[must_use]
    pub fn with_optional(mut self, optional: bool) -> Self {
        self.optional = optional;
        self.rendered = quote_word(&format!(
            "{}{}",
            if optional { "-" } else { "" },
            self.path.replace('%', "%%")
        ));
        self
    }
    /// Explicitly returns the original literal absolute path.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }
    /// Returns whether a missing provider file is explicitly optional.
    #[must_use]
    pub const fn is_optional(&self) -> bool {
        self.optional
    }
    /// Explicitly returns native provider declaration syntax.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.rendered
    }
}
impl fmt::Debug for ServiceEnvironmentFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ServiceEnvironmentFile")
            .field("optional", &self.optional)
            .finish_non_exhaustive()
    }
}

impl QuadletDocumentBuilder {
    /// Appends one protected container assignment with a literal key and ordered deferred value.
    ///
    /// Keys use [`ContainerEnvironmentName`]; strict [`ServiceEnvironmentName`] also converts
    /// infallibly. Literal key dollars/percents receive exec escaping, never reference semantics.
    ///
    /// # Errors
    /// Returns [`RenderError::WrongUnitType`] for a non-container document.
    pub fn push_container_environment_deferred(
        &mut self,
        name: impl Into<ContainerEnvironmentName>,
        value: DeferredWord,
    ) -> Result<(), RenderError> {
        let ContainerEnvironmentName(name) = name.into();
        self.push_protected_container(
            ContainerKey::Environment,
            "Environment",
            quote_word(&format!("{}={}", exec_literal(&name), value.exec_text())),
        )
    }
    /// Appends a native bind-source word with an absolute literal target and native option tokens.
    ///
    /// Accepts either a pure absolute literal word or one direct leading reference followed
    /// only by literals. Prefix/interior and multiple references are outside this focused boundary.
    /// Runtime provider values must yield nonempty absolute host paths without colons, NUL, or
    /// control characters. This method never acquires or validates those runtime values.
    /// Literal source colons and Quadlet-reference suffixes are rejected. Pure absolute literal
    /// sources also reject dollars, whitespace, quotes, and backslashes because older generators
    /// infer mount dependencies without the required escaping. Literal percent remains supported.
    /// Target endings containing whitespace/backslashes are rejected to avoid raw-field loss.
    ///
    /// # Errors
    /// Returns [`RenderError::DeferredValue`] for invalid source/target/options, or
    /// [`RenderError::WrongUnitType`] for a non-container document.
    pub fn push_container_bind_source_deferred(
        &mut self,
        source: DeferredWord,
        target: impl Into<String>,
        options: impl IntoIterator<Item = String>,
    ) -> Result<(), RenderError> {
        let starts_path = match source.parts().split_first() {
            Some((DeferredWordPart::Reference(_), suffix)) => {
                suffix.iter().all(|part| matches!(part, DeferredWordPart::Literal(_)))
            }
            _ => {
                source
                    .parts()
                    .iter()
                    .all(|part| matches!(part, DeferredWordPart::Literal(_)))
                    && source
                        .parts()
                        .iter()
                        .find(|part| !matches!(part, DeferredWordPart::Literal(value) if value.is_empty()))
                        .is_some_and(|part| matches!(part, DeferredWordPart::Literal(value) if value.starts_with('/')))
            }
        };
        let pure_literal = source
            .parts()
            .iter()
            .all(|part| matches!(part, DeferredWordPart::Literal(_)));
        if pure_literal && source.parts().iter().any(|part| matches!(part, DeferredWordPart::Literal(value) if value.chars().any(|character| character.is_whitespace() || matches!(character, '$' | '\'' | '"' | '\\')))) {
            return Err(DeferredValueError::UnsupportedLiteralBindSource.into());
        }
        let source_text = source.exec_text();
        if !starts_path
            || source_text.contains(':')
            || [".volume", ".artifact"]
                .iter()
                .any(|suffix| source_text.ends_with(suffix))
        {
            return Err(DeferredValueError::InvalidBindSource.into());
        }
        let target = target.into();
        if !target.starts_with('/')
            || target.contains(':')
            || validate_literal(&target).is_err()
            || target
                .chars()
                .last()
                .is_some_and(|character| character.is_whitespace() || character == '\\')
        {
            return Err(DeferredValueError::InvalidBindTarget.into());
        }
        let options: Vec<_> = options.into_iter().collect();
        if options.iter().any(|option| {
            option.is_empty()
                || !option
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'=' | b'-'))
        }) {
            return Err(DeferredValueError::InvalidBindOption.into());
        }
        let mut text = format!("{source_text}:{}", exec_literal(&target));
        if !options.is_empty() {
            text.push(':');
            text.push_str(&options.join(","));
        }
        self.push_protected_container(ContainerKey::Volume, "Volume", text)
    }
    /// Appends a literal service assignment. Files override inline assignments under systemd rules.
    ///
    /// # Errors
    /// This validated provider append is infallible; the result shares the builder API convention.
    pub fn push_service_environment_literal(
        &mut self,
        assignment: ServiceEnvironmentAssignment,
    ) -> Result<(), RenderError> {
        self.push_protected(
            SectionKind::Service,
            EntryKind::GenericSystemd,
            "Environment",
            assignment.rendered,
        );
        Ok(())
    }
    /// Appends an externally provisioned provider file in caller order; no file access occurs.
    ///
    /// # Errors
    /// This validated provider append is infallible; the result shares the builder API convention.
    pub fn push_service_environment_file(&mut self, file: ServiceEnvironmentFile) -> Result<(), RenderError> {
        self.push_protected(
            SectionKind::Service,
            EntryKind::GenericSystemd,
            "EnvironmentFile",
            file.rendered,
        );
        Ok(())
    }
    fn push_protected_container(&mut self, key: ContainerKey, name: &str, value: String) -> Result<(), RenderError> {
        if self.unit_type != QuadletUnitType::Container {
            return Err(RenderError::WrongUnitType {
                document: self.unit_type,
                entry: QuadletUnitType::Container,
            });
        }
        self.push_protected(SectionKind::Container, EntryKind::Container(key), name, value);
        Ok(())
    }
    fn push_protected(&mut self, section: SectionKind, kind: EntryKind, key: &str, value: String) {
        self.entries.push(GeneratedEntry {
            section,
            kind,
            key: key.to_owned(),
            value: GeneratedValue::Protected(value),
        });
    }
}
fn validate_literal(value: &str) -> Result<(), DeferredValueError> {
    if value.chars().any(char::is_control) {
        Err(DeferredValueError::InvalidLiteral)
    } else {
        Ok(())
    }
}
fn exec_literal(value: &str) -> String {
    value.replace('$', "$$").replace('%', "%%")
}
fn quote_word(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}
