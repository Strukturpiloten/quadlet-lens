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
const MISSING_IMAGE_SOURCE: DiagnosticCode = DiagnosticCode::new("QLM0008");
const EMPTY_IMAGE_SOURCE: DiagnosticCode = DiagnosticCode::new("QLM0009");
const CONFLICTING_RELOAD_KEYS: DiagnosticCode = DiagnosticCode::new("QLM0010");
const START_WITH_POD_WITHOUT_POD: DiagnosticCode = DiagnosticCode::new("QLM0011");
const READ_ONLY_TMPFS_WITHOUT_READ_ONLY: DiagnosticCode = DiagnosticCode::new("QLM0012");
const CONFLICTING_USERNS_MAPPING: DiagnosticCode = DiagnosticCode::new("QLM0013");
const CONFLICTING_UID_MAPPING: DiagnosticCode = DiagnosticCode::new("QLM0014");
const CONFLICTING_GID_MAPPING: DiagnosticCode = DiagnosticCode::new("QLM0015");
const MAPPING_WITH_POD: DiagnosticCode = DiagnosticCode::new("QLM0016");

/// Native Quadlet unit types supported by the typed model.
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
    /// A `.build` unit.
    Build,
    /// An `.image` unit.
    Image,
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
            "build" => Some(Self::Build),
            "image" => Some(Self::Image),
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
            Self::Build => SectionKind::Build,
            Self::Image => SectionKind::Image,
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
    /// Native Quadlet `[Build]` section.
    Build,
    /// Native Quadlet `[Image]` section.
    Image,
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
            "Build" => Self::Build,
            "Image" => Self::Image,
            _ => Self::Unknown,
        }
    }

    const fn is_native(self) -> bool {
        matches!(
            self,
            Self::Container | Self::Pod | Self::Network | Self::Volume | Self::Build | Self::Image
        )
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
    /// Authored DNS resolver address passed to the container.
    DNS,
    /// Authored DNS resolver option passed to the container.
    DNSOption,
    /// Authored DNS search domain passed to the container.
    DNSSearch,
    /// Authored host port or port range exposed by the container.
    ExposeHostPort,
    /// Authored OCI annotation assignment attached to the container.
    Annotation,
    /// Authored `AppArmor` confinement profile passed to the container.
    AppArmor,
    /// Authored systemd boolean controlling Podman's no-new-privileges option.
    NoNewPrivileges,
    /// Authored seccomp profile selection passed to the container.
    SeccompProfile,
    /// Authored systemd boolean disabling container security-label separation.
    SecurityLabelDisable,
    /// Authored `SELinux` file-type label applied to container files.
    SecurityLabelFileType,
    /// Authored `SELinux` MLS/MCS label level applied to the container.
    SecurityLabelLevel,
    /// Authored systemd boolean enabling nested container security labeling.
    SecurityLabelNested,
    /// Authored `SELinux` process-type label applied to the container.
    SecurityLabelType,
    /// Authored container path list passed to Podman's mask security option.
    Mask,
    /// Authored container path list passed to Podman's unmask security option.
    Unmask,
    /// Authored logging driver passed to the container.
    LogDriver,
    /// Authored logging options passed to the container.
    LogOpt,
    /// Authored static IPv4 address passed to the container.
    IP,
    /// Authored static IPv6 address passed to the container.
    IP6,
    /// Authored alias assigned to the container on its selected network.
    NetworkAlias,
    /// Authored command run by Podman when reloading the container.
    ReloadCmd,
    /// Authored signal sent by Podman when reloading the container.
    ReloadSignal,
    /// Authored automatic image-update policy retained without image-pull interpretation.
    AutoUpdate,
    /// Authored cgroup-management mode retained without cgroup interpretation.
    CgroupsMode,
    /// Authored environment-inheritance selection retained without reading the process environment.
    EnvironmentHost,
    /// Authored supplementary group-ID mapping. Physical entries remain ordered and unparsed.
    GIDMap,
    /// Authored proxy-environment selection retained without inspecting proxy environment variables.
    HttpProxy,
    /// Authored native `--mount` spelling. Physical entries remain ordered and unparsed.
    Mount,
    /// Authored temporary-filesystem read-only selection retained without mount interpretation.
    ReadOnlyTmpfs,
    /// Authored retry-count text retained without integer parsing or default selection.
    Retry,
    /// Authored retry-delay text retained without duration parsing or default selection.
    RetryDelay,
    /// Authored pod-start selection retained without systemd activation interpretation.
    StartWithPod,
    /// Authored subordinate group-mapping selection retained without host-file access.
    SubGIDMap,
    /// Authored subordinate user-mapping selection retained without host-file access.
    SubUIDMap,
    /// Authored timezone selection retained without host timezone lookup.
    Timezone,
    /// Authored user-ID mapping. Physical entries remain ordered and unparsed.
    UIDMap,
    /// Authored action selected after a health-check failure retained without health execution.
    HealthOnFailure,
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
    /// Authored policy controlling the pod when one member exits.
    ExitPolicy,
    /// Authored timeout passed to the generated pod stop action.
    StopTimeout,
    /// Authored generated service-name text for the pod unit.
    ServiceName,
}

