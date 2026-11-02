# v-ide — manual

v-ide is a terminal note-taking app: an "Obsidian for the terminal" with
vim-style editing, a pretty markdown reader, `[[wiki-links]]`, tags, daily
notes and Lua plugins. It only ever touches **Markdown** files.

## Opening a vault

A *vault* is just a folder of `.md` files. Open one by running `v-ide` inside
it, or by passing a path:

```
v-ide                # use the current folder
v-ide ~/notes        # open a folder
v-ide ~/notes/a.md   # open a single note (its folder becomes the vault)
```

If the folder has no notes yet, v-ide shows a start screen and offers to
create your first note.

For the best rendering, set your terminal font to a **Nerd Font** (the
installer fetches *JetBrainsMono Nerd Font* unless `V_IDE_NO_FONT=1`; legacy `VIBOOK_NO_FONT=1` still works).

## Two modes: read and edit

A note opens in **read** mode — markdown is rendered (headings, bold, links,
tables, syntax-highlighted code). Press `i` to **edit** the raw text, and
`<Esc>` (or `jj`) to go back. `<C-t>` toggles between the two at any time.
Files that are not markdown (e.g. `:e main.rs`) open in edit mode, and `<C-t>`
only says `read view: markdown only`.

Content uses the full width of the column (no centring), like vim or a plain
editor. One pane fills the screen; split panes each fill their share.

## Line wrapping

By default **nothing wraps** (like vim `nowrap`): long lines run off the right
edge. A dim `›` in the last column means the line continues; once you scroll
right a dim `‹` appears on the left. The view scrolls horizontally to follow
the cursor in both read and edit modes — just move right (`l`, `w`, `$`).

Three wrap modes, via `<Space> s w` or `:wrap <mode>`:

- `none` — wrap nothing (default).
- `text` — wrap prose: paragraphs, quotes, list items.
- `all` — wrap everything except tables.

Set the startup default in `~/.config/v-ide/init.lua`:

```lua
vibook.opt.wrap = "text"   -- "none" | "text" | "all"
```

Tables never wrap in any mode — scroll horizontally to read wide ones.

In read mode `0` and `$` jump to the start and end of the whole wrapped line,
while `g0` and `g$` stay on the current screen row (as in vim).

## Code blocks and syntax highlighting

Fenced code blocks are syntax-highlighted by their language tag, e.g.:

````
```tsx
const ok = props.refreshSlot ? <div/> : null; // highlighted
```
````

Grammars are shared libraries built from source on your machine (`git` and a C
compiler are needed); `install.sh` does it for you (skip with
`V_IDE_NO_GRAMMARS=1`; without network it only warns). Out of the box v-ide knows
**rust, python, javascript, typescript, tsx, json, bash, go, c, toml, html,
yaml** (plus aliases like `rs`, `py`, `js`, `jsx`, `ts`, `sh`, `yml`). A tag
without an installed grammar, an unknown tag or a missing tag falls back to a
single colour.

Languages, aliases and pinned grammar revisions come from a
built-in `languages.toml`; `~/.config/v-ide/languages.toml` (or
`$XDG_CONFIG_HOME/v-ide/languages.toml`) replaces entries with the same `name`
and adds new ones:

```toml
[[language]]
name = "toy"
aliases = ["ty"]
inherits = ["javascript"]

[language.grammar]
git = "https://example.com/tree-sitter-toy"
rev = "<commit hash>"
subpath = "toy"
```

```
v-ide grammar fetch [LANG...]   # git sources into ~/.local/share/v-ide/sources
v-ide grammar build [LANG...]   # $CC (or cc) → grammars/<lang>.so|dylib + queries
v-ide grammar check [LANG...]   # load each grammar and report ok / the error
```

Without `LANG` every language from `languages.toml` is processed. Each language
prints `<lang> ok` or `<lang> failed: <reason>`, then a summary; the exit code is
1 if anything failed. Open a file named `grammar` as `v-ide ./grammar`.

The palette is themeable from `~/.config/v-ide/init.lua`:

```lua
vibook.color("keyword",  { fg = "magenta", bold = true })
vibook.color("string",   { fg = "green" })
vibook.color("number",   { fg = "yellow" })
vibook.color("comment",  { fg = "#6a737d", italic = true })
vibook.color("function", { fg = "blue" })
vibook.color("type",     { fg = "cyan" })
```

Token names: `keyword string number comment function type constant variable
operator punctuation property attribute`. Markdown faces (`heading1`, `bold`,
`link`, `quote`, …) are themeable the same way. Full list: the plugin guide.

Press `Enter` on a code block in read mode to copy it (see *Selecting and
copying*).

## Moving around

- `h j k l` or the **arrow keys** — move the cursor.
- `<Home>` / `<End>` — start / end of the line (also while editing and after
  an operator: `d<End>` = `d$`).
