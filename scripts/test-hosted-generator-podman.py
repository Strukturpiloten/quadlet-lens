#!/usr/bin/env python3
"""Authored, mocked prerequisite/launcher controls: never install or launch Podman."""

import copy
import ast
import importlib.util
import json
import os
import pathlib
import re
import signal
import subprocess
import tempfile
import unittest
from unittest import mock


SPEC = importlib.util.spec_from_file_location("hosted_generator_podman", pathlib.Path(__file__).with_name("hosted-generator-podman.py"))
assert SPEC is not None and SPEC.loader is not None
HOSTED = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(HOSTED)
CONTROL_SPEC = importlib.util.spec_from_file_location("hosted_timeout_control", pathlib.Path(__file__).with_name("hosted-generator-timeout-control.py"))
assert CONTROL_SPEC is not None and CONTROL_SPEC.loader is not None
CONTROL = importlib.util.module_from_spec(CONTROL_SPEC)
CONTROL_SPEC.loader.exec_module(CONTROL)

AUTHORED = {"host": {"os": "linux", "security": {"rootless": False}, "serviceIsRemote": False,
                     "ociRuntime": {"name": "crun", "version": "crun version 1.2.3"}},
            "version": {"Version": "4.9.3"}}
MOCK_TIMEOUT = {"schema": 1, "result": "passed", "uid": 0, "descendants": "sigkill-reaped", "elapsed_ms": 3000}


