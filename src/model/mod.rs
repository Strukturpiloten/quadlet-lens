//! Source-aware native Quadlet documents and the first-conversion value surface.

use std::{collections::BTreeSet, error::Error, fmt};

mod document_set;

pub use document_set::{
    DependencyEdge, DependencyGraph, DocumentSetError, NamedQuadletDocument, QuadletDocumentSet, ReferenceResolution,
    UnitFileName, UnitReference,
};

use crate::diagnostic::{Diagnostic, DiagnosticCode, Label, Severity};
use crate::path::{PathForm, classify_path};
use crate::source::{SourceId, SourceSpan, SourceText};
use crate::syntax::{ParseResult, SyntaxDocument, SyntaxLineKind};

const MISSING_SECTION: DiagnosticCode = DiagnosticCode::new("QLM0001");
const MISSING_IMAGE: DiagnosticCode = DiagnosticCode::new("QLM0002");
const FOREIGN_NATIVE_SECTION: DiagnosticCode = DiagnosticCode::new("QLM0003");
const REPEATED_SINGLETON: DiagnosticCode = DiagnosticCode::new("QLM0004");
const EMPTY_IMAGE: DiagnosticCode = DiagnosticCode::new("QLM0005");
const CONFLICTING_IMAGE_ROOTFS: DiagnosticCode = DiagnosticCode::new("QLM0006");
const EMPTY_ROOTFS: DiagnosticCode = DiagnosticCode::new("QLM0007");

/// Native Quadlet unit types supported by the first conversion milestone.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum QuadletUnitType {
    /// A `.container` unit.
    Container,
    /// A `.pod` unit.
    Pod,
    /// A `.network` unit.
    Network,
    /// A `.volume` unit.
    Volume,
}

impl QuadletUnitType {
    /// Infers a supported unit type from a lowercase file extension.
    #[must_use]
    pub fn from_extension(extension: &str) -> Option<Self> {
        match extension {
            "container" => Some(Self::Container),
            "pod" => Some(Self::Pod),
            "network" => Some(Self::Network),
            "volume" => Some(Self::Volume),
            _ => None,
        }
    }

    /// Returns the native section required by this unit type.
    #[must_use]
    pub const fn native_section(self) -> SectionKind {
        match self {
            Self::Container => SectionKind::Container,
            Self::Pod => SectionKind::Pod,
            Self::Network => SectionKind::Network,
            Self::Volume => SectionKind::Volume,
        }
    }
}

/// A section's typed role without discarding its authored name.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum SectionKind {
    /// Generic systemd `[Unit]` section.
    Unit,
    /// Generic systemd `[Service]` section.
    Service,
    /// Generic systemd `[Install]` section.
    Install,
    /// Native Quadlet `[Container]` section.
    Container,
    /// Native Quadlet `[Pod]` section.
    Pod,
    /// Native Quadlet `[Network]` section.
    Network,
    /// Native Quadlet `[Volume]` section.
    Volume,
    /// Any other section, retained without interpretation.
    Unknown,
}

impl SectionKind {
    fn classify(name: &str) -> Self {
        match name {
            "Unit" => Self::Unit,
            "Service" => Self::Service,
            "Install" => Self::Install,
            "Container" => Self::Container,
            "Pod" => Self::Pod,
            "Network" => Self::Network,
            "Volume" => Self::Volume,
            _ => Self::Unknown,
        }
    }

    const fn is_native(self) -> bool {
        matches!(self, Self::Container | Self::Pod | Self::Network | Self::Volume)
    }
}

