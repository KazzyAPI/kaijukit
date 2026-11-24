# Confluence REST v2: schemas

Last reviewed: 2026-11-24 · generated · structure only.

## AbstractPageLinks

object

- `webui`: string
- `editui`: string
- `tinyui`: string

## AccountStatus

string (enum active|inactive|closed|unknown)


## AccountType

string (enum atlassian|app|customer|unknown)


## AdminKeyResponse

object

- `accountId`: string
- `expirationTime`: string (date-time)

## Ancestor

object

- `id`: string
- `type`: `AncestorType`

## AncestorType

string (enum page|whiteboard|database|embed|folder)


## AttachmentBulk

object

- `id`: string
- `status`: `ContentStatus`
- `title`: string
- `createdAt`: string (date-time)
- `pageId`: string
- `blogPostId`: string
- `customContentId`: string
- `mediaType`: string
- `mediaTypeDescription`: string
- `comment`: string
- `fileId`: string
- `fileSize`: integer (int64)
- `webuiLink`: string
- `downloadLink`: string
- `version`: `Version`
- `_links`: `AttachmentLinks`

## AttachmentCommentModel

object

- `id`: string
- `status`: `ContentStatus`
- `title`: string
- `attachmentId`: string
- `version`: `Version`
- `body`: `BodySingle`
- `_links`: `CommentLinks`

## AttachmentLinks

object

- `webui`: string
- `download`: string

## AttachmentSingle

object

- `id`: string
- `status`: `ContentStatus`
- `title`: string
- `createdAt`: string (date-time)
- `pageId`: string
- `blogPostId`: string
- `customContentId`: string
- `mediaType`: string
- `mediaTypeDescription`: string
- `comment`: string
- `fileId`: string
- `fileSize`: integer (int64)
- `webuiLink`: string
- `downloadLink`: string
- `version`: `Version`
- `labels`: object
  - `results`: array<`Label`>
  - `meta`: `OptionalFieldMeta`
  - `_links`: `OptionalFieldLinks`
- `properties`: object
  - `results`: array<`ContentProperty`>
  - `meta`: `OptionalFieldMeta`
  - `_links`: `OptionalFieldLinks`
- `operations`: object
  - `results`: array<`Operation`>
  - `meta`: `OptionalFieldMeta`
  - `_links`: `OptionalFieldLinks`
- `versions`: object
  - `results`: array<`Version`>
  - `meta`: `OptionalFieldMeta`
  - `_links`: `OptionalFieldLinks`
- `_links`: `AttachmentLinks`

## AttachmentSortOrder

string (enum created-date|-created-date|modified-date|-modified-date)


## AttachmentVersion

object

- `createdAt`: string (date-time)
- `message`: string
- `number`: integer (int32)
- `minorEdit`: boolean
- `authorId`: string
- `attachment`: `VersionedEntity`

## BlogPostBodyWrite

object

- `representation`: string (enum storage|atlas_doc_format|wiki)
- `value`: string

## BlogPostBulk

object

- `id`: string
- `status`: `BlogPostContentStatus`
- `title`: string
- `spaceId`: string
- `authorId`: string
- `createdAt`: string (date-time)
- `version`: `Version`
- `body`: `BodyBulk`
- `_links`: `AbstractPageLinks`

## BlogPostCommentModel

object

- `id`: string
- `status`: `ContentStatus`
- `title`: string
- `blogPostId`: string
- `version`: `Version`
- `body`: `BodyBulk`
- `_links`: `CommentLinks`

## BlogPostContentStatus

string (enum current|draft|historical|trashed|deleted|any)


## BlogPostInlineCommentModel

object

- `id`: string
- `status`: `ContentStatus`
- `title`: string
- `blogPostId`: string
- `version`: `Version`
- `body`: `BodyBulk`
- `resolutionStatus`: `InlineCommentResolutionStatus`
- `properties`: `InlineCommentProperties`
- `_links`: `CommentLinks`

## BlogPostNestedBodyWrite

object

- `storage`: `BlogPostBodyWrite`
- `atlas_doc_format`: `BlogPostBodyWrite`
- `wiki`: `BlogPostBodyWrite`

## BlogPostSingle

object

- `id`: string
- `status`: `BlogPostContentStatus`
- `title`: string
- `spaceId`: string
- `authorId`: string
- `createdAt`: string (date-time)
- `version`: `Version`
- `body`: `BodySingle`
- `labels`: object
  - `results`: array<`Label`>
  - `meta`: `OptionalFieldMeta`
  - `_links`: `OptionalFieldLinks`
- `properties`: object
  - `results`: array<`ContentProperty`>
  - `meta`: `OptionalFieldMeta`
  - `_links`: `OptionalFieldLinks`
- `operations`: object
  - `results`: array<`Operation`>
  - `meta`: `OptionalFieldMeta`
  - `_links`: `OptionalFieldLinks`
- `likes`: object
  - `results`: array<`Like`>
  - `meta`: `OptionalFieldMeta`
  - `_links`: `OptionalFieldLinks`
- `versions`: object
  - `results`: array<`Version`>
  - `meta`: `OptionalFieldMeta`
  - `_links`: `OptionalFieldLinks`
- `isFavoritedByCurrentUser`: boolean
- `_links`: `AbstractPageLinks`

## BlogPostSortOrder

string (enum id|-id|created-date|-created-date|modified-date|-modified-date)


## BlogPostVersion

object

- `createdAt`: string (date-time)
- `message`: string
- `number`: integer (int32)
- `minorEdit`: boolean
- `authorId`: string
- `blogpost`: `VersionedEntity`

## BodyBulk

object

- `storage`: `BodyType`
- `atlas_doc_format`: `BodyType`

## BodySingle

object

