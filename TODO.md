# VOStuff TODO

## Existing backlog
- Authz - actually restrict operations based on org/role
- Compose for running app in production
- Maybe change auth to separate identity from access tokens. Changes the follow on flow... now we always verify identity. Then create authz tokens for the org we want to use.
- OIDC authn

---

## Soft fields migration

The schema and API backend are updated. The following work remains to complete
the migration end-to-end.

### ~~1. Fix broken web UI types (blocking — UI will not compile)~~ ✓ DONE

~~The web UI (`crates/vostuff-web`) still uses the old hard-coded type model.~~

Completed 2026-03-20. `server_fns/items.rs`, `components/items_table.rs`, and
`pages/home.rs` all updated to use `kind_id`/`kind_name`/`soft_fields`. The
kind filter dropdown is hardcoded to the 8 shared kinds for now (pending item 2).
Edit mode renders all soft fields as text inputs.

### 2. New server fn: fetch kinds for filter dropdown

Add a `get_kinds` server fn in the web UI that calls the kinds API (section 3
below) so the kind filter dropdown is populated from the database rather than
hard-coded.

### 3. New API: kinds management

New handler file `crates/vostuff-api/src/api/handlers/kinds.rs` with:

| Method | Path | Description |
|--------|------|-------------|
| `GET`  | `/organizations/:org_id/kinds` | List all kinds visible to the org (shared + org-owned), each with their ordered fields |
| `POST` | `/organizations/:org_id/kinds` | Create a new org-owned kind |
| `GET`  | `/organizations/:org_id/kinds/:kind_id` | Get a single kind with full field list |
| `PATCH`| `/organizations/:org_id/kinds/:kind_id` | Update kind: display_name, add/remove fields, reorder fields. Org kinds only; shared kinds cannot be mutated. |
| `DELETE`| `/organizations/:org_id/kinds/:kind_id` | Delete an org kind. Must check no items currently use this kind. |
| `POST` | `/organizations/:org_id/kinds/:kind_id/override` | Copy a shared kind into the org. Returns the new org kind. Shared fields are referenced; display_order is copied. |
| `POST` | `/organizations/:org_id/kinds/:kind_id/revert` | Delete the org-owned override of a shared kind and reassign items back to the shared kind. Warn caller of items that will lose fields (items endpoint already handles count). |

Response model for a kind should include:
```json
{
  "id": "...",
  "org_id": null,
  "name": "vinyl",
  "display_name": "Vinyl",
  "is_shared": true,
  "fields": [
    { "id": "...", "name": "size", "display_name": "Size", "field_type": "enum",
      "display_order": 1, "enum_values": [{"value": "12_inch", "display_value": "12\"", "sort_order": 1}, ...] }
  ]
}
```

### 4. New API: fields management

New handler file `crates/vostuff-api/src/api/handlers/fields.rs` with:

| Method | Path | Description |
|--------|------|-------------|
| `GET`  | `/organizations/:org_id/fields` | List all fields visible to the org (shared + org-owned) |
| `POST` | `/organizations/:org_id/fields` | Create a new org-owned field. Validate name doesn't conflict with any shared field name. |
| `GET`  | `/organizations/:org_id/fields/:field_id` | Get a single field with enum values (if applicable) |
| `PATCH`| `/organizations/:org_id/fields/:field_id` | Update `display_name`. For org fields only: add/remove/reorder enum values. Shared fields are read-only. |
| `DELETE`| `/organizations/:org_id/fields/:field_id` | Delete an org field. Must check no kinds reference it. |

For enum field management (on PATCH):
- Adding an enum value is always safe
- Removing an enum value must warn how many items currently hold that value
  and strip it from `soft_fields` on confirmation (handled server-side)

### 5. Register new routes

In `crates/vostuff-api/src/api/handlers/mod.rs`:
- Add `pub mod kinds;` and `pub mod fields;`
- Wire routes into `build_router`

In `crates/vostuff-api/src/bin/api_server.rs`:
- Add new models to the OpenAPI `components(schemas(...))` block

### 6. New UI: kinds and fields management page

New page accessible from the org settings area:

- **Kinds tab**: lists shared kinds (read-only) and org kinds (editable).
  Each kind shows its fields in order. Actions:
  - "Override" on a shared kind → copy it to org (calls override endpoint)
  - "Revert" on an org-overridden kind → revert to shared
  - "New kind" → create form (name, display_name, select fields to include)
  - Edit an org kind: change display_name, add/remove fields, drag to reorder
  - Delete an org kind (disabled if items use it)

- **Fields tab**: lists shared fields (read-only) and org fields (editable).
  Actions:
  - "New field" → form: name (immutable after creation), display_name, type;
    if type=enum: inline enum value editor (value + display_value, drag to reorder)
  - Edit org field: change display_name, manage enum values
  - Delete org field (disabled if referenced by any kind)

### 7. Update item create/edit UI for soft fields

The item create and edit forms need to be soft-field-aware:

- On create: user selects a kind; the form dynamically shows the soft fields
  for that kind with appropriate input controls per type:
  - `string`/`text` → text input / textarea
  - `number` → number input
  - `boolean` → checkbox
  - `date` → date picker
  - `datetime` → datetime picker
  - `enum` → select/dropdown with allowed values
- On edit: load the item's kind, fetch its field definitions, render current
  `soft_fields` values into the form

### 8. Fix item detail view for soft fields

The expanded item detail panel in the items table currently has hard-coded
sections for vinyl, CD, cassette, DVD details. Replace with a dynamic renderer:
- Fetch the kind's fields (ordered by `display_order`)
- For each field, display its `display_name` and the value from `soft_fields`
  (formatted appropriately per type: dates, booleans, enum display values)
- Show nothing for fields with no value in `soft_fields`

### 9. Update CLZ importer

`crates/vostuff-api/src/bin/clz_importer.rs` still uses the old `item_type`
enum and detail table inserts. Update to:
- Look up kind IDs by name at startup (vinyl, cd, etc.)
- Build `soft_fields` JSONB when creating items
- Remove all detail table inserts

### 10. Fix integration tests

The integration tests in `crates/vostuff-api/tests/` use the old schema:
- `tests/common/mod.rs` — test setup creates items with `item_type`
- `tests/items_tests.rs` — assertions reference `item_type`, detail tables
- `tests/multi_tenancy_tests.rs` — may reference item_type

Update all test fixtures and assertions to use `kind_id`/`kind_name` and
`soft_fields`.

### 11. Items count warning for field/kind removal

The design doc requires the UI to show how many items will be affected before
removing a field from a kind or deleting an enum value. Add an API endpoint:

`GET /organizations/:org_id/kinds/:kind_id/fields/:field_id/impact`

Returns `{ "item_count": N }` — the number of items of this kind that have a
non-null value for this field in `soft_fields`.

Used by the UI before confirming field removal from a kind, kind deletion,
and enum value removal.
