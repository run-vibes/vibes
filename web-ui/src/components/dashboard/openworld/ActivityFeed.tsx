/**
 * Activity feed with scrolling event list
 *
 * Shows recent open-world events with auto-scroll on new events.
 */
import { useRef, useEffect } from 'react';
import { Panel } from '@vibes/design-system';
import type { OpenWorldActivityEntry } from '../../../hooks/useDashboard';
import { ActivityItem } from './ActivityItem';
import './ActivityFeed.css';

export interface ActivityFeedProps {
  events?: OpenWorldActivityEntry[];
  isLoading?: boolean;
  autoScroll?: boolean;
}

export function ActivityFeed({ events, isLoading, autoScroll = true }: ActivityFeedProps) {
  const feedRef = useRef<HTMLDivElement>(null);
  const prevEventsLength = useRef(events?.length ?? 0);

  // Auto-scroll to top when new events arrive
  useEffect(() => {
    if (autoScroll && feedRef.current && events) {
      if (events.length > prevEventsLength.current) {
        feedRef.current.scrollTo({ top: 0, behavior: 'smooth' });
      }
      prevEventsLength.current = events.length;
    }
  }, [events, autoScroll]);

  if (isLoading) {
    return (
      <Panel variant="crt" title="Event Feed" className="activity-feed">
        <div className="activity-feed__loading">
          <span>Loading events...</span>
        </div>
      </Panel>
    );
  }

  const isEmpty = !events?.length;

  return (
    <Panel variant="crt" title="Event Feed" className="activity-feed">
      <div className="activity-feed__header">
        <span className="activity-feed__count">
          {events?.length ?? 0} events
        </span>
      </div>

      {isEmpty ? (
        <div className="activity-feed__empty" data-testid="activity-feed-empty">
          <span className="activity-feed__empty-icon">●</span>
          <span>No recent activity</span>
        </div>
      ) : (
        <div className="activity-feed__list" ref={feedRef} data-testid="activity-feed-list">
          {events.map((event, index) => (
            <ActivityItem key={`${event.timestamp}-${index}`} event={event} />
          ))}
        </div>
      )}
    </Panel>
  );
}
