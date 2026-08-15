//! Named document sets and exact native unit-reference resolution.

use std::collections::{BTreeMap, BTreeSet};
use std::{error::Error, fmt};

use crate::diagnostic::{Diagnostic, DiagnosticCode, Label, Severity};
use crate::source::{SourceId, SourceSpan};

use super::{
    EntryKind, QuadletDocument, QuadletUnitType, SystemdUnitKey, TypedEntry, UnitReferenceKind, ValueKind,
    reference_by_suffix,
};

const MISSING_REFERENCE: DiagnosticCode = DiagnosticCode::new("QLG0001");
const AMBIGUOUS_REFERENCE: DiagnosticCode = DiagnosticCode::new("QLG0002");
const DUPLICATE_UNIT_NAME: DiagnosticCode = DiagnosticCode::new("QLG0003");

/// Validated basename of one supported Quadlet unit file.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UnitFileName {
    value: String,
    unit_type: QuadletUnitType,
}

impl UnitFileName {
    /// Validates a basename and infers its supported Quadlet unit type.
    ///
    /// # Errors
    ///
    /// Returns [`DocumentSetError::InvalidUnitFileName`] for an empty name, a path, or a missing
    /// stem/extension. Returns [`DocumentSetError::UnsupportedUnitFileExtension`] when the suffix
    /// is not part of the current typed model.
    pub fn new(value: impl Into<String>) -> Result<Self, DocumentSetError> {
        let value = value.into();
        if value.is_empty() || value.contains('/') || value.contains('\\') {
            return Err(DocumentSetError::InvalidUnitFileName(value));
        }
        let Some((stem, extension)) = value.rsplit_once('.') else {
            return Err(DocumentSetError::InvalidUnitFileName(value));
        };
        if stem.is_empty() || extension.is_empty() {
            return Err(DocumentSetError::InvalidUnitFileName(value));
        }
        let unit_type = QuadletUnitType::from_extension(extension)
            .ok_or_else(|| DocumentSetError::UnsupportedUnitFileExtension(value.clone()))?;
        Ok(Self { value, unit_type })
    }

    /// Returns the exact validated basename.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }

    /// Returns the unit type implied by the filename suffix.
    #[must_use]
    pub const fn unit_type(&self) -> QuadletUnitType {
        self.unit_type
    }
}

impl fmt::Display for UnitFileName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.value)
    }
}

/// One typed document paired with its unit-file basename.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamedQuadletDocument {
    name: UnitFileName,
    document: QuadletDocument,
}

impl NamedQuadletDocument {
    /// Validates that a filename and typed document describe the same unit type.
    ///
    /// # Errors
    ///
    /// Returns a [`DocumentSetError`] for invalid names, unsupported extensions, or a suffix that
    /// does not match the document's selected unit type.
    pub fn new(name: impl Into<String>, document: QuadletDocument) -> Result<Self, DocumentSetError> {
        let name = UnitFileName::new(name)?;
        if name.unit_type() != document.unit_type() {
            return Err(DocumentSetError::UnitTypeMismatch {
                name: name.as_str().to_owned(),
                filename_type: name.unit_type(),
                document_type: document.unit_type(),
            });
        }
        Ok(Self { name, document })
    }

    /// Returns the validated unit-file basename.
    #[must_use]
    pub const fn name(&self) -> &UnitFileName {
        &self.name
    }

    /// Returns the source-aware typed document.
    #[must_use]
    pub const fn document(&self) -> &QuadletDocument {
        &self.document
    }

    /// Decomposes the named document.
    #[must_use]
    pub fn into_parts(self) -> (UnitFileName, QuadletDocument) {
        (self.name, self.document)
    }
}

/// Resolution state of one authored native unit reference.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ReferenceResolution {
    /// Exactly one document owns the referenced basename.
    Resolved {
        /// Index into [`QuadletDocumentSet::documents`].
        document_index: usize,
    },
    /// No document in the set owns the referenced basename.
    Missing,
    /// More than one document owns the referenced basename.
    Ambiguous {
        /// Number of candidate documents.
        candidates: usize,
    },
}

/// One authored reference and its exact document-set resolution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnitReference {
    source_document: usize,
    target_name: String,
    kind: UnitReferenceKind,
    systemd_unit_key: Option<SystemdUnitKey>,
    span: SourceSpan,
    resolution: ReferenceResolution,
}

