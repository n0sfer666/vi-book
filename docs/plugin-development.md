# v-ide plugin development

v-ide is configured and extended with **Lua**. This guide is the complete
reference for plugin authors: the runtime model, the full `vibook` API, every
event and option, the syntax-highlighting palette, and a line-by-line
walkthrough of the bundled `links` plugin.

> Russian translation: [`plugin-development.ru.md`](plugin-development.ru.md).

---

## 1. How plugins are loaded

On start-up v-ide builds one sandboxed Lua runtime and runs, in order:

1. The **prelude** — defines the `vibook` table and helpers.
2. The **default plugins** bundled into the binary (currently `links`; see §8).
3. Your **user config**: `~/.config/v-ide/init.lua`
   (override the path with the `V_IDE_CONFIG` environment variable; the legacy
   `VIBOOK_CONFIG` and `~/.config/vi-book/init.lua` still work, with a warning).

Because your config runs *after* the default plugins, you can change their
behaviour through options (for example disable a default plugin) and add your
own commands, keymaps, palette colours and event handlers on top.

A configuration file is just a Lua script. There is no separate manifest: call
the `vibook.*` functions described below and they take effect immediately.

```lua
-- ~/.config/v-ide/init.lua
vibook.opt.leader = " "
vibook.command("hello", function() vibook.buf.set_text("hello!\n") end)
```

---

## 2. The sandbox

The runtime is deliberately small and safe:

- **Available** standard libraries: `string`, `table`, `math`, `utf8`.
- **Removed** by default: `io`, `os`, `debug`, `package`, and the loaders
  `dofile`, `loadfile`, `load`/`loadstring`. There is no network access.
- `print` and `warn` are `nil` — they would corrupt the TUI. Use the editor
  buffer or commands to surface information instead.
- A plugin error is reported in the status line; it **never** crashes v-ide.
- Long-running scripts are interrupted by an instruction budget, so an
  accidental infinite loop cannot freeze the event loop.

If a future capability needs the filesystem or a process, v-ide exposes a
*narrow, purpose-built* function for it (for example `vibook.open_external`,
§6) rather than handing a plugin raw `os.execute`.

---

## 3. Options — `vibook.opt`

| Option | Type | Default | Meaning |
|--------|------|---------|---------|
| `leader` | string | `" "` | Key that opens the which-key menu. |
| `insert_escape` | string | `"jj"` | Type this in edit mode to return to normal; `""` disables. |
| `menu_position` | string | `"bottom"` | `"bottom"` bar or `"center"` modal. |
| `code_editor` | string | `""` | External editor for code links; falls back to `$EDITOR`. |
| `plugins` | table | `{}` | Per-plugin toggles, e.g. `{ links = false }`. |

```lua
vibook.opt.leader = ","
vibook.opt.insert_escape = "jk"
vibook.opt.menu_position = "center"
vibook.opt.code_editor = "nvim"
vibook.opt.plugins = { links = true }
```

---

## 4. Key mappings — `vibook.keymap(mode, lhs, rhs)`

Normal-mode (`"n"`) mappings. The right-hand side is either a `:command` or a
sequence of keys fed to the editor. Built-in keys take precedence.

```lua
vibook.keymap("n", "<C-s>", ":w")   -- Ctrl-s writes
vibook.keymap("n", "H", "0")        -- H jumps to the line start
vibook.keymap("n", "Y", "y$")       -- Y yanks to end of line
```

A left-hand side is one key, with or without modifiers, in vim notation:
`<C-x>` (Ctrl), `<M-x>` or `<A-x>` (Alt), `<S-Tab>` (Shift), `<D-x>` (Super/Cmd),
and named keys `<Esc> <CR> <BS> <Tab> <Del> <Insert> <Home> <End> <PageUp>
<PageDown> <Up> <Down> <Left> <Right> <F1>`–`<F12> <Space> <lt> <Bar> <Bslash>`.
Names are case-insensitive; an unknown `<…>` is read as plain characters.
Control characters in a Lua string are keys too: `"\t"` is `<Tab>`, `"\27"` is
`<Esc>`, `"\1"` is `<C-a>`. Multi-key sequences such as `gh` are not applied yet.

