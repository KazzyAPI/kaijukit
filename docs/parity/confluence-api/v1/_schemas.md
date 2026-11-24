# Confluence REST v1: schemas

Last reviewed: 2026-11-24 · generated · structure only.

## AccountId

object

- `accountId`: string **required**

## AccountIdEmailRecord

object

- `accountId`: string **required**
- `email`: string **required**

## AccountIdEmailRecordArray

array<`AccountIdEmailRecord`>


## AddContentRestriction

object

- `operation`: string (enum read|update) **required**
- `restrictions`: object **required**
  - `user`: array<object>
    - `type`: string (enum known|unknown|anonymous|user) **required**
    - `username`: `GenericUserName`
    - `userKey`: `GenericUserKey`
    - `accountId`: `GenericAccountId` **required**
  - `group`: array<object>
    - `type`: string (enum group) **required**
    - `name`: string **required**

## AddContentRestrictionUpdateArray

array<`AddContentRestriction`>


## AffectedObject

object

- `name`: string **required**
- `objectType`: string **required**

## AsyncContentBody

object

- `value`: string
- `representation`: string (enum view|export_view|styled_view|storage|editor|editor2|anonymous_export_view|wiki|atlas_doc_format)
- `renderTaskId`: string
- `error`: string
- `status`: string (enum WORKING|QUEUED|FAILED|COMPLETED|RERUNNING)
- `embeddedContent`: array<`EmbeddedContent`>
- `webresource`: `WebResourceDependencies`
- `mediaToken`: object
  - `collectionIds`: array<string>
  - `contentId`: string
  - `expiryDateTime`: string
  - `fileIds`: array<string>
  - `token`: string
- `_expandable`: object
  - `content`: string
  - `embeddedContent`: string
  - `webresource`: string
  - `mediaToken`: string
- `_links`: `GenericLinks`

## AsyncContentBodyArray

array<`AsyncContentBody`>


## AsyncId

object

- `asyncId`: string **required**

## AsyncIdArray

array<`AsyncId`>


## AttachmentPropertiesUpdateBody

object

- `id`: string **required**
- `type`: string **required**
- `status`: string
- `title`: string
- `container`: `Container`
- `metadata`: object
  - `mediaType`: string
- `extensions`: object
- `version`: `Version` **required**

## AttachmentUpdate

object

- `version`: object **required**
  - `number`: integer (int32) **required**
- `id`: string **required**
- `type`: string (enum attachment) **required**
- `title`: string (maxLength 255)
- `metadata`: object
  - `mediaType`: string
  - `comment`: string
- `container`: object
  - `id`: string **required**
  - `type`: string **required**

## AuditRecord

object

- `author`: object **required**
  - `type`: string (enum user, default "user") **required**
  - `displayName`: string **required**
  - `operations`: array<`OperationCheckResult`> (nullable)
  - `username`: `GenericUserName`
  - `userKey`: `GenericUserKey`
  - `accountId`: `GenericAccountId`
  - `accountType`: string
  - `externalCollaborator`: boolean
  - `isExternalCollaborator`: boolean
  - `isGuest`: boolean
  - `publicName`: string
- `remoteAddress`: string **required**
- `creationDate`: integer (int64) **required**
- `summary`: string **required**
- `description`: string **required**
- `category`: string **required**
- `sysAdmin`: boolean **required**
- `superAdmin`: boolean
- `affectedObject`: `AffectedObject` **required**
- `changedValues`: array<`ChangedValue`> **required**
- `associatedObjects`: array<`AffectedObject`> **required**

## AuditRecordArray

object

- `results`: array<`AuditRecord`> **required**
- `start`: integer (int32) **required**
- `limit`: integer (int32) **required**
- `size`: integer (int32) **required**
- `_links`: `GenericLinks` **required**

## AuditRecordCreate

object

- `author`: object
  - `type`: string (enum user, default "user") **required**
  - `displayName`: string
  - `operations`: array<`OperationCheckResult`>
  - `username`: `GenericUserName`
  - `userKey`: `GenericUserKey`
- `remoteAddress`: string **required**
- `creationDate`: integer (int64)
- `summary`: string
- `description`: string
- `category`: string
- `sysAdmin`: boolean (default false)
- `affectedObject`: `AffectedObject`
- `changedValues`: array<`ChangedValue`>
- `associatedObjects`: array<`AffectedObject`>

## AvailableContentStates

object

- `spaceContentStates`: array<`ContentState`> **required**
- `customContentStates`: array<`ContentState`> **required**

## BlueprintTemplate

object

- `templateId`: string **required**
- `originalTemplate`: object **required**
  - `pluginKey`: string **required**
  - `moduleKey`: string **required**
- `referencingBlueprint`: string **required**
- `name`: string **required**
- `description`: string **required**
- `space`: object
- `labels`: array<`Label`> **required**
- `templateType`: string **required**
- `editorVersion`: string
- `body`: `ContentTemplateBody`
- `_expandable`: object
  - `body`: string
- `_links`: `GenericLinks` **required**

## BlueprintTemplateArray

object

- `results`: array<`BlueprintTemplate`> **required**
- `start`: integer (int32) **required**
- `limit`: integer (int32) **required**
- `size`: integer (int32) **required**
- `_links`: `GenericLinks` **required**

## Breadcrumb

object

- `label`: string **required**
- `url`: string **required**
- `separator`: string **required**

## BulkContentBodyConversionInput

object

- `conversionInputs`: array<`ContentBodyConversionInput`>

## BulkUserLookup

object

- `type`: string (enum known|unknown|anonymous|user) **required**
- `username`: `GenericUserName`
- `userKey`: `GenericUserKey`
- `accountId`: `GenericAccountId` **required**
- `accountType`: string **required**
- `email`: string **required**
- `publicName`: string **required**
- `profilePicture`: `Icon` **required**
- `displayName`: string **required**
- `timeZone`: string (nullable)
- `isExternalCollaborator`: boolean
- `isGuest`: boolean
- `operations`: array<`OperationCheckResult`>
- `details`: `UserDetails`
- `personalSpace`: `Space`
- `_expandable`: object **required**
  - `operations`: string
  - `details`: string
  - `personalSpace`: string
- `_links`: `GenericLinks` **required**

## BulkUserLookupArray

object

- `results`: array<`BulkUserLookup`> **required**
- `start`: integer (int32) **required**
- `limit`: integer (int32) **required**
- `size`: integer (int32) **required**
- `_links`: `GenericLinks` **required**

## ButtonLookAndFeel