impl UnitReference {
    /// Returns the index of the document containing the reference.
    #[must_use]
    pub const fn source_document(&self) -> usize {
        self.source_document
    }

    /// Returns the exact referenced unit-file basename.
    #[must_use]
    pub fn target_name(&self) -> &str {
        &self.target_name
    }

    /// Returns the native reference kind inferred from the authored value.
    #[must_use]
    pub const fn kind(&self) -> UnitReferenceKind {
        self.kind
    }

    /// Returns the `[Unit]` relationship that authored this reference.
    ///
    /// Native-section references return `None` because their relationship is implied by their
    /// native Quadlet key rather than a generic systemd directive.
    #[must_use]
    pub const fn systemd_unit_key(&self) -> Option<SystemdUnitKey> {
        self.systemd_unit_key
    }

    /// Returns the authored value span containing the reference.
    #[must_use]
    pub const fn span(&self) -> SourceSpan {
        self.span
    }

    /// Returns whether the reference resolved exactly, was missing, or was ambiguous.
    #[must_use]
    pub const fn resolution(&self) -> ReferenceResolution {
        self.resolution
    }
}

/// One resolved dependency edge between two documents.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DependencyEdge {
    source_document: usize,
    target_document: usize,
    kind: UnitReferenceKind,
    systemd_unit_key: Option<SystemdUnitKey>,
    span: SourceSpan,
}

impl DependencyEdge {
    /// Returns the referencing document index.
    #[must_use]
    pub const fn source_document(self) -> usize {
        self.source_document
    }

    /// Returns the referenced document index.
    #[must_use]
    pub const fn target_document(self) -> usize {
        self.target_document
    }

    /// Returns the native relationship kind.
    #[must_use]
    pub const fn kind(self) -> UnitReferenceKind {
        self.kind
    }

    /// Returns the `[Unit]` relationship that authored this edge.
    #[must_use]
    pub const fn systemd_unit_key(self) -> Option<SystemdUnitKey> {
        self.systemd_unit_key
    }

    /// Returns the source span that created this edge.
    #[must_use]
    pub const fn span(self) -> SourceSpan {
        self.span
    }
}

/// Exact reference inventory and resolved dependency edges for a document set.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DependencyGraph {
    references: Vec<UnitReference>,
    edges: Vec<DependencyEdge>,
}

impl DependencyGraph {
    /// Returns every native reference, including missing and ambiguous references.
    #[must_use]
    pub fn references(&self) -> &[UnitReference] {
        &self.references
    }

    /// Returns only references that resolve to exactly one document.
    #[must_use]
    pub fn edges(&self) -> &[DependencyEdge] {
        &self.edges
    }

    /// Returns whether every reference resolves to exactly one document.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.references
            .iter()
            .all(|reference| matches!(reference.resolution, ReferenceResolution::Resolved { .. }))
    }
}

/// Named Quadlet documents plus their exact native dependency graph.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuadletDocumentSet {
    documents: Vec<NamedQuadletDocument>,
    graph: DependencyGraph,
    diagnostics: Vec<Diagnostic>,
}

impl QuadletDocumentSet {
    /// Builds an exact-name index and resolves every native unit reference.
    ///
    /// Duplicate unit basenames remain in the set so references can be reported as ambiguous.
    ///
    /// # Errors
    ///
    /// Returns [`DocumentSetError::DuplicateSourceId`] because source identities must be unique for
    /// unambiguous source-labelled diagnostics.
    pub fn new(documents: impl IntoIterator<Item = NamedQuadletDocument>) -> Result<Self, DocumentSetError> {
        let documents: Vec<_> = documents.into_iter().collect();
        ensure_unique_source_ids(&documents)?;

        let mut by_name: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        for (index, document) in documents.iter().enumerate() {
            by_name
                .entry(document.name().as_str().to_owned())
                .or_default()
                .push(index);
        }

        let mut diagnostics = duplicate_name_diagnostics(&documents, &by_name);
        let mut references = Vec::new();
        let mut edges = Vec::new();
        for (source_document, named_document) in documents.iter().enumerate() {
            for entry in named_document.document().entries() {
                let ValueKind::UnitReference(kind) = entry.value_kind() else {
                    continue;
                };
                let Some(target_name) = entry.unit_reference_name() else {
                    continue;
                };
                resolve_reference(
                    source_document,
                    target_name.to_owned(),
                    kind,
                    None,
                    entry.value().primary().span(),
                    &by_name,
                    &mut diagnostics,
                    &mut references,
                    &mut edges,
                );
            }

            for reference in effective_systemd_unit_references(named_document.document()) {
                resolve_reference(
                    source_document,
                    reference.target_name,
                    reference.kind,
                    Some(reference.key),
                    reference.span,
                    &by_name,
                    &mut diagnostics,
                    &mut references,
                    &mut edges,
                );
            }
        }

        Ok(Self {
            documents,
            graph: DependencyGraph { references, edges },
            diagnostics,
        })
    }

