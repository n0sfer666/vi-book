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

Bundled languages (grammars are compiled into the binary — nothing to install):
**rust, python, javascript, jsx, typescript, tsx, json, bash, go, c, toml,
html, yaml** (plus aliases like `rs`, `py`, `ts`, `sh`). An unknown or missing
tag falls back to a single colour.

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

Press `<Space>` (the *leader* key) to open the menu — a quick list of common
actions: find a note, create one, toggle the tree, jump to today's daily note,
write, quit, open help. Pick an entry with the arrow keys and `<CR>`, or just
press the highlighted letter. Entries marked `→` open a submenu (e.g.
`s` Settings → `w` Wrap → `a`/`t`/`n`); `<Esc>` steps back one level.

## Finding and linking notes

- `<C-p>` — fuzzy-find any note in the vault.
- `<C-o>` / `<C-i>` — jump back / forward through visited notes.

## Splits (columns)

v-ide shows up to **three** independent columns side by side — each with its
own buffer, cursor and mode. `<C-w>` is the *window prefix*:

- `<C-w> w` — move focus to the next column.
- `<C-w> s` / `<C-w> v` — split (duplicate the current note into a new column).
- `<C-w> o` — keep only the active column.
- `<C-w> p` — focus the side panel (`:tree` / `:backlinks`); `<Esc>` returns.

Or use commands: `:vsplit [file]` opens a column (optionally another note),
`:only` collapses back to one.

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
status line shows the most important event with `(+N more)`. Writing a note that
is open in another column updates that column too (or marks it `[changed]` if it
has unsaved edits).

## Commands

Type `:` then a command:

- `:w` `:q` `:wq` `:q!` — write / quit.
- `:w!` `:e!` — write over / reload from a file changed outside v-ide.
- `:tree` `:backlinks` — toggle the side panels.
- `:today` `:daily-prev` `:daily-next` — daily notes.
- `:new <name>` — create `<name>.md` and open it.
- `:vsplit [file]` `:split [file]` `:only` `:close` — manage columns.

## Configuration

v-ide reads `~/.config/v-ide/init.lua` on start (no environment variables
needed). See the developer guide for the Lua API. The legacy
`~/.config/vi-book/init.lua` and `VIBOOK_CONFIG` still work, with a warning in
the status line; move the folder with `mv ~/.config/vi-book ~/.config/v-ide`
(or just `init.lua`, if `~/.config/v-ide` already exists).
