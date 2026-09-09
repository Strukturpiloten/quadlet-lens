# ADR 0014: Independent application generator conformance

Status: accepted

Date: 2026-09-09

## Context

The downloadable real-world corpus proves that QuadletLens can ingest selected native documents.
It does not prove that an actual Podman generator emits the intended service names, dependency
edges, publications, mounts, or commands for an application-sized document set. Expectations
derived from the same exporter as the documents would also be a circular oracle.

## Decision

QuadletLens retains two privacy-reviewed BoxFerry application unit sets at immutable revisions.
Every copied byte has a recorded SHA-256, license, redistribution decision, and source revision.
QuadletLens owns separate, structured `expected-systemd.toml` contracts for their generated
meaning.

Ordinary local, CI, and release checks parse and render every document, build exact native
document graphs, and assert independently selected environment, port, mount, command, reset,
ordering, external-environment, and specifier boundaries. These tests use only public
QuadletLens APIs and add no production dependency on BoxFerry.

An opt-in test-only runner selects one exact generator from `tools/generator-matrix.toml`, verifies
its image digest or source commit and reported version, mounts only the reviewed units read-only,
and checks exact generated-unit inventory and structured dependency, image, command, and Podman
argument values, plus ExecStart-scoped health and user option rules. Incidental whitespace and
complete output bytes are not the contract. A caller may supply an absolute unit directory only
with a repository-owned
`nextcloud` or `forgejo` contract. The directory must contain the exact reviewed file set and
hashes; caller-provided expectations are not accepted.

The tracked-current application check runs in the scheduled generator matrix and before release
publication. Parser acceptance, generator conformance, systemd activation, container execution,
SELinux effects, and application behavior remain separate claims.

## Consequences

BoxFerry can submit its actual emitted units to one native oracle without copying Podman grammar or
Quadlet generator logic. A generator-valid but semantically changed dependency, publication,
mount, or generated name fails independently. Updating an application or target generator
requires an explicit provenance, hash, expectation, and boundary review.

The dry-run evidence does not install units, read environment files, pull images, start workloads,
or prove runtime security and persistence behavior.

## Alternatives considered

### Depend on BoxFerry production types

Rejected because conversion and neutral application semantics remain outside QuadletLens.

### Accept caller-supplied expectations

Rejected because an exporter could then supply both the result and its oracle.

### Store whole generated-output snapshots

Rejected because incidental generator formatting would obscure the application semantics under
review.
