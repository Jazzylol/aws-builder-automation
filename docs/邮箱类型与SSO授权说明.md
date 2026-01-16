# 邮箱类型与 SSO 授权说明

## 🔍 核心概念

### SSO 授权 ≠ 邮箱类型

**重要**：SSO 授权与你使用什么邮箱**没有直接关系**！

---

## 📧 邮箱的两个作用

### 作用 1：注册 AWS Builder ID 账号 ✅

**任何邮箱都可以**：
- ✅ QQ 邮箱（@qq.com）
- ✅ 163 邮箱（@163.com）
- ✅ Gmail（@gmail.com）
- ✅ Outlook（@outlook.com）
- ✅ 自定义域名邮箱
- ✅ 任何能收邮件的邮箱

**要求**：
- 能接收验证码邮件
- 仅此而已

---

### 作用 2：通过 IMAP 自动获取验证码 ⚠️

**只有支持 OAuth2 的邮箱可以**：
- ✅ Outlook/Hotmail（@outlook.com, @hotmail.com）
- ✅ Gmail（@gmail.com）
- ❌ QQ 邮箱（@qq.com）- **不支持 OAuth2**
- ❌ 163 邮箱（@163.com）- **不支持 OAuth2**

**这就是为什么我们使用 Outlook 邮箱的原因**！

---

## 🔐 SSO 授权流程详解

### 完整流程

```
步骤 1: 用户输入邮箱（任何邮箱都可以）
   ↓
步骤 2: AWS 发送验证码到邮箱
   ↓
步骤 3: 用户输入验证码
   ↓
步骤 4: 用户设置密码
   ↓
步骤 5: 注册完成
   ↓
步骤 6: SSO 授权（与邮箱无关！）
   ↓
步骤 7: 获取 RefreshToken
```

**关键点**：
- SSO 授权发生在**注册完成之后**
- SSO 授权是 AWS 内部的授权流程
- 与你使用什么邮箱**完全无关**

---

## ❓ QQ 邮箱可以注册吗？

### 答案：可以，但有限制 ⚠️

#### ✅ 可以做的事情

1. **手动注册**
   - 使用 QQ 邮箱注册 AWS Builder ID
   - 手动输入验证码
   - 完成注册
   - 手动完成 SSO 授权

2. **使用 SSO 授权链接功能**
   - 生成 SSO 授权链接
   - 在浏览器中手动完成授权
   - 获取 RefreshToken

#### ❌ 不能做的事情

1. **自动批量注册**
   - QQ 邮箱不支持 OAuth2
   - 无法通过 IMAP 自动获取验证码
   - 必须手动输入验证码

2. **自动获取验证码**
   - 程序无法自动读取 QQ 邮箱
   - 必须手动查看邮件

---

## 📊 邮箱类型对比

### Outlook/Hotmail 邮箱 ⭐⭐⭐⭐⭐

| 功能 | 支持 |
|-----|------|
| 注册 AWS Builder ID | ✅ |
| 自动获取验证码 | ✅ |
| 批量注册 | ✅ |
| SSO 授权 | ✅ |

**推荐度**：⭐⭐⭐⭐⭐（最推荐）

---

### Gmail 邮箱 ⭐⭐⭐⭐

| 功能 | 支持 |
|-----|------|
| 注册 AWS Builder ID | ✅ |
| 自动获取验证码 | ✅ |
| 批量注册 | ✅ |
| SSO 授权 | ✅ |

**推荐度**：⭐⭐⭐⭐（推荐）

**注意**：
- Gmail 的 OAuth2 配置比 Outlook 复杂
- 需要在 Google Cloud Console 创建项目
- 本项目目前只支持 Outlook

---

### QQ 邮箱 ⭐⭐

| 功能 | 支持 |
|-----|------|
| 注册 AWS Builder ID | ✅ |
| 自动获取验证码 | ❌ |
| 批量注册 | ❌ |
| SSO 授权 | ✅ |