/// Container keys required by the first Compose-to-Quadlet conversion.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ContainerKey {
    /// Hostname-to-address mapping added to the container hosts file.
    AddHost,
    /// Container image or `.image`/`.build` reference.
    Image,
    /// Command arguments following the image.
    Exec,
    /// Environment assignments.
    Environment,
    /// Environment-file path.
    EnvironmentFile,
    /// Published port.
    PublishPort,
    /// Bind, named, anonymous, or `.volume` mount.
    Volume,
    /// Podman network argument or `.network` reference.
    Network,
    /// `.pod` reference.
    Pod,
    /// Container health command.
    HealthCmd,
    /// Ordered Podman argument escape hatch.
    PodmanArgs,
    /// Interval between regular health checks.
    HealthInterval,
    /// Failed checks required before the container becomes unhealthy.
    HealthRetries,
    /// Startup grace period before failures count.
    HealthStartPeriod,
    /// Maximum duration of one regular health check.
    HealthTimeout,
    /// Container startup notification mode, including health-gated readiness.
    Notify,
    /// Primary user inside the container.
    User,
    /// Primary group inside the container.
    Group,
    /// User-namespace mode passed to Podman.
    UserNS,
    /// Supplementary group assigned to the container process.
    GroupAdd,
    /// Working directory inside the container.
    WorkingDir,
    /// Whether the container root filesystem is read-only.
    ReadOnly,
    /// Podman secret reference and optional mount or environment exposure options.
    Secret,
    /// OCI label assignment attached to the container.
    Label,
    /// Host root filesystem used instead of a container image.
    Rootfs,
    /// Runtime name assigned to the generated Podman container.
    ContainerName,
    /// Entrypoint override passed to Podman, including JSON command-array syntax.
    Entrypoint,
    /// Authored selection value for Podman's minimal init process.
    RunInit,
    /// Authored signal value Podman uses when stopping the container.
    StopSignal,
    /// Authored stop-timeout value in seconds, including a native zero.
    StopTimeout,
    /// Authored image pull-policy value.
    Pull,
    /// Authored container process-ID limit.
    PidsLimit,
    /// Authored hostname available inside the container.
    HostName,
    /// Authored size of the container shared-memory filesystem.
    ShmSize,
    /// Authored capabilities removed from the container's default capability set.
    DropCapability,
    /// Authored capabilities added to the container's default capability set.
    AddCapability,
    /// Authored temporary-filesystem destination and optional mount options.
    Tmpfs,
    /// Authored kernel parameter assignments passed to the container.
    Sysctl,
    /// Authored resource-limit assignments passed to the container.
    Ulimit,
    /// Authored host-device mappings passed to the container.
    AddDevice,
    /// Authored memory limit passed to the container.
    Memory,
}

/// Pod keys required by the first Compose-to-Quadlet conversion.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum PodKey {
    /// Hostname-to-address mapping shared by the pod.
    AddHost,
    /// Runtime name assigned to the generated Podman pod.
    PodName,
    /// Published port owned by the pod.
    PublishPort,
    /// Podman network argument or `.network` reference.
    Network,
    /// Bind, named, anonymous, or `.volume` mount.
    Volume,
    /// User-namespace mode shared by containers in the pod.
    UserNS,
    /// Authored size of the pod shared-memory filesystem.
    ShmSize,
}

/// Network keys required by the first conversion.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum NetworkKey {
    /// Runtime name assigned to the generated Podman network.
    NetworkName,
}

/// Volume keys required by the first conversion.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum VolumeKey {
    /// Runtime name assigned to the generated Podman volume.
    VolumeName,
}

/// Typed role of an authored entry.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum EntryKind {
    /// Entry in a generic systemd section. The key remains open-ended by design.
    GenericSystemd,
    /// Recognized key in `[Container]`.
    Container(ContainerKey),
    /// Recognized key in `[Pod]`.
    Pod(PodKey),
    /// Recognized key in `[Network]`.
    Network(NetworkKey),
    /// Recognized key in `[Volume]`.
    Volume(VolumeKey),
    /// Unknown entry retained in its original section and position.
    Unknown,
}

impl EntryKind {
    /// Returns whether repeated entries are part of the documented first-conversion form.
    #[must_use]
    pub const fn is_repeatable(self) -> bool {
        matches!(
            self,
            Self::GenericSystemd
                | Self::Container(
                    ContainerKey::AddHost
                        | ContainerKey::Environment
                        | ContainerKey::EnvironmentFile
                        | ContainerKey::Label
                        | ContainerKey::Secret
                        | ContainerKey::PublishPort
                        | ContainerKey::Volume
                        | ContainerKey::Network
                        | ContainerKey::PodmanArgs
                        | ContainerKey::GroupAdd
                        | ContainerKey::DropCapability
                        | ContainerKey::AddCapability
                        | ContainerKey::Tmpfs
                        | ContainerKey::Sysctl
                        | ContainerKey::Ulimit
                        | ContainerKey::AddDevice
                )
                | Self::Pod(PodKey::AddHost | PodKey::PublishPort | PodKey::Network | PodKey::Volume)
                | Self::Unknown
        )
    }
}

