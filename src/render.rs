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
        ContainerKey, EntryKind, NetworkKey, PodKey, QuadletDocument, QuadletParseResult, QuadletUnitType, SectionKind,
        TypedModelError, VolumeKey,
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

/// Evidence-backed dependency and ordering directives in a generic systemd `[Unit]` section.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SystemdUnitKey {
    /// Strong requirement that also pulls the referenced unit into the transaction.
    Requires,
    /// Weak requirement that does not fail this unit when the referenced unit fails.
    Wants,
    /// Orders this unit after the referenced unit without pulling it into the transaction.
    After,
}

impl SystemdUnitKey {
    const fn name(self) -> &'static str {
        match self {
            Self::Requires => "Requires",
            Self::Wants => "Wants",
            Self::After => "After",
        }
    }
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

#[derive(Clone, Debug, Eq, PartialEq)]
struct GeneratedEntry {
    section: SectionKind,
    kind: EntryKind,
    key: String,
    value: EntryValue,
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
        self.push_systemd(SystemdSection::Unit, key.name(), value)
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
        let sections = [SectionKind::Unit, native, SectionKind::Service, SectionKind::Install];
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
    }
}

const fn network_key_name(key: NetworkKey) -> &'static str {
    match key {
        NetworkKey::NetworkName => "NetworkName",
    }
}

const fn volume_key_name(key: VolumeKey) -> &'static str {
    match key {
        VolumeKey::VolumeName => "VolumeName",
    }
}

const fn section_name(section: SectionKind) -> &'static str {
    match section {
        SectionKind::Unit => "Unit",
        SectionKind::Container => "Container",
        SectionKind::Pod => "Pod",
        SectionKind::Network => "Network",
        SectionKind::Volume => "Volume",
        SectionKind::Service => "Service",
        SectionKind::Install => "Install",
        SectionKind::Unknown => "Unknown",
    }
}