- `storage`: `BodyType`
- `atlas_doc_format`: `BodyType`
- `view`: `BodyType`

## BodyType

object

- `representation`: string
- `value`: string

## BulkAssignRolesRequest

object

- `assignments`: array<`BulkTransitionRoleAssignment`> **required**
- `spaceSelection`: `BulkTransitionSpaceSelection` **required**

## BulkRemoveAccessRequest

object

- `permissionCombinationIds`: array<string> **required**
- `spaceSelection`: `BulkTransitionSpaceSelection` **required**

## BulkTransitionCombinationEntry

object

- `combinationId`: string **required**
- `spaceCount`: integer (int64) **required**
- `principalCount`: integer (int64) **required**
- `permissions`: array<`BulkTransitionDecodedPermission`> **required**
- `principalTypes`: array<string (enum USER|GROUP|GUEST|ANONYMOUS|ALL_LICENSED_USERS_USER_CLASS|ALL_PRODUCT_ADMINS_USER_CLASS|APP|TEAM)> **required**

## BulkTransitionDecodedPermission

object

- `id`: string **required**
- `displayName`: string **required**

## BulkTransitionPrincipalTypeAssignment

object

- `principalType`: string (enum USER|GROUP|GUEST|ANONYMOUS|ALL_LICENSED_USERS_USER_CLASS|ALL_PRODUCT_ADMINS_USER_CLASS|APP) **required**
- `removeAccess`: boolean **required**
- `roleId`: string (nullable)

## BulkTransitionRoleAssignment

object

- `permissionCombinationId`: string **required**
- `principalTypeAssignments`: array<`BulkTransitionPrincipalTypeAssignment`> **required**

## BulkTransitionSpaceSelection

object

- `spaceType`: string (enum ALL|ALL_EXCEPT_PERSONAL|ALL_EXCEPT_SPECIFIC|PERSONAL|SPECIFIC) **required**
- `selectedSpaces`: array<`BulkTransitionSpaceTarget`>

## BulkTransitionSpaceTarget

object

- `id`: string **required**
- `key`: string **required**

## BulkTransitionTaskResponse

object

- `taskId`: string **required**
- `status`: string (enum IN_PROGRESS|COMPLETED|FAILED) **required**
- `statusUrl`: string **required**

## BulkTransitionTaskStatusResponse

object

- `taskId`: string **required**
- `status`: string (enum IN_PROGRESS|COMPLETED|FAILED) **required**
- `errorMessage`: string (nullable)

## ChildCustomContent

object

- `id`: string
- `status`: `OnlyArchivedAndCurrentContentStatus`
- `title`: string
- `type`: string
- `spaceId`: string

## ChildCustomContentSortOrder

string (enum created-date|-created-date|id|-id|modified-date|-modified-date)


## ChildPage

object

- `id`: string
- `status`: `OnlyArchivedAndCurrentContentStatus`
- `title`: string
- `spaceId`: string
- `childPosition`: integer (int32, nullable)

## ChildPageSortOrder

string (enum created-date|-created-date|id|-id|child-position|-child-position|modified-date|-modified-date)


## ChildrenCommentModel

object

- `id`: string
- `status`: `ContentStatus`
- `title`: string
- `parentCommentId`: string
- `version`: `Version`
- `body`: `BodyBulk`
- `_links`: `CommentLinks`

## ChildrenResponse

object

- `id`: string
- `status`: `OnlyArchivedAndCurrentContentStatus`
- `title`: string
- `type`: string
- `spaceId`: string
- `childPosition`: integer (int32, nullable)

## ClassificationLevel

object

- `id`: string
- `status`: `ClassificationLevelStatus`
- `order`: number
- `name`: string
- `description`: string
- `guideline`: string
- `color`: `ClassificationLevelColor`

## ClassificationLevelColor

string (enum RED|RED_BOLD|ORANGE|YELLOW|GREEN|BLUE|NAVY|TEAL|PURPLE|GREY|LIME)


## ClassificationLevelStatus

string (enum DRAFT|PUBLISHED|ARCHIVED)


## CommentBodyWrite

object

- `representation`: string (enum storage|atlas_doc_format|wiki)
- `value`: string

## CommentLinks

object

- `webui`: string

## CommentNestedBodyWrite

object

- `storage`: `CommentBodyWrite`
- `atlas_doc_format`: `CommentBodyWrite`
- `wiki`: `CommentBodyWrite`

## CommentSortOrder

string (enum created-date|-created-date|modified-date|-modified-date)


## CommentVersion

object

- `createdAt`: string (date-time)
- `message`: string
- `number`: integer (int32)
- `minorEdit`: boolean
- `authorId`: string
- `comment`: `VersionedEntity`

## ContentIdToContentTypeResponse

object

- `results`: map<string,string (enum page|blogpost|attachment|footer-comment|inline-comment) | string>

## ContentPermissionError

object

- `status`: integer (int32) **required**
- `code`: string **required**
- `title`: string **required**
- `detail`: string (nullable)

## ContentPermissionErrorResponse

object

- `errors`: array<`ContentPermissionError`> **required**

## ContentPermissionOutput

object

- `operation`: string (enum read|update) **required**
- `principal`: `ContentPermissionPrincipal` **required**
- `createdAt`: string (date-time)
- `createdBy`: string
- `createdByAaid`: string

## ContentPermissionPrincipal

object

- `principalType`: string (enum user|group) **required**
- `principalId`: string **required**

## ContentPermissionsRequest

object

- `permissions`: array<`OperationPermissionsInput`> **required**

## ContentPermissionsResponse

object

- `results`: array<`ContentPermissionOutput`> **required**
- `_links`: `MultiEntityLinks` **required**

## ContentPrincipalInput

object

- `principalType`: string (enum user|group) **required**
- `principalId`: string **required**

