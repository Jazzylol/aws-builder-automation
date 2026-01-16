use crate::database::{self, DbState};
use crate::models::*;
use crate::imap_client::ImapClient;
use crate::browser_automation::BrowserAutomation;
use tauri::State;
use tauri_plugin_clipboard_manager::ClipboardExt;
use anyhow::{Result, Context, anyhow};

#[tauri::command]
pub async fn get_accounts(
    db: State<'_, DbState>,
    status_filter: Option<String>,
) -> Result<Vec<Account>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let accounts = if let Some(status) = status_filter {
        database::get_accounts_by_status(&conn, &status)
    } else {
        database::get_all_accounts(&conn)
    };

    accounts.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_account(
    db: State<'_, DbState>,
    account: NewAccount,
) -> Result<i64, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::insert_account(&conn, account).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_account(
    db: State<'_, DbState>,
    update: AccountUpdate,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::update_account(&conn, update).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_account(
    db: State<'_, DbState>,
    id: i64,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::delete_account(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_all_accounts(
    db: State<'_, DbState>,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::delete_all_accounts(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_accounts(
    db: State<'_, DbState>,
    content: String,
) -> Result<ImportResult, String> {
    let mut success_count = 0;
    let mut error_count = 0;
    let mut errors = Vec::new();

    let lines: Vec<&str> = content.lines().collect();

    for (index, line) in lines.iter().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split("----").collect();

        if parts.len() != 4 {
            error_count += 1;
            errors.push(ImportError {
                line_number: index + 1,
                content: line.to_string(),
                reason: format!("Invalid format: expected 4 fields separated by '----', got {}", parts.len()),
            });
            continue;
        }

        let email = parts[0].trim();
        let password = parts[1].trim();
        let client_id = parts[2].trim();
        let refresh_token = parts[3].trim();

        // Validate email format
        if !email.contains('@') {
            error_count += 1;
            errors.push(ImportError {
                line_number: index + 1,
                content: line.to_string(),
                reason: "Invalid email address".to_string(),
            });
            continue;
        }

        // Validate that fields are not empty
        if email.is_empty() || password.is_empty() || client_id.is_empty() || refresh_token.is_empty() {
            error_count += 1;
            errors.push(ImportError {
                line_number: index + 1,
                content: line.to_string(),
                reason: "One or more fields are empty".to_string(),
            });
            continue;
        }

        let new_account = NewAccount {
            email: email.to_string(),
            email_password: password.to_string(),
            client_id: client_id.to_string(),
            refresh_token: refresh_token.to_string(),
        };

        let conn = db.0.lock().map_err(|e| e.to_string())?;
        match database::insert_account(&conn, new_account) {
            Ok(_) => success_count += 1,
            Err(e) => {
                error_count += 1;
                errors.push(ImportError {
                    line_number: index + 1,
                    content: line.to_string(),
                    reason: format!("Database error: {}", e),
                });
            }
        }
    }

    Ok(ImportResult {
        success_count,
        error_count,
        errors,
    })
}

#[tauri::command]
pub async fn get_settings(
    db: State<'_, DbState>,
) -> Result<Settings, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::get_settings(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_settings(
    db: State<'_, DbState>,
    settings: Settings,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::update_settings(&conn, settings).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_registration(
    db: State<'_, DbState>,
    account_id: i64,
) -> Result<String, String> {
    // Get account details
    let account = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        database::get_account_by_id(&conn, account_id).map_err(|e| e.to_string())?
    };

    // 备份原始邮箱 OAuth2 凭证（如果还没有备份）
    if account.email_client_id.is_none() {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        database::backup_email_credentials(&conn, account_id, &account.client_id, &account.refresh_token)
            .map_err(|e| e.to_string())?;
    }

    // Update status to in_progress
    {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        database::update_account(
            &conn,
            AccountUpdate {
                id: account_id,
                email: None,
                email_password: None,
                client_id: None,
                refresh_token: None,
                kiro_password: None,
                kiro_refresh_token: None,
                status: Some(AccountStatus::InProgress),
                error_reason: None,
            },
        )
        .map_err(|e| e.to_string())?;
    }

    // Get browser settings
    let settings = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        database::get_settings(&conn).map_err(|e| e.to_string())?
    };

    // Generate random English name for registration
    let random_name = generate_random_english_name();

    // Start registration process
    let result = perform_registration(
        &account.email,
        &account.email_password,
        &account.client_id,
        &account.refresh_token,
        &random_name,
        settings.browser_mode,
    ).await;

    match result {
        Ok(result_str) => {
            // Parse result - format: "password|||refreshToken|||clientId|||clientSecret" or "password|||token" or just "password"
            let parts: Vec<&str> = result_str.split("|||").collect();
            let kiro_password = parts[0].to_string();
            
            let (kiro_refresh_token, sso_client_id, sso_client_secret) = if parts.len() >= 4 {
                // 新格式: password|||refreshToken|||clientId|||clientSecret
                (Some(parts[1].to_string()), Some(parts[2].to_string()), Some(parts[3].to_string()))
            } else if parts.len() == 2 {
                // 旧格式: password|||token
                (Some(parts[1].to_string()), None, None)
            } else {
                (None, None, None)
            };

            // Update account with success
            let conn = db.0.lock().map_err(|e| e.to_string())?;
            database::update_account(
                &conn,
                AccountUpdate {
                    id: account_id,
                    email: None,
                    email_password: None,
                    client_id: sso_client_id,  // 更新为 SSO clientId
                    refresh_token: sso_client_secret,  // 暂存 clientSecret 到 refresh_token 字段
                    kiro_password: Some(kiro_password.clone()),
                    kiro_refresh_token,
                    status: Some(AccountStatus::Registered),
                    error_reason: None,
                },
            )
            .map_err(|e| e.to_string())?;

            Ok(format!("Registration completed successfully. Password: {}", kiro_password))
        }
        Err(e) => {
            // Update account with error
            let conn = db.0.lock().map_err(|e| e.to_string())?;
            database::update_account(
                &conn,
                AccountUpdate {
                    id: account_id,
                    email: None,
                    email_password: None,
                    client_id: None,
                    refresh_token: None,
                    kiro_password: None,
                    kiro_refresh_token: None,
                    status: Some(AccountStatus::Error),
                    error_reason: Some(e.to_string()),
                },
            )
            .map_err(|e| e.to_string())?;

            Err(e.to_string())
        }
    }
}

async fn perform_registration(
    email: &str,
    _email_password: &str,
    client_id: &str,
    refresh_token: &str,
    name: &str,
    browser_mode: BrowserMode,
) -> Result<String> {
    let (width, height) = BrowserAutomation::generate_random_window_size();
    let os_version = BrowserAutomation::generate_random_os_version();

    let config = BrowserConfig {
        mode: browser_mode,
        os: "Windows".to_string(),
        os_version,
        device_type: "PC".to_string(),
        language: "zh-CN".to_string(),
        window_width: width,
        window_height: height,
    };

    let automation = BrowserAutomation::new(config.clone());
    let browser = automation.launch_browser()?;
    let tab = browser.new_tab().context("Failed to create new tab")?;

    // Apply fingerprint protection
    automation.apply_fingerprint_protection(&tab)?;

    eprintln!("[注册] 开始注册流程，邮箱: {}", email);

    // Navigate to signin page
    eprintln!("[注册] 导航到登录页面...");
    tab.navigate_to("https://app.kiro.dev/signin")
        .context("Failed to navigate to signin page")?;
    tab.wait_until_navigated()?;

    random_delay(2000, 4000);  // 🔥 随机延迟 2-4 秒

    // Click the third button (Google sign in button)
    eprintln!("[注册] 查找 Builder ID 按钮...");
    let google_button_xpath = "/html/body/div[2]/div/div[1]/main/div/div/div/div/div/div/div/div[1]/button[3]";

    if automation.wait_for_element(&tab, google_button_xpath, 10).await? {
        eprintln!("[注册] 点击 Builder ID 按钮...");
        automation.click_element(&tab, google_button_xpath)?;
        random_delay(3000, 5000);  // 🔥 随机延迟 3-5 秒
    } else {
        eprintln!("[注册] 错误: Builder ID 按钮未找到");
        return Err(anyhow!("Google sign-in button not found"));
    }

    // Wait for email input page
    eprintln!("[注册] 等待邮箱输入页面...");
    let email_page_xpath = "/html/body/div[2]/div[2]/div[1]/div/div/div/form/div/div/div/div/div/div/div/div";

    if automation.wait_for_element(&tab, email_page_xpath, 15).await? {
        random_delay(300, 800);  // 🔥 随机延迟 0.3-0.8 秒

        // Input email
        eprintln!("[注册] 输入邮箱: {}", email);
        let email_input_xpath = "/html/body/div[2]/div[2]/div[1]/div/div/div/form/div/div/div/div/div/div/div/div/div[2]/div/div[2]/div/div/div/div/div/input";
        automation.input_text(&tab, email_input_xpath, email)?;
        random_delay(1500, 2500);  // 🔥 随机延迟 1.5-2.5 秒

        // Click continue button
        eprintln!("[注册] 点击继续按钮...");
        let continue_button_xpath = "/html/body/div[2]/div[2]/div[1]/div/div/div/form/div/div/div/div/div/div/div/div/div[3]/button";
        automation.click_element(&tab, continue_button_xpath)?;
        random_delay(3000, 5000);  // 🔥 随机延迟 3-5 秒
    } else {
        eprintln!("[注册] 错误: 邮箱输入页面未找到");
        return Err(anyhow!("Email input page not found"));
    }

    // ========== 正确流程：邮箱 → 姓名 → 验证码 → 密码 ==========
    
    // Step 2: 等待姓名输入页面
    eprintln!("[注册] 等待姓名输入页面...");
    let name_page_xpath = "/html/body/div[2]/div/div/div[2]/div/div/div/div[2]/div";

    if automation.wait_for_element(&tab, name_page_xpath, 15).await? {
        eprintln!("[注册] ✅ 姓名输入页面已找到");
        random_delay(400, 900);  // 🔥 随机延迟 0.4-0.9 秒

        // Input name
        eprintln!("[注册] 输入姓名: {}", name);
        let name_input_xpath = "/html/body/div[2]/div/div/div[1]/div/div/form/fieldset/div/div/div/div/div/div/div/div/div[3]/div/div[2]/div/div/div/div/div/input";
        automation.input_text(&tab, name_input_xpath, name)?;
        random_delay(1500, 2500);  // 🔥 随机延迟 1.5-2.5 秒

        // Click continue button
        eprintln!("[注册] 点击继续按钮...");
        let continue_button_xpath = "/html/body/div[2]/div/div/div[1]/div/div/form/fieldset/div/div/div/div/div/div/div/div/div[4]/button";
        automation.click_element(&tab, continue_button_xpath)?;
        random_delay(3000, 5000);  // 🔥 随机延迟 3-5 秒
    } else {
        eprintln!("[注册] ❌ 姓名输入页面未找到");
        return Err(anyhow!("Name input page not found"));
    }

    // Step 3: 等待验证码输入页面
    eprintln!("[注册] 等待验证码输入页面...");
    let verification_page_xpath = "/html/body/div[2]/div/div/div[1]/div/div/div[2]/form/fieldset/div/div/div/div/div/div";

    if automation.wait_for_element(&tab, verification_page_xpath, 15).await? {
        eprintln!("[注册] ✅ 验证码输入页面已加载");
        random_delay(400, 900);  // 🔥 随机延迟 0.4-0.9 秒

        // Fetch verification code using IMAP
        eprintln!("[注册] 开始通过 IMAP 获取验证码...");
        eprintln!("[注册] 邮箱: {}", email);
        eprintln!("[注册] client_id: {}...", &client_id[..20.min(client_id.len())]);
        let imap_client = ImapClient::new();

        let verification_code = match imap_client
            .wait_for_verification_code(client_id, refresh_token, email, 60)
            .await
        {
            Ok(code) => {
                eprintln!("[注册] ✅ 成功获取验证码: {}", code);
                code
            },
            Err(e) => {
                eprintln!("[注册] ⚠️ 首次获取验证码失败: {}", e);
                // If no code received after 60 seconds, click resend button
                eprintln!("[注册] 尝试点击重新发送按钮...");
                let resend_button_xpath = "/html/body/div[2]/div/div/div[1]/div/div/div[2]/form/fieldset/div/div/div/div/div/div/div[3]/div/div[2]/div/div/div/div/div/div[1]/div/div[2]/button";
                automation.click_element(&tab, resend_button_xpath).ok();
                random_delay(4000, 6000);  // 🔥 随机延迟 4-6 秒

                // Wait again for verification code
                eprintln!("[注册] 再次等待验证码...");
                imap_client
                    .wait_for_verification_code(client_id, refresh_token, email, 60)
                    .await?
            }
        };

        // Input verification code
        eprintln!("[注册] 正在输入验证码: {}", verification_code);
        let code_input_xpath = "/html/body/div[2]/div/div/div[1]/div/div/div[2]/form/fieldset/div/div/div/div/div/div/div[3]/div/div[2]/div/div/div/div/div/div[1]/div/div[1]/div/input";
        automation.input_text(&tab, code_input_xpath, &verification_code)?;
        random_delay(1500, 2500);  // 🔥 随机延迟 1.5-2.5 秒

        // Click continue button
        eprintln!("[注册] 点击继续按钮...");
        let continue_button_xpath = "/html/body/div[2]/div/div/div[1]/div/div/div[2]/form/fieldset/div/div/div/div/div/div/div[4]/button";
        automation.click_element(&tab, continue_button_xpath)?;
        random_delay(3000, 5000);  // 🔥 随机延迟 3-5 秒
    } else {
        eprintln!("[注册] ❌ 验证码输入页面未找到");
        return Err(anyhow!("Verification code page not found"));
    }

    // Step 4: 等待密码设置页面
    eprintln!("[注册] 等待密码设置页面...");
    let password_page_xpath = "/html/body/div[2]/div[2]/div[1]/div/div/div/form/div/div/div/div/div/div/div/div[2]/div[3]/button";

    if automation.wait_for_element(&tab, password_page_xpath, 15).await? {
        eprintln!("[注册] ✅ 密码设置页面已加载");
        random_delay(500, 1000);  // 🔥 随机延迟 0.5-1 秒

        // Generate a secure random password
        let password = generate_secure_password();
        eprintln!("[注册] 生成随机密码: {}", password);

        // Input password
        eprintln!("[注册] 输入密码...");
        let password_input_xpath = "/html/body/div[2]/div[2]/div[1]/div/div/div/form/div/div/div/div/div/div/div/div[1]/div[3]/div/div[2]/div/div/div/div/div/span/span/div/input";
        automation.input_text(&tab, password_input_xpath, &password)?;
        random_delay(1500, 2500);  // 🔥 随机延迟 1.5-2.5 秒

        // Input confirm password
        eprintln!("[注册] 输入确认密码...");
        let confirm_password_xpath = "/html/body/div[2]/div[2]/div[1]/div/div/div/form/div/div/div/div/div/div/div/div[1]/div[4]/div/div[2]/div/div/div/div/div/input";
        automation.input_text(&tab, confirm_password_xpath, &password)?;
        random_delay(1500, 2500);  // 🔥 随机延迟 1.5-2.5 秒

        // Click continue button
        eprintln!("[注册] 点击继续按钮完成注册...");
        let continue_button_xpath = "/html/body/div[2]/div[2]/div[1]/div/div/div/form/div/div/div/div/div/div/div/div[2]/div[3]/button";
        automation.click_element(&tab, continue_button_xpath)?;
        random_delay(3000, 5000);  // 🔥 随机延迟 3-5 秒

        // Wait for success page or Kiro main page
        eprintln!("[注册] 等待注册完成...");
        random_delay(4000, 6000);  // 🔥 随机延迟 4-6 秒
        
        // 直接进入 SSO 授权获取 RefreshToken（跳过 Cookies 提取，因为提取不到）
        eprintln!("[注册] 开始 SSO 授权获取 RefreshToken...");
        
        let sso_auth = crate::sso_auth::SsoAuth::new();
        match sso_auth.authorize_with_existing_tab(&tab).await {
            Ok(sso_result) => {
                eprintln!("[注册] ✅ SSO 授权成功！");
                eprintln!("[注册] RefreshToken: {}...", &sso_result.refresh_token[..30.min(sso_result.refresh_token.len())]);
                // 返回格式: password|||refreshToken|||clientId|||clientSecret
                automation.clear_browser_data()?;
                return Ok(format!("{}|||{}|||{}|||{}", 
                    password, 
                    sso_result.refresh_token,
                    sso_result.client_id,
                    sso_result.client_secret
                ));
            }
            Err(e) => {
                eprintln!("[注册] ❌ SSO 授权失败: {}", e);
                // SSO 失败，仅返回密码
                automation.clear_browser_data()?;
                return Ok(password);
            }
        }
    } else {
        eprintln!("[注册] ❌ 密码设置页面未找到");
        return Err(anyhow!("Password input page not found"));
    }
}

fn generate_secure_password() -> String {
    use rand::Rng;
    
    // 定义字符集
    const UPPERCASE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    const LOWERCASE: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
    const DIGITS: &[u8] = b"0123456789";
    const SPECIAL: &[u8] = b"!@#$%^&*";
    
    let mut rng = rand::thread_rng();
    let mut password = Vec::with_capacity(16);
    
    // 确保至少包含一个大写字母
    password.push(UPPERCASE[rng.gen_range(0..UPPERCASE.len())] as char);
    // 确保至少包含一个小写字母
    password.push(LOWERCASE[rng.gen_range(0..LOWERCASE.len())] as char);
    // 确保至少包含一个数字
    password.push(DIGITS[rng.gen_range(0..DIGITS.len())] as char);
    // 确保至少包含一个特殊符号
    password.push(SPECIAL[rng.gen_range(0..SPECIAL.len())] as char);
    
    // 剩余12个字符从所有字符集中随机选择
    let all_chars: Vec<u8> = [UPPERCASE, LOWERCASE, DIGITS, SPECIAL].concat();
    for _ in 0..12 {
        let idx = rng.gen_range(0..all_chars.len());
        password.push(all_chars[idx] as char);
    }
    
    // 打乱密码顺序
    use rand::seq::SliceRandom;
    password.shuffle(&mut rng);
    
    password.into_iter().collect()
}

/// 🔥 生成随机延迟（模拟真实用户操作）
fn random_delay(min_ms: u64, max_ms: u64) {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let delay_ms = rng.gen_range(min_ms..=max_ms);
    std::thread::sleep(std::time::Duration::from_millis(delay_ms));
}

/// 生成随机英文名（扩大姓名库，确保不重复）
fn generate_random_english_name() -> String {
    use rand::Rng;
    
    // 🔥 扩大英文名字库（从70个增加到150个）
    const FIRST_NAMES: &[&str] = &[
        // 男性名字
        "James", "John", "Robert", "Michael", "William", "David", "Richard", "Joseph", "Thomas", "Charles",
        "Christopher", "Daniel", "Matthew", "Anthony", "Mark", "Donald", "Steven", "Paul", "Andrew", "Joshua",
        "Liam", "Noah", "Oliver", "Elijah", "Lucas", "Mason", "Logan", "Alexander", "Ethan", "Jacob",
        "Benjamin", "Henry", "Sebastian", "Jack", "Aiden", "Owen", "Samuel", "Ryan", "Nathan", "Caleb",
        "Dylan", "Isaac", "Gabriel", "Christian", "Jonathan", "Tyler", "Austin", "Kevin", "Zachary", "Brandon",
        "Jordan", "Kyle", "Justin", "Aaron", "Eric", "Adam", "Jason", "Brian", "Sean", "Nicholas",
        "Ian", "Connor", "Hunter", "Cameron", "Evan", "Adrian", "Gavin", "Landon", "Wyatt", "Tristan",
        "Colin", "Blake", "Carson", "Dominic", "Maxwell", "Miles", "Sawyer", "Easton", "Jaxon", "Bentley",
        // 女性名字
        "Mary", "Patricia", "Jennifer", "Linda", "Barbara", "Elizabeth", "Susan", "Jessica", "Sarah", "Karen",
        "Nancy", "Lisa", "Betty", "Margaret", "Sandra", "Ashley", "Kimberly", "Emily", "Donna", "Michelle",
        "Emma", "Olivia", "Ava", "Isabella", "Sophia", "Mia", "Charlotte", "Amelia", "Harper", "Evelyn",
        "Abigail", "Ella", "Scarlett", "Grace", "Chloe", "Victoria", "Riley", "Aria", "Lily", "Aubrey",
        "Zoey", "Penelope", "Lillian", "Addison", "Layla", "Natalie", "Camila", "Hannah", "Brooklyn", "Zoe",
        "Nora", "Leah", "Savannah", "Audrey", "Claire", "Eleanor", "Skylar", "Ellie", "Samantha", "Stella",
        "Paisley", "Violet", "Mila", "Allison", "Alexa", "Anna", "Hazel", "Aaliyah", "Ariana", "Lucy",
    ];
    
    // 🔥 扩大英文姓氏库（从50个增加到100个）
    const LAST_NAMES: &[&str] = &[
        "Smith", "Johnson", "Williams", "Brown", "Jones", "Garcia", "Miller", "Davis", "Rodriguez", "Martinez",
        "Hernandez", "Lopez", "Gonzalez", "Wilson", "Anderson", "Thomas", "Taylor", "Moore", "Jackson", "Martin",
        "Lee", "Perez", "Thompson", "White", "Harris", "Sanchez", "Clark", "Ramirez", "Lewis", "Robinson",
        "Walker", "Young", "Allen", "King", "Wright", "Scott", "Torres", "Nguyen", "Hill", "Flores",
        "Green", "Adams", "Nelson", "Baker", "Hall", "Rivera", "Campbell", "Mitchell", "Carter", "Roberts",
        "Phillips", "Evans", "Turner", "Diaz", "Parker", "Cruz", "Edwards", "Collins", "Reyes", "Stewart",
        "Morris", "Morales", "Murphy", "Cook", "Rogers", "Gutierrez", "Ortiz", "Morgan", "Cooper", "Peterson",
        "Bailey", "Reed", "Kelly", "Howard", "Ramos", "Kim", "Cox", "Ward", "Richardson", "Watson",
        "Brooks", "Chavez", "Wood", "James", "Bennett", "Gray", "Mendoza", "Ruiz", "Hughes", "Price",
        "Alvarez", "Castillo", "Sanders", "Patel", "Myers", "Long", "Ross", "Foster", "Jimenez", "Powell",
    ];
    
    let mut rng = rand::thread_rng();
    let first_name = FIRST_NAMES[rng.gen_range(0..FIRST_NAMES.len())];
    let last_name = LAST_NAMES[rng.gen_range(0..LAST_NAMES.len())];
    
    format!("{} {}", first_name, last_name)
}

#[tauri::command]
pub async fn start_batch_registration(
    db: State<'_, DbState>,
) -> Result<String, String> {
    // Get all accounts with status 'not_registered'
    let accounts = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        database::get_accounts_by_status(&conn, "not_registered").map_err(|e| e.to_string())?
    };

    if accounts.is_empty() {
        return Ok("没有需要注册的账号".to_string());
    }

    let account_ids: Vec<i64> = accounts.iter().map(|a| a.id).collect();
    start_batch_registration_by_ids_internal(db, account_ids).await
}

#[tauri::command]
pub async fn start_batch_registration_by_ids(
    db: State<'_, DbState>,
    account_ids: Vec<i64>,
) -> Result<String, String> {
    if account_ids.is_empty() {
        return Ok("没有选择需要注册的账号".to_string());
    }
    start_batch_registration_by_ids_internal(db, account_ids).await
}

async fn start_batch_registration_by_ids_internal(
    db: State<'_, DbState>,
    account_ids: Vec<i64>,
) -> Result<String, String> {
    use rand::Rng;
    
    // Get accounts by ids
    let accounts: Vec<Account> = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        let mut result = Vec::new();
        for id in &account_ids {
            if let Ok(account) = database::get_account_by_id(&conn, *id) {
                // 只处理未注册、异常或进行中的账号
                if account.status == AccountStatus::NotRegistered 
                    || account.status == AccountStatus::Error 
                    || account.status == AccountStatus::InProgress {
                    result.push(account);
                }
            }
        }
        result
    };

    if accounts.is_empty() {
        return Ok("没有需要注册的账号".to_string());
    }

    let total_count = accounts.len();
    let mut success_count = 0;
    let mut error_count = 0;
    
    // 用于记录已使用的姓名，确保不重复
    let mut used_names = std::collections::HashSet::new();

    // Get browser settings once
    let settings = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        database::get_settings(&conn).map_err(|e| e.to_string())?
    };

    // Process each account sequentially
    for (index, account) in accounts.iter().enumerate() {
        eprintln!("========================================");
        eprintln!("[批量注册] 进度: {}/{}", index + 1, total_count);
        eprintln!("[批量注册] 当前账号: {}", account.email);
        eprintln!("========================================");
        
        // Update status to in_progress
        {
            let conn = db.0.lock().map_err(|e| e.to_string())?;
            database::update_account(
                &conn,
                AccountUpdate {
                    id: account.id,
                    email: None,
                    email_password: None,
                    client_id: None,
                    refresh_token: None,
                    kiro_password: None,
                    kiro_refresh_token: None,
                    status: Some(AccountStatus::InProgress),
                    error_reason: None,
                },
            )
            .map_err(|e| e.to_string())?;
        }

        // Generate unique random English name (确保不重复)
        let random_name = loop {
            let name = generate_random_english_name();
            if !used_names.contains(&name) {
                used_names.insert(name.clone());
                break name;
            }
        };
        
        eprintln!("[批量注册] 使用姓名: {}", random_name);

        // Start registration process
        let result = perform_registration(
            &account.email,
            &account.email_password,
            &account.client_id,
            &account.refresh_token,
            &random_name,
            settings.browser_mode.clone(),
        ).await;

        match result {
            Ok(result_str) => {
                // Parse result - format: "password|||refreshToken|||clientId|||clientSecret" or "password|||token" or just "password"
                let parts: Vec<&str> = result_str.split("|||").collect();
                let kiro_password = parts[0].to_string();
                
                let (kiro_refresh_token, sso_client_id, sso_client_secret) = if parts.len() >= 4 {
                    // 新格式: password|||refreshToken|||clientId|||clientSecret
                    (Some(parts[1].to_string()), Some(parts[2].to_string()), Some(parts[3].to_string()))
                } else if parts.len() == 2 {
                    // 旧格式: password|||token
                    (Some(parts[1].to_string()), None, None)
                } else {
                    (None, None, None)
                };

                // Update account with success
                let conn = db.0.lock().map_err(|e| e.to_string())?;
                database::update_account(
                    &conn,
                    AccountUpdate {
                        id: account.id,
                        email: None,
                        email_password: None,
                        client_id: sso_client_id,  // 更新为 SSO clientId
                        refresh_token: sso_client_secret,  // 暂存 clientSecret 到 refresh_token 字段
                        kiro_password: Some(kiro_password),
                        kiro_refresh_token,
                        status: Some(AccountStatus::Registered),
                        error_reason: None,
                    },
                )
                .map_err(|e| e.to_string())?;

                success_count += 1;
                eprintln!("[批量注册] ✅ 账号 {} 注册成功", account.email);
            }
            Err(e) => {
                // Update account with error
                let conn = db.0.lock().map_err(|e| e.to_string())?;
                database::update_account(
                    &conn,
                    AccountUpdate {
                        id: account.id,
                        email: None,
                        email_password: None,
                        client_id: None,
                        refresh_token: None,
                        kiro_password: None,
                        kiro_refresh_token: None,
                        status: Some(AccountStatus::Error),
                        error_reason: Some(e.to_string()),
                    },
                )
                .map_err(|e| e.to_string())?;

                error_count += 1;
                eprintln!("[批量注册] ❌ 账号 {} 注册失败: {}", account.email, e);
            }
        }
        
        // 🔥 关键改进：账号之间添加随机延迟（60-180秒）
        // 这是防止被检测为批量注册的最重要措施
        if index < accounts.len() - 1 {
            let mut rng = rand::thread_rng();
            let delay_seconds = rng.gen_range(60..=180);
            eprintln!("========================================");
            eprintln!("[批量注册] ⏰ 等待 {} 秒后继续下一个账号...", delay_seconds);
            eprintln!("[批量注册] 这是为了避免被检测为批量注册");
            eprintln!("========================================");
            std::thread::sleep(std::time::Duration::from_secs(delay_seconds));
        }
    }

    Ok(format!(
        "批量注册完成！总计: {}, 成功: {}, 失败: {}",
        total_count, success_count, error_count
    ))
}

