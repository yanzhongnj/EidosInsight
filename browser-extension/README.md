# EidosInsight 浏览器扩展安装说明

## 安装方式

### 方式一：通过应用内安装（推荐）

1. 启动 EidosInsight 桌面应用
2. 进入 **设置 → 浏览器扩展**，点击"安装连接组件"
3. 在 Chrome 中访问 `chrome://extensions/`
4. 开启"开发者模式"
5. 点击"加载已解压的扩展程序"，选择 `browser-extension/` 目录

### 方式二：手动注册 Native Host

**Windows**：
```powershell
# 1. 将 manifest 写入 AppData
$manifestDir = "$env:LOCALAPPDATA\EidosInsight"
New-Item -ItemType Directory -Force -Path $manifestDir

$hostExe = "C:\path\to\EidosInsight\eidosinsight-native-host.exe"  # 修改为实际路径
$manifest = @{
    name = "com.eidosinsight.desktop"
    description = "EidosInsight Native Messaging Host"
    path = $hostExe
    type = "stdio"
    allowed_origins = @("chrome-extension://YOUR_EXTENSION_ID/")
} | ConvertTo-Json

$manifestPath = "$manifestDir\native_host_manifest.json"
$manifest | Out-File -FilePath $manifestPath -Encoding UTF8

# 2. 写入注册表
$regKey = "HKCU:\Software\Google\Chrome\NativeMessagingHosts\com.eidosinsight.desktop"
New-Item -Path $regKey -Force
Set-ItemProperty -Path $regKey -Name "(Default)" -Value $manifestPath
```

**macOS**：
```bash
HOST_EXE="/Applications/EidosInsight.app/Contents/MacOS/eidosinsight-native-host"
MANIFEST_DIR="$HOME/Library/Application Support/Google/Chrome/NativeMessagingHosts"
mkdir -p "$MANIFEST_DIR"

cat > "$MANIFEST_DIR/com.eidosinsight.desktop.json" << EOF
{
  "name": "com.eidosinsight.desktop",
  "description": "EidosInsight Native Messaging Host",
  "path": "$HOST_EXE",
  "type": "stdio",
  "allowed_origins": ["chrome-extension://YOUR_EXTENSION_ID/"]
}
EOF
```

**Linux**：
```bash
HOST_EXE="/opt/eidosinsight/eidosinsight-native-host"
MANIFEST_DIR="$HOME/.config/google-chrome/NativeMessagingHosts"
mkdir -p "$MANIFEST_DIR"

cat > "$MANIFEST_DIR/com.eidosinsight.desktop.json" << EOF
{
  "name": "com.eidosinsight.desktop",
  "description": "EidosInsight Native Messaging Host",
  "path": "$HOST_EXE",
  "type": "stdio",
  "allowed_origins": ["chrome-extension://YOUR_EXTENSION_ID/"]
}
EOF
```

## 使用

安装完成后，在任意学术论文页面：
- 点击 Chrome 工具栏中的 EidosInsight 图标
- 或按快捷键 `Alt+S`

支持的网站：
- arXiv (arxiv.org)
- ACM Digital Library (dl.acm.org)
- IEEE Xplore (ieeexplore.ieee.org)
- Semantic Scholar (semanticscholar.org)
- OpenReview (openreview.net)
- NeurIPS (papers.nips.cc)
- ACL Anthology (aclanthology.org)
- PubMed (pubmed.ncbi.nlm.nih.gov)
- Springer Link (link.springer.com)
- ScienceDirect (sciencedirect.com)
- 以及任何使用 citation_* meta 标签的学术网站

## 扩展目录结构

```
browser-extension/
├── manifest.json          # Manifest V3 配置
├── popup/
│   ├── popup.html         # 弹出窗口 HTML
│   ├── popup.css          # 样式（深色主题）
│   └── popup.js           # 弹出窗口逻辑
├── content/
│   └── content_script.js  # 页面元数据提取
├── background/
│   └── service_worker.js  # 后台服务 (Native Messaging)
└── icons/
    ├── icon16.png
    ├── icon32.png
    ├── icon48.png
    └── icon128.png
```