/// Network keys required by the first conversion.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum NetworkKey {
    /// Runtime name assigned to the generated Podman network.
    NetworkName,
    /// Authored Podman network driver selection.
    Driver,
    /// Authored Podman network creation options.
    Options,
    /// Authored external-access restriction for the network.
    Internal,
    /// Authored dual-stack IPv6 network selection.
    IPv6,
    /// Authored IP address-management driver selection.
    IPAMDriver,
    /// Authored network subnet column value.
    Subnet,
    /// Authored gateway column value paired by the target with a subnet.
    Gateway,
    /// Authored allocatable address-range column value paired by the target with a subnet.
    IPRange,
    /// Authored OCI label assignment for the network.
    Label,
}

/// Volume keys required by the first conversion.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum VolumeKey {
    /// Runtime name assigned to the generated Podman volume.
    VolumeName,
    /// Authored Podman volume-driver selection.
    Driver,
    /// Authored raw mount-option string passed as one `o=` volume option.
    Options,
    /// Authored OCI label assignment attached to the volume.
    Label,
    /// Authored volume source passed as the local driver's `device` option.
    Device,
    /// Authored filesystem type passed as the local driver's `type` option.
    Type,
    /// Authored opaque copy-up selection retained without boolean coercion.
    Copy,
    /// Opaque containers.conf module text. Physical entries remain ordered and unparsed.
    ContainersConfModule,
    /// Opaque Podman global-argument text. Physical entries remain ordered and unparsed.
    GlobalArgs,
    /// Opaque Podman argument text. Physical entries remain ordered and unparsed.
    PodmanArgs,
    /// Opaque authored volume owner selection retained without user or UID interpretation.
    User,
    /// Opaque authored volume group selection retained without group or GID interpretation.
    Group,
    /// Opaque authored volume UID selection retained without numeric interpretation.
    UID,
    /// Opaque authored volume GID selection retained without numeric interpretation.
    GID,
    /// Opaque generated-service-name text retained without identity interpretation.
    ServiceName,
    /// Authored image source or exact `.image`/`.build` unit reference for the volume.
    Image,
}

/// Minimal native Build keys with evidence-backed typed construction.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum BuildKey {
    /// Name assigned to the image built by this unit. Multiple tags remain ordered and distinct.
    ImageTag,
    /// Build context selected for Podman's build command and, for `file` or `unit`, its service.
    SetWorkingDirectory,
    /// Containerfile selection. Every authored physical line remains ordered and distinct.
    File,
    /// Build stage selection retained without stage-name validation.
    Target,
    /// Build-time network selection or an exact `.network` unit reference.
    Network,
    /// Opaque build-result label text. Physical entries remain ordered and unparsed.
    Label,
    /// Opaque build argument text. Physical entries remain ordered and unparsed.
    BuildArg,
    /// Opaque build secret text. Physical entries remain ordered and unparsed.
    Secret,
    /// Opaque architecture selection retained without platform grammar parsing.
    Arch,
    /// Opaque architecture-variant selection retained without platform grammar parsing.
    Variant,
    /// Opaque image pull-policy selection retained without policy validation.
    Pull,
    /// Opaque Podman build argument text. Physical entries remain ordered and unparsed.
    PodmanArgs,
    /// Opaque build retry-count text retained without integer parsing or default selection.
    Retry,
    /// Opaque build retry-delay text retained without duration parsing or default selection.
    RetryDelay,
    /// Opaque build TLS-verification text retained without boolean parsing or default selection.
    TLSVerify,
    /// Opaque build force-removal text retained without boolean parsing or default selection.
    ForceRM,
    /// Opaque build supplementary-group text. Physical entries remain ordered and unparsed.
    GroupAdd,
    /// Opaque build DNS-server text. Physical entries remain ordered and unparsed.
    DNS,
    /// Opaque build DNS-option text. Physical entries remain ordered and unparsed.
    DNSOption,
    /// Opaque build DNS-search text. Physical entries remain ordered and unparsed.
    DNSSearch,
    /// Opaque build registry-authentication file text retained without path interpretation.
    AuthFile,
    /// Opaque build ignore-file text retained without path or ignore-rule interpretation.
    IgnoreFile,
    /// Opaque build OCI annotation text. Physical entries remain ordered and unparsed.
    Annotation,
    /// Opaque build environment text. Physical entries remain ordered and unparsed.
    Environment,
    /// Opaque containers.conf module text. Physical entries remain ordered and unparsed.
    ContainersConfModule,
    /// Opaque Podman global-argument text. Physical entries remain ordered and unparsed.
    GlobalArgs,
    /// Opaque generated-service-name text retained without identity interpretation.
    ServiceName,
    /// Opaque build volume text with only an exact `.volume` source-prefix reference classification.
    Volume,
}

