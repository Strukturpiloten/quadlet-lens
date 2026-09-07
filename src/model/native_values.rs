//! Bounded, source-aware native Quadlet value views.
//!
//! These views deliberately decode only native spelling and local structure. They neither select a
//! Podman target nor decide whether a native value has a portable cross-format representation.

use std::fmt;

use crate::{
    diagnostic::{Diagnostic, DiagnosticCode, Label, Severity},
    source::SourceSpan,
};

use super::{
    ContainerKey, EntryKind, PodKey, QuadletDocument, SectionKind, TypedEntry, logical_authored_value,
    systemd_environment_tokens,
};

const MALFORMED_PORT: DiagnosticCode = DiagnosticCode::new("QLM0029");
const DEFERRED_PORT: DiagnosticCode = DiagnosticCode::new("QLM0030");
const MALFORMED_MOUNT: DiagnosticCode = DiagnosticCode::new("QLM0031");
const DEFERRED_MOUNT: DiagnosticCode = DiagnosticCode::new("QLM0032");
const MALFORMED_COMMAND: DiagnosticCode = DiagnosticCode::new("QLM0033");
const DEFERRED_COMMAND: DiagnosticCode = DiagnosticCode::new("QLM0034");

/// Inclusive native port or port-range spelling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativePortRange {
    start: u16,
    end: u16,
}

impl NativePortRange {
    /// Returns the first port in the inclusive range.
    #[must_use]
    pub const fn start(self) -> u16 {
        self.start
    }

    /// Returns the last port in the inclusive range.
    #[must_use]
    pub const fn end(self) -> u16 {
        self.end
    }

    /// Returns the number of ports represented by this range.
    #[must_use]
    pub const fn len(self) -> u16 {
        self.end - self.start + 1
    }

    /// Returns whether the range contains no ports. Native port ranges are always non-empty.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        false
    }

    /// Returns whether this contains exactly one port.
    #[must_use]
    pub const fn is_single(self) -> bool {
        self.start == self.end
    }
}

/// Native Podman publication protocol.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum NativePortProtocol {
    /// Transmission Control Protocol.
    Tcp,
    /// User Datagram Protocol.
    Udp,
    /// Stream Control Transmission Protocol.
    Sctp,
}

/// One fully decoded native `PublishPort=` directive.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativePortSpecification {
    host: Option<String>,
    published: Option<NativePortRange>,
    container: NativePortRange,
    protocol: NativePortProtocol,
    span: SourceSpan,
}

impl NativePortSpecification {
    /// Returns the literal host address spelling, including brackets for IPv6, when supplied.
    #[must_use]
    pub fn host(&self) -> Option<&str> {
        self.host.as_deref()
    }

    /// Returns the requested host port or range. `None` lets Podman choose a host port.
    #[must_use]
    pub const fn published(&self) -> Option<NativePortRange> {
        self.published
    }

    /// Returns the required container port or range.
    #[must_use]
    pub const fn container(&self) -> NativePortRange {
        self.container
    }

    /// Returns the native protocol.
    #[must_use]
    pub const fn protocol(&self) -> NativePortProtocol {
        self.protocol
    }

    /// Returns the primary physical source-value segment span.
    #[must_use]
    pub const fn span(&self) -> SourceSpan {
        self.span
    }
}

/// One ordered `PublishPort=` result.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum NativePortPublication {
    /// A complete native publication.
    Publication(NativePortSpecification),
    /// A blank repeated directive resets preceding native publications.
    Reset {
        /// Primary physical source-value segment span.
        span: SourceSpan,
    },
    /// The authored value needs systemd-specifier expansion before it can be interpreted.
    Deferred {
        /// Primary physical source-value segment span.
        span: SourceSpan,
    },
    /// The authored value was malformed; inspect diagnostics for the recoverable reason.
    Unmodeled {
        /// Primary physical source-value segment span.
        span: SourceSpan,
    },
}

impl NativePortPublication {
    /// Returns the primary physical source-value segment span.
    #[must_use]
    pub const fn span(&self) -> SourceSpan {
        match self {
            Self::Publication(value) => value.span(),
            Self::Reset { span } | Self::Deferred { span } | Self::Unmodeled { span } => *span,
        }
    }
}