#[tauri::command]
pub async fn export_accounts(
    db: State<'_, DbState>,
    status_filter: Option<String>,
) -> Result<String, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let accounts = if let Some(status) = status_filter {
        database::get_accounts_by_status(&conn, &status)
    } else {
        database::get_all_accounts(&conn)
    };

    let accounts = accounts.map_err(|e| e.to_string())?;

    if accounts.is_empty() {
        return Ok(String::new());
    }

    let mut lines = Vec::new();
    for account in accounts {
        // Format: email----password----client_id----refresh_token
        let line = format!(
            "{}----{}----{}----{}",
            account.email,
            account.email_password,
            account.client_id,
            account.refresh_token
        );
        lines.push(line);
    }

    Ok(lines.join("\n"))
}

#[tauri::command]
pub async fn export_accounts_json(
    db: State<'_, DbState>,
    status_filter: Option<String>,
) -> Result<String, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let accounts = if let Some(status) = status_filter {
        database::get_accounts_by_status(&conn, &status)
    } else {
        database::get_all_accounts(&conn)
    };

    let accounts = accounts.map_err(|e| e.to_string())?;

    if accounts.is_empty() {
        return Ok("[]".to_string());
    }

    // 只导出已注册且有 kiro_refresh_token 的账号
    let registered_accounts: Vec<_> = accounts
        .iter()
        .filter(|a| a.status == AccountStatus::Registered && a.kiro_refresh_token.is_some())
        .collect();

    if registered_accounts.is_empty() {
        return Err("没有包含 Kiro Token 的已注册账号".to_string());
    }

    let json_array: Vec<serde_json::Value> = registered_accounts
        .iter()
        .map(|account| {
            serde_json::json!({
                "refreshToken": account.kiro_refresh_token.as_ref().unwrap_or(&String::new()),
                "clientId": account.client_id,  // SSO clientId
                "clientSecret": account.refresh_token,  // SSO clientSecret (存储在 refresh_token 字段)
                "region": "us-east-1",
                "provider": "BuilderId"
            })
        })
        .collect();

    serde_json::to_string_pretty(&json_array).map_err(|e| e.to_string())
}

