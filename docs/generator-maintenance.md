# Generator evidence maintenance

Use the [generator matrix guide](generator-matrix.md) to understand the exact pins, dry-run
boundary, application contracts, and execution lanes before changing evidence.

## Add a release

1. Confirm the newest stable upstream release and its exact tag.

2. Prefer an immutable official image and record its manifest digest.

3. Otherwise record the exact release commit and keep the builder pinned.

4. Verify the generator reports the expected version.

5. Run the smoke and full lanes. When retiring a patch or skipping an intermediate patch, run its
   exact source once against the same reviewed fixtures and record its result, fixture/harness
   identity, and bounded execution evidence in the historical ledger. A pending entry is only a
   source pin and cannot justify extending the catalogue. A locally added historical source entry
   must be removed from the active matrix before review.

6. Expand capability ranges only after reviewing results and evidence gaps. Keep genuine feature
   introduction and bug boundaries at their historical versions.

7. Update the checked date, independent application target contracts, and Renovate assessment.
   Renovate owns only per-minor discovery and the active builder reference in the matrix;
   reviewed source commits and immutable historical evidence have no automatic extraction owner.

A new upstream version is a tracked target before it is catalogue evidence.

## Add a fixture

Create the smallest fixture that distinguishes the behavior. Its manifest owns provenance,
environment, version selection, and expected fragments. Include rejection boundaries, reset or
repetition behavior, and exact source references where relevant.

Do not duplicate every expected fragment in prose. The fixture manifest and Rust assertion are the
reviewable contract.

Pull requests keep generator execution opt-in because container availability and source builds are
environment-dependent. The scheduled/manual workflow runs the complete recorded matrix. The same
reusable workflow is the Release native-validation lane: it checks out the exact candidate SHA,
runs both the full pinned generator matrix and tracked-current application contracts, and records
current-run, task-labelled evidence. This remains dry-run generator evidence: it does not install,
enable, start, or otherwise execute systemd units or workloads.
