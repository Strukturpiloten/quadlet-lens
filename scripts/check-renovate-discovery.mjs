import { spawnSync } from "node:child_process";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const repositoryRoot = new URL("../", import.meta.url);
const renovate = JSON.parse(readFileSync(new URL(".github/renovate.json", repositoryRoot), "utf8"));
const matrix = readFileSync(new URL("tools/generator-matrix.toml", repositoryRoot), "utf8");
const matrixFilePattern = "/^tools/generator-matrix\\.toml$/";
const upstreamDescription = "Discover the newest stable Podman release across minor lines";
const minorDescription =
  "Discover the newest stable Podman patch independently for each maintained minor line";

function release(value) {
  assert.match(value, /^[0-9]+\.[0-9]+\.[0-9]+$/);
  const parts = value.split(".").map(Number);
  assert(parts.every(Number.isSafeInteger));
  assert.equal(parts.join("."), value, "release components must use canonical decimal spelling");
  return parts;
}

function nextRelease(value, component) {
  const parts = release(value);
  parts[component] += 1;
  for (let index = component + 1; index < parts.length; index += 1) parts[index] = 0;
  assert(parts.every(Number.isSafeInteger));
  return parts.join(".");
}

function fields(contents) {
  const repository = contents.match(
    /^source_repository = "https:\/\/github\.com\/([^"\r\n]+)\.git"$/m,
  );
  assert(repository, "matrix must declare the canonical upstream repository");
  const rows = contents
    .split("\n")
    .filter((line) => line.startsWith("latest_"))
    .map((line) => {
      const match = line.match(/^(latest_(?:upstream|[0-9]+_[0-9]+)) = "([^"\r\n]+)"$/);
      assert(match, "discovery declarations must retain their owned canonical layout");
      const [, key, currentValue] = match;
      const components = release(currentValue);
      const minor = key.match(/^latest_([0-9]+)_([0-9]+)$/);
      if (minor) {
        assert.equal(
          `${components[0]}_${components[1]}`,
          `${minor[1]}_${minor[2]}`,
          "maintained release must match its minor key",
        );
      }
      return {
        key,
        datasource: "github-releases",
        packageName: repository[1],
        depName: minor ? `podman-${minor[1]}.${minor[2]}` : "podman-upstream",
        currentValue,
      };
    });
  assert.equal(
    new Set(rows.map((row) => row.key)).size,
    rows.length,
    "discovery keys must be unique",
  );
  assert.equal(rows.filter((row) => row.key === "latest_upstream").length, 1);
  assert(
    rows.some((row) => row.key !== "latest_upstream"),
    "maintained discovery rows must remain present",
  );
  return rows;
}

function extract(description, contents, configuration = renovate) {
  const owners = configuration.customManagers.filter(
    (manager) => manager.description === description,
  );
  assert.equal(owners.length, 1, `${description} must have one regex manager`);
  const [owner] = owners;
  assert.equal(owner.customType, "regex");
  assert.notEqual(owner.enabled, false);
  assert.deepEqual(owner.managerFilePatterns, [matrixFilePattern]);
  assert.equal(owner.extractVersionTemplate, "^v?(?<version>.*)$");
  assert.equal(owner.matchStrings.length, 1);
  return [...contents.matchAll(new RegExp(owner.matchStrings[0], "g"))].map((match) => ({
    ...match.groups,
  }));
}

function allOwners(contents, configuration) {
  const rows = [];
  for (const manager of configuration.customManagers) {
    if (manager.customType !== "regex" || manager.enabled === false) continue;
    const selected = manager.managerFilePatterns.some((pattern) => {
      assert.match(pattern, /^\/.*\/$/, "review new non-regex manager paths explicitly");
      return new RegExp(pattern.slice(1, -1)).test("tools/generator-matrix.toml");
    });
    if (!selected) continue;
    for (const pattern of manager.matchStrings) {
      for (const match of contents.matchAll(new RegExp(pattern, "g"))) {
        const dependency = { ...match.groups };
        const name = dependency.depName ?? manager.depNameTemplate;
        if (/^podman-(?:upstream|[0-9]+\.[0-9]+)$/.test(name)) rows.push({ manager, dependency });
      }
    }
  }
  return rows;
}

