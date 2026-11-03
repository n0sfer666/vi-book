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
The native Lua host and synchronous Ex context share these selectors; application
event integration remains pending.

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
The native autocmd frontends share this context; connecting the application
driver and lifecycle events remains a subsequent integration step.

## Autocmd execution and payloads

`editor::autocmd::Bus` owns the registry and compiled selectors. It freezes matching
record handles before callbacks, consumes `once` before invocation and skips
removed records. A truthy callback removes only its current expanded record.
Automatic nested dispatch obeys `nested`; explicit dispatch bypasses that flag.
Nesting stops at ten levels with E218. Errors become context diagnostics; API
dispatch continues, while Ex dispatch stops at the first failing handler. Ex boundaries retain an already reported error without
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
dictionaries and opaque VM function references have distinct variants. The VM owns
function values and must resolve and validate their runtime/function IDs while a
payload is dispatched. The native Lua host retains these owners through dispatch;
application event integration remains pending.

## Lua payload conversion

`lua::data::Codec` converts values from its Lua instance into the core data arena
and back. Keep the original Lua instance alive and supply handles from that same
instance, as required by mlua. Conversion uses explicit task stacks; input table
identities remain rooted while shared children are memoized. Exports create a
fresh table per reference, so input aliases and arbitrary metatables are not
retained. Empty dictionary markers are preserved, sparse arrays receive null
holes, strings/keys retain bytes and integral floats use native integer conversion.
Cycles, mixed/invalid keys, threads and non-null userdata are rejected.

Conversions share a budget of 1,000,000 expanded values and 64 MiB of string and
dictionary-key bytes per outer runtime invocation, including nested callbacks.
Containers, sparse-array null holes and every repeated subtree count toward the
expanded cost. Checks reserve queued children and byte copies before allocation;
exports preflight the shared arena before creating Lua values. Exceeding either
limit returns a data-conversion error. Caught errors retain the spent budget, and
an exceeded budget rejects further conversions until the next outer invocation.
Each standalone codec operation outside a runtime invocation gets a fresh budget.

Keep the returned `Encoded` owner alive while dispatching its data. It retains Lua
functions; the codec's lookup pool holds weak references and periodically removes
expired entries. Function IDs from another codec runtime or an expired owner fail
validation. A function already exported to Lua remains independently usable after
the owner is dropped. The codec installs `NIL` and `empty_dict` on a supplied API
table; null uses mlua's shared null sentinel. These helpers are not yet installed
in the application's legacy host.

`scripts/capture_autocmd_data.lua` generates the native Neovim 0.12.6 data cases in
`crates/lua/tests/fixtures/data_native.rs`. Use the same isolated HOME/XDG/log setup
as the other captures and verify that repeated capture is byte-identical.

## Native Lua autocmd bridge

`PluginHost::new_with_bus(Rc<editor::autocmd::Bus>)` enables the direct
`vibook.create_autocmd`, `del_autocmd`, `create_augroup`, `del_augroup_by_id`,
`del_augroup_by_name`, `get_autocmds`, `clear_autocmds` and `exec_autocmds` APIs. The default host
constructor retains the legacy API. The application module still uses that
constructor for standalone embedders. The application enables the native bus and
runs owned module actions outside the module registry borrow. Automatic lifecycle
and frequent event emitters remain a subsequent integration step.

Definitions accept native event strings/lists, groups, patterns or a buffer,
commands or Lua/named callbacks, and once/nested/description attributes. Listings
preserve callback identity and native metadata, including buffer aliases. Filters
retain the native differences between omitted groups in get and clear, empty
arrays, literal query patterns and comma-separated definition patterns. A failed
registration retains its valid prefix. Unsupported regexp insertions still fail
explicitly.

Buffer addressing and buffer-local listings require `load_with_context`; the bus
callback adapter establishes this context automatically. Public wrappers remain usable across scopes.
Callbacks receive id/group/event/match/file/buf/data, copy typed payloads, preserve
Lua function identity, and complete option writes before the caller continues.
Named `v:lua` callbacks resolve globals at invocation, receive no arguments and ignore their return;
other Vimscript names report E117 at invocation. A Lua callback's truthy return
removes its current expanded record.

The host owns the bus and API state; bus callbacks own the state, whose bus link
is weak. VM functions capture weak state/bus links. Scope setup and rollback pause
GC while replacing backends, enter the shared budgets before Lua allocation, and
preserve scripts' GC running/stopped choices. Script compilation precedes backend
publication. The native assertions in `scripts/check_autocmd_definitions.lua`
exercise the same contract chunks used by Rust tests; run them with headless
Neovim and isolated HOME/XDG/log paths under `target`.