## ContentProperty

object

- `id`: string
- `key`: string
- `value`: any
- `version`: `Version`

## ContentPropertyCreateRequest

object

- `key`: string
- `value`: any

## ContentPropertySortOrder

string (enum key|-key)


## ContentPropertyUpdateRequest

object

- `key`: string
- `value`: any
- `version`: object
  - `number`: integer (int32)
  - `message`: string

## ContentRestrictionState

string (enum OPEN|EDIT_RESTRICTED|VIEW_RESTRICTED)


## ContentSortOrder

string (enum created-date|-created-date|id|-id|modified-date|-modified-date|child-position|-child-position|title|-title)


## ContentStatus

string (enum current|draft|archived|historical|trashed|deleted|any)


## CreateFooterCommentModel

object

- `blogPostId`: string
- `pageId`: string
- `parentCommentId`: string
- `attachmentId`: string
- `customContentId`: string
- `body`: `CommentBodyWrite` | `CommentNestedBodyWrite`

## CreateInlineCommentModel

object

- `blogPostId`: string
- `pageId`: string
- `parentCommentId`: string
- `body`: `CommentBodyWrite` | `CommentNestedBodyWrite`
- `inlineCommentProperties`: object
  - `textSelection`: string
  - `textSelectionMatchCount`: integer
  - `textSelectionMatchIndex`: integer

## CustomContentBodyBulk

object

- `raw`: `BodyType`
- `storage`: `BodyType`
- `atlas_doc_format`: `BodyType`

## CustomContentBodyRepresentation

string (enum raw|storage|atlas_doc_format)


## CustomContentBodyRepresentationSingle

string (enum raw|storage|atlas_doc_format|view|export_view|anonymous_export_view)


## CustomContentBodySingle

object

- `raw`: `BodyType`
- `storage`: `BodyType`
- `atlas_doc_format`: `BodyType`
- `view`: `BodyType`

## CustomContentBodyWrite

object

- `representation`: string (enum storage|atlas_doc_format|raw)
- `value`: string

## CustomContentBulk

object

- `id`: string
- `type`: string
- `status`: `ContentStatus`
- `title`: string
- `spaceId`: string
- `pageId`: string
- `blogPostId`: string
- `customContentId`: string
- `authorId`: string
- `createdAt`: string (date-time)
- `version`: `Version`
- `body`: `CustomContentBodyBulk`
- `_links`: `CustomContentLinks`

## CustomContentCommentModel

object

- `id`: string
- `status`: `ContentStatus`
- `title`: string
- `customContentId`: string
- `version`: `Version`
- `body`: `BodySingle`
- `_links`: `CommentLinks`

## CustomContentLinks

object

- `webui`: string

## CustomContentNestedBodyWrite

object

- `storage`: `CustomContentBodyWrite`
- `atlas_doc_format`: `CustomContentBodyWrite`
- `raw`: `CustomContentBodyWrite`

## CustomContentSingle

object

- `id`: string
- `type`: string
- `status`: `ContentStatus`
- `title`: string
- `spaceId`: string
- `pageId`: string
- `blogPostId`: string
- `customContentId`: string
- `authorId`: string
- `createdAt`: string (date-time)
- `version`: `Version`
- `labels`: object
  - `results`: array<`Label`>
  - `meta`: `OptionalFieldMeta`
  - `_links`: `OptionalFieldLinks`
- `properties`: object
  - `results`: array<`ContentProperty`>
  - `meta`: `OptionalFieldMeta`
  - `_links`: `OptionalFieldLinks`
- `operations`: object
  - `results`: array<`Operation`>
  - `meta`: `OptionalFieldMeta`
  - `_links`: `OptionalFieldLinks`
- `versions`: object
  - `results`: array<`Version`>
  - `meta`: `OptionalFieldMeta`
  - `_links`: `OptionalFieldLinks`
- `body`: `CustomContentBodySingle`
- `_links`: `CustomContentLinks`

## CustomContentSortOrder

string (enum id|-id|created-date|-created-date|modified-date|-modified-date|title|-title)


## CustomContentVersion

object

- `createdAt`: string (date-time)
- `message`: string
- `number`: integer (int32)
- `minorEdit`: boolean
- `authorId`: string
- `custom`: `VersionedEntity`

## DataPolicyMetadata

object

- `anyContentBlocked`: boolean

## DataPolicySpace

object

- `id`: string
- `key`: string
- `name`: string
- `description`: `SpaceDescription`
- `dataPolicy`: object
  - `anyContentBlocked`: boolean
- `icon`: `SpaceIcon`
- `_links`: `SpaceLinks`

## DatabaseAdfDocument

object

- `version`: integer **required**
- `type`: string (enum doc) **required**
- `content`: array<any>

## DatabaseAiSummaryField

`DatabaseFieldBase` & object

- `type`: string (enum aiSummary) **required**
- `settings`: `DatabaseAiSummaryFieldSettings` **required**

## DatabaseAiSummaryFieldSettings

object

- `aiRunMetadata`: `DatabaseAiSummaryRunMetadata`

## DatabaseAiSummaryRunMetadata

object

- `lastAiRunUserAccountId`: string (nullable) **required**

## DatabaseAiSummaryValue

object

- `type`: string (enum aiSummary) **required**
- `plain`: string (nullable) **required**
- `rich`: `DatabaseAdfDocument` **required**
- `metadata`: `DatabaseAiSummaryValueMetadata` **required**

## DatabaseAiSummaryValueMetadata

object

- `lastUpdatedAt`: string (nullable) **required**
- `lastUpdatedBy`: string (enum ai|user, nullable) **required**
- `lastUpdatedUserAccountId`: string (nullable) **required**

## DatabaseAttachmentField

`DatabaseFieldBase` & object

