//! Structured, source-aware syntax diagnostics.

use crate::source::SourceSpan;

/// Stable machine-readable diagnostic identifier.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DiagnosticCode(&'static str);

impl DiagnosticCode {
    /// Returns the stable code string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }

    pub(crate) const fn new(value: &'static str) -> Self {
        Self(value)
    }
}

/// Diagnostic severity independent from terminal presentation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Severity {
    /// Input cannot be interpreted with the documented syntax contract.
    Error,
    /// Input remains usable but has a suspicious or incomplete construct.
    Warning,
    /// Informational context that does not affect validity.
    Note,
}

/// One source range associated with a diagnostic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Label {
    span: SourceSpan,
    message: &'static str,
}

impl Label {
    pub(crate) const fn new(span: SourceSpan, message: &'static str) -> Self {
        Self { span, message }
    }

    /// Returns the labelled source range.
    #[must_use]
    pub const fn span(&self) -> SourceSpan {
        self.span
    }

    /// Returns a value-free explanation of the range.
    #[must_use]
    pub const fn message(&self) -> &'static str {
        self.message
    }
}

/// Structured diagnostic returned alongside a recoverable syntax document.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    code: DiagnosticCode,
    severity: Severity,
    summary: &'static str,
    labels: Vec<Label>,
}

impl Diagnostic {
    pub(crate) fn new(code: DiagnosticCode, severity: Severity, summary: &'static str, label: Label) -> Self {
        Self {
            code,
            severity,
            summary,
            labels: vec![label],
        }
    }

    /// Returns the stable machine-readable code.
    #[must_use]
    pub const fn code(&self) -> DiagnosticCode {
        self.code
    }

    /// Returns the diagnostic severity.
    #[must_use]
    pub const fn severity(&self) -> Severity {
        self.severity
    }

    /// Returns the value-free human-readable summary.
    #[must_use]
    pub const fn summary(&self) -> &'static str {
        self.summary
    }

    /// Returns all associated source labels.
    #[must_use]
    pub fn labels(&self) -> &[Label] {
        &self.labels
    }
}
