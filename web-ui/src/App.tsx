import {
  createRouter,
  createRootRoute,
  createRoute,
  Outlet,
  Link,
  useLocation,
} from '@tanstack/react-router'
import { Header } from '@vibes/design-system'
import { Sessions } from './pages/Sessions'
import { Session } from './pages/Session'
import { QuarantinePage } from './pages/Quarantine'
import { FirehosePage } from './pages/Firehose'
import { AssessmentPage } from './pages/Assessment'
import { DebugPage } from './pages/Debug'
import { StreamsPage } from './pages/Streams'
import { SettingsPage } from './pages/Settings'
import { useAuth, useTheme } from './hooks'
import { useWebSocket } from './hooks/useWebSocket'

// Root layout component
function RootLayout() {
  const { addMessageHandler } = useWebSocket();
  const { identity, isLocal, isAuthenticated, isLoading } = useAuth({ addMessageHandler });
  const location = useLocation();
  const { theme, toggleTheme } = useTheme();

  const navItems = [
    { label: 'DASH', href: '/', isActive: location.pathname === '/' },
    { label: 'SESS', href: '/sessions', isActive: location.pathname.startsWith('/sessions') },
    { label: 'FIRE', href: '/firehose', isActive: location.pathname === '/firehose' },
    { label: 'GROOVE', href: '/groove', isGroove: true, isActive: location.pathname.startsWith('/groove') },
  ];

  return (
    <div className="app">
      <Header
        navItems={navItems}
        identity={isAuthenticated && identity ? { email: identity.email, provider: identity.identity_provider } : undefined}
        isLocal={!isLoading && isLocal}
        theme={theme}
        onThemeToggle={toggleTheme}
        settingsHref="/settings"
        renderLink={({ href, className, children }) => (
          <Link to={href} className={className}>{children}</Link>
        )}
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

const assessmentRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/groove/assessment',
  component: AssessmentPage,
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
  assessmentRoute,
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