object

- `backgroundColor`: string **required**
- `color`: string **required**

## CQLPersonalDataConvertedQueries

object

- `queryStrings`: array<string> **required**

## CQLPersonalDataMigrationRequest

object

- `queryStrings`: array<string> **required**

## ChangedValue

object

- `name`: string **required**
- `oldValue`: string **required**
- `hiddenOldValue`: string
- `newValue`: string **required**
- `hiddenNewValue`: string

## ConnectModule

object


## ConnectModules

object

- `modules`: array<`ConnectModule`> **required**

## Container

object (nullable)


## ContainerLookAndFeel

object

- `background`: string **required**
- `backgroundAttachment`: string (nullable)
- `backgroundBlendMode`: string (nullable)
- `backgroundClip`: string (nullable)
- `backgroundColor`: string (nullable) **required**
- `backgroundImage`: string (nullable) **required**
- `backgroundOrigin`: string (nullable)
- `backgroundPosition`: string (nullable)
- `backgroundRepeat`: string (nullable)
- `backgroundSize`: string (nullable) **required**
- `padding`: string **required**
- `borderRadius`: string **required**

## ContainerSummary

object

- `title`: string **required**
- `displayUrl`: string **required**

## Content

object

- `id`: string
- `type`: string **required**
- `status`: string **required**
- `title`: string
- `space`: `Space`
- `history`: `ContentHistory`
- `version`: `Version`
- `ancestors`: array<`Content`> (nullable)
- `operations`: array<`OperationCheckResult`>
- `children`: `ContentChildren`
- `childTypes`: `ContentChildType`
- `descendants`: `ContentChildren`
- `container`: `Container`
- `body`: object
  - `view`: `ContentBody`
  - `export_view`: `ContentBody`
  - `styled_view`: `ContentBody`
  - `storage`: `ContentBody`
  - `wiki`: `ContentBody`
  - `editor`: `ContentBody`
  - `editor2`: `ContentBody`
  - `anonymous_export_view`: `ContentBody`
  - `atlas_doc_format`: `ContentBody`
  - `dynamic`: `ContentBody`
  - `raw`: `ContentBody`
  - `_expandable`: object
    - `editor`: string
    - `view`: string
    - `export_view`: string
    - `styled_view`: string
    - `storage`: string
    - `editor2`: string
    - `anonymous_export_view`: string
    - `atlas_doc_format`: string
    - `wiki`: string
    - `dynamic`: string
    - `raw`: string
- `restrictions`: object
  - `read`: `ContentRestriction`
  - `update`: `ContentRestriction`
  - `_expandable`: object
    - `read`: string
    - `update`: string
  - `_links`: `GenericLinks`
- `metadata`: `ContentMetadata`
- `macroRenderedOutput`: map<string,object>
- `extensions`: object
- `_expandable`: object
  - `childTypes`: string
  - `container`: string
  - `metadata`: string
  - `operations`: string
  - `children`: string
  - `restrictions`: string
  - `history`: string
  - `ancestors`: string
  - `body`: string
  - `version`: string
  - `descendants`: string
  - `space`: string
  - `extensions`: string
  - `schedulePublishDate`: string
  - `schedulePublishInfo`: string
  - `macroRenderedOutput`: string
- `_links`: `GenericLinks`

## ContentArray

object

- `results`: array<`Content`> **required**
- `start`: integer (int32)
- `limit`: integer (int32)
- `size`: integer (int32) **required**
- `_links`: `GenericLinks` **required**

## ContentBlogpost

`Content` & object

- `metadata`: `ContentMetadata` **required**
- `_links`: `GenericLinks` **required**

## ContentBlueprintDraft

object

- `version`: object **required**
  - `number`: integer (int32) **required**
- `title`: string (maxLength 255) **required**
- `type`: string (enum page) **required**
- `status`: string (enum current, default "current")
- `space`: object
  - `key`: string (int32) **required**
- `ancestors`: array<object> (nullable)
  - `id`: string **required**

## ContentBody

object

- `value`: string **required**
- `representation`: string (enum view|export_view|styled_view|storage|editor|editor2|anonymous_export_view|wiki|atlas_doc_format|raw) **required**
- `embeddedContent`: array<`EmbeddedContent`>
- `webresource`: `WebResourceDependencies`
- `mediaToken`: object
  - `collectionIds`: array<string>
  - `contentId`: string
  - `expiryDateTime`: string
  - `fileIds`: array<string>
  - `token`: string
- `_expandable`: object
  - `content`: string
  - `embeddedContent`: string
  - `webresource`: string
  - `mediaToken`: string
- `_links`: `GenericLinks`

## ContentBodyConversionInput

object

- `to`: string **required**
- `allowCache`: boolean (default false)
- `spaceKeyContext`: string
- `contentIdContext`: string
- `embeddedContentRender`: string (enum current|version-at-save, default "current")
- `expand`: array<string>
- `body`: `ContentBodyCreate` **required**

## ContentBodyCreate

object

- `value`: string **required**
- `representation`: string (enum view|export_view|styled_view|storage|editor|editor2|anonymous_export_view|wiki|atlas_doc_format|plain|raw) **required**

## ContentBodyCreateStorage

object

- `value`: string **required**
- `representation`: string (enum storage|view|export_view|styled_view|editor|editor2|anonymous_export_view|wiki|atlas_doc_format) **required**

## ContentChildType

object

- `attachment`: object
  - `value`: boolean **required**
  - `_links`: `GenericLinks` **required**
- `comment`: object
  - `value`: boolean **required**
  - `_links`: `GenericLinks` **required**
- `page`: object
  - `value`: boolean **required**
  - `_links`: `GenericLinks` **required**
- `_expandable`: object
  - `all`: string
  - `attachment`: string
  - `comment`: string
  - `page`: string
  - `whiteboard`: string
  - `database`: string
  - `embed`: string
  - `folder`: string

## ContentChildren

object

- `attachment`: `ContentArray`
- `comment`: `ContentArray`
- `page`: `ContentArray`
- `whiteboard`: `ContentArray`
- `database`: `ContentArray`
- `embed`: `ContentArray`
- `folder`: `ContentArray`
- `_expandable`: object
  - `attachment`: string
  - `comment`: string
  - `page`: string
  - `whiteboard`: string
  - `database`: string
  - `embed`: string
  - `folder`: string
- `_links`: `GenericLinks`

## ContentCreate

object