/// Minimal native Image key with evidence-backed typed construction.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ImageKey {
    /// Opaque native image source retained without image-reference interpretation.
    Image,
    /// Opaque image resource-name override retained without substitution interpretation.
    ImageTag,
    /// Opaque generated-service-name text retained without identity interpretation.
    ServiceName,
    /// Opaque pull-all-tags text retained without boolean interpretation.
    AllTags,
    /// Opaque image architecture text retained without platform interpretation.
    Arch,
    /// Opaque image authentication-file text retained without path or credential interpretation.
    AuthFile,
    /// Opaque image certificate-directory text retained without path or certificate interpretation.
    CertDir,
    /// Opaque containers.conf module text. Physical entries remain ordered and unparsed.
    ContainersConfModule,
    /// Opaque image credential text retained without username/password interpretation.
    Creds,
    /// Opaque image decryption-key text retained without key or passphrase interpretation.
    DecryptionKey,
    /// Opaque image global-argument text. Physical entries remain ordered and unparsed.
    GlobalArgs,
    /// Opaque image operating-system selection retained without platform interpretation.
    OS,
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
    /// Recognized key in `[Build]`.
    Build(BuildKey),
    /// Recognized key in `[Image]`.
    Image(ImageKey),
}

impl EntryKind {
    /// Returns whether this entry's authored value must be redacted by repository-owned debug output.
    ///
    /// This is crate-private metadata rather than a value-classification API: only `Image=Creds`
    /// and `Image=DecryptionKey` have evidence-backed sensitive spellings at this layer.
    pub(crate) const fn has_sensitive_value(self) -> bool {
        matches!(self, Self::Image(ImageKey::Creds | ImageKey::DecryptionKey))
    }

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
                        | ContainerKey::DNS
                        | ContainerKey::DNSOption
                        | ContainerKey::DNSSearch
                        | ContainerKey::ExposeHostPort
                        | ContainerKey::Annotation
                        | ContainerKey::Mask
                        | ContainerKey::Unmask
                        | ContainerKey::LogOpt
                        | ContainerKey::NetworkAlias
                        | ContainerKey::GIDMap
                        | ContainerKey::Mount
                        | ContainerKey::UIDMap
                )
                | Self::Pod(PodKey::AddHost | PodKey::PublishPort | PodKey::Network | PodKey::Volume)
                | Self::Network(
                    NetworkKey::Options
                        | NetworkKey::Subnet
                        | NetworkKey::Gateway
                        | NetworkKey::IPRange
                        | NetworkKey::Label
                )
                | Self::Volume(
                    VolumeKey::Label | VolumeKey::ContainersConfModule | VolumeKey::GlobalArgs | VolumeKey::PodmanArgs
                )
                | Self::Build(
                    BuildKey::ImageTag
                        | BuildKey::File
                        | BuildKey::Network
                        | BuildKey::Label
                        | BuildKey::BuildArg
                        | BuildKey::Secret
                        | BuildKey::PodmanArgs
                        | BuildKey::GroupAdd
                        | BuildKey::DNS
                        | BuildKey::DNSOption
                        | BuildKey::DNSSearch
                        | BuildKey::Annotation
                        | BuildKey::Environment
                        | BuildKey::ContainersConfModule
                        | BuildKey::GlobalArgs
                        | BuildKey::Volume
                )
                | Self::Image(ImageKey::ContainersConfModule | ImageKey::GlobalArgs)
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
#[derive(Clone, Eq, PartialEq)]
pub struct SourcedText {
    text: String,
    span: SourceSpan,
    sensitive: bool,
}

impl fmt::Debug for SourcedText {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text: &dyn fmt::Debug = if self.sensitive {
            &"<redacted sensitive text>"
        } else {
            &self.text
        };
        formatter
            .debug_struct("SourcedText")
            .field("text", text)
            .field("span", &self.span)
            .finish()
    }
}

