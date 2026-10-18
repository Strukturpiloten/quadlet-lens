# ADR 0019: Rust baseline

- Status: accepted
- Date: 2026-10-18
- Supersedes: only the Rust minimum clauses of [ADR 0004](0004-versioned-capability-catalogue.md)
  and [ADR 0010](0010-consolidated-0.2-public-api.md)

## Context

The workspace's normal Rust toolchain is already pinned to 1.98.1, while its manifest, Clippy,
and public compatibility claims still name the earlier minimum. The maintained Rust repositories
now share an approved 1.98.1 baseline.

## Decision

QuadletLens requires Rust 1.98.1 or newer. The workspace package `rust-version` in `Cargo.toml`
owns the minimum; Clippy's `msrv` matches it. The exact normal compiler in `rust-toolchain.toml`
must be at least that minimum. Updating the normal compiler does not automatically raise the
public minimum.

Local validation and CI continue reading the minimum from Cargo metadata; Release reuses the
complete CI gate. Repository policy tests check the configuration and public claims together.
Renovate's existing Rust toolchain manager owns the normal compiler and keeps its updates in the
Rust toolchain group. A minimum increase remains an explicit compatibility decision.

## Consequences

Consumers on an earlier compiler must upgrade before updating QuadletLens. This replaces only the
older Rust minimum clauses; the public API, native Podman and systemd targets, immutable historical
evidence, and release versions retain their existing contracts.

## Alternatives considered

Keeping the earlier minimum would continue advertising a compiler baseline that differs from the
approved maintained baseline. Requiring the normal compiler and minimum to remain equal would
turn routine compiler updates into implicit compatibility changes.