- `type`: string (enum attachment) **required**
- `settings`: `DatabaseEmptyFieldSettings` **required**

## DatabaseAttachmentRef

object

- `mediaId`: string **required**

## DatabaseAttachmentValue

object

- `type`: string (enum attachment) **required**
- `value`: array<`DatabaseAttachmentRef`> (nullable) **required**

## DatabaseBacklinkField

`DatabaseFieldBase` & object

- `type`: string (enum backlink) **required**
- `settings`: `DatabaseBacklinkFieldSettings` **required**

## DatabaseBacklinkFieldSettings

object

- `sourceDatabaseContentId`: string

## DatabaseCheckboxField

`DatabaseFieldBase` & object

- `type`: string (enum checkbox) **required**
- `settings`: `DatabaseEmptyFieldSettings` **required**

## DatabaseCheckboxValue

object

- `type`: string (enum checkbox) **required**
- `value`: boolean (nullable) **required**

## DatabaseDateField

`DatabaseFieldBase` & object

- `type`: string (enum date) **required**
- `settings`: `DatabaseDateFieldSettings` **required**

## DatabaseDateFieldSettings

object

- `includesTime`: boolean **required**

## DatabaseDateValue

object

- `type`: string (enum date) **required**
- `value`: string (nullable, pattern ^\d{4}-\d{2}-\d{2}(?:T\d{2}:\d{2}(?::\d{2}(?:\.\d+)?)?(?:Z|[+-]\d{2}:\d{2}))?$) **required**

## DatabaseEmptyFieldSettings

object


## DatabaseEntry

object

- `id`: string **required**
- `databaseId`: string **required**
- `values`: map<string,`DatabaseFieldValue`> **required**

## DatabaseEntryDetailsField

`DatabaseFieldBase` & object

- `type`: string (enum entryDetails) **required**
- `settings`: `DatabaseEntryDetailsFieldSettings` **required**

## DatabaseEntryDetailsFieldSettings

object

- `sourceFieldId`: string
- `lookupFieldId`: string

## DatabaseEntryLinkField

`DatabaseFieldBase` & object

- `type`: string (enum entryLink) **required**
- `settings`: `DatabaseEmptyFieldSettings` **required**

## DatabaseEntryLinkValue

object

- `type`: string (enum entryLink) **required**
- `value`: array<`DatabaseEntryRef`> (nullable) **required**

## DatabaseEntryListResponse

object

- `results`: array<`DatabaseEntry`> **required**
- `_links`: `DatabasesPublicApiLinks` **required**
- `meta`: `DatabasePaginationMeta` **required**

## DatabaseEntryRef

object

- `entryId`: string **required**
- `databaseContentId`: string

## DatabaseEntrySingleResponse

object

- `id`: string **required**
- `databaseId`: string **required**
- `values`: map<string,`DatabaseFieldValue`> **required**
- `_links`: `DatabasesPublicApiLinks` **required**

## DatabaseError

object

- `id`: string (uuid)
- `status`: integer (int32) **required**
- `code`: string (enum INVALID_CURSOR|UNSUPPORTED_CURSOR_VERSION|CURSOR_QUERY_MISMATCH|INVALID_FILTER|UNSUPPORTED_FILTER_OPERATOR|FIELD_NOT_FILTERABLE|FIELD_NOT_SORTABLE|UNSUPPORTED_SORT_MODIFIER|INVALID_SORT|INVALID_PAGE_LIMIT|DATABASE_NOT_FOUND|ENTRY_NOT_FOUND|DATABASE_TOO_LARGE|SERVICE_DISABLED|SERVICE_UNAVAILABLE|REQUEST_TIMEOUT) **required**
- `title`: string **required**
- `detail`: string (nullable)

## DatabaseErrorResponse

object

- `errors`: array<`DatabaseError`> **required**

## DatabaseField

`DatabaseTextField` | `DatabaseAiSummaryField` | `DatabaseNumberField` | `DatabaseCheckboxField` | `DatabaseDateField` | `DatabaseSelectField` | `DatabaseHyperlinkField` | `DatabaseUserField` | `DatabaseAttachmentField` | `DatabaseJiraIssueField` | `DatabasePageLinkField` | `DatabaseEntryLinkField` | `DatabaseEntryDetailsField` | `DatabaseBacklinkField` | `DatabasePageDetailsField` | `DatabasePageLabelsField` | `DatabasePageStatusField` | `DatabaseJiraIssueDetailsField`


## DatabaseFieldBase

object

- `id`: string **required**
- `name`: string **required**
- `description`: string (nullable) **required**

## DatabaseFieldListResponse

object

- `results`: array<`DatabaseField`> **required**
- `_links`: `DatabasesPublicApiLinks` **required**
- `meta`: `DatabasePaginationMeta` **required**

## DatabaseFieldSettings

`DatabaseTextFieldSettings` | `DatabaseAiSummaryFieldSettings` | `DatabaseNumberFieldSettings` | `DatabaseSelectFieldSettings` | `DatabaseDateFieldSettings` | `DatabaseJiraIssueFieldSettings` | `DatabaseEntryDetailsFieldSettings` | `DatabaseJiraIssueDetailsFieldSettings` | `DatabasePageDetailsFieldSettings` | `DatabaseSourceFieldSettings` | `DatabaseBacklinkFieldSettings` | `DatabaseEmptyFieldSettings`


## DatabaseFieldType

string (enum text|aiSummary|number|checkbox|date|select|hyperlink|user|attachment|jiraIssue|pageLink|entryLink|entryDetails|backlink|pageDetails|pageLabels|pageStatus|jiraIssueDetails)


## DatabaseFieldValue

