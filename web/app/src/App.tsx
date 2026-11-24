import { Skeleton } from '@kaijukit/ui';
import { lazy, Suspense } from 'react';
import { Navigate, Outlet, Route, Routes, useLocation, useParams } from 'react-router-dom';
import { useAuth } from './auth';
import { Chrome } from './Chrome';
import { ErrorPage, NotFound } from './screens/Errors';
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
        <Route element={<Chrome />}>
          <Route path="/wiki" element={<Navigate to="/wiki/home" replace />} />
          <Route path="/wiki/home" element={<Home />} />
          <Route path="/wiki/spaces" element={<Spaces />} />
          <Route path="/wiki/spaces/:key" element={<SpaceShell />}>
            <Route index element={<Navigate to="overview" replace />} />
            <Route path="overview" element={<Overview />} />
            <Route path="pages/edit-v2/:id" element={<Suspense fallback={<Loading />}><EditorRoute /></Suspense>} />
            <Route path="pages/create" element={<Suspense fallback={<Loading />}><CreateRoute /></Suspense>} />
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
