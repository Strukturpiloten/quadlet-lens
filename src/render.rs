//! Validated programmatic construction of deterministic Quadlet documents.
//!
//! This module owns native section and key spelling, repeated-entry rules, physical-line safety,
//! deterministic section order, and parse-back validation. Entry values remain exact authored
//! values: callers are responsible for key-specific semantic validity, while future focused value
//! encoders can provide stronger construction APIs without changing the document builder.

use std::{error::Error, fmt};

use crate::{
    diagnostic::Diagnostic,
    model::{
        ArtifactKey, BuildKey, ContainerKey, EntryKind, ImageKey, KubeKey, NetworkKey, PodKey, QuadletDocument,
        QuadletKey, QuadletParseResult, QuadletUnitType, SectionKind, SystemdUnitKey, TypedModelError, VolumeKey,
    },
    source::SourceId,
};

/// An exact, single-physical-line native Quadlet value.
///
/// The value is retained exactly. It may contain native systemd quoting and specifiers, but it may
/// not contain line endings or NUL bytes. This type enforces physical-line safety only; it does
/// not validate command arguments, environment assignments, lifecycle values, mount options, or
/// other key-specific semantics.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EntryValue(String);

impl EntryValue {
    /// Creates an exact native value that fits on one physical line.
    ///
    /// # Errors
    ///
    /// Returns [`RenderError::InvalidValue`] when the value contains a line ending or NUL byte.
    pub fn new(value: impl Into<String>) -> Result<Self, RenderError> {
        let value = value.into();
        if value.bytes().any(|byte| matches!(byte, 0 | b'\n' | b'\r')) {
            return Err(RenderError::InvalidValue);
        }
        Ok(Self(value))
    }

    /// Returns the exact native spelling.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Safely constructible process-ID limit for a container.
///
/// This helper covers only the documented unlimited spelling (`-1`) and positive finite values
/// written as nonzero ASCII decimal text. It deliberately does not parse the decimal into a Rust
/// integer, so large values and leading zeros retain their exact spelling without overflow.
/// Parsed and raw [`EntryValue`] inputs remain uninterpreted, so authored zero and noncanonical
/// values can still be preserved.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PidsLimit(String);

impl PidsLimit {
    /// Creates an unlimited process-ID limit rendered as `-1`.
    #[must_use]
    pub fn unlimited() -> Self {
        Self("-1".to_owned())
    }

    /// Creates a positive finite process-ID limit from exact ASCII decimal spelling.
    ///
    /// # Errors
    ///
    /// Returns [`PidsLimitError::Empty`] for empty text, [`PidsLimitError::NonDecimal`] for any
    /// non-ASCII-digit byte, and [`PidsLimitError::Zero`] when every digit is zero.
    pub fn finite(limit: impl Into<String>) -> Result<Self, PidsLimitError> {
        let limit = limit.into();
        if limit.is_empty() {
            return Err(PidsLimitError::Empty);
        }
        if !limit.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(PidsLimitError::NonDecimal);
        }
        if !limit.bytes().any(|byte| byte != b'0') {
            return Err(PidsLimitError::Zero);
        }
        Ok(Self(limit))
    }

    /// Returns the exact native spelling.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<PidsLimit> for EntryValue {
    fn from(limit: PidsLimit) -> Self {
        Self(limit.0)
    }
}

/// Invalid input to [`PidsLimit::finite`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PidsLimitError {
    /// The finite decimal spelling is empty.
    Empty,
    /// The finite spelling contains a byte other than an ASCII decimal digit.
    NonDecimal,
    /// Zero is deliberately outside the typed construction contract.
    Zero,
}

impl fmt::Display for PidsLimitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("a finite process-ID limit must not be empty"),
            Self::NonDecimal => formatter.write_str("a finite process-ID limit must contain only ASCII decimal digits"),
            Self::Zero => formatter.write_str("a finite process-ID limit must be positive"),
        }
    }
}

impl Error for PidsLimitError {}

/// Safely constructible native shared-memory size for a container or pod.
///
/// The exact spelling is retained without parsing into a machine integer. Accepted values contain
/// a non-negative ASCII-decimal amount followed by no unit or one lowercase native unit: `b`,
/// `k`, `m`, or `g`. Leading zeros and arbitrary-precision amounts remain unchanged. Parsed and
/// raw [`EntryValue`] inputs remain uninterpreted.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShmSize(String);

