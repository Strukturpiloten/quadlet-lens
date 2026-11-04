# renovate: datasource=docker depName=docker.io/library/node
# release: 24.21.0-bookworm-slim
FROM docker.io/library/node@sha256:2fe369e969550cde8e867afc3fe370b260140cab4a23d467074295b42163d553 AS node_toolchain

# renovate: datasource=docker depName=mcr.microsoft.com/devcontainers/rust
# release: 2.0.14-bookworm
FROM mcr.microsoft.com/devcontainers/rust@sha256:d599b1bc98c3fc53825462ebf1610f39ef9145aeab485deffb8aaef44f349fe1

SHELL ["/bin/bash", "-o", "pipefail", "-c"]

COPY --from=node_toolchain /usr/local/ /usr/local/

# renovate: datasource=crate depName=cargo-deny
ARG CARGO_DENY_VERSION=0.20.2
# renovate: datasource=crate depName=cargo-llvm-cov
ARG CARGO_LLVM_COV_VERSION=0.9.0
# renovate: datasource=crate depName=cargo-semver-checks
ARG CARGO_SEMVER_CHECKS_VERSION=0.50.0
# renovate: datasource=crate depName=zizmor
ARG ZIZMOR_VERSION=1.28.0

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    rustup component add llvm-tools-preview \
    && cargo install --locked --version "${CARGO_DENY_VERSION}" cargo-deny \
    && cargo install --locked --version "${CARGO_LLVM_COV_VERSION}" cargo-llvm-cov \
    && cargo install --locked --version "${CARGO_SEMVER_CHECKS_VERSION}" cargo-semver-checks \
    && cargo install --locked --version "${ZIZMOR_VERSION}" zizmor

COPY .github/automation.json /tmp/quadlet-lens-tooling/.github/automation.json
COPY scripts/shared-ci.py scripts/install-file-tools.sh /tmp/quadlet-lens-tooling/scripts/
COPY package.json package-lock.json /opt/quadlet-lens-file-tools/

RUN set -eux; \
    npm ci --ignore-scripts --prefix /opt/quadlet-lens-file-tools; \
    ln -s /opt/quadlet-lens-file-tools/node_modules/.bin/markdownlint-cli2 \
      /usr/local/bin/markdownlint-cli2; \
    ln -s /opt/quadlet-lens-file-tools/node_modules/.bin/prettier /usr/local/bin/prettier; \
    bash /tmp/quadlet-lens-tooling/scripts/install-file-tools.sh /usr/local/bin; \
    python3 /tmp/quadlet-lens-tooling/scripts/shared-ci.py tools \
      --destination /usr/local/bin --tools actionlint lychee; \
    rm -r /tmp/quadlet-lens-tooling


LABEL org.opencontainers.image.authors="Martin Beckert <https://github.com/TheRealBecks>" \
      org.opencontainers.image.vendor="Strukturpiloten OHG" \
      org.opencontainers.image.url="https://www.strukturpiloten.de/"