- `id`: string (nullable)
- `title`: string (nullable, maxLength 255)
- `type`: string **required**
- `space`: object (nullable)
  - `id`: integer (int64, nullable)
  - `name`: string (nullable)
  - `icon`: `Icon`
  - `identifiers`: `GlobalSpaceIdentifier`
  - `description`: object (nullable)
    - `plain`: `SpaceDescription`
    - `view`: `SpaceDescription`
    - `_expandable`: object
      - `view`: string
      - `plain`: string
  - `homepage`: `Content`
  - `type`: string (nullable)
  - `metadata`: object (nullable)
    - `labels`: `LabelArray`
    - `_expandable`: object
  - `operations`: array<`OperationCheckResult`> (nullable)
  - `permissions`: array<`SpacePermission`> (nullable)
  - `status`: string (nullable)
  - `settings`: `SpaceSettings`
  - `theme`: `Theme`
  - `lookAndFeel`: `LookAndFeel`
  - `history`: object
    - `createdDate`: string (date-time) **required**
    - `createdBy`: `User`
  - `key`: string **required**
  - `links`: object (nullable)
- `status`: string (enum current|deleted|historical|draft, default "current")
- `container`: object (nullable)
  - `id`: integer | string **required**
  - `type`: string **required**
- `ancestors`: array<object> (nullable)
  - `id`: string **required**
- `body`: object
  - `view`: `ContentBodyCreate`
  - `export_view`: `ContentBodyCreate`
  - `styled_view`: `ContentBodyCreate`
  - `storage`: `ContentBodyCreate`
  - `editor`: `ContentBodyCreate`
  - `editor2`: `ContentBodyCreate`
  - `wiki`: `ContentBodyCreate`
  - `anonymous_export_view`: `ContentBodyCreate`
  - `plain`: `ContentBodyCreate`
  - `atlas_doc_format`: `ContentBodyCreate`
  - `raw`: `ContentBodyCreate`

## ContentHistory

object

- `latest`: boolean **required**
- `createdBy`: `User`
- `ownedBy`: `User`
- `lastOwnedBy`: `User`
- `createdDate`: string (date-time)
- `lastUpdated`: `Version`
- `previousVersion`: `Version`
- `contributors`: object
  - `publishers`: `UsersUserKeys`
- `nextVersion`: `Version`
- `_expandable`: object
  - `lastUpdated`: string
  - `previousVersion`: string
  - `contributors`: string
  - `nextVersion`: string
  - `ownedBy`: string
  - `lastOwnedBy`: string
- `_links`: `GenericLinks`

## ContentId

string


## ContentLookAndFeel

object

- `screen`: `ScreenLookAndFeel`
- `container`: `ContainerLookAndFeel`
- `header`: `ContainerLookAndFeel`
- `body`: `ContainerLookAndFeel`

## ContentMetadata

object

- `currentuser`: object
  - `favourited`: object
    - `isFavourite`: boolean
    - `favouritedDate`: string (date-time)
  - `lastmodified`: object
    - `version`: `Version`
    - `friendlyLastModified`: string
  - `lastcontributed`: object
    - `status`: string
    - `when`: string (date-time)
  - `viewed`: object
    - `lastSeen`: string (date-time)
    - `friendlyLastSeen`: string
  - `scheduled`: object
  - `_expandable`: object
    - `favourited`: string
    - `lastmodified`: string
    - `lastcontributed`: string
    - `viewed`: string
    - `scheduled`: string
- `properties`: `GenericLinks`
- `frontend`: object
- `labels`: `LabelArray` | array<`Label`>

## ContentPageResponse

object

- `results`: array<`Content`> **required**
- `start`: integer (int32) **required**
- `limit`: integer (int32) **required**
- `size`: integer (int32) **required**

## ContentPermissionRequest

object

- `subject`: `PermissionSubjectWithGroupId` **required**
- `operation`: string (enum read|update|delete) **required**

## ContentProperty

object

- `id`: string **required**
- `key`: string **required**
- `value`: array<string> | boolean | object | string **required**
- `version`: object
  - `when`: string (date-time) **required**
  - `message`: string **required**
  - `number`: integer (int32) **required**
  - `minorEdit`: boolean **required**
  - `contentTypeModified`: boolean
- `_links`: `GenericLinks` **required**
- `_expandable`: object
  - `content`: string
  - `additionalProperties`: string

## ContentPropertyArray

object

- `results`: array<`ContentProperty`> **required**
- `start`: integer (int32) **required**
- `limit`: integer (int32) **required**
- `size`: integer (int32) **required**
- `_links`: `GenericLinks` **required**

## ContentPropertyCreate

object

- `key`: string (maxLength 255) **required**
- `value`: array<string> | boolean | object | string **required**

## ContentPropertyCreateNoKey

object

- `value`: array<string> | boolean | object | string **required**

## ContentPropertyUpdate

object

- `value`: array<string> | boolean | object | string **required**
- `version`: object (nullable) **required**
  - `number`: integer | string **required**
  - `minorEdit`: boolean (default true)

## ContentRestriction

object

- `operation`: string (enum administer|copy|create|delete|export|move|purge|purge_version|read|restore|update|use) **required**
- `restrictions`: object
  - `user`: `UserArray`
  - `group`: `GroupArray`
  - `_expandable`: object
    - `user`: string
    - `group`: string
- `content`: `Content`
- `_expandable`: object **required**
  - `restrictions`: string
  - `content`: string
- `_links`: `GenericLinks` **required**

## ContentRestrictionAddOrUpdateArray

object | array<`ContentRestrictionUpdate`>


## ContentRestrictionArray

object

- `results`: array<`ContentRestriction`> **required**
- `start`: integer (int32) **required**
- `limit`: integer (int32) **required**
- `size`: integer (int32) **required**
- `restrictionsHash`: string **required**
- `_links`: `GenericLinks` **required**

## ContentRestrictionGroupInput

object

- `id`: string **required**
- `type`: string (enum group)

## ContentRestrictionUpdate

object

- `operation`: string (enum read|update) **required**
- `restrictions`: any | any **required**
  - `user`: object | array<`ContentRestrictionUserInput`>
  - `group`: object | array<`ContentRestrictionGroupInput`>
- `content`: object
  - `id`: string **required**

## ContentRestrictionUserInput

object

- `accountId`: string **required**
- `type`: string (enum known)

## ContentState

object

- `id`: integer (int64) **required**
- `name`: string **required**
- `color`: string **required**

## ContentStateResponse

object

- `contentState`: `ContentState`
- `lastUpdated`: string

## ContentStateRestInput

