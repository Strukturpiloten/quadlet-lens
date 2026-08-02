//! Source identities, byte spans, and line/column lookup.

/// Stable caller-selected identity for one source document.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceId(u32);

impl SourceId {
    /// Creates a source identity from its caller-owned numeric value.
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the caller-owned numeric value.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// Half-open UTF-8 byte range within one source document.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SourceSpan {
    source_id: SourceId,
    start: usize,
    end: usize,
}

impl SourceSpan {
    pub(crate) const fn new(source_id: SourceId, start: usize, end: usize) -> Self {
        Self { source_id, start, end }
    }

    /// Returns the source containing this span.
    #[must_use]
    pub const fn source_id(self) -> SourceId {
        self.source_id
    }

    /// Returns the inclusive starting byte offset.
    #[must_use]
    pub const fn start(self) -> usize {
        self.start
    }

    /// Returns the exclusive ending byte offset.
    #[must_use]
    pub const fn end(self) -> usize {
        self.end
    }

    /// Returns the length of the byte range.
    #[must_use]
    pub const fn len(self) -> usize {
        self.end - self.start
    }

    /// Returns whether the range contains no bytes.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }
}

/// One-based line and Unicode-scalar column within a source document.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LineColumn {
    line: usize,
    column: usize,
}

impl LineColumn {
    /// Returns the one-based line number.
    #[must_use]
    pub const fn line(self) -> usize {
        self.line
    }

    /// Returns the one-based Unicode-scalar column number.
    #[must_use]
    pub const fn column(self) -> usize {
        self.column
    }
}

/// Immutable UTF-8 source text with precomputed line starts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceText {
    id: SourceId,
    text: String,
    line_starts: Vec<usize>,
}

impl SourceText {
    pub(crate) fn new(id: SourceId, text: String) -> Self {
        let mut line_starts = vec![0];
        line_starts.extend(text.match_indices('\n').map(|(offset, _)| offset + 1));
        Self { id, text, line_starts }
    }

    /// Returns the source identity.
    #[must_use]
    pub const fn id(&self) -> SourceId {
        self.id
    }

    /// Returns the complete authored text.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the text selected by a matching, valid UTF-8 span.
    #[must_use]
    pub fn slice(&self, span: SourceSpan) -> Option<&str> {
        if span.source_id != self.id || span.start > span.end {
            return None;
        }
        self.text.get(span.start..span.end)
    }

    /// Resolves a valid UTF-8 byte offset to a one-based line and column.
    #[must_use]
    pub fn location(&self, offset: usize) -> Option<LineColumn> {
        if offset > self.text.len() || !self.text.is_char_boundary(offset) {
            return None;
        }

        let following = self.line_starts.partition_point(|start| *start <= offset);
        let line_index = following.saturating_sub(1);
        let line_start = *self.line_starts.get(line_index)?;
        let column = self.text.get(line_start..offset)?.chars().count() + 1;
        Some(LineColumn {
            line: line_index + 1,
            column,
        })
    }
}
