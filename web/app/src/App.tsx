import { Skeleton } from '@kaijukit/ui';
import { lazy, Suspense } from 'react';
import { Navigate, Outlet, Route, Routes, useLocation, useParams } from 'react-router-dom';
import { useAuth } from './auth';
import { TopBar } from './TopBar';
import { ErrorBoundary, ErrorPage, NotFound } from './screens/Errors';
import { Home } from './screens/Home';
import { Login } from './screens/Login';
import { Overview, PageRoute } from './screens/PageView';
import { SpaceShell } from './screens/SpaceShell';
import { Spaces } from './screens/Spaces';

// The editor (ProseMirror) loads as its own chunk, so view-only sessions never pay for it.
const editorModule = () => import('./screens/EditorScreen');
const EditorRoute = lazy(() => editorModule().then((m) => ({ default: m.EditorRoute })));
const CreateRoute = lazy(() => editorModule().then((m) => ({ default: m.CreateRoute })));
const Loading = () => <div className="kk-container" aria-busy="true"><Skeleton height={36} width="50%" /><Skeleton height={240} style={{ marginTop: 16 }} /></div>;

function RequireAuth() {
  const { user } = useAuth();
  const loc = useLocation();
  if (!user) return <Navigate to={`/wiki/login?next=${encodeURIComponent(loc.pathname + loc.search)}`} replace />;
  return <Outlet />;
}

function Chrome() {
  return (
    <>
      <a className="kk-skip" href="#main">Skip to content</a>
      <TopBar />
      <main id="main" className="kk-main">
        <ErrorBoundary>
          <Outlet />
        </ErrorBoundary>
      </main>
    </>
  );
}

function Logout() {
  const { logout } = useAuth();
  logout();
  return <Navigate to="/wiki/login" replace />;
}

function ErrorRoute() {
  return <ErrorPage code={useParams().code} />;
}

export function App() {
  return (
    <Routes>
      <Route path="/" element={<Navigate to="/wiki/home" replace />} />
      <Route path="/wiki/login" element={<Login />} />
      <Route path="/wiki/logout" element={<Logout />} />
      <Route element={<RequireAuth />}>
        <Route path="/wiki/spaces/:key/pages/edit-v2/:id" element={<ErrorBoundary><Suspense fallback={<Loading />}><EditorRoute /></Suspense></ErrorBoundary>} />
        <Route path="/wiki/spaces/:key/pages/create" element={<ErrorBoundary><Suspense fallback={<Loading />}><CreateRoute /></Suspense></ErrorBoundary>} />
        <Route element={<Chrome />}>
          <Route path="/wiki" element={<Navigate to="/wiki/home" replace />} />
          <Route path="/wiki/home" element={<Home />} />
          <Route path="/wiki/spaces" element={<Spaces />} />
          <Route path="/wiki/spaces/:key" element={<SpaceShell />}>
            <Route index element={<Navigate to="overview" replace />} />
            <Route path="overview" element={<Overview />} />
            <Route path="pages/:id/*" element={<PageRoute />} />
            <Route path="*" element={<NotFound />} />
          </Route>
          <Route path="/wiki/error/:code" element={<ErrorRoute />} />
          <Route path="*" element={<NotFound />} />
        </Route>
      </Route>
    </Routes>
  );
}
