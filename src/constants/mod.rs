// src/constants/mod.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[CONSTANTS]Xyn>=====S===t===u===d===i===o===s======[R|$>

use once_cell::sync::Lazy;
use std::time::Duration;
use std::env;
use tracing::Level;

// Custom Signature Line constant
pub const ARCMOON_SIGNATURE: &str = "~=#######D]======A===r===c====M===o===o===n=====<Lord[{}]Xyn>=====S===t===u===d===i===o===s======[R|$>";

// Git-related constants
pub const LICENSE_YEAR: &str = "2024";
pub const LICENSE_HOLDER: &str = "Lord Xyn";
pub const AUTHOR_NAME: &str = "Lord Xyn";
pub const AUTHOR_EMAIL: &str = "LordXyn@proton.me";
pub const GITHUB_URL: &str = "https://github.com/arcmoonstudios/xage";
pub const GIT_REMOTE: &str = "origin"; 
pub const GIT_BRANCH: &str = "Domain"; 
pub const GIT_COMMIT_MESSAGE: &str = "Automated update via xyngit"; 
pub const MAX_RETRIES: usize = 3; 
pub const RETRY_DELAY: Duration = Duration::from_secs(2); 

// OmniXelerator Task-related constants
pub const TASK_CHANNEL_BUFFER_SIZE: usize = 1000;
pub const PROGRESS_CHANNEL_BUFFER_SIZE: usize = 100;
pub const INITIAL_TASK_CONCURRENCY: usize = 4;
pub const DEFAULT_TASK_QUEUE_SIZE: usize = 1000;
pub const DEFAULT_MAX_CONCURRENT_TASKS: usize = 10;
pub const DEFAULT_UPDATE_INTERVAL_MS: u64 = 1000;
pub const METRICS_UPDATE_INTERVAL_MS: u64 = 1000;
pub const CHECKPOINT_INTERVAL: usize = 100;

// OmniXelerator - Resource utilization thresholds
pub const LOW_UTILIZATION_THRESHOLD: f32 = 0.3;
pub const HIGH_UTILIZATION_THRESHOLD: f32 = 0.8;
pub const ACCELERATION_THRESHOLD: f32 = 0.5;

// Timeout constants
pub const SHUTDOWN_TIMEOUT_SECONDS: u64 = 30;
pub const THREAD_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);
pub const KERNEL_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);

// OmniXelerator - Scaling threshold constants
pub const CPU_SCALE_THRESHOLD_ACCELERATE: f32 = 0.8;
pub const GPU_SCALE_THRESHOLD_ACCELERATE: f32 = 0.8;
pub const MEMORY_SCALE_THRESHOLD_ACCELERATE: f32 = 0.8;
pub const CPU_SCALE_THRESHOLD_DECELERATE: f32 = 0.2;
pub const GPU_SCALE_THRESHOLD_DECELERATE: f32 = 0.2;
pub const MEMORY_SCALE_THRESHOLD_DECELERATE: f32 = 0.2;

// OmniXelerator - Worker and GPU kernel constants
pub const MIN_WORKER_THREADS: usize = 1;
pub const MAX_WORKER_THREADS: usize = 32;
pub const MIN_GPU_KERNELS: usize = 1;
pub const MAX_GPU_KERNELS: usize = 8;

// Password and security-related constants
pub const PASSWORD_SALT_LENGTH: usize = 32;
pub const PASSWORD_HASH_ITERATIONS: u32 = 100_000;
pub const JWT_EXPIRATION: i64 = 3600;
pub const RATE_LIMIT_WINDOW: u64 = 60;
pub const RATE_LIMIT_MAX_REQUESTS: u32 = 100;
pub const ENABLE_EXPERIMENTAL_FEATURES: bool = false;
pub const USE_LEGACY_AUTH: bool = false;

