//! Ordered source syntax, preservation, spans, and recovery behavior.

use quadlet_lens::diagnostic::Severity;
use quadlet_lens::source::{SourceId, SourceSpan};
use quadlet_lens::syntax::{CanonicalRenderError, CommentMarker, LineEnding, SyntaxDocument, SyntaxLineKind};

const ORDERED: &str = include_str!("../fixtures/syntax/ordered-container/example.container");
const MALFORMED: &str = include_str!("../fixtures/syntax/malformed-lines/broken.container");

#[test]
fn preserves_order_repeated_keys_continuations_and_specifiers() -> Result<(), String> {
    let result = SyntaxDocument::parse(SourceId::new(7), ORDERED);
    if !result.is_valid() {
        return Err(format!("unexpected diagnostics: {:#?}", result.diagnostics()));
    }

    let document = result.document();
    assert_eq!(document.render_preserved(), ORDERED);

    let mut sections = Vec::new();
    let mut keys = Vec::new();
    let mut found_specifier = false;
    let mut found_continuation_comment = false;
    let mut continuation_lines = 0;

    for line in document.lines() {
        assert_eq!(line.ending(), LineEnding::Lf);
        match line.kind() {
            SyntaxLineKind::Section(section) => {
                sections.push(required_slice(document, section.name())?.to_owned());
            }
            SyntaxLineKind::Entry(entry) => {
                let key = required_slice(document, entry.key())?;
                let value = required_slice(document, entry.value())?;
                keys.push(key.to_owned());
                found_specifier |= value.contains("%h");
            }
            SyntaxLineKind::Continuation(continuation) => {
                continuation_lines += 1;
                assert_eq!(required_slice(document, continuation.value())?, "--label second=value");
                assert!(!continuation.continues());
            }
            SyntaxLineKind::Comment(comment) => {
                if comment.within_continuation() {
                    found_continuation_comment = true;
                    assert_eq!(comment.marker(), CommentMarker::Hash);
                }
            }
            SyntaxLineKind::Blank | SyntaxLineKind::Invalid => {}
        }
    }

    assert_eq!(sections, ["Unit", "Container", "Install"]);
    assert_eq!(keys.iter().filter(|key| key.as_str() == "After").count(), 2);
    assert!(found_specifier);
    assert!(found_continuation_comment);
    assert_eq!(continuation_lines, 1);

    let unicode_offset = ORDERED
        .find('ü')
        .ok_or_else(|| "fixture must contain a Unicode scalar".to_owned())?;
    let location = document
        .source()
        .location(unicode_offset)
        .ok_or_else(|| "Unicode offset must resolve".to_owned())?;
    assert_eq!(location.line(), 3);
    assert_eq!(location.column(), 15);
    Ok(())
}

#[test]
fn malformed_input_retains_source_and_stable_diagnostics() {
    let result = SyntaxDocument::parse(SourceId::new(11), MALFORMED);
    assert!(!result.is_valid());
    assert_eq!(result.document().render_preserved(), MALFORMED);

    let codes: Vec<_> = result
        .diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.code().as_str())
        .collect();
    assert_eq!(
        codes,
        ["QLS0003", "QLS0002", "QLS0001", "QLS0001", "QLS0004", "QLS0005"]
    );
    assert!(
        result
            .diagnostics()
            .iter()
            .all(|diagnostic| diagnostic.severity() == Severity::Error)
    );
    assert!(
        result
            .diagnostics()
            .iter()
            .flat_map(quadlet_lens::diagnostic::Diagnostic::labels)
            .all(|label| result.document().source().slice(label.span()).is_some())
    );
}

#[test]
fn preserves_crlf_and_missing_final_line_ending() -> Result<(), String> {
    let source = "[Container]  \r\nImage=example.invalid/image\r\nContainerName=example";
    let result = SyntaxDocument::parse(SourceId::new(13), source);
    assert!(result.is_valid());
    assert_eq!(result.document().render_preserved(), source);

    let endings: Vec<_> = result
        .document()
        .lines()
        .iter()
        .map(quadlet_lens::syntax::SyntaxLine::ending)
        .collect();
    assert_eq!(endings, [LineEnding::CrLf, LineEnding::CrLf, LineEnding::None]);

    let section = match result.document().lines()[0].kind() {
        SyntaxLineKind::Section(section) => section,
        other => return Err(format!("expected a section, got {other:?}")),
    };
    assert_eq!(required_slice(result.document(), section.name()), Ok("Container"));
    Ok(())
}

#[test]
fn canonical_rendering_normalizes_structure_without_collapsing_semantics() -> Result<(), String> {
    let source = include_str!("../fixtures/roundtrip/canonical-container/authored.container");
    let expected = include_str!("../fixtures/roundtrip/canonical-container/canonical.container");
    let parsed = SyntaxDocument::parse(SourceId::new(17), source);
    let canonical = parsed.render_canonical().map_err(|error| error.to_string())?;
    assert_eq!(canonical, expected);

    let reparsed = SyntaxDocument::parse(SourceId::new(18), canonical);
    assert!(reparsed.is_valid());
    assert_eq!(
        reparsed.render_canonical().map_err(|error| error.to_string())?,
        expected
    );
    Ok(())
}

#[test]
fn canonical_rendering_refuses_recoverable_invalid_input() {
    let parsed = SyntaxDocument::parse(SourceId::new(19), MALFORMED);
    let error = parsed.render_canonical();
    assert!(matches!(error, Err(CanonicalRenderError::InvalidSyntax(_))));
    assert_eq!(
        error.err().map(|error| error.diagnostics().len()),
        Some(parsed.diagnostics().len())
    );
}

#[test]
fn bounded_generated_corpus_is_preserved_and_canonical_is_idempotent() -> Result<(), String> {
    let endings = ["\n", "\r\n"];
    let markers = ['#', ';'];
    let values = [
        "\"quoted command with spaces\"",
        "%h/data:/var/lib/example:Z",
        "example.invalid/app:1.0@sha256:abcd",
    ];

    for ending in endings {
        for marker in markers {
            for value in values {
                let source = format!(
                    " {marker} generated{ending} [Container]{ending} Image = {value}{ending}PodmanArgs=--first \\{ending}{marker} continuation comment{ending}  --second{ending}"
                );
                let parsed = SyntaxDocument::parse(SourceId::new(23), source.clone());
                assert_eq!(parsed.document().render_preserved(), source);
                assert!(parsed.is_valid(), "{:#?}", parsed.diagnostics());

                let canonical = parsed.render_canonical().map_err(|error| error.to_string())?;
                let reparsed = SyntaxDocument::parse(SourceId::new(24), canonical.clone());
                assert!(reparsed.is_valid(), "{:#?}", reparsed.diagnostics());
                assert_eq!(
                    reparsed.render_canonical().map_err(|error| error.to_string())?,
                    canonical
                );
            }
        }
    }
    Ok(())
}

fn required_slice(document: &SyntaxDocument, span: SourceSpan) -> Result<&str, String> {
    document
        .source()
        .slice(span)
        .ok_or_else(|| format!("invalid source span: {span:?}"))
}
