#!/usr/bin/env python3
"""Closed hosted-only root timeout self-control, not a general process launcher."""

import ctypes
import json
import os
import pathlib
import select
import signal
import subprocess
import sys
import time


WORKER = r'''
import json, os, select, signal, sys
assert os.getuid() == os.geteuid() == 0
signal.signal(signal.SIGTERM, signal.SIG_IGN)
def identity(pid):
    raw = open(f"/proc/{pid}/stat").read().rsplit(")", 1)[1].split()
    return [pid, int(raw[19])]
controller, start = int(sys.argv[1]), int(sys.argv[2])
controller_fd = os.pidfd_open(controller)
assert identity(controller) == [controller, start]
assert not select.select([controller_fd], [], [], 0)[0]
child = os.fork()
if child == 0:
    pass
else:
    print(json.dumps({"uid": os.geteuid(), "worker": identity(os.getpid()), "child": identity(child)}), flush=True)
# No elapsed-time self-expiry: only the exact retained controller's death
# triggers this lifetime backstop. Both sides of fork retain the same pidfd.
select.select([controller_fd], [], [])
os.kill(os.getpid(), signal.SIGKILL)
'''

PREFIX = ["/usr/bin/sudo", "-n", "/usr/bin/timeout", "--signal=TERM", "--kill-after=2s", "1s"]


def identity(pid):
    raw = pathlib.Path(f"/proc/{pid}/stat").read_text().rsplit(")", 1)[1].split()
    uid = next(line.split()[1:] for line in pathlib.Path(f"/proc/{pid}/status").read_text().splitlines()
               if line.startswith("Uid:"))
    if uid != ["0", "0", "0", "0"]:
        raise ValueError("non-root probe process")
    return int(raw[19])


def children(pid):
    return [int(value) for value in pathlib.Path(f"/proc/{pid}/task/{pid}/children").read_text().split()]


def remember(held, pid):
    if pid in held:
        return
    descriptor = os.pidfd_open(pid)
    try:
        start = identity(pid)
    except BaseException:
        os.close(descriptor)
        raise
    held[pid] = (start, descriptor)


def collect(held, pid):
    for child in children(pid):
        try:
            remember(held, child)
            if len(held) > 12:
                raise ValueError("unexpected probe process count")
            collect(held, child)
        except (FileNotFoundError, ProcessLookupError):
            # /proc children can race both disappearance and pidfd_open ESRCH.
            pass


def cleanup(held, deadline, reap):
    while time.monotonic() < deadline:
        collect(held, os.getpid())
        for _start, descriptor in held.values():
            if not select.select([descriptor], [], [], 0)[0]:
                try:
                    signal.pidfd_send_signal(descriptor, signal.SIGKILL)
                except ProcessLookupError:
                    pass
        reap()
        if not children(os.getpid()) and time.monotonic() < deadline:
            return
        time.sleep(0.01)
    raise ValueError("probe cleanup incomplete")


def run(prefix):
    if prefix != PREFIX or os.getuid() != 0 or os.geteuid() != 0:
        raise ValueError("closed root probe only")
    # This controller creates only the fixed timeout tree. Adoption permits
    # actual waitpid SIGKILL evidence for the worker and its descendant.
    if ctypes.CDLL(None, use_errno=True).prctl(36, 1, 0, 0, 0) != 0:
        raise OSError("subreaper unavailable")
    started = time.monotonic()
    deadline = started + 5.5
    observation_deadline = started + 4.0  # Reserve 1.5s for exact failure cleanup/reaping.
    cancelled = False

    def cancel(_signum, _frame):
        nonlocal cancelled
        cancelled = True

    for signum in (signal.SIGTERM, signal.SIGINT, signal.SIGHUP):
        signal.signal(signum, cancel)
    held, reaped = {}, {}
    process = None

    def reap():
        for pid in held:
            if process is not None and pid == process.pid:
                continue
            try:
                actual, status = os.waitpid(pid, os.WNOHANG)
                if actual:
                    reaped[pid] = status
            except ChildProcessError:
                pass
        if process is not None:
            process.poll()

    try:
        process = subprocess.Popen([*prefix, "/usr/bin/python3", "-I", "-B", "-c", WORKER,
                                    str(os.getpid()), str(identity(os.getpid()))],
                                   stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
        remember(held, process.pid)
        os.set_blocking(process.stdout.fileno(), False)
        raw = bytearray()
        observed = None
        while time.monotonic() < observation_deadline:
            if cancelled:
                raise ValueError("probe cancelled")
            collect(held, os.getpid())
            if select.select([process.stdout], [], [], 0.01)[0]:
                raw.extend(os.read(process.stdout.fileno(), 4097 - len(raw)))
                if len(raw) > 4096:
                    raise ValueError("probe output exceeded bound")
                if observed is None and b"\n" in raw:
                    observed = json.loads(raw)
                    collect(held, os.getpid())
                    if set(observed) != {"uid", "worker", "child"} or observed["uid"] != 0:
                        raise ValueError("probe identity unavailable")
                    for pid, start in (observed["worker"], observed["child"]):
                        if pid not in held or held[pid][0] != start:
                            raise ValueError("probe custody unavailable")
            reap()
            if process.returncode is not None and not children(os.getpid()):
                break
        elapsed = time.monotonic() - started
        if observed is None or cancelled or not 2.5 <= elapsed < 4.0 or process.returncode not in (137, -signal.SIGKILL):
            raise ValueError("root timeout result unavailable")
        for pid, _start in (observed["worker"], observed["child"]):
            status = reaped.get(pid)
            if status is None or not os.WIFSIGNALED(status) or os.WTERMSIG(status) != signal.SIGKILL:
                raise ValueError("root descendant SIGKILL/reaping unavailable")
        if any(not select.select([descriptor], [], [], 0)[0] for _start, descriptor in held.values()):
            raise ValueError("root probe process remains")
        return {"schema": 1, "result": "passed", "uid": 0, "descendants": "sigkill-reaped",
                "elapsed_ms": round(elapsed * 1000)}
    finally:
        try:
            cleanup(held, deadline, reap)
        finally:
            if process is not None and process.stdout is not None:
                process.stdout.close()
            for _start, descriptor in held.values():
                os.close(descriptor)


if __name__ == "__main__":
    try:
        print(json.dumps(run(sys.argv[1:]), sort_keys=True), flush=True)
    except (OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError):
        print("root timeout self-control failed", file=sys.stderr)
        sys.exit(1)
