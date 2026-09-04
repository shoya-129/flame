#!/usr/bin/env bash
# ==============================================================================
# Flame & Blaze Toolchain Installer
# Cross-platform installer for Windows (PowerShell/WSL/Git Bash), Linux, and macOS
# ==============================================================================

set -e

BOLD="\033[1m"
GREEN="\033[1;32m"
BLUE="\033[1;34m"
YELLOW="\033[1;33m"
RED="\033[1;31m"
RESET="\033[0m"

echo -e "${BLUE}${BOLD}"
echo "  _    _      _ _             _____                 _                         "
echo " | |  | |    | | |           |  __ \               | |                        "
echo " | |__| | ___| | | ___       | |  | | _____   _____| | ___  _ __   ___ _ __   "
echo " |  __  |/ _ \ | |/ _ \      | |  | |/ _ \ \ / / _ \ |/ _ \| '_ \ / _ \ '__|  "
echo " | |  | |  __/ | | (_) |     | |__| |  __/\ V /  __/ | (_) | |_) |  __/ |     "
echo " |_|  |_|\___|_|_|\___/      |_____/ \___| \_/ \___|_|\___/| .__/ \___|_|     "
echo "                                                           | |                "
echo "                                                           |_|                "
echo -e "${RESET}"
echo -e "${BOLD}Installing Flame Language & Blaze Toolchain...${RESET}\n"

# 1. Detect Operating System and Environment
OS_TYPE="unknown"
IS_WSL=false

if grep -qi "microsoft" /proc/version 2>/dev/null || uname -r | grep -qi "microsoft"; then
    IS_WSL=true
fi

case "$(uname -s)" in
    Linux*)
        if [[ "$IS_WSL" == true ]]; then
            OS_TYPE="wsl"
        else
            OS_TYPE="linux"
        fi
        ;;
    Darwin*)    OS_TYPE="macos" ;;
    CYGWIN*|MINGW*|MSYS*) OS_TYPE="windows" ;;
    *)
        if [[ "$OS" == "Windows_NT" ]]; then
            OS_TYPE="windows"
        else
            OS_TYPE="unix"
        fi
        ;;
esac

# 2. Find Cargo command (support native cargo and Windows cargo.exe)
CARGO_CMD="cargo"
TARGET_IS_WINDOWS=false

if command -v cargo &> /dev/null; then
    CARGO_CMD="cargo"
    if [[ "$OS_TYPE" == "windows" ]]; then
        TARGET_IS_WINDOWS=true
    fi
elif command -v cargo.exe &> /dev/null; then
    CARGO_CMD="cargo.exe"
    TARGET_IS_WINDOWS=true
else
    echo -e "${RED}Error: Cargo is not installed or not in PATH.${RESET}"
    echo "Please install Rust and Cargo from https://rustup.rs/ before continuing."
    exit 1
fi

if [[ "$OS_TYPE" == "wsl" && "$CARGO_CMD" == "cargo.exe" ]]; then
    TARGET_IS_WINDOWS=true
    echo -e "Detected environment: ${GREEN}Windows (invoked via WSL bash)${RESET}"
else
    echo -e "Detected platform: ${GREEN}${OS_TYPE}${RESET}"
fi

CARGO_VERSION=$("$CARGO_CMD" --version)
echo -e "Rust/Cargo detected: ${GREEN}${CARGO_VERSION}${RESET}"

# Helper function to convert Windows paths to POSIX paths
to_posix_path() {
    local p="$1"
    if [[ -z "$p" ]]; then echo ""; return; fi
    if command -v wslpath &> /dev/null; then
        wslpath -u "$p" 2>/dev/null || echo "$p"
    elif command -v cygpath &> /dev/null; then
        cygpath -u "$p" 2>/dev/null || echo "$p"
    else
        echo "$p"
    fi
}