object

- `name`: string
- `color`: string
- `id`: integer (int64)

## ContentStateSettings

object

- `contentStatesAllowed`: boolean **required**
- `customContentStatesAllowed`: boolean **required**
- `spaceContentStatesAllowed`: boolean **required**
- `spaceContentStates`: array<`ContentState`>

## ContentTemplate

object

- `templateId`: string **required**
- `originalTemplate`: object
  - `pluginKey`: string
  - `moduleKey`: string
- `referencingBlueprint`: string
- `name`: string **required**
- `description`: string **required**
- `space`: object
- `labels`: array<`Label`> **required**
- `templateType`: string **required**
- `editorVersion`: string
- `body`: `ContentTemplateBody`
- `_expandable`: object
  - `body`: string
- `_links`: `GenericLinks` **required**

## ContentTemplateArray

object

- `results`: array<`ContentTemplate`> **required**
- `start`: integer (int32) **required**
- `limit`: integer (int32) **required**
- `size`: integer (int32) **required**
- `_links`: `GenericLinks` **required**

## ContentTemplateBody

object

- `view`: `ContentBody`
- `export_view`: `ContentBody`
- `styled_view`: `ContentBody`
- `storage`: `ContentBody`
- `editor`: `ContentBody`
- `editor2`: `ContentBody`
- `wiki`: `ContentBody`
- `atlas_doc_format`: `ContentBody`
- `anonymous_export_view`: `ContentBody`

## ContentTemplateBodyCreate

object

- `view`: `ContentBodyCreate`
- `export_view`: `ContentBodyCreate`
- `styled_view`: `ContentBodyCreate`
- `storage`: `ContentBodyCreate`
- `editor`: `ContentBodyCreate`
- `editor2`: `ContentBodyCreate`
- `wiki`: `ContentBodyCreate`
- `atlas_doc_format`: `ContentBodyCreate`
- `anonymous_export_view`: `ContentBodyCreate`

## ContentTemplateCreate

object

- `name`: string **required**
- `templateType`: string **required**
- `body`: `ContentTemplateBodyCreate` **required**
- `description`: string (maxLength 255)
- `labels`: array<`Label`>
- `space`: object (nullable)
  - `key`: string **required**

## ContentTemplateUpdate

object

- `templateId`: string **required**
- `name`: string **required**
- `templateType`: string (enum page) **required**
- `body`: `ContentTemplateBodyCreate` **required**
- `description`: string (maxLength 100)
- `labels`: array<`Label`>
- `space`: object (nullable)
  - `key`: string **required**

## ContentUpdate

object

- `version`: object (nullable) **required**
  - `number`: integer (int32) **required**
  - `message`: string (nullable)
- `title`: string (nullable, maxLength 255)
- `type`: string (nullable) **required**
- `status`: string (enum current|trashed|deleted|historical|draft)
- `ancestors`: array<object> (nullable)
  - `id`: integer | string **required**
- `body`: object
  - `view`: `ContentBodyCreate`
  - `export_view`: `ContentBodyCreate`
  - `styled_view`: `ContentBodyCreate`
  - `storage`: `ContentBodyCreateStorage`
  - `editor`: `ContentBodyCreate`
  - `editor2`: `ContentBodyCreate`
  - `wiki`: `ContentBodyCreate`
  - `atlas_doc_format`: `ContentBodyCreate`
  - `anonymous_export_view`: `ContentBodyCreate`

## CopyPageHierarchyRequest

object

- `copyAttachments`: boolean (default false)
- `copyPermissions`: boolean (default false)
- `copyProperties`: boolean (default false)
- `copyLabels`: boolean (default false)
- `copyCustomContents`: boolean (default false)
- `copyDescendants`: boolean (default true)
- `destinationPageId`: `ContentId` **required**
- `titleOptions`: `CopyPageHierarchyTitleOptions`

## CopyPageHierarchyTitleOptions

object

- `prefix`: string
- `replace`: string
- `search`: string

## CopyPageRequest

object

- `copyAttachments`: boolean (default false)
- `copyPermissions`: boolean (default false)
- `copyProperties`: boolean (default false)
- `copyLabels`: boolean (default false)
- `copyCustomContents`: boolean (default false)
- `destination`: `CopyPageRequestDestination` **required**
- `pageTitle`: string
- `body`: object
  - `storage`: `ContentBodyCreate`
  - `editor2`: `ContentBodyCreate`

## CopyPageRequestDestination

object

- `type`: string (enum space|existing_page|parent_page|parent_content) **required**
- `value`: string **required**

## DynamicModulesErrorMessage

object

- `message`: string **required**

## Embeddable

object


## EmbeddedContent

object

- `entityId`: integer (int64)
- `entityType`: string
- `entity`: `Embeddable`

## GenericAccountId

string (nullable)


## GenericLinks

map<string,object | string>


## GenericUserKey

string (nullable)


## GenericUserName

string (nullable)


## GlobalSpaceIdentifier

object

- `spaceIdentifier`: string **required**

## Group

object

- `type`: string (enum group, default "group") **required**
- `name`: string **required**
- `id`: string **required**
- `usageType`: string (enum USERBASE_GROUP|TEAM_COLLABORATION, default "USERBASE_GROUP")
- `managedBy`: string (enum ADMINS|EXTERNAL|TEAM_MEMBERS|OPEN, default "ADMINS")
- `_links`: `GenericLinks`

## GroupArray

object

- `results`: array<`Group`> **required**
- `start`: integer (int32) **required**
- `limit`: integer (int32) **required**
- `size`: integer (int32) **required**

## GroupArrayWithLinks

object

- `results`: array<`Group`> **required**
- `start`: integer (int32) **required**
- `limit`: integer (int32) **required**
- `size`: integer (int32) **required**
- `totalSize`: integer (int64, default 0)
- `_links`: `GenericLinks` **required**

## GroupCreate

object

- `type`: string (enum group, default "group") **required**
- `id`: string

## GroupName

object

- `name`: string **required**

## HeaderLookAndFeel

object

- `backgroundColor`: string **required**
- `button`: `ButtonLookAndFeel` **required**
- `primaryNavigation`: `NavigationLookAndFeel` **required**
- `secondaryNavigation`: `NavigationLookAndFeel` **required**
- `search`: `SearchFieldLookAndFeel` **required**

## HorizontalHeaderLookAndFeel

object

- `backgroundColor`: string **required**
- `button`: `ButtonLookAndFeel`
- `primaryNavigation`: `TopNavigationLookAndFeel` **required**
- `secondaryNavigation`: `NavigationLookAndFeel`
- `search`: `SearchFieldLookAndFeel`

