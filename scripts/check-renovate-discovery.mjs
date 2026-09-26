import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const repositoryRoot = new URL('../', import.meta.url);
const renovate = JSON.parse(readFileSync(new URL('.github/renovate.json', repositoryRoot), 'utf8'));
const matrix = readFileSync(new URL('tools/generator-matrix.toml', repositoryRoot), 'utf8');

function extract(description, contents) {
  const owners = renovate.customManagers.filter((manager) => manager.description === description);
  assert.equal(owners.length, 1, `${description} must have one regex manager`);
  const [owner] = owners;
  assert.equal(owner.customType, 'regex');
  assert.deepEqual(owner.managerFilePatterns, ['/^tools/generator-matrix\\.toml$/']);
  assert.equal(owner.matchStrings.length, 1);
  return [...contents.matchAll(new RegExp(owner.matchStrings[0], 'g'))].map((match) => ({ ...match.groups }));
}

function allowedCandidate(depName, candidate) {
  const owners = renovate.packageRules.filter(
    (rule) => rule.matchManagers?.includes('custom.regex') && rule.matchDepNames?.includes(depName),
  );
  assert.equal(owners.length, 1, `${depName} must have one candidate filter`);
  const [owner] = owners;
  assert.match(owner.allowedVersions, /^\/\^.*\$\/$/);
  return new RegExp(owner.allowedVersions.slice(1, -1)).test(candidate);
}

const upstream = extract('Discover the newest stable Podman release across minor lines', matrix);
assert.deepEqual(upstream, [
  {
    datasource: 'github-releases',
    packageName: 'podman-container-tools/podman',
    depName: 'podman-upstream',
    currentValue: '6.1.2',
  },
]);

const minor = extract('Discover the newest stable Podman patch independently for each maintained minor line', matrix);
assert.deepEqual(
  minor.map(({ datasource, packageName, depName, currentValue }) => [datasource, packageName, depName, currentValue]),
  [
    ['github-releases', 'podman-container-tools/podman', 'podman-5.4', '5.4.2'],
    ['github-releases', 'podman-container-tools/podman', 'podman-5.5', '5.5.2'],
    ['github-releases', 'podman-container-tools/podman', 'podman-5.6', '5.6.2'],
    ['github-releases', 'podman-container-tools/podman', 'podman-5.7', '5.7.1'],
    ['github-releases', 'podman-container-tools/podman', 'podman-5.8', '5.8.7'],
    ['github-releases', 'podman-container-tools/podman', 'podman-6.0', '6.0.2'],
    ['github-releases', 'podman-container-tools/podman', 'podman-6.1', '6.1.2'],
  ],
);

const reviewRules = renovate.packageRules.filter(
  (rule) =>
    rule.matchManagers?.includes('custom.regex') &&
    rule.matchPackageNames?.includes('podman-container-tools/podman'),
);
assert.equal(reviewRules.length, 1);
assert.equal(reviewRules[0].automerge, false);
assert.deepEqual(reviewRules[0].matchFileNames, ['tools/generator-matrix.toml']);
assert.equal(reviewRules[0].groupName, undefined);

assert(allowedCandidate('podman-5.8', '5.8.8'));
assert(!allowedCandidate('podman-5.8', '6.1.2'));
assert(!allowedCandidate('podman-5.8', '6.2.0'));
assert(allowedCandidate('podman-upstream', '6.2.0'));
assert(!allowedCandidate('podman-upstream', '6.2.0-rc1'));

const newerPatch = matrix.replace('latest_5_8 = "5.8.7"', 'latest_5_8 = "5.8.8"');
assert.notEqual(newerPatch, matrix);
assert.equal(newerPatch.replace('latest_5_8 = "5.8.8"', 'latest_5_8 = "5.8.7"'), matrix);
assert.equal(extract('Discover the newest stable Podman patch independently for each maintained minor line', newerPatch)[4].currentValue, '5.8.8');
assert.equal(extract('Discover the newest stable Podman release across minor lines', newerPatch)[0].currentValue, '6.1.2');

const newMinor = matrix.replace('latest_upstream = "6.1.2"', 'latest_upstream = "6.2.0"');
assert.notEqual(newMinor, matrix);
assert.equal(newMinor.replace('latest_upstream = "6.2.0"', 'latest_upstream = "6.1.2"'), matrix);
assert.equal(extract('Discover the newest stable Podman release across minor lines', newMinor)[0].currentValue, '6.2.0');
assert.equal(extract('Discover the newest stable Podman patch independently for each maintained minor line', newMinor)[6].currentValue, '6.1.2');

assert.match(newerPatch, /^tracked_current = "6\.1\.2"$/m);
assert.match(newMinor, /^tracked_current = "6\.1\.2"$/m);
console.log('Podman Renovate discovery extraction and candidate filters passed.');
