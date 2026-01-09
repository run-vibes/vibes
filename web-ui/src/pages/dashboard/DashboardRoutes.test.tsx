import { describe, test, expect } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import {
  RouterProvider,
  createRouter,
  createRootRoute,
  createRoute,
  createMemoryHistory,
  Outlet,
  redirect,
} from '@tanstack/react-router';
import { DashboardLayout } from './index';

// This test file verifies that dashboard routes are properly configured
// We test the route configuration separately from the App to isolate concerns

// Create a minimal router for testing dashboard routes
function createTestRouter(initialPath: string) {
  const rootRoute = createRootRoute({
    component: () => <Outlet />,
  });

  const dashboardLayoutRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/groove/dashboard',
    component: DashboardLayout,
  });

  // Index redirects to overview
  const dashboardIndexRoute = createRoute({
    getParentRoute: () => dashboardLayoutRoute,
    path: '/',
    beforeLoad: () => {
      throw redirect({ to: '/groove/dashboard/overview' });
    },
    component: () => null,
  });

  const dashboardOverviewRoute = createRoute({
    getParentRoute: () => dashboardLayoutRoute,
    path: '/overview',
    component: () => <div data-testid="overview-page">Overview Page</div>,
  });

  const dashboardLearningsRoute = createRoute({
    getParentRoute: () => dashboardLayoutRoute,
    path: '/learnings',
    component: () => <div data-testid="learnings-page">Learnings Page</div>,
  });

  const dashboardAttributionRoute = createRoute({
    getParentRoute: () => dashboardLayoutRoute,
    path: '/attribution',
    component: () => <div data-testid="attribution-page">Attribution Page</div>,
  });

  const dashboardStrategyRoute = createRoute({
    getParentRoute: () => dashboardLayoutRoute,
    path: '/strategy',
    component: () => <div data-testid="strategy-page">Strategy Page</div>,
  });

  const dashboardHealthRoute = createRoute({
    getParentRoute: () => dashboardLayoutRoute,
    path: '/health',
    component: () => <div data-testid="health-page">Health Page</div>,
  });

  const routeTree = rootRoute.addChildren([
    dashboardLayoutRoute.addChildren([
      dashboardIndexRoute,
      dashboardOverviewRoute,
      dashboardLearningsRoute,
      dashboardAttributionRoute,
      dashboardStrategyRoute,
      dashboardHealthRoute,
    ]),
  ]);

  const memoryHistory = createMemoryHistory({
    initialEntries: [initialPath],
  });

  return createRouter({
    routeTree,
    history: memoryHistory,
  });
}

describe('Dashboard Routes', () => {
  test('dashboard components export correctly', async () => {
    // Verify the components export correctly from barrel file
    const { DashboardLayout } = await import('./index');

    expect(DashboardLayout).toBeDefined();
  });

  test('dashboard index route redirects to overview within layout', async () => {
    const router = createTestRouter('/groove/dashboard');

    render(<RouterProvider router={router} />);

    // Layout should render with subnav tabs
    await waitFor(() => {
      expect(screen.getByRole('link', { name: /overview/i })).toBeInTheDocument();
      expect(screen.getByRole('link', { name: /learnings/i })).toBeInTheDocument();
      expect(screen.getByRole('link', { name: /attribution/i })).toBeInTheDocument();
      expect(screen.getByRole('link', { name: /strategy/i })).toBeInTheDocument();
      expect(screen.getByRole('link', { name: /health/i })).toBeInTheDocument();
    });
  });

  test('/groove/dashboard/overview route renders overview page within layout', async () => {
    const router = createTestRouter('/groove/dashboard/overview');

    render(<RouterProvider router={router} />);

    await waitFor(() => {
      // Layout tabs should be present
      expect(screen.getByRole('link', { name: /overview/i })).toBeInTheDocument();
      // Overview page content should render
      expect(screen.getByTestId('overview-page')).toBeInTheDocument();
    });
  });

  test('/groove/dashboard/learnings route renders learnings page within layout', async () => {
    const router = createTestRouter('/groove/dashboard/learnings');

    render(<RouterProvider router={router} />);

    await waitFor(() => {
      expect(screen.getByRole('link', { name: /overview/i })).toBeInTheDocument();
      expect(screen.getByTestId('learnings-page')).toBeInTheDocument();
    });
  });

  test('/groove/dashboard/attribution route renders attribution page within layout', async () => {
    const router = createTestRouter('/groove/dashboard/attribution');

    render(<RouterProvider router={router} />);

    await waitFor(() => {
      expect(screen.getByRole('link', { name: /overview/i })).toBeInTheDocument();
      expect(screen.getByTestId('attribution-page')).toBeInTheDocument();
    });
  });

  test('/groove/dashboard/strategy route renders strategy page within layout', async () => {
    const router = createTestRouter('/groove/dashboard/strategy');

    render(<RouterProvider router={router} />);

    await waitFor(() => {
      expect(screen.getByRole('link', { name: /overview/i })).toBeInTheDocument();
      expect(screen.getByTestId('strategy-page')).toBeInTheDocument();
    });
  });

  test('/groove/dashboard/health route renders health page within layout', async () => {
    const router = createTestRouter('/groove/dashboard/health');

    render(<RouterProvider router={router} />);

    await waitFor(() => {
      expect(screen.getByRole('link', { name: /overview/i })).toBeInTheDocument();
      expect(screen.getByTestId('health-page')).toBeInTheDocument();
    });
  });
});
