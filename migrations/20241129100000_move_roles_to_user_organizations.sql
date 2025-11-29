-- Move roles from users to user_organizations table
-- This makes roles organization-specific rather than global

-- Add roles column to user_organizations
ALTER TABLE user_organizations
ADD COLUMN roles TEXT[] NOT NULL DEFAULT '{USER}';

-- Create index for role lookups
CREATE INDEX idx_user_organizations_roles ON user_organizations USING GIN (roles);

-- Add check constraint to ensure valid roles
ALTER TABLE user_organizations
ADD CONSTRAINT valid_user_org_roles CHECK (
    roles <@ ARRAY['USER', 'ADMIN']::TEXT[]
);

-- Migrate existing role data from users to user_organizations
-- For each user, copy their roles to all their organization memberships
UPDATE user_organizations uo
SET roles = u.roles
FROM users u
WHERE uo.user_id = u.id;

-- Remove roles column from users table
ALTER TABLE users
DROP COLUMN roles;

-- Drop the old constraint and index from users
-- (They were already removed with the column drop, but documenting intent)