/// Ordered port directives plus recoverable native diagnostics.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativePortPublications {
    directives: Vec<NativePortPublication>,
    diagnostics: Vec<Diagnostic>,
}

impl NativePortPublications {
    pub(super) fn from_document(document: &QuadletDocument, section: SectionKind) -> Self {
        let mut directives = Vec::new();
        let mut diagnostics = Vec::new();
        for entry in entries(document, section, NativeKey::Port) {
            let span = entry.value().primary().span();
            match native_single_value(entry) {
                Ok(value) if value.is_empty() => directives.push(NativePortPublication::Reset { span }),
                Ok(value) if literal_specifier_value(&value).is_none() => {
                    diagnostics.push(diagnostic(
                        DEFERRED_PORT,
                        span,
                        "native PublishPort value contains a systemd specifier",
                        "expand the specifier in the target manager before interpreting the publication",
                    ));
                    directives.push(NativePortPublication::Deferred { span });
                }
                Ok(value) => match parse_port(&literal_specifier_value(&value).unwrap_or_default(), span) {
                    Ok(value) => directives.push(NativePortPublication::Publication(value)),
                    Err(message) => {
                        diagnostics.push(diagnostic(MALFORMED_PORT, span, message, "use a Podman publication form such as hostPort:containerPort or [IPv6]:hostPort:containerPort"));
                        directives.push(NativePortPublication::Unmodeled { span });
                    }
                },
                Err(()) => {
                    diagnostics.push(diagnostic(
                        MALFORMED_PORT,
                        span,
                        "native PublishPort value has incomplete quoting, escaping, or continuation syntax",
                        "use one complete systemd word",
                    ));
                    directives.push(NativePortPublication::Unmodeled { span });
                }
            }
        }
        Self {
            directives,
            diagnostics,
        }
    }

    /// Returns directives in authored order.
    #[must_use]
    pub fn directives(&self) -> &[NativePortPublication] {
        &self.directives
    }

    /// Returns recoverable malformed/deferred diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
}

/// One native long-mount option. Unknown option names remain native evidence.
#[derive(Clone, Eq, PartialEq)]
#[non_exhaustive]
pub enum NativeMountOption {
    /// A bare option such as `readonly`.
    Flag(String),
    /// A `name=value` option; no portability interpretation is implied.
    Assignment {
        /// Literal native option name.
        name: String,
        /// Literal native option value.
        value: String,
    },
}

impl fmt::Debug for NativeMountOption {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Flag(name) => formatter.debug_tuple("Flag").field(name).finish(),
            Self::Assignment { name, .. } => formatter
                .debug_struct("Assignment")
                .field("name", name)
                .field("value", &"<native option>")
                .finish(),
        }
    }
}

/// One fully decoded native `Volume=` or long `Mount=` specification.
#[derive(Clone, Eq, PartialEq)]
pub struct NativeMountSpecification {
    long_form: bool,
    source: Option<String>,
    destination: Option<String>,
    options: Vec<NativeMountOption>,
    span: SourceSpan,
}

impl NativeMountSpecification {
    /// Returns whether this came from long `Mount=` syntax.
    #[must_use]
    pub const fn is_long_form(&self) -> bool {
        self.long_form
    }
    /// Returns the literal source spelling when present. It can be relative or a `.volume`/`.image` reference.
    #[must_use]
    pub fn source(&self) -> Option<&str> {
        self.source.as_deref()
    }
    /// Returns the literal destination spelling when the grammar provides one.
    #[must_use]
    pub fn destination(&self) -> Option<&str> {
        self.destination.as_deref()
    }
    /// Returns options in authored order, including options this library does not semantically model.
    #[must_use]
    pub fn options(&self) -> &[NativeMountOption] {
        &self.options
    }
    /// Returns the primary physical source-value segment span.
    #[must_use]
    pub const fn span(&self) -> SourceSpan {
        self.span
    }
}