/// Native Quadlet unit referenced by an authored value.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum UnitReferenceKind {
    /// `.image` unit.
    Image,
    /// `.build` unit.
    Build,
    /// `.pod` unit.
    Pod,
    /// `.network` unit.
    Network,
    /// `.volume` unit.
    Volume,
}

/// Conservative lexical classification of a typed value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ValueKind {
    /// Value semantics intentionally remain opaque, for example a systemd command line.
    Opaque,
    /// A path value classified without expanding systemd specifiers.
    Path(PathForm),
    /// A native cross-file reference.
    UnitReference(UnitReferenceKind),
}

/// Owned authored text paired with its precise source span.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourcedText {
    text: String,
    span: SourceSpan,
}

impl SourcedText {
    fn from_span(source: &SourceText, span: SourceSpan) -> Result<Self, TypedModelError> {
        let text = source
            .slice(span)
            .ok_or(TypedModelError::InvalidSourceSpan(span))?
            .to_owned();
        Ok(Self { text, span })
    }

    /// Returns the exact authored text selected by the span.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the text's source span.
    #[must_use]
    pub const fn span(&self) -> SourceSpan {
        self.span
    }
}

/// One entry value with its physical continuation segments kept separate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthoredValue {
    primary: SourcedText,
    continuations: Vec<SourcedText>,
    has_continuation_marker: bool,
}

impl AuthoredValue {
    /// Returns the value on the entry's first physical line.
    #[must_use]
    pub const fn primary(&self) -> &SourcedText {
        &self.primary
    }

    /// Returns subsequent physical value segments in authored order.
    #[must_use]
    pub fn continuations(&self) -> &[SourcedText] {
        &self.continuations
    }

    /// Returns whether the value uses physical-line continuation syntax.
    #[must_use]
    pub fn is_continued(&self) -> bool {
        self.has_continuation_marker
    }
}

/// One typed entry retained in authored section and entry order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypedEntry {
    key: SourcedText,
    value: AuthoredValue,
    kind: EntryKind,
    value_kind: ValueKind,
    source_line: usize,
}

impl TypedEntry {
    /// Returns the exact authored key.
    #[must_use]
    pub const fn key(&self) -> &SourcedText {
        &self.key
    }

    /// Returns the loss-aware authored value.
    #[must_use]
    pub const fn value(&self) -> &AuthoredValue {
        &self.value
    }

    /// Returns the recognized native role, or [`EntryKind::Unknown`].
    #[must_use]
    pub const fn kind(&self) -> EntryKind {
        self.kind
    }

    /// Returns the conservative path or reference classification.
    #[must_use]
    pub const fn value_kind(&self) -> ValueKind {
        self.value_kind
    }

    /// Returns the zero-based physical source-line index.
    #[must_use]
    pub const fn source_line(&self) -> usize {
        self.source_line
    }

    /// Returns the exact referenced unit-file name when this value is a native reference.
    #[must_use]
    pub fn unit_reference_name(&self) -> Option<&str> {
        let ValueKind::UnitReference(_) = self.value_kind else {
            return None;
        };
        let value = self.value.primary.text.trim();
        match self.kind {
            EntryKind::Container(ContainerKey::Volume) | EntryKind::Pod(PodKey::Volume) => {
                Some(value.split_once(':').map_or(value, |(source, _)| source).trim())
            }
            EntryKind::Container(ContainerKey::Network | ContainerKey::Pod) | EntryKind::Pod(PodKey::Network) => {
                Some(first_token(value))
            }
            EntryKind::Container(ContainerKey::Image) => Some(value),
            _ => None,
        }
    }
}

/// One section occurrence. Repeated sections remain independent and ordered.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypedSection {
    name: SourcedText,
    kind: SectionKind,
    entries: Vec<TypedEntry>,
    source_line: usize,
}

impl TypedSection {
    /// Returns the exact authored section name.
    #[must_use]
    pub const fn name(&self) -> &SourcedText {
        &self.name
    }

    /// Returns the section's recognized role.
    #[must_use]
    pub const fn kind(&self) -> SectionKind {
        self.kind
    }

    /// Returns entries in authored order, including repeated and unknown keys.
    #[must_use]
    pub fn entries(&self) -> &[TypedEntry] {
        &self.entries
    }

