import { Button, EmptyState } from '@kaijukit/ui';
import { Component, type ReactNode } from 'react';
import { Link } from 'react-router-dom';
import { ApiError } from '../api';

export function NotFound() {
  return (
    <EmptyState
      title="We can't find that page"
      description="It may have been moved or deleted, or you may not have permission to see it."
      action={
        <Link className="kk-btn kk-btn--primary" to="/wiki/home">
          Go to home
        </Link>
      }
    />
  );
}

export function ErrorPanel({ error, onRetry }: { error: unknown; onRetry?: () => void }) {
  if (error instanceof ApiError && error.status === 404) return <NotFound />;
  const msg = error instanceof Error ? error.message : 'Something went wrong.';
  return (
    <div role="alert">
      <EmptyState title="Something went wrong" description={msg} action={onRetry && <Button onClick={onRetry}>Try again</Button>} />
    </div>
  );
}

export function ErrorPage({ code }: { code?: string }) {
  return code === '404' ? <NotFound /> : <ErrorPanel error={new Error(`Error ${code ?? ''}`.trim())} onRetry={() => location.reload()} />;
}

export class ErrorBoundary extends Component<{ children: ReactNode }, { error: unknown }> {
  state = { error: null as unknown };
  static getDerivedStateFromError(error: unknown) {
    return { error };
  }
  render() {
    return this.state.error ? <ErrorPanel error={this.state.error} onRetry={() => this.setState({ error: null })} /> : this.props.children;
  }
}
