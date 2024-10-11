// src/constants/mod.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[CONSTANTS]Xyn>=====S===t===u===d===i===o===s======[R|$>

use once_cell::sync::Lazy;
use std::time::Duration;
use std::env;
use tracing::Level;

// Signature and author-related constants
pub const ARCMOON_SIGNATURE: &str = "~=#######D]======A===r===c====M===o===o===n=====<Lord[{}]Xyn>=====S===t===u===d===i===o===s======[R|$>";
pub const LICENSE_YEAR: &str = "2024";
pub const LICENSE_HOLDER: &str = "Lord Xyn";
pub const AUTHOR_NAME: &str = "Lord Xyn";
pub const AUTHOR_EMAIL: &str = "LordXyn@proton.me";
pub const GITHUB_URL: &str = "https://github.com/arcmoonstudios/xage";

// Database-related constants
pub const DB_PATH: &str = "parallexelerator.db";

// Task-related constants
pub const TASK_CHANNEL_BUFFER_SIZE: usize = 1000;
pub const PROGRESS_CHANNEL_BUFFER_SIZE: usize = 100;
pub const INITIAL_TASK_CONCURRENCY: usize = 4;
pub const DEFAULT_TASK_QUEUE_SIZE: usize = 1000;
pub const DEFAULT_MAX_CONCURRENT_TASKS: usize = 10;
pub const DEFAULT_UPDATE_INTERVAL_MS: u64 = 1000;
pub const METRICS_UPDATE_INTERVAL_MS: u64 = 1000;
pub const CHECKPOINT_INTERVAL: usize = 100;

// Resource utilization thresholds
pub const LOW_UTILIZATION_THRESHOLD: f32 = 0.3;
pub const HIGH_UTILIZATION_THRESHOLD: f32 = 0.8;
pub const ACCELERATION_THRESHOLD: f32 = 0.5;

// Timeout constants
pub const SHUTDOWN_TIMEOUT_SECONDS: u64 = 30;
pub const THREAD_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);
pub const KERNEL_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);

// Scaling threshold constants
pub const CPU_SCALE_THRESHOLD_ACCELERATE: f32 = 0.8;
pub const GPU_SCALE_THRESHOLD_ACCELERATE: f32 = 0.8;
pub const MEMORY_SCALE_THRESHOLD_ACCELERATE: f32 = 0.8;
pub const CPU_SCALE_THRESHOLD_DECELERATE: f32 = 0.2;
pub const GPU_SCALE_THRESHOLD_DECELERATE: f32 = 0.2;
pub const MEMORY_SCALE_THRESHOLD_DECELERATE: f32 = 0.2;

// Worker and GPU kernel constants
pub const MIN_WORKER_THREADS: usize = 1;
pub const MAX_WORKER_THREADS: usize = 32;
pub const MIN_GPU_KERNELS: usize = 1;
pub const MAX_GPU_KERNELS: usize = 8;

// Git-related constants
pub const GIT_REMOTE: &str = "origin"; 
pub const GIT_BRANCH: &str = "main"; 
pub const GIT_COMMIT_MESSAGE: &str = "Automated update via xyngit"; 
pub const MAX_RETRIES: usize = 3; 
pub const RETRY_DELAY: Duration = Duration::from_secs(2); 


// Password and security-related constants
pub const PASSWORD_SALT_LENGTH: usize = 32;
pub const PASSWORD_HASH_ITERATIONS: u32 = 100_000;
pub const JWT_EXPIRATION: i64 = 3600;
pub const RATE_LIMIT_WINDOW: u64 = 60;
pub const RATE_LIMIT_MAX_REQUESTS: u32 = 100;
pub const ENABLE_EXPERIMENTAL_FEATURES: bool = false;
pub const USE_LEGACY_AUTH: bool = false;

// Prometheus and log-related constants
pub static PROMETHEUS_LISTENER: Lazy<String> = Lazy::new(|| env::var("PROMETHEUS_LISTENER").unwrap_or_else(|_| "0.0.0.0:9001".to_string()));
pub static PROMETHEUS_TEST_LISTENER: Lazy<String> = Lazy::new(|| env::var("PROMETHEUS_TEST_LISTENER").unwrap_or_else(|_| "127.0.0.1:0".to_string()));
pub static INITIAL_LOG_LEVEL: Lazy<Level> = Lazy::new(|| env::var("INITIAL_LOG_LEVEL").map(|v| v.parse().unwrap_or(Level::INFO)).unwrap_or(Level::INFO));
pub static LOG_FILE_PATH: Lazy<String> = Lazy::new(|| env::var("LOG_FILE_PATH").unwrap_or_else(|_| "xynpro.log".to_string()));

// Circuit breaker-related constants
pub static CIRCUIT_BREAKER_THRESHOLD: Lazy<usize> = Lazy::new(|| env::var("CIRCUIT_BREAKER_THRESHOLD").ok().and_then(|v| v.parse().ok()).unwrap_or(10));
pub static CIRCUIT_BREAKER_DURATION: Lazy<Duration> = Lazy::new(|| Duration::from_secs(env::var("CIRCUIT_BREAKER_DURATION").ok().and_then(|v| v.parse().ok()).unwrap_or(60)));
pub static BASE_DELAY: Lazy<Duration> = Lazy::new(|| Duration::from_millis(env::var("BASE_DELAY").ok().and_then(|v| v.parse().ok()).unwrap_or(100)));
pub static MAX_DELAY: Lazy<Duration> = Lazy::new(|| Duration::from_secs(env::var("MAX_DELAY").ok().and_then(|v| v.parse().ok()).unwrap_or(10)));
pub static DEFAULT_TIMEOUT: Lazy<Duration> = Lazy::new(|| Duration::from_secs(env::var("DEFAULT_TIMEOUT").ok().and_then(|v| v.parse().ok()).unwrap_or(30)));

// NTM constants
pub const MAX_MEMORY_SIZE: usize = 1024;
pub const MAX_KEY_SIZE: usize = 64;
pub const MEMORY_VECTOR_SIZE: usize = 32;
pub const DEFAULT_MEMORY_SIZE: usize = 1024;
pub const DEFAULT_MEMORY_VECTOR_SIZE: usize = 64;
pub const DEFAULT_CONTROLLER_SIZE: usize = 128;