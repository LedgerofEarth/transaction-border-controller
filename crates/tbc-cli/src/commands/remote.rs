//! Remote TBC Commands
//!
//! Connect to a running TBC instance and execute admin commands.
//! Uses Ed25519 signed requests for authentication.

use anyhow::{anyhow, Result};
use clap::{Args, Subcommand};
use ed25519_dalek::{Signer, SigningKey};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Remote command arguments
#[derive(Args)]
pub struct RemoteArgs {
    /// TBC admin endpoint URL
    #[arg(short, long, default_value = "http://localhost:8080")]
    pub url: String,

    /// Ed25519 private key (hex-encoded)
    #[arg(short, long)]
    pub key: Option<String>,

    /// Admin command to execute
    #[command(subcommand)]
    pub command: RemoteCommands,
}

#[derive(Subcommand)]
pub enum RemoteCommands {
    /// Ping the TBC (connectivity check)
    Ping,
    
    /// Get TBC health status
    Health,
    
    /// Get current configuration
    Config,
    
    /// Get connection statistics
    Stats,
    
    /// Get recent logs
    Logs {
        /// Number of log lines to retrieve
        #[arg(short, long, default_value = "50")]
        lines: usize,
        
        /// Filter by log level
        #[arg(short = 'L', long)]
        level: Option<String>,
    },
    
    /// List active connections
    Connections,
    
    /// Get verification layer status
    Layers,
    
    /// List registered admins
    Admins,
    
    /// Add a new admin
    AddAdmin {
        /// Admin name
        #[arg(short, long)]
        name: String,
        
        /// Public key (hex)
        #[arg(short, long)]
        pubkey: String,
        
        /// Role: super, operator, monitor
        #[arg(short, long, default_value = "monitor")]
        role: String,
    },
    
    /// Generate a new admin keypair
    Keygen {
        /// Output file for private key
        #[arg(short, long)]
        output: Option<String>,
    },
    
    /// Request graceful shutdown
    Shutdown {
        /// Delay in seconds before shutdown
        #[arg(short, long, default_value = "5")]
        delay: u64,
    },
}

/// Signed request structure
#[derive(Debug, Serialize, Deserialize)]
struct SignedRequest {
    public_key: String,
    timestamp: u64,
    command: String,
    args: serde_json::Value,
    signature: String,
}

/// Command result from TBC
#[derive(Debug, Deserialize)]
struct CommandResult {
    success: bool,
    command: String,
    data: Option<serde_json::Value>,
    error: Option<String>,
    timestamp: u64,
}

/// Handle remote command execution
pub async fn handle_remote(args: RemoteArgs) -> Result<()> {
    match &args.command {
        RemoteCommands::Keygen { output } => {
            return generate_keypair(output.as_deref());
        }
        _ => {}
    }

    // Load private key
    let key_hex = args.key
        .ok_or_else(|| anyhow!("Admin key required. Set TBC_ADMIN_KEY or use --key"))?;
    
    let signing_key = load_signing_key(&key_hex)?;
    let public_key = hex::encode(signing_key.verifying_key().as_bytes());

    // Build command
    let (cmd_name, cmd_args) = match &args.command {
        RemoteCommands::Ping => ("Ping", serde_json::json!(null)),
        RemoteCommands::Health => ("Health", serde_json::json!(null)),
        RemoteCommands::Config => ("GetConfig", serde_json::json!(null)),
        RemoteCommands::Stats => ("GetStats", serde_json::json!(null)),
        RemoteCommands::Logs { lines, level } => ("GetLogs", serde_json::json!({
            "lines": lines,
            "level": level,
        })),
        RemoteCommands::Connections => ("ListConnections", serde_json::json!(null)),
        RemoteCommands::Layers => ("GetLayerStatus", serde_json::json!(null)),
        RemoteCommands::Admins => ("ListAdmins", serde_json::json!(null)),
        RemoteCommands::AddAdmin { name, pubkey, role } => ("AddAdmin", serde_json::json!({
            "name": name,
            "public_key": pubkey,
            "role": role,
        })),
        RemoteCommands::Shutdown { delay } => ("Shutdown", serde_json::json!({
            "delay_secs": delay,
        })),
        RemoteCommands::Keygen { .. } => unreachable!(),
    };

    // Create signed request
    let request = create_signed_request(&signing_key, &public_key, cmd_name, cmd_args)?;

    // Send request
    let url = format!("{}/admin/exec", args.url.trim_end_matches('/'));
    let client = reqwest::Client::new();
    
    println!("Connecting to {}...", args.url);
    
    let response = client
        .post(&url)
        .json(&request)
        .send()
        .await
        .map_err(|e| anyhow!("Failed to connect: {}", e))?;

    let status = response.status();
    let result: CommandResult = response
        .json()
        .await
        .map_err(|e| anyhow!("Failed to parse response: {}", e))?;

    // Display result
    if result.success {
        println!("\n✓ {} succeeded", result.command);
        if let Some(data) = result.data {
            println!("{}", serde_json::to_string_pretty(&data)?);
        }
    } else {
        println!("\n✗ {} failed", result.command);
        if let Some(error) = result.error {
            println!("Error: {}", error);
        }
        if !status.is_success() {
            println!("HTTP Status: {}", status);
        }
    }

    Ok(())
}

