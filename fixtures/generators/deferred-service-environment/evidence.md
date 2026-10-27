# Deferred native generator evidence

This fixture is independently authored. Oracle execution uses the exact Podman 5.4.0 and 6.1.2
pins and commands recorded in `fixture.toml` and `tools/generator-matrix.toml`. Every selected
active generator, including the full Release matrix, also runs this contract. The Podman oracle
is Apache-2.0; no oracle implementation is copied into QuadletLens or this fixture.

The expected service retains intentional `${PROVIDER}` and `${ROOT}` references. Its ExecStart
word boundaries are checked without executing them. Provider file existence/content, systemd
manager expansion, absolute/nonempty runtime provider values without colons, NUL, or control characters, and workload
behavior remain
external requirements. Files override inline assignments and later files override earlier files
under systemd rules; the generator fixture only establishes declaration preservation and order.

Endpoint source research (no copied implementation) identifies pre-expansion source rewriting:

- Podman 5.4.0 `pkg/systemd/quadlet/quadlet.go`, released tag `v5.4.0`, resolver around line 1894
  and Volume construction around line 2188:
  <https://github.com/containers/podman/blob/v5.4.0/pkg/systemd/quadlet/quadlet.go>.
- Podman 6.1.2, released tag `v6.1.2`, resolver around line 1955 and Volume construction around
  line 2262: <https://github.com/containers/podman/blob/v6.1.2/pkg/systemd/quadlet/quadlet.go>.

Authored `.volume` sources can be interpreted as native unit references before service expansion
at both endpoints. `.artifact` sources gain that interpretation on newer generators, so the
focused API rejects both suffixes uniformly. Volume does not enable image-source rewriting;
`.image` remains a positive case here. A suffix hidden inside the runtime value of a bare
`${ROOT}` reference is unavailable to the generator and is not subject to this authored-shape
restriction. Long `Mount=type=bind` uses related native source resolution and is not a workaround.

No native result is claimed merely by adding this authored expectation. A passing exact endpoint
run is required evidence; never install or execute generated units to obtain it.

Bounded reverse classification follows systemd's embedded versus whole-word environment argument
semantics and single-pass literal escaping, independently implemented from public descriptions.
Version-specific source behavior was researched at systemd v252
(<https://github.com/systemd/systemd/blob/v252/src/basic/env-util.c#L694>) and v261.3
(<https://github.com/systemd/systemd/blob/v261.3/src/basic/env-util.c#L892>). No systemd code is copied
or redistributed here. This research is not an executed runtime/provider-availability result.

The focused bind source shape is restricted to pure absolute literals or one direct leading
reference followed only by literal suffix parts. Prefix/interior or multiple references are
rejected: native generation may derive literal `RequiresMountsFor` dependencies before service
expansion for such shapes. This fixture evidences the leading-reference shape only. Runtime
provider values must be nonempty absolute host paths without colons, NUL, or control characters;
the library never acquires these values or validates them at runtime.

`Volume=` is raw logical native field text at both researched endpoints: outer quotes and
backslashes are literal bytes, unlike Environment word syntax. The fixture deliberately includes
interior spaces, quotes, and backslashes in a reference-leading source; argv quoting belongs to
the generator. Native reverse Volume views preserve these bytes without word unquoting/C-decoding.
Long Mount retains the distinct word-then-CSV boundary.

Older generators derive pure absolute source `RequiresMountsFor` without the required unit-path
escaping, and that directive does not dollar-expand. The focused helper therefore rejects pure
absolute literal sources containing dollars, whitespace, quotes, or backslashes uniformly across
the supported range. Reference-leading sources avoid this inferred source dependency; literal
percent pairs remain meaningful in both specifier contexts. Raw target trailing whitespace and
backslashes are rejected to prevent native line trimming or continuation loss.

Container assignment keys have a separate native construction boundary from strict service names.
Endpoint LookupAllKeyVal splits the first equals and native environment parsing rejects empty keys
and trims leading whitespace; dotted key bytes are otherwise retained. The fixture includes
`NC_davstorage.request_timeout` and literal dollar/percent keys with native escaping. Broad keys
remain recoverable Unmodeled evidence in the narrower source semantic view; no broad reverse-name
support or portable process-environment naming claim follows from native construction.

## Observed endpoint results

On 2026-10-27 the exact pinned Podman 5.4.0 image generator and 6.1.2 source generator both passed
the authored fixture assertions through the recorded selected-version command. Observed results
cover generated argv boundaries and literal/reference spelling, dotted and literal dollar/percent
container keys, raw Volume source bytes, empty assignments, and service provider declaration/order
preservation. They do not execute manager expansion or acquire provider files.

The catalogue links two single-target generator evidence records to the existing container
environment, container volume, and generic systemd-section pass-through capabilities. Existing
capability ranges and IDs are unchanged. Other active versions are configured to execute this
fixture, but no full-matrix observation follows from these two endpoint results. Provider
availability, expansion, systemd activation, and workload behavior remain unverified.
