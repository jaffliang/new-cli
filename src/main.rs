#![deny(unsafe_code)]
use anyhow::{Context, Result};
use clap::Parser;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// 文件名 (默认: index)
    #[arg(default_value = "index")]
    filename: String,

    /// 文件后缀 (默认: html)
    #[arg(default_value = "html")]
    extension: String,
}

fn ensure_template_dir() -> Result<PathBuf> {
    // 获取用户主目录下的模板目录路径
    let template_dir = dirs::home_dir()
        .context("无法获取主目录")?
        .join(".new-cli")
        .join("template");

    // 如果模板目录不存在，创建它
    if !template_dir.exists() {
        fs::create_dir_all(&template_dir).context("无法创建模板目录")?;

        // Embed the default template content from the project's template/index.html at compile time
        let template_content = include_str!("../template/index.html");

        let target_template = template_dir.join("index.html");
        fs::write(&target_template, template_content).context("无法写入默认模板到用户目录")?;
    }

    Ok(template_dir)
}

fn get_default_editor() -> &'static str {
    if cfg!(target_os = "windows") {
        "notepad3" // Windows 默认使用 notepad
    } else if cfg!(target_os = "macos") {
        "open" // macOS 使用 open 命令
    } else {
        "xdg-open" // Linux 使用 xdg-open
    }
}

/// 查找模板文件
/// 如果指定的模板文件存在，则返回该文件路径
/// 如果不存在，尝试查找相同后缀的其他模板文件
/// 如果仍未找到，返回None
fn find_template_file(template_dir: &PathBuf, filename: &str, extension: &str) -> Option<PathBuf> {
    let canonical_template_dir = match fs::canonicalize(template_dir) {
        Ok(path) => path,
        Err(_) => return None, // Cannot canonicalize template_dir, unsafe to proceed
    };

    // 首先检查指定的模板文件是否存在并进行路径验证
    let specified_template_name = format!("{}.{}", filename, extension);
    let specified_template_path = template_dir.join(&specified_template_name);

    if specified_template_path.exists() {
        if let Ok(canonical_specified_path) = fs::canonicalize(&specified_template_path) {
            if canonical_specified_path.starts_with(&canonical_template_dir) {
                return Some(specified_template_path); // Return original path, not canonicalized one
            }
        }
        // If canonicalization fails or path is not within template_dir,
        // proceed to search other files (treat as if specific template not found securely)
    }

    // 如果指定模板不存在或不安全，查找相同后缀的任意文件
    if let Ok(entries) = fs::read_dir(template_dir) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == extension {
                        // Verify that this path is also within the template_dir
                        if let Ok(canonical_entry_path) = fs::canonicalize(&path) {
                            if canonical_entry_path.starts_with(&canonical_template_dir) {
                                return Some(path); // Return original path
                            }
                        }
                        // If canonicalization fails or path is not within template_dir, skip
                    }
                }
            }
        }
    }

    // 没有找到任何匹配后缀的模板
    None
}

// Public function for validating CLI inputs
pub fn validate_cli_inputs(filename: &str, extension: &str) -> Result<(), String> {
    let invalid_chars = ["/", "\\", ".."];
    for &char_set in &invalid_chars {
        if filename.contains(char_set) {
            return Err(format!(
                "错误：文件名 '{}' 包含无效字符 '{}'。",
                filename, char_set
            ));
        }
        if extension.contains(char_set) {
            return Err(format!(
                "错误：文件后缀 '{}' 包含无效字符 '{}'。",
                extension, char_set
            ));
        }
    }

    if filename.is_empty() {
        return Err("错误：文件名不能为空。".to_string());
    }

    if extension.is_empty() {
        return Err("错误：文件后缀不能为空。".to_string());
    }

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Validate filename and extension using the new function
    if let Err(e) = validate_cli_inputs(&cli.filename, &cli.extension) {
        eprintln!("{}", e);
        std::process::exit(1);
    }

    // 确保模板目录存在
    let template_dir = ensure_template_dir()?;

    // 获取模板内容
    let template_content = match find_template_file(&template_dir, &cli.filename, &cli.extension) {
        Some(template_path) => {
            // 找到了模板文件，读取其内容
            fs::read_to_string(&template_path)
                .with_context(|| format!("无法读取模板文件: {:?}", template_path))?
        },
        None => {
            // 没有找到任何匹配的模板文件，使用空内容
            println!("未找到模板 {}.{} 或任何 .{} 后缀的文件，将创建空文件", 
                    cli.filename, cli.extension, cli.extension);
            String::new()
        }
    };

    // 创建目标文件名
    let target_filename = format!("{}.{}", cli.filename, cli.extension);
    
    // --- Path validation for target file ---
    let current_dir = std::env::current_dir().context("无法获取当前目录")?;
    let canonical_current_dir = current_dir.canonicalize().context("无法规范化当前目录路径")?;
    
    let absolute_target_path = canonical_current_dir.join(&target_filename);

    // Ensure the target path is directly within the canonical current working directory
    if absolute_target_path.parent() != Some(canonical_current_dir.as_path()) {
        eprintln!(
            "错误：目标文件路径 '{:?}' 不在当前工作目录内。",
            absolute_target_path
        );
        std::process::exit(1);
    }
    // --- End of path validation ---

    // 写入新文件
    fs::write(&absolute_target_path, template_content)
        .with_context(|| format!("无法创建文件 {}", target_filename))?;

    println!("成功创建文件: {}", target_filename);

    // 使用默认编辑器打开新文件
    let editor = get_default_editor();
    match Command::new(editor)
        .arg(&absolute_target_path) // Use the validated absolute_target_path
        .spawn()
        .with_context(|| format!("无法使用 {} 打开文件", editor))
    {
        Ok(_) => println!("已使用 {} 打开文件", editor),
        Err(e) => println!("打开文件失败: {}", e),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_cli_inputs_valid() {
        assert!(validate_cli_inputs("index", "html").is_ok());
        assert!(validate_cli_inputs("my_file-123", "txt").is_ok());
    }

    #[test]
    fn test_validate_filename_empty() {
        let result = validate_cli_inputs("", "html");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "错误：文件名不能为空。");
    }

    #[test]
    fn test_validate_filename_invalid_chars() {
        let chars = ["/", "\\", ".."];
        for &char_set in &chars {
            let filename = format!("file{}", char_set);
            let result = validate_cli_inputs(&filename, "html");
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err(),
                format!(
                    "错误：文件名 '{}' 包含无效字符 '{}'。",
                    filename, char_set
                )
            );
        }
    }

    #[test]
    fn test_validate_extension_empty() {
        let result = validate_cli_inputs("index", "");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "错误：文件后缀不能为空。");
    }

    #[test]
    fn test_validate_extension_invalid_chars() {
        let chars = ["/", "\\", ".."];
        for &char_set in &chars {
            let extension = format!("ext{}", char_set);
            let result = validate_cli_inputs("index", &extension);
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err(),
                format!(
                    "错误：文件后缀 '{}' 包含无效字符 '{}'。",
                    extension, char_set
                )
            );
        }
    }

    #[test]
    fn test_validate_both_invalid_filename_takes_precedence() {
        // Test that filename error is reported first if both are invalid (due to order of checks)
        let result = validate_cli_inputs("file/", "ext/");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "错误：文件名 'file/' 包含无效字符 '/'。"
        );
    }
}
