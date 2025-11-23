-- Initial schema for VOStuff application
-- Multi-tenant stuff tracking system

-- Organizations table (base tenant isolation)
CREATE TABLE organizations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create SYSTEM organization
INSERT INTO organizations (id, name, description) 
VALUES ('00000000-0000-0000-0000-000000000000', 'SYSTEM', 'System organization for administrative data');

-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    identity VARCHAR(255) NOT NULL UNIQUE, -- email from OIDC
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- User organization memberships (many-to-many)
CREATE TABLE user_organizations (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    PRIMARY KEY (user_id, organization_id)
);

-- Locations within organizations
CREATE TABLE locations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(organization_id, name)
);

-- Collections within organizations
CREATE TABLE collections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    notes TEXT, -- Markdown formatted
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Tags within organizations
CREATE TABLE tags (
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    PRIMARY KEY (organization_id, name)
);

-- Item types enum
CREATE TYPE item_type AS ENUM (
    'vinyl', 'cd', 'cassette', 'book', 'score', 'electronics', 'misc'
);

-- Item states enum
CREATE TYPE item_state AS ENUM (
    'current', 'loaned', 'missing', 'disposed'
);

-- Vinyl specific enums
CREATE TYPE vinyl_size AS ENUM ('12_inch', '6_inch', 'other');
CREATE TYPE vinyl_speed AS ENUM ('33', '45', 'other');
CREATE TYPE vinyl_channels AS ENUM ('mono', 'stereo', 'surround', 'other');
CREATE TYPE grading AS ENUM ('mint', 'near_mint', 'excellent', 'good', 'fair', 'poor');

-- Main items table
CREATE TABLE items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    item_type item_type NOT NULL,
    state item_state NOT NULL DEFAULT 'current',
    name VARCHAR(255) NOT NULL,
    description TEXT,
    notes TEXT, -- Markdown formatted
    location_id UUID REFERENCES locations(id),
    date_entered TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    date_acquired DATE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Item state details for LOANED state
CREATE TABLE item_loan_details (
    item_id UUID PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    date_loaned DATE NOT NULL,
    date_due_back DATE,
    loaned_to VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Item state details for MISSING state
CREATE TABLE item_missing_details (
    item_id UUID PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    date_missing DATE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Item state details for DISPOSED state
CREATE TABLE item_disposed_details (
    item_id UUID PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    date_disposed DATE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Type-specific data tables

-- Vinyl records
CREATE TABLE vinyl_details (
    item_id UUID PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    size vinyl_size,
    speed vinyl_speed,
    channels vinyl_channels,
    disks INTEGER CHECK (disks > 0),
    media_grading grading,
    sleeve_grading grading,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- CDs
CREATE TABLE cd_details (
    item_id UUID PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    disks INTEGER CHECK (disks > 0),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Cassettes
CREATE TABLE cassette_details (
    item_id UUID PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    cassettes INTEGER CHECK (cassettes > 0),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Many-to-many relationships

-- Items to collections
CREATE TABLE item_collections (
    item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    collection_id UUID NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    PRIMARY KEY (item_id, collection_id)
);

-- Items to tags
CREATE TABLE item_tags (
    item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL,
    tag_name VARCHAR(100) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    PRIMARY KEY (item_id, organization_id, tag_name),
    FOREIGN KEY (organization_id, tag_name) REFERENCES tags(organization_id, name) ON DELETE CASCADE
);

-- Audit trail
CREATE TABLE audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_id UUID NOT NULL, -- Not FK to allow auditing deleted items
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    change_date TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    change_details TEXT NOT NULL
);

-- Indexes for performance
CREATE INDEX idx_items_organization_id ON items(organization_id);
CREATE INDEX idx_items_type ON items(item_type);
CREATE INDEX idx_items_state ON items(state);
CREATE INDEX idx_items_name ON items(name);
CREATE INDEX idx_items_date_acquired ON items(date_acquired);
CREATE INDEX idx_locations_organization_id ON locations(organization_id);
CREATE INDEX idx_collections_organization_id ON collections(organization_id);
CREATE INDEX idx_collections_name ON collections(name);
CREATE INDEX idx_audit_log_item_id ON audit_log(item_id);
CREATE INDEX idx_audit_log_organization_id ON audit_log(organization_id);
CREATE INDEX idx_audit_log_change_date ON audit_log(change_date);

-- Triggers for updated_at timestamps
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_organizations_updated_at BEFORE UPDATE ON organizations FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_locations_updated_at BEFORE UPDATE ON locations FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_collections_updated_at BEFORE UPDATE ON collections FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_items_updated_at BEFORE UPDATE ON items FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_vinyl_details_updated_at BEFORE UPDATE ON vinyl_details FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_cd_details_updated_at BEFORE UPDATE ON cd_details FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_cassette_details_updated_at BEFORE UPDATE ON cassette_details FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_item_loan_details_updated_at BEFORE UPDATE ON item_loan_details FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_item_missing_details_updated_at BEFORE UPDATE ON item_missing_details FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_item_disposed_details_updated_at BEFORE UPDATE ON item_disposed_details FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();