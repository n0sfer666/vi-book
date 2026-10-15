#!/usr/bin/env python3
import argparse
import datetime
import fcntl
import os
import platform
import pty
import random
import select
import shutil
import signal
import statistics
import struct
import subprocess
import sys
import tempfile
import termios
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BINARY = ROOT / "target" / "release" / "v-ide"
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
WORDS = "alpha beta gamma delta note link vault idea draft review plan task".split()


def note(rng, index, count):
    links = " ".join(f"[[note-{rng.randrange(count):05}]]" for _ in range(3))
    tags = " ".join(f"#tag-{rng.randrange(50)}" for _ in range(2))
    prose = "\n\n".join(" ".join(rng.choices(WORDS, k=60)) for _ in range(3))
    items = "\n".join(f"- {' '.join(rng.choices(WORDS, k=6))}" for _ in range(4))
    return (
        f"# Note {index}\n\n{tags}\n\n{prose}\n\n## Links\n\n{links}\n\n"
        f"{items}\n\n```rust\nfn note_{index}() -> usize {{ {index} }}\n```\n"
    )


def vault(base, count):
    path = base / f"vault-{count}"
    path.mkdir()
    rng = random.Random(count)
    for index in range(count):
        (path / f"note-{index:05}.md").write_text(note(rng, index, count))
    return path


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


def scenarios(small, large, mid, nvim):
    small_note = str(small / "note-00000.md")
    large_note = str(large / "note-00000.md")
    rows = [
        ("file, vault 10", "v-ide", [str(BINARY), small_note], small),
        ("file, vault 10", "nvim --clean", [nvim, "--clean", small_note], small),
        ("file, vault 10k", "v-ide", [str(BINARY), large_note], large),
        ("file, vault 10k", "nvim --clean", [nvim, "--clean", large_note], large),
        ("picker, vault 1k", "v-ide", [str(BINARY)], mid),
        ("picker, vault 10k", "v-ide", [str(BINARY)], large),
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


def main():
    parser = argparse.ArgumentParser(description="Startup time to first frame and peak RSS")
    parser.add_argument("--runs", type=int, default=10)
    args = parser.parse_args()
    subprocess.run(["cargo", "build", "--release", "--quiet"], cwd=ROOT, check=True)
    nvim = shutil.which("nvim")
    with tempfile.TemporaryDirectory(prefix="v-ide-bench-") as tmp:
        base = Path(tmp)
        home = base / "home"
        home.mkdir()
        env = environment(home)
        small, mid, large = vault(base, 10), vault(base, 1000), vault(base, 10000)
        header(nvim, args.runs)
        for name, editor, argv, cwd in scenarios(small, large, mid, nvim):
            millis, megabytes = run(argv, cwd, env, args.runs)
            print(f"| {name} | {editor} | {millis:.1f} | {megabytes:.1f} |", flush=True)


if __name__ == "__main__":
    main()