impl ShmSize {
    /// Creates a shared-memory size from exact native spelling.
    ///
    /// # Errors
    ///
    /// Returns [`ShmSizeError::Empty`] for empty text and [`ShmSizeError::InvalidFormat`] unless
    /// the value is an ASCII-decimal amount with an optional lowercase `b`, `k`, `m`, or `g` unit.
    pub fn new(size: impl Into<String>) -> Result<Self, ShmSizeError> {
        let size = size.into();
        if size.is_empty() {
            return Err(ShmSizeError::Empty);
        }
        let amount = match size.as_bytes().last() {
            Some(b'b' | b'k' | b'm' | b'g') => &size[..size.len() - 1],
            _ => size.as_str(),
        };
        if amount.is_empty() || !amount.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(ShmSizeError::InvalidFormat);
        }
        Ok(Self(size))
    }

    /// Creates Podman's documented explicit unlimited shared-memory value, `0`.
    #[must_use]
    pub fn unlimited() -> Self {
        Self("0".to_owned())
    }

    /// Returns whether the exact spelling denotes a zero amount, Podman's documented unlimited value.
    #[must_use]
    pub fn is_unlimited(&self) -> bool {
        let amount = match self.0.as_bytes().last() {
            Some(b'b' | b'k' | b'm' | b'g') => &self.0[..self.0.len() - 1],
            _ => self.0.as_str(),
        };
        amount.bytes().all(|byte| byte == b'0')
    }

    /// Returns the exact native spelling.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<ShmSize> for EntryValue {
    fn from(size: ShmSize) -> Self {
        Self(size.0)
    }
}

/// Invalid input to [`ShmSize::new`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ShmSizeError {
    /// The shared-memory size is empty.
    Empty,
    /// The value is not an ASCII-decimal amount with an optional supported lowercase unit.
    InvalidFormat,
}

impl fmt::Display for ShmSizeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("a shared-memory size must not be empty"),
            Self::InvalidFormat => formatter
                .write_str("a shared-memory size must be an ASCII decimal amount with optional unit b, k, m, or g"),
        }
    }
}

impl Error for ShmSizeError {}

/// Safely constructible native memory limit for a container.
///
/// The exact spelling is retained without parsing into a machine integer. Accepted values contain
/// a positive ASCII-decimal amount followed by no unit or one lowercase native unit: `b`, `k`,
/// `m`, or `g`. Leading zeros and arbitrary-precision amounts remain unchanged. Parsed and raw
/// [`EntryValue`] inputs remain uninterpreted.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Memory(String);

impl Memory {
    /// Creates a positive memory limit from exact native spelling.
    ///
    /// # Errors
    ///
    /// Returns [`MemoryError::Empty`] for empty text, [`MemoryError::InvalidFormat`] unless the
    /// value is an ASCII-decimal amount with an optional lowercase `b`, `k`, `m`, or `g` unit,
    /// and [`MemoryError::Zero`] when every amount digit is zero.
    pub fn new(memory: impl Into<String>) -> Result<Self, MemoryError> {
        let memory = memory.into();
        if memory.is_empty() {
            return Err(MemoryError::Empty);
        }
        let amount = match memory.as_bytes().last() {
            Some(b'b' | b'k' | b'm' | b'g') => &memory[..memory.len() - 1],
            _ => memory.as_str(),
        };
        if amount.is_empty() || !amount.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(MemoryError::InvalidFormat);
        }
        if !amount.bytes().any(|byte| byte != b'0') {
            return Err(MemoryError::Zero);
        }
        Ok(Self(memory))
    }

    /// Returns the exact native spelling.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<Memory> for EntryValue {
    fn from(memory: Memory) -> Self {
        Self(memory.0)
    }
}

/// Invalid input to [`Memory::new`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum MemoryError {
    /// The memory-limit spelling is empty.
    Empty,
    /// The value is not an ASCII-decimal amount with an optional supported lowercase unit.
    InvalidFormat,
    /// A memory limit must be positive.
    Zero,
}

impl fmt::Display for MemoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("a memory limit must not be empty"),
            Self::InvalidFormat => {
                formatter.write_str("a memory limit must be an ASCII decimal amount with optional unit b, k, m, or g")
            }
            Self::Zero => formatter.write_str("a memory limit must be positive"),
        }
    }
}

impl Error for MemoryError {}

/// Generic systemd section supported in generated Quadlet files.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SystemdSection {
    /// The generic `[Unit]` section.
    Unit,
    /// The generic `[Service]` section.
    Service,
    /// The generic `[Install]` section.
    Install,
}

impl SystemdSection {
    const fn kind(self) -> SectionKind {
        match self {
            Self::Unit => SectionKind::Unit,
            Self::Service => SectionKind::Service,
            Self::Install => SectionKind::Install,
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
struct GeneratedEntry {
    section: SectionKind,
    kind: EntryKind,
    key: String,
    value: EntryValue,
}

impl fmt::Debug for GeneratedEntry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = formatter.debug_struct("GeneratedEntry");
        debug
            .field("section", &self.section)
            .field("kind", &self.kind)
            .field("key", &self.key);
        if self.kind.has_sensitive_value() {
            debug.field("value", &"<redacted sensitive value>")
        } else {
            debug.field("value", &self.value)
        };
        debug.finish()
    }
}

/// Ordered builder for one supported Quadlet document.
///
/// Native entries use typed keys and must match the selected unit type. Repeated keys retain
/// insertion order; duplicate singleton native keys are rejected. Generated sections use the
/// deterministic order `[Unit]`, the selected native section, `[Service]`, and `[Install]`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuadletDocumentBuilder {
    unit_type: QuadletUnitType,
    entries: Vec<GeneratedEntry>,
}

