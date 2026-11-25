import { Avatar, Breadcrumbs, Button, Dialog, Icon, IconButton, TextField, Tooltip, useToast, type Crumb } from '@kaijukit/ui';
import { useQuery } from '@tanstack/react-query';
import { useState, type ReactNode } from 'react';
import { Link } from 'react-router-dom';
import { restrictionState } from './contentApi';
import { ContentIcon } from './ContentIcon';
import { exactTime, shortDate } from './format';
import { getJson, store } from './storage';
import { useUserNames } from './users';

export function useCopyLink(url: string) {
  const toast = useToast();
  return () =>
    void navigator.clipboard?.writeText(`${location.origin}${url}`).then(
      () => toast({ title: 'Link copied', appearance: 'success' }),
      () => toast({ title: "Couldn't copy the link", appearance: 'warning' }),
    );
}

/** Bar under the top bar: collapse chevron, content icon, breadcrumb-style title, then page-specific controls. */
export function PageHeaderBar({ crumbs, type, children }: { crumbs: Crumb[]; type?: string; children?: ReactNode }) {
  const [collapsed, setCollapsed] = useState(() => getJson('kk.pagehead.collapsed', false));
  const toggle = () => {
    store.set('kk.pagehead.collapsed', String(!collapsed));
    setCollapsed(!collapsed);
  };
  return (
    <div className="kk-pagehead" data-collapsed={collapsed}>
      <IconButton label={collapsed ? 'Expand page header' : 'Collapse page header'} aria-expanded={!collapsed} icon={<Icon name="chevronDown" style={{ transform: collapsed ? 'rotate(-90deg)' : undefined }} />} onClick={toggle} />
      <ContentIcon type={type} />
      <Breadcrumbs items={crumbs} renderLink={({ href, children: c }) => <Link to={href}>{c}</Link>} />
      {!collapsed && <div className="kk-pagehead__right">{children}</div>}
    </div>
  );
}

export function EditedAt({ iso, version }: { iso: string; version: number }) {
  return (
    <time className="kk-pagehead__edited" dateTime={iso} title={`${exactTime(iso)} (version ${version})`}>
      Edited {shortDate(iso)}
    </time>
  );
}

export function Collaborators({ ids }: { ids: (string | undefined)[] }) {
  const unique = [...new Set(ids.filter((i): i is string => !!i))];
  const name = useUserNames(unique);
  return (
    <span className="kk-avatars" role="group" aria-label="Collaborators">
      {unique.map((id) => (
        <Avatar key={id} name={name(id)} size="sm" />
      ))}
    </span>
  );
}

const RESTRICTION_TEXT = {
  OPEN: 'Anyone who can see this space can open this page.',
  VIEW_RESTRICTED: 'Viewing is restricted: only specific people and groups can open this page.',
  EDIT_RESTRICTED: 'Anyone who can see this space can view this page, but only specific people and groups can edit it.',
};

// The lock shows the page's restriction state; changing restrictions and inviting people aren't built yet.
export function ShareControl({ pageId, title, url }: { pageId: string; title: string; url: string }) {
  const [open, setOpen] = useState(false);
  const copy = useCopyLink(url);
  const state = useQuery({ queryKey: ['restriction-state', pageId], queryFn: () => restrictionState(pageId), retry: false }).data ?? 'OPEN';
  const locked = state !== 'OPEN';
  return (
    <>
      <span className="kk-share">
        <Button className="kk-share__main" data-restriction={state} onClick={() => setOpen(true)} aria-label={locked ? 'Share, restricted' : 'Share'}>
          <Icon name={locked ? 'lock' : 'unlock'} />
          <span className="kk-share__text">Share</span>
        </Button>
        <IconButton label="Copy link" icon={<Icon name="link" />} appearance="default" className="kk-share__copy" onClick={copy} />
      </span>
      <Dialog open={open} onClose={() => setOpen(false)} title={`Share "${title || 'Untitled'}"`} footer={<Button onClick={() => setOpen(false)}>Done</Button>}>
        <TextField label="Page link" readOnly value={`${location.origin}${url}`} onFocus={(e) => e.currentTarget.select()} />
        <Button onClick={copy}>
          <Icon name="copy" /> Copy link
        </Button>
        <p className="kk-muted" style={{ margin: 0 }}>
          <Icon name={locked ? 'lock' : 'unlock'} size={14} /> {RESTRICTION_TEXT[state]} Inviting people and editing restrictions aren't built yet.
        </p>
      </Dialog>
    </>
  );
}

export function NotBuilt({ icon, label }: { icon: Parameters<typeof Icon>[0]['name']; label: string }) {
  return (
    <Tooltip content={`${label} is not built yet`}>
      <Button appearance="subtle" className="kk-byline__stub" aria-disabled="true" onClick={(e) => e.preventDefault()}>
        <Icon name={icon} size={14} />
        {label}
      </Button>
    </Tooltip>
  );
}
