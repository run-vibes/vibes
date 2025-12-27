import {
  createRouter,
  createRootRoute,
  createRoute,
  Outlet,
  Link,
} from '@tanstack/react-router'

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

// Claude sessions page (placeholder)
function ClaudePage() {
  return (
    <div className="page">
      <h1>Claude Sessions</h1>
      <p>No active sessions</p>
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
  component: ClaudePage,
})

// Create route tree and router
const routeTree = rootRoute.addChildren([indexRoute, claudeRoute])

export const router = createRouter({ routeTree })

// Type registration for TanStack Router
declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router
  }
}
