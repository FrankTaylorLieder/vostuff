-- Add 'dvd' to the item_type enum
ALTER TYPE item_type ADD VALUE 'dvd';

-- Create dvd_details table (similar to cd_details)
CREATE TABLE dvd_details (
    item_id UUID PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    disks INTEGER CHECK (disks > 0),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Add trigger for updated_at
CREATE TRIGGER update_dvd_details_updated_at
    BEFORE UPDATE ON dvd_details
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
