use anyhow::{Result, Context, anyhow};
use reqwest::Client;
use serde_json::Value;
use regex::Regex;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_native_tls::TlsConnector;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

pub struct ImapClient {
    http_client: Client,
}

impl ImapClient {
    pub fn new() -> Self {
        Self {
            http_client: Client::new(),
        }
    }

    /// 使用 refresh_token 获取 access_token
    pub async fn get_access_token(
        &self,
        client_id: &str,
        refresh_token: &str,
    ) -> Result<String> {
        eprintln!("[IMAP] 正在获取 Outlook access_token...");
        eprintln!("[IMAP] client_id: {}...", &client_id[..20.min(client_id.len())]);
        
        let url = "https://login.microsoftonline.com/common/oauth2/v2.0/token";

        let params = [
            ("client_id", client_id),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
            ("scope", "https://outlook.office.com/IMAP.AccessAsUser.All offline_access"),
        ];

        let response = self
            .http_client
            .post(url)
            .form(&params)
            .send()
            .await
            .context("Failed to send token request")?;

        let status = response.status();
        let response_text = response.text().await.unwrap_or_default();

        if !status.is_success() {
            eprintln!("[IMAP] ❌ Token 请求失败，状态码: {}", status);
            eprintln!("[IMAP] 响应: {}", response_text);
            return Err(anyhow!("Token request failed: {}", response_text));
        }

        eprintln!("[IMAP] ✅ Token 请求成功");

        let json: Value = serde_json::from_str(&response_text)
            .context("Failed to parse token response")?;

        json.get("access_token")
            .and_then(|v| v.as_str())
            .map(|s| {
                eprintln!("[IMAP] 获取到 access_token: {}...", &s[..30.min(s.len())]);
                s.to_string()
            })
            .ok_or_else(|| anyhow!("No access token in response"))
    }

    /// 通过 IMAP + OAuth2 获取最近的邮件
    pub async fn fetch_emails_via_imap(
        &self,
        email: &str,
        access_token: &str,
        max_count: usize,
    ) -> Result<Vec<EmailInfo>> {
        eprintln!("Connecting to IMAP server for {}...", email);
        
        // 连接到 Outlook IMAP 服务器
        let tcp_stream = TcpStream::connect("outlook.office365.com:993")
            .await
            .context("Failed to connect to IMAP server")?;

        let connector = native_tls::TlsConnector::builder()
            .build()
            .context("Failed to create TLS connector")?;
        let connector = TlsConnector::from(connector);
        
        let mut tls_stream = connector
            .connect("outlook.office365.com", tcp_stream)
            .await
            .context("Failed to establish TLS connection")?;

        // 读取服务器欢迎消息
        let mut buffer = vec![0u8; 8192];
        let _ = tls_stream.read(&mut buffer).await?;
        eprintln!("Connected to IMAP server");

        // 构建 XOAUTH2 认证字符串
        let auth_string = format!("user={}\x01auth=Bearer {}\x01\x01", email, access_token);
        let base64_auth = BASE64.encode(auth_string.as_bytes());

        // 发送认证命令
        let auth_cmd = format!("A1 AUTHENTICATE XOAUTH2 {}\r\n", base64_auth);
        tls_stream.write_all(auth_cmd.as_bytes()).await?;
        
        let n = tls_stream.read(&mut buffer).await?;
        let auth_response = String::from_utf8_lossy(&buffer[..n]);
        
        if !auth_response.contains("A1 OK") {
            return Err(anyhow!("IMAP authentication failed: {}", auth_response));
        }
        eprintln!("IMAP authentication successful");

        // 选择收件箱
        tls_stream.write_all(b"A2 SELECT INBOX\r\n").await?;
        let n = tls_stream.read(&mut buffer).await?;
        let select_response = String::from_utf8_lossy(&buffer[..n]);
        
        // 解析邮件数量
        let exists_re = Regex::new(r"(\d+) EXISTS").unwrap();
        let total_emails = exists_re
            .captures(&select_response)
            .and_then(|c| c.get(1))
            .and_then(|m| m.as_str().parse::<usize>().ok())
            .unwrap_or(0);

        eprintln!("Inbox has {} emails", total_emails);

        if total_emails == 0 {
            tls_stream.write_all(b"A99 LOGOUT\r\n").await?;
            return Ok(vec![]);
        }

        // 获取最近的邮件
        let start = if total_emails > max_count { total_emails - max_count + 1 } else { 1 };
        let fetch_cmd = format!(
            "A3 FETCH {}:{} (BODY[HEADER.FIELDS (FROM SUBJECT DATE)] BODY[TEXT])\r\n",
            start, total_emails
        );
        
        tls_stream.write_all(fetch_cmd.as_bytes()).await?;
        
        // 读取所有响应
        let mut full_response = String::new();
        loop {
            let n = tls_stream.read(&mut buffer).await?;
            if n == 0 {
                break;
            }
            full_response.push_str(&String::from_utf8_lossy(&buffer[..n]));
            if full_response.contains("A3 OK") || full_response.contains("A3 NO") || full_response.contains("A3 BAD") {
                break;
            }
        }

        // 解析邮件
        let emails = self.parse_imap_response(&full_response);
        eprintln!("Parsed {} emails", emails.len());

        // 登出
        tls_stream.write_all(b"A99 LOGOUT\r\n").await?;

        Ok(emails)
    }

