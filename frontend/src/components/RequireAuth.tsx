import type { PropsWithChildren } from 'react';
import PageCard from './PageCard';

interface RequireAuthProps extends PropsWithChildren {
  isAuthenticated: boolean;
  isCheckingSession: boolean;
}

export default function RequireAuth({
  children,
  isAuthenticated,
  isCheckingSession,
}: RequireAuthProps) {
  if (isCheckingSession) {
    return (
      <PageCard
        eyebrow="Checking session"
        title="One moment"
        description="We are checking your login before opening this page."
      />
    );
  }

  if (!isAuthenticated) {
    return (
      <PageCard
        eyebrow="Login required"
        title="Please log in to continue"
        description="This page is available after you log in."
      />
    );
  }

  return children;
}