impl QuadletDocumentBuilder {
    /// Creates an empty document with the selected required native section.
    #[must_use]
    pub const fn new(unit_type: QuadletUnitType) -> Self {
        Self {
            unit_type,
            entries: Vec::new(),
        }
    }

    /// Returns the selected native unit type.
    #[must_use]
    pub const fn unit_type(&self) -> QuadletUnitType {
        self.unit_type
    }

    /// Appends a typed `[Container]` entry.
    ///
    /// # Errors
    ///
    /// Returns [`RenderError::WrongUnitType`] for a non-container document and
    /// [`RenderError::DuplicateSingleton`] for a repeated singleton key.
    pub fn push_container(&mut self, key: ContainerKey, value: EntryValue) -> Result<(), RenderError> {
        let attempted = container_key_name(key);
        if let Some(existing) = match key {
            ContainerKey::ReloadCmd => self.entries.iter().find_map(|entry| {
                (entry.kind == EntryKind::Container(ContainerKey::ReloadSignal)).then_some("ReloadSignal")
            }),
            ContainerKey::ReloadSignal => self
                .entries
                .iter()
                .find_map(|entry| (entry.kind == EntryKind::Container(ContainerKey::ReloadCmd)).then_some("ReloadCmd")),
            _ => None,
        } {
            return Err(RenderError::ConflictingSingletons {
                existing: existing.to_owned(),
                attempted: attempted.to_owned(),
            });
        }
        self.push_native(
            QuadletUnitType::Container,
            SectionKind::Container,
            EntryKind::Container(key),
            container_key_name(key),
            value,
        )
    }

    /// Appends a typed `[Pod]` entry.
    ///
    /// # Errors
    ///
    /// Returns [`RenderError::WrongUnitType`] for a non-pod document and
    /// [`RenderError::DuplicateSingleton`] for a repeated singleton key.
    pub fn push_pod(&mut self, key: PodKey, value: EntryValue) -> Result<(), RenderError> {
        self.push_native(
            QuadletUnitType::Pod,
            SectionKind::Pod,
            EntryKind::Pod(key),
            pod_key_name(key),
            value,
        )
    }

    /// Appends a typed `[Network]` entry.
    ///
    /// # Errors
    ///
    /// Returns [`RenderError::WrongUnitType`] for a non-network document and
    /// [`RenderError::DuplicateSingleton`] for a repeated singleton key.
    pub fn push_network(&mut self, key: NetworkKey, value: EntryValue) -> Result<(), RenderError> {
        self.push_native(
            QuadletUnitType::Network,
            SectionKind::Network,
            EntryKind::Network(key),
            network_key_name(key),
            value,
        )
    }

    /// Appends a typed `[Volume]` entry.
    ///
    /// # Errors
    ///
    /// Returns [`RenderError::WrongUnitType`] for a non-volume document and
    /// [`RenderError::DuplicateSingleton`] for a repeated singleton key.
    pub fn push_volume(&mut self, key: VolumeKey, value: EntryValue) -> Result<(), RenderError> {
        self.push_native(
            QuadletUnitType::Volume,
            SectionKind::Volume,
            EntryKind::Volume(key),
            volume_key_name(key),
            value,
        )
    }

    /// Appends a typed `[Build]` entry.
    ///
    /// `ImageTag`, `File`, `Network`, `Label`, `BuildArg`, `Secret`, `GroupAdd`, `DNS`, `DNSOption`, `DNSSearch`,
    /// `Annotation`, `Environment`, `ContainersConfModule`, `GlobalArgs`, `Volume`, and `PodmanArgs` entries remain repeatable and ordered; `SetWorkingDirectory`, `Target`, `Arch`, `Variant`,
    /// `Pull`, `Retry`, `RetryDelay`, `TLSVerify`, `ForceRM`, `AuthFile`, `IgnoreFile`, and `ServiceName` are singletons. Values are exact
    /// physical-line-safe native text and are not interpreted by the builder.
    ///
    /// # Errors
    ///
    /// Returns [`RenderError::WrongUnitType`] for a non-build document and
    /// [`RenderError::DuplicateSingleton`] for a repeated singleton Build key.
    pub fn push_build(&mut self, key: BuildKey, value: EntryValue) -> Result<(), RenderError> {
        self.push_native(
            QuadletUnitType::Build,
            SectionKind::Build,
            EntryKind::Build(key),
            build_key_name(key),
            value,
        )
    }