A right-hand side of exactly `<Nop>` (any case) makes the key do nothing.

These normal-mode keys are taken by built-ins and cannot be remapped yet:
`<F1>`, the leader (`<Space>` by default), `?`, `:`, `<C-w>`, `<CR>` (follow a
link), `<C-p>` (vault picker), `<C-o>` and `<Tab>` (jump list, notebook).

---

## 5. Menu, commands, events

### Menu entries — `vibook.menu_item(key, label, rhs)`

```lua
vibook.menu_item("g", "Open scratch", ":new scratch")
```

### Commands — `vibook.command(name, fn)`

Invoked with `:name`. Inside, read and rewrite the active buffer.

```lua
vibook.command("upper", function()
  vibook.buf.set_text(vibook.buf.get_text():upper())
end)

vibook.command("wc", function()
  local n = select(2, vibook.buf.get_text():gsub("%S+", ""))
  vibook.vault.create("wordcount-" .. n) -- surface the count as a note name
end)
```

### Events — `vibook.on(event, fn)`

| Event | Argument | Fired when |
|-------|----------|------------|
| `note_open` | — | a note has just been opened |
| `note_save` | — | a note has just been written |
| `follow_link` | `target` (string) | the user follows a `[[link]]` |

```lua
vibook.on("note_open", function()
  -- e.g. normalise the buffer on open
end)

vibook.on("follow_link", function(target)
  -- decide how a link target should open (see §6 and §9)
end)
```

Multiple handlers for the same event all run, in registration order.

---

## 6. Opening notes and files

These functions express an *intent*; v-ide performs the action after the
handler returns.

- `vibook.open(target)` — open `target` inside v-ide (a note path such as
  `"ideas.md"` or a subfolder path `"proj/spec.md"`).
- `vibook.open_external(target)` — open `target` in the external editor
  (`vibook.opt.code_editor`, else `$EDITOR`). v-ide resolves the file inside
  the vault (by relative path, otherwise by file name) and suspends the TUI
  while the editor runs.
- `vibook.vault.create(name)` — create a new note `name`.
- `vibook.vault.open(name)` — alias of `vibook.open` kept for symmetry with the
  other `vault.*` calls.

---

## 7. Reading state

### Buffer — `vibook.buf`

- `vibook.buf.get_text()` → the active buffer as a string.
- `vibook.buf.set_text(s)` → replace the active buffer.
- `vibook.buf.get_cursor()` → the cursor's character index.

### Vault — `vibook.vault`

- `vibook.vault.notes()` → list of note paths in the vault.
- `vibook.vault.tags()` → list of `#tags` seen across the vault.
- `vibook.vault.backlinks(path)` → notes that link to `path`.

The *vault* is the active folder v-ide was opened on; tag and link search are
scoped to it.

The vault is scanned on the first query (or picker, link follow, tree), not at
startup. If the scan fails, these functions raise a Lua error `vault: <reason>`
and the next call retries.

Saving or creating a note in v-ide updates these lists by rereading only that
note. Notes added, changed, removed or renamed outside v-ide are picked up the
same way within a fraction of a second; renaming or removing a folder rereads
only the notes inside it. If a changed note cannot be read, its previous lists
stay and the status line shows `vault: <note>: <reason>`. If the file watcher
cannot start, the status line shows `vault: watch: <reason>` once and outside
changes are not picked up; on Linux this usually means the inotify limit
`fs.inotify.max_user_watches` is too low for the vault. If the limit is reached
later, as the vault grows, the status line shows `vault: watch: <reason>` once
and folders created after that are not watched; everything else keeps working.
If the watcher reports lost events or an error, the vault is rescanned in full and
the error is shown as `vault: watch: <reason>`. If a note cannot be read during
that rescan, the lists of the whole vault stay as they were, the status line shows
`vault: <reason>` once, and the rescan is retried on the next change to any file
in the vault, so fixing the note brings the lists up to date.

---

## 8. Syntax-highlighting palette — `vibook.color(name, spec)`

