//! Ordered, loss-aware physical-line representation and deterministic rendering.

use std::{error::Error, fmt};

use crate::diagnostic::{Diagnostic, DiagnosticCode, Label, Severity};
use crate::source::{SourceId, SourceSpan, SourceText};

const INVALID_LINE: DiagnosticCode = DiagnosticCode::new("QLS0001");
const EMPTY_SECTION: DiagnosticCode = DiagnosticCode::new("QLS0002");
const ENTRY_BEFORE_SECTION: DiagnosticCode = DiagnosticCode::new("QLS0003");
const EMPTY_KEY: DiagnosticCode = DiagnosticCode::new("QLS0004");
const DANGLING_CONTINUATION: DiagnosticCode = DiagnosticCode::new("QLS0005");

/// Line-ending spelling retained for one physical source line.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LineEnding {
    /// Line-feed (`\n`) ending.
    Lf,
    /// Carriage-return plus line-feed (`\r\n`) ending.
    CrLf,
    /// Final physical line without a line ending.
    None,
}

/// Supported systemd comment marker.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommentMarker {
    /// Hash (`#`) comment.
    Hash,
    /// Semicolon (`;`) comment.
    Semicolon,
}

/// Comment-line details.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CommentLine {
    marker: CommentMarker,
    body: SourceSpan,
    within_continuation: bool,
}

impl CommentLine {
    /// Returns the authored comment marker.
    #[must_use]
    pub const fn marker(self) -> CommentMarker {
        self.marker
    }

    /// Returns the bytes following the comment marker.
    #[must_use]
    pub const fn body(self) -> SourceSpan {
        self.body
    }

    /// Returns whether the comment occurred inside a continued logical line.
    #[must_use]
    pub const fn within_continuation(self) -> bool {
        self.within_continuation
    }
}

/// Section-header details.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SectionLine {
    name: SourceSpan,
}

impl SectionLine {
    /// Returns the section-name bytes without brackets.
    #[must_use]
    pub const fn name(self) -> SourceSpan {
        self.name
    }
}

/// Configuration-entry details.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EntryLine {
    key: SourceSpan,
    value: SourceSpan,
    continues: bool,
}

impl EntryLine {
    /// Returns the key after ignoring whitespace adjacent to `=`.
    #[must_use]
    pub const fn key(self) -> SourceSpan {
        self.key
    }

    /// Returns the authored value after ignored leading whitespace.
    #[must_use]
    pub const fn value(self) -> SourceSpan {
        self.value
    }

    /// Returns whether the authored physical line ends in a continuation backslash.
    #[must_use]
    pub const fn continues(self) -> bool {
        self.continues
    }
}

/// Non-comment physical line continuing the preceding entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ContinuationLine {
    value: SourceSpan,
    continues: bool,
}

impl ContinuationLine {
    /// Returns the continued value bytes after leading indentation.
    #[must_use]
    pub const fn value(self) -> SourceSpan {
        self.value
    }

    /// Returns whether this physical line continues again.
    #[must_use]
    pub const fn continues(self) -> bool {
        self.continues
    }
}

/// Classification of one authored physical line.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SyntaxLineKind {
    /// Empty or whitespace-only line.
    Blank,
    /// Comment retained with its marker and continuation context.
    Comment(CommentLine),
    /// Section header retained in authored order.
    Section(SectionLine),
    /// `key=value` entry; repeated keys remain separate entries.
    Entry(EntryLine),
    /// Physical continuation of a preceding entry.
    Continuation(ContinuationLine),
    /// Recoverable line that does not match the syntax kernel.
    Invalid,
}

/// One ordered physical source line.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SyntaxLine {
    content: SourceSpan,
    full: SourceSpan,
    ending: LineEnding,
    kind: SyntaxLineKind,
}

impl SyntaxLine {
    /// Returns the content span without the line ending.
    #[must_use]
    pub const fn content(&self) -> SourceSpan {
        self.content
    }

    /// Returns the complete physical-line span including its line ending.
    #[must_use]
    pub const fn full(&self) -> SourceSpan {
        self.full
    }

