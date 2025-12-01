# VOStuff functional requirements

VOStuff is an application to record and track collections of stuff.

This document lists requirements for the application, each one starting with a
requirement ID, e.g. O1 or D3.

## Organisations

- O1 All stuff is contained in an organisation.
- O2 Users are authenticated using OIDC.
- O3 Multiple OIDC identity providers must be supported, including common ones
like Google Identity.
- O4 It must be possible to configure additional OIDC providers as needed.
- O5 Each user belongs to one or more organisations.
- O6 A users needs to select an organisation (when they are a member of more than
one) when using the API or UI.
- O7 A pre-defined SYSTEM organisation exists that owns and manages all the system data.
- O8 A pre-defined set of users belong to a SYSTEM organisation and can administer system data.

## Data

- D1 Each item of stuff has the following core fields:
    - ID: UUID
    - Date entered
    - Date acquired
    - Type: type of item, see below
    - State: the current state of the object, see below
    - Name: short name of the item, used for display purposes
    - Description: longer description of the item
    - Notes: Markdown formatted notes related to the item
    - Location: Current location of the item, one of a set of defined
    locations. See below.
    - Collections: reference to one or more collection that this item belongs
    to
    - Tags: List of zero or more tags associated with this item
    - Orgnisation: ID of the Organisation to which this item belongs
- D2 Each item has a type.
- D3 The following item types are to be implemented, some with additional
related data fields (only some types have additional fields):
    - Vinyl
        - Size, one of: 12 inch, 6 inch, other
        - Speed, one of: 33, 45, other
        - Channels, one of: mono, stereo, surround, other
        - Disks, non-zero integer
        - Media grading, one of: Mint, Near mint, Excellent, Good, Fair, Poor
        - Sleeve grading, one of: Mint, Near mint, Excellent, Good, Fair,
        Poor
    - CD
        - Disks, non-zero integer
    - Cassette
        - Cassettes, non-zero integer
    - Book
    - Score
    - Electronics
    - Misc
- D4 Collections are a user-defined list of collection to which items can
belong.
- D5 Each collection definition consists of the following data items:
    - ID: UUID
    - Name: short name of the collection, used for display purposes
    - Description: longer descripton of the collection
    - Notes: Markdown formatted notes related to the collection
    - Orgnisation: ID of the Organisation to which this collection belongs
- D6 Tags can be attached to items to enable classification. Tag definitions
have the following fields:
    - Name: short string, unique within organisation
    - Organisation: ID of organisation to which this tag belongs
- D9 Each item can be in one of the following states, some with additional
fields:
    - CURRENT: indicates the item is in our possession
    - LOANED: inidcates the item is currently loaned to someone, with the
    following fields:
        - Date loaned
        - Date due back (optional)
        - Loaned to: simple string of the person/entity who has the item
    - MISSING: indicates that item is missing, with the following fields:
        - Date missing
    - DISPOSED: indicates that the item has been disposed of, with the
    following fields:
        - Date disposed
- D11 An item can be moved between any state.
- D10 Locations are user created and consist of:
    - ID: UUID
    - Name: short string, used for display
- D7 Organisations are defined by the following fields:
    - ID: UUID
    - Name: short name of the organisation, used for display purposes
    - Description: longer description of the organisation
- D8 Users are registered with the system and recorded with the following
fields:
    - ID: UUID
    - Name: short name, used for display purposes
    - Identity: string, typically an email address, used to identify the user
    from the OIDC identity token
    - Orgnisations: List of organisation IDs to which this user belongs
- D9 All changes to any item (apart from audit records themselves) in this data
model should be tracked in an Audit table.
    - Each audit entry should contain:
        - ID: UUID
        - ID of item
        - Date of change
        - Details of change: A text summary of the changes made to the item.
- D12 A minimal set of fields need to be indexed to start with. We can add
additional indexes as needed.

## API

- A1 The API will use normal REST standards, with JSON as its primary format.
- A2 With the exception of login methods, all calls will require authentication
using a session token, identifying the user and their selected org.
- A3 A login API will allow the user to authenticate with an OIDC identity
token and create a session token identifying which of their orgnisations should
be used for the session.
- A4 APIs are needed to CRUD all the data items.
- A5 Item, collection and tag data are owned by specific organisations and can
only be operated upon in sessions with that organisation.
- A6 Organisation and user data is owned by the SYSTEM orgnisation and can only
be operated upon in sessions with that organisation.
- A10 To start with users are added to the system and associated with
organisations by an admin (member of the SYSTEM organisation).
- A7 The API needs to support an OIDC authentication flow.
- A8 When creating a session token (login), the API needs to support a flow
where the user can select from their available organisations and then resubmit
the identity token and orgnisation ID to get their session (authorisation)
token.
- A9 Session tokens should have a maximum lifetime of 12 hours. After this time
a new login request needs to be made. We may change this behaviour in future.

