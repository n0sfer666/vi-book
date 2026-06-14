#!/usr/bin/env sh
# vi-book installer — builds a release binary and puts `vi-book` on your PATH.
# Works from any shell (bash, zsh, fish, nu); only POSIX sh is used here.
set -eu

REPO_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
BIN_NAME="vi-book"

# Install a Nerd Font so code blocks and glyphs render well in the terminal.
# Idempotent, offline-safe, and skippable with VIBOOK_NO_FONT=1.
install_font() {
	case "$(uname -s)" in
	Darwin) FONT_DIR="$HOME/Library/Fonts" ;;
	*) FONT_DIR="$HOME/.local/share/fonts" ;;
	esac
	if ls "$FONT_DIR" 2>/dev/null | grep -qi "JetBrainsMono.*Nerd"; then
		echo "vi-book: JetBrainsMono Nerd Font already present"
		return 0
	fi
	if command -v curl >/dev/null 2>&1; then
		DL="curl -fLsS -o"
	elif command -v wget >/dev/null 2>&1; then
		DL="wget -qO"
	else
		echo "vi-book: no curl/wget; skipping font install (set a Nerd Font yourself)"
		return 0
	fi
	if ! command -v unzip >/dev/null 2>&1; then
		echo "vi-book: no unzip; skipping font install"
		return 0
	fi
	URL="https://github.com/ryanoasis/nerd-fonts/releases/latest/download/JetBrainsMono.zip"
	TMP=$(mktemp -d) || return 0
	echo "vi-book: downloading JetBrainsMono Nerd Font..."
	if $DL "$TMP/font.zip" "$URL" 2>/dev/null; then
		mkdir -p "$FONT_DIR"
		unzip -oq "$TMP/font.zip" '*.ttf' -d "$FONT_DIR" 2>/dev/null ||
			unzip -oq "$TMP/font.zip" -d "$FONT_DIR" 2>/dev/null || true
		command -v fc-cache >/dev/null 2>&1 && fc-cache -f "$FONT_DIR" >/dev/null 2>&1 || true
		echo "vi-book: installed JetBrainsMono Nerd Font to $FONT_DIR"
		echo "vi-book: set your terminal font to 'JetBrainsMono Nerd Font'"
	else
		echo "vi-book: font download failed (offline?); skipping"
	fi
	rm -rf "$TMP"
}

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
vibook.opt.code_editor = ""      -- editor for code links, e.g. "nvim" (falls back to $EDITOR)

-- Syntax highlighting palette (see docs/plugin-development.md):
-- vibook.color("keyword", { fg = "magenta", bold = true })
-- vibook.color("string", { fg = "green" })
LUA
	echo "vi-book: wrote default config to $CONFIG_FILE"
fi

if [ -z "${VIBOOK_NO_FONT:-}" ]; then
	install_font || true
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