    /// 解析 IMAP FETCH 响应
    fn parse_imap_response(&self, response: &str) -> Vec<EmailInfo> {
        let mut emails = Vec::new();
        let mut current_from = String::new();
        let mut current_subject = String::new();
        let mut current_body = String::new();
        let mut in_body = false;

        for line in response.lines() {
            let line_lower = line.to_lowercase();
            
            if line_lower.starts_with("from:") {
                current_from = line[5..].trim().to_string();
                in_body = false;
            } else if line_lower.starts_with("subject:") {
                current_subject = self.decode_mime_header(&line[8..].trim());
                in_body = false;
            } else if line.starts_with("* ") && line.contains("FETCH") {
                // 新邮件开始，保存之前的
                if !current_from.is_empty() {
                    emails.push(EmailInfo {
                        from: current_from.clone(),
                        subject: current_subject.clone(),
                        body: current_body.clone(),
                    });
                }
                current_from.clear();
                current_subject.clear();
                current_body.clear();
                in_body = false;
            } else if line.contains("BODY[TEXT]") {
                in_body = true;
            } else if in_body {
                current_body.push_str(line);
                current_body.push('\n');
            }
        }

        // 保存最后一封邮件
        if !current_from.is_empty() {
            emails.push(EmailInfo {
                from: current_from,
                subject: current_subject,
                body: current_body,
            });
        }

        emails
    }

    /// 解码 MIME 编码的邮件头
    fn decode_mime_header(&self, header: &str) -> String {
        let re = Regex::new(r"=\?([^?]+)\?([BQ])\?([^?]+)\?=").unwrap();
        
        if let Some(caps) = re.captures(header) {
            let encoding = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            let encoded = caps.get(3).map(|m| m.as_str()).unwrap_or("");
            
            if encoding.eq_ignore_ascii_case("B") {
                // Base64 编码
                if let Ok(decoded) = BASE64.decode(encoded) {
                    if let Ok(s) = String::from_utf8(decoded) {
                        return s;
                    }
                }
            }
        }
        
        header.to_string()
    }

    /// 从邮件正文中提取 6 位验证码
    pub fn extract_verification_code(body: &str) -> Option<String> {
        // 移除 HTML 标签
        let re_html = Regex::new(r"<[^>]+>").ok()?;
        let plain_text = re_html.replace_all(body, " ");

        // 查找 6 位数字验证码
        let re_code = Regex::new(r"\b(\d{6})\b").ok()?;

        if let Some(captures) = re_code.captures(&plain_text) {
            let code = captures.get(1).map(|m| m.as_str().to_string());
            if let Some(ref c) = code {
                eprintln!("Found verification code: {}", c);
            }
            return code;
        }

        None
    }