**推荐度**：⭐⭐（不推荐批量注册）

**限制**：
- 不支持 OAuth2
- 无法自动获取验证码
- 只能手动注册

---

### 163/126 邮箱 ⭐⭐

| 功能 | 支持 |
|-----|------|
| 注册 AWS Builder ID | ✅ |
| 自动获取验证码 | ❌ |
| 批量注册 | ❌ |
| SSO 授权 | ✅ |

**推荐度**：⭐⭐（不推荐批量注册）

**限制**：
- 不支持 OAuth2
- 无法自动获取验证码
- 只能手动注册

---

## 💡 使用 QQ 邮箱的方案

### 方案 1：手动注册（可行）⭐⭐⭐

**步骤**：
1. 使用 QQ 邮箱在 AWS 网站手动注册
2. 手动输入验证码
3. 完成注册
4. 使用程序的"SSO 授权链接"功能
5. 手动完成 SSO 授权
6. 获取 RefreshToken

**优点**：
- ✅ 可以使用 QQ 邮箱
- ✅ 不需要 OAuth2

**缺点**：
- ❌ 无法批量注册
- ❌ 必须手动操作
- ❌ 效率低

---

### 方案 2：混合使用（推荐）⭐⭐⭐⭐⭐

**策略**：
```
批量注册: 使用 Outlook 邮箱（自动化）
手动注册: 使用 QQ 邮箱（手动）
```

**示例**：
```
账号 1-8:  @outlook.com（批量注册）
账号 9-10: @qq.com（手动注册）
```

**优点**：
- ✅ 分散邮箱域名
- ✅ 降低风险
- ✅ 兼顾效率和安全

**缺点**：
- ⚠️ 需要手动注册部分账号

---

### 方案 3：只用 Outlook（最简单）⭐⭐⭐⭐⭐

**策略**：
```
全部使用 Outlook/Hotmail 邮箱
但分散域名：
- @outlook.com
- @hotmail.com
- @live.com
```

**优点**：
- ✅ 完全自动化
- ✅ 效率最高
- ✅ 实现简单

**缺点**：
- ⚠️ 所有邮箱都是微软系

---

## 🔧 如果你想用 QQ 邮箱批量注册

### 需要做的事情

#### 1. 添加 QQ 邮箱 IMAP 支持

**问题**：QQ 邮箱不支持 OAuth2，但支持传统的用户名+密码 IMAP

**解决方案**：修改代码支持传统 IMAP

**修改文件**：`src-tauri/src/imap_client.rs`

**添加代码**：
```rust
// 新增：传统 IMAP 登录（用于 QQ 邮箱）
pub async fn connect_with_password(
    &self,
    email: &str,
    password: &str,  // QQ 邮箱的授权码
) -> Result<Session<TlsStream<TcpStream>>> {
    let domain = if email.contains("@qq.com") {
        "imap.qq.com"
    } else if email.contains("@163.com") {
        "imap.163.com"
    } else {
        return Err(anyhow!("Unsupported email domain"));
    };

    let tls = TlsConnector::builder().build()?;
    let client = imap::connect((domain, 993), domain, &tls)?;
    
    let session = client
        .login(email, password)
        .map_err(|e| anyhow!("Login failed: {:?}", e))?;
    
    Ok(session)
}
```

#### 2. 获取 QQ 邮箱授权码

**步骤**：
1. 登录 QQ 邮箱网页版
2. 设置 → 账户
3. 开启 IMAP/SMTP 服务
4. 生成授权码（不是 QQ 密码！）
5. 保存授权码

#### 3. 修改数据库

**添加字段**：
```sql
ALTER TABLE accounts ADD COLUMN email_auth_type TEXT DEFAULT 'oauth2';
ALTER TABLE accounts ADD COLUMN email_password TEXT;
```

#### 4. 修改导入逻辑

**支持两种格式**：
```
# OAuth2 格式（Outlook）
email----password----client_id----refresh_token

# 传统格式（QQ 邮箱）
email----password----auth_code----traditional
```

