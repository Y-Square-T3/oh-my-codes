# 快速开始

本指南将引导你安装 OpenCode 并配置 oh-my-codes。

---

## 目录

- [第一部分：安装 OpenCode](#第一部分安装-opencode)
- [第二部分：安装 oh-my-codes](#第二部分安装-oh-my-codes)
- [第三部分：配置 oh-my-codes](#第三部分配置-oh-my-codes)
- [后续步骤](#后续步骤)

---

## 第一部分：安装 OpenCode

OpenCode 是一个开源的 AI 编程助手，支持终端界面、桌面应用和 IDE 扩展。oh-my-codes 需要先安装 OpenCode。

### 前提条件

- **Node.js**（v18 或更高版本）或现代终端模拟器
- LLM 提供商的 API 密钥（或使用 [OpenCode Zen](https://opencode.ai/zen) 获取精选模型）

### 通过 curl 安装

最快的安装方式：

```bash
curl -fsSL https://opencode.ai/install | bash
```

### 通过 npm 安装

```bash
npm install -g opencode-ai
```

### 验证安装

```bash
opencode --version
```
---

## 第二部分：安装 oh-my-codes

oh-my-codes 为 OpenCode 扩展以下功能：

- **账户与工作区管理** — 登录多个工作区，在它们之间切换
- **Token 用量追踪** — 监控和推送本地 Token 用量数据

你可以通过 `npx` 按需运行 oh-my-codes，也可以全局安装。

### 无需安装直接运行（npx）

无需安装，直接使用 `npx`：

```bash
npx oh-my-codes install
```

### 全局安装

如果经常使用，建议全局安装 oh-my-codes，这样可以在任何地方使用：

```bash
npm install -g oh-my-codes
```

安装完成后，你可以使用完整名称 `oh-my-codes` 或简短别名 `omc`：

```bash
omc install
omc account login <服务器地址>
```

> **提示：** 全局安装 oh-my-codes 后会自动获得 `omc` 别名。在下面的所有示例中，你可以将 `omc` 与 `oh-my-codes` 互换使用。

---

## 第三部分：配置 oh-my-codes

### 方式 A：交互式安装器（推荐）

在终端中运行交互式安装器：

```bash
omc install
```

安装器将引导你完成：

1. **账户登录** — 通过设备码流程认证（会打开浏览器）
2. **工作区选择** — 选择要激活的工作区
3. **插件注册** — 自动将 oh-my-codes 添加到你的 OpenCode 配置中

如果你已有账户，可以跳过登录步骤：

```bash
omc install --skip-login
```

### 方式 B：手动设置

在 OpenCode 配置文件的 `plugin` 数组中添加 `oh-my-codes@latest`。

配置文件位于 `~/.config/opencode/opencode.json`（或 `opencode.jsonc`）：

```jsonc
{
  "plugin": [
    "oh-my-codes@latest"
  ]
}
```

然后登录你的账户：

```bash
omc account login <服务器地址>
```

系统会提示你在浏览器中打开一个 URL 并输入设备码。授权后，选择一个工作区进行激活。

---

## 账户管理

oh-my-codes 提供 CLI 命令来管理你的账户和工作区。

### 登录

```bash
omc account login <服务器地址>
```

通过设备码流程进行认证。登录成功后，你可以选择一个工作区。

### 登出

```bash
omc account logout [邮箱]
```

从指定账户（通过邮箱）或当前活跃账户登出。

### 切换工作区

```bash
omc account switch
```

在已登录的工作区之间进行交互式切换。

### 列出账户

```bash
omc account list
```

显示所有已登录的账户及其活跃工作区。

---

## 模型管理

oh-my-codes 可以发现和管理通过已连接账户提供的模型。

### 列出模型

```bash
omc model list
omc model list --provider <提供商ID>
omc model list --json
```

列出从你账户的模型 API 中获取的所有可用模型。

### 刷新模型

```bash
omc model refresh
```

从你账户的 API 获取最新的模型列表。

### 清除模型

```bash
omc model clear
omc model clear --provider <提供商ID>
```

清除活跃账户的缓存模型。

---

## Token 用量追踪

oh-my-codes 会自动在本地追踪你的 Token 用量。你可以查看和推送这些数据。

### 查看状态

```bash
omc token-usages
omc tu status
```

显示本地缓存中未推送的用量记录数。

### 推送用量数据

```bash
omc token-usages push
omc tu push --json
```

将所有缓存的用量记录推送到你的账户服务器。

---

## 在 OpenCode 中使用

配置好 oh-my-codes 后，你可以在 OpenCode 中直接使用以下命令：

- **`/omc-login`** — 在 OpenCode 内登录 OMC 账户
- **`/omc-switch`** — 不离开 OpenCode 即可切换活跃工作区

这些命令使用与 CLI 相同的设备码流程和工作区选择方式。

---

## 配置文件参考

| 文件 | 位置 |
|------|------|
| OpenCode 配置 | `~/.config/opencode/opencode.json` 或 `opencode.jsonc` |
| oh-my-codes 数据 | 内部托管（SQLite 数据库） |

你可以通过设置 `OPENCODE_CONFIG_DIR` 来覆盖配置目录：

```bash
export OPENCODE_CONFIG_DIR=/path/to/custom/config
```

---

## 后续步骤

- 浏览 [OpenCode 文档](https://opencode.ai/docs) 了解更多 OpenCode 的使用方法