`DatabaseTextValue` | `DatabaseAiSummaryValue` | `DatabaseNumberValue` | `DatabaseCheckboxValue` | `DatabaseDateValue` | `DatabaseSelectValue` | `DatabaseHyperlinkValue` | `DatabaseUserValue` | `DatabaseAttachmentValue` | `DatabaseJiraIssueValue` | `DatabasePageLinkValue` | `DatabaseEntryLinkValue` | `DatabaseNullLookupValue`


## DatabaseHyperlinkField

`DatabaseFieldBase` & object

- `type`: string (enum hyperlink) **required**
- `settings`: `DatabaseEmptyFieldSettings` **required**

## DatabaseHyperlinkItem

object

- `url`: string **required**
- `label`: string

## DatabaseHyperlinkValue

object

- `type`: string (enum hyperlink) **required**
- `value`: array<`DatabaseHyperlinkItem`> (nullable) **required**

## DatabaseJiraIssueDetailsField

`DatabaseFieldBase` & object

- `type`: string (enum jiraIssueDetails) **required**
- `settings`: `DatabaseJiraIssueDetailsFieldSettings` **required**

## DatabaseJiraIssueDetailsFieldSettings

object

- `sourceFieldId`: string
- `jiraFieldKey`: string

## DatabaseJiraIssueField

`DatabaseFieldBase` & object

- `type`: string (enum jiraIssue) **required**
- `settings`: `DatabaseJiraIssueFieldSettings` **required**

## DatabaseJiraIssueFieldSettings

object

- `cloudId`: string
- `serverId`: string

## DatabaseJiraIssueRef

object

- `key`: string **required**
- `cloudId`: string
- `serverId`: string

## DatabaseJiraIssueValue

object

- `type`: string (enum jiraIssue) **required**
- `value`: array<`DatabaseJiraIssueRef`> (nullable) **required**

## DatabaseLinks

object

- `webui`: string

## DatabaseNullLookupValue

object

- `type`: string (enum entryDetails|backlink|pageDetails|pageLabels|pageStatus|jiraIssueDetails) **required**
- `value`: any (enum None, nullable) **required**

## DatabaseNumberField

`DatabaseFieldBase` & object

- `type`: string (enum number) **required**
- `settings`: `DatabaseNumberFieldSettings` **required**

## DatabaseNumberFieldSettings

object

- `format`: `DatabaseNumberFormat`

## DatabaseNumberFormat

object

- `type`: string (enum number|percent|currency|custom) **required**
- `prefix`: string
- `suffix`: string
- `decimalPlaces`: integer (minimum 0)

## DatabaseNumberValue

object

- `type`: string (enum number) **required**
- `value`: string (nullable, pattern ^(-?0\.\d+|0|-?[1-9]\d*(\.\d+)?)([eE][+-]?\d+)?$) **required**

## DatabasePageDetailsField

`DatabaseFieldBase` & object

- `type`: string (enum pageDetails) **required**
- `settings`: `DatabasePageDetailsFieldSettings` **required**

## DatabasePageDetailsFieldSettings

object

- `sourceFieldId`: string
- `pagePropertyKey`: string

## DatabasePageLabelsField

`DatabaseFieldBase` & object

- `type`: string (enum pageLabels) **required**
- `settings`: `DatabaseSourceFieldSettings` **required**

## DatabasePageLinkField

`DatabaseFieldBase` & object

- `type`: string (enum pageLink) **required**
- `settings`: `DatabaseEmptyFieldSettings` **required**

## DatabasePageLinkValue

object

- `type`: string (enum pageLink) **required**
- `value`: array<`DatabasePageRef`> (nullable) **required**

## DatabasePageRef

object

- `contentId`: string **required**

## DatabasePageStatusField

`DatabaseFieldBase` & object

- `type`: string (enum pageStatus) **required**
- `settings`: `DatabaseSourceFieldSettings` **required**

## DatabasePaginationMeta

object

- `pageSize`: integer (int32, minimum 1, maximum 250) **required**
- `cursor`: string
- `hasMore`: boolean **required**

## DatabaseSelectField

`DatabaseFieldBase` & object

- `type`: string (enum select) **required**
- `settings`: `DatabaseSelectFieldSettings` **required**

## DatabaseSelectFieldSettings

object

- `options`: array<`DatabaseSelectOption`> **required**

## DatabaseSelectOption

object

- `id`: string **required**
- `label`: string **required**

## DatabaseSelectValue

object

- `type`: string (enum select) **required**
- `value`: array<`DatabaseSelectOption`> (nullable) **required**

## DatabaseSingle

object

- `id`: string
- `type`: string
- `status`: `ContentStatus`
- `title`: string
- `parentId`: string
- `parentType`: `ParentContentType`
- `position`: integer (int32, nullable)
- `authorId`: string
- `ownerId`: string
- `createdAt`: string (date-time)
- `spaceId`: string
- `version`: `Version`
- `_links`: `DatabaseLinks`

## DatabaseSourceFieldSettings

object

- `sourceFieldId`: string

## DatabaseTextField

`DatabaseFieldBase` & object

- `type`: string (enum text) **required**
- `settings`: `DatabaseTextFieldSettings` **required**

## DatabaseTextFieldSettings

object

- `richText`: boolean **required**

## DatabaseTextValue

object

- `type`: string (enum text) **required**
- `plain`: string (nullable) **required**
- `rich`: `DatabaseAdfDocument` **required**

## DatabaseUserField

`DatabaseFieldBase` & object

- `type`: string (enum user) **required**
- `settings`: `DatabaseEmptyFieldSettings` **required**

## DatabaseUserRef

object

- `accountId`: string **required**

## DatabaseUserValue

object

- `type`: string (enum user) **required**
- `value`: array<`DatabaseUserRef`> (nullable) **required**

## DatabaseView

object

- `id`: string **required**
- `name`: string **required**
- `type`: string (enum table|gallery|board|calendar|timeline) **required**

