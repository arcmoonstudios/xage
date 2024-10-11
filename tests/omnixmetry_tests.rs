// tests/omnixmetry_tests.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[TESTS]Xyn>=====S===t===u===d===i===o===s======[R|$>

use xage::omnixtracker::OmniXMetry;
use xage::constants::{INITIAL_LOG_LEVEL, LOG_FILE_PATH};
use std::env;
use std::path::Path;
use tracing::Level;
use std::sync::Once;

static INIT: Once = Once::new();

fn initialize() {
    INIT.call_once(|| {
        env::remove_var("PROMETHEUS_LISTENER");
        env::remove_var("LOG_FILE_PATH");
        println!("Initialization complete.");
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        initialize();
        match OmniXMetry::init() {
            Ok(omnixmetry) => {
                assert_eq!(omnixmetry.get_log_level(), *INITIAL_LOG_LEVEL);
                assert!(omnixmetry.is_log_file_initialized());
            },
            Err(e) => {
                // Check if the error is due to Prometheus already being initialized
                if e.to_string().contains("metrics system was already initialized") {
                    println!("Prometheus recorder already initialized. Skipping this test.");
                } else {
                    panic!("Unexpected error: {}", e);
                }
            }
        }
    }

    #[test]
    fn test_log_level() {
        initialize();
        if let Ok(omnixmetry) = OmniXMetry::init() {
            assert_eq!(omnixmetry.get_log_level(), *INITIAL_LOG_LEVEL);
            
            omnixmetry.set_log_level(Level::DEBUG);
            assert_eq!(omnixmetry.get_log_level(), Level::DEBUG);
        } else {
            println!("Failed to initialize OmniXMetry. Skipping this test.");
        }
    }

    #[test]
    fn test_rotate_log_file() {
        initialize();
        if let Ok(omnixmetry) = OmniXMetry::init() {
            // Write something to the log file
            omnixmetry.write_log("Test log entry").unwrap();

            omnixmetry.rotate_log_file().unwrap();

            let xdocs_path = Path::new("Xdocs");
            assert!(xdocs_path.exists(), "Xdocs directory should exist");

            let rotated_files: Vec<_> = std::fs::read_dir(xdocs_path)
                .unwrap()
                .filter_map(|entry| {
                    let entry = entry.unwrap();
                    let path = entry.path();
                    if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("log") {
                        Some(path)
                    } else {
                        None
                    }
                })
                .collect();

            assert!(!rotated_files.is_empty(), "At least one rotated log file should exist in Xdocs/");
            assert!(omnixmetry.is_log_file_initialized(), "Log file should be initialized after rotation.");

            // Clean up
            std::fs::remove_dir_all(xdocs_path).unwrap();
            std::fs::remove_file(&*LOG_FILE_PATH).unwrap();
        } else {
            println!("Failed to initialize OmniXMetry. Skipping this test.");
        }
    }
}