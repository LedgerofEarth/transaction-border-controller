//! Admin Authentication
//!
//! Implements Ed25519 public key authentication for admin access.
//!
//! Security Model:
//! - No passwords, only public key auth
//! - Requests must be signed with admin private key
//! - Timestamp prevents replay attacks
//! - All auth attempts are logged

use anyhow::{anyhow, Result};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use hex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

/// Maximum age of a signed request (5 minutes)
const MAX_REQUEST_AGE_SECS: u64 = 300;

/// Admin role levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdminRole {
    /// Full access - can modify config, view all data
    SuperAdmin,
    /// Can view status and logs, cannot modify
    Operator,
    /// Read-only access to health and metrics
    Monitor,
}

/// Registered admin with their public key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminEntry {
    pub name: String,
    pub public_key: String,  // hex-encoded Ed25519 public key
    pub role: AdminRole,
    pub created_at: u64,
    pub last_seen: Option<u64>,
}

/// Signed admin request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedRequest {
    /// The admin's public key (hex)
    pub public_key: String,
    
    /// Unix timestamp when request was signed
    pub timestamp: u64,
    
    /// The command/action being requested
    pub command: String,
    
    /// Command arguments (JSON)
    pub args: serde_json::Value,
    
    /// Ed25519 signature of: timestamp + command + args
    pub signature: String,
}

/// In-memory admin key store
pub struct AdminKeyStore {
    admins: RwLock<HashMap<String, AdminEntry>>,
    used_nonces: RwLock<Vec<(String, u64)>>,  // (pubkey, timestamp) for replay prevention
}

impl AdminKeyStore {
    pub fn new() -> Self {
        Self {
            admins: RwLock::new(HashMap::new()),
            used_nonces: RwLock::new(Vec::new()),
        }
    }

    /// Load admins from environment or config
    pub fn load_from_env(&self) -> Result<()> {
        // Load admin keys from TBC_ADMIN_KEYS env var
        // Format: "name1:pubkey1:role1,name2:pubkey2:role2"
        if let Ok(keys) = std::env::var("TBC_ADMIN_KEYS") {
            let mut admins = self.admins.write().map_err(|_| anyhow!("Lock poisoned"))?;
            
            for entry in keys.split(',') {
                let parts: Vec<&str> = entry.trim().split(':').collect();
                if parts.len() >= 2 {
                    let name = parts[0].to_string();
                    let pubkey = parts[1].to_string();
                    let role = match parts.get(2).map(|s| *s) {
                        Some("super") => AdminRole::SuperAdmin,
                        Some("operator") => AdminRole::Operator,
                        _ => AdminRole::Monitor,
                    };
                    
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    
                    admins.insert(pubkey.clone(), AdminEntry {
                        name,
                        public_key: pubkey,
                        role,
                        created_at: now,
                        last_seen: None,
                    });
                }
            }
            
            tracing::info!("Loaded {} admin keys", admins.len());
        }
        
        Ok(())
    }

    /// Register a new admin (SuperAdmin only)
    pub fn register_admin(&self, entry: AdminEntry) -> Result<()> {
        let mut admins = self.admins.write().map_err(|_| anyhow!("Lock poisoned"))?;
        admins.insert(entry.public_key.clone(), entry);
        Ok(())
    }

    /// Get admin by public key
    pub fn get_admin(&self, pubkey: &str) -> Option<AdminEntry> {
        let admins = self.admins.read().ok()?;
        admins.get(pubkey).cloned()
    }

