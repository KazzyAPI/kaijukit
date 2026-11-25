import { Avatar, Button, Dialog, Icon, IconButton, Menu, MenuItem, MenuLabel, MenuSeparator } from '@kaijukit/ui';
import { useEffect, useState } from 'react';
import { Link, useMatch, useNavigate } from 'react-router-dom';
import { useAuth } from './auth';
import { CreatePageDialog } from './CreatePageDialog';
import { CreateSpaceDialog } from './CreateSpaceDialog';
import { QuickSearch } from './QuickSearch';
import { store } from './storage';
import { useTheme } from './theme';

const SHORTCUTS: [string, string][] = [
  ['/', 'Search page titles'],
  ['/ in the editor', 'Open the insert menu'],
  ['Ctrl/Cmd + K', 'Add a link in the editor'],
  ['Ctrl/Cmd + B, I', 'Bold, italic'],
  ['Ctrl/Cmd + Z', 'Undo'],
];

// The only notification today is a local welcome note; the badge clears once the list is opened.
function Notifications() {
  const [read, setRead] = useState(() => store.get('kk.notif.read') === '1');
  return (
    <Menu
      label="Notifications"
      align="end"
      renderTrigger={(p) => (
        <IconButton
          label={read ? 'Notifications' : 'Notifications, 1 unread'}
          icon={
            <span className="kk-badged">
              <Icon name="bell" />
              {!read && <span className="kk-badge" aria-hidden="true">1</span>}
            </span>
          }
          {...p}
          onClick={() => {
            p.onClick();
            setRead(true);
            store.set('kk.notif.read', '1');
          }}
        />
      )}
    >
      <MenuLabel>Notifications</MenuLabel>
      <MenuItem onSelect={() => undefined}>Notifications aren't connected yet. Mentions and page updates will show up here once they are.</MenuItem>
      <MenuSeparator />
      <MenuItem disabled title="Not built yet" onSelect={() => undefined}>Notification settings</MenuItem>
    </Menu>
  );
}

export function TopBar({ navOpen, onToggleNav }: { navOpen: boolean; onToggleNav: () => void }) {
  const { user, logout } = useAuth();
  const navigate = useNavigate();
  const [theme, setTheme] = useTheme();
  const [spaceDialog, setSpaceDialog] = useState(false);
  const [pageDialog, setPageDialog] = useState(false);
  const [search, setSearch] = useState(false);
  const [keys, setKeys] = useState(false);
  const inSpace = useMatch('/wiki/spaces/:key/*')?.params.key;

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const t = e.target as HTMLElement;
      if (e.key === '/' && !e.ctrlKey && !e.metaKey && !t.closest('input, textarea, select, [contenteditable="true"], dialog')) {
        e.preventDefault();
        setSearch(true);
      }
    };
    document.addEventListener('keydown', onKey);
    return () => document.removeEventListener('keydown', onKey);
  }, []);

  return (
    <header className="kk-topbar">
      <IconButton label={navOpen ? 'Collapse sidebar' : 'Show sidebar'} icon={<Icon name="panel" />} aria-expanded={navOpen} onClick={onToggleNav} />
      <>
        <Menu label="Apps" renderTrigger={(p) => <IconButton label="Switch app" icon={<Icon name="grid" />} {...p} />}>
          <MenuLabel>Apps</MenuLabel>
          <MenuItem onSelect={() => navigate('/wiki/home')}>
            <span style={{ width: 16 }}><Icon name="check" /></span>
            Pages
          </MenuItem>
          <MenuItem disabled onSelect={() => undefined}>Other apps aren't built yet</MenuItem>
        </Menu>
      </>
      <Link to="/wiki/home" className="kk-topbar__logo" aria-label="kaijukit home">
        <svg width="24" height="24" viewBox="0 0 32 32" aria-hidden="true">
          <rect width="32" height="32" rx="7" fill="currentColor" />
          <path d="M9 22V10m0 7l8-7m-8 7l8 5" stroke="var(--kk-bg)" strokeWidth="3" fill="none" strokeLinecap="round" />
        </svg>
        <span>kaijukit</span>
      </Link>
      <button type="button" className="kk-topbar__search" onClick={() => setSearch(true)} aria-label="Search" aria-keyshortcuts="/">
        <Icon name="search" />
        <span className="kk-topbar__searchtext">Search</span>
      </button>
      <Menu
        label="Create"
        renderTrigger={(p) => (
          <Button appearance="primary" aria-label="Create" className="kk-topbar__create" {...p}>
            <Icon name="plus" />
            <span className="kk-topbar__createtext">Create</span>
          </Button>
        )}
      >
        <MenuItem onSelect={() => (inSpace ? navigate(`/wiki/spaces/${inSpace}/pages/create`) : setPageDialog(true))}>Page</MenuItem>
        <MenuItem onSelect={() => setSpaceDialog(true)}>Space</MenuItem>
      </Menu>
      <div className="kk-topbar__right">
        <Notifications />
        <>
          <Menu label="Help" align="end" renderTrigger={(p) => <IconButton label="Help" icon={<Icon name="help" />} {...p} />}>
            <MenuItem onSelect={() => setKeys(true)}>Keyboard shortcuts</MenuItem>
            <MenuItem disabled title="Not built yet" onSelect={() => undefined}>Documentation</MenuItem>
          </Menu>
        </>
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
          <MenuItem onSelect={() => navigate('/wiki/tasks')}>Tasks</MenuItem>
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
      <QuickSearch open={search} onClose={() => setSearch(false)} />
      <Dialog open={keys} onClose={() => setKeys(false)} title="Keyboard shortcuts" footer={<Button onClick={() => setKeys(false)}>Close</Button>}>
        <dl className="kk-keys">
          {SHORTCUTS.map(([k, d]) => (
            <div key={k}>
              <dt><kbd>{k}</kbd></dt>
              <dd>{d}</dd>
            </div>
          ))}
        </dl>
      </Dialog>
      <CreateSpaceDialog open={spaceDialog} onClose={() => setSpaceDialog(false)} />
      <CreatePageDialog open={pageDialog} onClose={() => setPageDialog(false)} />
    </header>
  );
}
