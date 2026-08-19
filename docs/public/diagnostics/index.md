# QuadletLens diagnostics

Parsing, document-set validation, capability evaluation, and generation return structured values. They do not print messages or terminate the process.

A diagnostic carries a stable machine-readable code, severity, message, and source labels. Labels use `source::SourceId` and byte spans, so the calling application should keep the matching filename and text when it wants to show line-oriented output.

## Handle recovery

Use `is_valid()` when errors must block the next stage, and inspect `diagnostics()` even when a usable document is present. Recovery can retain unknown entries and surrounding syntax so a caller can explain or preserve input it does not model.

Automation should branch on codes and typed classifications, never on display text. Human wording can improve while the machine contract remains stable.

Sensitive values redact their `Debug` representation by default. Applications remain responsible for avoiding raw source excerpts when those excerpts could contain credentials.

QuadletLens performs no filesystem writes, systemd calls, Podman calls, or network access while producing diagnostics.
