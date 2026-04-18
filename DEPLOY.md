# EidosInsight 打包部署说明

## 版本信息

- **应用名称**：EidosInsight
- **版本号**：0.1.0
- **标识符**：com.eidosinsight.desktop
- **目标平台**：Windows x64

---

## 一、构建环境要求

| 依赖 | 版本要求 | 用途 |
|------|---------|------|
| Rust | 1.95.0+ (stable) | 后端编译 |
| Node.js | 18+ | 前端构建 |
| pnpm | 8+ | 包管理器 |
| Windows SDK | 10+ | NSIS 打包 |
| WebView2 | Edge 内置或独立安装 | 运行时渲染引擎 |

### 关键 Rust 依赖

- `tauri = "2"` — 桌面应用框架
- `rusqlite = "0.31"` (bundled) — 内嵌 SQLite，无需额外安装
- `keyring = "3"` (windows-native) — Windows Credential Store 集成
- `lopdf = "0.34"` — PDF 解析（纯 Rust，无 Python 依赖）
- `reqwest = "0.12"` (rustls-tls) — HTTP 客户端，不依赖系统 OpenSSL

---

## 二、打包步骤

### 1. 确认环境

```powershell
# 验证 Rust 工具链
rustc --version    # 应输出 1.95.0+
cargo --version    # 应输出 1.95.0+

# 验证 Node.js 和 pnpm
node --version     # 应输出 v18+
pnpm --version     # 应输出 8+
```

### 2. 安装前端依赖

```powershell
cd EidosInsight
pnpm install
```

### 3. 执行完整打包

```powershell
pnpm tauri build
```

此命令会依次执行：
1. `pnpm build` → 前端 TypeScript 编译 + Vite 生产构建
2. `cargo build --release` → Rust 后端编译（约 2-3 分钟）
3. `makensis` → 生成 NSIS 安装包

### 4. 产物位置

```
src-tauri/target/release/
├── eidosinsight.exe                          # 独立可执行文件 (~9.6 MB)
├── eidosinsight-native-host.exe              # Chrome Native Messaging Host
└── bundle/nsis/
    └── EidosInsight_0.1.0_x64-setup.exe      # NSIS 安装包 (~4.4 MB)
```

### 5. 构建选项

```powershell
# 仅构建可执行文件，不打包安装程序
pnpm tauri build --no-bundle

# 指定打包格式（默认 nsis）
pnpm tauri build --bundles nsis

# 调试模式运行（不打包）
pnpm tauri dev
```

---

## 三、安装部署

### 方式一：NSIS 安装包（推荐）

1. 运行 `EidosInsight_0.1.0_x64-setup.exe`
2. 选择安装语言（简体中文 / English）
3. 选择安装路径（默认 `C:\Users\<用户>\AppData\Local\EidosInsight`）
4. 勾选是否创建桌面快捷方式
5. 点击"安装"等待完成
6. 启动应用

### 方式二：便携版

直接运行 `eidosinsight.exe`，无需安装。数据文件存储在：

```
%APPDATA%\com.eidosinsight.desktop\
├── eidosinsight.db      # SQLite 数据库（论文、笔记、标签等）
└── eidosinsight.db-wal  # WAL 日志
```

### WebView2 运行时

应用依赖 Microsoft Edge WebView2 运行时。如果系统未安装：

- Windows 10/11 通常已内置
- 安装包会自动提示下载 WebView2 Bootstrapper
- 也可手动安装：https://developer.microsoft.com/en-us/microsoft-edge/webview2/

---

## 四、首次启动配置（Onboarding）

应用首次启动会进入引导流程：

1. **选择 AI 模式** — 本地模式（Ollama）或云端模式（DeepSeek / OpenAI）
2. **配置本地 LLM** — 如选本地模式，输入 Ollama 地址（默认 `http://localhost:11434`）
3. **配置云端 API Key** — 如选云端模式，输入 API Key 并测试连接
4. **完成** — 进入主界面

### API Key 存储

API Key 通过 Windows Credential Store 安全存储（keyring windows-native），不写入数据库或配置文件。

---

## 五、功能验证清单

安装完成后，按以下步骤验证核心功能：

### 基础功能

| # | 验证项 | 操作步骤 | 预期结果 |
|---|--------|---------|---------|
| 1 | 应用启动 | 双击桌面图标或开始菜单 | 主界面正常显示，侧边栏可见 |
| 2 | 主题切换 | 设置 → 外观 → 深色/浅色/跟随系统 | 主题即时切换 |
| 3 | 导入论文 | 点击"导入" → 选择 PDF 文件 | 文件导入成功，论文卡片出现 |
| 3a | 批量导入 | 点击"导入" → 选择多个 PDF | 批量导入，进度条显示 |
| 3b | 拖放导入 | 拖 PDF 文件到窗口 | 文件导入成功 |

### 论文管理