## Icon

object

- `path`: string **required**
- `width`: integer (int32) **required**
- `height`: integer (int32) **required**
- `isDefault`: boolean **required**

## Label

object

- `prefix`: string **required**
- `name`: string **required**
- `id`: string **required**
- `label`: string **required**

## LabelArray

object

- `results`: array<`Label`> **required**
- `start`: integer (int32)
- `limit`: integer (int32)
- `size`: integer (int32) **required**
- `_links`: `GenericLinks`

## LabelCreate

object

- `prefix`: string **required**
- `name`: string **required**

## LabelCreateArray

array<`LabelCreate`>


## LabelDetails

object

- `label`: `Label` **required**
- `associatedContents`: `LabeledContentPageResponse`

## LabeledContent

object

- `contentType`: `LabeledContentType` **required**
- `contentId`: integer (int64) **required**
- `title`: string **required**

## LabeledContentPageResponse

object

- `results`: array<`LabeledContent`> **required**
- `start`: integer (int32)
- `limit`: integer (int32)
- `size`: integer (int32) **required**

## LabeledContentType

string (enum page|blogpost|attachment|page_template)


## LongTask

object

- `ari`: string
- `id`: string **required**
- `links`: object **required**
  - `status`: string

## LongTaskStatus

object

- `ari`: string
- `id`: string **required**
- `name`: object **required**
  - `key`: string **required**
  - `args`: array<object> **required**
- `elapsedTime`: integer (int64) **required**
- `percentageComplete`: integer (int32) **required**
- `successful`: boolean **required**
- `finished`: boolean **required**
- `messages`: array<`Message`> **required**
- `status`: string
- `errors`: array<`Message`>
- `additionalDetails`: object
  - `destinationId`: string
  - `destinationUrl`: string
  - `totalPageNeedToCopy`: integer
  - `additionalProperties`: string

## LongTaskStatusArray

object

- `results`: array<`LongTaskStatus`> **required**
- `start`: integer (int32) **required**
- `limit`: integer (int32) **required**
- `size`: integer (int32) **required**
- `_links`: `GenericLinks` **required**

## LongTaskStatusWithLinks

object

- `ari`: string
- `id`: string **required**
- `name`: object **required**
  - `key`: string **required**
  - `args`: array<object> **required**
- `elapsedTime`: integer (int64) **required**
- `percentageComplete`: integer (int32) **required**
- `successful`: boolean **required**
- `finished`: boolean **required**
- `messages`: array<`Message`> **required**
- `_links`: `GenericLinks` **required**
- `status`: string
- `errors`: array<`Message`>
- `additionalDetails`: object
  - `destinationId`: string (nullable)
  - `destinationUrl`: string
  - `totalPageNeedToCopy`: integer
  - `additionalProperties`: string

## LookAndFeel

object

- `headings`: object **required**
  - `color`: string **required**
- `links`: object **required**
  - `color`: string **required**
- `menus`: `MenusLookAndFeel` **required**
- `header`: `HeaderLookAndFeel` **required**
- `horizontalHeader`: `HorizontalHeaderLookAndFeel`
- `content`: `ContentLookAndFeel` **required**
- `bordersAndDividers`: object **required**
  - `color`: string **required**
- `spaceReference`: object (nullable)

## LookAndFeelSelection

object

- `spaceKey`: string **required**
- `lookAndFeelType`: string (enum global|custom|theme) **required**

## LookAndFeelSettings

object

- `selected`: string (enum global|custom) **required**
- `global`: `LookAndFeel` **required**
- `theme`: `LookAndFeel`
- `custom`: `LookAndFeel` **required**

## LookAndFeelWithLinks

`LookAndFeel` & object

- `_links`: `GenericLinks`

## MacroInstance

object

- `name`: string
- `body`: string
- `parameters`: object
- `_links`: `GenericLinks`

## MenusLookAndFeel

object

- `hoverOrFocus`: object **required**
  - `backgroundColor`: string **required**
- `color`: string **required**

## Message

object

- `translation`: string
- `args`: array<string | object> **required**

## NavigationLookAndFeel

object

- `color`: string **required**
- `highlightColor`: string (nullable)
- `hoverOrFocus`: object **required**
  - `backgroundColor`: string **required**
  - `color`: string **required**

## OperationCheckResult

object

- `operation`: string (enum administer|archive|clear_permissions|copy|create|create_space|delete|export|move|purge|purge_version|read|restore|restrict_content|update|use) **required**
- `targetType`: string **required**

## PermissionCheckResponse

object

- `hasPermission`: boolean **required**
- `errors`: array<`Message`>
- `_links`: `GenericLinks`

## PermissionSubject

object

- `type`: string (enum user|group) **required**
- `identifier`: string **required**

## PermissionSubjectWithGroupId

object

- `type`: string (enum user|group) **required**
- `identifier`: string **required**

## PropertyValue

array<string> | boolean | object | string


## Relation

object

- `name`: string **required**
- `relationData`: `RelationData`
- `source`: `Content` | `User` | `Space`
- `target`: `Content` | `User` | `Space`
- `_expandable`: object
  - `relationData`: string
  - `source`: string
  - `target`: string
- `_links`: `GenericLinks` **required**

## RelationArray

object

- `results`: array<`Relation`> **required**
- `start`: integer (int32) **required**
- `limit`: integer (int32) **required**
- `size`: integer (int32) **required**
- `_links`: `GenericLinks` **required**

## RelationData

object

- `createdBy`: `User`
- `createdDate`: string (date-time)
- `friendlyCreatedDate`: string

## RetentionPeriod

object

- `number`: integer (int32) **required**
- `units`: string (enum NANOS|MICROS|MILLIS|SECONDS|MINUTES|HOURS|HALF_DAYS|DAYS|WEEKS|MONTHS|YEARS|DECADES|CENTURIES|MILLENNIA|ERAS|FOREVER) **required**

## ScreenLookAndFeel

object

- `background`: string **required**
- `backgroundAttachment`: string (nullable)
- `backgroundBlendMode`: string (nullable)
- `backgroundClip`: string (nullable)
- `backgroundColor`: string (nullable)
- `backgroundImage`: string (nullable)
- `backgroundOrigin`: string (nullable)
- `backgroundPosition`: string (nullable)
- `backgroundRepeat`: string (nullable)
- `backgroundSize`: string (nullable)
- `layer`: object (nullable)
  - `width`: string
  - `height`: string
