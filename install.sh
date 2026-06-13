#!/usr/bin/env sh
# vi-book installer — builds a release binary and puts `vi-book` on your PATH.
# Works from any shell (bash, zsh, fish, nu); only POSIX sh is used here.
set -eu

REPO_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
BIN_NAME="vi-book"

# Pick an install dir: honour $PREFIX/$VIBOOK_BIN, else ~/.local/bin.
BIN_DIR="${VIBOOK_BIN:-${PREFIX:+$PREFIX/bin}}"
BIN_DIR="${BIN_DIR:-$HOME/.local/bin}"

echo "vi-book: building release binary..."
( cd "$REPO_DIR" && cargo build --release )

mkdir -p "$BIN_DIR"
install -m 0755 "$REPO_DIR/target/release/$BIN_NAME" "$BIN_DIR/$BIN_NAME"
echo "vi-book: installed to $BIN_DIR/$BIN_NAME"

# Seed a default config so no environment variable is ever required.
CONFIG_DIR="$HOME/.config/vi-book"
CONFIG_FILE="$CONFIG_DIR/init.lua"
if [ ! -e "$CONFIG_FILE" ]; then
	mkdir -p "$CONFIG_DIR"
	cat >"$CONFIG_FILE" <<'LUA'
-- vi-book configuration (Lua). See docs/developers.md for the full API.
vibook.opt.leader = " "          -- key that opens the menu
vibook.opt.insert_escape = "jj"  -- type in edit mode to leave it ("" disables)
vibook.opt.menu_position = "bottom" -- "bottom" or "center"
LUA
	echo "vi-book: wrote default config to $CONFIG_FILE"
fi

# PATH hint, per shell family.
case ":$PATH:" in
*":$BIN_DIR:"*) ;;
*)
	echo
	echo "vi-book: $BIN_DIR is not on your PATH. Add it:"
	echo "  bash/zsh : echo 'export PATH=\"$BIN_DIR:\$PATH\"' >> ~/.profile"
	echo "  fish     : fish_add_path $BIN_DIR"
	echo "  nushell  : \$env.PATH = (\$env.PATH | prepend '$BIN_DIR')"
	;;
esac

echo
echo "Done. Run 'vi-book' inside a folder of .md notes, or 'vi-book ~/notes'."