/// 导出单个账号的 Kiro JSON 信息并复制到剪贴板
#[tauri::command]
pub async fn export_single_account_json(
    db: State<'_, DbState>,
    app: tauri::AppHandle,
    account_id: i64,
) -> Result<String, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let account = database::get_account_by_id(&conn, account_id).map_err(|e| e.to_string())?;

    if account.status != AccountStatus::Registered {
        return Err("只能导出已注册的账号".to_string());
    }

    if account.kiro_refresh_token.is_none() {
        return Err("该账号没有 Kiro Token，无法导出".to_string());
    }

    let json_obj = serde_json::json!({
        "refreshToken": account.kiro_refresh_token.as_ref().unwrap_or(&String::new()),
        "clientId": account.client_id,
        "clientSecret": account.refresh_token,
        "region": "us-east-1",
        "provider": "BuilderId"
    });

    let json_str = serde_json::to_string_pretty(&json_obj).map_err(|e| e.to_string())?;
    
    // 使用 Tauri 剪贴板插件复制到剪贴板
    app.clipboard().write_text(&json_str).map_err(|e| e.to_string())?;
    
    Ok("JSON 已复制到剪贴板".to_string())
}

/// 获取账号的最新验证码
#[tauri::command]
pub async fn get_latest_verification_code(
    db: State<'_, DbState>,
    account_id: i64,
) -> Result<VerificationCodeResult, String> {
    // 获取账号信息
    let account = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        database::get_account_by_id(&conn, account_id).map_err(|e| e.to_string())?
    };

    eprintln!("[验证码] 开始获取账号 {} 的最新验证码", account.email);

    // 优先使用备份的邮箱凭证，如果没有则使用当前的
    let (email_client_id, email_refresh_token) = if let (Some(cid), Some(rt)) = (&account.email_client_id, &account.email_refresh_token) {
        eprintln!("[验证码] 使用备份的邮箱 OAuth2 凭证");
        (cid.clone(), rt.clone())
    } else {
        eprintln!("[验证码] 使用当前的 OAuth2 凭证");
        (account.client_id.clone(), account.refresh_token.clone())
    };

    // 使用 IMAP 获取最新验证码
    let imap_client = ImapClient::new();
    
    match imap_client.get_verification_code_from_recent_email(
        &email_client_id,
        &email_refresh_token,
        &account.email,
    ).await {
        Ok(code) => {
            eprintln!("[验证码] ✅ 成功获取验证码: {}", code);
            Ok(VerificationCodeResult {
                code,
                timestamp: chrono::Utc::now().timestamp(),
            })
        }
        Err(e) => {
            eprintln!("[验证码] ❌ 获取失败: {}", e);
            Err(format!("获取验证码失败: {}", e))
        }
    }
}

