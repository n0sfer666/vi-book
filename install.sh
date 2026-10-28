#!/usr/bin/env sh
# v-ide installer — builds a release binary and puts `v-ide` on your PATH.
# Works from any shell (bash, zsh, fish, nu); only POSIX sh is used here.
set -eu

REPO_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
BIN_NAME="v-ide"

# Install a Nerd Font so code blocks and glyphs render well in the terminal.
# Idempotent, offline-safe, and skippable with V_IDE_NO_FONT=1 (legacy: VIBOOK_NO_FONT=1).
install_font() {
	case "$(uname -s)" in
	Darwin) FONT_DIR="$HOME/Library/Fonts" ;;
	*) FONT_DIR="$HOME/.local/share/fonts" ;;
	esac
	if ls "$FONT_DIR" 2>/dev/null | grep -qi "JetBrainsMono.*Nerd"; then
		echo "v-ide: JetBrainsMono Nerd Font already present"
		return 0
	fi
	if command -v curl >/dev/null 2>&1; then
		DL="curl -fLsS -o"
	elif command -v wget >/dev/null 2>&1; then
		DL="wget -qO"
	else
		echo "v-ide: no curl/wget; skipping font install (set a Nerd Font yourself)"
		return 0
	fi
	if ! command -v unzip >/dev/null 2>&1; then
		echo "v-ide: no unzip; skipping font install"
		return 0
	fi
	URL="https://github.com/ryanoasis/nerd-fonts/releases/latest/download/JetBrainsMono.zip"
	TMP=$(mktemp -d) || return 0
	echo "v-ide: downloading JetBrainsMono Nerd Font..."
	if $DL "$TMP/font.zip" "$URL" 2>/dev/null; then
		mkdir -p "$FONT_DIR"
		unzip -oq "$TMP/font.zip" '*.ttf' -d "$FONT_DIR" 2>/dev/null ||
			unzip -oq "$TMP/font.zip" -d "$FONT_DIR" 2>/dev/null || true
		command -v fc-cache >/dev/null 2>&1 && fc-cache -f "$FONT_DIR" >/dev/null 2>&1 || true
		echo "v-ide: installed JetBrainsMono Nerd Font to $FONT_DIR"
		echo "v-ide: set your terminal font to 'JetBrainsMono Nerd Font'"
	else
		echo "v-ide: font download failed (offline?); skipping"
	fi
	rm -rf "$TMP"
}

# Pick an install dir: honour $V_IDE_BIN (legacy: $VIBOOK_BIN)/$PREFIX, else ~/.local/bin.
BIN_DIR="${V_IDE_BIN:-${VIBOOK_BIN:-${PREFIX:+$PREFIX/bin}}}"
BIN_DIR="${BIN_DIR:-$HOME/.local/bin}"

echo "v-ide: building release binary..."
( cd "$REPO_DIR" && cargo build --release )

mkdir -p "$BIN_DIR"
install -m 0755 "$REPO_DIR/target/release/$BIN_NAME" "$BIN_DIR/$BIN_NAME"
echo "v-ide: installed to $BIN_DIR/$BIN_NAME"
if [ -e "$BIN_DIR/vi-book" ]; then
	echo "v-ide: old binary $BIN_DIR/vi-book found; remove it with: rm \"$BIN_DIR/vi-book\""
fi

# Seed a default config so no environment variable is ever required.
CONFIG_DIR="$HOME/.config/v-ide"
CONFIG_FILE="$CONFIG_DIR/init.lua"
OLD_CONFIG_DIR="$HOME/.config/vi-book"
if [ ! -e "$CONFIG_FILE" ] && [ -e "$OLD_CONFIG_DIR/init.lua" ]; then
	if [ -d "$CONFIG_DIR" ]; then
		echo "v-ide: old config found; move it with: mv \"$OLD_CONFIG_DIR/init.lua\" \"$CONFIG_FILE\""
	else
		echo "v-ide: old config found; move it with: mv \"$OLD_CONFIG_DIR\" \"$CONFIG_DIR\""
	fi
elif [ ! -e "$CONFIG_FILE" ]; then
	mkdir -p "$CONFIG_DIR"
	cat >"$CONFIG_FILE" <<'LUA'
-- v-ide configuration (Lua). See docs/developers.md for the full API.
vibook.opt.leader = " "          -- key that opens the menu
vibook.opt.insert_escape = "jj"  -- type in edit mode to leave it ("" disables)
vibook.opt.menu_position = "bottom" -- "bottom" or "center"
vibook.opt.code_editor = ""      -- editor for code links, e.g. "nvim" (falls back to $EDITOR)

-- Syntax highlighting palette (see docs/plugin-development.md):
-- vibook.color("keyword", { fg = "magenta", bold = true })
-- vibook.color("string", { fg = "green" })
LUA
	echo "v-ide: wrote default config to $CONFIG_FILE"
fi

if [ -z "${V_IDE_NO_FONT:-${VIBOOK_NO_FONT:-}}" ]; then
	install_font || true
fi

if [ -z "${V_IDE_NO_GRAMMARS:-}" ]; then
	echo "v-ide: fetching and building code-block grammars..."
	GRAMMARS_OK=1
	"$BIN_DIR/$BIN_NAME" grammar fetch || GRAMMARS_OK=0
	"$BIN_DIR/$BIN_NAME" grammar build || GRAMMARS_OK=0
	if [ "$GRAMMARS_OK" = 1 ]; then
		echo "v-ide: grammars installed"
	else
		echo "v-ide: grammars not fully installed; retry later with: v-ide grammar fetch; v-ide grammar build"
	fi
fi

# PATH hint, per shell family.
case ":$PATH:" in
*":$BIN_DIR:"*) ;;
*)
	echo
	echo "v-ide: $BIN_DIR is not on your PATH. Add it:"
	echo "  bash/zsh : echo 'export PATH=\"$BIN_DIR:\$PATH\"' >> ~/.profile"
	echo "  fish     : fish_add_path $BIN_DIR"
	echo "  nushell  : \$env.PATH = (\$env.PATH | prepend '$BIN_DIR')"
	;;
esac

echo
echo "Done. Run 'v-ide' inside a folder of .md notes, or 'v-ide ~/notes'."
