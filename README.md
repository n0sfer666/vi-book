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

The installer also fetches **JetBrainsMono Nerd Font** (skip with
`VIBOOK_NO_FONT=1`). Set your terminal font to *JetBrainsMono Nerd Font* for the
best rendering of code blocks and glyphs.

## Keys (essentials)

- Notes open in **read** mode; `i` edits, `<Esc>` or `jj` leaves, `<C-t>` toggles.
- `<Space>` opens the **menu** (leader); `F1` / `?` open **help** (which then
  reveals the full manual / developer guide).
- `h j k l` or **arrows** move; `w`/`b`/`e` words; `W`/`B` blocks; `gg`/`G` ends.
- `<C-p>` fuzzy-find, `<CR>` follow `[[link]]`, `<C-o>/<C-i>` history.
- `<C-w>` is the window prefix: `<C-w> w` cycles split focus, `<C-w> p` focuses
  the side panel, `<C-w> s`/`v` splits, `<C-w> o` keeps only the active column.
- `:w :q :tree :backlinks :today :new <name>` and `:vsplit [file] :only` commands.
- Code blocks are syntax-highlighted; the palette is themeable from `init.lua`.
- Following a `[[link]]` to a code file opens it in your external editor.

See `docs/manual.md`, `docs/developers.md` and `docs/plugin-development.md`
(the Lua plugin guide; Russian: `docs/plugin-development.ru.md`).

## Building

```
cargo build --release
cargo test            # tests/ is an immutable contract
```

## License

MIT OR Apache-2.0
