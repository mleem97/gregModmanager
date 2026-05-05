#!/usr/bin/env bash
# gregModmanager Interactive Builder
# Styled CLI UI for local builds, run, and test workflows.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

# Colors
C_RESET='\033[0m'
C_PRIMARY='\033[36m'
C_SECONDARY='\033[32m'
C_WARN='\033[33m'
C_ERROR='\033[31m'
C_DEFAULT='\033[90m'
C_ONSURFACE='\033[37m'
C_BG='\033[48;5;240m'

SELECTED=0
ITEM_COUNT=10

# ---------------------------------------------------------------------------
# UI Helpers
# ---------------------------------------------------------------------------

clear_screen() {
    clear || printf '\033[2J\033[H'
}

print_banner() {
    clear_screen
    echo ""
    echo -e "  ${C_PRIMARY}================================================${C_RESET}"
    echo -e "     ${C_ONSURFACE}gregModmanager  -  Interactive Builder${C_RESET}"
    echo -e "     ${C_DEFAULT}Avalonia UI  |  .NET 9  |  Cross-Platform${C_RESET}"
    echo -e "  ${C_PRIMARY}================================================${C_RESET}"
    echo ""
}

print_menu_item() {
    local key="$1"
    local label="$2"
    local active="${3:-false}"
    if [[ "$active" == "true" ]]; then
        echo -e "  ${C_BG}${C_PRIMARY}> [$key]  $label${C_RESET}"
    else
        echo -e "    ${C_DEFAULT}[$key]  $label${C_RESET}"
    fi
}

show_menu() {
    print_banner

    local items=(
        "B:Build All (CI mirror)"
        "W:Build Windows Only"
        "L:Build Linux Only"
        "P:Build Linux Packages"
        "R:Run Avalonia (Debug)"
        "D:Run Avalonia (Release)"
        "T:Test (dotnet test)"
        "C:Clean artifacts"
        "I:Install locally (win-x64)"
        "Q:Quit"
    )

    local i=0
    for item in "${items[@]}"; do
        local key="${item%%:*}"
        local label="${item#*:}"
        if [[ $i -eq $SELECTED ]]; then
            print_menu_item "$key" "$label" "true"
        else
            print_menu_item "$key" "$label" "false"
        fi
        ((i++))
    done

    echo ""
    echo -e "  ${C_DEFAULT}Use arrow keys or number to select, Enter to confirm${C_RESET}"
}

pause_any_key() {
    echo ""
    echo -e "  ${C_DEFAULT}Press any key to continue...${C_RESET}"
    read -rs -n1
}

# ---------------------------------------------------------------------------
# Actions
# ---------------------------------------------------------------------------

invoke_build_all() {
    print_banner
    echo -e "  ${C_PRIMARY}[BUILD ALL]${C_RESET}"
    echo ""
    if bash "$REPO_ROOT/scripts/build.ps1" 2>/dev/null || pwsh "$REPO_ROOT/scripts/build.ps1" 2>/dev/null; then
        echo ""
        echo -e "  ${C_SECONDARY}Build completed successfully.${C_RESET}"
    else
        echo ""
        echo -e "  ${C_ERROR}Build failed.${C_RESET}"
    fi
    pause_any_key
}

invoke_build_windows() {
    print_banner
    echo -e "  ${C_PRIMARY}[BUILD WINDOWS]${C_RESET}"
    echo ""
    if bash "$REPO_ROOT/scripts/build.ps1" -SkipLinux -SkipLinuxPackages 2>/dev/null || \
       pwsh "$REPO_ROOT/scripts/build.ps1" -SkipLinux -SkipLinuxPackages 2>/dev/null; then
        echo ""
        echo -e "  ${C_SECONDARY}Windows build completed.${C_RESET}"
    else
        echo ""
        echo -e "  ${C_ERROR}Windows build failed.${C_RESET}"
    fi
    pause_any_key
}

invoke_build_linux() {
    print_banner
    echo -e "  ${C_PRIMARY}[BUILD LINUX]${C_RESET}"
    echo ""
    if bash "$REPO_ROOT/scripts/build.ps1" -SkipWindows -SkipLinuxPackages 2>/dev/null || \
       pwsh "$REPO_ROOT/scripts/build.ps1" -SkipWindows -SkipLinuxPackages 2>/dev/null; then
        echo ""
        echo -e "  ${C_SECONDARY}Linux build completed.${C_RESET}"
    else
        echo ""
        echo -e "  ${C_ERROR}Linux build failed.${C_RESET}"
    fi
    pause_any_key
}

