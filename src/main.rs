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

        // 从项目模板目录复制默认模板
        // let default_template = PathBuf::from("template/index.html");
        // let template_content = fs::read_to_string(&default_template)
        //     .context("无法读取默认模板文件")?;
        let template_content = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <meta http-equiv="X-UA-Compatible" content="ie=edge" />
  <title>Document</title>
  <style></style>
</head>
<body>

</body>
<script>
  
</script>
</html>"#;

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
    // 首先检查指定的模板文件是否存在
    let specified_template = template_dir.join(format!("{}.{}", filename, extension));
    if specified_template.exists() {
        return Some(specified_template);
    }

    // 如果指定模板不存在，查找相同后缀的任意文件
    if let Ok(entries) = fs::read_dir(template_dir) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == extension {
                        return Some(path);
                    }
                }
            }
        }
    }

    // 没有找到任何匹配后缀的模板
    None
}

fn main() -> Result<()> {
    let cli = Cli::parse();

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
    let target_path = PathBuf::from(&target_filename);

    // 写入新文件
    fs::write(&target_path, template_content)
        .with_context(|| format!("无法创建文件 {}", target_filename))?;

    println!("成功创建文件: {}", target_filename);

    // 使用默认编辑器打开新文件
    let editor = get_default_editor();
    match Command::new(editor)
        .arg(&target_path)
        .spawn()
        .with_context(|| format!("无法使用 {} 打开文件", editor))
    {
        Ok(_) => println!("已使用 {} 打开文件", editor),
        Err(e) => println!("打开文件失败: {}", e),
    }

    Ok(())
}
