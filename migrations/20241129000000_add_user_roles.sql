-- Add user roles support
-- Users can have multiple roles: USER (default), ADMIN

-- Add roles column to users table
ALTER TABLE users
ADD COLUMN roles TEXT[] NOT NULL DEFAULT '{USER}';

-- Create index for role lookups
CREATE INDEX idx_users_roles ON users USING GIN (roles);

-- Add check constraint to ensure valid roles
ALTER TABLE users
ADD CONSTRAINT valid_roles CHECK (
    roles <@ ARRAY['USER', 'ADMIN']::TEXT[]
);

-- Update existing users to have USER role
UPDATE users SET roles = '{USER}' WHERE roles IS NULL OR roles = '{}';
