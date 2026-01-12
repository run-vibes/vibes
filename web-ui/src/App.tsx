import {
  createRouter,
  createRootRoute,
  createRoute,
  Outlet,
  Link,
  useLocation,
  useNavigate,
  redirect,
} from '@tanstack/react-router'
import { Header, SubnavBar } from '@vibes/design-system'
import { Sessions } from './pages/Sessions'
import { Session } from './pages/Session'
import { QuarantinePage } from './pages/Quarantine'
import { FirehosePage } from './pages/Firehose'
import {
  AssessmentLayout,
  AssessmentStream,
  AssessmentStatus,
  AssessmentHistory,
} from './pages/assessment'
import {
  DashboardLayout,
  DashboardOverview,
  DashboardLearnings,
  DashboardAttribution,
  DashboardStrategy,
  DashboardHealth,
  DashboardOpenWorld,
} from './pages/dashboard'
import { DebugPage } from './pages/Debug'
import { StreamsPage } from './pages/Streams'
import { SettingsPage } from './pages/Settings'
import { NotFound } from './pages/NotFound'
import { useAuth, useTheme } from './hooks'
import { useWebSocket } from './hooks/useWebSocket'
import { useGrooveSettings } from './hooks/useGrooveSettings'
import { LearningIndicator } from './components/LearningIndicator'

// Root layout component
function RootLayout() {
  const { addMessageHandler } = useWebSocket();
  const { identity, isAuthenticated } = useAuth({ addMessageHandler });
  const location = useLocation();
  const navigate = useNavigate();
  const { theme, toggleTheme } = useTheme();
  const { settings: grooveSettings } = useGrooveSettings();

  const isGroovePath = location.pathname.startsWith('/groove');

  const navItems = [
    { label: 'SESSIONS', href: '/sessions', isActive: location.pathname.startsWith('/sessions') },
    { label: 'FIREHOSE', href: '/firehose', isActive: location.pathname === '/firehose' },
    { label: 'GROOVE', href: '/groove', isGroove: true, isActive: isGroovePath, hasSubnav: true },
  ];

  const grooveSubnavItems = [
    { label: 'Security', href: '/groove', icon: '🛡', isActive: location.pathname === '/groove' },
    { label: 'Assessment', href: '/groove/assessment/status', icon: '◈', isActive: location.pathname.startsWith('/groove/assessment') },
    { label: 'Dashboard', href: '/groove/dashboard/overview', icon: '📊', isActive: location.pathname.startsWith('/groove/dashboard') },
  ];

  const renderLink = ({ href, className, children }: { href: string; className: string; children: React.ReactNode }) => (
    <Link to={href} className={className}>{children}</Link>
  );

  return (
    <div className="app">
      <Header
        navItems={navItems}
        identity={isAuthenticated && identity ? { email: identity.email, provider: identity.identity_provider } : undefined}
        theme={theme}
        onThemeToggle={toggleTheme}
        settingsHref="/settings"
        renderLink={renderLink}
        toolbarItems={
          grooveSettings.showLearningIndicator && (
            <LearningIndicator
              state="idle"
              onClick={() => navigate({ to: '/groove/dashboard/overview' })}
            />
          )
        }
      />
      <SubnavBar
        isOpen={isGroovePath}
        label="GROOVE"
        items={grooveSubnavItems}
        plugin="groove"
        renderLink={renderLink}
      />
      <main className="main">
        <Outlet />
      </main>
    </div>
  )
}

// Define routes
const rootRoute = createRootRoute({
  component: RootLayout,
  notFoundComponent: NotFound,
})

const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  component: StreamsPage,
})

const sessionsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/sessions',
  component: Sessions,
})

const sessionRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/sessions/$sessionId',
  component: Session,
})

const grooveRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/groove',
  component: QuarantinePage,
})

const streamsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/streams',
  component: StreamsPage,
})

const firehoseRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/firehose',
  component: FirehosePage,
})

// Assessment routes - nested under layout
const assessmentLayoutRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/groove/assessment',
  component: AssessmentLayout,
  notFoundComponent: NotFound,
})

const assessmentStatusRoute = createRoute({
  getParentRoute: () => assessmentLayoutRoute,
  path: '/status',
  component: AssessmentStatus,
})

const assessmentStreamRoute = createRoute({
  getParentRoute: () => assessmentLayoutRoute,
  path: '/stream',
  component: AssessmentStream,
})

const assessmentHistoryRoute = createRoute({
  getParentRoute: () => assessmentLayoutRoute,
  path: '/history',
  component: AssessmentHistory,
})

// Redirect index to status
const assessmentIndexRoute = createRoute({
  getParentRoute: () => assessmentLayoutRoute,
  path: '/',
  beforeLoad: () => {
    throw redirect({ to: '/groove/assessment/status' });
  },
  component: () => null,
})

// Dashboard routes - nested under layout
const dashboardLayoutRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/groove/dashboard',
  component: DashboardLayout,
  notFoundComponent: NotFound,
})

const dashboardOverviewRoute = createRoute({
  getParentRoute: () => dashboardLayoutRoute,
  path: '/overview',
  component: DashboardOverview,
})

const dashboardLearningsRoute = createRoute({
  getParentRoute: () => dashboardLayoutRoute,
  path: '/learnings',
  component: DashboardLearnings,
})

const dashboardAttributionRoute = createRoute({
  getParentRoute: () => dashboardLayoutRoute,
  path: '/attribution',
  component: DashboardAttribution,
})

const dashboardStrategyRoute = createRoute({
  getParentRoute: () => dashboardLayoutRoute,
  path: '/strategy',
  component: DashboardStrategy,
})

const dashboardHealthRoute = createRoute({
  getParentRoute: () => dashboardLayoutRoute,
  path: '/health',
  component: DashboardHealth,
})

const dashboardOpenWorldRoute = createRoute({
  getParentRoute: () => dashboardLayoutRoute,
  path: '/openworld',
  component: DashboardOpenWorld,
})

// Redirect dashboard index to overview
const dashboardIndexRoute = createRoute({
  getParentRoute: () => dashboardLayoutRoute,
  path: '/',
  beforeLoad: () => {
    throw redirect({ to: '/groove/dashboard/overview' });
  },
  component: () => null,
})

const debugRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/debug',
  component: DebugPage,
})

const settingsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/settings',
  component: SettingsPage,
})

// Create route tree and router
const routeTree = rootRoute.addChildren([
  indexRoute,
  sessionsRoute,
  sessionRoute,
  grooveRoute,
  streamsRoute,
  firehoseRoute,
  assessmentLayoutRoute.addChildren([
    assessmentIndexRoute,
    assessmentStatusRoute,
    assessmentStreamRoute,
    assessmentHistoryRoute,
  ]),
  dashboardLayoutRoute.addChildren([
    dashboardIndexRoute,
    dashboardOverviewRoute,
    dashboardLearningsRoute,
    dashboardAttributionRoute,
    dashboardStrategyRoute,
    dashboardHealthRoute,
    dashboardOpenWorldRoute,
  ]),
  debugRoute,
  settingsRoute,
])

export const router = createRouter({ routeTree })

// Type registration for TanStack Router
declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router
  }
}
