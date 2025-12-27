import {
  createRouter,
  createRootRoute,
  createRoute,
  Outlet,
  Link,
} from '@tanstack/react-router'
import { ClaudeSessions } from './pages/ClaudeSessions'
import { ClaudeSession } from './pages/ClaudeSession'
import { StatusPage } from './pages/Status'

// Root layout component
function RootLayout() {
  return (
    <div className="app">
      <header className="header">
        <nav>
          <Link to="/" className="logo">vibes</Link>
          <Link to="/claude">Sessions</Link>
        </nav>
      </header>
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

// Create route tree and router
const routeTree = rootRoute.addChildren([indexRoute, claudeRoute, sessionRoute, statusRoute])

export const router = createRouter({ routeTree })

// Type registration for TanStack Router
declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router
  }
}
