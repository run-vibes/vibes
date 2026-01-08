# Test coverage commands for vibes

# Default: show available coverage commands
default:
    @just coverage --list

# Generate coverage report (HTML, opens in browser)
report:
    cargo llvm-cov --html --open

# Generate HTML report without opening browser
html:
    cargo llvm-cov --html
    @echo "✓ Coverage report: target/llvm-cov/html/index.html"

# Print coverage summary to terminal
summary:
    cargo llvm-cov

# Generate LCOV format (for CI/codecov)
lcov:
    cargo llvm-cov --lcov --output-path target/lcov.info
    @echo "✓ LCOV report: target/lcov.info"

# Coverage for a specific package
package PACKAGE:
    cargo llvm-cov --package {{PACKAGE}} --html --open