function allowedCandidate(depName, candidate, configuration = renovate) {
  const owners = configuration.packageRules.filter(
    (rule) => rule.matchManagers?.includes("custom.regex") && rule.matchDepNames?.includes(depName),
  );
  assert.equal(owners.length, 1, `${depName} must have one candidate filter`);
  const [owner] = owners;
  assert.deepEqual(owner.matchManagers, ["custom.regex"]);
  assert.deepEqual(owner.matchDepNames, [depName]);
  assert.notEqual(owner.enabled, false);
  assert.match(owner.allowedVersions, /^\/\^.*\$\/$/);
  return new RegExp(owner.allowedVersions.slice(1, -1)).test(candidate);
}

function verifyDiscovery(contents, configuration = renovate) {
  const declarations = fields(contents);
  const expected = (row) => ({
    datasource: row.datasource,
    packageName: row.packageName,
    depName: row.depName,
    currentValue: row.currentValue,
  });
  const upstream = declarations.filter((row) => row.key === "latest_upstream");
  const maintained = declarations.filter((row) => row.key !== "latest_upstream");
  assert.deepEqual(extract(upstreamDescription, contents, configuration), upstream.map(expected));
  assert.deepEqual(extract(minorDescription, contents, configuration), maintained.map(expected));
  const owners = allOwners(contents, configuration);
  assert.equal(
    owners.length,
    declarations.length,
    "each discovery pin must have exactly one active owner",
  );
  for (const row of declarations) {
    assert.equal(
      owners.filter(({ dependency }) => dependency.depName === row.depName).length,
      1,
      `${row.depName} must have exactly one active owner`,
    );
    assert(allowedCandidate(row.depName, row.currentValue, configuration));
    assert(allowedCandidate(row.depName, nextRelease(row.currentValue, 2), configuration));
    assert(
      !allowedCandidate(row.depName, nextRelease(row.currentValue, 2) + "-rc.1", configuration),
    );
    if (row.key !== "latest_upstream") {
      assert(
        !allowedCandidate(row.depName, nextRelease(row.currentValue, 1), configuration),
        "minor filter must reject a different minor",
      );
      assert(!allowedCandidate(row.depName, nextRelease(row.currentValue, 0), configuration));
    } else {
      assert(allowedCandidate(row.depName, nextRelease(row.currentValue, 1), configuration));
      assert(allowedCandidate(row.depName, nextRelease(row.currentValue, 0), configuration));
    }
  }
  const rules = configuration.packageRules.filter(
    (rule) =>
      rule.matchManagers?.includes("custom.regex") &&
      rule.matchPackageNames?.includes(declarations[0].packageName),
  );
  assert.equal(rules.length, 1);
  assert.deepEqual(rules[0].matchFileNames, ["tools/generator-matrix.toml"]);
  assert.equal(rules[0].groupName, "Podman generator discovery");
  for (const field of ["automerge", "dependencyDashboardApproval", "minimumReleaseAge", "enabled"])
    assert.equal(rules[0][field], undefined);
  return declarations;
}

function changeField(contents, key, value) {
  const lines = contents.split("\n");
  const indices = lines.flatMap((line, index) => (line.startsWith(key + " = ") ? [index] : []));
  assert.equal(indices.length, 1);
  lines[indices[0]] = `${key} = "${value}"`;
  return lines.join("\n");
}

function fixedEvidence(contents) {
  return contents
    .split("\n")
    .filter((line) => !line.startsWith("latest_"))
    .join("\n");
}

const baseline = verifyDiscovery(matrix);
for (const row of baseline) {
  const advanced = changeField(
    matrix,
    row.key,
    nextRelease(row.currentValue, row.key === "latest_upstream" ? 1 : 2),
  );
  const changed = verifyDiscovery(advanced);
  assert.equal(
    fixedEvidence(advanced),
    fixedEvidence(matrix),
    "discovery updates must not alter tracked/current/native evidence",
  );
  for (const other of baseline.filter((candidate) => candidate.key !== row.key))
    assert.deepEqual(
      changed.find((candidate) => candidate.key === other.key),
      other,
      "newest and maintained discovery must advance independently",
    );
}

