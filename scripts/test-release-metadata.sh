#!/usr/bin/env bash
set -Eeuo pipefail
temporary_directory="$(mktemp -d)"
readonly temporary_directory
trap 'rm -rf -- "${temporary_directory}"' EXIT
changelog="${temporary_directory}/CHANGELOG.md"
readonly changelog
# Model published-tag states without writing Git refs or requiring repository history.
mkdir "${temporary_directory}/bin"
cat > "${temporary_directory}/bin/git" << 'GIT'
#!/usr/bin/env bash
if [[ "$1" == describe && -n "${QUADLET_LENS_TEST_LATEST_TAG:-}" ]]; then
  printf '%s\n' "${QUADLET_LENS_TEST_LATEST_TAG}"
else
  exit 1
fi
GIT
chmod +x "${temporary_directory}/bin/git"
write_changelog() {
  printf '%s\n' "$@" > "${changelog}"
}
check_metadata() {
  local version=$1
  local latest_tag=$2
  shift 2
  env \
    PATH="${temporary_directory}/bin:${PATH}" \
    QUADLET_LENS_RELEASE_METADATA_TEST_MODE=1 \
    QUADLET_LENS_TEST_CHANGELOG="${changelog}" \
    QUADLET_LENS_TEST_LATEST_TAG="${latest_tag}" \
    QUADLET_LENS_TEST_VERSION="${version}" \
    bash scripts/check-release-metadata.sh "$@"
}
expect_failure() {
  local label=$1
  local expected=$2
  local version=$3
  local latest_tag=$4
  local output
  shift 4
  if output="$(check_metadata "${version}" "${latest_tag}" "$@" 2>&1)"; then
    printf 'Release metadata negative case unexpectedly passed: %s\n' "${label}" >&2
    exit 1
  fi
  if [[ "${output}" != *"${expected}"* ]]; then
    printf 'Release metadata case %s did not report %s: %s\n' "${label}" "${expected}" "${output}" >&2
    exit 1
  fi
}
# A prepared version can remain unpublished while the next development changes land.
write_changelog \
  '# Changelog' \
  '' \
  '## [Unreleased]' \
  '' \
  '### Fixed' \
  '' \
  '- Pending fix after release preparation.' \
  '' \
  '## [0.10.0] - 2026-10-27' \
  '' \
  '### Fixed' \
  '' \
  '- Complete prepared release notes.'
for latest_tag in v0.9.0 v0.10.0 ''; do
  check_metadata 0.10.0 "${latest_tag}" --development > /dev/null
  expect_failure 'pending notes in release context' 'Unreleased must be empty' 0.10.0 "${latest_tag}" --release
  expect_failure 'default remains strict' 'Unreleased must be empty' 0.10.0 "${latest_tag}"
done
for mode in --development --release; do
  write_changelog \
    '# Changelog' '' '## [Unreleased]' '' \
    '## [0.10.0] - 2026-10-27' '' '- Complete release notes.'
  for latest_tag in v0.9.0 v0.10.0 ''; do
    check_metadata 0.10.0 "${latest_tag}" "${mode}" > /dev/null
    check_metadata 0.10.0 "${latest_tag}" > /dev/null
  done
  expect_failure 'newest release differs from crate version' 'must match crate version' 0.9.0 v0.9.0 "${mode}"
  write_changelog \
    '# Changelog' '' '## [Unreleased]' '' \
    '## [0.11.0] - 2026-10-28' '' '- Unprepared numbered release.' '' \
    '## [0.10.0] - 2026-10-27' '' '- Complete release notes.'
  expect_failure 'future numbered release' 'must match crate version' 0.10.0 v0.9.0 "${mode}"
  write_changelog \
    '# Changelog' '' '## [Unreleased]' '' \
    '## [0.10.0] - 2026-10-27' '' '- First section.' '' \
    '## [0.10.0] - 2026-10-27' '' '- Duplicate section.'
  expect_failure 'duplicate current-version release section' 'exactly one release section' 0.10.0 v0.9.0 "${mode}"
  write_changelog \
    '# Changelog' '' '## [Unreleased]' '' '## [Unreleased]' '' \
    '## [0.10.0] - 2026-10-27' '' '- Released fix.'
  expect_failure 'duplicate Unreleased section' 'exactly one Unreleased section' 0.10.0 v0.9.0 "${mode}"
  write_changelog '# Changelog' '' '## [0.10.0] - 2026-10-27' '' '- Released fix.'
  expect_failure 'missing Unreleased section' 'exactly one Unreleased section' 0.10.0 v0.9.0 "${mode}"
  write_changelog '# Changelog' '' '## [Unreleased]'
  expect_failure 'missing numbered release' 'must match crate version' 0.10.0 v0.9.0 "${mode}"
  write_changelog '# Changelog' '' '## [Unreleased]' '' '## [0.10.0]' '' '- Undated notes.'
  expect_failure 'undated current release' 'YYYY-MM-DD date' 0.10.0 v0.9.0 "${mode}"
  write_changelog '# Changelog' '' '## [Unreleased]' '' '## [0.10.0] - 2026-10-27' ''
  expect_failure 'empty numbered release' 'is empty' 0.10.0 v0.9.0 "${mode}"
  write_changelog '# Changelog' '' '## [Unreleased]' '' '## [0.10.0] - 2026-10-27' '' '---'
  expect_failure 'unusable numbered notes' 'contains no usable notes' 0.10.0 v0.9.0 "${mode}"
  rm -- "${changelog}"
  expect_failure 'missing changelog' 'Changelog does not exist' 0.10.0 v0.9.0 "${mode}"
done
for invalid_mode in --unknown '' --Development --release=1; do
  expect_failure 'invalid argument' 'Usage:' 0.10.0 '' "${invalid_mode}"
done
expect_failure 'multiple modes' 'Usage:' 0.10.0 '' --development --release
expect_failure 'extra argument' 'Usage:' 0.10.0 '' --release ignored
printf 'QuadletLens release metadata policy tests passed.\n'
