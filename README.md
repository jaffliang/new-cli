# new-cli

一个简单而强大的文件模板生成工具，帮助您快速创建各类文件。

## 功能特点

- 🚀 快速生成各类文件模板
- 📝 支持自定义模板
- 🔍 智能模板查找
- 📂 自动使用系统默认编辑器打开新创建的文件
- 🎨 内置默认HTML模板

## 安装

### 从源码安装

确保您已安装 Rust 开发环境，然后执行：

```bash
git clone https://github.com/yourusername/new-cli
cd new-cli
cargo install --path .
```

## 使用方法

基本命令格式：

```bash
new-cli [文件名] [文件后缀]
```

如果不指定参数，将默认创建 `index.html` 文件。

### 命令行参数

- `filename`: 文件名（默认：index）
- `extension`: 文件后缀（默认：html）

### 示例

1. 创建 HTML 文件：
```bash
new-cli index html
```

2. 创建 JavaScript 文件：
```bash
new-cli main js
```

3. 创建文本文件：
```bash
new-cli notes txt
```

## 模板系统

### 模板位置
模板文件存储在用户主目录下的 `.new-cli/template` 文件夹中：
- Windows: `C:\Users\<用户名>\.new-cli\template`
- macOS: `/Users/<用户名>/.new-cli/template`
- Linux: `/home/<用户名>/.new-cli/template`

### 模板查找逻辑

1. 首先查找与指定文件名完全匹配的模板
2. 如果未找到，则查找相同后缀的任意模板文件
3. 如果仍未找到，则创建一个空文件

### 自定义模板

您可以在模板目录中添加自己的模板文件。例如：
- `index.html` - HTML模板
- `main.js` - JavaScript模板
- `style.css` - CSS模板

## 系统要求

- Windows/macOS/Linux 操作系统
- Rust 1.56.0 或更高版本

## 注意事项

- Windows 系统默认使用 Notepad3 作为编辑器
- macOS 系统使用 `open` 命令打开文件
- Linux 系统使用 `xdg-open` 打开文件

## 贡献

欢迎提交 Issue 和 Pull Request！

## 许可证

MIT License

<div align="center">

[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)
[![GitHub license](https://img.shields.io/github/license/your-username/new-cli)](https://github.com/your-username/new-cli/blob/main/LICENSE)
[![Crates.io](https://img.shields.io/crates/v/new-cli.svg)](https://crates.io/crates/new-cli)

一个快速创建文件模板的命令行工具，支持自定义模板和多平台。

[English](./README_EN.md) | 简体中文
