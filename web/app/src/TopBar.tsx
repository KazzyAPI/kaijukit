import { Avatar, Button, Icon, Menu, MenuItem, MenuLabel, MenuSeparator } from '@kaijukit/ui';
import { useQuery } from '@tanstack/react-query';
import { useState } from 'react';
import { Link, useMatch, useNavigate } from 'react-router-dom';
import { listSpaces } from './api';
import { useAuth } from './auth';
import { CreateSpaceDialog } from './CreateSpaceDialog';
import { CreatePageDialog } from './CreatePageDialog';
import { useTheme } from './theme';

export function TopBar() {
  const { user, logout } = useAuth();
  const navigate = useNavigate();
  const [theme, setTheme] = useTheme();
  const [spaceDialog, setSpaceDialog] = useState(false);
  const [pageDialog, setPageDialog] = useState(false);
  const spaces = useQuery({ queryKey: ['spaces', 'current'], queryFn: () => listSpaces() });
  const inSpace = useMatch('/wiki/spaces/:key/*')?.params.key;

  return (
    <header className="kk-topbar">
      <Link to="/wiki/home" className="kk-topbar__logo">
        <svg width="24" height="24" viewBox="0 0 32 32" aria-hidden="true">
          <rect width="32" height="32" rx="7" fill="currentColor" />
          <path d="M9 22V10m0 7l8-7m-8 7l8 5" stroke="var(--kk-bg)" strokeWidth="3" fill="none" strokeLinecap="round" />
        </svg>
        <span>kaijukit</span>
      </Link>
      <nav aria-label="Global" className="kk-topbar__nav">
        <Menu
          label="Spaces"
          renderTrigger={(p) => (
            <Button appearance="subtle" {...p}>
              Spaces
              <Icon name="chevronRight" size={12} style={{ transform: 'rotate(90deg)' }} />
            </Button>
          )}
        >
          <MenuLabel>Spaces</MenuLabel>
          {spaces.data?.slice(0, 8).map((s) => (
            <MenuItem key={s.id} onSelect={() => navigate(`/wiki/spaces/${s.key}/overview`)}>
              {s.name}
            </MenuItem>
          ))}
          {spaces.isPending && <MenuLabel>Loading...</MenuLabel>}
          {spaces.data?.length === 0 && <MenuLabel>No spaces yet</MenuLabel>}
          <MenuSeparator />
          <MenuItem onSelect={() => navigate('/wiki/spaces')}>View all spaces</MenuItem>
        </Menu>
        <Menu
          label="Create"
          renderTrigger={(p) => (
            <Button appearance="primary" {...p}>
              <Icon name="plus" />
              Create
            </Button>
          )}
        >
          <MenuItem onSelect={() => (inSpace ? navigate(`/wiki/spaces/${inSpace}/pages/create`) : setPageDialog(true))}>Page</MenuItem>
          <MenuItem onSelect={() => setSpaceDialog(true)}>Space</MenuItem>
        </Menu>
      </nav>
      <div className="kk-topbar__right">
        <Menu
          label="Account"
          align="end"
          renderTrigger={(p) => (
            <Button appearance="subtle" className="kk-btn--icon" aria-label="Account" {...p}>
              <Avatar name={user?.email ?? 'Account'} size="sm" />
            </Button>
          )}
        >
          <MenuLabel>{user?.email}</MenuLabel>
          <MenuSeparator />
          <MenuLabel>Theme</MenuLabel>
          {(['light', 'dark', 'auto'] as const).map((t) => (
            <MenuItem key={t} onSelect={() => setTheme(t)}>
              <span style={{ width: 16 }}>{theme === t && <Icon name="check" />}</span>
              {t === 'auto' ? 'Match system' : t[0]!.toUpperCase() + t.slice(1)}
            </MenuItem>
          ))}
          <MenuSeparator />
          <MenuItem
            onSelect={() => {
              logout();
              navigate('/wiki/login');
            }}
          >
            Log out
          </MenuItem>
        </Menu>
      </div>
      <CreateSpaceDialog open={spaceDialog} onClose={() => setSpaceDialog(false)} />
      <CreatePageDialog open={pageDialog} onClose={() => setPageDialog(false)} />
    </header>
  );
}