    /// Returns the zero-based physical source-line index.
    #[must_use]
    pub const fn source_line(&self) -> usize {
        self.source_line
    }
}

/// Source-aware typed view of one supported Quadlet unit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuadletDocument {
    source_id: SourceId,
    source_span: SourceSpan,
    unit_type: QuadletUnitType,
    sections: Vec<TypedSection>,
}

impl QuadletDocument {
    /// Interprets a syntax document without modifying or normalizing its values.
    ///
    /// # Errors
    ///
    /// Returns [`TypedModelError::InvalidSourceSpan`] only if a parser-owned span
    /// cannot be resolved against its source document.
    pub fn interpret(
        unit_type: QuadletUnitType,
        syntax: &SyntaxDocument,
    ) -> Result<(Self, Vec<Diagnostic>), TypedModelError> {
        let mut sections: Vec<TypedSection> = Vec::new();
        let mut current_section = None;

        for (line_index, line) in syntax.lines().iter().enumerate() {
            match line.kind() {
                SyntaxLineKind::Section(section) => {
                    let name = SourcedText::from_span(syntax.source(), section.name())?;
                    let kind = SectionKind::classify(name.text());
                    sections.push(TypedSection {
                        name,
                        kind,
                        entries: Vec::new(),
                        source_line: line_index,
                    });
                    current_section = Some(sections.len() - 1);
                }
                SyntaxLineKind::Entry(entry) => {
                    let Some(section_index) = current_section else {
                        continue;
                    };
                    let key = SourcedText::from_span(syntax.source(), entry.key())?;
                    let primary = SourcedText::from_span(syntax.source(), entry.value())?;
                    let continuations = collect_continuations(syntax, line_index)?;
                    let section_kind = sections[section_index].kind;
                    let kind = classify_entry(section_kind, key.text());
                    let value_kind = classify_value(kind, primary.text());
                    sections[section_index].entries.push(TypedEntry {
                        key,
                        value: AuthoredValue {
                            primary,
                            continuations,
                            has_continuation_marker: entry.continues(),
                        },
                        kind,
                        value_kind,
                        source_line: line_index,
                    });
                }
                SyntaxLineKind::Blank
                | SyntaxLineKind::Comment(_)
                | SyntaxLineKind::Continuation(_)
                | SyntaxLineKind::Invalid => {}
            }
        }

        let document = Self {
            source_id: syntax.source().id(),
            source_span: SourceSpan::new(syntax.source().id(), 0, syntax.source().text().len()),
            unit_type,
            sections,
        };
        let diagnostics = document.validate_shape(syntax.source());
        Ok((document, diagnostics))
    }

    /// Parses source syntax and constructs its typed view in one operation.
    ///
    /// # Errors
    ///
    /// Returns [`TypedModelError::InvalidSourceSpan`] only if a parser-owned span
    /// cannot be resolved against its source document.
    pub fn parse(
        unit_type: QuadletUnitType,
        source_id: SourceId,
        text: impl Into<String>,
    ) -> Result<QuadletParseResult, TypedModelError> {
        let syntax = SyntaxDocument::parse(source_id, text);
        let (document, model_diagnostics) = Self::interpret(unit_type, syntax.document())?;
        Ok(QuadletParseResult {
            syntax,
            document,
            model_diagnostics,
        })
    }

    /// Returns the caller-selected source identity.
    #[must_use]
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    /// Returns the span covering the complete source document.
    #[must_use]
    pub const fn source_span(&self) -> SourceSpan {
        self.source_span
    }

    /// Returns the native unit type selected by the caller.
    #[must_use]
    pub const fn unit_type(&self) -> QuadletUnitType {
        self.unit_type
    }

    /// Returns sections and repeated section occurrences in authored order.
    #[must_use]
    pub fn sections(&self) -> &[TypedSection] {
        &self.sections
    }

    /// Iterates all typed entries in authored order.
    pub fn entries(&self) -> impl Iterator<Item = &TypedEntry> {
        self.sections.iter().flat_map(|section| section.entries.iter())
    }

