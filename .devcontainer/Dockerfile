# renovate: datasource=docker depName=docker.io/library/node
# release: 24.22.1-bookworm-slim
FROM docker.io/library/node:24.22.1-bookworm-slim@sha256:55fb5f5a527ea2c5c4a95f963e48ef90aa38c7b346054fdc8ee9957a2b55d205 AS node_toolchain

# renovate: datasource=docker depName=mcr.microsoft.com/devcontainers/rust
# release: 2.0.14-bookworm
FROM mcr.microsoft.com/devcontainers/rust:2.0.14-bookworm@sha256:d599b1bc98c3fc53825462ebf1610f39ef9145aeab485deffb8aaef44f349fe1

SHELL ["/bin/bash", "-o", "pipefail", "-c"]

COPY --from=node_toolchain /usr/local/ /usr/local/
COPY rust-toolchain.toml /opt/quadlet-lens-tooling/rust-toolchain.toml

# renovate: datasource=crate depName=cargo-deny
ARG CARGO_DENY_VERSION=0.20.2
# renovate: datasource=crate depName=cargo-llvm-cov
ARG CARGO_LLVM_COV_VERSION=0.9.0
# renovate: datasource=crate depName=cargo-semver-checks
ARG CARGO_SEMVER_CHECKS_VERSION=0.50.0
# renovate: datasource=crate depName=zizmor
ARG ZIZMOR_VERSION=1.28.0

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    set -eux; \
    rust_channel="$(python3 -c 'import pathlib, tomllib; print(tomllib.loads(pathlib.Path("/opt/quadlet-lens-tooling/rust-toolchain.toml").read_text())["toolchain"]["channel"])')"; \
    rustup toolchain install "${rust_channel}" --profile minimal --component clippy --component rustfmt --component llvm-tools-preview; \
    rustup default "${rust_channel}"; \
    cargo "+${rust_channel}" install --locked --version "${CARGO_DENY_VERSION}" cargo-deny; \
    cargo "+${rust_channel}" install --locked --version "${CARGO_LLVM_COV_VERSION}" cargo-llvm-cov; \
    cargo "+${rust_channel}" install --locked --version "${CARGO_SEMVER_CHECKS_VERSION}" cargo-semver-checks; \
    cargo "+${rust_channel}" install --locked --version "${ZIZMOR_VERSION}" zizmor

COPY .github/automation.json /opt/quadlet-lens-tooling/.github/automation.json
COPY scripts/shared-ci.py scripts/install-file-tools.sh /opt/quadlet-lens-tooling/scripts/
COPY package.json package-lock.json /opt/quadlet-lens-file-tools/

ENV UV_CACHE_DIR=/opt/quadlet-lens-tooling/uv-cache \
    UV_PYTHON_INSTALL_DIR=/opt/quadlet-lens-tooling/python

RUN set -eux; \
    npm ci --ignore-scripts --prefix /opt/quadlet-lens-file-tools; \
    ln -s /opt/quadlet-lens-file-tools/node_modules/.bin/markdownlint-cli2 \
      /usr/local/bin/markdownlint-cli2; \
    ln -s /opt/quadlet-lens-file-tools/node_modules/.bin/prettier /usr/local/bin/prettier; \
    bash /opt/quadlet-lens-tooling/scripts/install-file-tools.sh /usr/local/bin; \
    python3 /opt/quadlet-lens-tooling/scripts/shared-ci.py tools \
      --destination /usr/local/bin --tools actionlint lychee --python-tools; \
    mkdir -p /opt/quadlet-lens-tooling/target /opt/quadlet-lens-tooling/uv-cache /opt/quadlet-lens-tooling/python; \
    chown -R vscode:vscode /opt/quadlet-lens-tooling/target /opt/quadlet-lens-tooling/uv-cache /opt/quadlet-lens-tooling/python; \
    chmod -R a+rX /opt/quadlet-lens-tooling


LABEL org.opencontainers.image.authors="Martin Beckert <https://github.com/TheRealBecks>" \
      org.opencontainers.image.vendor="Strukturpiloten OHG" \
      org.opencontainers.image.url="https://www.strukturpiloten.de/"