    /// Appends a typed `[Image]` entry.
    ///
    /// `ContainersConfModule`, `GlobalArgs`, and `PodmanArgs` entries remain repeatable and ordered. All other Image keys, including `OS`, are
    /// singletons. `Creds` and `DecryptionKey` are debug-redacted after key assignment, while
    /// explicit rendering and raw-value access remain exact. Values are exact physical-line-safe
    /// native text and are not interpreted by the builder; it does not read paths or modules, parse configuration,
    /// validate a CLI, or model pull, runtime, graph, or conversion semantics.
    ///
    /// # Errors
    ///
    /// Returns [`RenderError::WrongUnitType`] for a non-image document and
    /// [`RenderError::DuplicateSingleton`] only for a repeated singleton Image key.
    pub fn push_image(&mut self, key: ImageKey, value: EntryValue) -> Result<(), RenderError> {
        self.push_native(
            QuadletUnitType::Image,
            SectionKind::Image,
            EntryKind::Image(key),
            image_key_name(key),
            value,
        )
    }

    /// Appends a typed `[Kube]` entry.
    ///
    /// `AutoUpdate`, `ConfigMap`, `ContainersConfModule`, `GlobalArgs`, `LogOpt`, `RemapGid`,
    /// `RemapUid`, `Network`,
    /// `PodmanArgs`, `PublishPort`, and required `Yaml` entries remain repeatable and ordered. All other Kube keys are
    /// singletons. Values are exact physical-line-safe native text; this builder neither reads
    /// files nor parses Kubernetes YAML, Podman arguments, ports, paths, or runtime behavior.
    ///
    /// # Errors
    ///
    /// Returns [`RenderError::WrongUnitType`] for a non-Kube document,
    /// [`RenderError::DuplicateSingleton`] for a repeated singleton Kube key, or
    /// [`RenderError::InvalidDocument`] when no nonblank required `Yaml=` source is present.
    pub fn push_kube(&mut self, key: KubeKey, value: EntryValue) -> Result<(), RenderError> {
        self.push_native(
            QuadletUnitType::Kube,
            SectionKind::Kube,
            EntryKind::Kube(key),
            kube_key_name(key),
            value,
        )
    }

    /// Appends a typed experimental `[Artifact]` entry.
    ///
    /// `ContainersConfModule`, `GlobalArgs`, and `PodmanArgs` entries remain repeatable and
    /// ordered. The required `Artifact` source and every other Artifact key are singletons.
    /// `Creds` and `DecryptionKey` are redacted only from repository-owned debug output;
    /// rendering and explicit raw-value access remain exact. Values remain physical-line-safe
    /// opaque native text: this builder does not access a registry or filesystem, parse
    /// credentials, select retry defaults, or perform an artifact pull.
    ///
    /// # Errors
    ///
    /// Returns [`RenderError::WrongUnitType`] for a non-artifact document,
    /// [`RenderError::DuplicateSingleton`] for a repeated singleton key, or
    /// [`RenderError::InvalidDocument`] when the required final `Artifact=` source is absent or
    /// blank at build time.
    pub fn push_artifact(&mut self, key: ArtifactKey, value: EntryValue) -> Result<(), RenderError> {
        self.push_native(
            QuadletUnitType::Artifact,
            SectionKind::Artifact,
            EntryKind::Artifact(key),
            artifact_key_name(key),
            value,
        )
    }

    /// Appends a shared `[Quadlet]` entry to any Quadlet unit type.
    ///
    /// # Errors
    ///
    /// Returns [`RenderError::DuplicateSingleton`] when `DefaultDependencies=` is repeated.
    pub fn push_quadlet(&mut self, key: QuadletKey, value: EntryValue) -> Result<(), RenderError> {
        self.push_generated(
            SectionKind::Quadlet,
            EntryKind::Quadlet(key),
            quadlet_key_name(key),
            value,
        )
    }

    /// Appends an open-ended entry to a generic systemd section.
    ///
    /// Generic entries retain insertion order and may repeat because their reset and list behavior
    /// is directive-specific and intentionally not guessed by this builder.
    ///
    /// # Errors
    ///
    /// Returns [`RenderError::InvalidKey`] when the key is not an ASCII alphanumeric directive
    /// name.
    pub fn push_systemd(
        &mut self,
        section: SystemdSection,
        key: impl Into<String>,
        value: EntryValue,
    ) -> Result<(), RenderError> {
        let key = key.into();
        if key.is_empty() || !key.bytes().all(|byte| byte.is_ascii_alphanumeric()) {
            return Err(RenderError::InvalidKey(key));
        }
        self.entries.push(GeneratedEntry {
            section: section.kind(),
            kind: EntryKind::GenericSystemd,
            key,
            value,
        });
        Ok(())
    }

    /// Appends an evidence-backed dependency or ordering directive to `[Unit]`.
    ///
    /// These entries remain repeatable and retain insertion order. The value is an exact systemd
    /// unit-list spelling; this method does not resolve unit names or infer relationships between
    /// Quadlet source files.
    ///
    /// # Errors
    ///
    /// Returns the same errors as [`Self::push_systemd`].
    pub fn push_systemd_unit(&mut self, key: SystemdUnitKey, value: EntryValue) -> Result<(), RenderError> {
        self.push_generated(SectionKind::Unit, EntryKind::SystemdUnit(key), key.name(), value)
    }

