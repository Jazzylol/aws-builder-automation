use anyhow::{Result, Context, anyhow};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::sync::Arc;
use headless_chrome::Tab;
use crate::browser_automation::BrowserAutomation;
use crate::models::BrowserConfig;

const OIDC_BASE: &str = "https://oidc.us-east-1.amazonaws.com";
const START_URL: &str = "https://view.awsapps.com/start";

#[derive(Debug, Serialize)]
struct RegisterClientRequest {
    #[serde(rename = "clientName")]
    client_name: String,
    #[serde(rename = "clientType")]
    client_type: String,
    scopes: Vec<String>,
    #[serde(rename = "grantTypes")]
    grant_types: Vec<String>,
    #[serde(rename = "issuerUrl")]
    issuer_url: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterClientResponse {
    #[serde(rename = "clientId")]
    pub client_id: String,
    #[serde(rename = "clientSecret")]
    pub client_secret: String,
}

#[derive(Debug, Serialize)]
struct DeviceAuthRequest {
    #[serde(rename = "clientId")]
    client_id: String,
    #[serde(rename = "clientSecret")]
    client_secret: String,
    #[serde(rename = "startUrl")]
    start_url: String,
}

#[derive(Debug, Deserialize)]
pub struct DeviceAuthResponse {
    #[serde(rename = "deviceCode")]
    pub device_code: String,
    #[serde(rename = "userCode")]
    pub user_code: String,
    #[serde(rename = "verificationUriComplete")]
    pub verification_uri_complete: String,
    pub interval: Option<u64>,
}

#[derive(Debug, Serialize)]
struct TokenRequest {
    #[serde(rename = "clientId")]
    client_id: String,
    #[serde(rename = "clientSecret")]
    client_secret: String,
    #[serde(rename = "grantType")]
    grant_type: String,
    #[serde(rename = "deviceCode")]
    device_code: String,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    #[serde(rename = "accessToken")]
    access_token: String,
    #[serde(rename = "refreshToken")]
    refresh_token: String,
    #[serde(rename = "expiresIn")]
    #[allow(dead_code)]
    expires_in: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Debug, Clone)]
pub struct SsoTokenResult {
    #[allow(dead_code)]
    pub access_token: String,
    pub refresh_token: String,
    pub client_id: String,
    pub client_secret: String,
}

pub struct SsoAuth {
    http_client: Client,
}

impl SsoAuth {
    pub fn new() -> Self {
        Self {
            http_client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap(),
        }
    }

    /// 注册 OIDC 客户端
    pub async fn register_client(&self) -> Result<RegisterClientResponse> {
        let request = RegisterClientRequest {
            client_name: "Kiro Account Manager".to_string(),
            client_type: "public".to_string(),
            scopes: vec![
                "codewhisperer:analysis".to_string(),
                "codewhisperer:completions".to_string(),
                "codewhisperer:conversations".to_string(),
                "codewhisperer:taskassist".to_string(),
                "codewhisperer:transformations".to_string(),
            ],
            grant_types: vec![
                "urn:ietf:params:oauth:grant-type:device_code".to_string(),
                "refresh_token".to_string(),
            ],
            issuer_url: START_URL.to_string(),
        };

        let response = self.http_client
            .post(format!("{}/client/register", OIDC_BASE))
            .json(&request)
            .send()
            .await
            .context("Failed to register OIDC client")?;

        let result: RegisterClientResponse = response.json().await
            .context("Failed to parse register client response")?;

        Ok(result)
    }

    /// 发起设备授权
    pub async fn start_device_authorization(
        &self,
        client_id: &str,
        client_secret: &str,
    ) -> Result<DeviceAuthResponse> {
        let request = DeviceAuthRequest {
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            start_url: START_URL.to_string(),
        };

        let response = self.http_client
            .post(format!("{}/device_authorization", OIDC_BASE))
            .json(&request)
            .send()
            .await
            .context("Failed to start device authorization")?;

        let result: DeviceAuthResponse = response.json().await
            .context("Failed to parse device authorization response")?;

        Ok(result)
    }