# 3. Locate Cargo bin directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]:-$0}" )" 2>/dev/null && pwd )"
if [[ -z "$SCRIPT_DIR" || ! -d "$SCRIPT_DIR" ]]; then
    SCRIPT_DIR="$(pwd)"
fi

CARGO_BIN=""

if [[ "$TARGET_IS_WINDOWS" == true ]]; then
    # Query USERPROFILE from Windows environment if needed
    WIN_USERPROFILE="$USERPROFILE"
    if [[ -z "$WIN_USERPROFILE" ]] && command -v powershell.exe &> /dev/null; then
        WIN_USERPROFILE="$(powershell.exe -NoProfile -Command '$env:USERPROFILE' 2>/dev/null | tr -d '\r')"
    fi
    if [[ -z "$WIN_USERPROFILE" ]] && command -v cmd.exe &> /dev/null; then
        WIN_USERPROFILE="$(cmd.exe /c "echo %USERPROFILE%" 2>/dev/null | tr -d '\r')"
    fi

    if [[ -n "$WIN_USERPROFILE" ]]; then
        CARGO_BIN="$(to_posix_path "$WIN_USERPROFILE/.cargo/bin")"
    else
        CARGO_BIN="$HOME/.cargo/bin"
    fi
else
    if [[ -n "$CARGO_HOME" ]]; then
        if [[ "$CARGO_HOME" == */bin ]]; then
            CARGO_BIN="$CARGO_HOME"
        else
            CARGO_BIN="$CARGO_HOME/bin"
        fi
    else
        CARGO_BIN="$HOME/.cargo/bin"
    fi
fi

mkdir -p "$CARGO_BIN" 2>/dev/null || true

# 4. Build and install fmp & flamelang binaries via Cargo
echo -e "\n${BOLD}[1/3] Building and installing Flame binaries (fmp & flamelang)...${RESET}"

if [[ -f "$SCRIPT_DIR/Cargo.toml" ]]; then
    echo -e "Installing from local repository at ${GREEN}$SCRIPT_DIR${RESET}..."
    if [[ "$CARGO_CMD" == "cargo.exe" && -n "$(command -v wslpath)" ]]; then
        WIN_BUILD_DIR="$(wslpath -w "$SCRIPT_DIR")"
        cargo.exe install --path "$WIN_BUILD_DIR" --force
    else
        "$CARGO_CMD" install --path "$SCRIPT_DIR" --force
    fi
else
    echo -e "Installing flamelang from Cargo registry (crates.io)..."
    if ! "$CARGO_CMD" install --force flamelang; then
        echo -e "${YELLOW}Registry install not yet available or failed; installing latest from Git repository...${RESET}"
        "$CARGO_CMD" install --git https://github.com/shoya-129/flame.git --force
    fi
fi

# Ensure fmp command executable exists and is linked
if [[ "$TARGET_IS_WINDOWS" == true ]]; then
    FLAMELANG_EXE="$CARGO_BIN/flamelang.exe"
    FMP_EXE="$CARGO_BIN/fmp.exe"

    if [[ -f "$FLAMELANG_EXE" ]]; then
        cp "$FLAMELANG_EXE" "$FMP_EXE" 2>/dev/null || true
    elif [[ -f "$FMP_EXE" ]]; then
        cp "$FMP_EXE" "$FLAMELANG_EXE" 2>/dev/null || true
    fi

    # Create batch and cmd shims for Windows command prompt and powershell
    cat << 'EOF' > "$CARGO_BIN/fmp.cmd"
@"%~dp0fmp.exe" %*
EOF
    cat << 'EOF' > "$CARGO_BIN/fmp.bat"
@"%~dp0fmp.exe" %*
EOF
    chmod +x "$CARGO_BIN/fmp.cmd" "$CARGO_BIN/fmp.bat" 2>/dev/null || true
else
    # Linux / macOS symlinks
    if [[ -f "$CARGO_BIN/flamelang" ]]; then
        ln -sf "$CARGO_BIN/flamelang" "$CARGO_BIN/fmp"
    elif [[ -f "$CARGO_BIN/fmp" ]]; then
        ln -sf "$CARGO_BIN/fmp" "$CARGO_BIN/flamelang"
    fi
