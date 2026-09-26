# ADR 0018: Active generator patches and historical range evidence

- Status: accepted
- Date: 2026-09-26
- Supersedes: ADR 0006's active-matrix retention rule and ADR 0015's single discovery field

## Context

ADR 0006 requires executing every Podman release in a claimed generator evidence range and
records exact execution sources in the generator matrix. Keeping every superseded patch in every
scheduled and Release run makes that matrix grow without bound. ADR 0015's single newest-upstream
discovery field can also miss a patch on a maintained older minor line. Issue #139 records
prospective 5.8.7 and 6.1.2 targets. The active 5.8.6 and 6.1.0 targets remain until the
intermediate 6.1.1 and all retiring patches have durable generator evidence. The 6.1.0
`Container.ImageVolume` introduction remains a
historical semantic boundary regardless of the active test pin.

## Decision

The [active generator matrix](../../tools/generator-matrix.toml) holds the releases rerun by smoke,
full, scheduled, and exact-candidate Release validation. A superseded patch may leave that matrix
only after its exact source and successful generator result are recorded in the separate,
version-controlled [historical ledger](../../tools/generator-history.toml). A skipped intermediate
patch must first pass the same complete fixture suite before the catalogue range can cross it.
The ledger preserves each immutable image reference or released tag and peeled source commit,
the digest-pinned builder for source builds, source license, test date, exact fixture and harness
identity, command, and result. A record awaiting execution is a source pin, not generator evidence.
Successful records are historical evidence, not fresh validation of later release candidates.
An actual passing probe whose uncommitted harness source cannot be retrieved is retained as
`preliminary-passed`, with immutable hashes and a separate artifact. It does not justify
retirement, capability expansion, or target admission. A final `passed` record needs a rerun
against the reviewed, retrievable harness and fixtures; the preliminary hashes are never rewritten.

The catalogue may extend only after every released patch in the extended range has been executed
against the relevant reviewed fixtures. A later change to fixture meaning, generator assertions,
or the capability claim requires the affected historical patch to be rerun or the range to be
narrowed. Replacing an operational patch target never moves a genuine feature introduction or bug
boundary. Renovate owns `latest_upstream` for new minor-line discovery and a separate
`latest_<major>_<minor>` signal for each maintained minor line's patches. These signals can advance
independently and cannot change an active generator,
capability claim, or application contract. Only reviewed changes advance `tracked_current`, the
catalogue, and application contracts together.

Once a retiring patch has durable successful evidence, the active matrix can contract to one
latest-patch lane per maintained minor line, except that the 5.4.0 support-floor lane remains
alongside the latest 5.4 patch. Until then, previously active patches remain in the full Release
lane even if they also have pending retirement records. Advancing an active lane requires exact
source or image provenance, a complete generator run, and review of the historical range it
replaces. Pending records cannot support a new range claim. The release gate reruns every active
lane; it does not rerun genuinely retired entries on each candidate.

## Consequences

Release validation remains complete for the active matrix and independently authored application
contracts, while historical patch coverage has explicit provenance without growing the release
runtime indefinitely. Historical results establish dry-run generator text only. They do not prove
systemd activation, Podman command execution, rootless or rootful runtime behavior, or application
deployment.

## Alternatives

Keeping 6.1.1 and every superseded patch in the active matrix was rejected because it grows the
ongoing native gate without improving current patch coverage. Inferring 6.1.1 behavior solely from
a source diff was rejected because ADR 0006 requires generator execution for the entire claimed
range.
