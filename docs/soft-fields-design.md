# Soft fields

Goal: Move away from pre-defined fixed types of object in the database. Instead
have a soft configurable fields which can be combined into a set of default and
org defined types.

## User experience

On day 1, an org will see the default shared kinds (the same as the fixed types
we have today). They can create records against these kinds just as they can
today. These shared kinds and the default fields are read only.

On day 2, a user can define new fields, which will belong to that org. They
cannot override or change the default fields. Fields have a unique name with
the scope of that org (i.e. the fields defined in that org union with the
shared fields). Fields have an optional display name which is used in the UI.

A user can define their own kinds which consist of a set of required fields and
then any grouping of additional fields from either the shared fields or
org-defined fields.

Further a user can change the non-required fields of a shared kind. At this
point the shared kind is copied into org and acts like it was created there.

When a user adds a new field to a kind all their items of that kind will have
that field, but it will be blank.

When a user removes a field from a kind the field will be removed from all
existing items of that kind with data in that field. In the UI a warning will
show how many items are impacted by this change before it is accepted.

The following field types will be supported:

- String
- Text (string interpretted as Markdown)
- Date
- Datetime
- Number
- Enum: type + list of string values (each value has an optional Display String)
  - E.g. Grading: mint ("Mint"), near-mint ("Near mint"), etc
- Boolean

The user visible required fields of a type are:

- id: UUID - read only
- kind: the kind of the item.
- name: String
- state: ItemState
- description: Option<Text>
- notes: Option<Text>
- location_id: Option<location UUID>
- date_entered: DateTime
- date_acquired: Option<DateTime>

There is a page to manage the fields and kinds. Showing both the shared and
org-owned fields and kinds and allowing the user to add/edit new fields and
override/add/edit kinds.

The order of fields in a kind can be defined by the user. 

## Implementation notes

We'll keep the required fields of an item as the existing Item table.

The kind of an item will be stored as a UUID foreign key to a new kinds table.
Kinds will have a org_id. Shared kinds have a NIL org_id, org-owned kinds have
their org_id.

These soft kinds are defining the additional fields, replacing the hard coded
VinylDetails, CdDetails etc tables. Any enums in the required fields are
implemented natively. All the remaining enums (used in the extension kinds) are
implement as soft enums (as described above).

Shared kinds and fields are defined with a NULL org_id.

When a user customizes are shared kind, the kind is copied into the org. It
does not have a parent reference and any later changes to the shared version
are not reflected into the org's version of the kind. The user can revert to
the original shared kind, but any additional fields added by the user will be
lost. This is the same experience has when the user removes a field from any of
their kinds.

All the existing details tables (relating to item kinds) must be migrated into
soft kinds and ffields. These will be seeded into the DB as part of migration.

The existing LoadDetails, MissingDetails and DisposedDetails are to be kept as
natively implemented tables are they are fixed.

All soft fields are optional.

The API should include soft fields in the returned data for an item. Similarly
update APIs shouuld enable updates for soft fields too.

Values for all fields, include soft enum fields need to be validated by the API
and rejected if they do not match the required type or defined enum values.

Soft field values are stored as a JSONB column on items. The unique name of the
field is used in the JSON. Field unique names cannot be renamed. The user can
change the display name of a field at any time.

We will remove the existing item_type and details tables during implementation
of this feature. All tools (e.g. CLZ importer and sample data creator) need to
be updated to use the new soft fields and kinds model instead of the old
hard-coded types.


