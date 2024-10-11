// tests/omnixerror_tests.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[TESTS]Xyn>=====S===t===u===d===i===o===s======[R|$>

#[cfg(test)]
mod tests {
    use xage::omnixtracker::OmniXError;
    use std::time::Duration;

    #[test]
    fn test_operation_failed_error() {
        let error = OmniXError::OperationFailed {
            operation: "test".to_string(),
            details: "Some error occurred".to_string() 
        };
        assert_eq!(format!("{}", error), "Operation failed during test: Some error occurred");
    }

    #[test]
    fn test_retry_limit_exceeded_error() {
        let error = OmniXError::RetryLimitExceeded {
            retries: 5,
            last_error: "Last attempt failed".to_string(),
        };
        assert_eq!(format!("{}", error), "Retry limit exceeded after 5 attempts: Last attempt failed");
    }

    #[test]
    fn test_circuit_breaker_activated_error() {
        let error = OmniXError::CircuitBreakerActivated {
            count: 10,
            duration: Duration::from_secs(60),
        };
        assert_eq!(format!("{}", error), "Circuit breaker activated after 10 errors in 60s");
    }

    #[test]
    fn test_operation_timeout_error() {
        let error = OmniXError::OperationTimeout {
            duration: Duration::from_secs(30),
        };
        assert_eq!(format!("{}", error), "Operation timed out after 30s");
    }

    #[test]
    fn test_file_system_error() {
        let error = OmniXError::FileSystemError("Disk not found".to_string());
        assert_eq!(format!("{}", error), "File system error: Disk not found");
    }

    #[test]
    fn test_env_var_error() {
        let error = OmniXError::EnvVarError("Missing environment variable".to_string());
        assert_eq!(format!("{}", error), "Environment variable error: Missing environment variable");
    }

    #[test]
    fn test_project_creation_error() {
        let error = OmniXError::ProjectCreationError("Unable to create project".to_string());
        assert_eq!(format!("{}", error), "Project creation error: Unable to create project");
    }

    #[test]
    fn test_metrics_init_error() {
        let error = OmniXError::MetricsInitError("Failed to initialize metrics".to_string());
        assert_eq!(format!("{}", error), "Metrics initialization error: Failed to initialize metrics");
    }

    #[test]
    fn test_logging_error() {
        let error = OmniXError::LoggingError("Log file not found".to_string());
        assert_eq!(format!("{}", error), "Logging error: Log file not found");
    }

    #[test]
    fn test_database_error() {
        let error = OmniXError::DatabaseError("Connection failed".to_string());
        assert_eq!(format!("{}", error), "Database error: Connection failed");
    }

    #[test]
    fn test_network_error() {
        let error = OmniXError::NetworkError("Network unreachable".to_string());
        assert_eq!(format!("{}", error), "Network error: Network unreachable");
    }

    #[test]
    fn test_authentication_error() {
        let error = OmniXError::AuthenticationError("Invalid credentials".to_string());
        assert_eq!(format!("{}", error), "Authentication error: Invalid credentials");
    }

    #[test]
    fn test_authorization_error() {
        let error = OmniXError::AuthorizationError("Access denied".to_string());
        assert_eq!(format!("{}", error), "Authorization error: Access denied");
    }

    #[test]
    fn test_validation_error() {
        let error = OmniXError::ValidationError("Input is invalid".to_string());
        assert_eq!(format!("{}", error), "Validation error: Input is invalid");
    }
}