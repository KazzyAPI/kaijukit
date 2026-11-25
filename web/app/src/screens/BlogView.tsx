import { parseAdf, wordCount } from '@kaijukit/editor/adf';
import { AdfView } from '@kaijukit/editor/render';
import { Avatar, Icon, Skeleton } from '@kaijukit/ui';
import { useQuery } from '@tanstack/react-query';
import { useOutletContext, useParams } from 'react-router-dom';
import { resolveMedia, type Space } from '../api';
import { getBlog } from '../contentApi';
import { shortDate, readingMinutes } from '../format';
import { PageHeaderBar } from '../PageHeader';
import { QueryState } from '../QueryState';
import { useUserNames } from '../users';
import { NotFound } from './Errors';

export function BlogView() {
  const { id } = useParams();
  const space = useOutletContext<Space>();
  const blog = useQuery({ queryKey: ['blog', id], queryFn: () => getBlog(id!) });
  const name = useUserNames([blog.data?.authorId]);
  return (
    <div className="kk-pane">
      <QueryState query={blog} skeleton={<div className="kk-canvas"><Skeleton height={40} width="60%" style={{ margin: '16px 0 12px' }} /><Skeleton height={14} /></div>}>
        {(b) => {
          if (b.spaceId !== space.id) return <NotFound />;
          const doc = parseAdf(b.body?.atlas_doc_format?.value);
          return (
            <>
              <PageHeaderBar type="blogpost" crumbs={[{ label: space.name, href: `/wiki/spaces/${space.key}/overview` }, { label: b.title || 'Untitled' }]} />
              <div className="kk-pagebody">
                <article className="kk-canvas">
                  <h1 className="kk-canvas__title">{b.title || 'Untitled'}</h1>
                  <div className="kk-byline">
                    <span className="kk-byline__item">
                      <Avatar name={name(b.authorId)} size="sm" />
                      By {name(b.authorId)}
                    </span>
                    <span className="kk-byline__item">{shortDate(b.createdAt)}</span>
                    <span className="kk-byline__item">
                      <Icon name="book" size={14} />
                      {readingMinutes(wordCount(doc))} min read
                    </span>
                  </div>
                  <AdfView doc={doc} resolveMedia={resolveMedia} />
                  <p className="kk-muted">Editing blog posts, comments and labels on them aren't built yet.</p>
                </article>
              </div>
            </>
          );
        }}
      </QueryState>
    </div>
  );
}
