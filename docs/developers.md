# v-ide — developer / plugin guide

v-ide is configured and extended with **Lua**, loaded from
`~/.config/v-ide/init.lua` (override with the `V_IDE_CONFIG` env var; the
legacy `VIBOOK_CONFIG` and `~/.config/vi-book/init.lua` still work, with a
warning). The Lua runtime is sandboxed: no `io`/`os`/network unless explicitly granted, and a
plugin error never crashes the editor.

> This is the quick reference. For the full guide — sandbox model, every event,
> the highlighting palette and a worked plugin example — see
> [`plugin-development.md`](plugin-development.md)
> (Russian: [`plugin-development.ru.md`](plugin-development.ru.md)).

## Options — `vibook.opt`

```lua
vibook.opt.leader = " "          -- key that opens the menu (default: space)
vibook.opt.insert_escape = "jj"  -- type this in edit mode to leave it (default: "jj"; "" disables)
vibook.opt.menu_position = "bottom" -- "bottom" (which-key bar) or "center" (modal)
vibook.opt.code_editor = "nvim"  -- external editor for code links (falls back to $EDITOR)
vibook.opt.plugins = { links = false } -- toggle a default plugin off
```

## Key mappings — `vibook.keymap`

Mappings accept modes `n`, `i`, `x`/`v`, `o` and `c`. The right-hand side is
a sequence of keys, `<Cmd>command<CR>` segments, or a mixture of both.
`<Cmd>` executes without leaving the current mode. Legacy `:command` strings
still execute directly; they will become ordinary keys in a later step.

```lua
vibook.keymap("n", "<C-s>", ":w")     -- Ctrl-s writes
vibook.keymap("n", "H", "0")          -- H jumps to line start
vibook.keymap("n", "<leader>x", "dd") -- leader, then x deletes the line
vibook.keymap("n", "Q", "j<Cmd>only<CR>k") -- keys around a command
```

Built-in `jj` → `<Esc>` is an insert-mode mapping from `vibook.opt.insert_escape`;
set it to `"jk"` to use that instead, or `""` to turn it off.

## Custom menu entries — `vibook.menu_item`

Add your own actions to the leader menu. The right-hand side is a `:command`.

```lua
vibook.menu_item("g", "Open scratch", ":new scratch")
vibook.opt.menu_position = "center" -- make the menu a centered modal
```

## Commands & events

```lua
vibook.command("upper", function()
  vibook.buf.set_text(vibook.buf.get_text():upper())
end)

vibook.on("note_save", function() --[[ ... ]] end)
vibook.on("note_open", function() --[[ ... ]] end)
vibook.on("follow_link", function(target) --[[ route the link target ]] end)
```

## Palette — `vibook.color(name, spec)`

```lua
vibook.color("keyword", { fg = "magenta", bold = true })
vibook.color("string",  { fg = "green" })
vibook.color("link",    { fg = "blue", underline = true })
```

Token names: `keyword string number comment function type constant variable
operator punctuation property attribute`. Markdown faces: `text heading1
heading2 heading3 bold italic codespan codeblock frame link hashtag quote
frontmatter`.

## API surface

- `vibook.buf.get_text()` / `set_text(s)` / `get_cursor()`
- `vibook.vault.notes()` / `tags()` / `backlinks(path)` / `create(name)` /
  `open(name)`
- `vibook.open(target)` / `vibook.open_external(target)`
- `vibook.color(name, spec)`

## Building

```
cargo build --release        # static binary at target/release/v-ide
./install.sh                 # build + install to ~/.local/bin
cargo test                   # the tests/ directory is an immutable contract
```