    fn validate_shape(&self, source: &SourceText) -> Vec<Diagnostic> {
        let expected = self.unit_type.native_section();
        let mut diagnostics = Vec::new();
        let mut expected_sections = self.sections.iter().filter(|section| section.kind == expected);
        let first_expected = expected_sections.next();

        if first_expected.is_none() {
            diagnostics.push(Diagnostic::new(
                MISSING_SECTION,
                Severity::Error,
                "Quadlet unit is missing its required native section",
                Label::new(
                    SourceSpan::new(source.id(), 0, source.text().len()),
                    "add the native section required by the selected unit type",
                ),
            ));
        }

        for section in &self.sections {
            if section.kind.is_native() && section.kind != expected {
                diagnostics.push(Diagnostic::new(
                    FOREIGN_NATIVE_SECTION,
                    Severity::Warning,
                    "Quadlet unit contains a native section for another unit type",
                    Label::new(
                        section.name.span(),
                        "this section does not match the selected file type",
                    ),
                ));
            }
        }

        let mut singletons = BTreeSet::new();
        for entry in self.entries() {
            if !entry.kind.is_repeatable() && !singletons.insert(entry.kind) {
                diagnostics.push(Diagnostic::new(
                    REPEATED_SINGLETON,
                    Severity::Warning,
                    "single-value Quadlet key is repeated",
                    Label::new(entry.key.span(), "the later value may replace an earlier value"),
                ));
            }
        }

        if self.unit_type == QuadletUnitType::Container {
            diagnostics.extend(self.validate_container_source(first_expected));
        }

        diagnostics
    }

    fn validate_container_source(&self, container_section: Option<&TypedSection>) -> Vec<Diagnostic> {
        let container_entries: Vec<_> = self
            .sections
            .iter()
            .filter(|section| section.kind == SectionKind::Container)
            .flat_map(|section| section.entries.iter())
            .collect();
        let images: Vec<_> = container_entries
            .iter()
            .copied()
            .filter(|entry| entry.kind == EntryKind::Container(ContainerKey::Image))
            .collect();
        let root_filesystems: Vec<_> = container_entries
            .iter()
            .copied()
            .filter(|entry| entry.kind == EntryKind::Container(ContainerKey::Rootfs))
            .collect();
        let mut diagnostics = Vec::new();

        if images.is_empty() && root_filesystems.is_empty() {
            if let Some(section) = container_section {
                diagnostics.push(Diagnostic::new(
                    MISSING_IMAGE,
                    Severity::Error,
                    "container unit is missing its required image or root filesystem",
                    Label::new(
                        section.name.span(),
                        "add either `Image=` or `Rootfs=` to this Container section",
                    ),
                ));
            }
        }
        if !images.is_empty() && !root_filesystems.is_empty() {
            diagnostics.push(Diagnostic::new(
                CONFLICTING_IMAGE_ROOTFS,
                Severity::Error,
                "container Image and Rootfs entries conflict",
                Label::new(
                    root_filesystems[0].value.primary.span(),
                    "remove either this Rootfs entry or every Image entry",
                ),
            ));
        }
        diagnostics.extend(
            images
                .iter()
                .filter(|entry| entry.value.primary.text.trim().is_empty())
                .map(|entry| {
                    Diagnostic::new(
                        EMPTY_IMAGE,
                        Severity::Error,
                        "container Image entry is empty",
                        Label::new(entry.value.primary.span(), "provide an image or unit reference"),
                    )
                }),
        );
        diagnostics.extend(
            root_filesystems
                .iter()
                .filter(|entry| entry.value.primary.text.trim().is_empty())
                .map(|entry| {
                    Diagnostic::new(
                        EMPTY_ROOTFS,
                        Severity::Error,
                        "container Rootfs entry is empty",
                        Label::new(entry.value.primary.span(), "provide a Podman root filesystem"),
                    )
                }),
        );
        diagnostics
    }
}

/// Combined syntax and typed-model result for one source file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuadletParseResult {
    syntax: ParseResult,
    document: QuadletDocument,
    model_diagnostics: Vec<Diagnostic>,
}

impl QuadletParseResult {
    /// Returns the complete loss-aware syntax parse result.
    #[must_use]
    pub const fn syntax(&self) -> &ParseResult {
        &self.syntax
    }

    /// Returns the typed document even when recoverable diagnostics exist.
    #[must_use]
    pub const fn document(&self) -> &QuadletDocument {
        &self.document
    }

    /// Returns diagnostics produced by native typed-model validation.
    #[must_use]
    pub fn model_diagnostics(&self) -> &[Diagnostic] {
        &self.model_diagnostics
    }