    /// Renders, reparses, and validates the complete generated document.
    ///
    /// # Errors
    ///
    /// Returns [`RenderError::InvalidDocument`] when native shape validation fails, or
    /// [`RenderError::TypedModel`] for an internal source-span consistency failure.
    pub fn build(&self, source_id: SourceId) -> Result<GeneratedQuadletDocument, RenderError> {
        let text = self.render_text();
        let parsed = QuadletDocument::parse(self.unit_type, source_id, text).map_err(RenderError::TypedModel)?;
        if !parsed.is_valid() {
            let mut diagnostics = parsed.syntax().diagnostics().to_vec();
            diagnostics.extend_from_slice(parsed.model_diagnostics());
            return Err(RenderError::InvalidDocument(diagnostics));
        }
        Ok(GeneratedQuadletDocument { parsed })
    }

    fn push_native(
        &mut self,
        required: QuadletUnitType,
        section: SectionKind,
        kind: EntryKind,
        key: &'static str,
        value: EntryValue,
    ) -> Result<(), RenderError> {
        if self.unit_type != required {
            return Err(RenderError::WrongUnitType {
                document: self.unit_type,
                entry: required,
            });
        }
        self.push_generated(section, kind, key, value)
    }

    fn push_generated(
        &mut self,
        section: SectionKind,
        kind: EntryKind,
        key: &str,
        value: EntryValue,
    ) -> Result<(), RenderError> {
        if !kind.is_repeatable() && self.entries.iter().any(|entry| entry.kind == kind) {
            return Err(RenderError::DuplicateSingleton(key.to_owned()));
        }
        self.entries.push(GeneratedEntry {
            section,
            kind,
            key: key.to_owned(),
            value,
        });
        Ok(())
    }

    fn render_text(&self) -> String {
        let native = self.unit_type.native_section();
        let sections = [
            SectionKind::Unit,
            SectionKind::Quadlet,
            native,
            SectionKind::Service,
            SectionKind::Install,
        ];
        let mut output = String::new();
        let mut wrote_section = false;

        for section in sections {
            let entries: Vec<_> = self.entries.iter().filter(|entry| entry.section == section).collect();
            if entries.is_empty() && section != native {
                continue;
            }
            if wrote_section {
                output.push('\n');
            }
            wrote_section = true;
            output.push('[');
            output.push_str(section_name(section));
            output.push_str("]\n");
            for entry in entries {
                output.push_str(&entry.key);
                output.push('=');
                output.push_str(entry.value.as_str());
                output.push('\n');
            }
        }
        output
    }
}

/// Successfully generated and parse-back-validated Quadlet document.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeneratedQuadletDocument {
    parsed: QuadletParseResult,
}

impl GeneratedQuadletDocument {
    /// Returns the deterministic generated source text.
    #[must_use]
    pub fn text(&self) -> &str {
        self.parsed.syntax().document().render_preserved()
    }

    /// Returns the validated native typed document.
    #[must_use]
    pub const fn document(&self) -> &QuadletDocument {
        self.parsed.document()
    }

    /// Returns the complete syntax and model result.
    #[must_use]
    pub const fn parse_result(&self) -> &QuadletParseResult {
        &self.parsed
    }

    /// Decomposes the generated document into its complete parse result.
    #[must_use]
    pub fn into_parse_result(self) -> QuadletParseResult {
        self.parsed
    }
}

/// Failure while constructing or validating generated Quadlet source.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RenderError {
    /// A native value contained a line ending or NUL byte.
    InvalidValue,
    /// A generic systemd key was empty or contained non-alphanumeric bytes.
    InvalidKey(String),
    /// A native key did not belong to the builder's selected unit type.
    WrongUnitType {
        /// Unit type selected for the document.
        document: QuadletUnitType,
        /// Unit type required by the attempted entry.
        entry: QuadletUnitType,
    },
    /// A singleton native key was added more than once.
    DuplicateSingleton(String),
    /// Two mutually exclusive singleton native keys were added to one document.
    ConflictingSingletons {
        /// Existing native key.
        existing: String,
        /// Attempted native key.
        attempted: String,
    },
    /// Generated source failed syntax or native-model validation.
    InvalidDocument(Vec<Diagnostic>),
    /// Parser-owned spans could not be interpreted consistently.
    TypedModel(TypedModelError),
}

impl RenderError {
    /// Returns validation diagnostics for an invalid generated document.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        match self {
            Self::InvalidDocument(diagnostics) => diagnostics,
            _ => &[],
        }
    }
}

