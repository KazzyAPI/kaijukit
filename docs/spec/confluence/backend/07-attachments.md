# 07 Attachments and media

Last reviewed: 2026-11-24

Structure: [v2 Attachment](../../../parity/confluence-api/v2/attachment.md), [v1 Content - attachments](../../../parity/confluence-api/v1/content-attachments.md).

- **Upload:** v1 `multipart/form-data` (`file`, optional `comment`, `minorEdit`) to a content id; creates attachment or a new version if the filename already exists on that content (same filename = new version). Requires `create/attachment` on the space and edit access to the container.
- **Limits:** max file size is a site setting (default `[V]`); over limit -> 413 with the reference message. Blocked extensions/MIME list is configurable. Filename normalization (trim, forbidden chars) `[V]`.
- **Metadata:** title (filename), mediaType (sniffed from content, not trusted from the client), fileSize, fileId (UUID used by the media viewer), comment, version, `webuiLink`, `downloadLink` (`/wiki/download/attachments/{containerId}/{filename}?version=&modificationDate=&api=v2`), thumbnails.
- **Download:** authorizes against the container's read access; `Content-Disposition` attachment for unsafe types; inline for images/PDF; Range requests supported; cache headers.
- **Versions:** each upload of an existing name is a version; list versions, fetch a version, delete a version, restore (v1).
- **Delete:** moves to trash with the container? Attachments have their own trash state; purge permanently `[V]`.
- **Move/copy:** follow the container; copying a page copies attachments when requested.
- **Preview/processing:** async jobs generate thumbnails (images), page renders (PDF, Office via converter `[V]`), text extraction for search, antivirus scan hook (pluggable, off by default) before the file becomes downloadable.
- **Storage:** object store interface (S3-compatible or local FS), content-addressed by sha256 with reference counts; DB holds metadata.
- **Embedding:** storage `ri:attachment` / ADF `media`+`mediaSingle` referencing `fileId` + collection; renaming an attachment keeps `fileId` stable but rewrites filename-based storage references `[V]`.
- **Labels/properties/comments** on attachments follow the same APIs as other content.
- **Media API compatibility:** the editor uploads through a media endpoint (token + `fileId`) rather than the REST attachment API `[V]`; implement both: REST (compat) and an internal media service the UI uses, both producing the same attachment rows.
