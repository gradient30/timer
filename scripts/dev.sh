#!/usr/bin/env bash

# TimerApp project management script (cross-platform)
# Usage: ./scripts/dev.sh [command] [args]

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
TIMER_DIR="$PROJECT_ROOT/timer"
SRC_TAURI_DIR="$TIMER_DIR/src-tauri"
DOCS_DIR="$PROJECT_ROOT/docs"
MANIFEST_PATH="$SRC_TAURI_DIR/Cargo.toml"
LOCAL_ACTIVATION_ENV="$PROJECT_ROOT/config/local/activation.env"
PUBLIC_ACTIVATION_ENV="$PROJECT_ROOT/config/public/activation.env.example"
DEV_CARGO_FEATURES="${DEV_CARGO_FEATURES:-activation-admin}"

print_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[OK]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[--]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

is_windows() {
    [[ "${OSTYPE:-}" == "msys" ]] || [[ "${OSTYPE:-}" == "cygwin" ]] || [[ "${OSTYPE:-}" == "mingw"* ]] || [[ -n "${WINDIR:-}" ]]
}

require_cmd() {
    local cmd="$1"
    if ! command -v "$cmd" >/dev/null 2>&1; then
        print_error "Missing required command: $cmd"
        exit 1
    fi
}

open_dir() {
    local path="$1"
    if is_windows; then
        if command -v cygpath >/dev/null 2>&1; then
            explorer.exe "$(cygpath -aw "$path")" >/dev/null 2>&1 &
        else
            explorer.exe "$path" >/dev/null 2>&1 &
        fi
    elif [[ "${OSTYPE:-}" == "darwin"* ]]; then
        open "$path" >/dev/null 2>&1 &
    else
        if command -v xdg-open >/dev/null 2>&1; then
            xdg-open "$path" >/dev/null 2>&1 &
        else
            print_error "Cannot open docs directory automatically (xdg-open not found)"
            print_info "Open manually: $path"
            return 1
        fi
    fi
}

ensure_local_activation_env() {
    if [[ -f "$LOCAL_ACTIVATION_ENV" ]]; then
        return 0
    fi

    if [[ ! -f "$PUBLIC_ACTIVATION_ENV" ]]; then
        print_error "Missing template: $PUBLIC_ACTIVATION_ENV"
        exit 1
    fi

    mkdir -p "$(dirname "$LOCAL_ACTIVATION_ENV")"
    cp "$PUBLIC_ACTIVATION_ENV" "$LOCAL_ACTIVATION_ENV"
    print_warning "Created $LOCAL_ACTIVATION_ENV from template. Edit secrets before release."
}

run_dev() {
    require_cmd npm
    ensure_local_activation_env
    print_info "Starting development server (features=$DEV_CARGO_FEATURES)..."
    (
        cd "$TIMER_DIR"
        npm run tauri dev -- --features "$DEV_CARGO_FEATURES"
    )
}

run_build() {
    require_cmd cargo
    ensure_local_activation_env
    print_info "Building project (features=$DEV_CARGO_FEATURES)..."
    (
        cd "$SRC_TAURI_DIR"
        cargo build --features "$DEV_CARGO_FEATURES"
    )
    print_success "Build completed"
}

run_icons() {
    require_cmd python
    require_cmd npm
    print_info "Generating clock app icon assets..."
    python "$PROJECT_ROOT/scripts/generate-app-icon.py"
    (
        cd "$TIMER_DIR"
        npx tauri icon src-tauri/icons/app-icon.png -o src-tauri/icons
    )
    print_success "Icon assets updated"
}

run_release() {
    require_cmd npm
    ensure_local_activation_env
    run_icons
    print_info "Building public release package (no activation-admin)..."
    (
        cd "$TIMER_DIR"
        npm run tauri build
    )
}