    /// Returns the authored line-ending spelling.
    #[must_use]
    pub const fn ending(&self) -> LineEnding {
        self.ending
    }

    /// Returns the syntax classification.
    #[must_use]
    pub const fn kind(&self) -> SyntaxLineKind {
        self.kind
    }
}

/// Recoverable result of parsing one source document.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseResult {
    document: SyntaxDocument,
    diagnostics: Vec<Diagnostic>,
}

impl ParseResult {
    /// Returns the loss-aware syntax document even when diagnostics exist.
    #[must_use]
    pub const fn document(&self) -> &SyntaxDocument {
        &self.document
    }

    /// Returns all structured syntax diagnostics in source order.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Returns whether parsing produced no error-severity diagnostics.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.diagnostics
            .iter()
            .all(|diagnostic| diagnostic.severity() != Severity::Error)
    }

    /// Decomposes the result into its document and diagnostics.
    #[must_use]
    pub fn into_parts(self) -> (SyntaxDocument, Vec<Diagnostic>) {
        (self.document, self.diagnostics)
    }

    /// Renders valid syntax in a deterministic structural form.
    ///
    /// Canonical rendering retains line order, repeated keys, comments, value
    /// bytes, continuation structure, and specifiers. It normalizes indentation,
    /// whitespace around assignments, and line endings, and always terminates a
    /// non-empty document with LF.
    ///
    /// # Errors
    ///
    /// Returns [`CanonicalRenderError::InvalidSyntax`] when parsing produced an
    /// error diagnostic, or [`CanonicalRenderError::InvalidSourceSpan`] if an
    /// internal source span cannot be resolved.
    pub fn render_canonical(&self) -> Result<String, CanonicalRenderError> {
        if !self.is_valid() {
            return Err(CanonicalRenderError::InvalidSyntax(self.diagnostics.clone()));
        }
        self.document.render_canonical_valid()
    }
}

/// Failure to render a canonical syntax document.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CanonicalRenderError {
    /// Parsing produced one or more error diagnostics.
    InvalidSyntax(Vec<Diagnostic>),
    /// A parser-owned source span could not be resolved.
    InvalidSourceSpan(SourceSpan),
}

impl CanonicalRenderError {
    /// Returns syntax diagnostics when invalid input blocked rendering.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        match self {
            Self::InvalidSyntax(diagnostics) => diagnostics,
            Self::InvalidSourceSpan(_) => &[],
        }
    }
}

impl fmt::Display for CanonicalRenderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSyntax(diagnostics) => write!(
                formatter,
                "canonical rendering requires valid syntax; found {} diagnostic(s)",
                diagnostics.len()
            ),
            Self::InvalidSourceSpan(span) => write!(formatter, "invalid parser source span: {span:?}"),
        }
    }
}

impl Error for CanonicalRenderError {}

/// Immutable ordered syntax document that preserves the complete authored source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SyntaxDocument {
    source: SourceText,
    lines: Vec<SyntaxLine>,
}

impl SyntaxDocument {
    /// Parses UTF-8 source into ordered physical lines and structured diagnostics.
    #[must_use]
    pub fn parse(source_id: SourceId, text: impl Into<String>) -> ParseResult {
        let source = SourceText::new(source_id, text.into());
        let (lines, diagnostics) = parse_lines(&source);
        ParseResult {
            document: Self { source, lines },
            diagnostics,
        }
    }

    /// Returns the complete source and its location lookup.
    #[must_use]
    pub const fn source(&self) -> &SourceText {
        &self.source
    }

    /// Returns physical lines in authored order.
    #[must_use]
    pub fn lines(&self) -> &[SyntaxLine] {
        &self.lines
    }

    /// Returns the authored source bytes unchanged.
    #[must_use]
    pub fn render_preserved(&self) -> &str {
        self.source.text()
    }

