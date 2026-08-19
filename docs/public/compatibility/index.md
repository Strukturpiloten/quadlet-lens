# Quadlet compatibility

Quadlet syntax changes across Podman releases and can also depend on systemd behavior. QuadletLens therefore evaluates capabilities against an explicit target instead of guessing from the development machine.

```rust
use quadlet_lens::capability::{
    CapabilityCatalogue, PodmanTarget, PodmanVersion, SupportClassification,
};

let catalogue = CapabilityCatalogue::supported_range().expect("valid embedded catalogue");
let target = PodmanTarget::new(
    PodmanVersion::new(5, 4, 0),
    Some(PodmanVersion::new(6, 1, 0)),
).expect("valid target range");
let support = catalogue.evaluate("quadlet.container.image", target);
assert_eq!(support.classification(), SupportClassification::Native);
```

`CapabilityCatalogue::supported_range` embeds the reviewed catalogue shipped with the crate. The same data is available as a [downloadable TOML catalogue](https://boxferry.dev/docs/libraries/quadlet-lens/catalogue/v1/podman-supported-range.toml) for tooling that does not link the Rust library.

## Read classifications conservatively

The catalogue distinguishes native support, argument fallbacks, deprecation, removal, unsupported behavior, and unknown evidence. A parsed key is not automatically supported by every target. Unknown means that the retained evidence cannot prove the requested claim.

Catalogue records carry sources and version boundaries. Ordinary tests validate the retained data offline; they do not run Podman, systemd, or a network request. The full evidence process is documented in the repository's [capability model](https://github.com/Strukturpiloten/quadlet-lens/blob/main/docs/capability-model.md).
