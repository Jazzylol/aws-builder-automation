# Kiro 账号注册和获取 RefreshToken 完整流程

## 概述

本项目实现了 Kiro 账号的自动化注册，并在注册完成后自动获取 AWS SSO RefreshToken（以 `aor` 开头），用于导入到 kiro-account-manager 进行账号管理。

## 前置条件

### 账号准备
每个待注册账号需要以下信息：
- **email**: Outlook 邮箱地址
- **email_password**: 邮箱密码（用于 IMAP 获取验证码）
- **client_id**: Microsoft OAuth2 Client ID
- **refresh_token**: Microsoft OAuth2 Refresh Token（用于 IMAP 认证）

### Microsoft OAuth2 配置
需要在 Azure Portal 注册应用，获取：
- Client ID
- Refresh Token（具有 `IMAP.AccessAsUser.All` 和 `offline_access` 权限）

---

## 完整流程图

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           1. 浏览器自动化注册流程                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐                  │
│  │ 打开 Kiro    │───▶│ 点击 Builder │───▶│ 输入邮箱     │                  │
│  │ 登录页面     │    │ ID 按钮      │    │ 点击继续     │                  │
│  └──────────────┘    └──────────────┘    └──────────────┘                  │
│                                                 │                           │
│                                                 ▼                           │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐                  │
│  │ 输入验证码   │◀───│ IMAP 获取    │◀───│ 等待验证码   │                  │
│  │ 点击继续     │    │ 验证码       │    │ 页面加载     │                  │
│  └──────────────┘    └──────────────┘    └──────────────┘                  │
│         │                   ▲                                               │
│         │                   │ 检查 INBOX + Junk 文件夹                      │
│         ▼                                                                   │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐                  │
│  │ 输入姓名     │───▶│ 设置密码     │───▶│ 注册完成     │                  │
│  │ 点击继续     │    │ 点击继续     │    │              │                  │
│  └──────────────┘    └──────────────┘    └──────────────┘                  │
│                                                 │                           │
└─────────────────────────────────────────────────┼───────────────────────────┘
                                                  │
                                                  ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                        2. SSO 授权获取 RefreshToken                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐                  │
│  │ 注册 OIDC    │───▶│ 发起设备     │───▶│ 获取授权 URL │                  │
│  │ 客户端       │    │ 授权请求     │    │ 和 User Code │                  │
│  └──────────────┘    └──────────────┘    └──────────────┘                  │
│         │                                       │                           │
│         │ 获取 clientId + clientSecret          │                           │
│         ▼                                       ▼                           │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐                  │
│  │ 轮询获取     │◀───│ 点击确认     │◀───│ 浏览器打开   │                  │
│  │ Token        │    │ 授权按钮     │    │ 授权页面     │                  │
│  └──────────────┘    └──────────────┘    └──────────────┘                  │
│         │                                                                   │
│         │ 获取 accessToken (aoa) + refreshToken (aor)                       │
│         ▼                                                                   │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │ 返回结果: password|||refreshToken|||clientId|||clientSecret          │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 详细步骤说明

### 第一阶段：浏览器自动化注册

#### 1.1 启动浏览器
- 使用 `headless_chrome` 库启动 Chrome 浏览器
- 应用指纹保护脚本（Canvas、WebGL、AudioContext 等）
- 设置随机窗口大小和 User-Agent

#### 1.2 导航到登录页面
```
URL: https://app.kiro.dev/signin
```

#### 1.3 点击 Builder ID 按钮
- XPath: `/html/body/div[2]/div/div[1]/main/div/div/div/div/div/div/div/div[1]/button[3]`

#### 1.4 输入邮箱
- 等待邮箱输入页面加载
- 输入邮箱地址
- 点击继续按钮

#### 1.5 获取验证码（IMAP）
- 等待验证码页面加载
- 等待 8 秒让邮件到达
- 通过 IMAP + OAuth2 连接 Outlook 邮箱
- **同时检查 INBOX 和 Junk 文件夹**（AWS 验证码邮件可能在垃圾箱）
- 查找来自 Amazon/AWS 的验证邮件
- 提取 6 位数字验证码

#### 1.6 输入验证码
- 在页面输入框中填入验证码
- 点击继续按钮

#### 1.7 输入姓名
- 等待姓名输入页面
- 输入随机生成的姓名
- 点击继续按钮

