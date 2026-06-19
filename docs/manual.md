# vi-book — manual

vi-book is a terminal note-taking app: an "Obsidian for the terminal" with
vim-style editing, a pretty markdown reader, `[[wiki-links]]`, tags, daily
notes and Lua plugins. It only ever touches **Markdown** files.

## Opening a vault

A *vault* is just a folder of `.md` files. Open one by running vi-book inside
it, or by passing a path:

```
vi-book              # use the current folder
vi-book ~/notes      # open a folder
vi-book ~/notes/a.md # open a single note (its folder becomes the vault)
```

If the folder has no notes yet, vi-book shows a start screen and offers to
create your first note.

For the best rendering, set your terminal font to a **Nerd Font** (the
installer fetches *JetBrainsMono Nerd Font* unless `VIBOOK_NO_FONT=1`).

## Two modes: read and edit

A note opens in **read** mode — markdown is rendered (headings, bold, links,
tables, syntax-highlighted code). Press `i` to **edit** the raw text, and
`<Esc>` (or `jj`) to go back. `<C-t>` toggles between the two at any time.

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

Set the startup default in `~/.config/vi-book/init.lua`:

```lua
vibook.opt.wrap = "text"   -- "none" | "text" | "all"
```

Tables never wrap in any mode — scroll horizontally to read wide ones.

## Code blocks and syntax highlighting

Fenced code blocks are syntax-highlighted by their language tag, e.g.:

````
```tsx
const ok = props.refreshSlot ? <div/> : null; // highlighted
```
````

Bundled languages (grammars are compiled into the binary — nothing to install):
**rust, python, javascript, jsx, typescript, tsx, json, bash, go, c, toml,
html, yaml** (plus aliases like `rs`, `py`, `ts`, `sh`). An unknown or missing
tag falls back to a single colour.

The palette is themeable from `~/.config/vi-book/init.lua`:

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
- `w` / `b` / `e` — next / previous / end of word.
- `W` / `B` — next / previous *block*: a heading, paragraph, list item or tag.
- `gg` / `G` — top / bottom of the note.
- `<CR>` on a `[[link]]` — follow it. If several notes match, a picker opens.
  A link to a **code file** (e.g. `[[main.rs]]`) opens in your external editor
  (`vibook.opt.code_editor`, falling back to `$EDITOR`).

## Selecting and copying

In **read** mode you can select and copy without leaving the rendered view:

- `v` / `V` / `Ctrl-V` — start a character / line / block selection; the
  selection is highlighted. Motions extend it; `Esc` cancels.
- `y` — copy the selection; `Y` — copy the current line.
- `Enter` on a fenced **code block** copies the whole block.

Copied text goes to your **system clipboard** (via OSC 52, so it works over SSH
too) and to vi-book's own register, so you can also paste it with `p` in edit
mode. A status message confirms the copy. (OSC 52 must be enabled in your
terminal; most modern terminals support it, and tmux needs `set -g set-clipboard
on`.)

Editing keys (`d`, `c`, `x`, `p`, `o`, `a`, …) drop into edit mode at the same
spot, so the document still feels like one buffer.

## The menu (leader key)

Press `<Space>` (the *leader* key) to open the menu — a quick list of common
actions: find a note, create one, toggle the tree, jump to today's daily note,
write, quit, open help. Pick an entry with the arrow keys and `<CR>`, or just
press the highlighted letter. Entries marked `→` open a submenu (e.g.
`s` Settings → `w` Wrap → `a`/`t`/`n`); `<Esc>` steps back one level.

## Finding and linking notes

- `<C-p>` — fuzzy-find any note in the vault.
- `<C-o>` / `<C-i>` — jump back / forward through visited notes.

## Splits (columns)

vi-book shows up to **three** independent columns side by side — each with its
own buffer, cursor and mode. `<C-w>` is the *window prefix*:

- `<C-w> w` — move focus to the next column.
- `<C-w> s` / `<C-w> v` — split (duplicate the current note into a new column).
- `<C-w> o` — keep only the active column.
- `<C-w> p` — focus the side panel (`:tree` / `:backlinks`); `<Esc>` returns.

Or use commands: `:vsplit [file]` opens a column (optionally another note),
`:only` collapses back to one.

## Commands

Type `:` then a command:

- `:w` `:q` `:wq` `:q!` — write / quit.
- `:tree` `:backlinks` — toggle the side panels.
- `:today` `:daily-prev` `:daily-next` — daily notes.
- `:new <name>` — create `<name>.md` and open it.
- `:vsplit [file]` `:split [file]` `:only` `:close` — manage columns.

## Configuration

vi-book reads `~/.config/vi-book/init.lua` on start (no environment variables
needed). See the developer guide for the Lua API.
