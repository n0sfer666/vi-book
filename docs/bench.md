# v-ide — startup benchmark

`scripts/bench_startup.py` measures time to first frame and peak RSS of v-ide against
`nvim --clean`.

```nu
python3 -I scripts/bench_startup.py --runs 10
```

The script builds `target/release/v-ide`, generates deterministic vaults (10, 1000 and 10000
notes) in a temp directory, and starts each editor in a PTY (120x40, `TERM=xterm-256color`,
empty `HOME`, no user config). It answers terminal queries the way a basic xterm does
(DA1, DSR, OSC 10/11) in the order they arrive.

- **First frame** — time from spawn to the last output byte before 100 ms of silence.
  This includes any trailing writes the editor makes right after drawing: nvim's content is on
  screen ~35 ms before its last byte (it enables focus events last).
- **Peak RSS** — `ru_maxrss` from `wait4` after `SIGKILL` at first frame.
- Values are medians over `--runs`. The script prints table rows; copy them here by hand.

## Baseline (S07 1)

2026-10-15 · a5abe5b · Apple M3 Pro · Darwin · NVIM v0.12.6
runs: 10 · median · 120x40 · quiet 100 ms

| scenario | editor | first frame, ms | peak RSS, MB |
|---|---|---:|---:|
| file, vault 10 | v-ide | 28.3 | 14.1 |
| file, vault 10 | nvim --clean | 110.7 | 10.3 |
| file, vault 10k | v-ide | 4577.0 | 28.8 |
| file, vault 10k | nvim --clean | 110.7 | 10.9 |
| picker, vault 1k | v-ide | 469.9 | 11.8 |
| picker, vault 10k | v-ide | 4589.8 | 25.9 |

The startup vault scan dominates: opening a single file in a 10k-note vault costs the same as the
picker. S07 (4) targets the "file, vault 10k" row.

## Lazy Lua VM (S07 2)

2026-10-19 · f20ff0f · Apple M3 Pro · Darwin · NVIM v0.12.6
runs: 10 · median · 120x40 · quiet 100 ms

| scenario | editor | first frame, ms | peak RSS, MB |
|---|---|---:|---:|
| file, vault 10 | v-ide | 27.7 | 13.8 |
| file, vault 10 | nvim --clean | 110.3 | 10.3 |
| file, vault 10k | v-ide | 4657.0 | 28.6 |
| file, vault 10k | nvim --clean | 113.4 | 10.9 |
| picker, vault 1k | v-ide | 429.9 | 11.4 |
| picker, vault 10k | v-ide | 4365.0 | 25.7 |

Without init.lua the VM no longer starts at launch; the gain is within noise (−0.6 ms, −0.3 MB on
the small vault). The vault scan still dominates the 10k rows.

## Lazy vault index (S07 4a)

2026-10-20 · bb379f1 + S07 4a · Apple M3 Pro · Darwin · NVIM v0.12.6
runs: 10 · median · 120x40 · quiet 100 ms

| scenario | editor | first frame, ms | peak RSS, MB |
|---|---|---:|---:|
| file, vault 10 | v-ide | 32.8 | 13.8 |
| file, vault 10 | nvim --clean | 120.8 | 10.3 |
| file, vault 10k | v-ide | 24.0 | 13.9 |
| file, vault 10k | nvim --clean | 117.0 | 10.9 |
| picker, vault 1k | v-ide | 431.4 | 10.8 |
| picker, vault 10k | v-ide | 4263.4 | 19.2 |

Opening a file no longer scans the vault: the 10k row drops from 4657 ms to 24 ms and its RSS from
28.6 MB to 13.9 MB. The picker still pays the full scan; S07 4c (mtime cache) targets it.

## Fast vault scan (S07 4a2)

2026-10-21 · ce88aa6 + parallel scan (a19e1bd) · Apple M3 Pro · Darwin · NVIM v0.12.6
runs: 10 · median · 120x40 · quiet 100 ms

| scenario | editor | first frame, ms | peak RSS, MB |
|---|---|---:|---:|
| file, vault 10 | v-ide | 48.0 | 13.7 |
| file, vault 10 | nvim --clean | 119.5 | 10.3 |
| file, vault 10k | v-ide | 41.9 | 13.7 |
| file, vault 10k | nvim --clean | 117.5 | 10.9 |
| picker, vault 1k | v-ide | 75.7 | 13.0 |
| picker, vault 10k | v-ide | 644.5 | 25.7 |

The scan no longer rescans text per code zone, and notes are parsed on all cores. The picker on
10k notes drops from 4216 ms to 645 ms (A/B against b9b9b9c in the same session: 4216 / 430 ms
for 10k / 1k). Peak RSS of the picker grows from 19.1 to 25.7 MB: each worker holds its own
tree-sitter trees. The "file" rows do not scan; their spread between runs of the same commit
(33–48 ms) is noise. A real vault of 1444 notes (10 MB) scans in 0.41 s instead of 3.6 s.
