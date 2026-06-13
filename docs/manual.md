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

## Two modes: read and edit

A note opens in **read** mode — markdown is rendered (headings, bold, links,
tables). Press `i` to **edit** the raw text, and `<Esc>` (or `jj`) to go back.
`<C-t>` toggles between the two at any time.

## Moving around

- `h j k l` or the **arrow keys** — move the cursor.
- `w` / `b` / `e` — next / previous / end of word.
- `W` / `B` — next / previous *block*: a heading, paragraph, list item or tag.
- `gg` / `G` — top / bottom of the note.
- `<CR>` on a `[[link]]` — follow it. If several notes match, a picker opens.

## The menu (leader key)

Press `<Space>` (the *leader* key) to open the menu — a quick list of common
actions: find a note, create one, toggle the tree, jump to today's daily note,
write, quit, open help. Pick an entry with the arrow keys and `<CR>`, or just
press the highlighted letter.

## Finding and linking notes

- `<C-p>` — fuzzy-find any note in the vault.
- `<C-w>` — focus the side panel (`:tree` or `:backlinks`); `<C-w>` again or
  `<Esc>` returns focus to the note.
- `<C-o>` / `<C-i>` — jump back / forward through visited notes.

## Commands

Type `:` then a command:

- `:w` `:q` `:wq` `:q!` — write / quit.
- `:tree` `:backlinks` — toggle the side panels.
- `:today` `:daily-prev` `:daily-next` — daily notes.
- `:new <name>` — create `<name>.md` and open it.

## Configuration

vi-book reads `~/.config/vi-book/init.lua` on start (no environment variables
needed). See the developer guide for the Lua API.
