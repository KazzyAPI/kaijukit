import { Button, TextField } from '@kaijukit/ui';
import { useState } from 'react';
import { Navigate, useNavigate, useSearchParams } from 'react-router-dom';
import { ApiError, verifyLogin } from '../api';
import { useAuth } from '../auth';

export function safeNext(next: string | null) {
  return next && next.startsWith('/wiki/') && !next.startsWith('//') ? next : '/wiki/home';
}

export function Login() {
  const { user, login } = useAuth();
  const [params] = useSearchParams();
  const navigate = useNavigate();
  const [email, setEmail] = useState('');
  const [token, setToken] = useState('');
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);
  const next = safeNext(params.get('next'));
  if (user) return <Navigate to={next} replace />;

  const submit = async () => {
    setBusy(true);
    setError('');
    try {
      await verifyLogin(email.trim(), token);
      login({ email: email.trim(), token });
      navigate(next, { replace: true });
    } catch (e) {
      setError(e instanceof ApiError && e.status === 401 ? 'Email or API token is incorrect.' : e instanceof Error ? e.message : 'Sign in failed.');
    } finally {
      setBusy(false);
    }
  };

  return (
    <main className="kk-login">
      <form
        className="kk-login__card"
        onSubmit={(e) => {
          e.preventDefault();
          void submit();
        }}
      >
        <h1 className="kk-login__title">Log in to kaijukit</h1>
        <TextField label="Email" type="email" autoComplete="username" required autoFocus value={email} onChange={(e) => setEmail(e.target.value)} />
        <TextField label="API token" type="password" autoComplete="current-password" required value={token} onChange={(e) => setToken(e.target.value)} />
        {error && (
          <div className="kk-error-box" role="alert">
            {error}
          </div>
        )}
        <Button type="submit" appearance="primary" loading={busy} disabled={!email || !token}>
          Log in
        </Button>
      </form>
    </main>
  );
}
