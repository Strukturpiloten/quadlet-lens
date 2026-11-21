#!/usr/bin/env python3
"""Explicit rootful hosted Podman prerequisite/launcher; never a Docker fallback."""

from __future__ import annotations

import errno
import hashlib
import json
import os
import pathlib
import re
import signal
import subprocess
import sys
import time
from collections.abc import Mapping
from contextlib import contextmanager


class Rejected(ValueError):
    """A refusal with a fixed message written by this launcher."""


class PreparationFailed(Exception):
    """A preparation stage and a cause safe for hosted logs."""


def require(value: bool, message: str) -> None:
    if not value:
        raise Rejected(message)


@contextmanager
def preparation_stage(stage: str):
    # Subprocess exceptions can carry command arguments and captured streams.
    # Parser and I/O exceptions can include input values and host paths.
    try:
        yield
    except Rejected as error:
        raise PreparationFailed(f"{stage}: {error}") from None
    except subprocess.TimeoutExpired:
        raise PreparationFailed(f"{stage}: command timed out") from None
    except subprocess.CalledProcessError as error:
        status = error.returncode
        reason = f"killed by signal {-status}" if status < 0 else f"command exited {status}"
        raise PreparationFailed(f"{stage}: {reason}") from None
    except subprocess.SubprocessError:
        raise PreparationFailed(f"{stage}: subprocess failure") from None
    except OSError as error:
        reason = errno.errorcode.get(error.errno or 0, "I/O failure")
        raise PreparationFailed(f"{stage}: {reason}") from None
    except (ValueError, KeyError, TypeError, RecursionError):
        raise PreparationFailed(f"{stage}: invalid or missing data") from None


def hosted_environment(environment: Mapping[str, str], release: str) -> None:
    require(
        environment.get("GITHUB_ACTIONS") == "true" and environment.get("RUNNER_OS") == "Linux",
        "hosted launcher requires a Linux GitHub Actions runner",
    )
    fields = dict(line.split("=", 1) for line in release.splitlines() if "=" in line)
    require(
        fields.get("ID", "").strip('"') == "ubuntu"
        and re.fullmatch(r"[0-9]{2}\.[0-9]{2}", fields.get("VERSION_ID", "").strip('"'))
        is not None,
        "hosted launcher requires the workflow's Ubuntu prerequisite",
    )


def privileged_timeout_prefix(seconds: int) -> list[str]:
    return [
        "/usr/bin/sudo",
        "-n",
        "/usr/bin/timeout",
        "--signal=TERM",
        "--kill-after=2s",
        f"{seconds}s",
    ]


def command(arguments: list[str], environment: Mapping[str, str]) -> list[str]:
    require(
        bool(arguments) and arguments[0] in {"run", "ps", "inspect", "rm", "info", "version"},
        "unsupported hosted Podman operation",
    )
    seconds = {"ps": 16, "inspect": 6, "rm": 6, "info": 8, "version": 8}.get(arguments[0])
    if seconds is None:
        value = environment.get("QUADLET_LENS_ENGINE_RUN_TIMEOUT_SECONDS", "1800")
        require(value in {"1800", "240"}, "invalid hosted generator command budget")
        seconds = int(value)
    # Timeout shares Podman's root privileges; an outer unprivileged timeout
    # alone cannot reliably terminate a privileged child behind sudo.
    return [*privileged_timeout_prefix(seconds), "/usr/bin/podman", "--remote=false", *arguments]


