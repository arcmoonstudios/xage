// src/omnixtracker/omnixerror.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[OMNIXTRACKER]Xyn>=====S===t===u===d===i===o===s======[R|$>

use crate::omnixtracker::omnixmetry::OmniXMetry;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use parking_lot::{Mutex, RwLock};
use tracing::{error, info, warn};
use git2::Error as GitError;
use thiserror::Error;
use std::fmt;

// Define the NTMError enum for specific error types related to computations.
#[derive(Error, Debug)]
pub enum NTMError {
    #[error("Shape mismatch: expected {expected:?}, actual {actual:?}")]
    ShapeMismatch { expected: Vec<usize>, actual: Vec<usize> },
    
    #[error("A computation error occurred.")]
    ComputationError,
    
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    
    #[error("Memory error: {0}")]
    MemoryError(String),
}

// OmniXError enum for general error handling across the application.
#[derive(Error, Debug)]
pub enum OmniXError {
    #[error("Operation failed during {operation}: {details}")]
    OperationFailed { operation: String, details: String },
    
    #[error("Retry limit exceeded after {retries} attempts: {last_error}")]
    RetryLimitExceeded { retries: usize, last_error: String },
    
    #[error("Circuit breaker activated after {count} errors in {duration:?}")]
    CircuitBreakerActivated { count: usize, duration: Duration },
    
    #[error("Operation timed out after {duration:?}")]
    OperationTimeout { duration: Duration },
    
    #[error("File system error: {0}")]
    FileSystemError(String),
    
    #[error("Environment variable error: {0}")]
    EnvVarError(String),
    
    #[error("Project creation error: {0}")]
    ProjectCreationError(String),
    
    #[error("Metrics initialization error: {0}")]
    MetricsInitError(String),
    
    #[error("Logging error: {0}")]
    LoggingError(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
    
    #[error("Authorization error: {0}")]
    AuthorizationError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    // Integrate NTMError variants into OmniXError
    #[error("Shape mismatch: expected {expected:?}, actual {actual:?}")]
    NTMShapeMismatch { expected: Vec<usize>, actual: Vec<usize> },
    
    #[error("A computation error occurred.")]
    NTMComputationError,
    
    #[error("Invalid argument: {0}")]
    NTMInvalidArgument(String),
    
    #[error("Memory error: {0}")]
    NTMMemoryError(String),
}

// Implement logging for OmniXError
impl OmniXError {
    pub fn log(&self) {
        match self {
            OmniXError::OperationFailed { .. } | OmniXError::RetryLimitExceeded { .. } => {
                error!("{}", self);
            }
            OmniXError::CircuitBreakerActivated { .. } | OmniXError::OperationTimeout { .. } => {
                warn!("{}", self);
            }
            _ => {
                info!("{}", self);
            }
        }
    }
}

// Error handling functions
pub fn handle_build_error(error: Box<dyn std::error::Error>) -> OmniXError {
    match error.downcast::<std::io::Error>() {
        Ok(io_error) => OmniXError::FileSystemError(io_error.to_string()),
        Err(error) => OmniXError::OperationFailed {
            operation: "Build".to_string(),
            details: error.to_string(),
        },
    }
}

pub fn handle_main_error(error: Box<dyn std::error::Error>) -> OmniXError {
    match error.downcast::<std::io::Error>() {
        Ok(io_error) => OmniXError::FileSystemError(io_error.to_string()),
        Err(error) => match error.downcast::<std::env::VarError>() {
            Ok(var_error) => OmniXError::EnvVarError(var_error.to_string()),
            Err(error) => OmniXError::ProjectCreationError(error.to_string()),
        },
    }
}

pub fn handle_metrics_error(error: impl std::error::Error) -> OmniXError {
    OmniXError::MetricsInitError(error.to_string())
}

// Define configuration struct for error management
#[derive(Debug, Clone)]
pub struct OmniXErrorManagerConfig {
    pub max_retries: usize,
    pub circuit_breaker_threshold: usize,
    pub circuit_breaker_duration: Duration,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub timeout: Duration,
}

// Default implementation for OmniXErrorManagerConfig
impl Default for OmniXErrorManagerConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            circuit_breaker_threshold: 10,
            circuit_breaker_duration: Duration::from_secs(60),
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            timeout: Duration::from_secs(30),
        }
    }
}

// Implement conversions from other error types to OmniXError
impl From<GitError> for OmniXError {
    fn from(err: GitError) -> Self {
        OmniXError::OperationFailed {
            operation: "Git operation failed".to_string(),
            details: err.to_string(),
        }
    }
}

impl From<anyhow::Error> for OmniXError {
    fn from(err: anyhow::Error) -> Self {
        OmniXError::OperationFailed {
            operation: "Unknown operation".to_string(),
            details: err.to_string(),
        }
    }
}

// Circuit state management
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CircuitState {
    Closed,
    Open(Instant),
    HalfOpen,
}

impl fmt::Display for CircuitState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CircuitState::Closed => write!(f, "Closed"),
            CircuitState::Open(_) => write!(f, "Open"),
            CircuitState::HalfOpen => write!(f, "Half-Open"),
        }
    }
}