run_check() {
    require_cmd cargo
    ensure_local_activation_env
    print_info "Checking Rust code..."
    (
        cd "$SRC_TAURI_DIR"
        print_info "Running cargo check"
        cargo check --features "$DEV_CARGO_FEATURES"
        print_info "Running cargo clippy -D warnings"
        cargo clippy --features "$DEV_CARGO_FEATURES" -- -D warnings
    )
    print_success "Checks completed"
}

run_test() {
    require_cmd cargo
    ensure_local_activation_env
    print_info "Running tests..."
    (
        cd "$SRC_TAURI_DIR"
        cargo test --features "$DEV_CARGO_FEATURES"
    )
}

run_setup_config() {
    ensure_local_activation_env
    print_success "Local config ready: $LOCAL_ACTIVATION_ENV"
    print_info "Public templates live in: $PROJECT_ROOT/config/public/"
}

run_setup_hooks() {
    local hooks_path="$PROJECT_ROOT/.githooks"
    if [[ ! -f "$hooks_path/commit-msg" ]]; then
        print_error "Missing hook: $hooks_path/commit-msg"
        exit 1
    fi
    chmod +x "$hooks_path/commit-msg" 2>/dev/null || true
    git -C "$PROJECT_ROOT" config core.hooksPath .githooks
    print_success "Git hooks enabled: core.hooksPath=.githooks"
    print_info "commit-msg hook will strip Cursor co-author trailers"
}

run_clean() {
    print_info "Cleaning build artifacts..."

    if [[ -d "$SRC_TAURI_DIR/target" ]]; then
        rm -rf "$SRC_TAURI_DIR/target"
        print_success "Removed timer/src-tauri/target"
    else
        print_warning "target directory not found, skipped"
    fi

    if [[ -d "$TIMER_DIR/dist" ]]; then
        rm -rf "$TIMER_DIR/dist"
        print_success "Removed timer/dist"
    else
        print_warning "dist directory not found, skipped"
    fi
}

run_docs() {
    if [[ ! -d "$DOCS_DIR" ]]; then
        print_error "Docs directory not found: $DOCS_DIR"
        exit 1
    fi
    print_info "Opening docs directory..."
    open_dir "$DOCS_DIR"
}

run_activation() {
    require_cmd cargo
    local count="${1:-1}"
    if ! [[ "$count" =~ ^[0-9]+$ ]] || [[ "$count" -lt 1 ]]; then
        print_error "Count must be a positive integer"
        exit 1
    fi

    print_info "Generating activation codes (count=$count)..."
    cargo run --bin activation_gen --manifest-path "$MANIFEST_PATH" -- "$count"
}

show_help() {
    cat <<EOF
TimerApp project management script

Usage:
  ./scripts/dev.sh [command] [args]

Commands:
  dev                  Start Tauri development server
  build                Build Rust project (src-tauri)
  check                Run cargo check and cargo clippy -D warnings
  test                 Run cargo test
  clean                Remove timer/dist and timer/src-tauri/target
  docs                 Open docs directory
  icons                Regenerate clock icon assets (icon.ico / tray / MSI)
  release              Build Tauri release package (runs icons first)
  activation [count]   Generate offline activation codes (default: 1)
  setup-config         Create config/local/activation.env from public template
  setup-hooks          Enable .githooks (strip Cursor co-author on commit)
  help                 Show this help

Examples:
  ./scripts/dev.sh dev
  ./scripts/dev.sh check
  ./scripts/dev.sh activation 10
EOF
}

main() {
    local cmd="${1:-help}"
    case "$cmd" in
        dev) run_dev ;;
        build) run_build ;;
        check) run_check ;;
        test) run_test ;;
        clean) run_clean ;;
        docs) run_docs ;;
        icons) run_icons ;;
        release) run_release ;;
        activation) run_activation "${2:-1}" ;;
        setup-config) run_setup_config ;;
        setup-hooks) run_setup_hooks ;;
        help|--help|-h) show_help ;;
        *)
            print_error "Unknown command: $cmd"
            show_help
            exit 1
            ;;
    esac
}

main "$@"
