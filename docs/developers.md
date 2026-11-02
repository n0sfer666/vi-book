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
vibook.opt.leader = " "          -- sequence that opens the menu (default: space)
vibook.opt.insert_escape = "jj"  -- type this in edit mode to leave it (default: "jj"; "" disables)
vibook.opt.menu_position = "bottom" -- "bottom" (which-key bar) or "center" (modal)
vibook.opt.code_editor = "nvim"  -- external editor for code links (falls back to $EDITOR)
vibook.opt.plugins = { links = false } -- toggle a default plugin off
vibook.opt.number = true
vibook.opt.read_number = false
vibook.o.timeoutlen = 400
local number = vibook.opt.number:get()
```

Core options also have current-context scalar `o/go/bo/wo` and object
`opt/opt_local/opt_global` accessors. The default configuration shows numbers
in Edit and hides them in Read; the two settings are independent.

## Key mappings — `vibook.keymap`

Mappings accept modes `n`, `i`, `x`/`v`, `o` and `c`. The right-hand side is
a sequence of keys, `<Cmd>command<CR>` segments, or a mixture of both.
`<Cmd>` executes without leaving the current mode. A leading `:` is ordinary
input; add `<CR>` to submit the command. Lua mappings also accept a mode list
and boolean `remap`, `nowait`, `unique` and `silent` options.

```lua
vibook.keymap("n", "<C-s>", "<Cmd>w<CR>")     -- Ctrl-s writes
vibook.keymap("n", "H", "0")          -- H jumps to line start
vibook.keymap("n", "<leader>x", "dd") -- leader, then x deletes the line
vibook.keymap("n", "Q", "j<Cmd>only<CR>k") -- keys around a command
```

Built-in `jj` → `<Esc>` is an insert-mode mapping from `vibook.opt.insert_escape`;
set it to `"jk"` to use that instead, or `""` to turn it off.

Use `:map` and its mode-prefixed forms to inspect mappings at runtime.
Definitions covering several modes are displayed as a group. Errors and
other application messages are available through `:messages` (last 1000
nonempty entries); `:messages clear` clears the history. Multiline output
uses the message pager described in the manual.

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


## Buffer ownership

`Workspace` owns one `Document` per `DocId`. Windows keep buffer handles and
cursor history; the hidden-buffer pool remains global and ordered by recent
use. `current()` and `current_mut()` access the active document. `windows()`
and `pool()` return borrowed document projections. Closing a window preserves
a document referenced elsewhere; `forget()` refuses visible or pooled buffers.
Text, undo and saved-file state are shared. Cursor bookmarks retain semantic
line adjustments through edits and undo, while desired columns stay local to
the view. UI scroll history is keyed by window and buffer IDs.

## Autocmd pattern foundation

`editor::autocmd::patterns` lazily splits comma-separated definitions, retaining
escaped commas and nested brace groups. Compile each item with `Pattern::compile`
and match with `matches(short_name, full_name, ignore_case)`. Patterns without a
slash use the full name's tail; slash patterns try both supplied names. Callers
resolve paths and freeze matching record handles before invoking callbacks.

The ordinary glob grammar includes `*`, Unicode-scalar `?`, nested `{a,b}` groups,
empty alternatives, character ranges, `[^...]`, escapes and the twelve standard
POSIX named classes. `[!x]` contains a literal exclamation mark. An unclosed `[` is
literal; unmatched braces and reversed ranges are errors. Vim-regexp insertions,
control escapes, equivalence classes and Vim-specific named classes return an
explicit unsupported-pattern error. Regexp integration belongs to S18–S19.
The parser and finite-state matcher do not recurse or expand brace alternatives.

Native reference: Neovim 0.12.6. `crates/editor/tests/autocmd/capture_glob.lua` and
`capture_glob_unicode.lua` produce the Rust matrix fixtures. The latter exercises
Unicode case equivalences and named classes. `scripts/capture_autocmd_unicode.lua`
extracts scalar case-equivalence groups and lower/upper/printable ranges into
`pattern/unicode_data.rs`; that file is generated data, not handwritten code.
Run captures with headless Neovim (`-u NONE -i NONE -n -l`) from the repository,
with HOME, XDG directories and NVIM_LOG_FILE isolated under its `target` directory.
Repeat captures must produce identical files; do not edit expected data by hand.
This library foundation is not yet connected to Lua/Ex autocmd execution.

## Synchronous editor context

`editor::execution::Context` borrows the canonical workspace for one execution.
Its option accessors release the workspace borrow before invoking an `Observer`;
a callback can read current values and execute a nested `set` command inline.
Option notifications are nonrecursive, include old local/global values and fire
for same-value writes. Queries, invalid writes and `set all&` do not notify.
Autocmd errors preserve applied writes: direct option setters report them through
context messages, while Ex execution also returns an error after processing valid
arguments. The TUI `set` adapter delegates parsing and mutation to this context.

`PluginHost::load_with_context` temporarily connects existing Lua option accessors
to the live context. Public accessor objects remain stable; captured option values
remain snapshots. Backends are restored after success or error, and legacy host
calls outside a context retain their snapshot/pending-write behavior. A caller must
apply legacy pending option writes to their original target before entering a live
context; entry rejects unflushed writes without changing the workspace or queue.
Nested VM
entries share the outer instruction budget; a new outer invocation resets it.
The native autocmd registry/frontends and lifecycle event wiring are subsequent
integration steps; the current observer is the synchronous execution foundation.

## Autocmd execution and payloads

`editor::autocmd::Bus` owns the registry and compiled selectors. It freezes matching
record handles before callbacks, consumes `once` before invocation and skips
removed records. A truthy callback removes only its current expanded record.
Automatic nested dispatch obeys `nested`; explicit dispatch bypasses that flag.
Nesting stops at ten levels with E218. Errors become context diagnostics while
subsequent records continue. Ex boundaries retain an already reported error without
emitting it twice; an independent callback error still creates its own diagnostic.
The bus also implements the context's option observer;
explicit events inherit active option values without creating a new option frame.
Groups, filters and listings retain the registry's native metadata rules.

Registry mutations keep callback actions alive until mutable borrows are released,
including failed definitions and group/buffer cleanup. No workspace or registry
borrow remains across callbacks or their destructors. Case policy is supplied by
the host and applied when matching handles are frozen.

`DataBuilder` stores typed payload values in an immutable flat arena with checked,
backward-only edges. Deep values and shared children do not cause recursive drops.
Strings and dictionary keys retain bytes; scalar numbers, nulls, arrays,
dictionaries and opaque VM function identities have distinct variants. The VM owns
function values and must resolve and validate their runtime/function IDs while a
payload is dispatched. Native Lua conversion and the application event/frontends
remain integration work; this bus is currently a core library foundation.