## UI

### Stack

The UI will be built on Leptos:

- Using SSR + Hydration as it offers:
    - Improved handling of security tokens.
    - Provides better initial rendering performance compared to client side
    rendering.
    - Wraps server calls as server-side functions.
- We'll separate the client web server from the API web server.
    - This ensures the API is kept usable by other clients.
    - Provide good separation between the client and server elements of the
    solution.
- The application will use custom CSS for styling.

### Layout

This will essentially be a one screen application:

- A main page with filtering elements at the top and a results table below
listing the matching items.
- The filtering section will support:
    - By: type, tag, location.
    - Can select zero or more of each filter item from a drop down listing the
    items and selectable by a checkbox.
        - When not dropped down, the filter will be a rendered view of the
        selected items. Truncating the render as needed to fit the space, but
        indicating how many items are in the select list of filter items.
- The results table will show the primary fields in columns.
    - Each item can be expanded, opening a view with all the appropriate fields
    for that type shown (all the core fields not shown and any type-specific
    fields).
- The results can be ordered by any of the primary fields. Clicking on the
header for the that column selects it for ordering and subsequent clicks change
the ordering direction.
- The primary fields are:
    - Item type
    - Name
    - Item state
    - Location
- There will be a button to add a new item.
    - It will pop up a window with the core fields and a type selector.
    - When a type is selected, additional fields are populated as needed for
    that type.
    - The type can be changed during creation, with any data entered for a type
    specific field being remembered if the type is changed away then back
    again.
    - There will be a OK button to create the item, and a cancel button.
- There will be an edit button which will bring up an edit window for the
currently selected item in the main results page.
    - The edit window is like the create window, but is pre-populated with data
    from the item.
    - The item type cannot be changed whilst editing.
- There will be a delete button which will delete the currently selected item
in the main view after the user has confirmed this action.
- The application will show an authentication screen if no valid token is
available.
    - Authentication needs to handle initial username password acquisition and
    org selection as a secondary action.
- The current username is displayed at the top right of the main page, together
with a logout button.

### Core UI Requirements

- UI1 The results table must support pagination with configurable page size
(10, 25, 50, 100 items per page).
    - Previous/Next buttons and page number selection.
    - Display showing "X-Y of Z items" to indicate current position.

- UI2 A search/filter bar must allow text search across item name and
description fields.
    - Search should be case-insensitive.
    - Can be combined with other filters (type, tag, location, collection,
    state).

- UI3 The filter section must include filtering by:
    - Item state (CURRENT, LOANED, MISSING, DISPOSED)
    - Collection membership
    - In addition to the already specified: type, tag, location

- UI4 Markdown notes fields must be editable and viewable:
    - A markdown editor with preview for entering/editing notes on items and
    collections.
    - Rendered markdown display in the expanded item view and collection views.
    - Basic markdown formatting support: headings, lists, bold, italic, links,
    code blocks.

### Collections Management

- UI5 A Collections view/page must be accessible from the main navigation.
    - Lists all collections for the current organization.
    - Shows collection name, description, and item count.
    - Supports search/filter by collection name.

- UI6 A "Create Collection" button must allow users to create new collections.
    - Popup/modal window with fields: name, description, notes (markdown).
    - OK button to create, Cancel button to abort.

- UI7 Collections must be editable via an Edit button.
    - Same popup as creation, pre-populated with existing data.
    - Cannot change collection ID.

- UI8 Collections must be deletable via a Delete button.
    - Confirmation dialog required.
    - Clarify what happens to items in the collection (collection reference
    removed, items remain).

- UI9 Clicking on a collection should show all items in that collection.
    - Essentially applying a collection filter to the main items view.

- UI10 During item creation/editing, collections must be assignable via a
multi-select dropdown.
    - Shows all available collections for the organization.
    - Can select multiple collections.
    - Can create new collections inline (optional enhancement).

### Locations Management

- UI11 A Locations management interface must be accessible (sidebar, settings,
or dedicated page).
    - Lists all locations for the current organization.
    - Shows location name and item count.

- UI12 A "Create Location" button must allow users to create new locations.
    - Simple dialog with location name field.
    - OK/Cancel buttons.

- UI13 Locations must be editable via an Edit button.
    - Dialog with location name, pre-populated.
    - OK/Cancel buttons.

- UI14 Locations must be deletable via a Delete button.
    - Confirmation required.
    - Cannot delete if items reference this location (show error) OR provide
    option to reassign items to another location.

### Tags Management

- UI15 A Tags management interface must be accessible (sidebar, settings, or
dedicated page).
    - Lists all tags for the current organization.
    - Shows tag name and usage count (number of items with this tag).