#### 1.8 设置密码
- 等待密码设置页面
- 生成 16 位随机安全密码
- 输入密码和确认密码
- 点击继续完成注册

---

### 第二阶段：SSO 授权获取 RefreshToken

注册完成后，复用当前浏览器 Tab（已登录状态），执行 SSO 授权：

#### 2.1 注册 OIDC 客户端
```
POST https://oidc.us-east-1.amazonaws.com/client/register

请求体:
{
  "clientName": "Kiro Account Manager",
  "clientType": "public",
  "scopes": [
    "codewhisperer:analysis",
    "codewhisperer:completions",
    "codewhisperer:conversations",
    "codewhisperer:taskassist",
    "codewhisperer:transformations"
  ],
  "grantTypes": [
    "urn:ietf:params:oauth:grant-type:device_code",
    "refresh_token"
  ],
  "issuerUrl": "https://view.awsapps.com/start"
}

响应:
{
  "clientId": "...",
  "clientSecret": "..."
}
```

#### 2.2 发起设备授权
```
POST https://oidc.us-east-1.amazonaws.com/device_authorization

请求体:
{
  "clientId": "...",
  "clientSecret": "...",
  "startUrl": "https://view.awsapps.com/start"
}

响应:
{
  "deviceCode": "...",
  "userCode": "XXXX-XXXX",
  "verificationUriComplete": "https://device.sso.us-east-1.amazonaws.com/?user_code=XXXX-XXXX",
  "interval": 1
}
```

#### 2.3 浏览器完成授权
- 在已登录的浏览器 Tab 中打开 `verificationUriComplete` URL
- 因为已经登录，直接显示授权确认页面
- 自动点击确认按钮

#### 2.4 轮询获取 Token
```
POST https://oidc.us-east-1.amazonaws.com/token

请求体:
{
  "clientId": "...",
  "clientSecret": "...",
  "grantType": "urn:ietf:params:oauth:grant-type:device_code",
  "deviceCode": "..."
}

响应:
{
  "accessToken": "aoa...",    // 以 aoa 开头
  "refreshToken": "aor...",   // 以 aor 开头 ← 这是我们需要的
  "expiresIn": 3600
}
```

---

## 数据存储

### 数据库字段映射
| 字段 | 存储内容 |
|------|----------|
| `email` | 邮箱地址 |
| `email_password` | 邮箱密码 |
| `client_id` | SSO clientId（授权后更新） |
| `refresh_token` | SSO clientSecret（授权后更新） |
| `kiro_password` | Kiro 账号密码 |
| `kiro_refresh_token` | SSO RefreshToken (aor...) |
| `status` | 账号状态 |

### 返回格式
注册成功后返回：
```
password|||refreshToken|||clientId|||clientSecret
```

---

## 导出格式

### JSON 导出（用于 kiro-account-manager）
```json
[
  {
    "refreshToken": "aor...",
    "clientId": "...",
    "clientSecret": "...",
    "region": "us-east-1",
    "provider": "BuilderId"
  }
]
```

### 导入方式
在 kiro-account-manager 中：
1. 点击"选择 JSON 文件"按钮
2. 选择导出的 JSON 文件
3. 自动解析并导入账号

---

## 关键代码文件

| 文件 | 功能 |
|------|------|
| `src-tauri/src/commands.rs` | 注册流程主逻辑 |
| `src-tauri/src/sso_auth.rs` | SSO 授权模块 |
| `src-tauri/src/imap_client.rs` | IMAP 邮件获取验证码 |
| `src-tauri/src/browser_automation.rs` | 浏览器自动化操作 |
| `src-tauri/src/database.rs` | 数据库操作 |

---

## 注意事项

1. **验证码邮件位置**：AWS 验证码邮件可能在 Junk（垃圾箱）文件夹，代码会同时检查 INBOX 和 Junk

2. **Token 类型**：
   - `aoa` 开头：AccessToken（短期有效）
   - `aor` 开头：RefreshToken（长期有效，这是我们需要的）

3. **Provider 类型**：
   - `BuilderId`：使用 SSO 授权的账号（有 clientId/clientSecret）
   - `Google`/`GitHub`：社交登录账号（无 clientId/clientSecret）

4. **浏览器复用**：SSO 授权时复用注册完成后的浏览器 Tab，无需重新登录
