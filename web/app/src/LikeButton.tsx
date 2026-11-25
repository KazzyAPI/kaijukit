import { type Kind } from './api';
import { Button, Icon, Tooltip, useToast } from '@kaijukit/ui';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { pageLikes, setLike, type Likes } from './contentApi';
import { useMyId, useUserNames } from './users';

export function LikeButton({ pageId, of = 'page' }: { pageId: string; of?: Kind }) {
  const qc = useQueryClient();
  const toast = useToast();
  const me = useMyId();
  const key = ['likes', pageId];
  const likes = useQuery({ queryKey: key, queryFn: () => pageLikes(pageId, of) });
  const name = useUserNames(likes.data?.accountIds ?? []);
  const liked = !!me && !!likes.data?.accountIds.includes(me);
  const toggle = useMutation({
    mutationFn: (next: boolean) => setLike(pageId, next),
    onMutate: async (next) => {
      await qc.cancelQueries({ queryKey: key });
      const before = qc.getQueryData<Likes>(key);
      if (before && me) qc.setQueryData<Likes>(key, { count: before.count + (next ? 1 : -1), accountIds: next ? [...before.accountIds, me] : before.accountIds.filter((i) => i !== me) });
      return { before };
    },
    onError: (e, _n, ctx) => {
      qc.setQueryData(key, ctx?.before);
      toast({ title: "Couldn't update your like", description: e.message, appearance: 'error' });
    },
    onSettled: () => qc.invalidateQueries({ queryKey: key }),
  });
  const count = likes.data?.count ?? 0;
  const who = likes.data?.accountIds.map(name).join(', ');
  return (
    <Tooltip content={who || 'Be the first to like this'}>
      <Button appearance="subtle" className="kk-like" aria-pressed={liked} disabled={!likes.data || !me} onClick={() => toggle.mutate(!liked)}>
        <Icon name="thumbUp" size={14} />
        {liked ? 'Liked' : 'Like'}
        <span className="kk-like__count" aria-label={`${count} ${count === 1 ? 'like' : 'likes'}`}>{count}</span>
      </Button>
    </Tooltip>
  );
}