    /// Returns whether both syntax and typed-model validation have no errors.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.syntax.is_valid()
            && self
                .model_diagnostics
                .iter()
                .all(|diagnostic| diagnostic.severity() != Severity::Error)
    }

    /// Decomposes the result without dropping either diagnostic layer.
    #[must_use]
    pub fn into_parts(self) -> (ParseResult, QuadletDocument, Vec<Diagnostic>) {
        (self.syntax, self.document, self.model_diagnostics)
    }
}

/// Internal consistency failure while interpreting parser-owned spans.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TypedModelError {
    /// A span could not be resolved against the syntax document that owns it.
    InvalidSourceSpan(SourceSpan),
}

impl fmt::Display for TypedModelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSourceSpan(span) => write!(formatter, "invalid parser source span: {span:?}"),
        }
    }
}

impl Error for TypedModelError {}

fn collect_continuations(syntax: &SyntaxDocument, entry_line: usize) -> Result<Vec<SourcedText>, TypedModelError> {
    let mut values = Vec::new();
    for line in syntax.lines().iter().skip(entry_line + 1) {
        match line.kind() {
            SyntaxLineKind::Continuation(continuation) => {
                values.push(SourcedText::from_span(syntax.source(), continuation.value())?);
            }
            SyntaxLineKind::Comment(comment) if comment.within_continuation() => {}
            _ => break,
        }
    }
    Ok(values)
}

fn classify_entry(section: SectionKind, key: &str) -> EntryKind {
    match section {
        SectionKind::Unit | SectionKind::Service | SectionKind::Install => EntryKind::GenericSystemd,
        SectionKind::Container => match key {
            "AddHost" => EntryKind::Container(ContainerKey::AddHost),
            "Image" => EntryKind::Container(ContainerKey::Image),
            "Exec" => EntryKind::Container(ContainerKey::Exec),
            "Environment" => EntryKind::Container(ContainerKey::Environment),
            "EnvironmentFile" => EntryKind::Container(ContainerKey::EnvironmentFile),
            "Label" => EntryKind::Container(ContainerKey::Label),
            "Secret" => EntryKind::Container(ContainerKey::Secret),
            "PublishPort" => EntryKind::Container(ContainerKey::PublishPort),
            "Volume" => EntryKind::Container(ContainerKey::Volume),
            "Network" => EntryKind::Container(ContainerKey::Network),
            "Pod" => EntryKind::Container(ContainerKey::Pod),
            "HealthCmd" => EntryKind::Container(ContainerKey::HealthCmd),
            "Notify" => EntryKind::Container(ContainerKey::Notify),
            "HealthInterval" => EntryKind::Container(ContainerKey::HealthInterval),
            "HealthRetries" => EntryKind::Container(ContainerKey::HealthRetries),
            "HealthStartPeriod" => EntryKind::Container(ContainerKey::HealthStartPeriod),
            "HealthTimeout" => EntryKind::Container(ContainerKey::HealthTimeout),
            "PodmanArgs" => EntryKind::Container(ContainerKey::PodmanArgs),
            "User" => EntryKind::Container(ContainerKey::User),
            "Group" => EntryKind::Container(ContainerKey::Group),
            "UserNS" => EntryKind::Container(ContainerKey::UserNS),
            "GroupAdd" => EntryKind::Container(ContainerKey::GroupAdd),
            "WorkingDir" => EntryKind::Container(ContainerKey::WorkingDir),
            "ReadOnly" => EntryKind::Container(ContainerKey::ReadOnly),
            "Rootfs" => EntryKind::Container(ContainerKey::Rootfs),
            "ContainerName" => EntryKind::Container(ContainerKey::ContainerName),
            "Entrypoint" => EntryKind::Container(ContainerKey::Entrypoint),
            "RunInit" => EntryKind::Container(ContainerKey::RunInit),
            "StopSignal" => EntryKind::Container(ContainerKey::StopSignal),
            "StopTimeout" => EntryKind::Container(ContainerKey::StopTimeout),
            "Pull" => EntryKind::Container(ContainerKey::Pull),
            "PidsLimit" => EntryKind::Container(ContainerKey::PidsLimit),
            "HostName" => EntryKind::Container(ContainerKey::HostName),
            "ShmSize" => EntryKind::Container(ContainerKey::ShmSize),
            "DropCapability" => EntryKind::Container(ContainerKey::DropCapability),
            "AddCapability" => EntryKind::Container(ContainerKey::AddCapability),
            "Tmpfs" => EntryKind::Container(ContainerKey::Tmpfs),
            "Sysctl" => EntryKind::Container(ContainerKey::Sysctl),
            "Ulimit" => EntryKind::Container(ContainerKey::Ulimit),
            "AddDevice" => EntryKind::Container(ContainerKey::AddDevice),
            "Memory" => EntryKind::Container(ContainerKey::Memory),
            _ => EntryKind::Unknown,
        },
        SectionKind::Pod => match key {
            "AddHost" => EntryKind::Pod(PodKey::AddHost),
            "PodName" => EntryKind::Pod(PodKey::PodName),
            "PublishPort" => EntryKind::Pod(PodKey::PublishPort),
            "Network" => EntryKind::Pod(PodKey::Network),
            "Volume" => EntryKind::Pod(PodKey::Volume),
            "UserNS" => EntryKind::Pod(PodKey::UserNS),
            "ShmSize" => EntryKind::Pod(PodKey::ShmSize),
            _ => EntryKind::Unknown,
        },
        SectionKind::Network => match key {
            "NetworkName" => EntryKind::Network(NetworkKey::NetworkName),
            _ => EntryKind::Unknown,
        },
        SectionKind::Volume => match key {
            "VolumeName" => EntryKind::Volume(VolumeKey::VolumeName),
            _ => EntryKind::Unknown,
        },
        SectionKind::Unknown => EntryKind::Unknown,
    }
}