    /// Returns named documents in caller-provided order.
    #[must_use]
    pub fn documents(&self) -> &[NamedQuadletDocument] {
        &self.documents
    }

    /// Returns the native reference inventory and resolved edges.
    #[must_use]
    pub const fn graph(&self) -> &DependencyGraph {
        &self.graph
    }

    /// Returns duplicate-name and reference-resolution diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Returns whether every unit filename is unique and every reference resolves exactly once.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.graph.is_complete()
            && self
                .diagnostics
                .iter()
                .all(|diagnostic| diagnostic.severity() != Severity::Error)
    }

    /// Returns the uniquely named document, or `None` when the name is missing or ambiguous.
    #[must_use]
    pub fn document(&self, name: &str) -> Option<&NamedQuadletDocument> {
        let mut matching = self
            .documents
            .iter()
            .filter(|document| document.name().as_str() == name);
        let first = matching.next()?;
        matching.next().is_none().then_some(first)
    }
}

#[derive(Debug)]
struct SystemdRelationshipReference {
    target_name: String,
    kind: UnitReferenceKind,
    key: SystemdUnitKey,
    span: SourceSpan,
}

fn effective_systemd_unit_references(document: &QuadletDocument) -> Vec<SystemdRelationshipReference> {
    let mut references: Vec<SystemdRelationshipReference> = Vec::new();
    for entry in document.entries() {
        let EntryKind::SystemdUnit(key) = entry.kind() else {
            continue;
        };
        let Some(value) = logical_entry_value(entry) else {
            continue;
        };
        if value.trim().is_empty() {
            references.retain(|reference| reference.key != key);
            continue;
        }
        let Some(tokens) = systemd_unit_tokens(&value) else {
            continue;
        };
        references.extend(tokens.into_iter().filter_map(|target_name| {
            let kind = reference_by_suffix(&target_name)?;
            Some(SystemdRelationshipReference {
                target_name,
                kind,
                key,
                span: entry.value().primary().span(),
            })
        }));
    }
    references
}

fn logical_entry_value(entry: &TypedEntry) -> Option<String> {
    let mut logical = String::new();
    let segments = std::iter::once(entry.value().primary())
        .chain(entry.value().continuations())
        .collect::<Vec<_>>();
    if entry.value().is_continued() && segments.last().is_none_or(|segment| segment.text().ends_with('\\')) {
        return None;
    }
    for (index, segment) in segments.iter().enumerate() {
        if !logical.is_empty() {
            logical.push(' ');
        }
        let value = segment.text().trim_end();
        let value = if index + 1 < segments.len() {
            value.strip_suffix('\\').unwrap_or(value)
        } else {
            value
        };
        logical.push_str(value);
    }
    Some(logical)
}

/// Splits the systemd unit-list subset used by relationship directives.
///
/// Quotes and backslash escapes group whitespace without changing the authored document. A
/// malformed unterminated quote or escape produces no graph claims for that physical entry.
fn systemd_unit_tokens(value: &str) -> Option<Vec<String>> {
    let mut tokens = Vec::new();
    let mut token = String::new();
    let mut quote = None;
    let mut escaped = false;
    let mut started = false;

    for character in value.chars() {
        if escaped {
            token.push(character);
            escaped = false;
            started = true;
        } else if character == '\\' {
            escaped = true;
            started = true;
        } else if matches!(character, '\'' | '"') {
            if quote == Some(character) {
                quote = None;
            } else if quote.is_none() {
                quote = Some(character);
            } else {
                token.push(character);
            }
            started = true;
        } else if character.is_whitespace() && quote.is_none() {
            if started {
                tokens.push(std::mem::take(&mut token));
                started = false;
            }
        } else {
            token.push(character);
            started = true;
        }
    }

    if escaped || quote.is_some() {
        return None;
    }
    if started {
        tokens.push(token);
    }
    Some(tokens)
}