## Explicit Lua event execution and modelines

The opt-in native host also provides `vibook.exec_autocmds(events, opts)`.
It dispatches inline through the same bus and current context, supports group,
pattern lists/commas, buffer/buf, modeline and typed data, and retains payload
function owners through nested dispatch. Callback errors are diagnostics and do
not abort subsequent records. Invalid later event names preserve the executed
prefix and return an API error. Public calls require a live context. Explicit
buffer execution supplies that buffer's event identity without changing the
current buffer.

`modeline` defaults to true for this API. If at least one handler ran and the
whole event list was valid, modelines are processed once afterward, against the
current buffer. Callback errors do not skip this step. The buffer-local
`modeline`/`ml` option and global `modelines`/`mls` option control processing;
defaults are on (off for root on Unix) and five lines. Zero or negative scan
counts disable scanning. The initial count selects the first lines from top to bottom, then the last
lines from bottom to top without duplicates. Processing cannot reenter through
an option callback.

The parser supports vi/vim/Vim/ex markers, native version comparisons against
801 (the Neovim reference's Vim version), colon-separated settings and terminated
`set`/`se` forms. It applies the editor's supported options with local scope and
OptionSet notifications. Unknown options and invalid settings produce diagnostics,
preserve the valid prefix and stop subsequent modelines. Modelines execute no Ex
commands; list/reset-all commands are rejected. This does not implement options
outside the current option registry or enable automatic processing on file reads.

File-event callback matches use absolute names; User, FileType and the other
native non-file events retain literal matches. `capture_exec_names.lua` captures
all supported event classifications, and `capture_modelines.lua` captures the
modeline matrix. `check_autocmd_exec.lua` runs the same payload/order/modeline
assertions used by Rust tests. Run captures with the isolated headless Neovim
setup described above; generated expectations must remain reproducible.

## Synchronous Ex frontend

A context whose observer exposes the shared autocmd bus accepts `autocmd`,
`augroup` and `doautocmd`, including their normal abbreviations. For example,
construct it with `Context::with_observer(&mut workspace, &bus)`. Registration,
queries and explicit dispatch share the Lua API's registry. Ex records have no
API id. The bus retains the selected Ex group across calls. Ex group deletion
keeps existing handlers under `--Deleted--`; API deletion removes handlers, and
both reject deletion of the current group. Clear operations use the selected
group by default; unqualified listings and explicit dispatch include all groups.

Ex validates the entire event list before mutating or dispatching. Definitions
support ordinary patterns, buffer selectors, `++once`, `++nested` and legacy
`nested`. Bang replacement works per expanded pattern. A definition consumes the
rest of its line, including bars; other supported commands accept bar/newline
chains and comments. Long runs of comments are parsed iteratively. Listings
provide event/group/pattern/action content; script provenance and native Lua
function display numbers are not modeled.

Ex dispatch stops at the first failing handler and returns an already reported
error. API dispatch keeps its existing continue-and-diagnose behavior. OptionSet
follows the active Ex boundary while retaining automatic nesting restrictions.
`doautocmd <nomodeline>` skips the shared modeline phase; otherwise modelines run
after matching handlers, including after a handler failure.

`vibook.cmd(string)` executes the supported commands inline in a live context.
The existing `vibook.buf` text/cursor accessors also follow that context, including
saved accessor functions and nested callbacks. Outside a context they retain
the legacy snapshot behavior. Entry rejects pending legacy text writes, and
scope restoration preserves completed writes and current-buffer changes on error.

Core services include `set`, `enew[!]`, numeric `buffer[!]`, file naming,
`edit[!]` and `write[!]`. They retain canonical documents and existing undo,
readonly and external-conflict behavior. New file targets use exclusive creation
unless forced. `edit!` preserves a loaded hidden buffer and reloads only the current
buffer. IO completes before the next Lua statement. Filename expansions, writing a different named buffer's contents to
another path, ranges and a full Vimscript command language remain S11 work;
unsupported forms fail explicitly. Force-write filesystem limitations described
above remain unchanged. The application runs configuration, plugin commands and legacy hooks with owned
Lua hosts and a live context. Explicit events and OptionSet share its native bus.
The application Ex adapter accepts these frontends, and current-buffer changes
restore the window view. Standalone module callers retain their request-based
interface. Automatic lifecycle, read/write and frequent event emitters are the
next integration step.