    fn render_canonical_valid(&self) -> Result<String, CanonicalRenderError> {
        let mut rendered = String::new();
        for line in &self.lines {
            match line.kind() {
                SyntaxLineKind::Blank => {}
                SyntaxLineKind::Comment(comment) => {
                    rendered.push(match comment.marker() {
                        CommentMarker::Hash => '#',
                        CommentMarker::Semicolon => ';',
                    });
                    rendered.push_str(self.required_slice(comment.body())?);
                }
                SyntaxLineKind::Section(section) => {
                    rendered.push('[');
                    rendered.push_str(self.required_slice(section.name())?);
                    rendered.push(']');
                }
                SyntaxLineKind::Entry(entry) => {
                    rendered.push_str(self.required_slice(entry.key())?);
                    rendered.push('=');
                    rendered.push_str(self.required_slice(entry.value())?);
                }
                SyntaxLineKind::Continuation(continuation) => {
                    rendered.push('\t');
                    rendered.push_str(self.required_slice(continuation.value())?);
                }
                SyntaxLineKind::Invalid => {
                    return Err(CanonicalRenderError::InvalidSourceSpan(line.content()));
                }
            }
            rendered.push('\n');
        }
        Ok(rendered)
    }

    fn required_slice(&self, span: SourceSpan) -> Result<&str, CanonicalRenderError> {
        self.source
            .slice(span)
            .ok_or(CanonicalRenderError::InvalidSourceSpan(span))
    }
}

#[derive(Clone, Copy)]
struct PhysicalLine {
    start: usize,
    content_end: usize,
    full_end: usize,
    ending: LineEnding,
}

fn parse_lines(source: &SourceText) -> (Vec<SyntaxLine>, Vec<Diagnostic>) {
    let mut lines = Vec::new();
    let mut diagnostics = Vec::new();
    let mut within_section = false;
    let mut continuing = false;
    let mut continuation_origin = None;

    for physical in physical_lines(source.text()) {
        let content = &source.text()[physical.start..physical.content_end];
        let trimmed_start = trim_start_offset(content);
        let trimmed = &content[trimmed_start..];
        let content_span = SourceSpan::new(source.id(), physical.start, physical.content_end);
        let full_span = SourceSpan::new(source.id(), physical.start, physical.full_end);

        let kind = if trimmed.is_empty() {
            SyntaxLineKind::Blank
        } else if let Some(marker) = comment_marker(trimmed) {
            let marker_offset = physical.start + trimmed_start;
            SyntaxLineKind::Comment(CommentLine {
                marker,
                body: SourceSpan::new(source.id(), marker_offset + 1, physical.content_end),
                within_continuation: continuing,
            })
        } else if continuing {
            let continues = content.ends_with('\\');
            let value_start = physical.start + trimmed_start;
            continuing = continues;
            SyntaxLineKind::Continuation(ContinuationLine {
                value: SourceSpan::new(source.id(), value_start, physical.content_end),
                continues,
            })
        } else if trimmed.starts_with('[') {
            parse_section(
                source.id(),
                physical,
                trimmed_start,
                trimmed,
                &mut within_section,
                &mut diagnostics,
            )
        } else if let Some(equals) = content.find('=') {
            let (entry, continues) =
                parse_entry(source.id(), physical, content, equals, within_section, &mut diagnostics);
            continuing = continues;
            if continues {
                continuation_origin = Some(content_span);
            }
            SyntaxLineKind::Entry(entry)
        } else {
            diagnostics.push(Diagnostic::new(
                INVALID_LINE,
                Severity::Error,
                "line is not a section, comment, or configuration entry",
                Label::new(content_span, "expected `[Section]` or `key=value`"),
            ));
            SyntaxLineKind::Invalid
        };

        if !continuing {
            continuation_origin = None;
        }
        lines.push(SyntaxLine {
            content: content_span,
            full: full_span,
            ending: physical.ending,
            kind,
        });
    }

    if continuing {
        if let Some(span) = continuation_origin {
            diagnostics.push(Diagnostic::new(
                DANGLING_CONTINUATION,
                Severity::Error,
                "continued configuration entry reaches the end of the file",
                Label::new(span, "expected another physical value line"),
            ));
        }
    }

    (lines, diagnostics)
}

