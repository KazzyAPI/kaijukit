import { Button, Icon, useToast } from '@kaijukit/ui';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { errorTitle } from './api';
import { setWatch, watchStatus } from './permApi';

export const useWatch = (kind: 'content' | 'space', id: string) => {
  const qc = useQueryClient();
  const toast = useToast();
  const key = ['watch', kind, id];
  const status = useQuery({ queryKey: key, queryFn: () => watchStatus(kind, id), retry: false });
  const toggle = useMutation({
    mutationFn: (on: boolean) => setWatch(kind, id, on),
    onSuccess: (_, on) => {
      qc.setQueryData(key, on);
      toast({ title: on ? `You're watching this ${kind === 'space' ? 'space' : 'page'}` : 'Stopped watching', appearance: 'success' });
    },
    onError: (e) => toast({ title: errorTitle(e, "Couldn't change watching"), description: e.message, appearance: 'error' }),
  });
  return { watching: status.data, ready: status.isSuccess, toggle: () => toggle.mutate(!status.data), pending: toggle.isPending };
};

/** Byline toggle; hidden when the server has no watch endpoint (status query failed). */
export function WatchButton({ kind, id }: { kind: 'content' | 'space'; id: string }) {
  const w = useWatch(kind, id);
  if (!w.ready) return null;
  return (
    <Button appearance="subtle" className="kk-byline__stub" aria-pressed={!!w.watching} loading={w.pending} onClick={w.toggle}>
      <Icon name="eye" size={14} />
      {w.watching ? 'Watching' : 'Watch'}
    </Button>
  );
}
