# 01 Data model

Last reviewed: 2026-11-24

## Entities

The fields below are what the reference exposes on the wire. Internal table layout is our own design (see [ADR 0002](../../../decisions/0002-own-stack.md)); only wire shape is a parity concern.

| Entity | Identity | Key fields |
|---|---|---|
| Space | numeric `id` (string in JSON), unique `key` | name, type (`global`,`personal`; others `[V]` e.g. knowledge base), status (`current`,`archived`), homepageId, description (plain+view), authorId, createdAt, currentActiveAlias, icon, labels, permissions, roles, settings, theme/look-and-feel |
| Content (base) | numeric `id` | type, status, title, spaceId, authorId, createdAt, version, body (multiple representations), parentId, parentType, position, labels, properties, restrictions, operations |
| Page | Content, type `page` | `subtype` `live` or `page` (live docs) `[V]`, parent = page/folder/whiteboard/database/smart-link or space root |
| Blog post | Content, type `blogpost` | posting date; title unique per space per day `[V]` |
| Folder, Whiteboard, Database, Smart Link (`embed`) | Content in the tree | share the hierarchy endpoints (children/descendants/ancestors) |
| Custom content | Content, type `ac:<app-key>:<type>` | app-defined body (`raw`, `storage`, ...), container (space/page/blog/custom), used by apps |
| Comment | Content | `footer` or `inline`; parent comment; pageId/blogPostId/attachmentId/customContentId; inline: markerRef, originalSelection, resolutionStatus, resolutionLastModifierId, resolutionLastModifiedAt |
| Attachment | Content | mediaType, fileSize, fileId, mediaTypeDescription, webuiLink, downloadLink, comment, version; container page/blog/custom |
| Version | per content | number (from 1), message, minorEdit, authorId, createdAt, page body snapshot |
| Label | id, name, prefix (`global`,`my`,`team`,`system`) | attached to content, spaces, attachments, templates `[V]` |
| Content property | id, key, value (JSON), version | per content; also space properties, app properties (per app, not per content) |
| User | `accountId` | displayName, email (privacy-gated), publicName, profilePicture, accountType (`atlassian`,`app`,`customer`), timeZone, locale, isExternalCollaborator `[V]` |
| Group | id, name | members |
| Restriction | content + operation (`read`,`update`) | subjects: users + groups |
| Space permission | space + subject + operation(key,target) | see [04](04-permissions.md) |
| Template | id, name, type (`page`,`blogpost`), body, labels, space or global | blueprints `[V]` |
| Task | id, localId, spaceId, pageId, blogPostId, status (`complete`,`incomplete`), body, createdBy, assignedTo, completedBy, createdAt, updatedAt, dueAt, completedAt | extracted from body task lists |
| Like | content + user | |
| Watcher | content or space + user | |
| Content state | id, name, color, spaceId | lightweight status on a page (v1 "Content states") |
| Classification level | id, name, status, color, description | data classification |
| Redaction | content + range | v2 Redactions |
| Audit record | author, remoteAddress, creationDate, summary, description, category, sysAdmin, affectedObject, changedValues, associatedObjects | |
| Long-running task | id, status, percentageComplete, elapsedTime | export, copy, move etc. |

## Relationships

- A space has exactly one homepage (page) and any number of root-level pages (outside the homepage tree).
- Content tree: parent -> children with an integer `position` per parent. Ancestors/descendants are derived from the tree (v2 `Ancestors`, `Children`, `Descendants` ops). Descendants have a depth parameter.
- Content `status` values: `current`, `draft`, `trashed`, `historical`, `archived`, `deleted` (purged, only visible in audit). Each content has at most one `current` row; `historical` rows are prior versions; a `draft` may exist alongside a `current` one (unpublished edits, "draft of a published page") `[V]`.
- Children of a trashed page are trashed with it and restored with it `[V]`.
- Labels, properties and restrictions belong to the content id, not to a version.

## Persistence design constraints (our implementation)

- Postgres. Entity ids are 64-bit, allocated from one sequence shared by all content types so ids are globally unique across types (parity: v1 and v2 accept the same id for the same content).
- Body stored per (content id, version, representation); canonical stored form is ADF. Storage format is derived on read and cached, and is accepted on write (converted to ADF). See [03](03-body-formats.md).
- Custom field-like data (properties, app properties) is JSONB with the size limit of the reference `[V]`.
- Soft delete: `trashed` row retained with `trashedAt`; purge job removes after retention `[V]` (Cloud default retention to confirm).
- Every write appends an audit/event row in the same transaction (see [11](11-events-jobs.md)).
