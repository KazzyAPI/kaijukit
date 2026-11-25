import { Avatar, Icon, IconButton, Menu, MenuItem, MenuLabel, MenuSeparator, Skeleton, type IconName } from '@kaijukit/ui';
import { useQuery } from '@tanstack/react-query';
import { useId, useState, type ReactNode } from 'react';
import { NavLink, useNavigate } from 'react-router-dom';
import { listSpaces, recentPages, type Space } from './api';
import { pageUrl } from './format';
import { useCurrentPageId } from './hooks';
import { PageTree } from './PageTree';
import { CreateKindDialog } from './CreateKinds';
import { spaceBlogs } from './contentApi';

function Flyout({ icon, label, chevron = true, children }: { icon: IconName; label: string; chevron?: boolean; children: ReactNode }) {
  return (
    <Menu
      label={label}
      side="right"
      renderTrigger={(p) => (
        <button type="button" className="kk-nav__row" {...p}>
          <Icon name={icon} />
          <span className="kk-nav__label">{label}</span>
          {chevron && <Icon name="chevronRight" size={14} />}
        </button>
      )}
    >
      {children}
    </Menu>
  );
}

// A menu needs at least one menuitem child to be valid ARIA, so empty states are disabled items.
const Note = ({ children }: { children: ReactNode }) => <MenuItem disabled onSelect={() => undefined}>{children}</MenuItem>;

function RecentItems() {
  const navigate = useNavigate();
  const recent = useQuery({ queryKey: ['recent'], queryFn: recentPages });
  const spaces = useQuery({ queryKey: ['spaces', 'current'], queryFn: () => listSpaces() });
  if (recent.isPending) return <Note>Loading...</Note>;
  if (recent.isError) return <Note>Couldn't load recent pages</Note>;
  if (!recent.data.length) return <Note>No recent pages yet</Note>;
  return (
    <>
      <MenuLabel>Recently updated</MenuLabel>
      {recent.data.slice(0, 8).map((p) => {
        const key = spaces.data?.find((s) => s.id === p.spaceId)?.key;
        return key ? (
          <MenuItem key={p.id} onSelect={() => navigate(pageUrl(key, p.id, p.title))}>
            {p.title || 'Untitled'}
          </MenuItem>
        ) : null;
      })}
    </>
  );
}

function MoreItems() {
  const navigate = useNavigate();
  return <MenuItem onSelect={() => navigate('/wiki/admin/groups')}>Groups</MenuItem>;
}