impl SourcedText {
    fn from_span(source: &SourceText, span: SourceSpan) -> Result<Self, TypedModelError> {
        let text = source
            .slice(span)
            .ok_or(TypedModelError::InvalidSourceSpan(span))?
            .to_owned();
        Ok(Self {
            text,
            span,
            sensitive: false,
        })
    }

    const fn with_sensitive_value(mut self, sensitive: bool) -> Self {
        self.sensitive = sensitive;
        self
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
#[derive(Clone, Eq, PartialEq)]
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

    /// Returns whether explicit raw-value access needs sensitive-data handling.
    ///
    /// This is currently true only for recognized `[Image] Creds=` and `DecryptionKey=` entries.
    /// Rendering and [`Self::value`] retain the exact authored text, so callers must avoid
    /// exposing that text.
    #[must_use]
    pub const fn is_sensitive(&self) -> bool {
        self.kind.has_sensitive_value()
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
            EntryKind::Container(ContainerKey::Volume)
            | EntryKind::Pod(PodKey::Volume)
            | EntryKind::Build(BuildKey::Volume) => {
                Some(value.split_once(':').map_or(value, |(source, _)| source).trim())
            }
            EntryKind::Container(ContainerKey::Network | ContainerKey::Pod) | EntryKind::Pod(PodKey::Network) => {
                Some(first_token(value))
            }
            EntryKind::Container(ContainerKey::Image)
            | EntryKind::Build(BuildKey::Network)
            | EntryKind::Volume(VolumeKey::Image) => Some(value),
            _ => None,
        }
    }
}

impl fmt::Debug for TypedEntry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = formatter.debug_struct("TypedEntry");
        debug
            .field("key", &self.key)
            .field("kind", &self.kind)
            .field("value_kind", &self.value_kind)
            .field("source_line", &self.source_line);
        if self.kind.has_sensitive_value() {
            debug.field("value", &"<redacted sensitive value>")
        } else {
            debug.field("value", &self.value)
        };
        debug.finish()
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
                    let section_kind = sections[section_index].kind;
                    let kind = classify_entry(section_kind, key.text());
                    let sensitive = kind.has_sensitive_value();
                    let primary =
                        SourcedText::from_span(syntax.source(), entry.value())?.with_sensitive_value(sensitive);
                    let continuations = collect_continuations(syntax, line_index, sensitive)?;
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
        if self.unit_type == QuadletUnitType::Image {
            diagnostics.extend(self.validate_image_source(first_expected));
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
        let images = container_entries_with_key(&container_entries, ContainerKey::Image);
        let root_filesystems = container_entries_with_key(&container_entries, ContainerKey::Rootfs);
        let mut diagnostics = validate_container_workload_sources(container_section, &images, &root_filesystems);
        diagnostics.extend(validate_container_reload_keys(&container_entries));
        diagnostics.extend(validate_container_relationships(&container_entries));
        diagnostics.extend(validate_empty_container_workload_sources(&images, &root_filesystems));
        diagnostics
    }

    fn validate_image_source(&self, image_section: Option<&TypedSection>) -> Vec<Diagnostic> {
        let images: Vec<_> = self
            .sections
            .iter()
            .filter(|section| section.kind == SectionKind::Image)
            .flat_map(|section| section.entries.iter())
            .filter(|entry| entry.kind == EntryKind::Image(ImageKey::Image))
            .collect();
        let mut diagnostics = Vec::new();
        if images.is_empty() {
            if let Some(section) = image_section {
                diagnostics.push(Diagnostic::new(
                    MISSING_IMAGE_SOURCE,
                    Severity::Error,
                    "image unit is missing its required image source",
                    Label::new(section.name.span(), "add `Image=` to this Image section"),
                ));
            }
        }
        diagnostics.extend(
            images
                .iter()
                .filter(|entry| entry.value.primary.text.trim().is_empty())
                .map(|entry| {
                    Diagnostic::new(
                        EMPTY_IMAGE_SOURCE,
                        Severity::Error,
                        "image unit Image entry is empty",
                        Label::new(entry.value.primary.span(), "provide an image source"),
                    )
                }),
        );
        diagnostics
    }
}

/// Combined syntax and typed-model result for one source file.
#[derive(Clone, Eq, PartialEq)]
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

