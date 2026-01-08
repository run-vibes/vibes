# Web UI commands for vibes

# Default: show available web commands
default:
    @just web --list

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