impl fmt::Debug for NativeMountSpecification {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("NativeMountSpecification")
            .field("long_form", &self.long_form)
            .field("has_source", &self.source.is_some())
            .field("has_destination", &self.destination.is_some())
            .field("options", &self.options)
            .field("span", &self.span)
            .finish()
    }
}

/// One ordered native mount result.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum NativeMountDirective {
    /// A complete native mount; unknown options remain preserved within it.
    Mount(NativeMountSpecification),
    /// A blank repeated directive resets preceding mounts.
    Reset {
        /// Primary physical source-value segment span.
        span: SourceSpan,
    },
    /// The authored value contains a systemd specifier requiring target context.
    Deferred {
        /// Primary physical source-value segment span.
        span: SourceSpan,
    },
    /// The authored value is malformed; inspect diagnostics for detail.
    Unmodeled {
        /// Primary physical source-value segment span.
        span: SourceSpan,
    },
}

impl NativeMountDirective {
    /// Returns the primary physical source-value segment span.
    #[must_use]
    pub const fn span(&self) -> SourceSpan {
        match self {
            Self::Mount(value) => value.span(),
            Self::Reset { span } | Self::Deferred { span } | Self::Unmodeled { span } => *span,
        }
    }
}

/// Ordered mount directives plus recoverable native diagnostics.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeMountSpecifications {
    directives: Vec<NativeMountDirective>,
    diagnostics: Vec<Diagnostic>,
}

impl NativeMountSpecifications {
    pub(super) fn from_document(document: &QuadletDocument, section: SectionKind) -> Self {
        let mut directives = Vec::new();
        let mut diagnostics = Vec::new();
        for entry in entries(document, section, NativeKey::Mount) {
            let span = entry.value().primary().span();
            let long_form = matches!(entry.kind(), EntryKind::Container(ContainerKey::Mount));
            match native_single_value(entry) {
                Ok(value) if value.is_empty() => directives.push(NativeMountDirective::Reset { span }),
                Ok(value) if literal_specifier_value(&value).is_none() => {
                    diagnostics.push(diagnostic(
                        DEFERRED_MOUNT,
                        span,
                        "native mount value contains a systemd specifier",
                        "expand the specifier in the target manager before resolving the mount",
                    ));
                    directives.push(NativeMountDirective::Deferred { span });
                }
                Ok(value) => match if long_form {
                    parse_long_mount(&literal_specifier_value(&value).unwrap_or_default(), span)
                } else {
                    parse_short_mount(&literal_specifier_value(&value).unwrap_or_default(), span)
                } {
                    Ok(value) => directives.push(NativeMountDirective::Mount(value)),
                    Err(message) => {
                        diagnostics.push(diagnostic(
                            MALFORMED_MOUNT,
                            span,
                            message,
                            "use a complete native Volume= or type=TYPE long Mount= specification",
                        ));
                        directives.push(NativeMountDirective::Unmodeled { span });
                    }
                },
                Err(()) => {
                    diagnostics.push(diagnostic(
                        MALFORMED_MOUNT,
                        span,
                        "native mount value has incomplete quoting, escaping, or continuation syntax",
                        "use one complete systemd word",
                    ));
                    directives.push(NativeMountDirective::Unmodeled { span });
                }
            }
        }
        Self {
            directives,
            diagnostics,
        }
    }
    /// Returns directives in authored order.
    #[must_use]
    pub fn directives(&self) -> &[NativeMountDirective] {
        &self.directives
    }
    /// Returns recoverable malformed/deferred diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
}

/// Container command field decoded from native syntax.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum NativeCommandKind {
    /// `Exec=` bounded lexical argv tokenization without systemd execution semantics.
    Exec,
    /// `Entrypoint=` native command spelling.
    Entrypoint,
}

/// Native spelling used for a decoded command.
///
/// This records source syntax only. It does not imply portability or execution
/// support for a particular Podman or systemd target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum NativeCommandSyntax {
    /// `Exec=` systemd lexical word tokenization.
    ExecWords,
    /// `Entrypoint=` JSON string-array syntax.
    EntrypointJsonArray,
    /// `Entrypoint=` literal executable spelling.
    ///
    /// Its decoded argument vector contains the complete literal as one item.
    EntrypointLiteral,
}