## DatabaseViewListResponse

object

- `results`: array<`DatabaseView`> **required**
- `_links`: `DatabasesPublicApiLinks` **required**
- `meta`: `DatabasePaginationMeta` **required**

## DatabasesPublicApiLinks

object

- `self`: string (pattern ^/wiki/api/v2/databases/) **required**
- `base`: string (uri) **required**
- `next`: string (pattern ^/wiki/api/v2/databases/)

## DeleteSpaceRoleResponse

object

- `taskId`: string

## DescendantsResponse

object

- `id`: string
- `status`: `OnlyArchivedAndCurrentContentStatus`
- `title`: string
- `type`: string
- `parentId`: string
- `depth`: integer (int32)
- `childPosition`: integer (int32, nullable)

## DetailedVersion

object

- `number`: integer (int32)
- `authorId`: string
- `message`: string
- `createdAt`: string (date-time)
- `minorEdit`: boolean
- `contentTypeModified`: boolean
- `collaborators`: array<string>
- `prevVersion`: integer (int32)
- `nextVersion`: integer (int32)

## FolderLinks

object

- `webui`: string

## FolderSingle

object

- `id`: string
- `type`: string
- `status`: `ContentStatus`
- `title`: string
- `parentId`: string
- `parentType`: `ParentContentType`
- `position`: integer (int32, nullable)
- `authorId`: string
- `ownerId`: string
- `createdAt`: string (date-time)
- `spaceId`: string
- `version`: `Version`
- `_links`: `FolderLinks`

## FooterCommentModel

object

- `id`: string
- `status`: `ContentStatus`
- `title`: string
- `blogPostId`: string
- `pageId`: string
- `attachmentId`: string
- `customContentId`: string
- `parentCommentId`: string
- `version`: `Version`
- `properties`: object
  - `results`: array<`ContentProperty`>
  - `meta`: `OptionalFieldMeta`
  - `_links`: `OptionalFieldLinks`
- `operations`: object
  - `results`: array<`Operation`>
  - `meta`: `OptionalFieldMeta`
  - `_links`: `OptionalFieldLinks`
- `likes`: object
  - `results`: array<`Like`>
  - `meta`: `OptionalFieldMeta`
  - `_links`: `OptionalFieldLinks`
- `versions`: object
  - `results`: array<`Version`>
  - `meta`: `OptionalFieldMeta`
  - `_links`: `OptionalFieldLinks`
- `body`: `BodySingle`
- `_links`: `CommentLinks`

## Icon

object

- `path`: string **required**
- `isDefault`: boolean **required**

## InlineCommentChildrenModel

object

- `id`: string
- `status`: `ContentStatus`
- `title`: string
- `parentCommentId`: string
- `version`: `Version`
- `body`: `BodyBulk`
- `resolutionStatus`: `InlineCommentResolutionStatus`
- `properties`: `InlineCommentProperties`
- `_links`: `CommentLinks`

## InlineCommentModel

object

- `id`: string
- `status`: `ContentStatus`
- `title`: string
- `blogPostId`: string
- `pageId`: string
- `parentCommentId`: string
- `version`: `Version`
- `body`: `BodySingle`
- `resolutionLastModifierId`: string
- `resolutionLastModifiedAt`: string (date-time)
- `resolutionStatus`: `InlineCommentResolutionStatus`
- `properties`: object
  - `results`: array<`ContentProperty`>
  - `meta`: `OptionalFieldMeta`
  - `_links`: `OptionalFieldLinks`
  - `inlineMarkerRef`: string
  - `inlineOriginalSelection`: string
- `operations`: object
  - `results`: array<`Operation`>
  - `meta`: `OptionalFieldMeta`
  - `_links`: `OptionalFieldLinks`
- `likes`: object
  - `results`: array<`Like`>
  - `meta`: `OptionalFieldMeta`
  - `_links`: `OptionalFieldLinks`
- `versions`: object
  - `results`: array<`Version`>
  - `meta`: `OptionalFieldMeta`
  - `_links`: `OptionalFieldLinks`
- `_links`: `CommentLinks`

## InlineCommentProperties

object

- `inlineMarkerRef`: string
- `inlineOriginalSelection`: string

## InlineCommentResolutionStatus

string (enum open|reopened|resolved|dangling)


## Label

object

- `id`: string
- `name`: string
- `prefix`: string

## LabelSortOrder

string (enum created-date|-created-date|id|-id|name|-name)


## Like

object

- `accountId`: string

## ListSpacePermissionCombinationsResponse

object

- `results`: array<`BulkTransitionCombinationEntry`> **required**
- `generatedAt`: string (nullable)
- `cursor`: string (nullable)

## MultiEntityLinks

object

- `next`: string
- `base`: string

## OnlyArchivedAndCurrentContentStatus

string (enum current|archived)


## Operation

object

- `operation`: string
- `targetType`: string

## OperationPermissionsInput

object

- `operation`: string (enum read|update) **required**
- `principals`: array<`ContentPrincipalInput`> **required**

## OptionalFieldLinks

object

- `self`: string

## OptionalFieldMeta

object

- `hasMore`: boolean
- `cursor`: string

## PageBodyWrite

object

- `representation`: string (enum storage|atlas_doc_format|wiki)
- `value`: string

## PageBulk

object

- `id`: string
- `status`: `ContentStatus`
- `title`: string
- `spaceId`: string
- `parentId`: string
- `parentType`: `ParentContentType`
- `position`: integer (int32, nullable)
- `authorId`: string
- `ownerId`: string (nullable)
- `lastOwnerId`: string (nullable)
- `subtype`: string (nullable)
- `createdAt`: string (date-time)
- `version`: `Version`
- `body`: `BodyBulk`
- `_links`: `AbstractPageLinks`

