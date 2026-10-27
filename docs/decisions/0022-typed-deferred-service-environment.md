# ADR 0022: Typed deferred service-environment words

- Status: accepted
- Date: 2026-10-27
- Extends: [ADR 0009](0009-validated-programmatic-generation.md),
  [ADR 0011](0011-authored-environment-and-systemd-target-boundaries.md), and
  [ADR 0021](0021-protected-inert-extension-sections.md)

## Context

Consumers need to retain explicitly selected service-environment references in container
assignments and bind sources without constructing systemd quoting themselves. Existing literal
assignment APIs deliberately reject specifiers and cannot distinguish a literal dollar spelling
from an intentional reference. Service providers are external runtime declarations, separate from
container environment values and caller-authorized decoded container environment files.

## Decision

Keep validated ASCII `ServiceEnvironmentName` for references/providers, and add distinct protected
`ContainerEnvironmentName` for literal assignment keys. Container keys reject empty text, equals,
controls/NUL, and leading whitespace while preserving native dotted and dollar/percent keys.
Escape keys as exec literals too; broad construction is not a broad reverse-name support claim.
Add protected ordered `DeferredWordPart` literals and
references, and validated `DeferredWord`. Empty words are supported for empty assignments. Focused
builder methods render container environment and bind-source words. Only reference parts emit
canonical `${NAME}` syntax; literal dollars are doubled in ExecStart-bound contexts and literal
percents are escaped. Environment assignment words receive systemd quoting. Volume is raw field text; its quote and
backslash bytes are literal, and the native generator owns generated argv quoting. No shell, executable expression,
host lookup, substitution evaluation, or provider acquisition is introduced.

Bind sources use native `Volume=` with an absolute literal target and separate native option
tokens. The source is either a pure absolute literal word or one direct leading reference with
only literal suffix parts. Prefix/interior and multiple references are outside the evidenced
boundary because generators can derive dependencies before manager expansion. Pure absolute
literal sources also reject dollars, whitespace, quotes, and backslashes because older generator
inferred mount dependencies cannot preserve those bytes. Percent literals remain supported.
Reference-leading suffixes may retain special literal bytes without inferred source dependencies.
Targets reject trailing whitespace/backslashes to avoid raw line trimming/continuation loss. Reject literal source/target colons, control characters, option delimiter injection, and
non-path literal sources. Reject authored sources ending `.volume` or `.artifact`, which native
generators can resolve as Quadlet references before service expansion. `.image` remains supported
by this Volume path. Actual nonempty absolute provider values without colons, NUL, or control characters remain an external requirement;
these values and their suffixes are unavailable to the generator. The endpoint fixture records
exact oracle provenance and these limitations.

Add protected literal `ServiceEnvironmentAssignment` and ordered `ServiceEnvironmentFile`
declarations. Provider file paths are absolute literals without glob syntax and mandatory by
default; optional missing files require explicit selection. Service `Environment=` dollars remain
literal and percents are escaped; service assignment values are never recursively expanded.
Systemd file providers override inline assignments, with later files overriding earlier files.
The library preserves declarations and order without reading, creating, or resolving files.

Protected values use private protected generated storage, never public printable `EntryValue`
conversions. Debug protects standalone names/parts/words/assignments/files and generated results.
Fresh source containing physical environment/provider/volume/mount key candidates is
conservatively protected through syntax, typed values, and derived views, including malformed or
continuation-consumed headers, independently of X metadata. This changes display protection only.
Explicit raw access and rendering retain original bytes.

For bounded reverse environment/mount views, decode paired dollars and percents exactly once.
Unescaped braced references are deferred; short Volume words starting with a single unescaped
dollar also need manager context. Embedded unbraced dollars remain literal in assignment and
long-mount contexts. Do not reconstruct provider policy or evaluate references. Service inline
and caller-decoded provider values are not exec-decoded; command views retain their existing
lexical contract. Decoded dollar-bearing literals must use the new literal-part renderer when
exported, while existing literal construction API spelling remains unchanged. This extends the
source/value distinction in ADR 0011 and bounded native views in ADR 0013. Authored standard continuations cannot
consume protected generated boundaries.

## Consequences

- Native quoting stays in QuadletLens, while caller allowlists and cross-format policy stay outside.
- Existing literal APIs remain distinct and unchanged.
- Provider availability and runtime manager expansion are not generator acceptance claims.
- Generator tests are dry-run only; never execute or install generated units or generated commands.
- Additional typed providers or native fields require their own evidenced focused boundary.

## Alternatives

Raw expression strings would blur literals/references and duplicate native escaping in adapters.
Shell-based evaluation would add an execution model outside the library's scope. Reusing container
file resolution would conflate already decoded authorized values with external service runtime
providers. Long `Mount=type=bind` is not a native reference-rewriting workaround and adds a distinct
CSV grammar; the bounded Volume representation is explicit instead.