def native_timeout_control() -> dict:
    """Actually exercise UID-0 timeout enforcement, only during hosted prepare."""
    hosted_environment(os.environ, pathlib.Path("/etc/os-release").read_text(encoding="utf-8"))
    controller = pathlib.Path(__file__).with_name("hosted-generator-timeout-control.py")
    controller_digest = hashlib.sha256(controller.read_bytes()).hexdigest()
    started = time.monotonic()
    deadline = (
        started + 9.0
    )  # Root backstop: six seconds plus two-second kill grace; one for launch/reap.
    cancelled = False
    previous = {}

    def cancel(_signum, _frame):
        nonlocal cancelled
        cancelled = True

    process = None
    try:
        for signum in (signal.SIGTERM, signal.SIGINT, signal.SIGHUP):
            previous[signum] = signal.signal(signum, cancel)
        process = subprocess.Popen(
            [
                *privileged_timeout_prefix(6),
                "/usr/bin/python3",
                "-I",
                "-B",
                str(controller),
                *privileged_timeout_prefix(1),
            ],
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            start_new_session=True,
        )
        raw, _ = process.communicate(timeout=max(0.001, deadline - time.monotonic()))
        require(not cancelled, "root timeout control cancelled")
        require(time.monotonic() < deadline, "root timeout control exceeded its deadline")
        status = process.returncode
        if status is not None and status != 0:
            raise subprocess.CalledProcessError(status, "root timeout control")
        require(status == 0 and len(raw) <= 4096, "invalid root timeout control output")
        actual = json.loads(raw)
        require(
            set(actual) == {"schema", "result", "uid", "descendants", "elapsed_ms"}
            and actual["schema"] == 1
            and actual["result"] == "passed"
            and type(actual["uid"]) is int
            and actual["uid"] == 0
            and actual["descendants"] == "sigkill-reaped"
            and type(actual["elapsed_ms"]) is int
            and 2500 <= actual["elapsed_ms"] < 4000,
            "native root timeout evidence unavailable",
        )
        require(
            hashlib.sha256(controller.read_bytes()).hexdigest() == controller_digest,
            "root timeout controller changed",
        )
        return {**actual, "controller_sha256": controller_digest}
    finally:
        try:
            if process is not None:
                # Never send unprivileged arbitrary kills at root descendants.
                # The fixed controller cleans exact pidfd-owned children; its
                # original root backstop remains authoritative on cancellation.
                if process.poll() is None:
                    process.wait(timeout=max(0.001, deadline - time.monotonic()))
                if process.stdout is not None:
                    process.stdout.close()
        finally:
            for signum, handler in previous.items():
                signal.signal(signum, handler)


def profile(raw: bytes, package: str) -> dict:
    actual = json.loads(raw)
    host = actual["host"]
    require(
        host["os"] == "linux"
        and host["security"]["rootless"] is False
        and host["serviceIsRemote"] is False,
        "hosted Podman must be local and rootful",
    )
    version = actual["version"]["Version"]
    require(
        isinstance(version, str) and re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", version) is not None,
        "Podman version is unavailable",
    )
    require(
        re.fullmatch(r"[A-Za-z0-9.+:~_-]{1,128}", package) is not None,
        "Podman package identity is unavailable",
    )
    runtime = host["ociRuntime"]
    require(
        isinstance(runtime["name"], str) and isinstance(runtime["version"], str),
        "OCI runtime identity is unavailable",
    )
    return {
        "schema": 1,
        "engine": "podman",
        "version": version,
        "package": package,
        "rootless": False,
        "remote": False,
        "oci_runtime": runtime["name"],
        "oci_runtime_version": runtime["version"],
        "installation": "ubuntu-signed-apt-prerequisite",
    }


def prepare(destination: pathlib.Path) -> None:
    with preparation_stage("check evidence destination"):
        require(
            destination == pathlib.Path("target/release-evidence/outer-podman.json"),
            "invalid prerequisite output",
        )
    with preparation_stage("install Podman prerequisite"):
        if not pathlib.Path("/usr/bin/podman").is_file():
            # Ubuntu's signed APT archive owns package versions/integrity. This is
            # a documented host-prerequisite exception, not an inner generator pin.
            subprocess.run(
                [
                    *privileged_timeout_prefix(80),
                    "/bin/sh",
                    "-ceu",
                    "apt-get update && apt-get install --yes --no-install-recommends podman",
                ],
                check=True,
                timeout=85,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
    with preparation_stage("read rootful Podman info"):
        raw = subprocess.run(
            command(["info", "--format", "json"], os.environ),
            check=True,
            timeout=12,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
        ).stdout
    with preparation_stage("read Podman package identity"):
        package = subprocess.run(
            ["/usr/bin/dpkg-query", "--show", "--showformat=${Version}", "podman"],
            check=True,
            timeout=3,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
        ).stdout.decode("ascii")
    with preparation_stage("validate rootful Podman profile"):
        metadata = profile(raw, package)
    with preparation_stage("verify root timeout control"):
        metadata["timeout_self_control"] = native_timeout_control()
    with preparation_stage("write outer Podman evidence"):
        destination.parent.mkdir(parents=True, exist_ok=True)
        with destination.open("x", encoding="utf-8") as output:
            json.dump(metadata, output, sort_keys=True)
            output.write("\n")


def main(arguments: list[str]) -> int:
    try:
        if len(arguments) == 2 and arguments[0] == "prepare":
            with preparation_stage("check hosted runner"):
                hosted_environment(
                    os.environ, pathlib.Path("/etc/os-release").read_text(encoding="utf-8")
                )
            prepare(pathlib.Path(arguments[1]))
        else:
            hosted_environment(
                os.environ, pathlib.Path("/etc/os-release").read_text(encoding="utf-8")
            )
            argv = command(arguments, os.environ)
            os.execv(argv[0], argv)
        return 0
    except PreparationFailed as error:
        print(f"hosted Podman preparation failed at {error}", file=sys.stderr)
        return 1
    except (OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError):
        print("hosted Podman prerequisite/operation failed", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