/// 登录已注册账号并提取 Kiro refresh token
#[tauri::command]
pub async fn extract_kiro_token(
    db: State<'_, DbState>,
    account_id: i64,
) -> Result<String, String> {
    // Get account details
    let account = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        database::get_account_by_id(&conn, account_id).map_err(|e| e.to_string())?
    };

    // 检查账号是否已注册
    if account.status != AccountStatus::Registered {
        return Err("只能对已注册的账号提取 Token".to_string());
    }

    // 检查是否已有 kiro_password
    let kiro_password = account.kiro_password.ok_or("账号没有 Kiro 密码，无法登录")?;

    // Get browser settings
    let settings = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        database::get_settings(&conn).map_err(|e| e.to_string())?
    };

    // 执行登录并提取 token（传递完整的账号信息用于获取验证码）
    let result = perform_login_and_extract_token(
        &account.email,
        &kiro_password,
        &account.client_id,      // IMAP OAuth2 client_id
        &account.refresh_token,  // IMAP OAuth2 refresh_token
        settings.browser_mode,
    ).await;

    match result {
        Ok(token_result) => {
            // 解析结果 - 格式: "refreshToken|||clientId|||clientSecret" 或只有 "refreshToken"
            let parts: Vec<&str> = token_result.split("|||").collect();
            let refresh_token = parts[0].to_string();
            let (sso_client_id, sso_client_secret) = if parts.len() >= 3 {
                (Some(parts[1].to_string()), Some(parts[2].to_string()))
            } else {
                (None, None)
            };

            // 更新数据库中的凭证
            let conn = db.0.lock().map_err(|e| e.to_string())?;
            database::update_account(
                &conn,
                AccountUpdate {
                    id: account_id,
                    email: None,
                    email_password: None,
                    client_id: sso_client_id,
                    refresh_token: sso_client_secret,
                    kiro_password: None,
                    kiro_refresh_token: Some(refresh_token.clone()),
                    status: None,
                    error_reason: None,
                },
            )
            .map_err(|e| e.to_string())?;

            Ok(format!("成功提取 Kiro Token: {}...", &refresh_token[..20.min(refresh_token.len())]))
        }
        Err(e) => Err(e.to_string()),
    }
}