| # | 验证项 | 操作步骤 | 预期结果 |
|---|--------|---------|---------|
| 4 | 文件夹分类 | 右键文件夹 → 新建 → 拖入论文 | 论文归入文件夹 |
| 5 | 标签管理 | 论文详情 → 添加标签 | 标签创建并关联 |
| 6 | 收藏 | 论文卡片 → 星标 | 收藏状态切换 |
| 7 | 搜索 | 搜索栏输入关键词 | FTS5 全文检索返回结果 |
| 8 | 排序筛选 | 库页面 → 排序/筛选下拉 | 列表按条件重排 |

### PDF 阅读

| # | 验证项 | 操作步骤 | 预期结果 |
|---|--------|---------|---------|
| 9 | 打开阅读器 | 点击论文 → 打开 PDF | PDF 渲染正常，页面导航可用 |
| 10 | 翻页/缩放 | 阅读器工具栏操作 | 页面切换流畅，缩放正确 |
| 11 | 阅读进度 | 滚动浏览后返回列表 | 进度百分比更新 |

### 笔记系统

| # | 验证项 | 操作步骤 | 预期结果 |
|---|--------|---------|---------|
| 12 | 创建笔记 | 阅读器侧栏 → 新建笔记 | Markdown 编辑器打开 |
| 13 | 笔记标注 | 选中文本 → 创建标注 | 标注关联到 PDF 页码 |
| 14 | 双向链接 | 编辑器输入 `[[` | 触发笔记链接自动补全 |
| 15 | 反向链接 | 打开被引用的笔记 | 底部显示反向链接列表 |
| 16 | 笔记搜索 | 全局搜索 → 笔记 Tab | 笔记全文检索，高亮匹配 |

### AI 助手

| # | 验证项 | 操作步骤 | 预期结果 |
|---|--------|---------|---------|
| 17 | 术语解释 | AI 面板 → 输入术语 → 解释 | AI 流式返回解释内容 |
| 18 | 摘要生成 | AI 面板 → 生成摘要 | 论文摘要生成 |
| 19 | Insight 提取 | AI 面板 → 提取 Insight | 关键洞察提取 |
| 20 | 连接测试 | 设置 → AI 配置 → 测试连接 | 连接成功/失败明确提示 |

### 自动化

| # | 验证项 | 操作步骤 | 预期结果 |
|---|--------|---------|---------|
| 21 | 创建定时任务 | 自动化 → 新建任务 → 设置计划 | 任务创建并激活 |
| 22 | 技能创建 | 自动化 → 技能 → 创建技能 | 3 步向导生成技能定义 |
| 23 | 执行历史 | 自动化 → 历史记录 | 显示执行记录和状态 |

---

## 六、常见问题

### Q: 启动后白屏
A: 检查 WebView2 是否安装。在 PowerShell 运行 `Get-ItemProperty 'HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}'` 确认。

### Q: API Key 保存后测试连接失败
A: 确认 API Key 正确。EidosInsight 使用 Windows Credential Store 存储 Key，如果之前安装过旧版本，可能在 Credential Manager 中有残留空 Key，需手动清除：
1. 打开"控制面板 → 凭据管理器 → Windows 凭据"
2. 查找 `com.eidosinsight.deepseek` 或 `com.eidosinsight.openai`
3. 删除后重新在应用中保存

### Q: PDF 无法渲染
A: PDF 通过 base64 传输到前端 pdfjs-dist 渲染。大文件（>50MB）可能需要较长加载时间。确认 `pdfjs-dist` worker 正常加载（开发者工具 → Network → 无 404）。

### Q: Ollama 连接失败
A: 确认 Ollama 服务已启动（`ollama serve`），默认地址 `http://localhost:11434`。如使用自定义地址，在设置中修改。

### Q: 数据库位置
A: `%APPDATA%\com.eidosinsight.desktop\eidosinsight.db`。可直接用 SQLite 工具查看。

---

## 七、卸载

- **NSIS 安装版**：控制面板 → 程序与功能 → 卸载 EidosInsight
- **便携版**：删除 `eidosinsight.exe` 即可
- **数据清理**：删除 `%APPDATA%\com.eidosinsight.desktop\` 目录
- **API Key 清理**：凭据管理器 → 删除 `com.eidosinsight.*` 条目

---

## 八、构建配置参考

### tauri.conf.json 关键配置

```json
{
  "productName": "EidosInsight",
  "version": "0.1.0",
  "identifier": "com.eidosinsight.desktop",
  "bundle": {
    "targets": ["nsis"],
    "windows": {
      "webviewInstallMode": { "type": "downloadBootstrapper" },
      "nsis": {
        "languages": ["SimpChinese", "English"],
        "displayLanguageSelector": true
      }
    }
  }
}
```

### Cargo.toml Release 优化

```toml
[profile.release]
panic = "abort"       # 减少 panic 处理代码
codegen-units = 1     # 单编译单元，更好优化
lto = true            # 链接时优化
opt-level = "s"       # 优化体积
strip = true          # 去除调试符号
```

---

*文档生成时间：2026-04-18*
*EidosInsight v0.1.0*