    /// 从最近的邮件中获取验证码
    pub async fn get_verification_code_from_recent_email(
        &self,
        client_id: &str,
        refresh_token: &str,
        email: &str,
    ) -> Result<String> {
        eprintln!("[邮件] 开始检查邮箱: {}", email);
        
        // 获取 access_token
        let access_token = self.get_access_token(client_id, refresh_token).await?;

        // 同时检查 INBOX 和 Junk 文件夹
        let folders = vec!["INBOX", "Junk"];
        let mut all_emails = Vec::new();
        
        for folder in &folders {
            eprintln!("[邮件] 正在检查文件夹: {}", folder);
            match self.fetch_emails_from_folder(email, &access_token, folder, 5).await {
                Ok(emails) => {
                    eprintln!("[邮件] {} 中找到 {} 封邮件", folder, emails.len());
                    for (i, e) in emails.iter().enumerate() {
                        eprintln!("[邮件]   {}. 发件人: {}, 主题: {}", i+1, e.from, e.subject);
                    }
                    all_emails.extend(emails);
                }
                Err(e) => {
                    eprintln!("[邮件] ⚠️ 无法获取 {} 中的邮件: {}", folder, e);
                }
            }
        }

        eprintln!("[邮件] 共获取到 {} 封邮件，开始查找验证码...", all_emails.len());

        // 查找验证码 - 从最新的邮件开始（倒序遍历）
        let recent_emails: Vec<_> = all_emails.iter().rev().take(5).collect();
        
        for (idx, email_info) in recent_emails.iter().enumerate() {
            let from_lower = email_info.from.to_lowercase();
            let subject_lower = email_info.subject.to_lowercase();

            // 检查是否是验证邮件
            let is_verification_email = 
                from_lower.contains("amazon") ||
                from_lower.contains("aws") ||
                from_lower.contains("kiro") ||
                from_lower.contains("noreply") ||
                from_lower.contains("no-reply") ||
                from_lower.contains("signin") ||
                subject_lower.contains("verification") ||
                subject_lower.contains("verify") ||
                subject_lower.contains("验证") ||
                subject_lower.contains("code") ||
                subject_lower.contains("otp");

            eprintln!("[邮件] 检查第 {} 封邮件: 发件人={}, 主题={}, 是验证邮件={}", 
                idx+1, email_info.from, email_info.subject, is_verification_email);

            if is_verification_email {
                // 查找所有6位数字，取最后一个（通常是验证码）
                let re_code = Regex::new(r"\b(\d{6})\b").unwrap();
                let mut last_code = None;
                for cap in re_code.captures_iter(&email_info.body) {
                    last_code = Some(cap.get(1).unwrap().as_str().to_string());
                }
                if let Some(code) = last_code {
                    eprintln!("[邮件] ✅ 找到验证码: {}", code);
                    return Ok(code);
                } else {
                    eprintln!("[邮件] ⚠️ 这封邮件中没有找到6位数字验证码");
                }
            }
        }

        eprintln!("[邮件] ❌ 在所有邮件中都没有找到验证码");
        Err(anyhow!("No verification code found in recent emails (checked INBOX and Junk)"))
    }
    