/// Load Ed25519 signing key from hex string
fn load_signing_key(hex_key: &str) -> Result<SigningKey> {
    let key_bytes = hex::decode(hex_key.trim())
        .map_err(|_| anyhow!("Invalid key hex"))?;
    
    let key_array: [u8; 32] = key_bytes
        .try_into()
        .map_err(|_| anyhow!("Key must be 32 bytes"))?;
    
    Ok(SigningKey::from_bytes(&key_array))
}

/// Create a signed admin request
fn create_signed_request(
    signing_key: &SigningKey,
    public_key: &str,
    command: &str,
    args: serde_json::Value,
) -> Result<SignedRequest> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| anyhow!("System time error"))?
        .as_secs();

    // Build message to sign
    let message = format!(
        "{}:{}:{}",
        timestamp,
        command,
        serde_json::to_string(&args).unwrap_or_default()
    );

    // Sign
    let signature = signing_key.sign(message.as_bytes());
    let signature_hex = hex::encode(signature.to_bytes());

    Ok(SignedRequest {
        public_key: public_key.to_string(),
        timestamp,
        command: command.to_string(),
        args,
        signature: signature_hex,
    })
}

/// Generate a new Ed25519 keypair
fn generate_keypair(output: Option<&str>) -> Result<()> {
    use rand::RngCore;
    
    // Generate 32 random bytes for the secret key
    let mut secret_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut secret_bytes);
    
    let signing_key = SigningKey::from_bytes(&secret_bytes);
    let verifying_key = signing_key.verifying_key();
    
    let private_hex = hex::encode(signing_key.to_bytes());
    let public_hex = hex::encode(verifying_key.as_bytes());
    
    println!("\n🔑 Generated new Ed25519 admin keypair\n");
    println!("Public Key:  {}", public_hex);
    println!("Private Key: {}", private_hex);
    println!();
    println!("⚠️  SAVE YOUR PRIVATE KEY SECURELY!");
    println!();
    println!("To use this key:");
    println!("  1. Add to TBC: TBC_ADMIN_KEYS=\"admin:{}:super\"", public_hex);
    println!("  2. Set in CLI: export TBC_ADMIN_KEY=\"{}\"", private_hex);
    println!();
    
    if let Some(path) = output {
        std::fs::write(path, &private_hex)?;
        println!("Private key saved to: {}", path);
        
        let pub_path = format!("{}.pub", path);
        std::fs::write(&pub_path, &public_hex)?;
        println!("Public key saved to: {}", pub_path);
    }
    
    Ok(())
}

