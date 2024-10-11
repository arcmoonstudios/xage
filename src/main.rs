// src/main.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[MAIN]Xyn>=====S===t===u===d===i===o===s======[R|$>

use crate::omnixtracker::{OmniXMetry, OmniXError, OmniXErrorManager, OmniXErrorManagerConfig, InitError, handle_init_error, setup_global_subscriber};
use crate::constants::{CIRCUIT_BREAKER_THRESHOLD, CIRCUIT_BREAKER_DURATION, BASE_DELAY, MAX_DELAY, DEFAULT_TIMEOUT};
use anyhow::Result;
use dotenv::dotenv;
use std::env::args; 
use tracing::info;
use std::env; 

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    info!("Welcome to Lord Xyn's Domain! Initializing systems...");

    // Check if placeholders in .env are replaced
    if env::var("GPG_PASSPHRASE").unwrap_or_default() == "your_gpg_passphrase_placeholder" {
        return Err(anyhow::anyhow!("Please replace the GPG_PASSPHRASE placeholder in the .env file with your actual GPG passphrase."));
    }
    if env::var("GIT_REMOTE_URL").unwrap_or_default() == "https://github.com/your/repo.git" {
        return Err(anyhow::anyhow!("Please replace the GIT_REMOTE_URL placeholder in the .env file with the actual URL of your GitHub repository."));
    }
    if env::var("JWT_SECRET").unwrap_or_default() == "your_JWT_passphrase_placeholder" {
        return Err(anyhow::anyhow!("Please replace the JWT_SECRET placeholder in the .env file with your actual JWT secret."));
    }

    // Initialize OmniXMetry for logging and metrics
    let omnixmetry = OmniXMetry::init()?;
    setup_global_subscriber(omnixmetry.clone())?;
    info!("OmniXMetry initialized successfully.");

    // Initialize OmniXErrorManager for error handling
    let error_manager_config = OmniXErrorManagerConfig {
        max_retries: env::var("MAX_RETRIES").unwrap_or_else(|_| "3".to_string()).parse().unwrap_or(3),
        circuit_breaker_threshold: *CIRCUIT_BREAKER_THRESHOLD,
        circuit_breaker_duration: *CIRCUIT_BREAKER_DURATION,
        base_delay: *BASE_DELAY,
        max_delay: *MAX_DELAY,
        timeout: *DEFAULT_TIMEOUT,
    };
    let omnix_error_manager = OmniXErrorManager::new(error_manager_config.clone());
    info!("OmniXErrorManager initialized successfully.");

    // Use the omnix_error_manager to ensure it's not unused
    let _ = &omnix_error_manager;

    // Process command-line arguments
    let args: Vec<String> = args().collect(); // Collect args into a vector

    // Check for the minimum number of arguments
    if args.len() < 2 {
        return Err(anyhow::anyhow!("Usage: {} --version", args[0]));
    }

    // Process command-line arguments
    match args[1].as_str() {
        "--version" => {
            println!("xynpro version 0.1.0");
            return Ok(());
        }
        _ => {
            // Future commands can be added here
            info!("No valid command provided. Exiting.");
        }
    }

    Ok(())
}