import { Link } from '@tanstack/react-router';
import './NotFound.css';

export function NotFound() {
  return (
    <div className="not-found">
      <div className="not-found__content">
        <h1 className="not-found__code">404</h1>
        <p className="not-found__message">Page not found</p>
        <p className="not-found__hint">
          The route you're looking for doesn't exist.
        </p>
        <Link to="/" className="not-found__link">
          ← Back to home
        </Link>
      </div>
    </div>
  );
}