impl fmt::Debug for QuadletParseResult {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = formatter.debug_struct("QuadletParseResult");
        if self.document.entries().any(TypedEntry::is_sensitive) {
            debug.field(
                "syntax",
                &"<loss-aware syntax; access syntax() explicitly for raw source>",
            );
        } else {
            debug.field("syntax", &self.syntax);
        }
        debug
            .field("document", &self.document)
            .field("model_diagnostics", &self.model_diagnostics)
            .finish()
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

fn collect_continuations(
    syntax: &SyntaxDocument,
    entry_line: usize,
    sensitive: bool,
) -> Result<Vec<SourcedText>, TypedModelError> {
    let mut values = Vec::new();
    for line in syntax.lines().iter().skip(entry_line + 1) {
        match line.kind() {
            SyntaxLineKind::Continuation(continuation) => {
                values.push(
                    SourcedText::from_span(syntax.source(), continuation.value())?.with_sensitive_value(sensitive),
                );
            }
            SyntaxLineKind::Comment(comment) if comment.within_continuation() => {}
            _ => break,
        }
    }
    Ok(values)
}

fn container_entries_with_key<'a>(entries: &[&'a TypedEntry], key: ContainerKey) -> Vec<&'a TypedEntry> {
    entries
        .iter()
        .copied()
        .filter(|entry| entry.kind == EntryKind::Container(key))
        .collect()
}

fn validate_container_workload_sources(
    container_section: Option<&TypedSection>,
    images: &[&TypedEntry],
    root_filesystems: &[&TypedEntry],
) -> Vec<Diagnostic> {
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
    diagnostics
}

fn validate_container_reload_keys(entries: &[&TypedEntry]) -> Vec<Diagnostic> {
    let reload_commands = container_entries_with_key(entries, ContainerKey::ReloadCmd);
    let reload_signals = container_entries_with_key(entries, ContainerKey::ReloadSignal);
    if reload_commands.is_empty() || reload_signals.is_empty() {
        return Vec::new();
    }
    vec![Diagnostic::new(
        CONFLICTING_RELOAD_KEYS,
        Severity::Error,
        "container ReloadCmd and ReloadSignal entries conflict",
        Label::new(
            reload_signals[0].key.span(),
            "remove either ReloadSignal or ReloadCmd from this Container section",
        ),
    )]
}

