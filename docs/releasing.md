# Release policy

QuadletLens is not publishable yet: the root package deliberately has `publish = false`. Its
version is declared once in the workspace `Cargo.toml`, and all packages inherit the workspace
edition, MSRV, license, homepage, and repository metadata.

A crates.io release workflow will be introduced only after the public API, capability catalogue,
support policy, release notes, and package contents are ready. Until then, GitHub tags must not
imply a published crate. CI and the Dev Container can validate the code without publication
credentials.

When publication is enabled, follow the same security boundary as ComposeLens: a protected
`release` environment, a version derived from Cargo metadata, an immutable GitHub Actions
supply chain, crates.io trusted publishing after the first token-authenticated release, artifact
attestation, and an immutable GitHub release.
