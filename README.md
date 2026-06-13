# vi-book

A terminal note-taking app with vim-style editing — "Obsidian for the terminal".

vim-style editing, a markdown reader, `[[wiki-links]]`, tags, daily notes and
Lua plugins — over plain Markdown files.

## Install

```
./install.sh            # build release + put `vi-book` on PATH + seed config
```

Then run `vi-book` inside a folder of notes, `vi-book ~/notes`, or
`vi-book note.md`. An empty folder shows a start screen offering to create your
first note. No environment variables are required; config lives at
`~/.config/vi-book/init.lua`.

## Keys (essentials)

- Notes open in **read** mode; `i` edits, `<Esc>` or `jj` leaves, `<C-t>` toggles.
- `<Space>` opens the **menu** (leader); `F1` / `?` open **help** (which then
  reveals the full manual / developer guide).
- `h j k l` or **arrows** move; `w`/`b`/`e` words; `W`/`B` blocks; `gg`/`G` ends.
- `<C-p>` fuzzy-find, `<CR>` follow `[[link]]`, `<C-w>` panel, `<C-o>/<C-i>` history.
- `:w :q :tree :backlinks :today :new <name>` commands.

See `docs/manual.md` and `docs/developers.md` (the Lua API).

## Building

```
cargo build --release
cargo test            # tests/ is an immutable contract
```

## License

MIT OR Apache-2.0
