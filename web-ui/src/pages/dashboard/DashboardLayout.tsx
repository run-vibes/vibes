import { Link, Outlet } from '@tanstack/react-router';
import './DashboardLayout.css';

const TABS = [
  { label: 'Overview', href: '/groove/dashboard/overview', exact: false },
  { label: 'Learnings', href: '/groove/dashboard/learnings', exact: false },
  { label: 'Attribution', href: '/groove/dashboard/attribution', exact: false },
  { label: 'Strategy', href: '/groove/dashboard/strategy', exact: false },
  { label: 'Health', href: '/groove/dashboard/health', exact: false },
  { label: 'OpenWorld', href: '/groove/dashboard/openworld', exact: false },
];

export function DashboardLayout() {
  return (
    <div className="dashboard-layout">
      <header className="dashboard-header">
        <div className="dashboard-header-left">
          <h1 className="dashboard-title">DASHBOARD</h1>
        </div>
        <nav className="dashboard-tabs">
          {TABS.map((tab) => (
            <Link
              key={tab.href}
              to={tab.href}
              className="dashboard-tab"
              activeOptions={{ exact: tab.exact }}
              activeProps={{ className: 'dashboard-tab active' }}
              inactiveProps={{ className: 'dashboard-tab' }}
            >
              {tab.label}
            </Link>
          ))}
        </nav>
      </header>
      <div className="dashboard-content">
        <Outlet />
      </div>
    </div>
  );
}
