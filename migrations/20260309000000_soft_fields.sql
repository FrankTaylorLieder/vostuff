-- Soft fields migration
--
-- Replaces hard-coded item_type enum and per-type detail tables with a
-- flexible kinds + fields system. Soft field values are stored as JSONB
-- on the items table.

-- ============================================================
-- New enum
-- ============================================================

CREATE TYPE field_type AS ENUM ('string', 'text', 'date', 'datetime', 'number', 'enum', 'boolean');

-- ============================================================
-- New tables
-- ============================================================

CREATE TABLE kinds (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id       UUID REFERENCES organizations(id) ON DELETE CASCADE,  -- NULL = shared
    name         VARCHAR(255) NOT NULL,    -- immutable internal key
    display_name VARCHAR(255),             -- mutable UI label
    created_at   TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at   TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Uniqueness across shared kinds and within each org (NULL != NULL in UNIQUE)
CREATE UNIQUE INDEX idx_kinds_shared_name ON kinds(name) WHERE org_id IS NULL;
CREATE UNIQUE INDEX idx_kinds_org_name    ON kinds(org_id, name) WHERE org_id IS NOT NULL;

CREATE TABLE fields (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id       UUID REFERENCES organizations(id) ON DELETE CASCADE,  -- NULL = shared
    name         VARCHAR(255) NOT NULL,    -- immutable, used as JSONB key
    display_name VARCHAR(255),             -- mutable UI label
    field_type   field_type NOT NULL,
    created_at   TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at   TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE UNIQUE INDEX idx_fields_shared_name ON fields(name) WHERE org_id IS NULL;
CREATE UNIQUE INDEX idx_fields_org_name    ON fields(org_id, name) WHERE org_id IS NOT NULL;
-- Note: uniqueness of org field names against shared field names is enforced at the API layer

CREATE TABLE enum_values (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    field_id      UUID NOT NULL REFERENCES fields(id) ON DELETE CASCADE,
    value         VARCHAR(255) NOT NULL,   -- stored value used in JSONB and API
    display_value VARCHAR(255),            -- optional UI label
    sort_order    INTEGER NOT NULL DEFAULT 0,
    UNIQUE(field_id, value)
);

CREATE TABLE kind_fields (
    kind_id       UUID NOT NULL REFERENCES kinds(id) ON DELETE CASCADE,
    field_id      UUID NOT NULL REFERENCES fields(id) ON DELETE CASCADE,
    display_order INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (kind_id, field_id)
);

-- ============================================================
-- Updated_at triggers for new tables
-- ============================================================

CREATE TRIGGER update_kinds_updated_at
    BEFORE UPDATE ON kinds
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_fields_updated_at
    BEFORE UPDATE ON fields
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ============================================================
-- Add new columns to items (nullable initially for migration)
-- ============================================================

ALTER TABLE items ADD COLUMN kind_id     UUID REFERENCES kinds(id);
ALTER TABLE items ADD COLUMN soft_fields JSONB NOT NULL DEFAULT '{}';

-- ============================================================
-- Seed: shared kinds (fixed UUIDs for reference during migration)
-- ============================================================

INSERT INTO kinds (id, org_id, name, display_name) VALUES
    ('00000000-0000-0000-0000-000000000001', NULL, 'vinyl',       'Vinyl'),
    ('00000000-0000-0000-0000-000000000002', NULL, 'cd',          'CD'),
    ('00000000-0000-0000-0000-000000000003', NULL, 'cassette',    'Cassette'),
    ('00000000-0000-0000-0000-000000000004', NULL, 'book',        'Book'),
    ('00000000-0000-0000-0000-000000000005', NULL, 'score',       'Score'),
    ('00000000-0000-0000-0000-000000000006', NULL, 'electronics', 'Electronics'),
    ('00000000-0000-0000-0000-000000000007', NULL, 'misc',        'Misc'),
    ('00000000-0000-0000-0000-000000000008', NULL, 'dvd',         'DVD');

-- ============================================================
-- Seed: shared fields (fixed UUIDs for reference during migration)
-- ============================================================

INSERT INTO fields (id, org_id, name, display_name, field_type) VALUES
    ('00000000-0000-0000-0001-000000000001', NULL, 'size',           'Size',           'enum'),
    ('00000000-0000-0000-0001-000000000002', NULL, 'speed',          'Speed',          'enum'),
    ('00000000-0000-0000-0001-000000000003', NULL, 'channels',       'Channels',       'enum'),
    ('00000000-0000-0000-0001-000000000004', NULL, 'media_grading',  'Media Grading',  'enum'),
    ('00000000-0000-0000-0001-000000000005', NULL, 'sleeve_grading', 'Sleeve Grading', 'enum'),
    ('00000000-0000-0000-0001-000000000006', NULL, 'disks',          'Disks',          'number'),
    ('00000000-0000-0000-0001-000000000007', NULL, 'cassettes',      'Cassettes',      'number');

-- ============================================================
-- Seed: enum values for shared fields
-- ============================================================

-- size (vinyl)
INSERT INTO enum_values (field_id, value, display_value, sort_order) VALUES
    ('00000000-0000-0000-0001-000000000001', '12_inch', '12"',   1),
    ('00000000-0000-0000-0001-000000000001', '6_inch',  '6"',    2),
    ('00000000-0000-0000-0001-000000000001', 'other',   'Other', 3);

-- speed (vinyl)
INSERT INTO enum_values (field_id, value, display_value, sort_order) VALUES
    ('00000000-0000-0000-0001-000000000002', '33',    '33 RPM', 1),
    ('00000000-0000-0000-0001-000000000002', '45',    '45 RPM', 2),
    ('00000000-0000-0000-0001-000000000002', 'other', 'Other',  3);

-- channels (vinyl)
INSERT INTO enum_values (field_id, value, display_value, sort_order) VALUES
    ('00000000-0000-0000-0001-000000000003', 'mono',    'Mono',    1),
    ('00000000-0000-0000-0001-000000000003', 'stereo',  'Stereo',  2),
    ('00000000-0000-0000-0001-000000000003', 'surround','Surround',3),
    ('00000000-0000-0000-0001-000000000003', 'other',   'Other',   4);

-- media_grading (vinyl) -- separate field from sleeve_grading
INSERT INTO enum_values (field_id, value, display_value, sort_order) VALUES
    ('00000000-0000-0000-0001-000000000004', 'mint',      'Mint',      1),
    ('00000000-0000-0000-0001-000000000004', 'near_mint', 'Near Mint', 2),
    ('00000000-0000-0000-0001-000000000004', 'excellent', 'Excellent', 3),
    ('00000000-0000-0000-0001-000000000004', 'good',      'Good',      4),
    ('00000000-0000-0000-0001-000000000004', 'fair',      'Fair',      5),
    ('00000000-0000-0000-0001-000000000004', 'poor',      'Poor',      6);

-- sleeve_grading (vinyl)
INSERT INTO enum_values (field_id, value, display_value, sort_order) VALUES
    ('00000000-0000-0000-0001-000000000005', 'mint',      'Mint',      1),
    ('00000000-0000-0000-0001-000000000005', 'near_mint', 'Near Mint', 2),
    ('00000000-0000-0000-0001-000000000005', 'excellent', 'Excellent', 3),
    ('00000000-0000-0000-0001-000000000005', 'good',      'Good',      4),
    ('00000000-0000-0000-0001-000000000005', 'fair',      'Fair',      5),
    ('00000000-0000-0000-0001-000000000005', 'poor',      'Poor',      6);

-- ============================================================
-- Seed: kind_fields (which fields belong to which shared kind)
-- ============================================================

INSERT INTO kind_fields (kind_id, field_id, display_order) VALUES
    -- vinyl
    ('00000000-0000-0000-0000-000000000001', '00000000-0000-0000-0001-000000000001', 1),  -- size
    ('00000000-0000-0000-0000-000000000001', '00000000-0000-0000-0001-000000000002', 2),  -- speed
    ('00000000-0000-0000-0000-000000000001', '00000000-0000-0000-0001-000000000003', 3),  -- channels
    ('00000000-0000-0000-0000-000000000001', '00000000-0000-0000-0001-000000000006', 4),  -- disks
    ('00000000-0000-0000-0000-000000000001', '00000000-0000-0000-0001-000000000004', 5),  -- media_grading
    ('00000000-0000-0000-0000-000000000001', '00000000-0000-0000-0001-000000000005', 6),  -- sleeve_grading
    -- cd
    ('00000000-0000-0000-0000-000000000002', '00000000-0000-0000-0001-000000000006', 1),  -- disks
    -- cassette
    ('00000000-0000-0000-0000-000000000003', '00000000-0000-0000-0001-000000000007', 1),  -- cassettes
    -- dvd
    ('00000000-0000-0000-0000-000000000008', '00000000-0000-0000-0001-000000000006', 1);  -- disks
    -- book, score, electronics, misc have no soft fields initially

-- ============================================================
-- Data migration: assign kind_id from old item_type
-- ============================================================

UPDATE items SET kind_id = '00000000-0000-0000-0000-000000000001' WHERE item_type = 'vinyl';
UPDATE items SET kind_id = '00000000-0000-0000-0000-000000000002' WHERE item_type = 'cd';
UPDATE items SET kind_id = '00000000-0000-0000-0000-000000000003' WHERE item_type = 'cassette';
UPDATE items SET kind_id = '00000000-0000-0000-0000-000000000004' WHERE item_type = 'book';
UPDATE items SET kind_id = '00000000-0000-0000-0000-000000000005' WHERE item_type = 'score';
UPDATE items SET kind_id = '00000000-0000-0000-0000-000000000006' WHERE item_type = 'electronics';
UPDATE items SET kind_id = '00000000-0000-0000-0000-000000000007' WHERE item_type = 'misc';
UPDATE items SET kind_id = '00000000-0000-0000-0000-000000000008' WHERE item_type = 'dvd';

-- ============================================================
-- Data migration: migrate detail table data into soft_fields JSONB
-- ============================================================

-- vinyl_details
UPDATE items
SET soft_fields = jsonb_strip_nulls(jsonb_build_object(
    'size',           vd.size::text,
    'speed',          vd.speed::text,
    'channels',       vd.channels::text,
    'disks',          vd.disks,
    'media_grading',  vd.media_grading::text,
    'sleeve_grading', vd.sleeve_grading::text
))
FROM vinyl_details vd
WHERE items.id = vd.item_id;

-- cd_details
UPDATE items
SET soft_fields = jsonb_strip_nulls(jsonb_build_object(
    'disks', cd.disks
))
FROM cd_details cd
WHERE items.id = cd.item_id;

-- dvd_details
UPDATE items
SET soft_fields = jsonb_strip_nulls(jsonb_build_object(
    'disks', dvd.disks
))
FROM dvd_details dvd
WHERE items.id = dvd.item_id;

-- cassette_details
UPDATE items
SET soft_fields = jsonb_strip_nulls(jsonb_build_object(
    'cassettes', cs.cassettes
))
FROM cassette_details cs
WHERE items.id = cs.item_id;

-- ============================================================
-- Enforce kind_id NOT NULL now that all rows are assigned
-- ============================================================

ALTER TABLE items ALTER COLUMN kind_id SET NOT NULL;

-- ============================================================
-- Drop old detail tables and enums
-- ============================================================

DROP TABLE vinyl_details;
DROP TABLE cd_details;
DROP TABLE dvd_details;
DROP TABLE cassette_details;

ALTER TABLE items DROP COLUMN item_type;

DROP TYPE vinyl_size;
DROP TYPE vinyl_speed;
DROP TYPE vinyl_channels;
DROP TYPE grading;
DROP TYPE item_type;

-- ============================================================
-- Update indexes
-- ============================================================

-- idx_items_type is automatically dropped by PostgreSQL when item_type column is dropped above

CREATE INDEX idx_items_kind_id     ON items(kind_id);
CREATE INDEX idx_items_soft_fields ON items USING GIN(soft_fields);
