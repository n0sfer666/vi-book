# vi-book — developer / plugin guide

vi-book is configured and extended with **Lua**, loaded from
`~/.config/vi-book/init.lua` (override with the `VIBOOK_CONFIG` env var). The
Lua runtime is sandboxed: no `io`/`os`/network unless explicitly granted, and a
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

Normal-mode mappings. The right-hand side is either a `:command` or a sequence
of keys fed to the editor.

```lua
vibook.keymap("n", "<C-s>", ":w")     -- Ctrl-s writes
vibook.keymap("n", "H", "0")          -- H jumps to line start
vibook.keymap("n", "<leader>x", ...)  -- (multi-key maps are not applied yet)
```

Built-in `jj` → `<Esc>` is provided by `vibook.opt.insert_escape`; set it to
`"jk"` to use that instead, or `""` to turn it off.

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
cargo build --release        # static binary at target/release/vi-book
./install.sh                 # build + install to ~/.local/bin
cargo test                   # the tests/ directory is an immutable contract
```