---

## 📊 实施难度对比

| 方案 | 难度 | 时间 | 推荐度 |
|-----|------|------|--------|
| 手动注册 QQ 邮箱 | ⭐ 简单 | 每个 5 分钟 | ⭐⭐⭐ |
| 混合使用 | ⭐ 简单 | 无需修改代码 | ⭐⭐⭐⭐⭐ |
| 只用 Outlook | ⭐ 简单 | 无需修改代码 | ⭐⭐⭐⭐⭐ |
| 添加 QQ 邮箱支持 | ⭐⭐⭐ 较难 | 2-3 小时 | ⭐⭐ |

---

## 🎯 推荐方案

### 对于你的情况

**推荐：混合使用方案** ⭐⭐⭐⭐⭐

```
批量注册（8个）: 
- 4 个 @outlook.com
- 4 个 @hotmail.com

手动注册（2个）:
- 2 个 @qq.com
```

**优点**：
- ✅ 分散邮箱域名（降低风险）
- ✅ 无需修改代码
- ✅ 兼顾效率和安全
- ✅ QQ 邮箱看起来更"真实"

**操作步骤**：

**第 1-4 天**：批量注册 Outlook 邮箱
```bash
# 使用程序批量注册
第 1 天: 2 个 @outlook.com
第 2 天: 2 个 @hotmail.com
第 3 天: 2 个 @outlook.com
第 4 天: 2 个 @hotmail.com
```

**第 5 天**：手动注册 QQ 邮箱
```bash
# 手动在 AWS 网站注册
1. 访问 https://app.kiro.dev/signin
2. 使用 QQ 邮箱注册
3. 手动输入验证码
4. 完成注册
5. 使用程序的"SSO 授权链接"功能获取 Token
```

---

## ❓ 常见问题

### Q1：QQ 邮箱注册的账号和 Outlook 注册的账号有区别吗？

**答**：没有区别！
- 注册完成后，都是 AWS Builder ID 账号
- 都可以使用 Kiro
- 都可以获取 RefreshToken
- 唯一区别：注册时使用的邮箱不同

---

### Q2：我已经用 QQ 邮箱注册了，怎么获取 Token？

**答**：使用"SSO 授权链接"功能
1. 在程序中找到该账号
2. 点击"生成 SSO 授权链接"
3. 复制链接
4. 在浏览器中打开（已登录的浏览器）
5. 手动点击"确认并继续"和"允许访问"
6. 点击"获取 Token"按钮

---

### Q3：为什么不直接支持 QQ 邮箱批量注册？

**答**：技术限制
- QQ 邮箱不支持 OAuth2
- 使用授权码需要每个邮箱单独配置
- 实现复杂度高
- 性价比低（不如直接用 Outlook）

---

### Q4：Gmail 可以用吗？

**答**：理论上可以，但需要配置
- Gmail 支持 OAuth2
- 但需要在 Google Cloud Console 创建项目
- 配置比 Outlook 复杂
- 本项目目前不支持（可以后续添加）

---

## 📅 更新日期

2026-01-16

---

## 🎉 总结

### 关键点

1. **SSO 授权与邮箱类型无关**
   - 任何邮箱都可以注册 AWS Builder ID
   - 任何邮箱都可以完成 SSO 授权

2. **自动获取验证码需要 OAuth2**
   - Outlook/Hotmail：✅ 支持
   - Gmail：✅ 支持（需配置）
   - QQ/163：❌ 不支持

3. **推荐方案**
   - 批量注册：使用 Outlook
   - 手动注册：可以用 QQ
   - 混合使用：最佳方案

### 你的选择

- **只想批量注册**：用 Outlook/Hotmail
- **想分散风险**：混合使用（Outlook + QQ）
- **不在乎效率**：全部手动注册（任何邮箱）

**记住：邮箱只是用来接收验证码的，与 SSO 授权无关！** ✅