Code blocks are highlighted with tree-sitter grammars installed by
`v-ide grammar fetch` and `v-ide grammar build` (rust, python, javascript, typescript,
tsx, json, bash, go, c, toml, html, yaml and common aliases; see the manual).
Recolour any token — or any markdown face — from your config.

`spec` fields: `fg` (colour), `bold`, `italic`, `underline`, `dim`. `fg`
accepts terminal colour names (`"red"`, `"magenta"`, `"cyan"`, …) and hex
(`"#82aaff"`).

```lua
vibook.color("keyword",  { fg = "magenta", bold = true })
vibook.color("string",   { fg = "green" })
vibook.color("number",   { fg = "yellow" })
vibook.color("comment",  { fg = "#6a737d", italic = true })
vibook.color("function", { fg = "blue" })
vibook.color("type",     { fg = "cyan" })

-- markdown faces are themeable too:
vibook.color("heading1", { fg = "magenta", bold = true })
vibook.color("link",     { fg = "blue", underline = true })
```

**Code token names:** `keyword`, `string`, `number`, `comment`, `function`,
`type`, `constant`, `variable`, `operator`, `punctuation`, `property`,
`attribute`.

**Markdown face names:** `text`, `heading1`, `heading2`, `heading3`, `bold`,
`italic`, `codespan`, `codeblock`, `frame`, `link`, `hashtag`, `quote`,
`frontmatter`.

> v-ide is a terminal app: it cannot choose your *font*. For ligatures and
> glyphs, set your terminal to a Nerd Font (the installer offers
> JetBrainsMono Nerd Font).

---

## 9. Worked example — the `links` plugin

This is the default plugin shipped in the binary
(`crates/lua/assets/plugins/links.lua`). It decides how the target of a followed
`[[link]]` opens: text notes in v-ide, code files in your editor, bare names
left to v-ide's own resolver.

```lua
local text_ext = {
  md = true, markdown = true, txt = true, text = true, org = true, rst = true,
}

local function disabled()
  return vibook.opt.plugins ~= nil and vibook.opt.plugins.links == false
end

local function extension(target)
  return target:match("%.([%w]+)$")
end

vibook.on("follow_link", function(target)
  if disabled() then
    return
  end
  local ext = extension(target)
  if ext == nil then
    return                      -- [[note]] → v-ide resolves aliases/anchors
  end
  if text_ext[ext:lower()] then
    vibook.open(target)         -- [[notes.md]] → open inside v-ide
  else
    vibook.open_external(target) -- [[main.rs]] → external editor
  end
end)
```

How it works:

1. `vibook.on("follow_link", …)` registers a handler that receives the link
   target string (everything inside `[[ ]]`, minus any `#anchor`/`|alias`).
2. `disabled()` honours `vibook.opt.plugins.links = false`, so a user can turn
   the plugin off without removing it.
3. A target **without** an extension returns early. The handler produced no
   open intent, so v-ide falls back to its built-in resolver (which handles
   name collisions, aliases and anchors).
4. A **text** extension calls `vibook.open` — v-ide opens it internally.
5. Any other extension calls `vibook.open_external` — v-ide finds the file in
   the vault and hands it to your editor.

### Make your own variant

Disable the default and register your own routing — e.g. open images with a
viewer:

```lua
vibook.opt.plugins = { links = false }

vibook.on("follow_link", function(target)
  local ext = (target:match("%.([%w]+)$") or ""):lower()
  if ext == "png" or ext == "jpg" then
    vibook.open_external(target)   -- with code_editor set to an image viewer
  elseif ext == "md" or ext == "" then
    vibook.open(target)
  else
    vibook.open_external(target)
  end
end)
```

---

## 10. Building and testing

```
cargo build --release   # one static binary at target/release/v-ide
./install.sh            # build + install + seed config + fetch a Nerd Font
cargo test              # the tests/ directory is an immutable contract
```

When you add a new capability to the runtime, cover it with a test under
`tests/` (load a script into `PluginHost`, then assert the observable effect),
matching the existing `lua_*` test files.