- `gutterTop`: string (nullable)
- `gutterRight`: string (nullable)
- `gutterBottom`: string (nullable)
- `gutterLeft`: string (nullable)

## SearchFieldLookAndFeel

object

- `backgroundColor`: string **required**
- `color`: string **required**

## SearchPageResponseSearchResult

object

- `results`: array<`SearchResult`> **required**
- `start`: integer (int32) **required**
- `limit`: integer (int32) **required**
- `size`: integer (int32) **required**
- `totalSize`: integer (int32) **required**
- `cqlQuery`: string **required**
- `searchDuration`: integer (int32) **required**
- `archivedResultCount`: integer (int32)
- `_links`: `GenericLinks` **required**

## SearchResult

object

- `content`: `Content`
- `user`: `User`
- `space`: `Space`
- `title`: string **required**
- `excerpt`: string **required**
- `url`: string **required**
- `resultParentContainer`: `ContainerSummary`
- `resultGlobalContainer`: `ContainerSummary`
- `breadcrumbs`: array<`Breadcrumb`> **required**
- `entityType`: string **required**
- `iconCssClass`: string **required**
- `lastModified`: string (date-time) **required**
- `friendlyLastModified`: string
- `score`: number

## Space

object

- `id`: integer (int64)
- `key`: string **required**
- `alias`: string
- `name`: string **required**
- `icon`: `Icon`
- `description`: object
  - `plain`: `SpaceDescription`
  - `view`: `SpaceDescription`
  - `_expandable`: object
    - `view`: string
    - `plain`: string
- `homepage`: `Content`
- `type`: string **required**
- `metadata`: object
  - `labels`: `LabelArray`
  - `_expandable`: object
- `operations`: array<`OperationCheckResult`>
- `permissions`: array<`SpacePermission`>
- `status`: string **required**
- `settings`: `SpaceSettings`
- `theme`: `Theme`
- `lookAndFeel`: `LookAndFeel`
- `history`: object
  - `createdDate`: string (date-time) **required**
  - `createdBy`: `User`
- `_expandable`: object **required**
  - `settings`: string
  - `metadata`: string
  - `operations`: string
  - `lookAndFeel`: string
  - `permissions`: string
  - `icon`: string
  - `description`: string
  - `theme`: string
  - `history`: string
  - `homepage`: string
  - `identifiers`: string
- `_links`: `GenericLinks` **required**

## SpaceArray

object

- `results`: array<`Space`> **required**
- `start`: integer (int32) **required**
- `limit`: integer (int32) **required**
- `size`: integer (int32) **required**
- `_links`: `GenericLinks` **required**

## SpaceCreate

object

- `name`: string (maxLength 200) **required**
- `key`: string
- `alias`: string
- `description`: `SpaceDescriptionCreate`
- `permissions`: array<`SpacePermissionCreate`> (nullable)

## SpaceDescription

object

- `value`: string **required**
- `representation`: string (enum plain|view) **required**
- `embeddedContent`: array<object> **required**

## SpaceDescriptionCreate

object

- `plain`: object **required**
  - `value`: string
  - `representation`: string

## SpacePermission

object

- `id`: integer (int64)
- `subjects`: object
  - `user`: object
    - `results`: array<`User`> **required**
    - `size`: integer (int32) **required**
    - `start`: integer (int32)
    - `limit`: integer (int32)
  - `group`: object
    - `results`: array<`Group`> **required**
    - `size`: integer (int32) **required**
    - `start`: integer (int32)
    - `limit`: integer (int32)
  - `_expandable`: object
    - `user`: string
    - `group`: string
- `operation`: `OperationCheckResult` **required**
- `anonymousAccess`: boolean (default false) **required**
- `unlicensedAccess`: boolean (default false) **required**

## SpacePermissionCreate

object

- `subjects`: object
  - `user`: object
    - `results`: array<`User`> **required**
    - `size`: integer (int32) **required**
  - `group`: object
    - `results`: array<`GroupCreate`> **required**
    - `size`: integer (int32) **required**
- `operation`: `OperationCheckResult` **required**
- `anonymousAccess`: boolean (default false) **required**
- `unlicensedAccess`: boolean (default false) **required**

## SpacePermissionCustomContent

object

- `subject`: `PermissionSubject` **required**
- `operations`: array<object> **required**
  - `key`: string (enum read|create|delete) **required**
  - `target`: string **required**
  - `access`: boolean **required**

## SpacePermissionRequest

object

- `subject`: `PermissionSubject` **required**
- `operation`: object **required**
  - `key`: string (enum administer|archive|copy|create|delete|export|move|purge|purge_version|read|restore|restrict_content|update|use) **required**
  - `target`: string (enum page|blogpost|comment|attachment|space) **required**
- `_links`: `GenericLinks`

## SpacePermissionV2

object

- `id`: integer (int64) **required**
- `subject`: `PermissionSubject` **required**
- `operation`: object **required**
  - `key`: string (enum administer|archive|copy|create|delete|export|move|purge|purge_version|read|restore|restrict_content|update|use) **required**
  - `target`: string (enum page|blogpost|comment|attachment|space) **required**
- `_links`: `GenericLinks`

## SpaceProperty

object

- `id`: string **required**
- `key`: string **required**
- `value`: array<string> | boolean | object | string **required**
- `version`: `Version`
- `space`: `Space`
- `_links`: `GenericLinks`
- `_expandable`: object **required**
  - `version`: string
  - `space`: string

## SpacePropertyArray

object

- `results`: array<`SpaceProperty`> **required**
- `start`: integer (int32) **required**
- `limit`: integer (int32) **required**
- `size`: integer (int32) **required**
- `_links`: `GenericLinks` **required**

## SpacePropertyCreate

object

- `key`: string **required**
- `value`: `PropertyValue` **required**
- `space`: object
  - `key`: string

## SpacePropertyCreateNoKey

object

- `value`: `PropertyValue` **required**

## SpacePropertyUpdate

object

- `key`: string
- `value`: array<string> | boolean | object | string **required**
- `version`: `Version` **required**
- `space`: object
  - `key`: string

## SpaceSettings

object

- `routeOverrideEnabled`: boolean **required**
- `editor`: object
  - `page`: string **required**
  - `blogpost`: string **required**
  - `default`: string **required**
- `contentMode`: string (enum standard|compact, nullable)
- `spaceKey`: string
- `_links`: `GenericLinks` **required**

## SpaceSettingsUpdate

object