- UI16 A "Create Tag" button must allow users to create new tags.
    - Simple dialog with tag name field.
    - Tag names must be unique within organization.
    - OK/Cancel buttons.

- UI17 Tags must be editable via an Edit button.
    - Dialog to rename tag.
    - Updates all items using this tag.

- UI18 Tags must be deletable via a Delete button.
    - Confirmation required.
    - Removes tag from all items.

- UI19 During item creation/editing, tags must be assignable via a multi-select
or tag input widget.
    - Shows existing tags as suggestions.
    - Can create new tags inline.
    - Can remove tags from the item.

### Item State Management

- UI20 The item edit/view interface must support state transitions.
    - A "Change State" button or state dropdown in the edit view.
    - When selecting a new state, appropriate additional fields appear:
        - LOANED: Date loaned (default: today), Date due back (optional),
        Loaned to (text field)
        - MISSING: Date missing (default: today)
        - DISPOSED: Date disposed (default: today)
        - CURRENT: No additional fields

- UI21 State-specific information must be displayed in the expanded item view.
    - For LOANED items: Show who has it, when loaned, when due back.
    - For MISSING items: Show when reported missing.
    - For DISPOSED items: Show when disposed.

- UI22 The main results table should visually distinguish items by state.
    - Color coding or icons for different states.
    - LOANED items could show due date if approaching.

### Organization Management

- UI23 Users belonging to multiple organizations must be able to switch between
organizations without logging out.
    - Organization selector in the top navigation (next to username).
    - Dropdown showing all user's organizations.
    - Selecting an organization refreshes the view with that org's data.
    - Preserves session, only changes active organization context.

- UI24 The top navigation must display the current organization name clearly.
    - Shows which organization's data is currently being viewed.

### Admin UI (for SYSTEM organization members)

- UI25 An Admin section must be accessible only to SYSTEM organization members.
    - Separate navigation item or admin dashboard.
    - Includes organization management and user management.

- UI26 Organization management interface must support:
    - List all organizations with name, description, user count, item count.
    - Create new organization (name, description).
    - Edit existing organization (name, description).
    - Delete organization (with confirmation, cascade considerations).
    - View organization details including member list.

- UI27 User management interface must support:
    - List all users with name, identity, organization memberships.
    - Create new user (name, identity, password).
    - Edit user (name, identity, update password).
    - Delete user (with confirmation).
    - View user details.

- UI28 User-organization membership management must support:
    - Add user to organization with role selection (USER, ADMIN, OWNER).
    - Remove user from organization (with confirmation).
    - Update user's roles within an organization.
    - View user's roles across all their organizations.

- UI29 Role-based access control must be reflected in the UI:
    - USER: Can view and create items, locations, collections, tags.
    - ADMIN: All USER permissions plus ability to edit/delete all items,
    manage locations, collections, tags.
    - OWNER: All ADMIN permissions plus ability to manage organization
    settings and user memberships.
    - Certain UI elements should be hidden/disabled based on user's role.

### Audit and History

- UI30 An audit log viewer must be available for each item.
    - "View History" button in the item detail/edit view.
    - Shows chronological list of all changes to the item.
    - Each entry shows: date/time, type of change, summary of what changed.
    - May show user who made the change (if that's added to audit model).

- UI31 A system-wide audit log should be accessible to ADMIN and OWNER roles.
    - Shows recent changes across all items in the organization.
    - Filterable by: date range, item, user (if user tracking added).
    - Useful for compliance and troubleshooting.

### Error Handling and Feedback

- UI32 All user actions must provide clear feedback.
    - Success messages for create/update/delete operations.
    - Error messages with actionable information.
    - Loading indicators for async operations.

- UI33 Network errors and API failures must be handled gracefully.
    - Display user-friendly error messages.
    - Offer retry options where appropriate.
    - Don't expose raw error details to non-admin users.

- UI34 Session expiration must be handled elegantly.
    - When JWT expires (after 24 hours), redirect to login.
    - Preserve the current page/context if possible to return after re-auth.
    - Show warning message before expiration (e.g., at 23 hours).

### Responsive Design

- UI35 The UI must be responsive and work on different screen sizes.
    - Desktop: Full layout with side-by-side filters and results.
    - Tablet: Adjusted layout, possibly collapsible filters.
    - Mobile: Stacked layout, filters in expandable panel.

- UI36 Touch-friendly interface elements for tablet/mobile.
    - Adequate button sizes for touch targets.
    - Swipe gestures where appropriate (e.g., to delete items).

### Accessibility

- UI37 The UI must follow basic accessibility guidelines.
    - Proper semantic HTML.
    - ARIA labels where needed.
    - Keyboard navigation support.
    - Sufficient color contrast.
    - Screen reader compatibility.
