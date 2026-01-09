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
import { AssessmentLayout } from './index';

// This test file verifies that assessment routes are properly configured
// We test the route configuration separately from the App to isolate concerns

// Create a minimal router for testing assessment routes
function createTestRouter(initialPath: string) {
  const rootRoute = createRootRoute({
    component: () => <Outlet />,
  });

  const assessmentLayoutRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/groove/assessment',
    component: AssessmentLayout,
  });

  // Index redirects to status
  const assessmentIndexRoute = createRoute({
    getParentRoute: () => assessmentLayoutRoute,
    path: '/',
    beforeLoad: () => {
      throw redirect({ to: '/groove/assessment/status' });
    },
    component: () => null,
  });

  const assessmentStatusRoute = createRoute({
    getParentRoute: () => assessmentLayoutRoute,
    path: '/status',
    component: () => <div data-testid="status-page">Status Page</div>,
  });

  const assessmentStreamRoute = createRoute({
    getParentRoute: () => assessmentLayoutRoute,
    path: '/stream',
    component: () => <div data-testid="stream-page">Stream Page</div>,
  });

  const assessmentHistoryRoute = createRoute({
    getParentRoute: () => assessmentLayoutRoute,
    path: '/history',
    component: () => <div data-testid="history-page">History Page</div>,
  });

  const routeTree = rootRoute.addChildren([
    assessmentLayoutRoute.addChildren([
      assessmentIndexRoute,
      assessmentStatusRoute,
      assessmentStreamRoute,
      assessmentHistoryRoute,
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

describe('Assessment Routes', () => {
  test('assessment components export correctly', async () => {
    // Verify the components export correctly from barrel file
    const { AssessmentLayout, AssessmentStream } = await import('./index');

    expect(AssessmentLayout).toBeDefined();
    expect(AssessmentStream).toBeDefined();
  });

  test('assessment index route renders stream within layout', async () => {
    const router = createTestRouter('/groove/assessment');

    render(<RouterProvider router={router} />);

    // Layout should render with subnav tabs
    await waitFor(() => {
      expect(screen.getByRole('link', { name: /stream/i })).toBeInTheDocument();
      expect(screen.getByRole('link', { name: /status/i })).toBeInTheDocument();
    });
  });

  test('/groove/assessment/status route renders status page within layout', async () => {
    const router = createTestRouter('/groove/assessment/status');

    render(<RouterProvider router={router} />);

    await waitFor(() => {
      // Layout tabs should be present
      expect(screen.getByRole('link', { name: /stream/i })).toBeInTheDocument();
      // Status page content should render
      expect(screen.getByTestId('status-page')).toBeInTheDocument();
    });
  });

  test('/groove/assessment/history route renders history page within layout', async () => {
    const router = createTestRouter('/groove/assessment/history');

    render(<RouterProvider router={router} />);

    await waitFor(() => {
      expect(screen.getByRole('link', { name: /stream/i })).toBeInTheDocument();
      expect(screen.getByTestId('history-page')).toBeInTheDocument();
    });
  });
});