fn validate_empty_container_workload_sources(
    images: &[&TypedEntry],
    root_filesystems: &[&TypedEntry],
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
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

struct ContainerRelationshipEntries<'a> {
    pod: Vec<&'a TypedEntry>,
    user_namespace: Vec<&'a TypedEntry>,
    direct_maps: [Vec<&'a TypedEntry>; 2],
    subordinate_maps: [Vec<&'a TypedEntry>; 2],
    start_with_pod: Vec<&'a TypedEntry>,
    read_only_tmpfs: Vec<&'a TypedEntry>,
    read_only: Vec<&'a TypedEntry>,
}

impl<'a> ContainerRelationshipEntries<'a> {
    fn collect(entries: &[&'a TypedEntry]) -> Self {
        Self {
            pod: container_entries_with_key(entries, ContainerKey::Pod),
            user_namespace: container_entries_with_key(entries, ContainerKey::UserNS),
            direct_maps: [
                container_entries_with_key(entries, ContainerKey::UIDMap),
                container_entries_with_key(entries, ContainerKey::GIDMap),
            ],
            subordinate_maps: [
                container_entries_with_key(entries, ContainerKey::SubUIDMap),
                container_entries_with_key(entries, ContainerKey::SubGIDMap),
            ],
            start_with_pod: container_entries_with_key(entries, ContainerKey::StartWithPod),
            read_only_tmpfs: container_entries_with_key(entries, ContainerKey::ReadOnlyTmpfs),
            read_only: container_entries_with_key(entries, ContainerKey::ReadOnly),
        }
    }
}

fn validate_container_relationships(entries: &[&TypedEntry]) -> Vec<Diagnostic> {
    let relationships = ContainerRelationshipEntries::collect(entries);
    let active_pod = effective_singleton(&relationships.pod);
    let active_direct_maps = [
        reset_aware_entries(&relationships.direct_maps[0]),
        reset_aware_entries(&relationships.direct_maps[1]),
    ];
    let active_subordinate_maps = [
        effective_singleton(&relationships.subordinate_maps[0]),
        effective_singleton(&relationships.subordinate_maps[1]),
    ];
    let has_active_mappings =
        active_direct_maps.iter().any(|maps| !maps.is_empty()) || active_subordinate_maps.iter().any(Option::is_some);
    let mut diagnostics = Vec::new();

    if let Some((entry, true)) = effective_boolean(&relationships.start_with_pod) {
        if active_pod.is_none() {
            diagnostics.push(container_relationship_diagnostic(
                START_WITH_POD_WITHOUT_POD,
                Severity::Warning,
                "container StartWithPod entry has no Pod entry",
                entry,
                "add Pod= or remove StartWithPod=",
            ));
        }
    }
    if let Some((entry, true)) = effective_boolean(&relationships.read_only_tmpfs) {
        if !matches!(effective_boolean(&relationships.read_only), Some((_, true))) {
            diagnostics.push(container_relationship_diagnostic(
                READ_ONLY_TMPFS_WITHOUT_READ_ONLY,
                Severity::Warning,
                "container ReadOnlyTmpfs entry has no ReadOnly entry",
                entry,
                "add ReadOnly= or remove ReadOnlyTmpfs=",
            ));
        }
    }
    if let Some(user_namespace) = effective_singleton(&relationships.user_namespace) {
        if has_active_mappings {
            diagnostics.push(container_relationship_diagnostic(
                CONFLICTING_USERNS_MAPPING,
                Severity::Error,
                "container UserNS and explicit user mappings conflict",
                user_namespace,
                "remove UserNS= or the explicit mapping entries",
            ));
        }
    }
    if let Some(subordinate_user) = active_subordinate_maps[0] {
        if !active_direct_maps[0].is_empty() {
            diagnostics.push(container_relationship_diagnostic(
                CONFLICTING_UID_MAPPING,
                Severity::Error,
                "container UIDMap and SubUIDMap entries conflict",
                subordinate_user,
                "remove UIDMap= or SubUIDMap=",
            ));
        }
    }
    if let Some(subordinate_group) = active_subordinate_maps[1] {
        if !active_direct_maps[1].is_empty() {
            diagnostics.push(container_relationship_diagnostic(
                CONFLICTING_GID_MAPPING,
                Severity::Error,
                "container GIDMap and SubGIDMap entries conflict",
                subordinate_group,
                "remove GIDMap= or SubGIDMap=",
            ));
        }
    }
    if let Some(pod) = active_pod {
        if has_active_mappings {
            diagnostics.push(container_relationship_diagnostic(
                MAPPING_WITH_POD,
                Severity::Error,
                "container explicit user mappings conflict with Pod membership",
                pod,
                "move namespace mapping policy to the Pod or remove Pod=",
            ));
        }
    }
    diagnostics
}

fn container_relationship_diagnostic(
    code: DiagnosticCode,
    severity: Severity,
    message: &'static str,
    entry: &TypedEntry,
    label: &'static str,
) -> Diagnostic {
    Diagnostic::new(code, severity, message, Label::new(entry.key.span(), label))
}

fn classify_entry(section: SectionKind, key: &str) -> EntryKind {
    match section {
        SectionKind::Unit | SectionKind::Service | SectionKind::Install => EntryKind::GenericSystemd,
        SectionKind::Container => classify_container_entry(key),
        SectionKind::Pod => match key {
            "AddHost" => EntryKind::Pod(PodKey::AddHost),
            "PodName" => EntryKind::Pod(PodKey::PodName),
            "PublishPort" => EntryKind::Pod(PodKey::PublishPort),
            "Network" => EntryKind::Pod(PodKey::Network),
            "Volume" => EntryKind::Pod(PodKey::Volume),
            "UserNS" => EntryKind::Pod(PodKey::UserNS),
            "ShmSize" => EntryKind::Pod(PodKey::ShmSize),
            "ExitPolicy" => EntryKind::Pod(PodKey::ExitPolicy),
            "StopTimeout" => EntryKind::Pod(PodKey::StopTimeout),
            "ServiceName" => EntryKind::Pod(PodKey::ServiceName),
            _ => EntryKind::Unknown,
        },
        SectionKind::Network => match key {
            "NetworkName" => EntryKind::Network(NetworkKey::NetworkName),
            "Driver" => EntryKind::Network(NetworkKey::Driver),
            "Options" => EntryKind::Network(NetworkKey::Options),
            "Internal" => EntryKind::Network(NetworkKey::Internal),
            "IPv6" => EntryKind::Network(NetworkKey::IPv6),
            "IPAMDriver" => EntryKind::Network(NetworkKey::IPAMDriver),
            "Subnet" => EntryKind::Network(NetworkKey::Subnet),
            "Gateway" => EntryKind::Network(NetworkKey::Gateway),
            "IPRange" => EntryKind::Network(NetworkKey::IPRange),
            "Label" => EntryKind::Network(NetworkKey::Label),
            _ => EntryKind::Unknown,
        },
        SectionKind::Volume => classify_volume_entry(key),
        SectionKind::Build => classify_build_entry(key),
        SectionKind::Image => classify_image_entry(key),
        SectionKind::Unknown => EntryKind::Unknown,
    }
}

fn classify_container_entry(key: &str) -> EntryKind {
    match key {
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
        "DNS" => EntryKind::Container(ContainerKey::DNS),
        "DNSOption" => EntryKind::Container(ContainerKey::DNSOption),
        "DNSSearch" => EntryKind::Container(ContainerKey::DNSSearch),
        "ExposeHostPort" => EntryKind::Container(ContainerKey::ExposeHostPort),
        "Annotation" => EntryKind::Container(ContainerKey::Annotation),
        "AppArmor" => EntryKind::Container(ContainerKey::AppArmor),
        "NoNewPrivileges" => EntryKind::Container(ContainerKey::NoNewPrivileges),
        "SeccompProfile" => EntryKind::Container(ContainerKey::SeccompProfile),
        "SecurityLabelDisable" => EntryKind::Container(ContainerKey::SecurityLabelDisable),
        "SecurityLabelFileType" => EntryKind::Container(ContainerKey::SecurityLabelFileType),
        "SecurityLabelLevel" => EntryKind::Container(ContainerKey::SecurityLabelLevel),
        "SecurityLabelNested" => EntryKind::Container(ContainerKey::SecurityLabelNested),
        "SecurityLabelType" => EntryKind::Container(ContainerKey::SecurityLabelType),
        "Mask" => EntryKind::Container(ContainerKey::Mask),
        "Unmask" => EntryKind::Container(ContainerKey::Unmask),
        "LogDriver" => EntryKind::Container(ContainerKey::LogDriver),
        "LogOpt" => EntryKind::Container(ContainerKey::LogOpt),
        "IP" => EntryKind::Container(ContainerKey::IP),
        "IP6" => EntryKind::Container(ContainerKey::IP6),
        "NetworkAlias" => EntryKind::Container(ContainerKey::NetworkAlias),
        "ReloadCmd" => EntryKind::Container(ContainerKey::ReloadCmd),
        "ReloadSignal" => EntryKind::Container(ContainerKey::ReloadSignal),
        "AutoUpdate" => EntryKind::Container(ContainerKey::AutoUpdate),
        "CgroupsMode" => EntryKind::Container(ContainerKey::CgroupsMode),
        "EnvironmentHost" => EntryKind::Container(ContainerKey::EnvironmentHost),
        "GIDMap" => EntryKind::Container(ContainerKey::GIDMap),
        "HttpProxy" => EntryKind::Container(ContainerKey::HttpProxy),
        "Mount" => EntryKind::Container(ContainerKey::Mount),
        "ReadOnlyTmpfs" => EntryKind::Container(ContainerKey::ReadOnlyTmpfs),
        "Retry" => EntryKind::Container(ContainerKey::Retry),
        "RetryDelay" => EntryKind::Container(ContainerKey::RetryDelay),
        "StartWithPod" => EntryKind::Container(ContainerKey::StartWithPod),
        "SubGIDMap" => EntryKind::Container(ContainerKey::SubGIDMap),
        "SubUIDMap" => EntryKind::Container(ContainerKey::SubUIDMap),
        "Timezone" => EntryKind::Container(ContainerKey::Timezone),
        "UIDMap" => EntryKind::Container(ContainerKey::UIDMap),
        "HealthOnFailure" => EntryKind::Container(ContainerKey::HealthOnFailure),
        _ => EntryKind::Unknown,
    }
}

fn classify_image_entry(key: &str) -> EntryKind {
    match key {
        "Image" => EntryKind::Image(ImageKey::Image),
        "ImageTag" => EntryKind::Image(ImageKey::ImageTag),
        "ServiceName" => EntryKind::Image(ImageKey::ServiceName),
        "AllTags" => EntryKind::Image(ImageKey::AllTags),
        "Arch" => EntryKind::Image(ImageKey::Arch),
        "AuthFile" => EntryKind::Image(ImageKey::AuthFile),
        "CertDir" => EntryKind::Image(ImageKey::CertDir),
        "ContainersConfModule" => EntryKind::Image(ImageKey::ContainersConfModule),
        "Creds" => EntryKind::Image(ImageKey::Creds),
        "DecryptionKey" => EntryKind::Image(ImageKey::DecryptionKey),
        "GlobalArgs" => EntryKind::Image(ImageKey::GlobalArgs),
        "OS" => EntryKind::Image(ImageKey::OS),
        _ => EntryKind::Unknown,
    }
}

fn classify_build_entry(key: &str) -> EntryKind {
    let key = match key {
        "ImageTag" => BuildKey::ImageTag,
        "SetWorkingDirectory" => BuildKey::SetWorkingDirectory,
        "File" => BuildKey::File,
        "Target" => BuildKey::Target,
        "Network" => BuildKey::Network,
        "Label" => BuildKey::Label,
        "BuildArg" => BuildKey::BuildArg,
        "Secret" => BuildKey::Secret,
        "Arch" => BuildKey::Arch,
        "Variant" => BuildKey::Variant,
        "Pull" => BuildKey::Pull,
        "PodmanArgs" => BuildKey::PodmanArgs,
        "Retry" => BuildKey::Retry,
        "RetryDelay" => BuildKey::RetryDelay,
        "TLSVerify" => BuildKey::TLSVerify,
        "ForceRM" => BuildKey::ForceRM,
        "GroupAdd" => BuildKey::GroupAdd,
        "DNS" => BuildKey::DNS,
        "DNSOption" => BuildKey::DNSOption,
        "DNSSearch" => BuildKey::DNSSearch,
        "AuthFile" => BuildKey::AuthFile,
        "IgnoreFile" => BuildKey::IgnoreFile,
        "Annotation" => BuildKey::Annotation,
        "Environment" => BuildKey::Environment,
        "ContainersConfModule" => BuildKey::ContainersConfModule,
        "GlobalArgs" => BuildKey::GlobalArgs,
        "ServiceName" => BuildKey::ServiceName,
        "Volume" => BuildKey::Volume,
        _ => return EntryKind::Unknown,
    };
    EntryKind::Build(key)
}

fn classify_volume_entry(key: &str) -> EntryKind {
    let key = match key {
        "VolumeName" => VolumeKey::VolumeName,
        "Driver" => VolumeKey::Driver,
        "Options" => VolumeKey::Options,
        "Label" => VolumeKey::Label,
        "Device" => VolumeKey::Device,
        "Type" => VolumeKey::Type,
        "Copy" => VolumeKey::Copy,
        "ContainersConfModule" => VolumeKey::ContainersConfModule,
        "GlobalArgs" => VolumeKey::GlobalArgs,
        "PodmanArgs" => VolumeKey::PodmanArgs,
        "User" => VolumeKey::User,
        "Group" => VolumeKey::Group,
        "UID" => VolumeKey::UID,
        "GID" => VolumeKey::GID,
        "ServiceName" => VolumeKey::ServiceName,
        "Image" => VolumeKey::Image,
        _ => return EntryKind::Unknown,
    };
    EntryKind::Volume(key)
}

fn effective_singleton<'a>(entries: &[&'a TypedEntry]) -> Option<&'a TypedEntry> {
    entries
        .last()
        .copied()
        .filter(|entry| !entry.value.primary.text.trim().is_empty())
}

