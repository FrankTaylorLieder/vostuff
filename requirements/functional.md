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
    - Collections: reference to one or more collection that this item belongs
    to
    - Tags: List of zero or more tags associated with this item
    - Orgnisation: ID of the Organisation to which this item belongs
- D2 Each item has a type.
- D3 The following item types are to be implemented, some with additional
related data fields (only vinyl currently has additional fields):
    - Vinyl
        - Size, one of: 12 inch, 6 inch, other
        - Speed, one of: 33, 45, other
        - Channels, one of: mono, stereo, surround, other
        - Disks, non-zero integer
        - Media grading, one of: Mint, Near mint, Excellent, Good, Fair, Poor
        - Sleeve grading, one of: Mint, Near mint, Excellent, Good, Fair, Poor
    - CD
    - Cassette
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
        - Details of change

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
- A6 Organisation and user data is owned by the SYSTEM orgnisation and only be
operated upon in sessions with at organisation.
- A7 The API needs to support an OIDC authentication flow.
- A8 When creating a session token (login), the API needs to support a flow
where the user can select from their available orgnisations and then resubmit
the identity token and orgnisation ID to get their session (authorisation)
token.

## UI

(To be defined later.)
