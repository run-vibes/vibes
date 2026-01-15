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
import { AssessmentStream, AssessmentHistory } from './pages/assessment'
import { StatusPage } from './pages/groove/StatusPage'
import { LearningsPage } from './pages/groove/LearningsPage'
import { StrategyPage } from './pages/groove/StrategyPage'
import { OpenWorldPage } from './pages/groove/OpenWorldPage'
import { TrendsPage } from './pages/groove/TrendsPage'
import { DebugPage } from './pages/Debug'
import { StreamsPage } from './pages/Streams'
import { SettingsPage } from './pages/Settings'
import { ModelsPage } from './pages/Models'
import { AgentsPage } from './pages/Agents'
import { Traces } from './pages/Traces'
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
    { label: 'AGENTS', href: '/agents', isActive: location.pathname === '/agents' },
    { label: 'FIREHOSE', href: '/firehose', isActive: location.pathname === '/firehose' },
    { label: 'TRACES', href: '/traces', isActive: location.pathname === '/traces' },
    { label: 'MODELS', href: '/models', isActive: location.pathname === '/models' },
    { label: 'GROOVE', href: '/groove', isGroove: true, isActive: isGroovePath, hasSubnav: true },
  ];

  const grooveSubnavItems = [
    { label: 'Status', href: '/groove/status', isActive: location.pathname === '/groove/status' },
    { label: 'Learnings', href: '/groove/learnings', isActive: location.pathname === '/groove/learnings' },
    { label: 'Security', href: '/groove/security', isActive: location.pathname === '/groove/security' },
    { label: 'Stream', href: '/groove/stream', isActive: location.pathname === '/groove/stream' },
    { label: 'Strategy', href: '/groove/strategy', isActive: location.pathname === '/groove/strategy' },
  ];

  const grooveMoreItems = [
    { label: 'OpenWorld', href: '/groove/openworld', isActive: location.pathname === '/groove/openworld' },
    { label: 'History', href: '/groove/history', isActive: location.pathname === '/groove/history' },
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
        pathname={location.pathname}
        toolbarItems={
          grooveSettings.showLearningIndicator && (
            <LearningIndicator
              state="idle"
              onClick={() => navigate({ to: '/groove/status' })}
            />
          )
        }
      />
      <SubnavBar
        isOpen={isGroovePath}
        items={grooveSubnavItems}
        moreItems={grooveMoreItems}
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

// Groove index redirects to status
const grooveIndexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/groove',
  beforeLoad: () => {
    throw redirect({ to: '/groove/status' });
  },
  component: () => null,
})

const grooveStatusRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/groove/status',
  component: StatusPage,
})

const grooveLearningsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/groove/learnings',
  component: LearningsPage,
})

const grooveSecurityRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/groove/security',
  component: QuarantinePage,
})

const grooveStreamRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/groove/stream',
  component: AssessmentStream,
})

const grooveStrategyRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/groove/strategy',
  component: StrategyPage,
})

const grooveOpenWorldRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/groove/openworld',
  component: OpenWorldPage,
})

const grooveHistoryRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/groove/history',
  component: AssessmentHistory,
})

const grooveTrendsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/groove/trends',
  component: TrendsPage,
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

const modelsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/models',
  component: ModelsPage,
})

const agentsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/agents',
  component: AgentsPage,
})

const tracesRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/traces',
  component: Traces,
})

// Create route tree and router
const routeTree = rootRoute.addChildren([
  indexRoute,
  sessionsRoute,
  sessionRoute,
  agentsRoute,
  tracesRoute,
  grooveIndexRoute,
  grooveStatusRoute,
  grooveLearningsRoute,
  grooveSecurityRoute,
  grooveStreamRoute,
  grooveStrategyRoute,
  grooveOpenWorldRoute,
  grooveHistoryRoute,
  grooveTrendsRoute,
  streamsRoute,
  firehoseRoute,
  modelsRoute,
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