- `routeOverrideEnabled`: boolean
- `contentMode`: string (enum standard|compact, nullable)

## SpaceUpdate

object

- `name`: string (nullable, maxLength 200)
- `description`: `SpaceDescriptionCreate`
- `homepage`: object (nullable)
- `type`: string
- `status`: string (nullable)

## SpaceWatch

object

- `type`: string **required**
- `watcher`: `WatchUser` **required**
- `spaceKey`: string
- `labelName`: string
- `prefix`: string

## SpaceWatchArray

object

- `results`: array<`SpaceWatch`> **required**
- `start`: integer (int32) **required**
- `limit`: integer (int32) **required**
- `size`: integer (int32) **required**
- `_links`: `GenericLinks`

## SuperBatchWebResources

object

- `uris`: object
  - `all`: array<string> | string
  - `css`: array<string> | string
  - `js`: array<string> | string
- `tags`: object
  - `all`: string
  - `css`: string
  - `data`: string
  - `js`: string
- `metatags`: string
- `_expandable`: object

## SystemInfoEntity

object

- `cloudId`: string **required**
- `commitHash`: string **required**
- `baseUrl`: string
- `fallbackBaseUrl`: string
- `edition`: string
- `siteTitle`: string
- `defaultLocale`: string
- `defaultTimeZone`: string
- `microsPerimeter`: string

## Task

object

- `globalId`: integer (int64) **required**
- `id`: integer (int64) **required**
- `contentId`: integer (int64) **required**
- `status`: string **required**
- `title`: string
- `description`: string
- `body`: string
- `creator`: string **required**
- `assignee`: string
- `completeUser`: string
- `createDate`: integer (int64) **required**
- `dueDate`: integer (int64)
- `updateDate`: integer (int64)
- `completeDate`: integer (int64)
- `_links`: `GenericLinks`

## TaskPageResponse

object

- `results`: array<`Task`> **required**
- `start`: integer (int32) **required**
- `limit`: integer (int32) **required**
- `size`: integer (int32) **required**

## TaskStatusUpdate

object

- `status`: string (enum complete|incomplete) **required**

## Theme

object

- `themeKey`: string **required**
- `name`: string
- `description`: string
- `icon`: `Icon`
- `_links`: `GenericLinks`

## ThemeArray

object

- `results`: array<`ThemeNoLinks`> **required**
- `start`: integer (int32) **required**
- `limit`: integer (int32) **required**
- `size`: integer (int32) **required**
- `_links`: `GenericLinks` **required**

## ThemeNoLinks

object

- `themeKey`: string **required**
- `name`: string
- `description`: string
- `icon`: `Icon`

## ThemeUpdate

object

- `themeKey`: string **required**

## TopNavigationLookAndFeel

object

- `color`: string (nullable)
- `highlightColor`: string **required**
- `hoverOrFocus`: object
  - `backgroundColor`: string
  - `color`: string

## User

object

- `type`: string (enum known|unknown|anonymous|user) **required**
- `username`: `GenericUserName`
- `userKey`: `GenericUserKey`
- `accountId`: `GenericAccountId`
- `accountType`: string (enum atlassian|app|)
- `email`: string (nullable)
- `publicName`: string
- `profilePicture`: `Icon`
- `displayName`: string (nullable)
- `timeZone`: string (nullable)
- `externalCollaborator`: boolean
- `isExternalCollaborator`: boolean
- `isGuest`: boolean
- `operations`: array<`OperationCheckResult`> (nullable)
- `details`: `UserDetails`
- `personalSpace`: `Space`
- `_expandable`: object
  - `operations`: string
  - `details`: string
  - `personalSpace`: string
- `_links`: `GenericLinks`

## UserAnonymous

object

- `type`: string **required**
- `profilePicture`: `Icon` **required**
- `displayName`: string **required**
- `operations`: array<`OperationCheckResult`>
- `_expandable`: object
  - `operations`: string
- `_links`: `GenericLinks` **required**

## UserArray

object

- `results`: array<`User`> **required**
- `start`: integer (int32)
- `limit`: integer (int32)
- `size`: integer (int32)
- `totalSize`: integer (int64, default 0)
- `_links`: `GenericLinks`

## UserDetails

object

- `business`: object
  - `position`: string
  - `department`: string
  - `location`: string
- `personal`: object
  - `phone`: string
  - `im`: string
  - `website`: string
  - `email`: string

## UserProperty

object

- `key`: string **required**
- `value`: object **required**
- `id`: string **required**
- `lastModifiedDate`: string (date-time) **required**
- `createdDate`: string (date-time) **required**
- `_links`: `GenericLinks`

## UserPropertyCreate

object

- `value`: object **required**

## UserPropertyKeyArray

object

- `results`: array<object> **required**
  - `key`: string
- `start`: integer (int32)
- `limit`: integer (int32)
- `size`: integer (int32)
- `_links`: `GenericLinks`

## UserPropertyUpdate

object

- `value`: object **required**

## UserWatch

object

- `watching`: boolean **required**

## UsersUserKeys

object

- `users`: array<`User`>
- `userKeys`: array<string>
- `_links`: `GenericLinks`

## Version

object

- `by`: `User`
- `when`: string (date-time, nullable) **required**
- `friendlyWhen`: string (nullable)
- `message`: string (nullable)
- `number`: integer (int32) **required**
- `minorEdit`: boolean **required**
- `content`: `Content`
- `collaborators`: `UsersUserKeys`
- `_expandable`: object
  - `content`: string
  - `collaborators`: string
- `_links`: `GenericLinks`
- `contentTypeModified`: boolean
- `confRev`: string (nullable)
- `syncRev`: string (nullable)
- `syncRevSource`: string (nullable)

## VersionArray

object

- `results`: array<`Version`> **required**
- `start`: integer (int32) **required**
- `limit`: integer (int32) **required**
- `size`: integer (int32) **required**
- `_links`: `GenericLinks` **required**

## VersionRestore

object

- `operationKey`: string (enum restore) **required**
- `params`: object **required**
  - `versionNumber`: integer (int32) **required**
  - `message`: string **required**
  - `restoreTitle`: boolean (default false)

## Watch

object

- `type`: string **required**
- `watcher`: `WatchUser` **required**
- `contentId`: integer (int64) **required**

## WatchArray

object

- `results`: array<`Watch`> **required**
- `start`: integer (int32) **required**
- `limit`: integer (int32) **required**
- `size`: integer (int32) **required**
- `_links`: `GenericLinks` **required**

## WatchUser

