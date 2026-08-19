# Parse and render Quadlet

Call `model::QuadletDocument::parse` with the unit type, source identifier, and text. The result retains both the typed document and the syntax document used for source-aware diagnostics and preserved rendering.

## Choose an output mode

- `render_preserved` reproduces untouched source presentation.
- `render_canonical` writes deterministic presentation for a parsed document.
- `render::QuadletDocumentBuilder` constructs a new typed document and validates the generated text before success.

Canonical rendering does not check the development host, run the Podman generator, or select a Podman version. Evaluate capabilities separately when an output must target a particular version range.

## Generate a document

```rust
use quadlet_lens::{
    model::{ContainerKey, EntryValue, QuadletUnitType},
    render::QuadletDocumentBuilder,
    source::SourceId,
};

let mut builder = QuadletDocumentBuilder::new(QuadletUnitType::Container);
let image = EntryValue::new("example.invalid/web:1").expect("valid value");
builder.push_container(ContainerKey::Image, image).expect("valid key");
let generated = builder.build(SourceId::new(1)).expect("valid document");
assert!(generated.text().contains("Image=example.invalid/web:1"));
```
