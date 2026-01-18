use anyhow::{Result, anyhow};
use argon2::{
    Argon2,
    password_hash::{
        PasswordHash, PasswordHasher as ArgonHasher, PasswordVerifier, SaltString, rand_core::OsRng,
    },
};
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Password hashing utilities using Argon2
pub struct PasswordHasher;

impl PasswordHasher {
    /// Hash a password using Argon2 with a random salt
    pub fn hash_password(password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash =
            <Argon2 as ArgonHasher>::hash_password(&argon2, password.as_bytes(), &salt)
                .map_err(|e| anyhow!("Failed to hash password: {}", e))?;
        Ok(password_hash.to_string())
    }

    /// Verify a password against a stored hash
    pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
        let parsed_hash =
            PasswordHash::new(hash).map_err(|e| anyhow!("Failed to parse password hash: {}", e))?;
        let argon2 = Argon2::default();
        Ok(argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}

/// JWT token claims for authenticated users
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,             // Subject (user ID)
    pub identity: String,      // User identity (email)
    pub organization_id: Uuid, // Selected organization
    pub roles: Vec<String>,    // User roles in this organization
    pub iat: i64,              // Issued at
    pub exp: i64,              // Expiration time
}

/// Follow-on token claims for org selection (short-lived)
#[derive(Debug, Serialize, Deserialize)]
pub struct FollowOnClaims {
    pub sub: Uuid,        // Subject (user ID)
    pub identity: String, // User identity (email)
    pub iat: i64,         // Issued at
    pub exp: i64,         // Expiration time (5 minutes)
}

/// JWT token manager
pub struct TokenManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
}

impl TokenManager {
    /// Create a new token manager with the given secret
    pub fn new(secret: &str) -> Self {
        let encoding_key = EncodingKey::from_secret(secret.as_ref());
        let decoding_key = DecodingKey::from_secret(secret.as_ref());

        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_required_spec_claims(&["exp", "sub", "iat"]);

        Self {
            encoding_key,
            decoding_key,
            validation,
        }
    }

    /// Generate a JWT token for a user with selected organization
    pub fn generate_token(
        &self,
        user_id: Uuid,
        identity: String,
        organization_id: Uuid,
        roles: Vec<String>,
        expires_in_hours: i64,
    ) -> Result<String> {
        let now = Utc::now();
        let exp = now + Duration::hours(expires_in_hours);

        let claims = Claims {
            sub: user_id,
            identity,
            organization_id,
            roles,
            iat: now.timestamp(),
            exp: exp.timestamp(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| anyhow!("Failed to generate token: {}", e))
    }

    /// Generate a follow-on token for org selection (5 minute expiry)
    pub fn generate_follow_on_token(&self, user_id: Uuid, identity: String) -> Result<String> {
        let now = Utc::now();
        let exp = now + Duration::minutes(5);

        let claims = FollowOnClaims {
            sub: user_id,
            identity,
            iat: now.timestamp(),
            exp: exp.timestamp(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| anyhow!("Failed to generate follow-on token: {}", e))
    }

    /// Validate a follow-on token
    pub fn validate_follow_on_token(&self, token: &str) -> Result<FollowOnClaims> {
        let token_data = decode::<FollowOnClaims>(token, &self.decoding_key, &self.validation)
            .map_err(|e| anyhow!("Failed to validate follow-on token: {}", e))?;

        Ok(token_data.claims)
    }

    /// Validate and decode a JWT token
    pub fn validate_token(&self, token: &str) -> Result<Claims> {
        let token_data = decode::<Claims>(token, &self.decoding_key, &self.validation)
            .map_err(|e| anyhow!("Failed to validate token: {}", e))?;

        Ok(token_data.claims)
    }
}

/// Authentication context passed to handlers
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: Uuid,
    pub identity: String,
    pub organization_id: Uuid,
    pub roles: Vec<String>,
    pub is_authenticated: bool,
}

impl AuthContext {
    /// Create an unauthenticated context
    pub fn unauthenticated() -> Self {
        Self {
            user_id: Uuid::nil(),
            identity: String::new(),
            organization_id: Uuid::nil(),
            roles: Vec::new(),
            is_authenticated: false,
        }
    }

    /// Create an authenticated context from JWT claims
    pub fn from_claims(claims: Claims) -> Self {
        Self {
            user_id: claims.sub,
            identity: claims.identity,
            organization_id: claims.organization_id,
            roles: claims.roles,
            is_authenticated: true,
        }
    }

    /// Check if user belongs to a specific organization (matches their selected org)
    pub fn has_org_access(&self, org_id: Uuid) -> bool {
        self.is_authenticated && self.organization_id == org_id
    }

    /// Get the user's selected organization ID
    pub fn organization_id(&self) -> Uuid {
        self.organization_id
    }

    /// Check if user is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.is_authenticated
    }

    /// Check if user has a specific role
    pub fn has_role(&self, role: &str) -> bool {
        self.is_authenticated && self.roles.contains(&role.to_string())
    }

    /// Check if user is admin
    pub fn is_admin(&self) -> bool {
        self.has_role("ADMIN")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "test_password_123";
        let hash = PasswordHasher::hash_password(password).unwrap();

        // Should be able to verify correct password
        assert!(PasswordHasher::verify_password(password, &hash).unwrap());

        // Should reject incorrect password
        assert!(!PasswordHasher::verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_jwt_token() {
        let manager = TokenManager::new("test_secret_key_for_testing");
        let user_id = Uuid::new_v4();
        let identity = "test@example.com".to_string();
        let org_id = Uuid::new_v4();
        let roles = vec!["USER".to_string(), "ADMIN".to_string()];

        // Generate token
        let token = manager
            .generate_token(user_id, identity.clone(), org_id, roles.clone(), 24)
            .unwrap();

        // Validate token
        let claims = manager.validate_token(&token).unwrap();
        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.identity, identity);
        assert_eq!(claims.organization_id, org_id);
        assert_eq!(claims.roles, roles);
    }

    #[test]
    fn test_follow_on_token() {
        let manager = TokenManager::new("test_secret_key_for_testing");
        let user_id = Uuid::new_v4();
        let identity = "test@example.com".to_string();

        // Generate follow-on token
        let token = manager
            .generate_follow_on_token(user_id, identity.clone())
            .unwrap();

        // Validate token
        let claims = manager.validate_follow_on_token(&token).unwrap();
        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.identity, identity);
    }

    #[test]
    fn test_auth_context() {
        let org_id = Uuid::new_v4();
        let other_org_id = Uuid::new_v4();

        let context = AuthContext {
            user_id: Uuid::new_v4(),
            identity: "test@example.com".to_string(),
            organization_id: org_id,
            roles: vec!["USER".to_string(), "ADMIN".to_string()],
            is_authenticated: true,
        };

        assert!(context.has_org_access(org_id));
        assert!(!context.has_org_access(other_org_id));
        assert_eq!(context.organization_id(), org_id);
        assert!(context.is_authenticated());
        assert!(context.has_role("USER"));
        assert!(context.has_role("ADMIN"));
        assert!(!context.has_role("SUPERUSER"));
        assert!(context.is_admin());
    }
}
