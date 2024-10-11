// tests/utils_lxsl_tests.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[TESTS]Xyn>=====S===t===u===d===i===o===s======[R|$>
// tests/utils_lxsl_tests.rs
use crate::{
    constants::ARCMOON_SIGNATURE,
    utils::lxsl::LordXynSignatureLine,
};
use std::path::Path;

#[test]
fn test_generate_signature_line() {
    let file_path = "src/utils/lxsl.rs";
    let signature = LordXynSignatureLine::generate_signature_line(file_path);
    assert!(signature.contains("// src/utils/"));
    assert!(signature.contains(ARCMOON_SIGNATURE));
    assert!(signature.contains("<Lord[UTILS]Xyn>"));
}

#[test]
fn test_build_signature_path() {
    let path_parts = &["src", "utils", "lxsl.rs"];
    let signature_path = LordXynSignatureLine::build_signature_path(path_parts);
    assert_eq!(signature_path, "src/utils/");
}

#[test]
fn test_build_xyn_signature() {
    let path_parts = &["src", "utils", "lxsl.rs"];
    let xyn_signature = LordXynSignatureLine::build_xyn_signature(path_parts);
    assert_eq!(xyn_signature, "LXSL");
}

#[test]
fn test_get_comment_prefix() {
    assert_eq!(LordXynSignatureLine::get_comment_prefix("rs"), "//");
    assert_eq!(LordXynSignatureLine::get_comment_prefix("py"), "#");
    assert_eq!(LordXynSignatureLine::get_comment_prefix("html"), "<!--");
    assert_eq!(LordXynSignatureLine::get_comment_prefix("unknown"), "");
}

#[test]
fn test_is_invalid_xyn_signature() {
    assert!(LordXynSignatureLine::is_invalid_xyn_signature("// src/utils/lxsl.rs"));
    assert!(!LordXynSignatureLine::is_invalid_xyn_signature("// src/utils/lxsl.rs ~=#######D]======<Lord[UTILS]Xyn>====="));
}

#[test]
fn test_is_xyn_signature() {
    assert!(LordXynSignatureLine::is_xyn_signature("// src/utils/lxsl.rs ~=#######D]======<Lord[UTILS]Xyn>====="));
    assert!(!LordXynSignatureLine::is_xyn_signature("// src/utils/lxsl.rs"));
}

#[test]
fn test_should_skip_file() {
    assert!(LordXynSignatureLine::should_skip_file("Cargo.lock"));
    assert!(LordXynSignatureLine::should_skip_file("image.png"));
    assert!(!LordXynSignatureLine::should_skip_file("src/main.rs"));
}

// Note: The enforce_signature_at_line_1 function is not tested here as it involves file I/O.
// Consider adding integration tests or mocking the file system for thorough testing.