# Paper Format Checker

论文格式检查桌面应用 - 使用 AI 自动检查论文格式是否符合要求

## 功能特点

- 📄 **文档解析** - 支持解析 Word (.docx) 和 PDF 格式的论文文件
- 🤖 **AI 智能检查** - 使用 LLM API 自动分析论文格式问题
- ⚙️ **灵活配置** - 支持 MiniMax 和 OpenAI 多种 LLM 提供商
- 📊 **详细报告** - 提供问题分类、严重程度和修改建议

## 系统要求

- macOS 10.15+ 或 Windows 10+
- 至少 4GB 内存

## 安装

### 方式一：下载预编译版本

从 [Releases](https://github.com/yourusername/paper-format-checker/releases) 下载对应平台的安装包：

- macOS: `.dmg` 文件
- Windows: `.exe` 文件

### 方式二：源码编译

```bash
# 克隆仓库
git clone https://github.com/yourusername/paper-format-checker.git
cd paper-format-checker

# 安装依赖
npm install

# 构建应用
npm run tauri build
```

## 使用说明

### 1. 配置 API Key

首次使用需要配置 LLM API：

1. 点击界面右上角的 ⚙️ **设置** 按钮
2. 选择提供商（MiniMax 或 OpenAI）
3. 输入 API Key
4. 选择要使用的模型
5. 点击保存

### 2. 上传文件

1. 点击"选择格式要求文件"上传学校的论文格式规范文档
2. 点击"选择论文文件"上传你需要检查的论文

### 3. 开始检查

点击"开始检查"按钮，应用会：
- 解析论文内容
- 调用 LLM 分析格式问题
- 生成格式检查报告

### 4. 查看结果

检查结果会显示：
- 发现的问题数量
- 问题分类（严重/重要/次要）
- 每项问题的详细描述和修改建议

## 开发技术

- **前端**: React + TypeScript + Tailwind CSS
- **后端**: Tauri 2.x (Rust)
- **文档解析**: zip, quick-xml, lopdf
- **AI 集成**: MiniMax / OpenAI API

## 项目结构

```
paper-format-checker/
├── src/                    # React 前端源码
│   ├── App.tsx            # 主应用组件
│   └── main.tsx           # 入口文件
├── src-tauri/             # Rust 后端源码
│   ├── src/
│   │   ├── lib.rs         # 主逻辑
│   │   ├── docx_parser.rs # Word 解析
│   │   ├── pdf_parser.rs  # PDF 解析
│   │   └── llm.rs         # LLM API 集成
│   └── Cargo.toml         # Rust 依赖
└── package.json           # Node 依赖
```

## 配置 LLM

### MiniMax 配置

- 模型: `abab6.5s-chat` 或 `abab6.5-chat`

### OpenAI 配置

- 模型: `gpt-4o-mini` 或 `gpt-4o`

## 许可证

MIT License