    /// 轮询获取 Token
    async fn poll_for_token(
        &self,
        client_id: &str,
        client_secret: &str,
        device_code: &str,
        interval: u64,
    ) -> Result<TokenResponse> {
        let request = TokenRequest {
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            grant_type: "urn:ietf:params:oauth:grant-type:device_code".to_string(),
            device_code: device_code.to_string(),
        };

        let mut poll_interval = interval;

        for _ in 0..120 {
            tokio::time::sleep(Duration::from_secs(poll_interval)).await;

            let response = self.http_client
                .post(format!("{}/token", OIDC_BASE))
                .json(&request)
                .send()
                .await?;

            if response.status().is_success() {
                let result: TokenResponse = response.json().await?;
                return Ok(result);
            }

            if response.status() == 400 {
                let error: ErrorResponse = response.json().await?;
                match error.error.as_str() {
                    "authorization_pending" => continue,
                    "slow_down" => {
                        poll_interval += 5;
                        continue;
                    }
                    "expired_token" => return Err(anyhow!("Device code expired")),
                    _ => return Err(anyhow!("Token error: {}", error.error)),
                }
            }
        }

        Err(anyhow!("Timeout waiting for authorization"))
    }

    /// 公开的轮询获取 Token 方法（供外部调用）
    pub async fn poll_for_token_public(
        &self,
        client_id: &str,
        client_secret: &str,
        device_code: &str,
        interval: u64,
    ) -> Result<SsoTokenResult> {
        let token = self.poll_for_token(client_id, client_secret, device_code, interval).await?;
        Ok(SsoTokenResult {
            access_token: token.access_token,
            refresh_token: token.refresh_token,
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
        })
    }