fi

# 5. Determine and setup Blaze definition directories
echo -e "\n${BOLD}[2/3] Setting up Blaze standard library definition directory...${RESET}"

TARGET_DIRS=()

if [[ "$TARGET_IS_WINDOWS" == true ]]; then
    WIN_LOCALAPPDATA="$LOCALAPPDATA"
    if [[ -z "$WIN_LOCALAPPDATA" ]] && command -v powershell.exe &> /dev/null; then
        WIN_LOCALAPPDATA="$(powershell.exe -NoProfile -Command '$env:LOCALAPPDATA' 2>/dev/null | tr -d '\r')"
    fi
    WIN_PROGRAMFILES="$PROGRAMFILES"
    if [[ -z "$WIN_PROGRAMFILES" ]] && command -v powershell.exe &> /dev/null; then
        WIN_PROGRAMFILES="$(powershell.exe -NoProfile -Command '$env:ProgramFiles' 2>/dev/null | tr -d '\r')"
    fi

    if [[ -n "$WIN_PROGRAMFILES" ]]; then
        P_DIR="$(to_posix_path "$WIN_PROGRAMFILES/Blaze/std")"
        if [[ -n "$P_DIR" ]] && [ -w "$(dirname "$P_DIR")" ]; then
            TARGET_DIRS+=("$P_DIR")
        fi
    fi
    if [[ -n "$WIN_LOCALAPPDATA" ]]; then
        L_DIR="$(to_posix_path "$WIN_LOCALAPPDATA/Blaze/std")"
        if [[ -n "$L_DIR" ]]; then
            TARGET_DIRS+=("$L_DIR")
        fi
    fi
    if [[ -n "$WIN_USERPROFILE" ]]; then
        U_DIR="$(to_posix_path "$WIN_USERPROFILE/.blaze/std")"
        if [[ -n "$U_DIR" ]]; then
            TARGET_DIRS+=("$U_DIR")
        fi
    fi
    TARGET_DIRS+=("$HOME/.blaze/std")
else
    TARGET_DIRS+=("$HOME/.blaze/std")
    if [[ $EUID -eq 0 ]]; then
        TARGET_DIRS+=("/usr/local/share/blaze/std")
    fi
fi

# Locate source Blaze/std directory
SOURCE_BLAZE=""
if [[ -d "$SCRIPT_DIR/Blaze/std" ]]; then
    SOURCE_BLAZE="$SCRIPT_DIR/Blaze/std"
elif [[ -d "$SCRIPT_DIR/std" ]]; then
    SOURCE_BLAZE="$SCRIPT_DIR/std"
fi

CLEANUP_TEMP=""
if [[ -z "$SOURCE_BLAZE" || ! -d "$SOURCE_BLAZE" ]]; then
    echo -e "${YELLOW}Fetching Blaze standard library definitions from repository...${RESET}"
    TEMP_DIR="$(mktemp -d 2>/dev/null || mktemp -d -t 'flame_std')"
    CLEANUP_TEMP="$TEMP_DIR"
    if curl -fsSL "https://github.com/shoya-129/flame/archive/refs/heads/main.tar.gz" | tar -xz -C "$TEMP_DIR" 2>/dev/null; then
        if [[ -d "$TEMP_DIR/flame-main/Blaze/std" ]]; then
            SOURCE_BLAZE="$TEMP_DIR/flame-main/Blaze/std"
        elif [[ -d "$TEMP_DIR/flame-main/std" ]]; then
            SOURCE_BLAZE="$TEMP_DIR/flame-main/std"
        fi
    fi
fi

