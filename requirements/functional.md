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
- The application will use Tailwind CSS for styling.

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