class HostedPodmanTests(unittest.TestCase):
    def test_shared_workflow_preserves_budget_and_all_consumers(self):
        root = pathlib.Path(__file__).resolve().parents[1]
        workflow = (root / ".github/workflows/generator-matrix.yml").read_text()
        self.assertIn("timeout-minutes: 45", workflow)
        allocations = (("Prepare and record rootful outer Podman", 2),
                       ("Run every pinned supported generator", 33),
                       ("Validate real-application generated semantics", 5))
        for name, minutes in allocations:
            self.assertRegex(workflow, rf"name: {re.escape(name)}\n\s+timeout-minutes: {minutes}\n")
        self.assertIn("timeout-minutes: 2\n        run: bash ./scripts/cleanup-generator-containers.sh", workflow)
        self.assertEqual(sum(minutes for _, minutes in allocations) + 2 + 3, 45)
        release = (root / ".github/workflows/release.yml").read_text()
        self.assertIn("uses: ./.github/workflows/generator-matrix.yml", release)
        self.assertIn('.outer_runtime.engine == "podman"', release)
        self.assertIn(".outer_runtime.rootless == false", release)
        self.assertIn(".outer_runtime.remote == false", release)
        self.assertIn('.outer_runtime.timeout_self_control.result == "passed"', release)
        self.assertIn('.outer_runtime.timeout_self_control.descendants == "sigkill-reaped"', release)

    def test_privilege_correct_local_engine_and_budgets(self):
        for operation, seconds in (("run", 1800), ("ps", 16), ("inspect", 6), ("rm", 6), ("info", 8), ("version", 8)):
            self.assertEqual(HOSTED.command([operation, "authored-argument"], {}),
                             ["/usr/bin/sudo", "-n", "/usr/bin/timeout", "--signal=TERM", "--kill-after=2s",
                              f"{seconds}s", "/usr/bin/podman", "--remote=false", operation, "authored-argument"])
        self.assertIn("240s", HOSTED.command(["run"], {"QUADLET_LENS_ENGINE_RUN_TIMEOUT_SECONDS": "240"}))
        for arguments in ([], ["docker"], ["--remote=true", "run"]):
            with self.assertRaises(ValueError):
                HOSTED.command(arguments, {})
        with self.assertRaises(ValueError):
            HOSTED.command(["run"], {"QUADLET_LENS_ENGINE_RUN_TIMEOUT_SECONDS": "3600"})

    def test_reviewed_host_only(self):
        HOSTED.hosted_environment({"GITHUB_ACTIONS": "true", "RUNNER_OS": "Linux"}, 'ID=ubuntu\nVERSION_ID="24.04"\n')
        for environment, release in (({}, 'ID=ubuntu\nVERSION_ID="24.04"'),
                                     ({"GITHUB_ACTIONS": "true", "RUNNER_OS": "Linux"}, 'ID=fedora\nVERSION_ID=45')):
            with self.assertRaises(ValueError):
                HOSTED.hosted_environment(environment, release)

    def test_actual_profile_refuses_rootless_remote_or_missing_version(self):
        self.assertEqual(HOSTED.profile(json.dumps(AUTHORED).encode(), "4.9.3+ds1-1ubuntu0.2"),
                         {"schema": 1, "engine": "podman", "version": "4.9.3", "package": "4.9.3+ds1-1ubuntu0.2",
                          "rootless": False, "remote": False, "oci_runtime": "crun",
                          "oci_runtime_version": "crun version 1.2.3", "installation": "ubuntu-signed-apt-prerequisite"})
        for field in ("rootless", "remote", "version"):
            value = copy.deepcopy(AUTHORED)
            if field == "rootless":
                value["host"]["security"]["rootless"] = True
            elif field == "remote":
                value["host"]["serviceIsRemote"] = True
            else:
                value["version"]["Version"] = "unavailable"
            with self.assertRaises(ValueError):
                HOSTED.profile(json.dumps(value).encode(), "4.9.3+ds1")

    def test_setup_installation_and_failure_are_not_silently_skipped(self):
        original = pathlib.Path.cwd()
        with tempfile.TemporaryDirectory(prefix="quadlet-hosted-podman-control-") as scratch:
            try:
                os.chdir(scratch)
                responses = [subprocess.CompletedProcess([], 0),
                             subprocess.CompletedProcess([], 0, json.dumps(AUTHORED).encode()),
                             subprocess.CompletedProcess([], 0, b"4.9.3+ds1")]
                with mock.patch.object(pathlib.Path, "is_file", return_value=False), \
                     mock.patch.object(HOSTED, "native_timeout_control", return_value=MOCK_TIMEOUT) as native, \
                     mock.patch.object(HOSTED.subprocess, "run", side_effect=responses) as run:
                    destination = pathlib.Path("target/release-evidence/outer-podman.json")
                    HOSTED.prepare(destination)
                    self.assertEqual(json.loads(destination.read_text())["engine"], "podman")
                    self.assertEqual(run.call_args_list[0].args[0],
                                     ["/usr/bin/sudo", "-n", "/usr/bin/timeout", "--signal=TERM", "--kill-after=2s", "80s",
                                      "/bin/sh", "-ceu", "apt-get update && apt-get install --yes --no-install-recommends podman"])
                    self.assertEqual(len(run.call_args_list), 3)
                    native.assert_called_once_with()
                    self.assertEqual(json.loads(destination.read_text())["timeout_self_control"], MOCK_TIMEOUT)
                destination.unlink()
                with mock.patch.object(pathlib.Path, "is_file", return_value=False), \
                     mock.patch.object(HOSTED.subprocess, "run", side_effect=subprocess.CalledProcessError(1, "authored")):
                    with self.assertRaises(subprocess.CalledProcessError):
                        HOSTED.prepare(destination)
                    self.assertFalse(destination.exists())
            finally:
                os.chdir(original)

    def test_native_probe_is_mocked_and_uses_the_production_prefix(self):
        controller = pathlib.Path(__file__).with_name("hosted-generator-timeout-control.py")
        assignments = {node.targets[0].id: ast.literal_eval(node.value)
                       for node in ast.parse(controller.read_text()).body
                       if isinstance(node, ast.Assign) and isinstance(node.targets[0], ast.Name)}
        self.assertEqual(assignments["PREFIX"], HOSTED.privileged_timeout_prefix(1))
        ast.parse(assignments["WORKER"])
        self.assertIn("controller_fd = os.pidfd_open(controller)", assignments["WORKER"])
        self.assertIn("assert identity(controller) == [controller, start]", assignments["WORKER"])
        self.assertIn("select.select([controller_fd], [], [])", assignments["WORKER"])
        for forbidden in ("monotonic", "sleep", "start_new_session"):
            self.assertNotIn(forbidden, assignments["WORKER"])
        process = mock.Mock(returncode=0)
        process.communicate.return_value = (json.dumps(MOCK_TIMEOUT).encode(), None)
        process.poll.return_value = 0
        with mock.patch.object(HOSTED, "hosted_environment"), \
             mock.patch.object(HOSTED.signal, "signal"), \
             mock.patch.object(HOSTED.subprocess, "Popen", return_value=process) as spawn:
            observed = HOSTED.native_timeout_control()
        self.assertEqual(observed["descendants"], "sigkill-reaped")
        argv = spawn.call_args.args[0]
        self.assertEqual(argv[:6], HOSTED.privileged_timeout_prefix(6))
        self.assertEqual(argv[-6:], HOSTED.privileged_timeout_prefix(1))
        self.assertTrue(argv[-7].endswith("/scripts/hosted-generator-timeout-control.py"))
        process.communicate.assert_called_once()
        self.assertLessEqual(process.communicate.call_args.kwargs["timeout"], 9)

    def test_failed_native_probe_never_publishes_prerequisite_evidence(self):
        original = pathlib.Path.cwd()
        with tempfile.TemporaryDirectory(prefix="quadlet-hosted-podman-refusal-") as scratch:
            try:
                os.chdir(scratch)
                with mock.patch.object(pathlib.Path, "is_file", return_value=True), \
                     mock.patch.object(HOSTED.subprocess, "run", side_effect=[
                         subprocess.CompletedProcess([], 0, json.dumps(AUTHORED).encode()),
                         subprocess.CompletedProcess([], 0, b"4.9.3+ds1")]), \
                     mock.patch.object(HOSTED, "native_timeout_control", side_effect=ValueError("authored refusal")):
                    destination = pathlib.Path("target/release-evidence/outer-podman.json")
                    with self.assertRaises(ValueError):
                        HOSTED.prepare(destination)
                    self.assertFalse(destination.exists())
            finally:
                os.chdir(original)

    def test_cancelled_native_probe_rejects_even_a_successful_mock_report(self):
        handlers = {}
        def handler(signum, callback):
            previous = handlers.get(signum, signal.SIG_DFL)
            handlers[signum] = callback
            return previous
        def completed(**_kwargs):
            handlers[signal.SIGTERM](signal.SIGTERM, None)
            return json.dumps(MOCK_TIMEOUT).encode(), None
        process = mock.Mock(returncode=0)
        process.communicate.side_effect = completed
        process.poll.return_value = 0
        with mock.patch.object(HOSTED, "hosted_environment"), \
             mock.patch.object(HOSTED.signal, "signal", side_effect=handler), \
             mock.patch.object(HOSTED.subprocess, "Popen", return_value=process):
            with self.assertRaises(ValueError):
                HOSTED.native_timeout_control()
        self.assertTrue(all(value == signal.SIG_DFL for value in handlers.values()))

    def test_pidfd_open_disappearance_during_collect_is_not_cleanup_failure(self):
        held = {}
        with mock.patch.object(CONTROL, "children", return_value=[201]), \
             mock.patch.object(CONTROL.os, "pidfd_open", side_effect=ProcessLookupError):
            CONTROL.collect(held, 100)
        self.assertEqual(held, {})

    def model_ineffective_inner_timeout(self, cancel_at=None):
        # Independent fake kernel/process observations. No sudo, prctl, pidfd,
        # signal, fork, /proc read or process launch is actually performed.
        live, reaped, killed, handlers = {200, 201, 202}, set(), [], {}
        state = {"clock": 0.0, "read": False, "cancelled": False}
        process = mock.Mock(pid=200, returncode=None)
        process.stdout.fileno.return_value = 55

        def clock():
            value = state["clock"]
            state["clock"] += 0.05
            if cancel_at is not None and value >= cancel_at and not state["cancelled"]:
                state["cancelled"] = True
                handlers[signal.SIGTERM](signal.SIGTERM, None)
            return value

        def descendants(pid):
            if pid == 100:
                return ([200] if 200 not in reaped else []) + [p for p in (201, 202) if p not in reaped and p - 1 not in live]
            if pid in (200, 201):
                return [pid + 1] if pid + 1 not in reaped and pid in live else []
            return []

        def ready(readers, *_args):
            if readers == [process.stdout]:
                return ([process.stdout] if not state["read"] else []), [], []
            descriptor = readers[0]
            return ([descriptor] if descriptor - 1000 not in live else []), [], []

        def read(_descriptor, _limit):
            state["read"] = True
            return b'{"uid":0,"worker":[201,2010],"child":[202,2020]}\n'

        def send(descriptor, signum):
            self.assertIn(descriptor, (1200, 1201, 1202))
            self.assertEqual(signum, signal.SIGKILL)
            killed.append((descriptor, state["clock"]))
            live.discard(descriptor - 1000)

        def wait(pid, _options):
            if pid in live or pid in reaped:
                raise ChildProcessError
            reaped.add(pid)
            return pid, signal.SIGKILL

        def poll():
            if 200 not in live:
                process.returncode = -signal.SIGKILL
                reaped.add(200)
            return process.returncode

        process.poll.side_effect = poll
        with mock.patch.object(CONTROL.ctypes, "CDLL") as library, \
             mock.patch.object(CONTROL.os, "getuid", return_value=0), \
             mock.patch.object(CONTROL.os, "geteuid", return_value=0), \
             mock.patch.object(CONTROL.os, "getpid", return_value=100), \
             mock.patch.object(CONTROL.os, "pidfd_open", side_effect=lambda pid: pid + 1000), \
             mock.patch.object(CONTROL.os, "close"), \
             mock.patch.object(CONTROL.os, "set_blocking"), \
             mock.patch.object(CONTROL.os, "read", side_effect=read), \
             mock.patch.object(CONTROL.os, "waitpid", side_effect=wait), \
             mock.patch.object(CONTROL, "identity", side_effect=lambda pid: pid * 10), \
             mock.patch.object(CONTROL, "children", side_effect=descendants), \
             mock.patch.object(CONTROL.select, "select", side_effect=ready), \
             mock.patch.object(CONTROL.signal, "signal", side_effect=lambda sig, handler: handlers.update({sig: handler})), \
             mock.patch.object(CONTROL.signal, "pidfd_send_signal", side_effect=send), \
             mock.patch.object(CONTROL.time, "monotonic", side_effect=clock), \
             mock.patch.object(CONTROL.time, "sleep"), \
             mock.patch.object(CONTROL.subprocess, "Popen", return_value=process):
            library.return_value.prctl.return_value = 0
            with self.assertRaises(ValueError):
                CONTROL.run(CONTROL.PREFIX)
        self.assertEqual({fd for fd, _when in killed}, {1200, 1201, 1202})
        self.assertTrue(all(when < 5.5 for _fd, when in killed))
        self.assertEqual(reaped, {200, 201, 202})
        self.assertFalse(live)
        self.assertLess(state["clock"], 5.5)

    def test_ineffective_inner_timeout_fails_and_uses_reserved_exact_cleanup(self):
        self.model_ineffective_inner_timeout()

    def test_late_cancel_fails_and_reaps_inside_the_original_deadline(self):
        self.model_ineffective_inner_timeout(cancel_at=3.7)


if __name__ == "__main__":
    unittest.main()