impl fmt::Display for RenderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidValue => formatter.write_str("generated Quadlet values must fit on one physical line"),
            Self::InvalidKey(key) => write!(formatter, "invalid generic systemd key `{key}`"),
            Self::WrongUnitType { document, entry } => {
                write!(formatter, "cannot add a {entry:?} entry to a {document:?} document")
            }
            Self::DuplicateSingleton(key) => write!(formatter, "singleton Quadlet key `{key}` is repeated"),
            Self::ConflictingSingletons { existing, attempted } => {
                write!(
                    formatter,
                    "singleton Quadlet keys `{existing}` and `{attempted}` conflict"
                )
            }
            Self::InvalidDocument(diagnostics) => {
                write!(
                    formatter,
                    "generated Quadlet document has {} diagnostic(s)",
                    diagnostics.len()
                )
            }
            Self::TypedModel(error) => write!(formatter, "generated Quadlet model is inconsistent: {error}"),
        }
    }
}

impl Error for RenderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::TypedModel(error) => Some(error),
            _ => None,
        }
    }
}

const fn container_key_name(key: ContainerKey) -> &'static str {
    match key {
        ContainerKey::AddHost => "AddHost",
        ContainerKey::Image => "Image",
        ContainerKey::Exec => "Exec",
        ContainerKey::Environment => "Environment",
        ContainerKey::EnvironmentFile => "EnvironmentFile",
        ContainerKey::Label => "Label",
        ContainerKey::Secret => "Secret",
        ContainerKey::PublishPort => "PublishPort",
        ContainerKey::Volume => "Volume",
        ContainerKey::Network => "Network",
        ContainerKey::Pod => "Pod",
        ContainerKey::HealthCmd => "HealthCmd",
        ContainerKey::Notify => "Notify",
        ContainerKey::HealthInterval => "HealthInterval",
        ContainerKey::HealthRetries => "HealthRetries",
        ContainerKey::HealthStartPeriod => "HealthStartPeriod",
        ContainerKey::HealthTimeout => "HealthTimeout",
        ContainerKey::PodmanArgs => "PodmanArgs",
        ContainerKey::User => "User",
        ContainerKey::Group => "Group",
        ContainerKey::UserNS => "UserNS",
        ContainerKey::GroupAdd => "GroupAdd",
        ContainerKey::WorkingDir => "WorkingDir",
        ContainerKey::ReadOnly => "ReadOnly",
        ContainerKey::Rootfs => "Rootfs",
        ContainerKey::ContainerName => "ContainerName",
        ContainerKey::Entrypoint => "Entrypoint",
        ContainerKey::RunInit => "RunInit",
        ContainerKey::StopSignal => "StopSignal",
        ContainerKey::StopTimeout => "StopTimeout",
        ContainerKey::Pull => "Pull",
        ContainerKey::PidsLimit => "PidsLimit",
        ContainerKey::HostName => "HostName",
        ContainerKey::ShmSize => "ShmSize",
        ContainerKey::DropCapability => "DropCapability",
        ContainerKey::AddCapability => "AddCapability",
        ContainerKey::Tmpfs => "Tmpfs",
        ContainerKey::Sysctl => "Sysctl",
        ContainerKey::Ulimit => "Ulimit",
        ContainerKey::AddDevice => "AddDevice",
        ContainerKey::Memory => "Memory",
        ContainerKey::DNS => "DNS",
        ContainerKey::DNSOption => "DNSOption",
        ContainerKey::DNSSearch => "DNSSearch",
        ContainerKey::ExposeHostPort => "ExposeHostPort",
        ContainerKey::Annotation => "Annotation",
        ContainerKey::AppArmor => "AppArmor",
        ContainerKey::NoNewPrivileges => "NoNewPrivileges",
        ContainerKey::SeccompProfile => "SeccompProfile",
        ContainerKey::SecurityLabelDisable => "SecurityLabelDisable",
        ContainerKey::SecurityLabelFileType => "SecurityLabelFileType",
        ContainerKey::SecurityLabelLevel => "SecurityLabelLevel",
        ContainerKey::SecurityLabelNested => "SecurityLabelNested",
        ContainerKey::SecurityLabelType => "SecurityLabelType",
        ContainerKey::Mask => "Mask",
        ContainerKey::Unmask => "Unmask",
        ContainerKey::LogDriver => "LogDriver",
        ContainerKey::LogOpt => "LogOpt",
        ContainerKey::IP => "IP",
        ContainerKey::IP6 => "IP6",
        ContainerKey::NetworkAlias => "NetworkAlias",
        ContainerKey::ReloadCmd => "ReloadCmd",
        ContainerKey::ReloadSignal => "ReloadSignal",
        ContainerKey::AutoUpdate => "AutoUpdate",
        ContainerKey::CgroupsMode => "CgroupsMode",
        ContainerKey::EnvironmentHost => "EnvironmentHost",
        ContainerKey::GIDMap => "GIDMap",
        ContainerKey::HttpProxy => "HttpProxy",
        ContainerKey::Mount => "Mount",
        ContainerKey::ReadOnlyTmpfs => "ReadOnlyTmpfs",
        ContainerKey::Retry => "Retry",
        ContainerKey::RetryDelay => "RetryDelay",
        ContainerKey::StartWithPod => "StartWithPod",
        ContainerKey::SubGIDMap => "SubGIDMap",
        ContainerKey::SubUIDMap => "SubUIDMap",
        ContainerKey::Timezone => "Timezone",
        ContainerKey::UIDMap => "UIDMap",
        ContainerKey::HealthOnFailure => "HealthOnFailure",
        ContainerKey::ContainersConfModule => "ContainersConfModule",
        ContainerKey::GlobalArgs => "GlobalArgs",
        ContainerKey::HealthLogDestination => "HealthLogDestination",
        ContainerKey::HealthMaxLogCount => "HealthMaxLogCount",
        ContainerKey::HealthMaxLogSize => "HealthMaxLogSize",
        ContainerKey::HealthStartupCmd => "HealthStartupCmd",
        ContainerKey::HealthStartupInterval => "HealthStartupInterval",
        ContainerKey::HealthStartupRetries => "HealthStartupRetries",
        ContainerKey::HealthStartupSuccess => "HealthStartupSuccess",
        ContainerKey::HealthStartupTimeout => "HealthStartupTimeout",
        ContainerKey::ImageVolume => "ImageVolume",
        ContainerKey::ServiceName => "ServiceName",
    }
}