- `w` / `b` / `e` — next / previous / end of word.
- `W` / `B` — next / previous *block*: a heading, paragraph, list item or tag.
- `gg` / `G` — top / bottom of the note.
- `<CR>` on a `[[link]]` — follow it. If several notes match, a picker opens.
  A link to a **code file** (e.g. `[[main.rs]]`) opens in your external editor
  (`vibook.opt.code_editor`, falling back to `$EDITOR`).
- `<CR>` also follows a standard `[text](file.md)` markdown link and a bare URL
  (URLs open in your browser) — in both **read** and **edit** mode.

## Selecting and copying

In **read** mode you can select and copy without leaving the rendered view:

- `v` / `V` / `Ctrl-V` — start a character / line / block selection; the
  selection is highlighted. Motions extend it; `Esc` cancels.
- `y` — copy the selection; `Y` — copy the current line.
- `Enter` on a fenced **code block** copies the whole block.

Yanking — `y`/`Y` here, and `y`, `yy`, visual `y` in edit mode — copies to your
**system clipboard** (locally via the OS clipboard, and over SSH via OSC 52) as
well as to v-ide's own register. Conversely, `p`/`P` paste whichever is
**newer**: v-ide's register or the system clipboard, so you can paste text you
copied in another app. A status message confirms the copy. (For the OSC 52 path,
your terminal must allow it; most modern terminals do, and tmux needs `set -g
set-clipboard on`.)

Editing keys (`d`, `c`, `x`, `p`, `o`, `a`, …) drop into edit mode at the same
spot, so the document still feels like one buffer.

## The menu (leader key)

Press `<Space>` (the default *leader*) to open the menu — a quick list of common
actions: find a note, create one, toggle the tree, jump to today's daily note,
write, quit, open help. Pick an entry with the arrow keys and `<CR>`, or just
press the highlighted letter. Entries marked `→` open a submenu (e.g.
`s` Settings → `w` Wrap → `a`/`t`/`n`); `<Esc>` steps back one level.

## Finding and linking notes

- `<C-p>` — fuzzy-find any note in the vault.
- `<C-o>` / `<C-i>` — jump back / forward through visited notes.

## Splits (columns)

v-ide shows up to **three** columns side by side. Windows displaying the
same buffer share text, undo, unsaved changes and saves. Each window keeps
its own cursor and scroll position for every buffer it has shown. `<C-w>`
is the *window prefix*:

- `<C-w> w` — move focus to the next column.
- `<C-w> s` / `<C-w> v` — split (show the current buffer in another column).
- `<C-w> o` — keep only the active column.
- `<C-w> p` — focus the side panel (`:tree` / `:backlinks`); `<Esc>` returns.

Or use commands: `:vsplit [file]` opens a column (optionally another note),
`:only` collapses back to one. Opening an existing file reuses its resident
buffer, including unsaved edits. Save modified columns before `:close` or
`:only`.

## Files changed outside v-ide

Every open buffer is checked against its file several times a second, so edits
from another program, `git checkout` or `$EDITOR` show up on their own. Your own
`:w` and rewrites with the same content are not treated as changes.

- **No unsaved changes** — the buffer is reloaded and the status line shows
  `"<note>" reloaded`. Cursor and scroll stay; `u` brings back the text from before
  the reload. While you are typing or selecting, the reload waits until you are
  back in normal mode.
- **Unsaved changes** — your text is kept, the status line shows
  `"<note>" changed on disk (:e! load, :w! overwrite)` and the buffer is marked
  `[changed]`. Plain `:w` and `:wq` check the file first and refuse until you
  pick: `:e!` takes the file from disk (`u` brings your text back), `:w!` writes
  your text over it.
- **File deleted** — the text stays, the status line shows `"<note>" deleted on disk`
  and the buffer is marked `[deleted]`; `:w` writes the file again. `:e!` on a
  note that was never written says `"<note>" not on disk`.

The marks also appear in `:buffers`. A mark goes away when the file on disk is
back to what you last loaded or saved. When several buffers change at once, the
status line shows the most important event with `(+N more)`. All columns showing the same buffer share its text, undo history, saved state,
and disk marks.

## Commands

Type `:` then a command:

- `:w` `:q` `:wq` `:q!` — write / quit.
- `:w!` `:e!` — write over / reload from a file changed outside v-ide.
- `:tree` `:backlinks` — toggle the side panels.
- `:today` `:daily-prev` `:daily-next` — daily notes.
- `:new <name>` — create `<name>.md` and open it.
- `:vsplit [file]` `:split [file]` `:only` `:close` — manage columns.

## Editor options

`:set`, `:setlocal`, and `:setglobal` support these options:

| Option | Alias | Scope | Default |
| --- | --- | --- | --- |
| `timeoutlen` | `tm` | Global | `1000` milliseconds |
| `readonly` | `ro` | Buffer | Off |
| `number` | `nu` | Window and buffer view | Off |

Use `:setlocal number` to show source line numbers in Edit, or
`:setlocal nonumber` to hide them. A split copies the current view setting;
subsequent changes are independent. Each window remembers its setting for
each buffer. A first visit reuses the buffer's last remembered window setting,
or the new window's own global default if no previous setting exists.