## PageCommentModel

object

- `id`: string
- `status`: `ContentStatus`
- `title`: string
- `pageId`: string
- `version`: `Version`
- `body`: `BodyBulk`
- `_links`: `CommentLinks`

## PageInlineCommentModel

object

- `id`: string
- `status`: `ContentStatus`
- `title`: string
- `pageId`: string
- `version`: `Version`
- `body`: `BodyBulk`
- `resolutionStatus`: `InlineCommentResolutionStatus`
- `properties`: `InlineCommentProperties`
- `_links`: `CommentLinks`

## PageNestedBodyWrite

object

- `storage`: `PageBodyWrite`
- `atlas_doc_format`: `PageBodyWrite`
- `wiki`: `PageBodyWrite`

## PageSingle

object

- `id`: string
- `status`: `ContentStatus`
- `title`: string
- `spaceId`: string
- `parentId`: string
- `parentType`: `ParentContentType`
- `position`: integer (int32, nullable)
- `authorId`: string
- `ownerId`: string (nullable)
- `lastOwnerId`: string (nullable)
- `createdAt`: string (date-time)
- `version`: `Version`
- `body`: `BodySingle`
- `labels`: object
  - `results`: array<`Label`>
  - `meta`: `OptionalFieldMeta`
  - `_links`: `OptionalFieldLinks`
- `properties`: object
  - `results`: array<`ContentProperty`>
  - `meta`: `OptionalFieldMeta`
  - `_links`: `OptionalFieldLinks`
- `operations`: object
  - `results`: array<`Operation`>
  - `meta`: `OptionalFieldMeta`
  - `_links`: `OptionalFieldLinks`
- `likes`: object
  - `results`: array<`Like`>
  - `meta`: `OptionalFieldMeta`
  - `_links`: `OptionalFieldLinks`
- `versions`: object
  - `results`: array<`Version`>
  - `meta`: `OptionalFieldMeta`
  - `_links`: `OptionalFieldLinks`
- `isFavoritedByCurrentUser`: boolean
- `_links`: `AbstractPageLinks`

## PageSortOrder

string (enum id|-id|created-date|-created-date|modified-date|-modified-date|title|-title)


## PageVersion

object

- `createdAt`: string (date-time)
- `message`: string
- `number`: integer (int32)
- `minorEdit`: boolean
- `authorId`: string
- `page`: `VersionedEntity`

## ParentContentType

string (enum page|whiteboard|database|embed|folder)


## PermittedOperationsResponse

object

- `operations`: array<`Operation`>

## PrimaryBodyRepresentation

string (enum storage|atlas_doc_format)


## PrimaryBodyRepresentationSingle

string (enum storage|atlas_doc_format|view|export_view|anonymous_export_view|styled_view|editor)


## Principal

object

- `principalType`: `PrincipalType`
- `principalId`: string

## PrincipalType

string (enum USER|GROUP|ACCESS_CLASS)


## Redaction

object

- `redactions`: array<`RedactionPointer`>

## RedactionPointer

object

- `pointer`: string **required**
- `from`: integer (minimum 0)
- `to`: integer (minimum 0)
- `reason`: string (nullable)

## RedactionPointerResponse

object

- `pointer`: string
- `from`: integer
- `to`: integer
- `reason`: string
- `redactionId`: string (uuid)

## RedactionResponse

object

- `body`: `RedactionSectionResponse`
- `title`: `RedactionSectionResponse`

## RedactionSectionResponse

object

- `redactions`: array<`RedactionPointerResponse`>

## RestrictionStateResponse

object

- `state`: `ContentRestrictionState` **required**
- `createdAt`: string (date-time)
- `createdBy`: string
- `createdByAaid`: string

## RoleType

string (enum SYSTEM|CUSTOM)


## SmartLinkLinks

object

- `webui`: string

## SmartLinkSingle

object

- `id`: string
- `type`: string
- `status`: `ContentStatus`
- `title`: string
- `parentId`: string
- `parentType`: `ParentContentType`
- `position`: integer (int32, nullable)
- `authorId`: string
- `ownerId`: string
- `createdAt`: string (date-time)
- `embedUrl`: string
- `spaceId`: string
- `version`: `Version`
- `_links`: `SmartLinkLinks`

## SpaceBulk

object

- `id`: string
- `key`: string
- `name`: string
- `type`: `SpaceType`
- `status`: `SpaceStatus`
- `authorId`: string
- `spaceOwnerId`: string
- `currentActiveAlias`: string
- `createdAt`: string (date-time)
- `homepageId`: string
- `description`: `SpaceDescription`
- `icon`: `SpaceIcon`
- `_links`: `SpaceLinks`

## SpaceDescription

object

- `plain`: `BodyType`
- `view`: `BodyType`

## SpaceDescriptionBodyRepresentation

string (enum plain|view)


## SpaceIcon

object

- `path`: string
- `apiDownloadLink`: string

## SpaceLinks

object

- `webui`: string

## SpacePermission

object

- `id`: string
- `displayName`: string
- `description`: string
- `requiredPermissionIds`: array<string>

## SpacePermissionAssignment

object

- `id`: string
- `principal`: object
  - `type`: string (enum user|group|role)
  - `id`: string
- `operation`: object
  - `key`: string (enum use|create|read|update|delete|copy|move|export|purge|purge_version|administer|restore|create_space|restrict_content|archive)
  - `targetType`: string (enum page|blogpost|comment|attachment|whiteboard|database|embed|folder|space|application|userProfile)

## SpaceProperty

object

- `id`: string
- `key`: string
- `value`: any
- `createdAt`: string (date-time)
- `createdBy`: string
- `version`: object
  - `createdAt`: string (date-time)
  - `createdBy`: string
  - `message`: string
  - `number`: integer (int32)

