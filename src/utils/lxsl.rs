// src/utils/lxsl.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[UTILS]Xyn>=====S===t===u===d===i===o===s======[R|$>

use crate::constants::ARCMOON_SIGNATURE;
use std::fs::OpenOptions;
use std::io::{self, BufRead, Write};
use std::path::Path;

pub struct LordXynSignatureLine;

impl LordXynSignatureLine {
    pub fn generate_signature_line(file_path: &str) -> String {
        let normalized_path = Path::new(file_path)
            .to_str()
            .unwrap_or(file_path)
            .replace('\\', "/");

        let path_parts: Vec<&str> = normalized_path.split('/').collect();

        let extension = Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        let comment_prefix = Self::get_comment_prefix(extension);

        if comment_prefix.is_empty() {
            return String::new();
        }

        let signature_path = Self::build_signature_path(&path_parts);
        let xyn_signature = Self::build_xyn_signature(&path_parts);

        format!(
            "{} {} {}",
            comment_prefix,
            signature_path,
            ARCMOON_SIGNATURE.replace("{}", &xyn_signature)
        )
    }

    pub fn enforce_signature_at_line_1(file_path: &str) -> io::Result<()> {
        let path = Path::new(file_path);

        if Self::should_skip_file(file_path) {
            return Ok(());
        }

        let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
        let comment_prefix = Self::get_comment_prefix(extension);

        if comment_prefix.is_empty() {
            return Ok(());
        }

        let file = OpenOptions::new().read(true).open(&path)?;
        let lines: Vec<String> = io::BufReader::new(file).lines().collect::<Result<_, _>>()?;

        let cleaned_lines: Vec<String> = lines
            .into_iter()
            .enumerate()
            .filter_map(|(idx, line)| {
                if idx < 10 && (Self::is_invalid_xyn_signature(&line) || Self::is_xyn_signature(&line)) {
                    None
                } else {
                    Some(line)
                }
            })
            .collect();

        let signature = Self::generate_signature_line(file_path);
        let mut file = OpenOptions::new().write(true).truncate(true).open(path)?;

        if !signature.is_empty() {
            writeln!(file, "{}", signature)?;
        }

        for line in cleaned_lines {
            writeln!(file, "{}", line)?;
        }

        Ok(())
    }

    pub fn build_signature_path(path_parts: &[&str]) -> String {
        if path_parts.len() > 1 {
            path_parts
                .iter()
                .take(path_parts.len() - 1)
                .map(|comp| format!("{}/", comp))
                .collect()
        } else {
            String::new()
        }
    }

    pub fn build_xyn_signature(path_parts: &[&str]) -> String {
        path_parts
            .last()
            .map(|last_part| {
                Path::new(last_part)
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .unwrap_or("UNKNOWN")
                    .to_uppercase()
                    .replace('_', "-")
            })
            .unwrap_or_else(|| "UNKNOWN".to_string())
    }

    pub fn get_comment_prefix(extension: &str) -> &str {
        match extension {
            "rs" | "js" | "ts" | "cpp" | "c" | "java" => "//",
            "py" | "sh" | "rb" | "pl" => "#",
            "html" | "xml" => "<!--",
            "css" | "scss" => "/*",
            "sql" | "txt" => "--",
            _ => "",
        }
    }

    pub fn is_invalid_xyn_signature(line: &str) -> bool {
        let has_valid_path = line.contains("// src") || line.contains("// build") || line.contains("// tests");
        let has_signature_format = line.contains("~=#######D]") && line.contains("<Lord[") && line.contains("]Xyn>");
        has_valid_path && !has_signature_format
    }

    pub fn is_xyn_signature(line: &str) -> bool {
        let has_valid_path = line.contains("// src") || line.contains("// build") || line.contains("// tests");
        let has_signature_format = line.contains("~=#######D]") && line.contains("<Lord[") && line.contains("]Xyn>");
        has_valid_path && has_signature_format
    }

    pub fn should_skip_file(file_path: &str) -> bool {
        let skip_extensions = [
            "lock", "log", "png", "jpg", "jpeg", "gif", "pyc", "toml", "exe", "dll", "so", "bin",
        ];

        Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .map_or(false, |ext| skip_extensions.contains(&ext))
    }
}