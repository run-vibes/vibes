import { Outlet, Link } from '@tanstack/react-router';
import './AssessmentLayout.css';

const TABS = [
  { label: 'Stream', href: '/groove/assessment', exact: true },
  { label: 'Status', href: '/groove/assessment/status', exact: false },
  { label: 'History', href: '/groove/assessment/history', exact: false },
];

export function AssessmentLayout() {
  return (
    <div className="assessment-layout">
      <nav className="assessment-tabs">
        {TABS.map((tab) => (
          <Link
            key={tab.href}
            to={tab.href}
            className="assessment-tab"
            activeOptions={{ exact: tab.exact }}
            activeProps={{ className: 'assessment-tab active' }}
            inactiveProps={{ className: 'assessment-tab' }}
          >
            {tab.label}
          </Link>
        ))}
      </nav>
      <div className="assessment-content">
        <Outlet />
      </div>
    </div>
  );
}
