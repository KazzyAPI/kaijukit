import { Icon, type IconName } from '@kaijukit/ui';

const ICONS: Record<string, IconName> = { page: 'page', whiteboard: 'whiteboard', database: 'database', folder: 'folder', blogpost: 'blog' };

export const ContentIcon = ({ type = 'page', size = 16 }: { type?: string; size?: number }) => <Icon name={ICONS[type] ?? 'page'} size={size} />;
