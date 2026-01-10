# Web UI commands for vibes

# Run commands from project root, not .justfiles/
set working-directory := '..'

# Default: show available web commands
default:
    @echo "Web UI commands:"
    @echo "  just web build       Build web-ui (required for server)"
    @echo "  just web typecheck   TypeScript type check (no emit)"
    @echo "  just web test        Run web-ui tests"
    @echo "  just web install     Install npm dependencies"
    @echo "  just web e2e         E2E tests with Playwright"
    @echo "  just web e2e-headed  E2E tests in headed mode (visible browser)"
    @echo "  just web e2e-debug   E2E tests in debug mode"
    @echo "  just web e2e-setup   Install Playwright browsers"

# Build web-ui (required for server)
build:
    npm run build

# TypeScript type check (no emit)
typecheck:
    npm run typecheck --workspace=vibes-web-ui

# Run web-ui tests
test:
    npm test --workspace=vibes-web-ui -- --run

# Install npm dependencies (uses workspaces)
install:
    npm ci

# E2E tests with Playwright
e2e:
    npm run test:e2e

# E2E tests in headed mode (visible browser)
e2e-headed:
    npm run test:e2e:headed

# E2E tests in debug mode
e2e-debug:
    npm run test:e2e -- --debug

# Install Playwright browsers
e2e-setup:
    npx playwright install chromium --with-deps

# Run visual regression tests only
visual:
    npm run test:e2e -- --grep "Visual"

# Update visual regression baselines
visual-update:
    npm run test:e2e -- --grep "Visual" --update-snapshots

# Run workflow tests only (generates videos)
workflows:
    npm run test:e2e -- --grep "Workflow"