    /// List all admins
    pub fn list_admins(&self) -> Vec<AdminEntry> {
        if let Ok(admins) = self.admins.read() {
            admins.values().cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// Update last seen time
    pub fn update_last_seen(&self, pubkey: &str) {
        if let Ok(mut admins) = self.admins.write() {
            if let Some(admin) = admins.get_mut(pubkey) {
                admin.last_seen = Some(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                );
            }
        }
    }

    /// Check if request timestamp was already used (replay prevention)
    fn check_replay(&self, pubkey: &str, timestamp: u64) -> Result<()> {
        let mut nonces = self.used_nonces.write().map_err(|_| anyhow!("Lock poisoned"))?;
        
        // Clean old nonces (older than MAX_REQUEST_AGE_SECS)
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        nonces.retain(|(_, ts)| now - ts < MAX_REQUEST_AGE_SECS * 2);
        
        // Check if this (pubkey, timestamp) combo was used
        let key = (pubkey.to_string(), timestamp);
        if nonces.contains(&key) {
            return Err(anyhow!("Replay detected: timestamp already used"));
        }
        
        // Add to used nonces
        nonces.push(key);
        Ok(())
    }
}

/// Admin authentication handler
pub struct AdminAuth {
    key_store: AdminKeyStore,
}

impl AdminAuth {
    pub fn new() -> Self {
        let key_store = AdminKeyStore::new();
        if let Err(e) = key_store.load_from_env() {
            tracing::warn!("Failed to load admin keys from env: {}", e);
        }
        Self { key_store }
    }

    pub fn with_key_store(key_store: AdminKeyStore) -> Self {
        Self { key_store }
    }

    /// Verify a signed request
    pub fn verify_request(&self, req: &SignedRequest) -> Result<AdminEntry> {
        // Check timestamp freshness
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| anyhow!("System time error"))?
            .as_secs();
        
        if now < req.timestamp {
            return Err(anyhow!("Request timestamp is in the future"));
        }
        
        if now - req.timestamp > MAX_REQUEST_AGE_SECS {
            return Err(anyhow!("Request expired (timestamp too old)"));
        }

        // Check replay
        self.key_store.check_replay(&req.public_key, req.timestamp)?;

        // Get admin entry
        let admin = self.key_store
            .get_admin(&req.public_key)
            .ok_or_else(|| anyhow!("Unknown admin public key"))?;

        // Verify signature
        self.verify_signature(req)?;

        // Update last seen
        self.key_store.update_last_seen(&req.public_key);

        tracing::info!(
            admin = %admin.name,
            role = ?admin.role,
            command = %req.command,
            "Admin request authenticated"
        );

        Ok(admin)
    }

    /// Verify Ed25519 signature
    fn verify_signature(&self, req: &SignedRequest) -> Result<()> {
        // Decode public key
        let pubkey_bytes = hex::decode(&req.public_key)
            .map_err(|_| anyhow!("Invalid public key hex"))?;
        
        let pubkey_array: [u8; 32] = pubkey_bytes
            .try_into()
            .map_err(|_| anyhow!("Public key must be 32 bytes"))?;
        
        let verifying_key = VerifyingKey::from_bytes(&pubkey_array)
            .map_err(|_| anyhow!("Invalid Ed25519 public key"))?;

        // Build message to verify
        let message = format!(
            "{}:{}:{}",
            req.timestamp,
            req.command,
            serde_json::to_string(&req.args).unwrap_or_default()
        );

        // Decode signature
        let sig_bytes = hex::decode(&req.signature)
            .map_err(|_| anyhow!("Invalid signature hex"))?;
        
        let sig_array: [u8; 64] = sig_bytes
            .try_into()
            .map_err(|_| anyhow!("Signature must be 64 bytes"))?;
        
        let signature = Signature::from_bytes(&sig_array);

        // Verify
        verifying_key
            .verify(message.as_bytes(), &signature)
            .map_err(|_| anyhow!("Signature verification failed"))?;

        Ok(())
    }

    /// Check if admin has required role
    pub fn check_role(&self, admin: &AdminEntry, required: AdminRole) -> Result<()> {
        let has_permission = match required {
            AdminRole::Monitor => true,  // Everyone can monitor
            AdminRole::Operator => matches!(admin.role, AdminRole::SuperAdmin | AdminRole::Operator),
            AdminRole::SuperAdmin => matches!(admin.role, AdminRole::SuperAdmin),
        };

        if !has_permission {
            return Err(anyhow!(
                "Insufficient permissions: {} requires {:?}, have {:?}",
                admin.name,
                required,
                admin.role
            ));
        }

        Ok(())
    }

    pub fn key_store(&self) -> &AdminKeyStore {
        &self.key_store
    }
}

impl Default for AdminAuth {
    fn default() -> Self {
        Self::new()
    }
}

