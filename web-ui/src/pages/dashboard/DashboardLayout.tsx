import { Link, Outlet } from '@tanstack/react-router';
import './DashboardLayout.css';

const TABS = [
  { label: 'Overview', href: '/groove/dashboard/overview', exact: false },
  { label: 'Learnings', href: '/groove/dashboard/learnings', exact: false },
  { label: 'Attribution', href: '/groove/dashboard/attribution', exact: false },
  { label: 'Strategy', href: '/groove/dashboard/strategy', exact: false },
  { label: 'Health', href: '/groove/dashboard/health', exact: false },
];

export function DashboardLayout() {
  return (
    <div className="dashboard-layout">
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
      <div className="dashboard-content">
        <Outlet />
      </div>
    </div>
  );
}