/// A decoded native command argument vector.
#[derive(Clone, Eq, PartialEq)]
pub struct NativeCommand {
    arguments: Vec<String>,
    syntax: NativeCommandSyntax,
    span: SourceSpan,
}

impl NativeCommand {
    /// Returns decoded arguments in authored order.
    #[must_use]
    pub fn arguments(&self) -> &[String] {
        &self.arguments
    }

    /// Returns the native syntax from which the arguments were decoded.
    #[must_use]
    pub const fn syntax(&self) -> NativeCommandSyntax {
        self.syntax
    }
    /// Returns the primary physical source-value segment span.
    #[must_use]
    pub const fn span(&self) -> SourceSpan {
        self.span
    }
}

impl fmt::Debug for NativeCommand {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("NativeCommand")
            .field("argument_count", &self.arguments.len())
            .field("syntax", &self.syntax)
            .field("span", &self.span)
            .finish()
    }
}

/// One ordered Container `Exec=` or `Entrypoint=` result.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum NativeCommandDirective {
    /// A fully decoded command.
    Command {
        /// Native key that supplied the command.
        kind: NativeCommandKind,
        /// Decoded protected command arguments.
        command: NativeCommand,
    },
    /// A blank command directive.
    Reset {
        /// Native key that supplied the blank directive.
        kind: NativeCommandKind,
        /// Primary physical source-value segment span.
        span: SourceSpan,
    },
    /// The command contains a systemd specifier requiring target context.
    Deferred {
        /// Native key that needs target context.
        kind: NativeCommandKind,
        /// Primary physical source-value segment span.
        span: SourceSpan,
    },
    /// The command syntax is malformed.
    Unmodeled {
        /// Native key that supplied malformed syntax.
        kind: NativeCommandKind,
        /// Primary physical source-value segment span.
        span: SourceSpan,
    },
}

/// Ordered container commands plus recoverable native diagnostics.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeContainerCommands {
    directives: Vec<NativeCommandDirective>,
    diagnostics: Vec<Diagnostic>,
}

impl NativeContainerCommands {
    pub(super) fn from_document(document: &QuadletDocument) -> Self {
        let mut directives = Vec::new();
        let mut diagnostics = Vec::new();
        for entry in document.entries().filter(|entry| {
            matches!(
                entry.kind(),
                EntryKind::Container(ContainerKey::Exec | ContainerKey::Entrypoint)
            )
        }) {
            let span = entry.value().primary().span();
            let kind = if matches!(entry.kind(), EntryKind::Container(ContainerKey::Exec)) {
                NativeCommandKind::Exec
            } else {
                NativeCommandKind::Entrypoint
            };
            match logical_authored_value(entry) {
                None => {
                    diagnostics.push(diagnostic(
                        MALFORMED_COMMAND,
                        span,
                        "native command has incomplete continuation syntax",
                        "complete the physical continuation before interpreting the command",
                    ));
                    directives.push(NativeCommandDirective::Unmodeled { kind, span });
                }
                Some(value) if value.is_empty() => directives.push(NativeCommandDirective::Reset { kind, span }),
                Some(value) if literal_specifier_value(&value).is_none() => {
                    diagnostics.push(diagnostic(
                        DEFERRED_COMMAND,
                        span,
                        "native command contains a systemd specifier",
                        "expand the specifier in the target manager before relying on command arguments",
                    ));
                    directives.push(NativeCommandDirective::Deferred { kind, span });
                }
                Some(value) => {
                    let value = literal_specifier_value(&value).unwrap_or_default();
                    let parsed = match kind {
                        NativeCommandKind::Exec => systemd_environment_tokens(&value)
                            .filter(|arguments| !arguments.is_empty())
                            .map(|arguments| (arguments, NativeCommandSyntax::ExecWords)),
                        NativeCommandKind::Entrypoint => {
                            if let Ok(arguments) = serde_json::from_str::<Vec<String>>(&value) {
                                Some((arguments, NativeCommandSyntax::EntrypointJsonArray))
                            } else {
                                Some((vec![value], NativeCommandSyntax::EntrypointLiteral))
                            }
                        }
                    };
                    if let Some((arguments, syntax)) = parsed {
                        directives.push(NativeCommandDirective::Command {
                            kind,
                            command: NativeCommand {
                                arguments,
                                syntax,
                                span,
                            },
                        });
                    } else {
                        diagnostics.push(diagnostic(
                            MALFORMED_COMMAND,
                            span,
                            "native Exec command has malformed or unsupported lexical quoting or escaping",
                            "use a non-empty Exec= command within the documented lexical word grammar",
                        ));
                        directives.push(NativeCommandDirective::Unmodeled { kind, span });
                    }
                }
            }
        }
        Self {
            directives,
            diagnostics,
        }
    }
    /// Returns directives in authored order.
    #[must_use]
    pub fn directives(&self) -> &[NativeCommandDirective] {
        &self.directives
    }
    /// Returns recoverable malformed/deferred diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
}

