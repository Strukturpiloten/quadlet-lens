# QuadletLens model

QuadletLens separates source syntax, typed document meaning, relationships, and target compatibility. Parsing one file does not silently inspect other files or the machine where the application runs.

| Stage             | Main type                     | Use it for                                                      |
| ----------------- | ----------------------------- | --------------------------------------------------------------- |
| Parsed file       | `model::QuadletDocument`      | Read typed keys while retaining source syntax.                  |
| Named file        | `model::NamedQuadletDocument` | Associate a validated filename with a document.                 |
| File set          | `model::QuadletDocumentSet`   | Validate names and build cross-file relationships.              |
| Capability target | `capability::PodmanTarget`    | Select an explicit minimum and optional maximum Podman version. |

The model covers `.container`, `.volume`, `.network`, `.pod`, `.image`, `.build`, `.kube`, and `.artifact` units plus the supported systemd sections. Unknown or malformed entries remain available as syntax evidence where recovery is possible.

## Direct use

```rust
use quadlet_lens::{model::{QuadletDocument, QuadletUnitType}, source::SourceId};

let source = "[Container]\nImage=example.invalid/web:1\n";
let parsed = QuadletDocument::parse(
    QuadletUnitType::Container,
    SourceId::new(1),
    source,
).expect("valid Quadlet input");
assert!(parsed.is_valid());
```

Parsing and model inspection are in-memory operations with no runtime side effects.