invoke_build_linux_packages() {
    print_banner
    echo -e "  ${C_PRIMARY}[BUILD LINUX PACKAGES]${C_RESET}"
    echo ""
    if bash "$REPO_ROOT/scripts/build.ps1" -SkipWindows 2>/dev/null || \
       pwsh "$REPO_ROOT/scripts/build.ps1" -SkipWindows 2>/dev/null; then
        echo ""
        echo -e "  ${C_SECONDARY}Linux packages built.${C_RESET}"
    else
        echo ""
        echo -e "  ${C_ERROR}Linux packages failed.${C_RESET}"
    fi
    pause_any_key
}

invoke_run_debug() {
    print_banner
    echo -e "  ${C_PRIMARY}[RUN DEBUG]${C_RESET}"
    echo ""
    dotnet run --project "$REPO_ROOT/GregModmanager.Avalonia/GregModmanager.Avalonia.csproj" -c Debug || true
    pause_any_key
}

invoke_run_release() {
    print_banner
    echo -e "  ${C_PRIMARY}[RUN RELEASE]${C_RESET}"
    echo ""
    dotnet run --project "$REPO_ROOT/GregModmanager.Avalonia/GregModmanager.Avalonia.csproj" -c Release || true
    pause_any_key
}

invoke_test() {
    print_banner
    echo -e "  ${C_PRIMARY}[TEST]${C_RESET}"
    echo ""
    if dotnet test "$REPO_ROOT/GregModmanager.sln" --verbosity normal; then
        echo ""
        echo -e "  ${C_SECONDARY}Tests completed.${C_RESET}"
    else
        echo ""
        echo -e "  ${C_ERROR}Tests failed.${C_RESET}"
    fi
    pause_any_key
}

invoke_clean() {
    print_banner
    echo -e "  ${C_PRIMARY}[CLEAN]${C_RESET}"
    echo ""
    local dirs=("artifacts" "installer/Output" "GregModmanager.Avalonia/bin" "GregModmanager.Avalonia/obj" "bin" "obj")
    for d in "${dirs[@]}"; do
        local p="$REPO_ROOT/$d"
        if [[ -d "$p" ]]; then
            rm -rf "$p"
            echo -e "  ${C_WARN}removed: $d${C_RESET}"
        fi
    done
    echo ""
    echo -e "  ${C_SECONDARY}Clean finished.${C_RESET}"
    pause_any_key
}

invoke_install_local() {
    print_banner
    echo -e "  ${C_PRIMARY}[INSTALL LOCAL]${C_RESET}"
    echo ""
    echo -e "  ${C_WARN}Install-Local is Windows-only (PowerShell).${C_RESET}"
    echo -e "  ${C_DEFAULT}Run: .\\scripts\\install-local.ps1  from PowerShell.${C_RESET}"
    pause_any_key
}

invoke_choice() {
    local idx="$1"
    case "$idx" in
        0) invoke_build_all ;;
        1) invoke_build_windows ;;
        2) invoke_build_linux ;;
        3) invoke_build_linux_packages ;;
        4) invoke_run_debug ;;
        5) invoke_run_release ;;
        6) invoke_test ;;
        7) invoke_clean ;;
        8) invoke_install_local ;;
        9) exit 0 ;;
    esac
}

# ---------------------------------------------------------------------------
# Input Loop
# ---------------------------------------------------------------------------

main_loop() {
    while true; do
        show_menu

        # Read a single key
        IFS= read -rs -n1 key

        case "$key" in
            $'\x1b')
                # Escape sequence (arrow keys)
                read -rs -n2 seq
                case "$seq" in
                    '[A') # Up
                        if ((SELECTED > 0)); then ((SELECTED--)); fi
                        ;;
                    '[B') # Down
                        if ((SELECTED < ITEM_COUNT - 1)); then ((SELECTED++)); fi
                        ;;
                esac
                ;;
            '') # Enter
                invoke_choice "$SELECTED"
                ;;
            'b'|'B') invoke_build_all ;;
            'w'|'W') invoke_build_windows ;;
            'l'|'L') invoke_build_linux ;;
            'p'|'P') invoke_build_linux_packages ;;
            'r'|'R') invoke_run_debug ;;
            'd'|'D') invoke_run_release ;;
            't'|'T') invoke_test ;;
            'c'|'C') invoke_clean ;;
            'i'|'I') invoke_install_local ;;
            'q'|'Q') exit 0 ;;
        esac
    done
}

main_loop