#[allow(clippy::too_many_arguments)]
fn resolve_reference(
    source_document: usize,
    target_name: String,
    kind: UnitReferenceKind,
    systemd_unit_key: Option<SystemdUnitKey>,
    span: SourceSpan,
    by_name: &BTreeMap<String, Vec<usize>>,
    diagnostics: &mut Vec<Diagnostic>,
    references: &mut Vec<UnitReference>,
    edges: &mut Vec<DependencyEdge>,
) {
    let candidates = by_name.get(&target_name).map_or(&[][..], Vec::as_slice);
    let resolution = match candidates {
        [] => {
            diagnostics.push(Diagnostic::new(
                MISSING_REFERENCE,
                Severity::Error,
                "Quadlet unit reference has no matching document",
                Label::new(span, "add the referenced unit file to this document set"),
            ));
            ReferenceResolution::Missing
        }
        [target_document] => {
            edges.push(DependencyEdge {
                source_document,
                target_document: *target_document,
                kind,
                systemd_unit_key,
                span,
            });
            ReferenceResolution::Resolved {
                document_index: *target_document,
            }
        }
        multiple => {
            diagnostics.push(Diagnostic::new(
                AMBIGUOUS_REFERENCE,
                Severity::Error,
                "Quadlet unit reference matches multiple documents",
                Label::new(span, "make unit-file basenames unique in this document set"),
            ));
            ReferenceResolution::Ambiguous {
                candidates: multiple.len(),
            }
        }
    };
    references.push(UnitReference {
        source_document,
        target_name,
        kind,
        systemd_unit_key,
        span,
        resolution,
    });
}

/// Invalid filename/document metadata that prevents safe document-set construction.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum DocumentSetError {
    /// A unit name is empty, contains path separators, or lacks a complete suffix.
    InvalidUnitFileName(String),
    /// A unit suffix is outside the current typed-model surface.
    UnsupportedUnitFileExtension(String),
    /// The filename suffix and selected typed document kind differ.
    UnitTypeMismatch {
        /// Authored unit-file basename.
        name: String,
        /// Unit type inferred from the filename.
        filename_type: QuadletUnitType,
        /// Unit type selected while parsing the document.
        document_type: QuadletUnitType,
    },
    /// Two documents use the same caller-owned source identity.
    DuplicateSourceId(SourceId),
}

impl fmt::Display for DocumentSetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUnitFileName(name) => write!(formatter, "invalid Quadlet unit-file basename `{name}`"),
            Self::UnsupportedUnitFileExtension(name) => {
                write!(formatter, "unsupported Quadlet unit-file extension in `{name}`")
            }
            Self::UnitTypeMismatch {
                name,
                filename_type,
                document_type,
            } => write!(
                formatter,
                "Quadlet filename `{name}` implies {filename_type:?}, but the document is {document_type:?}"
            ),
            Self::DuplicateSourceId(source_id) => {
                write!(formatter, "duplicate Quadlet source identity {}", source_id.get())
            }
        }
    }
}

impl Error for DocumentSetError {}

fn ensure_unique_source_ids(documents: &[NamedQuadletDocument]) -> Result<(), DocumentSetError> {
    let mut source_ids = BTreeSet::new();
    for document in documents {
        let source_id = document.document().source_id();
        if !source_ids.insert(source_id) {
            return Err(DocumentSetError::DuplicateSourceId(source_id));
        }
    }
    Ok(())
}

fn duplicate_name_diagnostics(
    documents: &[NamedQuadletDocument],
    by_name: &BTreeMap<String, Vec<usize>>,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for indexes in by_name.values().filter(|indexes| indexes.len() > 1) {
        for index in indexes.iter().skip(1) {
            let document = &documents[*index];
            diagnostics.push(Diagnostic::new(
                DUPLICATE_UNIT_NAME,
                Severity::Error,
                "document set contains a duplicate Quadlet unit-file basename",
                Label::new(
                    document.document().source_span(),
                    "give this document a unique unit-file basename",
                ),
            ));
        }
    }
    diagnostics
}