fn classify_value(kind: EntryKind, raw: &str) -> ValueKind {
    let value = raw.trim();
    match kind {
        EntryKind::Container(ContainerKey::Image) => reference_by_suffix(value)
            .filter(|kind| matches!(kind, UnitReferenceKind::Image | UnitReferenceKind::Build))
            .map_or(ValueKind::Opaque, ValueKind::UnitReference),
        EntryKind::Container(ContainerKey::EnvironmentFile) => {
            let path = value.strip_prefix('-').unwrap_or(value).trim_start();
            ValueKind::Path(classify_path(path))
        }
        EntryKind::Container(ContainerKey::Rootfs) => ValueKind::Path(classify_path(value)),
        EntryKind::Container(ContainerKey::Volume) => {
            let source = value.split_once(':').map_or(value, |(source, _)| source);
            if reference_by_suffix(source) == Some(UnitReferenceKind::Volume) {
                ValueKind::UnitReference(UnitReferenceKind::Volume)
            } else {
                ValueKind::Path(classify_path(source))
            }
        }
        EntryKind::Container(ContainerKey::Network) => reference_by_suffix(first_token(value))
            .filter(|kind| *kind == UnitReferenceKind::Network)
            .map_or(ValueKind::Opaque, ValueKind::UnitReference),
        EntryKind::Container(ContainerKey::Pod) => reference_by_suffix(first_token(value))
            .filter(|kind| *kind == UnitReferenceKind::Pod)
            .map_or(ValueKind::Opaque, ValueKind::UnitReference),
        EntryKind::Pod(PodKey::Volume) => {
            let source = value.split_once(':').map_or(value, |(source, _)| source);
            if reference_by_suffix(source) == Some(UnitReferenceKind::Volume) {
                ValueKind::UnitReference(UnitReferenceKind::Volume)
            } else {
                ValueKind::Path(classify_path(source))
            }
        }
        EntryKind::Pod(PodKey::Network) => reference_by_suffix(first_token(value))
            .filter(|kind| *kind == UnitReferenceKind::Network)
            .map_or(ValueKind::Opaque, ValueKind::UnitReference),
        _ => ValueKind::Opaque,
    }
}

fn first_token(value: &str) -> &str {
    value.split_ascii_whitespace().next().unwrap_or(value)
}

fn reference_by_suffix(value: &str) -> Option<UnitReferenceKind> {
    let (stem, suffix) = value.rsplit_once('.')?;
    if stem.is_empty() {
        return None;
    }
    match suffix {
        "image" => Some(UnitReferenceKind::Image),
        "build" => Some(UnitReferenceKind::Build),
        "pod" => Some(UnitReferenceKind::Pod),
        "network" => Some(UnitReferenceKind::Network),
        "volume" => Some(UnitReferenceKind::Volume),
        _ => None,
    }
}