#[derive(Clone, Copy)]
enum NativeKey {
    Port,
    Mount,
}
fn entries(document: &QuadletDocument, section: SectionKind, wanted: NativeKey) -> impl Iterator<Item = &TypedEntry> {
    document
        .sections()
        .iter()
        .filter(move |candidate| candidate.kind() == section)
        .flat_map(move |candidate| candidate.entries().iter())
        .filter(move |entry| {
            matches!(
                (section, wanted, entry.kind()),
                (
                    SectionKind::Container,
                    NativeKey::Port,
                    EntryKind::Container(ContainerKey::PublishPort)
                ) | (SectionKind::Pod, NativeKey::Port, EntryKind::Pod(PodKey::PublishPort))
                    | (
                        SectionKind::Container,
                        NativeKey::Mount,
                        EntryKind::Container(ContainerKey::Volume | ContainerKey::Mount),
                    )
                    | (SectionKind::Pod, NativeKey::Mount, EntryKind::Pod(PodKey::Volume))
            )
        })
}

fn native_single_value(entry: &TypedEntry) -> Result<String, ()> {
    let value = logical_authored_value(entry).ok_or(())?;
    let tokens = systemd_environment_tokens(&value).ok_or(())?;
    if tokens.is_empty() {
        Ok(String::new())
    } else if tokens.len() == 1 {
        Ok(tokens.into_iter().next().unwrap_or_default())
    } else {
        Err(())
    }
}

/// Decodes the context-independent systemd literal-percent escape.
///
/// Other specifiers require a manager context and are deliberately not interpreted by this
/// source-only model.
fn literal_specifier_value(value: &str) -> Option<String> {
    let mut decoded = String::with_capacity(value.len());
    let mut characters = value.chars();
    while let Some(character) = characters.next() {
        if character != '%' {
            decoded.push(character);
            continue;
        }
        match characters.next() {
            Some('%') => decoded.push('%'),
            _ => return None,
        }
    }
    Some(decoded)
}

fn parse_port(value: &str, span: SourceSpan) -> Result<NativePortSpecification, &'static str> {
    let (addressing, protocol) = value
        .rsplit_once('/')
        .map_or((value, "tcp"), |(addressing, protocol)| (addressing, protocol));
    let protocol = match protocol {
        "tcp" => NativePortProtocol::Tcp,
        "udp" => NativePortProtocol::Udp,
        "sctp" => NativePortProtocol::Sctp,
        _ => return Err("native PublishPort protocol must be tcp, udp, or sctp"),
    };
    let (host, published, container) = if let Some(rest) = addressing.strip_prefix('[') {
        let (host, rest) = rest
            .split_once(']')
            .ok_or("bracketed IPv6 host address is missing closing bracket")?;
        let rest = rest
            .strip_prefix(':')
            .ok_or("bracketed IPv6 host address must be followed by publication ports")?;
        let (published, container) = rest
            .split_once(':')
            .ok_or("bracketed IPv6 host address needs host and container ports")?;
        (
            Some(format!("[{host}]")),
            parse_optional_range(published)?,
            parse_range(container)?,
        )
    } else {
        let parts: Vec<_> = addressing.split(':').collect();
        match parts.as_slice() {
            [container] => (None, None, parse_range(container)?),
            [published, container] => (None, parse_optional_range(published)?, parse_range(container)?),
            [host, published, container] if !host.is_empty() => (
                Some((*host).to_owned()),
                parse_optional_range(published)?,
                parse_range(container)?,
            ),
            _ => return Err("native PublishPort address form is not supported; bracket IPv6 addresses"),
        }
    };
    if published.is_some_and(|published| published.len() != container.len()) {
        return Err("native PublishPort host and container ranges must have equal length");
    }
    Ok(NativePortSpecification {
        host,
        published,
        container,
        protocol,
        span,
    })
}

