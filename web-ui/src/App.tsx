import { useState, useEffect } from 'react'
import {
  createRouter,
  createRootRoute,
  createRoute,
  Outlet,
  Link,
  useLocation,
} from '@tanstack/react-router'
import { Header } from '@vibes/design-system'
import { ClaudeSessions } from './pages/ClaudeSessions'
import { ClaudeSession } from './pages/ClaudeSession'
import { StatusPage } from './pages/Status'
import { QuarantinePage } from './pages/Quarantine'
import { useAuth } from './hooks/useAuth'
import { useWebSocket } from './hooks/useWebSocket'

// Root layout component
function RootLayout() {
  const { addMessageHandler } = useWebSocket();
  const { identity, isLocal, isAuthenticated, isLoading } = useAuth({ addMessageHandler });
  const location = useLocation();
  const [theme, setTheme] = useState<'dark' | 'light'>('dark');

  useEffect(() => {
    document.documentElement.setAttribute('data-theme', theme);
  }, [theme]);

  const navItems = [
    { label: 'Sessions', href: '/claude', isActive: location.pathname.startsWith('/claude') },
    { label: 'Groove', href: '/groove', isGroove: true, isActive: location.pathname.startsWith('/groove') },
  ];

  return (
    <div className="app">
      <Header
        navItems={navItems}
        identity={isAuthenticated && identity ? { email: identity.email, provider: identity.identity_provider } : undefined}
        isLocal={!isLoading && isLocal}
        theme={theme}
        onThemeToggle={() => setTheme(t => t === 'dark' ? 'light' : 'dark')}
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

// Home page
function HomePage() {
  return (
    <div className="page">
      <h1>vibes</h1>
      <p>Remote control for your Claude Code sessions</p>
      <Link to="/claude" className="button">
        View Sessions
      </Link>
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
  component: HomePage,
})

const claudeRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/claude',
  component: ClaudeSessions,
})

const sessionRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/claude/$sessionId',
  component: ClaudeSession,
})

const statusRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/status',
  component: StatusPage,
})

const grooveRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/groove',
  component: QuarantinePage,
})

// Create route tree and router
const routeTree = rootRoute.addChildren([indexRoute, claudeRoute, sessionRoute, statusRoute, grooveRoute])

export const router = createRouter({ routeTree })

// Type registration for TanStack Router
declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router
  }
}
