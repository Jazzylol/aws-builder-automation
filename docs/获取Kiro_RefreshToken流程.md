# 获取 Kiro RefreshToken 完整流程

## 方式一：全自动流程（推荐）

### 脚本
`kiro_auto_flow.mjs` - 全自动注册 + 获取 RefreshToken

### 使用步骤

1. 创建配置文件 `accounts_to_process.json`：
```json
[{
  "email": "your_email@outlook.com",
  "outlookClientId": "9e5f94bc-e8a4-4e73-b8be-63364c29d753",
  "outlookRefreshToken": "M.C...(你的 Outlook refresh_token)",
  "name": "Zhang Wei",
  "needRegister": true,
  "password": null
}]
```

2. 运行脚本：
```bash
node kiro_auto_flow.mjs
```

3. 脚本会自动：
   - 注册 AWS Builder ID（如果 needRegister=true）
   - 自动获取邮箱验证码（检查收件箱和垃圾邮件）
   - 自动完成 SSO 设备授权
   - 生成导入文件 `kiro_batch_import.json`

---

## 方式二：手动 SSO 授权

### 脚本
`sso_auth.mjs` - SSO 设备授权脚本（需手动在浏览器授权）

### 步骤

1. 运行脚本：
```bash
node sso_auth.mjs
```

2. 在浏览器中打开输出的授权链接，登录并授权

3. ⚠️ **重要**：AWS 验证码邮件会发到 **垃圾邮件(Junk)文件夹**！

4. 授权成功后 Token 自动保存到 `kiro_token.json`

---

## 导入到 kiro-account-manager

### JSON 格式
```json
[{
  "refreshToken": "aor...",
  "clientId": "xxx",
  "clientSecret": "xxx",
  "region": "us-east-1",
  "provider": "BuilderId"
}]
```

### 导入方式
使用 **"选择 JSON 文件"** 按钮选择导入文件（不要手动粘贴，clientSecret 太长会被截断）

---

## 注意事项

1. Token 有效期约 1 小时，过期需重新获取
2. 有 clientId/clientSecret 的是 IdC 账号，provider 必须是 `BuilderId`
3. 没有 clientId/clientSecret 的是 Social 账号，provider 是 `Google`/`GitHub`
4. AWS 验证码邮件会发到**垃圾邮件**文件夹