fn parse_optional_range(value: &str) -> Result<Option<NativePortRange>, &'static str> {
    if value.is_empty() {
        Ok(None)
    } else {
        parse_range(value).map(Some)
    }
}
fn parse_range(value: &str) -> Result<NativePortRange, &'static str> {
    let (start, end) = value
        .split_once('-')
        .map_or((value, value), |(start, end)| (start, end));
    let start = start
        .parse::<u16>()
        .ok()
        .filter(|value| *value != 0)
        .ok_or("native port must be a non-zero 16-bit integer")?;
    let end = end
        .parse::<u16>()
        .ok()
        .filter(|value| *value != 0)
        .ok_or("native port range end must be a non-zero 16-bit integer")?;
    (start <= end)
        .then_some(NativePortRange { start, end })
        .ok_or("native port range end precedes its start")
}

fn parse_short_mount(value: &str, span: SourceSpan) -> Result<NativeMountSpecification, &'static str> {
    let parts: Vec<_> = value.split(':').collect();
    let (source, destination, option_values) = match parts.as_slice() {
        [destination] => (None, *destination, Vec::new()),
        [source, destination] => (Some((*source).to_owned()), *destination, Vec::new()),
        [source, destination, options] => (Some((*source).to_owned()), *destination, options.split(',').collect()),
        _ => return Err("native Volume value has too many colon-separated fields"),
    };
    if destination.is_empty() {
        return Err("native Volume destination is empty");
    }
    let options = option_values
        .into_iter()
        .map(|option| NativeMountOption::Flag(option.to_owned()))
        .collect();
    Ok(NativeMountSpecification {
        long_form: false,
        source,
        destination: Some(destination.to_owned()),
        options,
        span,
    })
}

fn parse_long_mount(value: &str, span: SourceSpan) -> Result<NativeMountSpecification, &'static str> {
    let mut source = None;
    let mut destination = None;
    let mut type_seen = false;
    let mut options = Vec::new();
    for option in value.split(',') {
        if option.is_empty() {
            return Err("native Mount contains an empty comma-separated option");
        }
        if let Some((name, option_value)) = option.split_once('=') {
            if name.is_empty() || option_value.is_empty() {
                return Err("native Mount option must have non-empty name and value");
            }
            if name == "type" {
                type_seen = true;
            }
            if matches!(name, "source" | "src") {
                source = Some(option_value.to_owned());
            }
            if matches!(name, "destination" | "dst" | "target") {
                destination = Some(option_value.to_owned());
            }
            options.push(NativeMountOption::Assignment {
                name: name.to_owned(),
                value: option_value.to_owned(),
            });
        } else {
            options.push(NativeMountOption::Flag(option.to_owned()));
        }
    }
    if !type_seen {
        return Err("native long Mount requires type=TYPE");
    }
    if destination.is_none() {
        return Err("native long Mount requires destination=, dst=, or target=");
    }
    Ok(NativeMountSpecification {
        long_form: true,
        source,
        destination,
        options,
        span,
    })
}

fn diagnostic(code: DiagnosticCode, span: SourceSpan, summary: &'static str, help: &'static str) -> Diagnostic {
    Diagnostic::new(code, Severity::Warning, summary, Label::new(span, help))
}
