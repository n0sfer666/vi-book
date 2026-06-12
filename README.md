# vi-book

A terminal note-taking app with vim-style editing — "Obsidian for the terminal".

Status: early development. The editor core (rope buffer, vim modal layer,
undo/redo) is being built first as a library; the TUI, markdown rendering,
vaults with `[[links]]` and Lua plugins follow.

## Building

```
cargo build --release
```

## License

MIT OR Apache-2.0