const duplicate = structuredClone(renovate);
duplicate.customManagers.push({
  ...structuredClone(
    renovate.customManagers.find((manager) => manager.description === upstreamDescription),
  ),
  description: "Synthetic duplicate discovery owner",
});
assert.throws(() => verifyDiscovery(matrix, duplicate), /exactly one active owner/);
const missing = structuredClone(renovate);
missing.customManagers = missing.customManagers.filter(
  (manager) => manager.description !== upstreamDescription,
);
assert.throws(() => verifyDiscovery(matrix, missing), /one regex manager/);
const minor = baseline.find((row) => row.key !== "latest_upstream");
const broadFilter = structuredClone(renovate);
broadFilter.packageRules.find((rule) =>
  rule.matchDepNames?.includes(minor.depName),
).allowedVersions = "/^[0-9]+\\.[0-9]+\\.[0-9]+$/";
assert.throws(() => verifyDiscovery(matrix, broadFilter), /different minor/);
const noFilter = structuredClone(renovate);
noFilter.packageRules = noFilter.packageRules.filter(
  (rule) => !rule.matchDepNames?.includes(minor.depName),
);
assert.throws(() => verifyDiscovery(matrix, noFilter), /one candidate filter/);
assert.throws(
  () => verifyDiscovery(changeField(matrix, minor.key, nextRelease(minor.currentValue, 1))),
  /minor key/,
);
const prerelease = changeField(matrix, minor.key, nextRelease(minor.currentValue, 2) + "-rc.1");
assert.throws(() => verifyDiscovery(prerelease));
console.log("Podman Renovate discovery extraction and candidate filters passed.");