`:set` changes both the effective value and its default. `:setlocal` changes
only the current value; global-only options such as `timeoutlen` still change
globally. `:setglobal` changes the default without changing existing local
values. New buffers start with `readonly` off.

Query with `:set number? timeoutlen?`; list changed values with `:set`, or all
supported values with `:set all`. Scoped lists contain only local options.
Use `invnumber` or `number!` to toggle, `number&` to reset to the default, and
`number<` to copy the current global value. `all&` resets the supported options
in the chosen scope. Numeric options also accept `+=`, `-=`, and `^=`.
`timeoutlen` accepts values from `0` through `2147483647`; zero resolves mapping
prefixes immediately. An invalid argument stops the command while preserving
changes made by earlier arguments.

`readonly` is currently set explicitly. It is shared by every window showing
the same buffer and protects `:w` and `:wq` with E45. For a named buffer, `:w!`
clears its local `readonly` flag before attempting the write; its global
default is preserved. Editing and undo remain available.

## Key mappings

Use `:nmap Q dd` for a recursive normal-mode mapping or `:nnoremap Q dd` to
feed keys directly to built-ins. The `v`/`x`, `o`, `i` and `c` prefixes select
visual, operator-pending, insert and command-line mode. `:map`/`:noremap`
target normal, visual and operator-pending together; `:map!`/`:noremap!`
target insert and command-line mode. Only full command names are supported.

`:nunmap Q` removes a mapping; `:nmapclear` clears a mode. The same prefixes
and `!` forms apply to `:unmap` and `:mapclear`. The built-in insert escape
is a regular mapping: `:iunmap jj`, `:imapclear` or `:mapclear!` removes it.

Both sides accept vim key notation and `<Leader>`. A right-hand side may
mix keys with `<Cmd>command<CR>` segments, which execute without changing
the current mode: `:nnoremap Q j<Cmd>only<CR>k`. A plain `:command<CR>` goes
through the command line. An unterminated `<Cmd>` reports E1255; mapping
errors discard the rest of the expansion. `<Nop>` does nothing.

`<unique>` rejects an existing mapping (E227); `<silent>` and `<special>`
are accepted without additional effects. `<nowait>` runs a complete match
without waiting for a longer one, regardless of definition order; already
queued keys can still select a longer complete match. This deliberately
differs from Neovim's definition-order-dependent `<nowait>` behavior.
`<buffer>`, `<expr>` and `<script>` are not supported. Pipes in the right-hand
side are literal keys, not command separators.

`:map` lists mappings in its modes; `:nmap Q` filters left-hand sides by
prefix. Rows show the modes, left-hand side, `*` for a nonrecursive mapping,
and right-hand side. A common definition stays grouped until a mode is
overridden or removed; independent definitions remain separate. Rows are
sorted by left-hand-side notation, with newer definitions first for equal
keys. `v` remains an alias of `x`: there is no select mode.

The leader can be a key sequence in Vim notation, for example
`vibook.opt.leader = "<C-a>x"`. Type the whole sequence to open the menu;
longer mappings take priority and may wait up to one second. An incomplete
or mismatched sequence returns its keys to normal editing. Empty leader
values report an error and preserve the previous sequence.

## Messages

`:messages` shows the last 1000 nonempty messages, including repetitions.
`:messages clear` clears the history. Reading history does not add it again.
One-line messages stay at the bottom; longer output wraps by display width
and expands upward. Background messages enter history while an open pager
keeps the text you are reading unchanged.

At `-- More --`, `Enter` / `j` / Down advances one line, `d` half a screen,
and Space / `f` / PageDown / Ctrl-f a screen. Use `k` / Up / Backspace,
`u`, and `b` / PageUp / Ctrl-b to go back. `g` and `G` reach the beginning
and end. `q`, Esc or Ctrl-c stops paging. In Normal mode, `:` starts a
command line; in Insert mode, it returns to Normal and discards the
interrupted mapping's remaining keys and commands.
In Visual mode, `:` ends the selection, discards that mapping tail, and
opens an Ex draft with the `'<,'>` range prefix. Ex range execution is
not yet supported.
While an operator is pending, `:` cancels the operator and opens Ex,
preserving the mapping tail as in Normal mode.

The end of the output shows `Press ENTER or type command to continue`.
Enter or Space closes it; other command keys continue editing. Paging keys
can still scroll or be consumed at this prompt, as in Vim. Mappings pause
at `-- More --`; the remaining keys and commands continue according to the
prompt they reach, retaining their remap settings. Pager navigation itself
does not use mappings.

Output produced while typing an Ex command or search does not open a
prompt: the draft stays intact and Enter submits it. Messages still enter
history.

## Configuration

v-ide reads `~/.config/v-ide/init.lua` on start (no environment variables
needed). See the developer guide for the Lua API. The legacy
`~/.config/vi-book/init.lua` and `VIBOOK_CONFIG` still work, with a warning in
the status line; move the folder with `mv ~/.config/vi-book ~/.config/v-ide`
(or just `init.lua`, if `~/.config/v-ide` already exists).
