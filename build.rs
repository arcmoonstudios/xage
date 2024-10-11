// build.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[BUILD]Xyn>=====S===t===u===d===i===o===s======[R|$>

use std::path::{Path, Component};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use walkdir::WalkDir;
use anyhow::Result;
use std::env;

fn main() -> Result<()> {
    println!("cargo:rerun-if-env-changed=CONFIG_PATH");

    let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "config/default.toml".to_string());

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let dest_path = Path::new(&out_dir).join("config.rs");
    let mut f = File::create(&dest_path).expect("Could not create config.rs");
    writeln!(f, "pub const CONFIG_PATH: &str = \"{}\";", config_path)
        .expect("Could not write to config.rs");

    add_custom_headers(&env::current_dir()?)?;

    Ok(())
}

fn add_custom_headers(project_path: &Path) -> Result<()> {
    for entry in WalkDir::new(project_path)
        .into_iter()
        .filter_entry(|e| !e.file_name().to_str().map(|s| s.starts_with('.')).unwrap_or(false))
    {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");
            if let Some(header) = generate_custom_header(project_path, path, extension) {
                add_header_to_file(path, &header)?; // Add header to file (enforces on line 1)
            }
        }
    }
    Ok(())
}

fn generate_custom_header(project_root: &Path, path: &Path, extension: &str) -> Option<String> {
    // Ignore certain extensions
    if extension == "toml" {
        return None;
    }

    // Get the relative path
    let relative_path = path.strip_prefix(project_root).unwrap_or(path);
    let components: Vec<_> = relative_path.components().collect();

    // Generate the module name
    let module_name = if components.is_empty() {
        "UNKNOWN".to_string()
    } else if components.len() == 1 {
        // Files in the root directory
        let file_name = components[0].as_os_str().to_str().unwrap_or("UNKNOWN");
        if file_name == "build.rs" {
            "BUILD".to_string()
        } else {
            file_name.split('.').next().unwrap_or("UNKNOWN").to_uppercase()
        }
    } else if components.len() == 2 && components[0].as_os_str() == "src" {
        // Files directly under src/
        components[1].as_os_str().to_str().unwrap_or("UNKNOWN").split('.').next().unwrap_or("UNKNOWN").to_uppercase()
    } else {
        // Files in subdirectories
        components.iter().rev().nth(1)
            .and_then(|c| match c {
                Component::Normal(name) => name.to_str(),
                _ => None,
            })
            .unwrap_or("UNKNOWN")
            .to_uppercase()
    };

    let module_name = module_name.replace('_', "-");

    // Determine the comment syntax based on file extension
    let comment_syntax = match extension {
        "rs" | "js" | "ts" | "jsx" | "tsx" | "css" | "scss" => "//",
        "py" | "sh" | "bash" | "md" => "#",
        "html" | "xml" => "<!--",
        "sql" | "txt" => "--",
        _ => return None,
    };

    let display_path = relative_path.display().to_string();

    // Return the custom header string
    Some(format!(
        "{} {} ~=#######D]======A===r===c====M===o===o===n=====<Lord[{}]Xyn>=====S===t===u===d===i===o===s======[R|$>\n",
        comment_syntax,
        display_path,
        module_name
    ))
}

fn add_header_to_file(path: &Path, new_header: &str) -> Result<()> {
    let mut content = String::new();
    {
        let mut file = File::open(path)?;
        file.read_to_string(&mut content)?;
    }

    let lines: Vec<&str> = content.lines().collect();
    let mut new_lines = Vec::new();
    let mut header_added = false;

    // Enforce header is added on line 1
    if !lines.is_empty() {
        if is_header(lines[0]) {
            // If the first line already contains a valid header, we don't add a new one.
            new_lines.push(lines[0]);
            header_added = true;
        } else {
            // Add the new header to the first line
            new_lines.push(new_header.trim_end());
            header_added = true;
        }
    }

    // Append the rest of the content (excluding existing signatures in the first 10 lines)
    for (i, &line) in lines.iter().enumerate() {
        if !is_header(line) || i >= 10 {
            new_lines.push(line);
        }
    }

    if !header_added {
        // If no header has been added, insert it at the beginning of the file
        new_lines.insert(0, new_header.trim_end());
    }

    // Write the new content to the file
    let new_content = new_lines.join("\n");
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(path)?;
    file.write_all(new_content.as_bytes())?;

    Ok(())
}

fn is_header(line: &str) -> bool {
    // Check for specific header components
    (line.contains("//src") || line.contains("// src") || line.contains("// build") || line.contains("# README") || line.contains("-- X") || line.contains("// tests")) &&
    line.contains("~=#######D]======A===r===c====M===o===o===n=====<Lord[") &&
    line.contains("]Xyn>=====S===t===u===d===i===o===s======[R|$>")
}