/// 批量提取所有已注册账号的 Kiro token
#[tauri::command]
pub async fn batch_extract_kiro_tokens(
    db: State<'_, DbState>,
) -> Result<String, String> {
    // 获取所有已注册但没有 kiro_refresh_token 的账号
    let accounts = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        database::get_accounts_by_status(&conn, "registered").map_err(|e| e.to_string())?
    };

    let accounts_without_token: Vec<_> = accounts
        .into_iter()
        .filter(|a| a.kiro_refresh_token.is_none() && a.kiro_password.is_some())
        .collect();

    if accounts_without_token.is_empty() {
        return Ok("没有需要提取 Token 的账号".to_string());
    }

    let total_count = accounts_without_token.len();
    let mut success_count = 0;
    let mut error_count = 0;

    // Get browser settings
    let settings = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        database::get_settings(&conn).map_err(|e| e.to_string())?
    };

    for account in accounts_without_token {
        let kiro_password = match &account.kiro_password {
            Some(p) => p.clone(),
            None => continue,
        };

        eprintln!("Extracting token for account: {}", account.email);

        let result = perform_login_and_extract_token(
            &account.email,
            &kiro_password,
            &account.client_id,      // IMAP OAuth2 client_id
            &account.refresh_token,  // IMAP OAuth2 refresh_token
            settings.browser_mode.clone(),
        ).await;

        match result {
            Ok(token_result) => {
                // 解析结果 - 格式: "refreshToken|||clientId|||clientSecret" 或只有 "refreshToken"
                let parts: Vec<&str> = token_result.split("|||").collect();
                let refresh_token = parts[0].to_string();
                let (sso_client_id, sso_client_secret) = if parts.len() >= 3 {
                    (Some(parts[1].to_string()), Some(parts[2].to_string()))
                } else {
                    (None, None)
                };

                let conn = db.0.lock().map_err(|e| e.to_string())?;
                database::update_account(
                    &conn,
                    AccountUpdate {
                        id: account.id,
                        email: None,
                        email_password: None,
                        client_id: sso_client_id,
                        refresh_token: sso_client_secret,
                        kiro_password: None,
                        kiro_refresh_token: Some(refresh_token),
                        status: None,
                        error_reason: None,
                    },
                )
                .map_err(|e| e.to_string())?;

                success_count += 1;
            }
            Err(e) => {
                eprintln!("Failed to extract token for {}: {}", account.email, e);
                error_count += 1;
            }
        }

        // 每个账号之间等待一下
        std::thread::sleep(std::time::Duration::from_secs(2));
    }

    Ok(format!(
        "Token 提取完成！总计: {}, 成功: {}, 失败: {}",
        total_count, success_count, error_count
    ))
}

