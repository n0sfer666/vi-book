#!/usr/bin/env python3
import argparse
import datetime
import fcntl
import os
import platform
import pty
import select
import shutil
import signal
import statistics
import struct
import subprocess
import sys
import termios
import time
from pathlib import Path

from bench_vault import vault, wiki

ROOT = Path(__file__).resolve().parent.parent
BINARY = ROOT / "target" / "release" / "v-ide"
BENCH = ROOT / ".bench"
CLEANUP_WAIT = 5
ROWS, COLS = 40, 120
QUIET = 0.1
DEADLINE = 10.0
REPLIES = {
    b"\x1b[c": b"\x1b[?62;22c",
    b"\x1b[0c": b"\x1b[?62;22c",
    b"\x1b[5n": b"\x1b[0n",
    b"\x1b[6n": b"\x1b[1;1R",
    b"\x1b]10;?": b"\x1b]10;rgb:ffff/ffff/ffff\x1b\\",
    b"\x1b]11;?": b"\x1b]11;rgb:0000/0000/0000\x1b\\",
}
def environment(home):
    env = {
        key: value
        for key, value in os.environ.items()
        if key not in ("COLORTERM", "V_IDE_CONFIG", "VIBOOK_CONFIG", "NVIM", "VIMINIT")
    }
    env.update(HOME=str(home), TERM="xterm-256color", XDG_CONFIG_HOME=str(home / ".config"))
    return env


def reply(fd, chunk):
    found = []
    for query, answer in REPLIES.items():
        start = chunk.find(query)
        while start >= 0:
            found.append((start, answer))
            start = chunk.find(query, start + 1)
    for _, answer in sorted(found):
        os.write(fd, answer)


def first_frame(fd):
    start = time.perf_counter()
    last = None
    while True:
        wait = QUIET if last else DEADLINE
        ready, _, _ = select.select([fd], [], [], wait)
        if not ready:
            break
        try:
            chunk = os.read(fd, 65536)
        except OSError:
            break
        if not chunk:
            break
        reply(fd, chunk)
        last = time.perf_counter()
    if last is None:
        raise RuntimeError("no output before deadline")
    return last - start


def measure(argv, cwd, env):
    pid, fd = pty.fork()
    if pid == 0:
        fcntl.ioctl(0, termios.TIOCSWINSZ, struct.pack("HHHH", ROWS, COLS, 0, 0))
        os.chdir(cwd)
        os.execvpe(argv[0], argv, env)
    try:
        elapsed = first_frame(fd)
    finally:
        os.kill(pid, signal.SIGKILL)
        _, _, usage = os.wait4(pid, 0)
        os.close(fd)
    scale = 1 if sys.platform == "darwin" else 1024
    return elapsed, usage.ru_maxrss * scale


def run(argv, cwd, env, runs):
    samples = [measure(argv, cwd, env) for _ in range(runs)]
    millis = statistics.median(t for t, _ in samples) * 1000
    megabytes = statistics.median(r for _, r in samples) / (1024 * 1024)
    return millis, megabytes


def scenarios(small, large, mid, wiki_like, nvim):
    small_note = str(small / "note-00000.md")
    large_note = str(large / "note-00000.md")
    rows = [
        ("file, vault 10", "v-ide", [str(BINARY), small_note], small),
        ("file, vault 10", "nvim --clean", [nvim, "--clean", small_note], small),
        ("file, vault 10k", "v-ide", [str(BINARY), large_note], large),
        ("file, vault 10k", "nvim --clean", [nvim, "--clean", large_note], large),
        ("picker, vault 1k", "v-ide", [str(BINARY)], mid),
        ("picker, vault 10k", "v-ide", [str(BINARY)], large),
        (f"picker, wiki-like {wiki_like[1]}", "v-ide", [str(BINARY)], wiki_like[0]),
    ]
    return [row for row in rows if nvim or row[1] == "v-ide"]


def output(command):
    result = subprocess.run(command, capture_output=True, text=True, check=False)
    return result.stdout.strip().splitlines()[0] if result.stdout.strip() else "?"


def machine():
    if sys.platform == "darwin":
        return output(["sysctl", "-n", "machdep.cpu.brand_string"])
    return platform.processor() or platform.machine()


def header(nvim, runs):
    commit = output(["git", "-C", str(ROOT), "rev-parse", "--short", "HEAD"])
    version = output([nvim, "--version"]) if nvim else "not found"
    print(f"{datetime.date.today()} · {commit} · {machine()} · {platform.system()} · {version}")
    print(f"runs: {runs} · median · {COLS}x{ROWS} · quiet {int(QUIET * 1000)} ms")
    print()
    print("| scenario | editor | first frame, ms | peak RSS, MB |")
    print("|---|---|---:|---:|")


def cleanup():
    print(f"\nDelete {BENCH}? [Y/n] (auto in {CLEANUP_WAIT} s) ", end="", flush=True)
    ready, _, _ = select.select([sys.stdin], [], [], CLEANUP_WAIT)
    answer = sys.stdin.readline().strip().lower() if ready else ""
    if answer.startswith("n"):
        print(f"kept {BENCH}")
        return
    shutil.rmtree(BENCH, ignore_errors=True)
    print(f"deleted {BENCH}")


def main():
    parser = argparse.ArgumentParser(description="Startup time to first frame and peak RSS")
    parser.add_argument("--runs", type=int, default=10)
    args = parser.parse_args()
    subprocess.run(["cargo", "build", "--release", "--quiet"], cwd=ROOT, check=True)
    nvim = shutil.which("nvim")
    shutil.rmtree(BENCH, ignore_errors=True)
    home = BENCH / "home"
    home.mkdir(parents=True)
    env = environment(home)
    try:
        small, mid, large = vault(BENCH, 10), vault(BENCH, 1000), vault(BENCH, 10000)
        wiki_like = wiki(BENCH)
        header(nvim, args.runs)
        for name, editor, argv, cwd in scenarios(small, large, mid, wiki_like, nvim):
            millis, megabytes = run(argv, cwd, env, args.runs)
            print(f"| {name} | {editor} | {millis:.1f} | {megabytes:.1f} |", flush=True)
    finally:
        cleanup()


if __name__ == "__main__":
    main()