fn parse_entry(
    source_id: SourceId,
    physical: PhysicalLine,
    content: &str,
    equals: usize,
    within_section: bool,
    diagnostics: &mut Vec<Diagnostic>,
) -> (EntryLine, bool) {
    let content_span = SourceSpan::new(source_id, physical.start, physical.content_end);
    let key_start_in_line = trim_start_offset(&content[..equals]);
    let key_end_in_line = trim_end_offset(&content[..equals]);
    let value_start_in_line = equals + 1 + trim_start_offset(&content[equals + 1..]);
    let continues = content.ends_with('\\');
    let key_span = SourceSpan::new(
        source_id,
        physical.start + key_start_in_line,
        physical.start + key_end_in_line,
    );

    if key_span.is_empty() {
        diagnostics.push(Diagnostic::new(
            EMPTY_KEY,
            Severity::Error,
            "configuration entry has an empty key",
            Label::new(content_span, "expected a key before `=`"),
        ));
    }
    if !within_section {
        diagnostics.push(Diagnostic::new(
            ENTRY_BEFORE_SECTION,
            Severity::Error,
            "configuration entry appears before any section",
            Label::new(content_span, "add a section header before this entry"),
        ));
    }

    (
        EntryLine {
            key: key_span,
            value: SourceSpan::new(source_id, physical.start + value_start_in_line, physical.content_end),
            continues,
        },
        continues,
    )
}

fn parse_section(
    source_id: SourceId,
    physical: PhysicalLine,
    trimmed_start: usize,
    trimmed: &str,
    within_section: &mut bool,
    diagnostics: &mut Vec<Diagnostic>,
) -> SyntaxLineKind {
    let content_span = SourceSpan::new(source_id, physical.start, physical.content_end);
    let trimmed_end = trim_end_offset(trimmed);
    let header = &trimmed[..trimmed_end];
    if !header.ends_with(']') {
        diagnostics.push(Diagnostic::new(
            INVALID_LINE,
            Severity::Error,
            "section header is malformed",
            Label::new(content_span, "expected a closing `]` at the end of the line"),
        ));
        return SyntaxLineKind::Invalid;
    }

    let name_start = physical.start + trimmed_start + 1;
    let name_end = physical.start + trimmed_start + header.len() - 1;
    let name = SourceSpan::new(source_id, name_start, name_end);
    if name.is_empty() {
        diagnostics.push(Diagnostic::new(
            EMPTY_SECTION,
            Severity::Error,
            "section header has an empty name",
            Label::new(content_span, "expected a name between `[` and `]`"),
        ));
    } else {
        *within_section = true;
    }
    SyntaxLineKind::Section(SectionLine { name })
}

fn physical_lines(text: &str) -> Vec<PhysicalLine> {
    let mut result = Vec::new();
    let mut start = 0;

    for (newline, _) in text.match_indices('\n') {
        let (content_end, ending) = if newline > start && text.as_bytes()[newline - 1] == b'\r' {
            (newline - 1, LineEnding::CrLf)
        } else {
            (newline, LineEnding::Lf)
        };
        result.push(PhysicalLine {
            start,
            content_end,
            full_end: newline + 1,
            ending,
        });
        start = newline + 1;
    }

    if start < text.len() {
        result.push(PhysicalLine {
            start,
            content_end: text.len(),
            full_end: text.len(),
            ending: LineEnding::None,
        });
    }

    result
}

fn trim_start_offset(value: &str) -> usize {
    value.len() - value.trim_start_matches([' ', '\t']).len()
}

fn trim_end_offset(value: &str) -> usize {
    value.trim_end_matches([' ', '\t']).len()
}

fn comment_marker(value: &str) -> Option<CommentMarker> {
    match value.as_bytes().first() {
        Some(b'#') => Some(CommentMarker::Hash),
        Some(b';') => Some(CommentMarker::Semicolon),
        _ => None,
    }
}