async fn perform_login_and_extract_token(
    email: &str,
    kiro_password: &str,
    imap_client_id: &str,      // IMAP OAuth2 client_id
    imap_refresh_token: &str,  // IMAP OAuth2 refresh_token
    browser_mode: BrowserMode,
) -> Result<String> {
    eprintln!("[提取Token] 开始为已注册账号 {} 提取 Token...", email);
    
    let (width, height) = BrowserAutomation::generate_random_window_size();
    let os_version = BrowserAutomation::generate_random_os_version();

    let config = BrowserConfig {
        mode: browser_mode,
        os: "Windows".to_string(),
        os_version,
        device_type: "PC".to_string(),
        language: "zh-CN".to_string(),
        window_width: width,
        window_height: height,
    };

    let automation = BrowserAutomation::new(config);
    let browser = automation.launch_browser()?;
    let tab = browser.new_tab().context("Failed to create new tab")?;

    // Apply fingerprint protection
    automation.apply_fingerprint_protection(&tab)?;

    // Navigate to signin page
    eprintln!("[提取Token] 步骤1: 导航到登录页面...");
    tab.navigate_to("https://app.kiro.dev/signin")
        .context("Failed to navigate to signin page")?;
    tab.wait_until_navigated()?;

    std::thread::sleep(std::time::Duration::from_secs(3));

    // Click the third button (Builder ID sign in button)
    let builder_id_button_xpath = "/html/body/div[2]/div/div[1]/main/div/div/div/div/div/div/div/div[1]/button[3]";

    if automation.wait_for_element(&tab, builder_id_button_xpath, 10).await? {
        eprintln!("[提取Token] 步骤2: 点击 Builder ID 按钮...");
        automation.click_element(&tab, builder_id_button_xpath)?;
        std::thread::sleep(std::time::Duration::from_secs(4));
    } else {
        return Err(anyhow!("Builder ID sign-in button not found"));
    }

    // Wait for email input page
    let email_page_xpath = "/html/body/div[2]/div[2]/div[1]/div/div/div/form/div/div/div/div/div/div/div/div";

    if automation.wait_for_element(&tab, email_page_xpath, 15).await? {
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Input email
        eprintln!("[提取Token] 步骤3: 输入邮箱 {}...", email);
        let email_input_xpath = "/html/body/div[2]/div[2]/div[1]/div/div/div/form/div/div/div/div/div/div/div/div/div[2]/div/div[2]/div/div/div/div/div/input";
        automation.input_text(&tab, email_input_xpath, email)?;
        std::thread::sleep(std::time::Duration::from_millis(2000));

        // Click continue button
        let continue_button_xpath = "/html/body/div[2]/div[2]/div[1]/div/div/div/form/div/div/div/div/div/div/div/div/div[3]/button";
        automation.click_element(&tab, continue_button_xpath)?;
        std::thread::sleep(std::time::Duration::from_secs(4));
    } else {
        return Err(anyhow!("Email input page not found"));
    }

    // Wait for password input page (login flow, not registration)
    eprintln!("[提取Token] 步骤4: 等待密码输入页面...");
    std::thread::sleep(std::time::Duration::from_secs(1));

    // 使用 JavaScript 查找密码输入框
    let password_input_script = format!(r#"
        (function() {{
            // 查找密码输入框
            const passwordInput = document.querySelector('input[type="password"]') ||
                                 document.querySelector('input[name="password"]') ||
                                 document.querySelector('input[autocomplete="current-password"]');
            
            if (passwordInput) {{
                passwordInput.focus();
                const nativeInputValueSetter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, "value").set;
                nativeInputValueSetter.call(passwordInput, "{}");
                passwordInput.dispatchEvent(new Event('input', {{ bubbles: true }}));
                passwordInput.dispatchEvent(new Event('change', {{ bubbles: true }}));
                return true;
            }}
            return false;
        }})()
    "#, kiro_password);

    let password_result = tab.evaluate(&password_input_script, true);
    if password_result.is_err() || password_result.unwrap().value.and_then(|v| v.as_bool()) != Some(true) {
        return Err(anyhow!("Password input not found"));
    }
    eprintln!("[提取Token] 步骤5: 输入密码完成，点击继续...");

    std::thread::sleep(std::time::Duration::from_millis(2000));

    // Click continue/sign in button - 尝试多种方式
    // 方式1: 使用 XPath 点击继续按钮（与注册流程一致）
    let password_continue_xpath = "/html/body/div[2]/div[2]/div[1]/div/div/div/form/div/div/div/div/div/div/div/div/div[3]/button";
    
    let clicked = if let Ok(true) = automation.wait_for_element(&tab, password_continue_xpath, 5).await {
        automation.click_element(&tab, password_continue_xpath).is_ok()
    } else {
        false
    };

    // 方式2: 如果 XPath 失败，使用 JavaScript 查找按钮
    if !clicked {
        eprintln!("[提取Token] XPath 点击失败，尝试 JavaScript 方式...");
        let sign_in_script = r#"
            (function() {
                // 优先使用 data-testid 选择器（最精确）
                const signInBtn = document.querySelector('button[data-testid="test-primary-button"]') ||
                                 document.querySelector('button[type="submit"]') ||
                                 document.querySelector('form button') ||
                                 Array.from(document.querySelectorAll('button')).find(b => 
                                     b.innerText.includes('继续') ||
                                     b.innerText.toLowerCase().includes('sign in') ||
                                     b.innerText.toLowerCase().includes('continue') ||
                                     b.innerText.includes('登录'));
                if (signInBtn) {
                    signInBtn.click();
                    return true;
                }
                return false;
            })()
        "#;
        tab.evaluate(sign_in_script, true).ok();
    }

    eprintln!("[提取Token] 等待页面响应...");
    std::thread::sleep(std::time::Duration::from_secs(3));

    // 步骤6: 获取验证码
    eprintln!("[提取Token] 步骤6: 开始获取验证码...");
    
    // 等待验证码邮件到达
    std::thread::sleep(std::time::Duration::from_secs(5));
    
    // 通过 IMAP 获取验证码
    let imap_client = crate::imap_client::ImapClient::new();
    let verification_code = imap_client
        .wait_for_verification_code(imap_client_id, imap_refresh_token, email, 120)
        .await
        .context("Failed to get verification code from email")?;
    
    eprintln!("[提取Token] 步骤7: 获取到验证码: {}", verification_code);
    
    // 输入验证码
    let code_input_script = format!(r#"
        (function() {{
            const codeInput = document.querySelector('input[name="code"]') ||
                             document.querySelector('input[autocomplete="one-time-code"]') ||
                             document.querySelector('fieldset input[type="text"]') ||
                             document.querySelector('form input[type="text"]:not([type="email"])');
            
            if (codeInput) {{
                codeInput.focus();
                const nativeInputValueSetter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, "value").set;
                nativeInputValueSetter.call(codeInput, "{}");
                codeInput.dispatchEvent(new Event('input', {{ bubbles: true }}));
                codeInput.dispatchEvent(new Event('change', {{ bubbles: true }}));
                return true;
            }}
            return false;
        }})()
    "#, verification_code);

    tab.evaluate(&code_input_script, true).ok();
    std::thread::sleep(std::time::Duration::from_millis(1000));
    
    eprintln!("[提取Token] 步骤9: 验证码输入完成，点击继续...");

    // 点击继续按钮
    let verify_continue_script = r#"
        (function() {
            const continueBtn = document.querySelector('button[data-testid="test-primary-button"]') ||
                               document.querySelector('button[type="submit"]') ||
                               document.querySelector('fieldset button') ||
                               Array.from(document.querySelectorAll('button')).find(b => 
                                   b.innerText.includes('继续') ||
                                   b.innerText.toLowerCase().includes('continue'));
            if (continueBtn) {
                continueBtn.click();
                return true;
            }
            return false;
        })()
    "#;
    tab.evaluate(verify_continue_script, true).ok();
    
    eprintln!("[提取Token] 步骤10: 准备 SSO 授权...");
    
    // 注册 SSO 客户端并获取设备码
    let sso_auth = crate::sso_auth::SsoAuth::new();
    
    let client = sso_auth.register_client().await
        .map_err(|e| anyhow!("注册客户端失败: {}", e))?;
    
    let device_auth = sso_auth.start_device_authorization(&client.client_id, &client.client_secret).await
        .map_err(|e| anyhow!("获取设备码失败: {}", e))?;
    
    eprintln!("[SSO] Device code: {}", device_auth.user_code);
    
    // 直接在当前浏览器中导航到授权页面
    eprintln!("[SSO] 导航到授权页面: {}", device_auth.verification_uri_complete);
    let nav_script = format!(r#"window.location.href = "{}";"#, device_auth.verification_uri_complete);
    tab.evaluate(&nav_script, true).ok();
    
    eprintln!("[SSO] ========================================");
    eprintln!("[SSO] 请在浏览器中手动完成以下操作：");
    eprintln!("[SSO] 1. 点击「确认并继续」按钮");
    eprintln!("[SSO] 2. 点击「允许访问」按钮");
    eprintln!("[SSO] ========================================");
    
    // 轮询获取 Token（等待用户手动完成授权）
    eprintln!("[SSO] 等待用户手动完成授权，轮询获取 Token...");
    let interval = device_auth.interval.unwrap_or(2);
    
    let token = sso_auth.poll_for_token_public(
        &client.client_id,
        &client.client_secret,
        &device_auth.device_code,
        interval,
    ).await;
    
    match token {
        Ok(token_result) => {
            eprintln!("[提取Token] ✅ 成功获取 RefreshToken: {}...", 
                &token_result.refresh_token[..30.min(token_result.refresh_token.len())]);
            automation.clear_browser_data()?;
            return Ok(format!("{}|||{}|||{}", 
                token_result.refresh_token,
                client.client_id,
                client.client_secret
            ));
        }
        Err(e) => {
            eprintln!("[提取Token] ❌ 获取 Token 失败: {}", e);
            automation.clear_browser_data()?;
            return Err(anyhow!("获取 Token 失败: {}", e));
        }
    }
}


/// 批量更新邮箱 OAuth2 凭证
#[tauri::command]
pub async fn update_email_credentials(
    db: State<'_, DbState>,
    content: String,
) -> Result<String, String> {
    let mut success_count = 0;
    let mut error_count = 0;
    let mut not_found_count = 0;

    let lines: Vec<&str> = content.lines().collect();

    for (index, line) in lines.iter().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split("----").collect();

        if parts.len() != 4 {
            eprintln!("[更新凭证] 第 {} 行格式错误: 期望4个字段，实际{}个", index + 1, parts.len());
            error_count += 1;
            continue;
        }

        let email = parts[0].trim();
        let _password = parts[1].trim();
        let client_id = parts[2].trim();
        let refresh_token = parts[3].trim();

        // 查找对应的账号
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        
        let account_result: Result<i64, rusqlite::Error> = conn.query_row(
            "SELECT id FROM accounts WHERE email = ?1",
            [email],
            |row| row.get(0),
        );

        match account_result {
            Ok(account_id) => {
                // 更新邮箱凭证
                match conn.execute(
                    "UPDATE accounts SET email_client_id = ?1, email_refresh_token = ?2 WHERE id = ?3",
                    rusqlite::params![client_id, refresh_token, account_id],
                ) {
                    Ok(_) => {
                        eprintln!("[更新凭证] ✅ 成功更新账号: {}", email);
                        success_count += 1;
                    }
                    Err(e) => {
                        eprintln!("[更新凭证] ❌ 更新失败 {}: {}", email, e);
                        error_count += 1;
                    }
                }
            }
            Err(_) => {
                eprintln!("[更新凭证] ⚠️ 未找到账号: {}", email);
                not_found_count += 1;
            }
        }
    }

    Ok(format!(
        "邮箱凭证更新完成！成功: {}, 失败: {}, 未找到: {}",
        success_count, error_count, not_found_count
    ))
}