// Opt-in native API evidence uses an already verified canonical cache. Ordinary file/Python
// checks remain deterministic and never acquire a Renovate image or invoke an engine.
if (process.argv.length > 2) {
  assert.deepEqual(process.argv.slice(2), ["--native-node-owner"]);
  const provided = process.env.BOXFERRY_RENOVATE_PACKAGE;
  assert(provided, "--native-node-owner requires an existing BOXFERRY_RENOVATE_PACKAGE cache");
  const verification = spawnSync(
    "python3",
    [
      "-B",
      "-c",
      String.raw`from pathlib import Path
import json
import sys
import boxferry_ci
from boxferry_ci.renovate_runtime import prepare_runtime, run_command
module = boxferry_ci.__file__
assert module is not None
shared = Path(module).resolve().parents[2]
def cache_only(arguments, timeout):
    if arguments[0] != "node":
        raise ValueError("explicit native extraction refuses runtime engine acquisition")
    return run_command(arguments, timeout)
runtime = prepare_runtime(shared / "renovate-runtime.json", Path(sys.argv[1]),
                          engine="podman", command=cache_only)
print(json.dumps({"runtime": str(runtime), "shared": str(shared.parent)}))
`,
      provided,
    ],
    {
      encoding: "utf8",
      timeout: 60000,
    },
  );
  assert.equal(verification.status, 0, verification.stdout + verification.stderr);
  const verified = JSON.parse(verification.stdout);
  const probe = String.raw`const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

(async () => {
  const [runtime, root, shared] = process.argv.slice(1);
  assert(runtime && root && shared, "provide verified runtime, consumer and selected source paths");
  const pin = JSON.parse(fs.readFileSync(path.join(shared, "automation/renovate-runtime.json")));
  const expected = pin.image.match(/^ghcr\.io\/renovatebot\/renovate:([0-9]+\.[0-9]+\.[0-9]+)@sha256:[a-f0-9]{64}$/);
  assert(expected, "selected runtime must retain version and immutable digest");
  const version = JSON.parse(fs.readFileSync(path.join(runtime, "package.json"))).version;
  assert.equal(version, expected[1]);
  const base = path.join(runtime, "dist");
  await require(path.join(base, "logger/index.js")).init();
  assert.equal(require(path.join(base, "util/regex.js")).regexEngineStatus.type, "available");
  require(path.join(base, "config/global.js")).GlobalConfig.set({ localDir: root });
  const memory = require(path.join(base, "util/cache/memory/index.js"));
  memory.init();
  const input = JSON.parse(fs.readFileSync(path.join(root, ".github/renovate.json")));
  const { errors = [], warnings = [], ...raw } = await require(path.join(base, "config/migrate-validate.js")).migrateAndValidate({ repoIsOnboarded: true }, input);
  assert.deepEqual(errors, []);
  assert.deepEqual(warnings, []);
  const revision = JSON.parse(fs.readFileSync(path.join(root, ".github/automation.json"))).revision;
  for (const reference of raw.extends) {
    const match = reference.match(/^github>Strukturpiloten\/boxferry\/\/renovate-presets\/(common|rust)#([a-f0-9]{40})$/);
    assert(match && match[2] === revision, "presets must bind the selected consumer source");
    memory.set("preset:" + reference, JSON.parse(fs.readFileSync(path.join(shared, "renovate-presets", match[1] + ".json"))));
  }
  const { validateConfig } = require(path.join(base, "config/validation.js"));
  const validation = await validateConfig("repository", raw);
  assert.deepEqual(validation.errors, []);
  assert.deepEqual(validation.warnings, []);
  const { config } = await require(path.join(base, "config/presets/index.js")).resolveConfigPresets(raw);
  const native = require(path.join(base, "modules/manager/github-actions/index.js"));
  const regex = require(path.join(base, "modules/manager/custom/regex/index.js"));
  const { matchRegexOrGlobList } = require(path.join(base, "util/string-match.js"));
  const { applyPackageRules } = require(path.join(base, "util/package-rules/index.js"));
  const file = ".github/workflows/ci.yml";
  const source = fs.readFileSync(path.join(root, file), "utf8");
  async function nodeRows(content) {
    const found = [];
    async function consider(manager, dependency, managerConfig = {}) {
      require(path.join(base, "workers/repository/extract/manager-files.js")).normalizeDepNames(dependency);
      if (!(dependency.depName === "node" || dependency.packageName === "actions/node-versions")) return;
      const effective = await applyPackageRules({ ...config, ...managerConfig, ...dependency, manager, packageFile: file, updateType: "patch" }, "consumer Node inventory absence");
      if (effective.enabled !== false) found.push({ manager, dependency });
    }
    if ((!config.enabledManagers || config.enabledManagers.includes("github-actions")) && config["github-actions"]?.enabled !== false) {
      for (const dependency of (await native.extractPackageFile(content, file, config["github-actions"] ?? {}))?.deps ?? []) await consider("github-actions", dependency, config["github-actions"]);
    }
    if (!config.enabledManagers || config.enabledManagers.includes("custom.regex")) for (const manager of config.customManagers ?? []) {
      if (manager.customType !== "regex" || manager.enabled === false || !matchRegexOrGlobList(file, manager.managerFilePatterns)) continue;
      for (const dependency of regex.extractPackageFile(content, file, manager)?.deps ?? []) await consider("custom.regex", dependency, manager);
    }
    return found;
  }
  const actual = await nodeRows(source);
  assert.equal(actual.length, 0, "consumer must not define an independent Node version");
  assert(source.includes("uses: ./_automation/.github/actions/shared-node"));
  assert(!/^\s*node-version:/m.test(source));
  const image = fs.readFileSync(path.join(root, ".devcontainer/Dockerfile"), "utf8");
  const releases = [...image.matchAll(/^# release: ([0-9]+\.[0-9]+\.[0-9]+)-[^\n]+\nFROM docker\.io\/library\/node:/gm)];
  assert.equal(releases.length, 1);
  // Synthetic extraction fixture only: no Action or runtime is executed or acquired.
  const fixture = ["name: synthetic independent Node", "on: workflow_dispatch", "jobs:", "  check:", "    runs-on: ubuntu-latest", "    steps:", "      - uses: actions/setup-node@" + "a".repeat(40), "        with:", "          node-version: " + releases[0][1], ""].join("\n");
  const independent = await nodeRows(fixture);
  assert.equal(independent.length, 1);
  assert.equal(independent[0].manager, "github-actions");
  assert.equal(independent[0].dependency.depType, "uses-with");
  assert.throws(() => assert.equal(independent.length, 0, "consumer must not define an independent Node version"), /independent Node version/);
  console.log(JSON.stringify({ runtime: pin.image, runtimeVersion: version, consumerRevision: revision, declaration: file, independentConsumerNodeRows: actual, negativeIndependentLiteralRows: independent, authority: "same-A shared-node; producer composite functional query and actual dynamic lookup are covered centrally by issue593 v3", operations: "extraction/effective rules only; no lookup, network, engine or source writes" }, null, 2));
})().catch(error => { console.error(error); process.exit(1); });
`;
  const result = spawnSync(
    process.execPath,
    ["-e", probe, verified.runtime, new URL("../", import.meta.url).pathname, verified.shared],
    {
      encoding: "utf8",
      timeout: 60000,
    },
  );
  assert.equal(result.status, 0, result.stdout + result.stderr);
  process.stdout.write(result.stdout);
}
