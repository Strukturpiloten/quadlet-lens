# Release process

QuadletLens releases are started manually through the protected GitHub Actions
[release workflow](../.github/workflows/release.yml). The workflow has no version input and
authenticates to crates.io only through trusted publishing.

## Version sources

- The package version is declared once in the workspace `Cargo.toml`.
- The MSRV is declared once as `rust-version` in the workspace `Cargo.toml`.
- The current development toolchain is declared in `rust-toolchain.toml`.
- The release workflow derives the package version with `cargo metadata`.

Do not add a second version file or type a version into a workflow form. Cargo's manifest is the
package-version source of truth.

## One-time GitHub setup

In the QuadletLens repository settings:

1. Create an environment named `release`.
2. Add Martin “Becks” Beckert as a required reviewer for that environment.
3. Restrict deployment branches to the default branch.
4. Keep the default workflow token permission read-only. The release job requests only its
   explicit write permissions.
5. Protect the default branch and the `v*` tag namespace with rulesets. Permit the release
   workflow to create release tags.
6. Enable immutable releases under the repository's release settings.

The GitHub environment stores no crates.io API token or repository secret.

## Completed first-release bootstrap and trusted publishing

[Crates.io's trusted-publishing guidance](https://crates.io/docs/trusted-publishing) requires a
crate's first version to be published by an authenticated owner before a trusted publisher can be
attached. This bootstrap was completed for `0.1.0` using the following process, retained here for
recovery and audit context:

1. Create a short-lived crates.io token restricted as narrowly as the new-crate bootstrap allows.
2. From the reviewed, clean default-branch commit, run `cargo publish --locked` locally.
3. Revoke the bootstrap token immediately after crates.io confirms `quadlet-lens` 0.1.0.
4. Configure this exact trusted-publisher identity in the new crate's settings:

   - GitHub owner: `Strukturpiloten`
   - repository: `quadlet-lens`
   - workflow: `release.yml`
   - environment: `release`

5. Require trusted publishing for the crate.
6. Run the GitHub release workflow from the same commit. It detects that 0.1.0 already exists,
   skips registry authentication and publication, and creates the attested tag and immutable
   GitHub release.

Do not add the bootstrap token as a GitHub secret or variable. It is used only by the maintainer's
local first-publish command and then revoked. For every later version, the authentication action
exchanges the job's GitHub OIDC identity for a short-lived token; `CARGO_REGISTRY_TOKEN` exists
only in the publication step. No bootstrap-token path belongs in the workflow.

## Routine release

For later versions, update only the workspace package version, changelog, and matching
`docs/releases/<version>.md` release notes in a reviewed pull request. After CI succeeds, run the
release workflow from the default branch and approve the `release` environment deployment.

### Release-writing style

- Keep the changelog terse and make release notes a short list of user-visible feature families.
- Put detailed model contracts, capability ranges, fixtures, and evidence in their canonical
  documents; link to them instead of repeating them in release material.
- State compatibility and evidence boundaries once. Group keys that share the same behavior.
- If release notes read like a technical chapter, move the detail into the relevant topic document
  and leave a short summary.

Do not create the tag or GitHub release manually. The workflow re-runs the release gates,
including a version-derived public-API comparison with the latest normal crates.io release, verifies
the locked package, creates a checksum and provenance attestation, creates an annotated tag and
workflow-owned draft release, publishes the crate, and then publishes the immutable GitHub
release. The semver action is pinned by full commit and exact release tag; Renovate maintains both.
The same explicit patch-level comparison runs in normal CI, matching the documented 0.1.x
source-compatibility policy.
The separately scheduled generator matrix remains the exhaustive external conformance tier; the
release job validates its exact matrix contract without downloading all historical generators.

## Recovering a failed workflow

The crates.io publication-state probe runs before tag and draft creation. A later retry from the
same commit verifies an existing tag, replaces only the matching workflow-owned draft by numeric
release ID, and skips a crate version already present on crates.io.

If fixing the workflow requires a new commit, delete only the unpublished remote tag first so the
corrected workflow can bind the version to the new commit. Never delete or replace a tag or release
that has already been published.

The workflow uses the numeric release ID and asset-upload URL returned directly by GitHub's
create-release API. Do not replace this with `gh release view`, `gh release list`, or the
published-release-by-tag endpoint: drafts can use `untagged-*` URLs and must not depend on
eventual list visibility.