/// 生成 SSO 授权链接
#[tauri::command]
pub async fn generate_sso_auth_url(
    _db: State<'_, DbState>,
) -> Result<String, String> {
    let sso_auth = crate::sso_auth::SsoAuth::new();
    
    // 注册 SSO 客户端
    let client = sso_auth.register_client().await
        .map_err(|e| format!("注册客户端失败: {}", e))?;
    
    // 获取设备授权码
    let device_auth = sso_auth.start_device_authorization(&client.client_id, &client.client_secret).await
        .map_err(|e| format!("获取设备码失败: {}", e))?;
    
    eprintln!("[SSO] 生成授权链接成功");
    eprintln!("[SSO] Device code: {}", device_auth.user_code);
    eprintln!("[SSO] URL: {}", device_auth.verification_uri_complete);
    
    // 返回格式: url|||device_code|||client_id|||client_secret|||interval
    Ok(format!(
        "{}|||{}|||{}|||{}|||{}", 
        device_auth.verification_uri_complete,
        device_auth.device_code,
        client.client_id,
        client.client_secret,
        device_auth.interval.unwrap_or(2)
    ))
}

/// 轮询获取 SSO Token（用于手动授权后获取 Token）
#[tauri::command]
pub async fn poll_sso_token(
    db: State<'_, DbState>,
    account_id: i64,
    device_code: String,
    client_id: String,
    client_secret: String,
    interval: u64,
) -> Result<String, String> {
    eprintln!("[SSO] 开始轮询获取 Token...");
    
    let sso_auth = crate::sso_auth::SsoAuth::new();
    
    let token = sso_auth.poll_for_token_public(
        &client_id,
        &client_secret,
        &device_code,
        interval,
    ).await;
    
    match token {
        Ok(token_result) => {
            eprintln!("[SSO] ✅ 成功获取 RefreshToken");
            
            // 更新数据库
            let conn = db.0.lock().map_err(|e| e.to_string())?;
            database::update_account(
                &conn,
                AccountUpdate {
                    id: account_id,
                    email: None,
                    email_password: None,
                    client_id: Some(client_id.clone()),
                    refresh_token: Some(client_secret.clone()),
                    kiro_password: None,
                    kiro_refresh_token: Some(token_result.refresh_token.clone()),
                    status: Some(AccountStatus::Registered),
                    error_reason: None,
                },
            )
            .map_err(|e| e.to_string())?;
            
            Ok(format!("成功获取 Token: {}...", &token_result.refresh_token[..20.min(token_result.refresh_token.len())]))
        }
        Err(e) => {
            eprintln!("[SSO] ❌ 获取 Token 失败: {}", e);
            Err(format!("获取 Token 失败: {}", e))
        }
    }
}
