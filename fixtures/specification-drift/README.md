# Pinned Podman manual evidence

`podman-systemd.unit.5-v6.1.0.md.gz.b64` is deterministic gzip/base64 source evidence for the
official aggregate Quadlet manual at Podman `v6.1.0`. It is retrieved from
`https://raw.githubusercontent.com/podman-container-tools/podman/v6.1.0/docs/source/markdown/podman-systemd.unit.5.md`,
is upstream Apache-2.0 material, and reconstructs to SHA-256
`5b1f681f6358d8715057b52f5b4d29c13530f2ed2fa507f7f9437f86361bac33`.

`podman-v6.1.0-LICENSE` is the exact Apache-2.0 root license retrieved from
`https://raw.githubusercontent.com/podman-container-tools/podman/v6.1.0/LICENSE` for the
evidence's upstream distribution. That upstream tag has no root `NOTICE` file, so there is no
NOTICE material to reproduce. This README retains the evidence source, version, license, and
digest attribution alongside the copied license.

Policy tests decode it only in a temporary directory, verify the digest, and extract the exact
inventory rows offline. The compressed evidence and its upstream license are included in the Cargo
package so the packaged policy test remains self-contained.