    /// 从指定文件夹获取邮件
    pub async fn fetch_emails_from_folder(
        &self,
        email: &str,
        access_token: &str,
        folder: &str,
        max_count: usize,
    ) -> Result<Vec<EmailInfo>> {
        eprintln!("Connecting to IMAP server for {} (folder: {})...", email, folder);
        
        // 连接到 Outlook IMAP 服务器
        let tcp_stream = TcpStream::connect("outlook.office365.com:993")
            .await
            .context("Failed to connect to IMAP server")?;

        let connector = native_tls::TlsConnector::builder()
            .build()
            .context("Failed to create TLS connector")?;
        let connector = TlsConnector::from(connector);
        
        let mut tls_stream = connector
            .connect("outlook.office365.com", tcp_stream)
            .await
            .context("Failed to establish TLS connection")?;

        // 读取服务器欢迎消息
        let mut buffer = vec![0u8; 8192];
        let _ = tls_stream.read(&mut buffer).await?;

        // 构建 XOAUTH2 认证字符串
        let auth_string = format!("user={}\x01auth=Bearer {}\x01\x01", email, access_token);
        let base64_auth = BASE64.encode(auth_string.as_bytes());

        // 发送认证命令
        let auth_cmd = format!("A1 AUTHENTICATE XOAUTH2 {}\r\n", base64_auth);
        tls_stream.write_all(auth_cmd.as_bytes()).await?;
        
        let n = tls_stream.read(&mut buffer).await?;
        let auth_response = String::from_utf8_lossy(&buffer[..n]);
        
        if !auth_response.contains("A1 OK") {
            return Err(anyhow!("IMAP authentication failed: {}", auth_response));
        }

        // 选择指定文件夹
        let select_cmd = format!("A2 SELECT \"{}\"\r\n", folder);
        tls_stream.write_all(select_cmd.as_bytes()).await?;
        let n = tls_stream.read(&mut buffer).await?;
        let select_response = String::from_utf8_lossy(&buffer[..n]);
        
        // 解析邮件数量
        let exists_re = Regex::new(r"(\d+) EXISTS").unwrap();
        let total_emails = exists_re
            .captures(&select_response)
            .and_then(|c| c.get(1))
            .and_then(|m| m.as_str().parse::<usize>().ok())
            .unwrap_or(0);

        eprintln!("{} has {} emails", folder, total_emails);

        if total_emails == 0 {
            tls_stream.write_all(b"A99 LOGOUT\r\n").await?;
            return Ok(vec![]);
        }

        // 获取最近的邮件
        let start = if total_emails > max_count { total_emails - max_count + 1 } else { 1 };
        let fetch_cmd = format!(
            "A3 FETCH {}:{} (BODY[HEADER.FIELDS (FROM SUBJECT DATE)] BODY[TEXT])\r\n",
            start, total_emails
        );
        
        tls_stream.write_all(fetch_cmd.as_bytes()).await?;
        
        // 读取所有响应
        let mut full_response = String::new();
        loop {
            let n = tls_stream.read(&mut buffer).await?;
            if n == 0 {
                break;
            }
            full_response.push_str(&String::from_utf8_lossy(&buffer[..n]));
            if full_response.contains("A3 OK") || full_response.contains("A3 NO") || full_response.contains("A3 BAD") {
                break;
            }
        }

        // 解析邮件
        let emails = self.parse_imap_response(&full_response);
        eprintln!("Parsed {} emails from {}", emails.len(), folder);

        // 登出
        tls_stream.write_all(b"A99 LOGOUT\r\n").await?;

        Ok(emails)
    }

    /// 等待验证码（带超时）
    pub async fn wait_for_verification_code(
        &self,
        client_id: &str,
        refresh_token: &str,
        email: &str,
        timeout_seconds: u64,
    ) -> Result<String> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(timeout_seconds);

        eprintln!("[验证码] ========================================");
        eprintln!("[验证码] 开始等待验证码");
        eprintln!("[验证码] 邮箱: {}", email);
        eprintln!("[验证码] 超时时间: {}秒", timeout_seconds);
        eprintln!("[验证码] client_id: {}...", &client_id[..20.min(client_id.len())]);
        eprintln!("[验证码] ========================================");

        let mut attempt = 0;
        let mut token_error_count = 0; // 记录 token 错误次数
        
        loop {
            attempt += 1;
            let elapsed = start.elapsed().as_secs();
            
            if start.elapsed() > timeout {
                eprintln!("[验证码] ❌ 超时！已等待 {}秒，共尝试 {} 次", elapsed, attempt);
                return Err(anyhow!("Timeout waiting for verification code after {}s", timeout_seconds));
            }

            eprintln!("[验证码] 第 {} 次尝试获取验证码 (已等待 {}秒)...", attempt, elapsed);

            match self.get_verification_code_from_recent_email(client_id, refresh_token, email).await {
                Ok(code) => {
                    eprintln!("[验证码] ✅ 成功获取验证码: {}", code);
                    return Ok(code);
                },
                Err(e) => {
                    let error_str = e.to_string();
                    
                    // 检查是否是 token 失效错误
                    if error_str.contains("invalid_grant") || error_str.contains("Token request failed") {
                        token_error_count += 1;
                        eprintln!("[验证码] ⚠️ Token 错误 ({}/2): {}", token_error_count, e);
                        
                        // 最多重试 1 次（共 2 次尝试）
                        if token_error_count >= 2 {
                            eprintln!("[验证码] ❌ Token 失效，跳过此账号");
                            return Err(anyhow!("OAuth2 token 失效，请更新账号的 refresh_token"));
                        }
                    } else {
                        eprintln!("[验证码] ⚠️ 获取失败: {}", e);
                    }
                    
                    eprintln!("[验证码] 5秒后重试...");
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct EmailInfo {
    pub from: String,
    pub subject: String,
    pub body: String,
}
