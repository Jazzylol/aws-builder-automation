mod database;
mod models;
mod imap_client;
mod browser_automation;
mod commands;
mod sso_auth;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            // Initialize database
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = database::init_database(&app_handle).await {
                    eprintln!("Failed to initialize database: {}", e);
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_accounts,
            commands::add_account,
            commands::update_account,
            commands::delete_account,
            commands::delete_all_accounts,
            commands::import_accounts,
            commands::get_settings,
            commands::update_settings,
            commands::start_registration,
            commands::start_batch_registration,
            commands::start_batch_registration_by_ids,
            commands::export_accounts,
            commands::export_accounts_json,
            commands::export_single_account_json,
            commands::extract_kiro_token,
            commands::batch_extract_kiro_tokens,
            commands::get_latest_verification_code,
            commands::update_email_credentials,
            commands::generate_sso_auth_url,
            commands::poll_sso_token,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