if [[ -n "$SOURCE_BLAZE" && -d "$SOURCE_BLAZE" ]]; then
    COPIED_COUNT=0
    PRIMARY_BLAZE_DIR=""

    for DEST in "${TARGET_DIRS[@]}"; do
        mkdir -p "$DEST" 2>/dev/null || true
        if [[ -d "$DEST" && -w "$DEST" ]]; then
            cp -r "$SOURCE_BLAZE"/* "$DEST/" 2>/dev/null || true
            echo -e "  Installed definitions to: ${GREEN}$DEST${RESET}"
            if [[ -z "$PRIMARY_BLAZE_DIR" ]]; then
                PRIMARY_BLAZE_DIR="$(dirname "$DEST")"
            fi
            COPIED_COUNT=$((COPIED_COUNT + 1))
        fi
    done

    if [[ $COPIED_COUNT -eq 0 ]]; then
        FALLBACK_DEST="$HOME/.blaze/std"
        mkdir -p "$FALLBACK_DEST"
        cp -r "$SOURCE_BLAZE"/* "$FALLBACK_DEST/"
        PRIMARY_BLAZE_DIR="$HOME/.blaze"
        echo -e "  Installed definitions to: ${GREEN}$FALLBACK_DEST${RESET}"
    fi
else
    echo -e "${YELLOW}Warning: Standard library definitions could not be located.${RESET}"
    echo -e "${YELLOW}Definitions can be initialized later via 'fmp update'.${RESET}"
fi

if [[ -n "$CLEANUP_TEMP" && -d "$CLEANUP_TEMP" ]]; then
    rm -rf "$CLEANUP_TEMP" 2>/dev/null || true
fi

# 6. Shell environment and permanent PATH persistence
echo -e "\n${BOLD}[3/3] Setting up environment and permanently persisting PATH...${RESET}"

if [[ "$TARGET_IS_WINDOWS" == true && -n "$(command -v powershell.exe)" ]]; then
    # Permanently append Cargo bin to Windows User PATH via PowerShell if not already present
    powershell.exe -NoProfile -ExecutionPolicy Bypass -Command '
        $target = [System.IO.Path]::Combine($env:USERPROFILE, ".cargo", "bin")
        $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
        if (-not ($userPath -split ";" -contains $target)) {
            $newPath = ($userPath.TrimEnd(";") + ";" + $target).TrimStart(";")
            [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
        }
        $blazeDir = [System.IO.Path]::Combine($env:LOCALAPPDATA, "Blaze")
        [Environment]::SetEnvironmentVariable("BLAZE_HOME", $blazeDir, "User")
    ' 2>/dev/null || true
else
    # Linux / macOS shell rc file persistence
    for RC in "$HOME/.bashrc" "$HOME/.zshrc" "$HOME/.profile"; do
        if [[ -f "$RC" ]]; then
            if ! grep -q '\.cargo/bin' "$RC" 2>/dev/null; then
                echo -e "\n# Flame language and Cargo toolchain" >> "$RC"
                echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> "$RC"
            fi
            if ! grep -q 'BLAZE_HOME' "$RC" 2>/dev/null; then
                echo "export BLAZE_HOME=\"$PRIMARY_BLAZE_DIR\"" >> "$RC"
            fi
        fi
    done
fi

echo -e "\n${GREEN}${BOLD}✓ Flame and Blaze toolchain successfully installed!${RESET}\n"
echo -e "  Primary Command:  ${GREEN}fmp${RESET} (also available as ${BLUE}flamelang${RESET})"
echo -e "  Binary Location:  ${BLUE}$CARGO_BIN/fmp${RESET}"
echo -e "  Blaze Definitions:${BLUE}$PRIMARY_BLAZE_DIR/std${RESET}"

echo -e "\n${BOLD}Quick Start:${RESET}"
echo -e "  Check version:    ${GREEN}fmp --version${RESET}"
echo -e "  CLI Help menu:    ${GREEN}fmp help${RESET}"
echo -e "  Update release:   ${GREEN}fmp update${RESET}"
echo -e "  Uninstall:        ${GREEN}fmp uninstall${RESET}\n"
