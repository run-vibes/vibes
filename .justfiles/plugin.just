# Plugin management commands for vibes

# Run commands from project root, not .justfiles/
set working-directory := '..'

# Default: show available plugin commands
default:
    @echo "Plugin management commands:"
    @echo "  just plugin list              List installed plugins"
    @echo "  just plugin install-groove    Install groove plugin (build, copy, enable)"
    @echo "  just plugin uninstall-groove  Uninstall groove plugin"

# List installed plugins
list:
    cargo run --release -p vibes-cli -- plugin list

# Install groove plugin (build, copy, and enable)
install-groove:
    #!/usr/bin/env bash
    set -euo pipefail

    # Build release first
    just build-release

    PLUGIN_DIR="${HOME}/.config/vibes/plugins/groove"
    mkdir -p "${PLUGIN_DIR}"

    # Detect platform and copy the appropriate library
    if [[ "$(uname)" == "Darwin" ]]; then
        cp target/release/libvibes_groove.dylib "${PLUGIN_DIR}/groove.dylib"
        echo "✓ Installed groove.dylib to ${PLUGIN_DIR}"
    else
        cp target/release/libvibes_groove.so "${PLUGIN_DIR}/groove.so"
        echo "✓ Installed groove.so to ${PLUGIN_DIR}"
    fi

    # Enable the plugin in the registry
    ./target/release/vibes plugin enable groove
    echo "✓ Plugin enabled in registry"

    echo ""
    echo "✓ Groove plugin installed and enabled"
    echo ""
    echo "Run 'vibes plugin list' to verify."

# Uninstall groove plugin
uninstall-groove:
    #!/usr/bin/env bash
    set -euo pipefail

    # Disable the plugin first (ignore errors if not enabled)
    ./target/release/vibes plugin disable groove 2>/dev/null || true

    # Remove the plugin directory
    rm -rf "${HOME}/.config/vibes/plugins/groove"
    echo "✓ Groove plugin uninstalled"
