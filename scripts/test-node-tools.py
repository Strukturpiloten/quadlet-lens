#!/usr/bin/env python3
"""Consumer routing and argument contracts, independent of producer acquisition."""

import json
import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


class NodeToolContracts(unittest.TestCase):
    @unittest.skipUnless(
        os.environ.get("QUADLET_LENS_TEST_NODE_INSTALLATION"),
        "requires selected locked tool fixture",
    )
    def test_real_check_fix_stdin_and_conflicting_parent(self) -> None:
        selection = os.environ["QUADLET_LENS_TEST_NODE_INSTALLATION"]
        if selection == "selected":
            revision = json.loads((ROOT / ".github/automation.json").read_text())["revision"]
            installation = ROOT / "target/boxferry-node-tools" / revision
        else:
            installation = Path(selection).resolve()
        with tempfile.TemporaryDirectory(prefix="quadlet-formatter-") as temporary:
            parent = Path(temporary)
            (parent / ".prettierrc.json").write_text('{"tabWidth": 9}')
            (parent / ".editorconfig").write_text("root = true\n[*]\nindent_size = 9\n")
            (parent / ".markdownlint.json").write_text('{"MD013": {"line_length": 5}}')
            consumer = parent / "consumer"
            consumer.mkdir()
            for name in (".prettierrc.json", ".prettierignore", ".markdownlint.json"):
                shutil.copyfile(ROOT / name, consumer / name)
            (consumer / "space name.json").write_text('{"hello":true}')
            (consumer / "README.md").write_text("#Title\n")
            (consumer / "CHANGELOG.md").write_text("#Ignored\n")

            def run(
                tool: str, *arguments: str, input: str | None = None
            ) -> subprocess.CompletedProcess[str]:
                return subprocess.run(
                    [
                        "bash",
                        str(installation / "scripts/run-node-tool.sh"),
                        str(installation),
                        tool,
                        *arguments,
                    ],
                    cwd=consumer,
                    input=input,
                    text=True,
                    capture_output=True,
                    check=False,
                )

            prettier = (
                "--config",
                ".prettierrc.json",
                "--no-editorconfig",
                "--ignore-path",
                ".prettierignore",
            )
            self.assertEqual(run("prettier", *prettier, "--check", "space name.json").returncode, 1)
            self.assertEqual(run("prettier", *prettier, "--write", "space name.json").returncode, 0)
            self.assertEqual(
                run("prettier", *prettier, "--check", "space name.json", "CHANGELOG.md").returncode,
                0,
            )
            self.assertEqual((consumer / "CHANGELOG.md").read_text(), "#Ignored\n")
            stdin = run(
                "prettier", *prettier, "--stdin-filepath", "space name.json", input='{"hello":true}'
            )
            self.assertEqual(stdin.returncode, 0)
            self.assertEqual(stdin.stdout, '{ "hello": true }\n')
            markdown = ("--config", ".markdownlint.json")
            self.assertEqual(run("markdownlint-cli2", *markdown, ":README.md").returncode, 1)
            self.assertEqual(
                run("markdownlint-cli2", *markdown, "--fix", ":README.md").returncode, 0
            )
            self.assertEqual(run("markdownlint-cli2", *markdown, ":README.md").returncode, 0)
            help_result = run("markdownlint-cli2", "--help")
            self.assertEqual(help_result.returncode, 2)
            self.assertEqual(help_result.stderr, "")
            self.assertIn("Syntax: markdownlint-cli2 glob0 [glob1] ", help_result.stdout)
            self.assertIn("[--help]", help_result.stdout)
            self.assertIn("[--no-globs]", help_result.stdout)
            (consumer / "broken.json").write_text("{")
            self.assertNotEqual(run("prettier", *prettier, "--check", "broken.json").returncode, 0)

    def test_local_arguments_stdin_and_failure(self) -> None:
        with tempfile.TemporaryDirectory(prefix="quadlet-node-") as temporary:
            root = Path(temporary)
            scripts = root / "scripts"
            scripts.mkdir()
            shutil.copyfile(ROOT / "scripts/node-tool.sh", scripts / "node-tool.sh")
            (scripts / "shared-ci.py").write_text(
                "import json, sys\n"
                "print(json.dumps([sys.argv[1:], sys.stdin.read()]))\n"
                "raise SystemExit(19)\n"
            )
            arguments = ["prettier", "--stdin-filepath", "space name.md"]
            environment = dict(os.environ)
            environment.pop("QUADLET_LENS_NODE_TOOLS_ROOT", None)
            result = subprocess.run(
                ["bash", str(scripts / "node-tool.sh"), *arguments],
                cwd="/",
                input="unchanged stdin",
                text=True,
                capture_output=True,
                env=environment,
                check=False,
            )
            self.assertEqual(result.returncode, 19)
            self.assertEqual(
                json.loads(result.stdout),
                [
                    ["node-tool", "--cwd", str(root), *arguments],
                    "unchanged stdin",
                ],
            )

    def test_image_root_cannot_be_redirected(self) -> None:
        result = subprocess.run(
            ["bash", str(ROOT / "scripts/node-tool.sh"), "prettier", "--version"],
            env={**os.environ, "QUADLET_LENS_NODE_TOOLS_ROOT": "/tmp/untrusted"},
            capture_output=True,
            check=False,
        )
        self.assertEqual(result.returncode, 2)
        self.assertIn(b"rebuild the Dev Container", result.stderr)

    def test_file_selection_and_explicit_configuration(self) -> None:
        source = (ROOT / "scripts/check-files.sh").read_text()
        self.assertEqual(
            source.count(
                "scripts/node-tool.sh prettier --config .prettierrc.json --no-editorconfig"
            ),
            4,
        )
        self.assertEqual(
            source.count("scripts/node-tool.sh markdownlint-cli2 --config .markdownlint.json"), 2
        )
        self.assertEqual(source.count("--ignore-path .prettierignore --ignore-unknown"), 4)
        for contract in (
            "--fix",
            "--write",
            "--check",
            'markdown_literals+=(":${file}")',
            "':(exclude,glob)fixtures/**'",
            "check_yaml_document_markers",
            "node scripts/check-renovate-discovery.mjs",
            "tombi lint --error-on-warnings --offline",
        ):
            self.assertIn(contract, source)

    def test_image_and_hosted_share_one_authority(self) -> None:
        image = (ROOT / ".devcontainer/Dockerfile").read_text()
        self.assertIn("--installation-root /opt/quadlet-lens-file-tools --install-only", image)
        self.assertIn("QUADLET_LENS_NODE_TOOLS_ROOT=/opt/quadlet-lens-file-tools", image)
        route = (ROOT / "scripts/node-tool.sh").read_text()
        self.assertIn("/opt/quadlet-lens-file-tools/boxferry-node-tools.py image-node-tool", route)
        self.assertNotIn("shared-ci.py", route.split("fi\n")[0])
        startup = (ROOT / ".devcontainer/verify-tools.sh").read_text()
        self.assertIn('"--help"', startup)
        self.assertIn("markdownlint-cli2 v{expected} (markdownlint v{library['version']})", startup)
        self.assertFalse((ROOT / "package.json").exists())
        self.assertFalse((ROOT / "package-lock.json").exists())
        self.assertNotIn("npm ci", (ROOT / ".github/workflows/ci.yml").read_text())


if __name__ == "__main__":
    unittest.main()