    /// 使用已有的浏览器 Tab 完成 SSO 授权（复用登录状态，无需重新登录）
    pub async fn authorize_with_existing_tab(
        &self,
        tab: &Arc<Tab>,
    ) -> Result<SsoTokenResult> {
        eprintln!("[SSO] Starting SSO authorization with existing browser session...");

        // Step 1: 注册客户端
        eprintln!("[SSO] Step 1: Registering OIDC client...");
        let client = self.register_client().await?;
        eprintln!("[SSO] Client registered");

        // Step 2: 发起设备授权
        eprintln!("[SSO] Step 2: Starting device authorization...");
        let device_auth = self.start_device_authorization(&client.client_id, &client.client_secret).await?;
        eprintln!("[SSO] Device code: {}", device_auth.user_code);
        eprintln!("[SSO] Verification URL: {}", device_auth.verification_uri_complete);

        // Step 3: 在已有的 Tab 中打开授权页面（复用登录状态）
        eprintln!("[SSO] Step 3: Opening authorization page in existing tab...");
        
        // 先等待当前页面稳定
        std::thread::sleep(std::time::Duration::from_secs(2));
        
        // 打印当前页面 URL
        let get_url_script = r#"window.location.href"#;
        if let Ok(result) = tab.evaluate(get_url_script, true) {
            if let Some(value) = result.value {
                if let Some(url) = value.as_str() {
                    eprintln!("[SSO] 当前页面 URL: {}", url);
                }
            }
        }
        
        // 尝试导航到验证 URL
        eprintln!("[SSO] 导航到: {}", device_auth.verification_uri_complete);
        match tab.navigate_to(&device_auth.verification_uri_complete) {
            Ok(_) => {
                eprintln!("[SSO] 导航成功，等待页面加载...");
            }
            Err(e) => {
                eprintln!("[SSO] 导航失败: {}, 尝试使用 JavaScript 方式...", e);
                // 使用 JavaScript 方式导航
                let nav_script = format!(r#"window.location.href = "{}";"#, device_auth.verification_uri_complete);
                tab.evaluate(&nav_script, true)
                    .context("Failed to navigate using JavaScript")?;
            }
        }
        
        // 等待导航完成
        let _ = tab.wait_until_navigated();
        std::thread::sleep(std::time::Duration::from_secs(3));

        // 点击确认按钮（因为已经登录，应该直接显示确认页面）
        eprintln!("[SSO] 等待授权页面加载...");
        std::thread::sleep(std::time::Duration::from_secs(2));
        
        // 第一步：点击"确认并继续"按钮
        for attempt in 1..=5 {
            eprintln!("[SSO] 尝试点击「确认并继续」按钮 ({}/5)...", attempt);
            
            let confirm_continue_script = r#"
                (function() {
                    const allButtons = Array.from(document.querySelectorAll('button'));
                    console.log('All buttons:', allButtons.map(b => b.innerText.trim()));
                    
                    // 尝试通过文本查找"确认并继续"按钮
                    const buttons = Array.from(document.querySelectorAll('button, input[type="submit"]'));
                    for (const btn of buttons) {
                        const text = (btn.innerText || btn.value || '').trim();
                        const textLower = text.toLowerCase();
                        // 匹配"确认并继续" / "Confirm and continue"
                        if (text.includes('确认并继续') || 
                            text.includes('确认') ||
                            textLower.includes('confirm and continue') ||
                            textLower.includes('confirm')) {
                            console.log('Found confirm button:', text);
                            btn.click();
                            return 'clicked: ' + text;
                        }
                    }
                    
                    // 尝试选择器
                    const selectors = [
                        'button[data-analytics="confirmDeviceButton"]',
                        'button[type="submit"]',
                        'input[type="submit"]'
                    ];
                    
                    for (const sel of selectors) {
                        const btn = document.querySelector(sel);
                        if (btn) {
                            btn.click();
                            return 'clicked selector: ' + sel;
                        }
                    }
                    
                    return 'no button found';
                })()
            "#;

            let click_result = tab.evaluate(confirm_continue_script, true);
            if let Ok(result) = &click_result {
                if let Some(value) = &result.value {
                    let result_str = value.as_str().unwrap_or("");
                    eprintln!("[SSO] 点击结果: {}", result_str);
                    if result_str.starts_with("clicked") {
                        eprintln!("[SSO] ✅ 成功点击「确认并继续」按钮");
                        break;
                    }
                }
            }
            
            if attempt < 5 {
                std::thread::sleep(std::time::Duration::from_secs(2));
            }
        }
        
        // 等待页面跳转到"允许访问"页面
        std::thread::sleep(std::time::Duration::from_secs(3));
        
        // 第二步：点击"允许访问"按钮
        for attempt in 1..=5 {
            eprintln!("[SSO] 尝试点击「允许访问」按钮 ({}/5)...", attempt);
            
            let allow_script = r#"
                (function() {
                    const allButtons = Array.from(document.querySelectorAll('button'));
                    console.log('All buttons:', allButtons.map(b => b.innerText.trim()));
                    
                    // 尝试通过文本查找"允许访问"按钮
                    const buttons = Array.from(document.querySelectorAll('button, input[type="submit"]'));
                    for (const btn of buttons) {
                        const text = (btn.innerText || btn.value || '').trim();
                        const textLower = text.toLowerCase();
                        // 匹配"允许访问" / "Allow access" / "Allow"
                        if (text.includes('允许访问') || 
                            text.includes('允许') ||
                            textLower.includes('allow access') ||
                            textLower.includes('allow')) {
                            console.log('Found allow button:', text);
                            btn.click();
                            return 'clicked: ' + text;
                        }
                    }
                    
                    // 尝试选择器
                    const selectors = [
                        'button[data-testid="allow-button"]',
                        'button[type="submit"]',
                        'input[type="submit"]'
                    ];
                    
                    for (const sel of selectors) {
                        const btn = document.querySelector(sel);
                        if (btn) {
                            btn.click();
                            return 'clicked selector: ' + sel;
                        }
                    }
                    
                    return 'no button found';
                })()
            "#;

            let click_result = tab.evaluate(allow_script, true);
            if let Ok(result) = &click_result {
                if let Some(value) = &result.value {
                    let result_str = value.as_str().unwrap_or("");
                    eprintln!("[SSO] 点击结果: {}", result_str);
                    if result_str.starts_with("clicked") {
                        eprintln!("[SSO] ✅ 成功点击「允许访问」按钮");
                        break;
                    }
                }
            }
            
            if attempt < 5 {
                std::thread::sleep(std::time::Duration::from_secs(2));
            }
        }
        
        std::thread::sleep(std::time::Duration::from_secs(3));

        // Step 4: 轮询获取 Token
        eprintln!("[SSO] Step 4: Polling for token...");
        let interval = device_auth.interval.unwrap_or(1);
        let token = self.poll_for_token(
            &client.client_id,
            &client.client_secret,
            &device_auth.device_code,
            interval,
        ).await?;

        eprintln!("[SSO] Successfully obtained refresh token: {}...", &token.refresh_token[..30.min(token.refresh_token.len())]);

        Ok(SsoTokenResult {
            access_token: token.access_token,
            refresh_token: token.refresh_token,
            client_id: client.client_id,
            client_secret: client.client_secret,
        })
    }

    /// 完成 SSO 授权（已经导航到授权页面后调用）
    pub async fn complete_authorization_with_tab(
        &self,
        tab: &Arc<Tab>,
        client: &RegisterClientResponse,
        device_auth: &DeviceAuthResponse,
    ) -> Result<SsoTokenResult> {
        eprintln!("[SSO] 完成 SSO 授权流程...");
        
        // 等待授权页面加载
        std::thread::sleep(std::time::Duration::from_secs(3));
        
        // 打印页面上所有按钮用于调试
        let debug_script = r#"
            (function() {
                const buttons = Array.from(document.querySelectorAll('button'));
                return buttons.map((b, i) => `Button ${i}: "${b.innerText.trim()}" type=${b.type} class=${b.className.substring(0,50)}`).join('\n');
            })()
        "#;
        if let Ok(result) = tab.evaluate(debug_script, true) {
            if let Some(value) = result.value {
                if let Some(s) = value.as_str() {
                    eprintln!("[SSO] 页面上的按钮:\n{}", s);
                }
            }
        }
        
        // 第一步：点击"确认并继续"按钮
        for attempt in 1..=5 {
            eprintln!("[SSO] 尝试点击「确认并继续」按钮 ({}/5)...", attempt);
            
            let confirm_continue_script = r#"
                (function() {
                    // 优先使用 id 选择器（最精确）
                    const btnById = document.querySelector('#cli_verification_btn');
                    if (btnById) {
                        btnById.click();
                        return 'clicked: #cli_verification_btn';
                    }
                    
                    const buttons = Array.from(document.querySelectorAll('button, input[type="submit"]'));
                    for (const btn of buttons) {
                        const text = (btn.innerText || btn.value || '').trim();
                        const textLower = text.toLowerCase();
                        if (text.includes('确认并继续') || 
                            text.includes('确认') ||
                            textLower.includes('confirm and continue') ||
                            textLower.includes('confirm')) {
                            btn.click();
                            return 'clicked: ' + text;
                        }
                    }
                    
                    const selectors = [
                        'button[data-analytics="confirmDeviceButton"]',
                        'button[data-analytics="accept-user-code"]',
                        'button[type="submit"]',
                        'input[type="submit"]'
                    ];
                    
                    for (const sel of selectors) {
                        const btn = document.querySelector(sel);
                        if (btn) {
                            btn.click();
                            return 'clicked selector: ' + sel;
                        }
                    }
                    
                    return 'no button found';
                })()
            "#;

            match tab.evaluate(confirm_continue_script, true) {
                Ok(result) => {
                    if let Some(value) = &result.value {
                        let result_str = value.as_str().unwrap_or("");
                        eprintln!("[SSO] 点击结果: {}", result_str);
                        if result_str.starts_with("clicked") {
                            eprintln!("[SSO] ✅ 成功点击「确认并继续」按钮");
                            break;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[SSO] ⚠️ evaluate 失败: {}", e);
                }
            }
            
            if attempt < 5 {
                std::thread::sleep(std::time::Duration::from_secs(2));
            }
        }
        
        // 等待页面跳转到"允许访问"页面
        std::thread::sleep(std::time::Duration::from_secs(3));
        
        // 第二步：点击"允许访问"按钮
        for attempt in 1..=5 {
            eprintln!("[SSO] 尝试点击「允许访问」按钮 ({}/5)...", attempt);
            
            let allow_script = r#"
                (function() {
                    const buttons = Array.from(document.querySelectorAll('button, input[type="submit"]'));
                    for (const btn of buttons) {
                        const text = (btn.innerText || btn.value || '').trim();
                        const textLower = text.toLowerCase();
                        if (text.includes('允许访问') || 
                            text.includes('允许') ||
                            textLower.includes('allow access') ||
                            textLower.includes('allow')) {
                            btn.click();
                            return 'clicked: ' + text;
                        }
                    }
                    
                    const selectors = [
                        'button[data-testid="allow-button"]',
                        'button[type="submit"]',
                        'input[type="submit"]'
                    ];
                    
                    for (const sel of selectors) {
                        const btn = document.querySelector(sel);
                        if (btn) {
                            btn.click();
                            return 'clicked selector: ' + sel;
                        }
                    }
                    
                    return 'no button found';
                })()
            "#;

            match tab.evaluate(allow_script, true) {
                Ok(result) => {
                    if let Some(value) = &result.value {
                        let result_str = value.as_str().unwrap_or("");
                        eprintln!("[SSO] 点击结果: {}", result_str);
                        if result_str.starts_with("clicked") {
                            eprintln!("[SSO] ✅ 成功点击「允许访问」按钮");
                            break;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[SSO] ⚠️ evaluate 失败: {}", e);
                }
            }
            
            if attempt < 5 {
                std::thread::sleep(std::time::Duration::from_secs(2));
            }
        }
        
        std::thread::sleep(std::time::Duration::from_secs(3));

        // 轮询获取 Token
        eprintln!("[SSO] 轮询获取 Token...");
        let interval = device_auth.interval.unwrap_or(1);
        let token = self.poll_for_token(
            &client.client_id,
            &client.client_secret,
            &device_auth.device_code,
            interval,
        ).await?;

        eprintln!("[SSO] ✅ 成功获取 refresh token: {}...", &token.refresh_token[..30.min(token.refresh_token.len())]);

        Ok(SsoTokenResult {
            access_token: token.access_token,
            refresh_token: token.refresh_token,
            client_id: client.client_id.clone(),
            client_secret: client.client_secret.clone(),
        })
    }

    /// 自动完成 SSO 授权（新开浏览器，需要登录）
    pub async fn auto_authorize(
        &self,
        email: &str,
        password: &str,
        browser_config: BrowserConfig,
    ) -> Result<SsoTokenResult> {
        eprintln!("[SSO] Starting SSO authorization for {} (new browser)", email);

        // Step 1: 注册客户端
        eprintln!("[SSO] Step 1: Registering OIDC client...");
        let client = self.register_client().await?;

        // Step 2: 发起设备授权
        eprintln!("[SSO] Step 2: Starting device authorization...");
        let device_auth = self.start_device_authorization(&client.client_id, &client.client_secret).await?;
        eprintln!("[SSO] Verification URL: {}", device_auth.verification_uri_complete);

        // Step 3: 新开浏览器完成授权
        eprintln!("[SSO] Step 3: Opening new browser for authorization...");
        let automation = BrowserAutomation::new(browser_config);
        let browser = automation.launch_browser()?;
        let tab = browser.new_tab().context("Failed to create new tab")?;

        automation.apply_fingerprint_protection(&tab)?;

        tab.navigate_to(&device_auth.verification_uri_complete)
            .context("Failed to navigate to verification URL")?;
        tab.wait_until_navigated()?;

        std::thread::sleep(std::time::Duration::from_secs(3));

        // 需要登录
        eprintln!("[SSO] Entering login credentials...");
        
        // 输入邮箱
        let email_script = format!(r#"
            (function() {{
                const input = document.querySelector('input[type="email"]') ||
                             document.querySelector('input[name="email"]');
                if (input) {{
                    input.focus();
                    const nativeInputValueSetter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, "value").set;
                    nativeInputValueSetter.call(input, "{}");
                    input.dispatchEvent(new Event('input', {{ bubbles: true }}));
                    input.dispatchEvent(new Event('change', {{ bubbles: true }}));
                    return true;
                }}
                return false;
            }})()
        "#, email);

        tab.evaluate(&email_script, true).ok();
        std::thread::sleep(std::time::Duration::from_millis(1000));

        // 点击继续
        let continue_script = r#"
            (function() {
                const btn = document.querySelector('button[type="submit"]');
                if (btn) { btn.click(); return true; }
                return false;
            })()
        "#;
        tab.evaluate(continue_script, true).ok();
        std::thread::sleep(std::time::Duration::from_secs(3));

        // 输入密码
        let password_script = format!(r#"
            (function() {{
                const input = document.querySelector('input[type="password"]');
                if (input) {{
                    input.focus();
                    const nativeInputValueSetter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, "value").set;
                    nativeInputValueSetter.call(input, "{}");
                    input.dispatchEvent(new Event('input', {{ bubbles: true }}));
                    input.dispatchEvent(new Event('change', {{ bubbles: true }}));
                    return true;
                }}
                return false;
            }})()
        "#, password);

        tab.evaluate(&password_script, true).ok();
        std::thread::sleep(std::time::Duration::from_millis(1000));

        tab.evaluate(continue_script, true).ok();
        std::thread::sleep(std::time::Duration::from_secs(5));

        // 点击确认按钮
        let confirm_script = r#"
            (function() {
                const selectors = [
                    'button[data-testid="allow-button"]',
                    'button[data-analytics="confirmDeviceButton"]',
                    'input[type="submit"]'
                ];
                
                for (const sel of selectors) {
                    const btn = document.querySelector(sel);
                    if (btn) { btn.click(); return true; }
                }
                
                const buttons = Array.from(document.querySelectorAll('button, input[type="submit"]'));
                for (const btn of buttons) {
                    const text = (btn.innerText || btn.value || '').toLowerCase();
                    if (text.includes('confirm') || text.includes('allow')) {
                        btn.click();
                        return true;
                    }
                }
                return false;
            })()
        "#;

        tab.evaluate(confirm_script, true).ok();
        std::thread::sleep(std::time::Duration::from_secs(3));

        automation.clear_browser_data()?;

        // Step 4: 轮询获取 Token
        eprintln!("[SSO] Step 4: Polling for token...");
        let interval = device_auth.interval.unwrap_or(1);
        let token = self.poll_for_token(
            &client.client_id,
            &client.client_secret,
            &device_auth.device_code,
            interval,
        ).await?;

        eprintln!("[SSO] Successfully obtained refresh token!");

        Ok(SsoTokenResult {
            access_token: token.access_token,
            refresh_token: token.refresh_token,
            client_id: client.client_id,
            client_secret: client.client_secret,
        })
    }
}
