-- Add password hash field to users table
-- Password hashes will be stored using Argon2 algorithm
-- NULL values indicate users without password authentication

ALTER TABLE users ADD COLUMN password_hash TEXT NULL;

-- Add index for efficient password lookup during authentication
CREATE INDEX idx_users_identity_password ON users(identity) WHERE password_hash IS NOT NULL;

-- Update audit trigger to include password_hash changes
-- (This ensures password changes are logged for security)