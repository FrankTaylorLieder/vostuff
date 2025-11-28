use anyhow::{anyhow, Result};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher as ArgonHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Password hashing utilities using Argon2
pub struct PasswordHasher;

impl PasswordHasher {
    /// Hash a password using Argon2 with a random salt
    pub fn hash_password(password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = <Argon2 as ArgonHasher>::hash_password(&argon2, password.as_bytes(), &salt)
            .map_err(|e| anyhow!("Failed to hash password: {}", e))?;
        Ok(password_hash.to_string())
    }

    /// Verify a password against a stored hash
    pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| anyhow!("Failed to parse password hash: {}", e))?;
        let argon2 = Argon2::default();
        Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }
}

/// JWT token claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,         // Subject (user ID)
    pub identity: String,  // User identity (email)
    pub organizations: Vec<Uuid>, // Organizations user belongs to
    pub iat: i64,         // Issued at
    pub exp: i64,         // Expiration time
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

    /// Generate a JWT token for a user
    pub fn generate_token(
        &self,
        user_id: Uuid,
        identity: String,
        organizations: Vec<Uuid>,
        expires_in_hours: i64,
    ) -> Result<String> {
        let now = Utc::now();
        let exp = now + Duration::hours(expires_in_hours);

        let claims = Claims {
            sub: user_id,
            identity,
            organizations,
            iat: now.timestamp(),
            exp: exp.timestamp(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| anyhow!("Failed to generate token: {}", e))
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
    pub organizations: Vec<Uuid>,
    pub is_authenticated: bool,
}

impl AuthContext {
    /// Create an unauthenticated context
    pub fn unauthenticated() -> Self {
        Self {
            user_id: Uuid::nil(),
            identity: String::new(),
            organizations: Vec::new(),
            is_authenticated: false,
        }
    }

    /// Create an authenticated context from JWT claims
    pub fn from_claims(claims: Claims) -> Self {
        Self {
            user_id: claims.sub,
            identity: claims.identity,
            organizations: claims.organizations,
            is_authenticated: true,
        }
    }

    /// Check if user belongs to a specific organization
    pub fn has_org_access(&self, org_id: Uuid) -> bool {
        self.is_authenticated && self.organizations.contains(&org_id)
    }

    /// Check if user is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.is_authenticated
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
        let organizations = vec![Uuid::new_v4()];

        // Generate token
        let token = manager.generate_token(user_id, identity.clone(), organizations.clone(), 24).unwrap();

        // Validate token
        let claims = manager.validate_token(&token).unwrap();
        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.identity, identity);
        assert_eq!(claims.organizations, organizations);
    }

    #[test]
    fn test_auth_context() {
        let org_id = Uuid::new_v4();
        let other_org_id = Uuid::new_v4();

        let context = AuthContext {
            user_id: Uuid::new_v4(),
            identity: "test@example.com".to_string(),
            organizations: vec![org_id],
            is_authenticated: true,
        };

        assert!(context.has_org_access(org_id));
        assert!(!context.has_org_access(other_org_id));
        assert!(context.is_authenticated());
    }
}