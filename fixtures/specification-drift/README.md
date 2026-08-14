# Pinned Podman manual evidence

`podman-systemd.unit.5-v6.0.2.md.gz.b64` is deterministic gzip/base64 source evidence for the
official aggregate Quadlet manual at Podman `v6.0.2`. It is retrieved from
`https://raw.githubusercontent.com/podman-container-tools/podman/v6.0.2/docs/source/markdown/podman-systemd.unit.5.md`,
is upstream Apache-2.0 material, and reconstructs to SHA-256
`3b9fc55c9f342a0071aa83d2655178f1216f1348ed9269cb9564746e39debe70`.

`podman-v6.0.2-LICENSE` is the exact Apache-2.0 root license retrieved from
`https://raw.githubusercontent.com/podman-container-tools/podman/v6.0.2/LICENSE` for the
evidence's upstream distribution. That upstream tag has no root `NOTICE` file, so there is no
NOTICE material to reproduce. This README retains the evidence source, version, license, and
digest attribution alongside the copied license.

Policy tests decode it only in a temporary directory, verify the digest, and extract the exact
inventory rows offline. The compressed evidence and its upstream license are included in the Cargo
package so the packaged policy test remains self-contained.