function SpaceItems() {
  const navigate = useNavigate();
  const spaces = useQuery({ queryKey: ['spaces', 'current'], queryFn: () => listSpaces() });
  return (
    <>
      <MenuLabel>Spaces</MenuLabel>
      {spaces.isPending && <Note>Loading...</Note>}
      {spaces.isError && <Note>Couldn't load spaces</Note>}
      {spaces.data?.length === 0 && <Note>No spaces yet</Note>}
      {spaces.data?.slice(0, 10).map((s) => (
        <MenuItem key={s.id} onSelect={() => navigate(`/wiki/spaces/${s.key}/overview`)}>
          {s.name}
        </MenuItem>
      ))}
      <MenuSeparator />
      <MenuItem onSelect={() => navigate('/wiki/spaces')}>View all spaces</MenuItem>
    </>
  );
}

function Section({ title, actions, children }: { title: string; actions?: ReactNode; children: ReactNode }) {
  const [open, setOpen] = useState(true);
  const id = useId();
  return (
    <section className="kk-nav__sec" aria-label={title}>
      <div className="kk-nav__sechead">
        <h2 className="kk-nav__sectitle">
          <button type="button" aria-expanded={open} aria-controls={id} onClick={() => setOpen(!open)}>
            <Icon name="chevronRight" size={14} style={{ transform: open ? 'rotate(90deg)' : undefined }} />
            {title}
          </button>
        </h2>
        <span className="kk-nav__secactions">{actions}</span>
      </div>
      <div id={id} hidden={!open}>
        {children}
      </div>
    </section>
  );
}

const todoItem = (label: string) => (
  <MenuItem disabled title="Not built yet" onSelect={() => undefined}>
    {label}
  </MenuItem>
);

function SpaceBlock({ space }: { space: Space }) {
  const navigate = useNavigate();
  const currentId = useCurrentPageId();
  const create = () => navigate(`/wiki/spaces/${space.key}/pages/create`);
  const [kind, setKind] = useState<'blog' | 'folder' | null>(null);
  const blogs = useQuery({ queryKey: ['blogs', space.id], queryFn: () => spaceBlogs(space.id) });
  return (
    <div className="kk-nav__group">
      <div className="kk-nav__space">
        <NavLink to={`/wiki/spaces/${space.key}/overview`} className="kk-nav__spacelink" end>
          <Avatar name={space.name} shape="square" size="sm" />
          <span className="kk-nav__label">{space.name}</span>
        </NavLink>
        <Menu label="Space actions" align="end" renderTrigger={(t) => <IconButton label="Space actions" icon={<Icon name="more" />} {...t} />}>
          <MenuItem onSelect={() => navigate(`/wiki/spaces/${space.key}/overview`)}>Space overview</MenuItem>
          <MenuItem onSelect={() => navigate('/wiki/spaces')}>All spaces</MenuItem>
          <MenuItem onSelect={() => navigate(`/wiki/spaces/${space.key}/settings`)}>Space settings</MenuItem>
        </Menu>
      </div>
      <Section title="Shortcuts">
        <p className="kk-nav__empty">Shortcuts aren't built yet.</p>
      </Section>
      <Section
        title="Content"
        actions={
          <>
            <IconButton label="Create page" icon={<Icon name="plus" />} onClick={create} />
            <Menu label="Content actions" align="end" renderTrigger={(t) => <IconButton label="Content actions" icon={<Icon name="more" />} {...t} />}>
              <MenuItem onSelect={create}>New page</MenuItem>
              <MenuItem onSelect={() => setKind('folder')}>New folder</MenuItem>
              {todoItem('Reorder pages')}
            </Menu>
          </>
        }
      >
        <PageTree space={space} currentId={currentId} />
        <button type="button" className="kk-nav__row kk-nav__create" onClick={create}>
          <Icon name="plus" />
          <span className="kk-nav__label">Create</span>
        </button>
      </Section>
      <Section title="Blogs" actions={<IconButton label="Create blog post" icon={<Icon name="plus" />} onClick={() => setKind('blog')} />}>
        {blogs.isError && <p className="kk-nav__empty" role="alert">Couldn't load blog posts</p>}
        {blogs.data?.length === 0 && <p className="kk-nav__empty">No blogs in this space</p>}
        {blogs.data?.map((b) => (
          <NavLink key={b.id} to={`/wiki/spaces/${space.key}/blog/${b.id}`} className="kk-nav__row">
            <Icon name="blog" />
            <span className="kk-nav__label">{b.title || 'Untitled'}</span>
          </NavLink>
        ))}
      </Section>
      <CreateKindDialog kind={kind} space={space} onClose={() => setKind(null)} />
      <Section title="Calendars">
        <p className="kk-nav__empty">No calendars in this space</p>
      </Section>
      <Section
        title="Space apps"
        actions={
          <Menu label="Space apps actions" align="end" renderTrigger={(t) => <IconButton label="Space apps actions" icon={<Icon name="more" />} {...t} />}>
            {todoItem('Manage apps')}
          </Menu>
        }
      >
        <p className="kk-nav__empty">No apps installed</p>
      </Section>
    </div>
  );
}

export function LeftNav({ space, loading }: { space?: Space; loading: boolean }) {
  return (
    <nav className="kk-sidebar kk-nav" aria-label="Sidebar">
      <div className="kk-nav__group">
        <NavLink to="/wiki/home" className="kk-nav__row">
          <Icon name="home" />
          <span className="kk-nav__label">For you</span>
        </NavLink>
        <NavLink to="/wiki/tasks" className="kk-nav__row">
          <Icon name="checkbox" />
          <span className="kk-nav__label">Tasks</span>
        </NavLink>
        <Flyout icon="clock" label="Recent">
          <RecentItems />
        </Flyout>
        <Flyout icon="star" label="Starred">
          <Note>Nothing starred yet. Starring pages isn't built yet.</Note>
        </Flyout>
        <Flyout icon="space" label="Spaces">
          <SpaceItems />
        </Flyout>
        <Flyout icon="grid" label="Apps">
          <Note>Apps aren't built yet</Note>
        </Flyout>
        <Flyout icon="users" label="Teams">
          <Note>Teams aren't built yet</Note>
        </Flyout>
        <Flyout icon="more" label="More" chevron={false}>
          <MoreItems />
        </Flyout>
      </div>
      {space ? (
        <SpaceBlock space={space} />
      ) : (
        loading && (
          <div aria-busy="true" className="kk-nav__group">
            <Skeleton height={32} style={{ margin: '8px 0' }} />
            <Skeleton height={20} style={{ margin: '8px 0' }} />
            <Skeleton height={20} style={{ margin: '8px 0' }} />
          </div>
        )
      )}
    </nav>
  );
}