// OmniXMetry - Prometheus and log-related constants
pub static PROMETHEUS_LISTENER: Lazy<String> = Lazy::new(|| env::var("PROMETHEUS_LISTENER").unwrap_or_else(|_| "0.0.0.0:9001".to_string()));
pub static PROMETHEUS_TEST_LISTENER: Lazy<String> = Lazy::new(|| env::var("PROMETHEUS_TEST_LISTENER").unwrap_or_else(|_| "127.0.0.1:0".to_string()));
pub static INITIAL_LOG_LEVEL: Lazy<Level> = Lazy::new(|| env::var("INITIAL_LOG_LEVEL").map(|v| v.parse().unwrap_or(Level::INFO)).unwrap_or(Level::INFO));
pub static LOG_FILE_PATH: Lazy<String> = Lazy::new(|| env::var("LOG_FILE_PATH").unwrap_or_else(|_| "xynpro.log".to_string()));

// OmniXError - Circuit breaker-related constants
pub static CIRCUIT_BREAKER_THRESHOLD: Lazy<usize> = Lazy::new(|| env::var("CIRCUIT_BREAKER_THRESHOLD").ok().and_then(|v| v.parse().ok()).unwrap_or(10));
pub static CIRCUIT_BREAKER_DURATION: Lazy<Duration> = Lazy::new(|| Duration::from_secs(env::var("CIRCUIT_BREAKER_DURATION").ok().and_then(|v| v.parse().ok()).unwrap_or(60)));
pub static BASE_DELAY: Lazy<Duration> = Lazy::new(|| Duration::from_millis(env::var("BASE_DELAY").ok().and_then(|v| v.parse().ok()).unwrap_or(100)));
pub static MAX_DELAY: Lazy<Duration> = Lazy::new(|| Duration::from_secs(env::var("MAX_DELAY").ok().and_then(|v| v.parse().ok()).unwrap_or(10)));
pub static DEFAULT_TIMEOUT: Lazy<Duration> = Lazy::new(|| Duration::from_secs(env::var("DEFAULT_TIMEOUT").ok().and_then(|v| v.parse().ok()).unwrap_or(30)));

// APROAR - NTM constants

// Training parameters
pub const NTM_MEMORY_USAGE_THRESHOLD: f32 = 0.9; // Memory usage threshold for NTM (90% utilization)
pub const NTM_LEARNING_RATE: f32 = 0.0001; // Learning rate for NTM training (smaller for stability)
pub const NTM_BATCH_SIZE: usize = 16; // Batch size for NTM training (smaller for memory efficiency)
pub const NTM_EPOCHS: usize = 1000; // Number of epochs for NTM training (increased for better convergence)
pub const NTM_CLIP_VALUE: f32 = 5.0; // Gradient clipping value for NTM (reduced to prevent exploding gradients)

// NTM architecture parameters
pub const MAX_MEMORY_SIZE: usize = 4096; // Maximum memory size for NTM (increased for larger capacity)
pub const MAX_KEY_SIZE: usize = 128; // Maximum key size for NTM (increased for more complex addressing)
pub const MEMORY_VECTOR_SIZE: usize = 64; // Size of memory vector for NTM (balanced for capacity and efficiency)
pub const DEFAULT_MEMORY_SIZE: usize = 2048; // Default memory size for NTM (half of max for flexibility)
pub const DEFAULT_MEMORY_VECTOR_SIZE: usize = 64; // Default memory vector size for NTM (same as MEMORY_VECTOR_SIZE)
pub const DEFAULT_CONTROLLER_SIZE: usize = 256; // Default controller size for NTM (increased for more processing power)

// NTM operational parameters
pub const NTM_INPUT_SIZE: usize = 512; // Size of input vector (increased for more complex inputs)
pub const NTM_OUTPUT_SIZE: usize = 512; // Size of output vector (matched with input size)
pub const NTM_MEMORY_SIZE: usize = 2048; // Number of memory locations (same as DEFAULT_MEMORY_SIZE)
pub const NTM_MEMORY_VECTOR_SIZE: usize = 64; // Size of each memory vector (same as DEFAULT_MEMORY_VECTOR_SIZE)
pub const NTM_CONTROLLER_SIZE: usize = 256; // Size of controller hidden state (same as DEFAULT_CONTROLLER_SIZE)
pub const CONTEXT_WINDOW_SIZE: usize = 10000; // Number of recent items to keep in context (increased significantly)
