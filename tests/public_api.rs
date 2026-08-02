//! Consumer-facing compile and behavior contract for the supported 0.1.x API.

use quadlet_lens::capability::{CapabilityCatalogue, PodmanTarget, PodmanVersion, SupportClassification};
use quadlet_lens::model::{NamedQuadletDocument, QuadletDocument, QuadletDocumentSet, QuadletUnitType};
use quadlet_lens::path::{PathForm, classify_path};
use quadlet_lens::source::SourceId;

#[test]
fn supported_public_pipeline_compiles_and_keeps_stages_explicit() -> Result<(), Box<dyn std::error::Error>> {
    let source = "[Container]\nImage=example.invalid/app:1@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n";
    let parsed = QuadletDocument::parse(QuadletUnitType::Container, SourceId::new(1), source)?;

    assert!(parsed.is_valid());
    assert_eq!(parsed.syntax().document().render_preserved(), source);
    assert_eq!(parsed.syntax().render_canonical()?, source);

    let named = NamedQuadletDocument::new("app.container", parsed.document().clone())?;
    let documents = QuadletDocumentSet::new([named])?;
    assert!(documents.is_valid());
    assert!(documents.graph().is_complete());

    let catalogue = CapabilityCatalogue::supported_range()?;
    let target = PodmanTarget::new(PodmanVersion::new(5, 4, 0), Some(PodmanVersion::new(6, 0, 2)))?;
    let support = catalogue.evaluate("quadlet.container.image", target);
    assert_eq!(support.classification(), SupportClassification::Native);
    assert_eq!(classify_path("%h/application.env"), PathForm::SystemdSpecifier);

    Ok(())
}