const fn build_key_name(key: BuildKey) -> &'static str {
    match key {
        BuildKey::ImageTag => "ImageTag",
        BuildKey::SetWorkingDirectory => "SetWorkingDirectory",
        BuildKey::File => "File",
        BuildKey::Target => "Target",
        BuildKey::Network => "Network",
        BuildKey::Label => "Label",
        BuildKey::BuildArg => "BuildArg",
        BuildKey::Secret => "Secret",
        BuildKey::Arch => "Arch",
        BuildKey::Variant => "Variant",
        BuildKey::Pull => "Pull",
        BuildKey::PodmanArgs => "PodmanArgs",
        BuildKey::Retry => "Retry",
        BuildKey::RetryDelay => "RetryDelay",
        BuildKey::TLSVerify => "TLSVerify",
        BuildKey::ForceRM => "ForceRM",
        BuildKey::GroupAdd => "GroupAdd",
        BuildKey::DNS => "DNS",
        BuildKey::DNSOption => "DNSOption",
        BuildKey::DNSSearch => "DNSSearch",
        BuildKey::AuthFile => "AuthFile",
        BuildKey::IgnoreFile => "IgnoreFile",
        BuildKey::Annotation => "Annotation",
        BuildKey::Environment => "Environment",
        BuildKey::ContainersConfModule => "ContainersConfModule",
        BuildKey::GlobalArgs => "GlobalArgs",
        BuildKey::ServiceName => "ServiceName",
        BuildKey::Volume => "Volume",
    }
}

const fn image_key_name(key: ImageKey) -> &'static str {
    match key {
        ImageKey::Image => "Image",
        ImageKey::ImageTag => "ImageTag",
        ImageKey::ServiceName => "ServiceName",
        ImageKey::AllTags => "AllTags",
        ImageKey::Arch => "Arch",
        ImageKey::AuthFile => "AuthFile",
        ImageKey::CertDir => "CertDir",
        ImageKey::ContainersConfModule => "ContainersConfModule",
        ImageKey::Creds => "Creds",
        ImageKey::DecryptionKey => "DecryptionKey",
        ImageKey::GlobalArgs => "GlobalArgs",
        ImageKey::OS => "OS",
        ImageKey::PodmanArgs => "PodmanArgs",
        ImageKey::Policy => "Policy",
        ImageKey::Retry => "Retry",
        ImageKey::RetryDelay => "RetryDelay",
        ImageKey::TLSVerify => "TLSVerify",
        ImageKey::Variant => "Variant",
    }
}

const fn kube_key_name(key: KubeKey) -> &'static str {
    match key {
        KubeKey::AutoUpdate => "AutoUpdate",
        KubeKey::ConfigMap => "ConfigMap",
        KubeKey::ContainersConfModule => "ContainersConfModule",
        KubeKey::ExitCodePropagation => "ExitCodePropagation",
        KubeKey::GlobalArgs => "GlobalArgs",
        KubeKey::KubeDownForce => "KubeDownForce",
        KubeKey::LogDriver => "LogDriver",
        KubeKey::Network => "Network",
        KubeKey::PodmanArgs => "PodmanArgs",
        KubeKey::PublishPort => "PublishPort",
        KubeKey::ServiceName => "ServiceName",
        KubeKey::SetWorkingDirectory => "SetWorkingDirectory",
        KubeKey::UserNS => "UserNS",
        KubeKey::Yaml => "Yaml",
        KubeKey::LogOpt => "LogOpt",
        KubeKey::RemapGid => "RemapGid",
        KubeKey::RemapUid => "RemapUid",
        KubeKey::RemapUidSize => "RemapUidSize",
        KubeKey::RemapUsers => "RemapUsers",
    }
}

