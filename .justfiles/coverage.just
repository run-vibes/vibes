# Test coverage commands for vibes

# Run commands from project root, not .justfiles/
set working-directory := '..'

# Exclude nix store paths and vendored dependencies from coverage
IGNORE_REGEX := '(^/nix/store/|^vendor/)'

# Default: show available coverage commands
default:
    @echo "Coverage commands:"
    @echo "  just coverage report        Generate HTML report, opens in browser"
    @echo "  just coverage html          Generate HTML report without opening browser"
    @echo "  just coverage summary       Print coverage summary to terminal"
    @echo "  just coverage lcov          Generate LCOV format (for CI/codecov)"
    @echo "  just coverage package <pkg> Coverage for a specific package"

# Generate coverage report (HTML, opens in browser)
report:
    cargo llvm-cov --html --open --ignore-filename-regex '{{IGNORE_REGEX}}'

# Generate HTML report without opening browser
html:
    cargo llvm-cov --html --ignore-filename-regex '{{IGNORE_REGEX}}'
    @echo "✓ Coverage report: target/llvm-cov/html/index.html"

# Print coverage summary to terminal
summary:
    cargo llvm-cov --ignore-filename-regex '{{IGNORE_REGEX}}'

# Generate LCOV format (for CI/codecov)
lcov:
    cargo llvm-cov --lcov --output-path target/lcov.info --ignore-filename-regex '{{IGNORE_REGEX}}'
    @echo "✓ LCOV report: target/lcov.info"

# Coverage for a specific package
package PACKAGE:
    cargo llvm-cov --package {{PACKAGE}} --html --open --ignore-filename-regex '{{IGNORE_REGEX}}'