// Define the OmniXErrorManager struct
pub struct OmniXErrorManager {
    error_count: AtomicUsize,
    config: RwLock<OmniXErrorManagerConfig>,
    circuit_state: Mutex<CircuitState>,
    last_error_time: Mutex<Instant>,
    half_open_trial_count: AtomicUsize,
}

// Implement methods for OmniXErrorManager
impl OmniXErrorManager {
    pub fn new(config: OmniXErrorManagerConfig) -> Self {
        Self {
            error_count: AtomicUsize::new(0),
            config: RwLock::new(config),
            circuit_state: Mutex::new(CircuitState::Closed),
            last_error_time: Mutex::new(Instant::now()),
            half_open_trial_count: AtomicUsize::new(0),
        }
    }

    // Async error handling with retry logic
    pub async fn handle_error<T, F, Fut>(
        &self,
        operation: F,
        metrics: &OmniXMetry,
    ) -> Result<T, OmniXError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, OmniXError>>,
    {
        let config = self.config.read();

        if !self.check_circuit_state() {
            metrics.increment_counter("error_manager.circuit_opened".to_string(), 1);
            return Err(OmniXError::CircuitBreakerActivated {
                count: self.error_count.load(Ordering::Relaxed),
                duration: config.circuit_breaker_duration,
            });
        }

        for retries in 0..config.max_retries {
            let start_time = Instant::now();
            match tokio::time::timeout(config.timeout, operation()).await {
                Ok(Ok(result)) => {
                    self.error_count.store(0, Ordering::Relaxed);
                    self.close_circuit();
                    metrics.increment_counter("error_manager.successes".to_string(), 1);
                    metrics.update_gauge("error_manager.operation_latency".to_string(), start_time.elapsed().as_secs_f64());
                    info!("Operation succeeded on attempt {}", retries + 1);
                    return Ok(result);
                }
                Ok(Err(e)) => {
                    e.log();
                    self.error_count.fetch_add(1, Ordering::Relaxed);
                    *self.last_error_time.lock() = Instant::now();

                    metrics.increment_counter("error_manager.failures".to_string(), 1);
                    metrics.increment_counter(format!("error_manager.failures.{}", e), 1);

                    if self.error_count.load(Ordering::Relaxed) >= config.circuit_breaker_threshold {
                        self.open_circuit();
                        metrics.increment_counter("error_manager.circuit_tripped".to_string(), 1);
                        error!(
                            "Circuit breaker tripped after {} consecutive failures",
                            self.error_count.load(Ordering::Relaxed)
                        );
                        return Err(OmniXError::CircuitBreakerActivated {
                            count: self.error_count.load(Ordering::Relaxed),
                            duration: config.circuit_breaker_duration,
                        });
                    }

                    if retries == config.max_retries - 1 {
                        metrics.increment_counter("error_manager.max_retries_exceeded".to_string(), 1);
                        metrics.update_gauge("error_manager.operation_latency".to_string(), start_time.elapsed().as_secs_f64());
                        return Err(OmniXError::RetryLimitExceeded {
                            retries: retries + 1,
                            last_error: e.to_string(),
                        });
                    }

                    let delay = config
                        .base_delay
                        .mul_f32(2_f32.powi(retries as i32))
                        .min(config.max_delay);
                    tokio::time::sleep(delay).await;
                }
                Err(_) => {
                    metrics.increment_counter("error_manager.operation_timeout".to_string(), 1);
                    metrics.update_gauge("error_manager.operation_latency".to_string(), config.timeout.as_secs_f64());
                    return Err(OmniXError::OperationTimeout {
                        duration: config.timeout,
                    });
                }
            }
        }

        Err(OmniXError::RetryLimitExceeded {
            retries: config.max_retries,
            last_error: "Maximum retries reached".to_string(),
        })
    }

    fn check_circuit_state(&self) -> bool {
        let mut circuit_state = self.circuit_state.lock();
        match *circuit_state {
            CircuitState::Closed => true,
            CircuitState::Open(opened_at) => {
                let config = self.config.read();
                if opened_at.elapsed() >= config.circuit_breaker_duration {
                    *circuit_state = CircuitState::HalfOpen;
                    self.half_open_trial_count.store(0, Ordering::Relaxed);
                    warn!("Circuit breaker transitioning to HalfOpen state");
                    true
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => {
                if self.half_open_trial_count.fetch_add(1, Ordering::Relaxed) < 1 {
                    info!("Circuit breaker is Half-Open; allowing trial operation");
                    true
                } else {
                    warn!("Circuit breaker is Half-Open; trial limit reached");
                    false
                }
            }
        }
    }

    fn open_circuit(&self) {
        let mut circuit_state = self.circuit_state.lock();
        *circuit_state = CircuitState::Open(Instant::now());
        self.half_open_trial_count.store(0, Ordering::Relaxed);
        warn!("Circuit breaker opened");
    }

    fn close_circuit(&self) {
        let mut circuit_state = self.circuit_state.lock();
        *circuit_state = CircuitState::Closed;
        self.half_open_trial_count.store(0, Ordering::Relaxed);
        info!("Circuit breaker closed");
    }

    pub fn update_config(&self, new_config: OmniXErrorManagerConfig) {
        let mut config = self.config.write();
        *config = new_config;
        info!("OmniXErrorManager configuration updated");
    }
}