const fn artifact_key_name(key: ArtifactKey) -> &'static str {
    match key {
        ArtifactKey::Artifact => "Artifact",
        ArtifactKey::AuthFile => "AuthFile",
        ArtifactKey::CertDir => "CertDir",
        ArtifactKey::Creds => "Creds",
        ArtifactKey::DecryptionKey => "DecryptionKey",
        ArtifactKey::Quiet => "Quiet",
        ArtifactKey::Retry => "Retry",
        ArtifactKey::RetryDelay => "RetryDelay",
        ArtifactKey::ServiceName => "ServiceName",
        ArtifactKey::TLSVerify => "TLSVerify",
        ArtifactKey::ContainersConfModule => "ContainersConfModule",
        ArtifactKey::GlobalArgs => "GlobalArgs",
        ArtifactKey::PodmanArgs => "PodmanArgs",
    }
}

const fn quadlet_key_name(key: QuadletKey) -> &'static str {
    match key {
        QuadletKey::DefaultDependencies => "DefaultDependencies",
    }
}

const fn pod_key_name(key: PodKey) -> &'static str {
    match key {
        PodKey::AddHost => "AddHost",
        PodKey::PodName => "PodName",
        PodKey::PublishPort => "PublishPort",
        PodKey::Network => "Network",
        PodKey::Volume => "Volume",
        PodKey::UserNS => "UserNS",
        PodKey::ShmSize => "ShmSize",
        PodKey::ExitPolicy => "ExitPolicy",
        PodKey::StopTimeout => "StopTimeout",
        PodKey::ServiceName => "ServiceName",
        PodKey::ContainersConfModule => "ContainersConfModule",
        PodKey::DNS => "DNS",
        PodKey::DNSOption => "DNSOption",
        PodKey::DNSSearch => "DNSSearch",
        PodKey::GIDMap => "GIDMap",
        PodKey::GlobalArgs => "GlobalArgs",
        PodKey::HostName => "HostName",
        PodKey::IP => "IP",
        PodKey::IP6 => "IP6",
        PodKey::Label => "Label",
        PodKey::NetworkAlias => "NetworkAlias",
        PodKey::PodmanArgs => "PodmanArgs",
        PodKey::SubGIDMap => "SubGIDMap",
        PodKey::SubUIDMap => "SubUIDMap",
        PodKey::UIDMap => "UIDMap",
    }
}

const fn network_key_name(key: NetworkKey) -> &'static str {
    match key {
        NetworkKey::NetworkName => "NetworkName",
        NetworkKey::Driver => "Driver",
        NetworkKey::Options => "Options",
        NetworkKey::Internal => "Internal",
        NetworkKey::IPv6 => "IPv6",
        NetworkKey::IPAMDriver => "IPAMDriver",
        NetworkKey::Subnet => "Subnet",
        NetworkKey::Gateway => "Gateway",
        NetworkKey::IPRange => "IPRange",
        NetworkKey::Label => "Label",
        NetworkKey::ContainersConfModule => "ContainersConfModule",
        NetworkKey::DisableDNS => "DisableDNS",
        NetworkKey::DNS => "DNS",
        NetworkKey::GlobalArgs => "GlobalArgs",
        NetworkKey::InterfaceName => "InterfaceName",
        NetworkKey::NetworkDeleteOnStop => "NetworkDeleteOnStop",
        NetworkKey::PodmanArgs => "PodmanArgs",
        NetworkKey::ServiceName => "ServiceName",
    }
}

const fn volume_key_name(key: VolumeKey) -> &'static str {
    match key {
        VolumeKey::VolumeName => "VolumeName",
        VolumeKey::Driver => "Driver",
        VolumeKey::Options => "Options",
        VolumeKey::Label => "Label",
        VolumeKey::Device => "Device",
        VolumeKey::Type => "Type",
        VolumeKey::Copy => "Copy",
        VolumeKey::ContainersConfModule => "ContainersConfModule",
        VolumeKey::GlobalArgs => "GlobalArgs",
        VolumeKey::PodmanArgs => "PodmanArgs",
        VolumeKey::User => "User",
        VolumeKey::Group => "Group",
        VolumeKey::UID => "UID",
        VolumeKey::GID => "GID",
        VolumeKey::ServiceName => "ServiceName",
        VolumeKey::Image => "Image",
    }
}

const fn section_name(section: SectionKind) -> &'static str {
    match section {
        SectionKind::Unit => "Unit",
        SectionKind::Container => "Container",
        SectionKind::Pod => "Pod",
        SectionKind::Network => "Network",
        SectionKind::Volume => "Volume",
        SectionKind::Build => "Build",
        SectionKind::Image => "Image",
        SectionKind::Kube => "Kube",
        SectionKind::Artifact => "Artifact",
        SectionKind::Quadlet => "Quadlet",
        SectionKind::Service => "Service",
        SectionKind::Install => "Install",
        SectionKind::Unknown => "Unknown",
    }
}