fn reset_aware_entries<'a>(entries: &[&'a TypedEntry]) -> Vec<&'a TypedEntry> {
    let reset = entries
        .iter()
        .rposition(|entry| entry.value.primary.text.trim().is_empty());
    entries
        .iter()
        .skip(reset.map_or(0, |index| index + 1))
        .copied()
        .filter(|entry| !entry.value.primary.text.trim().is_empty())
        .collect()
}

fn effective_boolean<'a>(entries: &[&'a TypedEntry]) -> Option<(&'a TypedEntry, bool)> {
    let entry = effective_singleton(entries)?;
    let value = systemd_lookup_value(entry.value.primary.text());
    if ["1", "yes", "true", "on"]
        .iter()
        .any(|form| value.eq_ignore_ascii_case(form))
    {
        return Some((entry, true));
    }
    if ["0", "no", "false", "off"]
        .iter()
        .any(|form| value.eq_ignore_ascii_case(form))
    {
        return Some((entry, false));
    }
    None
}

/// Mirrors the semantic preprocessing performed by Podman's `UnitFile.Lookup` for one value.
///
/// Authored text remains source-owned and unchanged; this is used only for narrow diagnostics
/// that need the generator's matched-double-quote behavior before evaluating a boolean spelling.
fn systemd_lookup_value(value: &str) -> &str {
    let value = value.trim_end_matches(char::is_whitespace);
    value
        .strip_prefix('"')
        .and_then(|unquoted| unquoted.strip_suffix('"'))
        .unwrap_or(value)
}

fn classify_value(kind: EntryKind, raw: &str) -> ValueKind {
    let value = raw.trim();
    match kind {
        EntryKind::Container(ContainerKey::Image) | EntryKind::Volume(VolumeKey::Image) => reference_by_suffix(value)
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
        EntryKind::Build(BuildKey::Volume) => {
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
        EntryKind::Build(BuildKey::Network) => reference_by_suffix(value)
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
