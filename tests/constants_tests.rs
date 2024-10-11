// tests/constants_tests.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[TESTS]Xyn>=====S===t===u===d===i===o===s======[R|$>

#[cfg(test)]
mod tests {{
    use crate::constants::*;
    use std::env;

    #[test]
    fn test_project_directories() {{
        let expected_directories = vec![
            "src/omnixtracker",
            "src/constants",
            "src/utils",
            "tests",
            "Xdocs",
            "Xtls",
        ];
        assert_eq!(PROJECT_DIRECTORIES.to_vec(), expected_directories);
    }}

    #[test]
    fn test_password_salt_length() {{
        assert_eq!(PASSWORD_SALT_LENGTH, 32);
    }}

    #[test]
    fn test_password_hash_iterations() {{
        assert_eq!(PASSWORD_HASH_ITERATIONS, 100_000);
    }}

    #[test]
    fn test_jwt_expiration() {{
        assert_eq!(JWT_EXPIRATION, 3600);
    }}

    #[test]
    fn test_rate_limit_window() {{
        assert_eq!(RATE_LIMIT_WINDOW, 60);
    }}

    #[test]
    fn test_rate_limit_max_requests() {{
        assert_eq!(RATE_LIMIT_MAX_REQUESTS, 100);
    }}

    #[test]
    fn test_enable_experimental_features() {{
        assert_eq!(ENABLE_EXPERIMENTAL_FEATURES, false);
    }}

    #[test]
    fn test_use_legacy_auth() {{
        assert_eq!(USE_LEGACY_AUTH, false);
    }}

    #[test]
    fn test_prometheus_listener_default() {{
        env::remove_var("PROMETHEUS_LISTENER");
        assert_eq!(&*PROMETHEUS_LISTENER, "0.0.0.0:9001");
    }}

    #[test]
    fn test_initial_log_level_default() {{
        env::remove_var("INITIAL_LOG_LEVEL");
        assert_eq!(*INITIAL_LOG_LEVEL, tracing::Level::INFO);
    }}

    #[test]
    fn test_log_file_path_default() {{
        env::remove_var("LOG_FILE_PATH");
        assert_eq!(&*LOG_FILE_PATH, "xynpro.log");
    }}

    #[test]
    fn test_git_remote_default() {{
        env::remove_var("GIT_REMOTE");
        assert_eq!(&*GIT_REMOTE, "origin");
    }}

    #[test]
    fn test_git_branch_default() {{
        env::remove_var("GIT_BRANCH");
        assert_eq!(&*GIT_BRANCH, "main");
    }}

    #[test]
    fn test_git_commit_message_default() {{
        env::remove_var("GIT_COMMIT_MESSAGE");
        assert_eq!(&*GIT_COMMIT_MESSAGE, "Automated update via xyngit");
    }}

    #[test]
    fn test_circuit_breaker_threshold_default() {{
        env::remove_var("CIRCUIT_BREAKER_THRESHOLD");
        assert_eq!(*CIRCUIT_BREAKER_THRESHOLD, 10);
    }}

    #[test]
    fn test_circuit_breaker_duration_default() {{
        env::remove_var("CIRCUIT_BREAKER_DURATION");
        assert_eq!(*CIRCUIT_BREAKER_DURATION, std::time::Duration::from_secs(60));
    }}

    #[test]
    fn test_base_delay_default() {{
        env::remove_var("BASE_DELAY");
        assert_eq!(*BASE_DELAY, std::time::Duration::from_millis(100));
    }}

    #[test]
    fn test_max_delay_default() {{
        env::remove_var("MAX_DELAY");
        assert_eq!(*MAX_DELAY, std::time::Duration::from_secs(10));
    }}

    #[test]
    fn test_default_timeout_default() {{
        env::remove_var("DEFAULT_TIMEOUT");
        assert_eq!(*DEFAULT_TIMEOUT, std::time::Duration::from_secs(30));
    }}

    #[test]
    fn test_max_retries_default() {{
        env::remove_var("MAX_RETRIES");
        assert_eq!(get_max_retries(), 3);
    }}
}}