object

- `type`: string **required**
- `username`: `GenericUserName`
- `userKey`: `GenericUserKey`
- `accountId`: `GenericAccountId` **required**
- `profilePicture`: `Icon` **required**
- `displayName`: string **required**
- `timeZone`: string (nullable)
- `operations`: array<`OperationCheckResult`> (nullable) **required**
- `externalCollaborator`: boolean **required**
- `isGuest`: boolean (nullable) **required**
- `isExternalCollaborator`: boolean **required**
- `details`: `UserDetails`
- `accountType`: string **required**
- `email`: string **required**
- `publicName`: string **required**
- `personalSpace`: object (nullable) **required**

## WebResourceDependencies

object

- `_expandable`: object
  - `uris`: string | object
- `keys`: array<string>
- `contexts`: array<string>
- `uris`: object
  - `all`: array<string> | string
  - `css`: array<string> | string
  - `js`: array<string> | string
  - `_expandable`: object
    - `css`: array<string> | string
    - `js`: array<string> | string
- `tags`: object
  - `all`: string
  - `css`: string
  - `data`: string
  - `js`: string
  - `_expandable`: object
- `superbatch`: `SuperBatchWebResources`

## Security schemes

```json
{
 "basicAuth": {
  "type": "http",
  "description": "You can access this resource via basic auth.",
  "scheme": "basic"
 },
 "oAuthDefinitions": {
  "type": "oauth2",
  "description": "This API uses OAuth 2 with the authorizationCode grant flow.",
  "flows": {
   "authorizationCode": {
    "authorizationUrl": "https://auth.atlassian.com/authorize",
    "tokenUrl": "https://auth.atlassian.com/oauth/token",
    "scopes": {
     "read:confluence-content.all": "Read all content, including content body (expansions permitted). Note, APIs using this scope may also return data allowed by read:confluence-space.summary. However, this scope is not a substitute for read:confluence-space.summary.",
     "read:confluence-content.permission": "Read content permissions.",
     "read:confluence-content.summary": "Read a summary of the content, which is the content without expansions. Note, APIs using this scope may also return data allowed by read:confluence-space.summary. However, this scope is not a substitute for read:confluence-space.summary.",
     "write:confluence-content": "Permits the creation of pages, blogs, comments and questions.",
     "read:confluence-space.summary": "Read a summary of space information without expansions.",
     "write:confluence-space": "Create, update and delete space information.",
     "write:confluence-file": "Upload attachments.",
     "read:confluence-props": "Read content properties.",
     "write:confluence-props": "Write content properties.",
     "search:confluence": "Search Confluence. Note, APIs using this scope may also return data allowed by read:confluence-space.summary and read:confluence-content.summary. However, this scope is not a substitute for read:confluence-space.summary or read:confluence-content.summary.",
     "manage:confluence-configuration": "Manage global settings.",
     "read:confluence-groups": "Read user groups.",
     "write:confluence-groups": "Create, remove and update user groups.",
     "read:confluence-user": "Read users.",
     "readonly:content.attachment:confluence": "Download attachments of a Confluence page or blogpost that you have access to.",
     "read:content:confluence": "View content.",
     "read:content-details:confluence": "View content details.",
     "write:content:confluence": "Create and update content.",
     "delete:content:confluence": "Delete content.",
     "read:space-details:confluence": "View space details.",
     "read:analytics.content:confluence": "View analytics for content.",
     "read:audit-log:confluence": "View audit records.",
     "write:audit-log:confluence": "Create audit records.",
     "read:configuration:confluence": "View Confluence settings.",
     "write:configuration:confluence": "Update Confluence settings.",
     "read:page:confluence": "View pages.",
     "write:page:confluence": "Create and update pages.",
     "delete:page:confluence": "Delete pages.",
     "read:blogpost:confluence": "View blogposts.",
     "write:blogpost:confluence": "Create and update blogposts.",
     "delete:blogpost:confluence": "Delete blogposts.",
     "read:whiteboard:confluence": "View whiteboards.",
     "write:whiteboard:confluence": "Create and update whiteboards.",
     "delete:whiteboard:confluence": "Delete whiteboards.",
     "read:custom-content:confluence": "View custom content.",
     "write:custom-content:confluence": "Create and update custom content.",
     "delete:custom-content:confluence": "Delete custom content.",
     "read:attachment:confluence": "View and download content attachments.",
     "write:attachment:confluence": "Create and update attachments.",
     "delete:attachment:confluence": "Delete attachments.",
     "read:comment:confluence": "View comments.",
     "write:comment:confluence": "Create and update comments.",
     "delete:comment:confluence": "Delete comments.",
     "read:template:confluence": "View content templates.",
     "write:template:confluence": "Create, update and delete content templates.",
     "read:label:confluence": "View labels.",
     "write:label:confluence": "Add and remove labels.",
     "read:content.permission:confluence": "Check content permissions.",
     "read:content.property:confluence": "View content properties.",
     "write:content.property:confluence": "Create, update and delete content properties.",
     "read:content.restriction:confluence": "View content restrictions.",
     "write:content.restriction:confluence": "Update content restrictions.",
     "read:content.metadata:confluence": "View content summaries.",
     "read:watcher:confluence": "View content watchers.",
     "write:watcher:confluence": "Add and remove content watchers.",
     "read:group:confluence": "View groups.",
     "write:group:confluence": "Create and delete groups.",
     "read:inlinetask:confluence": "View tasks.",
     "write:inlinetask:confluence": "Update tasks.",
     "read:relation:confluence": "View entity relationships.",
     "write:relation:confluence": "Create and update entity relationships.",
     "read:space:confluence": "View spaces.",
     "write:space:confluence": "Create and update spaces.",
     "delete:space:confluence": "Delete spaces.",
     "read:space.permission:confluence": "View space permissions.",
     "write:space.permission:confluence": "Update space permissions.",
     "read:space.property:confluence": "View space properties.",
     "write:space.property:confluence": "Create, update and delete space properties.",
     "read:user.property:confluence": "View user properties.",
     "write:user.property:confluence": "Create, update and delete user properties.",
     "read:space.setting:confluence": "View space settings.",
     "write:space.setting:confluence": "Update space settings.",
     "read:user:confluence": "View user details.",
     "moderate:core-content:confluence": "Moderate core contents",
     "moderate:comment:confluence": "Moderate comments",
     "read:email-address:confluence": "View email addresses of all users regardless of the user\u2019s profile visibility settings."
    }
   }
  }
 }
}
```