## SpacePropertyCreateRequest

object

- `key`: string
- `value`: any

## SpacePropertyUpdateRequest

object

- `key`: string
- `value`: any
- `version`: object
  - `number`: integer (int32)
  - `message`: string

## SpaceRole

object

- `id`: string
- `type`: `RoleType`
- `name`: string
- `description`: string
- `spacePermissions`: array<string>

## SpaceRoleAssignment

object

- `principal`: `Principal`
- `roleId`: string

## SpaceSingle

object

- `id`: string
- `key`: string
- `name`: string
- `type`: `SpaceType`
- `status`: `SpaceStatus`
- `authorId`: string
- `spaceOwnerId`: string
- `createdAt`: string (date-time)
- `homepageId`: string
- `description`: `SpaceDescription`
- `icon`: `SpaceIcon`
- `labels`: object
  - `results`: array<`Label`>
  - `meta`: `OptionalFieldMeta`
  - `_links`: `OptionalFieldLinks`
- `properties`: object
  - `results`: array<`SpaceProperty`>
  - `meta`: `OptionalFieldMeta`
  - `_links`: `OptionalFieldLinks`
- `operations`: object
  - `results`: array<`Operation`>
  - `meta`: `OptionalFieldMeta`
  - `_links`: `OptionalFieldLinks`
- `permissions`: object
  - `results`: array<`SpacePermissionAssignment`>
  - `meta`: `OptionalFieldMeta`
  - `_links`: `OptionalFieldLinks`
- `_links`: `SpaceLinks`

## SpaceSortOrder

string (enum id|-id|key|-key|name|-name)


## SpaceStatus

string (enum current|archived|trashed)


## SpaceType

string (enum global|collaboration|knowledge_base|personal|system|onboarding|xflow_sample_space)


## Task

object

- `id`: string
- `localId`: string
- `spaceId`: string
- `pageId`: string
- `blogPostId`: string
- `status`: string (enum complete|incomplete)
- `body`: `TaskBodySingle`
- `createdBy`: string
- `assignedTo`: string
- `completedBy`: string
- `createdAt`: string (date-time)
- `updatedAt`: string (date-time)
- `dueAt`: string (date-time)
- `completedAt`: string (date-time)

## TaskBodySingle

object

- `storage`: `BodyType`
- `atlas_doc_format`: `BodyType`

## UpdateFooterCommentModel

object

- `version`: object
  - `number`: integer
  - `message`: string
- `body`: `CommentBodyWrite` | `CommentNestedBodyWrite`

## UpdateInlineCommentModel

object

- `version`: object
  - `number`: integer
  - `message`: string
- `body`: `CommentBodyWrite` | `CommentNestedBodyWrite`
- `resolved`: boolean

## UpdateSpaceRoleResponse

object

- `id`: string
- `type`: `RoleType`
- `name`: string
- `description`: string
- `taskId`: string

## User

object

- `displayName`: string
- `timeZone`: string
- `personalSpaceId`: string
- `isExternalCollaborator`: boolean
- `accountStatus`: `AccountStatus`
- `accountId`: string
- `email`: string
- `accountType`: `AccountType`
- `publicName`: string
- `profilePicture`: `Icon`

## Version

object

- `createdAt`: string (date-time)
- `message`: string
- `number`: integer (int32)
- `minorEdit`: boolean
- `authorId`: string

## VersionSortOrder

string (enum modified-date|-modified-date)


## VersionedEntity

object

- `title`: string
- `id`: string
- `body`: `BodyBulk`

## WhiteboardLinks

object

- `webui`: string
- `editui`: string

## WhiteboardSingle

object

- `id`: string
- `type`: string
- `status`: `ContentStatus`
- `title`: string
- `parentId`: string
- `parentType`: `ParentContentType`
- `position`: integer (int32, nullable)
- `authorId`: string
- `ownerId`: string
- `createdAt`: string (date-time)
- `spaceId`: string
- `version`: `Version`
- `_links`: `WhiteboardLinks`

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
     "read:page:confluence": "View pages and blogposts and their properties.",
     "read:space:confluence": "View spaces and their properties.",
     "read:attachment:confluence": "View attachments and their properties.",
     "read:comment:confluence": "View comments and their properties.",
     "read:custom-content:confluence": "View custom content and their properties.",
     "read:task:confluence": "View tasks.",
     "read:whiteboard:confluence": "View whiteboards and their properties.",
     "read:database:confluence": "View databases and their properties.",
     "read:embed:confluence": "View Smart Links in the content tree and their properties.",
     "read:folder:confluence": "View folders and their properties.",
     "read:hierarchical-content:confluence": "View children and descendants in the content tree.",
     "write:space:confluence": "Create and update spaces and their properties.",
     "write:page:confluence": "Create and update pages and blog posts and their properties.",
     "write:comment:confluence": "Create and update comments and their properties.",
     "write:custom-content:confluence": "Create and update custom content and their properties.",
     "write:whiteboard:confluence": "Create and update whiteboards and their properties.",
     "write:database:confluence": "Create and update databases and their properties.",
     "write:embed:confluence": "Create and update Smart Links in the content tree and their properties.",
     "write:folder:confluence": "Create and update folders and their properties.",
     "write:app-data:confluence": "Create, update and delete app properties.",
     "delete:custom-content:confluence": "Delete custom content.",
     "delete:page:confluence": "Delete pages and blog posts.",
     "delete:comment:confluence": "Delete comments.",
     "delete:whiteboard:confluence": "Delete whiteboards.",
     "delete:database:confluence": "Delete databases.",
     "delete:embed:confluence": "Delete Smart Links in the content tree.",
     "delete:folder:confluence": "Delete folders."
    }
   }
  }
 }
}
```
