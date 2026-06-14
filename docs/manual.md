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

Content is centred in both modes, so switching read ⇄ edit never shifts it.

Fenced code blocks are highlighted by language (```` ```python ````,
```` ```rust ````, …). The colour palette is themeable from `init.lua` with
`vibook.color(...)` — see the plugin guide.

## Moving around

- `h j k l` or the **arrow keys** — move the cursor.
- `w` / `b` / `e` — next / previous / end of word.
- `W` / `B` — next / previous *block*: a heading, paragraph, list item or tag.
- `gg` / `G` — top / bottom of the note.
- `<CR>` on a `[[link]]` — follow it. If several notes match, a picker opens.
  A link to a **code file** (e.g. `[[main.rs]]`) opens in your external editor
  (`vibook.opt.code_editor`, falling back to `$EDITOR`).

## The menu (leader key)

Press `<Space>` (the *leader* key) to open the menu — a quick list of common
actions: find a note, create one, toggle the tree, jump to today's daily note,
write, quit, open help. Pick an entry with the arrow keys and `<CR>`, or just
press the highlighted letter.

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
