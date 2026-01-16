Zhuanz@MacBook-Pro aws-builder-automation % npm r
un build

> tauri-app@0.1.0 build
> tsc && vite build

vite v7.2.7 building client environment for production...
✓ 1721 modules transformed.
dist/index.html                   0.44 kB │ gzip:  0.32 kB
dist/assets/index-Cr2ynSxY.css   17.67 kB │ gzip:  3.45 kB
dist/assets/index-LaPdRgKx.js   189.03 kB │ gzip: 58.99 kB
✓ built in 536ms
Zhuanz@MacBook-Pro aws-builder-automation % cd "/
Users/Zhuanz/Downloads/aws-builder-automation/src
-tauri" && cargo check
    Checking tauri-app v0.1.0 (/Users/Zhuanz/Downloads/aws-builder-automation/src-tauri)
warning: associated items `fetch_emails_via_imap` and `extract_verification_code` are never used
   --> src/imap_client.rs:71:18
    |
 14 | impl ImapClient {
    | --------------- associated items in this implementation
...
 71 |     pub async fn fetch_emails_via_imap(
    |                  ^^^^^^^^^^^^^^^^^^^^^
...
239 |     pub fn extract_verification_code(body...
    |            ^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: method `wait_for_condition` is never used
   --> src/browser_automation.rs:261:18
    |
 14 | impl BrowserAutomation {
    | ---------------------- method in this implementation
...
261 |     pub async fn wait_for_condition(
    |                  ^^^^^^^^^^^^^^^^^^

warning: method `auto_authorize` is never used
   --> src/sso_auth.rs:389:18
    |
 96 | impl SsoAuth {
    | ------------ method in this implementation
...
389 |     pub async fn auto_authorize(
    |                  ^^^^^^^^^^^^^^

warning: `tauri-app` (lib) generated 3 warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.85s
Zhuanz@MacBook-Pro src-tauri % 
Zhuanz@MacBook-Pro src-tauri % cargo check
    Checking tauri-app v0.1.0 (/Users/Zhuanz/Downloads/aws-builder-automation/src-tauri)
warning: associated items `fetch_emails_via_imap` and `extract_verification_code` are never used
   --> src/imap_client.rs:71:18
    |
 14 | impl ImapClient {
    | --------------- associated items in this implementation
...
 71 |     pub async fn fetch_emails_via_imap(
    |                  ^^^^^^^^^^^^^^^^^^^^^
...
239 |     pub fn extract_verification_code(body: ...
    |            ^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: method `wait_for_condition` is never used
   --> src/browser_automation.rs:261:18
    |
 14 | impl BrowserAutomation {
    | ---------------------- method in this implementation
...
261 |     pub async fn wait_for_condition(
    |                  ^^^^^^^^^^^^^^^^^^

warning: method `auto_authorize` is never used
   --> src/sso_auth.rs:389:18
    |
 96 | impl SsoAuth {
    | ------------ method in this implementation
...
389 |     pub async fn auto_authorize(
    |                  ^^^^^^^^^^^^^^

warning: `tauri-app` (lib) generated 3 warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.28s
Zhuanz@MacBook-Pro src-tauri % cd "/Users/Zhuanz/Dow
nloads/aws-builder-automation" && npm run build

> tauri-app@0.1.0 build
> tsc && vite build

vite v7.2.7 building client environment for production...
✓ 1721 modules transformed.
dist/index.html                   0.44 kB │ gzip:  0.32 kB
dist/assets/index-D6uzzIkh.css   17.82 kB │ gzip:  3.46 kB
dist/assets/index-Ch1jVCMl.js   189.52 kB │ gzip: 59.12 kB
✓ built in 534ms
Zhuanz@MacBook-Pro aws-builder-automation % cd "/Use
rs/Zhuanz/Downloads/aws-builder-automation/src-tauri
" && cargo check
    Checking tauri-app v0.1.0 (/Users/Zhuanz/Downloads/aws-builder-automation/src-tauri)
warning: associated items `fetch_emails_via_imap` and `extract_verification_code` are never used
   --> src/imap_client.rs:71:18
    |
 14 | impl ImapClient {
    | --------------- associated items in this implementation
...
 71 |     pub async fn fetch_emails_via_imap(
    |                  ^^^^^^^^^^^^^^^^^^^^^
...
239 |     pub fn extract_verification_code(body: ...
    |            ^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: method `wait_for_condition` is never used
   --> src/browser_automation.rs:261:18
    |
 14 | impl BrowserAutomation {
    | ---------------------- method in this implementation
...
261 |     pub async fn wait_for_condition(
    |                  ^^^^^^^^^^^^^^^^^^

warning: method `auto_authorize` is never used
   --> src/sso_auth.rs:389:18
    |
 96 | impl SsoAuth {
    | ------------ method in this implementation
...
389 |     pub async fn auto_authorize(
    |                  ^^^^^^^^^^^^^^

warning: `tauri-app` (lib) generated 3 warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.88s
Zhuanz@MacBook-Pro src-tauri % cd "/Users/Zhuanz/Dow
nloads/aws-builder-automation" && npm run build

> tauri-app@0.1.0 build
> tsc && vite build

vite v7.2.7 building client environment for production...
✓ 1721 modules transformed.
dist/index.html                   0.44 kB │ gzip:  0.32 kB
dist/assets/index-D6uzzIkh.css   17.82 kB │ gzip:  3.46 kB
dist/assets/index-BjrsA-PG.js   189.69 kB │ gzip: 59.22 kB
✓ built in 543ms
Zhuanz@MacBook-Pro aws-builder-automation % npm inst
all @tauri-apps/plugin-clipboard-manager

added 1 package, and audited 222 packages in 4s

35 packages are looking for funding
  run `npm fund` for details

4 high severity vulnerabilities

To address all issues (including breaking changes), run:
  npm audit fix --force

Run `npm audit` for details.
Zhuanz@MacBook-Pro aws-builder-automation % npm run 
build

> tauri-app@0.1.0 build
> tsc && vite build

vite v7.2.7 building client environment for production...
✓ 1722 modules transformed.
dist/index.html                   0.44 kB │ gzip:  0.32 kB
dist/assets/index-D6uzzIkh.css   17.82 kB │ gzip:  3.46 kB
dist/assets/index-sx2GT2wT.js   189.76 kB │ gzip: 59.23 kB
✓ built in 547ms
Zhuanz@MacBook-Pro aws-builder-automation % cd "/Use
rs/Zhuanz/Downloads/aws-builder-automation/src-tauri
" && cargo check
   Compiling ppv-lite86 v0.2.21
    Checking zerocopy v0.8.31
    Checking objc2-core-graphics v0.3.2
    Checking num-traits v0.2.19
    Checking zune-core v0.4.12
    Checking fax v0.2.6
    Checking quick-error v2.0.1
    Checking weezl v0.1.12
    Checking bytemuck v1.24.0
    Checking byteorder-lite v0.1.0
    Checking zune-jpeg v0.4.21
   Compiling rand_chacha v0.3.1
   Compiling rand_chacha v0.2.2
   Compiling rand v0.7.3
   Compiling rand v0.8.5
    Checking objc2-app-kit v0.3.2
    Checking pxfm v0.1.27
    Checking chrono v0.4.42
   Compiling phf_generator v0.11.3
   Compiling phf_generator v0.10.0
   Compiling phf_generator v0.8.0
   Compiling phf_macros v0.10.0
   Compiling phf_codegen v0.8.0
   Compiling string_cache_codegen v0.5.4
   Compiling phf_codegen v0.11.3
   Compiling phf_macros v0.11.3
   Compiling selectors v0.24.0
   Compiling markup5ever v0.14.1
    Checking half v2.7.1
    Checking ahash v0.8.12
    Checking hashbrown v0.14.5
    Checking rand_chacha v0.9.0
   Compiling phf v0.11.3
   Compiling phf v0.10.1
    Checking tiff v0.10.3
    Checking rand v0.9.2
    Checking hashlink v0.9.1
    Checking rusqlite v0.32.1
    Checking tungstenite v0.27.0
    Checking moxcms v0.7.11
    Checking headless_chrome v1.0.18
    Checking tauri-utils v2.8.1
   Compiling html5ever v0.29.1
   Compiling cssparser v0.29.6
   Compiling kuchikiki v0.8.8-speedreader
    Checking image v0.25.9
   Compiling tauri-plugin v2.5.2
   Compiling tauri-build v2.5.3
   Compiling tauri-codegen v2.5.2
   Compiling tauri v2.9.5
   Compiling tauri-macros v2.5.2
   Compiling tauri-plugin-fs v2.4.4
   Compiling tauri-plugin-clipboard-manager v2.3.2
   Compiling tauri-plugin-dialog v2.4.2
   Compiling tauri-plugin-opener v2.5.2
   Compiling tauri-app v0.1.0 (/Users/Zhuanz/Downloads/aws-builder-automation/src-tauri)
    Checking objc2-web-kit v0.3.2
    Checking tao v0.34.5
    Checking muda v0.17.1
    Checking window-vibrancy v0.6.0
    Checking rfd v0.15.4
    Checking arboard v3.6.1
    Checking wry v0.53.5
    Checking tauri-runtime v2.9.2
    Checking tauri-runtime-wry v2.9.3
warning: associated items `fetch_emails_via_imap` and `extract_verification_code` are never used
   --> src/imap_client.rs:71:18
    |
 14 | impl ImapClient {
    | --------------- associated items in this implementation
...
 71 |     pub async fn fetch_emails_via_imap(
    |                  ^^^^^^^^^^^^^^^^^^^^^
...
239 |     pub fn extract_verification_code(body: ...
    |            ^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: method `wait_for_condition` is never used
   --> src/browser_automation.rs:261:18
    |
 14 | impl BrowserAutomation {
    | ---------------------- method in this implementation
...
261 |     pub async fn wait_for_condition(
    |                  ^^^^^^^^^^^^^^^^^^

warning: method `auto_authorize` is never used
   --> src/sso_auth.rs:389:18
    |
 96 | impl SsoAuth {
    | ------------ method in this implementation
...
389 |     pub async fn auto_authorize(
    |                  ^^^^^^^^^^^^^^

warning: `tauri-app` (lib) generated 3 warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 17.55s
Zhuanz@MacBook-Pro src-tauri % cargo tauri dev --no-
watch &
sleep 5
kill %1 2>/dev/null || true
[1] 76566
Zhuanz@MacBook-Pro src-tauri % sleep 5
error: no such command: `tauri`

help: a command with a similar name exists: `miri`

help: view all installed commands with `cargo --list`
help: find a package to install `tauri` with `cargo search cargo-tauri`
[1]  + exit 101   cargo tauri dev --no-watch
Zhuanz@MacBook-Pro src-tauri % kill %1 2>/dev/null |
| true
Zhuanz@MacBook-Pro src-tauri % cargo build
warning: associated items `fetch_emails_via_imap` and `extract_verification_code` are never used
   --> src/imap_client.rs:71:18
    |
 14 | impl ImapClient {
    | --------------- associated items in this implementation
...
 71 |     pub async fn fetch_emails_via_imap(
    |                  ^^^^^^^^^^^^^^^^^^^^^
...
239 |     pub fn extract_verification_code(body: &str) -> Option<String> {
    |            ^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: method `wait_for_condition` is never used
   --> src/browser_automation.rs:261:18
    |
 14 | impl BrowserAutomation {
    | ---------------------- method in this implementation
...
261 |     pub async fn wait_for_condition(
    |                  ^^^^^^^^^^^^^^^^^^

warning: method `auto_authorize` is never used
   --> src/sso_auth.rs:389:18
    |
 96 | impl SsoAuth {
    | ------------ method in this implementation
...
389 |     pub async fn auto_authorize(
    |                  ^^^^^^^^^^^^^^

warning: `tauri-app` (lib) generated 3 warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.43s
Zhuanz@MacBook-Pro src-tauri % rm -rf target/debug/.
fingerprint/tauri-app*
Zhuanz@MacBook-Pro src-tauri % cargo build
   Compiling tauri-app v0.1.0 (/Users/Zhuanz/Downloads/aws-builder-automation/src-tauri)
warning: associated items `fetch_emails_via_imap` and `extract_verification_code` are never used
   --> src/imap_client.rs:71:18
    |
 14 | impl ImapClient {
    | --------------- associated items in this implementation
...
 71 |     pub async fn fetch_emails_via_imap(
    |                  ^^^^^^^^^^^^^^^^^^^^^
...
239 |     pub fn extract_verification_code(body: ...
    |            ^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: method `wait_for_condition` is never used
   --> src/browser_automation.rs:261:18
    |
 14 | impl BrowserAutomation {
    | ---------------------- method in this implementation
...
261 |     pub async fn wait_for_condition(
    |                  ^^^^^^^^^^^^^^^^^^

warning: method `auto_authorize` is never used
   --> src/sso_auth.rs:389:18
    |
 96 | impl SsoAuth {
    | ------------ method in this implementation
...
389 |     pub async fn auto_authorize(
    |                  ^^^^^^^^^^^^^^

warning: `tauri-app` (lib) generated 3 warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.87s
Zhuanz@MacBook-Pro src-tauri % cd ..
Zhuanz@MacBook-Pro aws-builder-automation % npm run tauri dev

> tauri-app@0.1.0 tauri
> tauri dev

     Running BeforeDevCommand (`npm run dev`)

> tauri-app@0.1.0 dev
> vite


  VITE v7.2.7  ready in 94 ms

  ➜  Local:   http://localhost:1420/
     Running DevCommand (`cargo  run --no-default-features --color always --`)
        Info Watching /Users/Zhuanz/Downloads/aws-builder-automation/src-tauri for changes...
warning: associated items `fetch_emails_via_imap` and `extract_verification_code` are never used
   --> src/imap_client.rs:71:18
    |
 14 | impl ImapClient {
    | --------------- associated items in this implementation
...
 71 |     pub async fn fetch_emails_via_imap(
    |                  ^^^^^^^^^^^^^^^^^^^^^
...
239 |     pub fn extract_verification_code(body: ...
    |            ^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: method `wait_for_condition` is never used
   --> src/browser_automation.rs:261:18
    |
 14 | impl BrowserAutomation {
    | ---------------------- method in this implementation
...
261 |     pub async fn wait_for_condition(
    |                  ^^^^^^^^^^^^^^^^^^

warning: method `auto_authorize` is never used
   --> src/sso_auth.rs:389:18
    |
 96 | impl SsoAuth {
    | ------------ method in this implementation
...
389 |     pub async fn auto_authorize(
    |                  ^^^^^^^^^^^^^^

warning: `tauri-app` (lib) generated 3 warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.34s
     Running `target/debug/tauri-app`
        Info File src-tauri/src/commands.rs changed. Rebuilding application...
     Running DevCommand (`cargo  run --no-default-features --color always --`)
        Info File src-tauri/src/commands.rs changed. Rebuilding application...
     Running DevCommand (`cargo  run --no-default-features --color always --`)
   Compiling tauri-app v0.1.0 (/Users/Zhuanz/Downloads/aws-builder-automation/src-tauri)
error[E0599]: no method named `clipboard` found for struct `AppHandle<R>` in the current scope
   --> src/commands.rs:822:9
    |
822 |     app.clipboard().write_text(&json_str).map_err(|e| e.to_string())?;
    |         ^^^^^^^^^ method not found in `AppHandle`
    |
   ::: /Users/Zhuanz/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tauri-plugin-clipboard-manager-2.3.2/src/lib.rs:34:8
    |
 34 |     fn clipboard(&self) -> &Clipboard<R>;
    |        --------- the method is available for `AppHandle` here
    |
    = help: items from traits can only be used if the trait is in scope
help: trait `ClipboardExt` which provides `clipboard` is implemented but not in scope; perhaps you want to import it
    |
  1 + use tauri_plugin_clipboard_manager::ClipboardExt;
    |

error[E0282]: type annotations needed
   --> src/commands.rs:822:52
    |
822 |     app.clipboard().write_text(&json_str).map_err(|e| e.to_string())?;
    |                                                    ^  - type must be known at this point
    |
help: consider giving this closure parameter an explicit type
    |
822 |     app.clipboard().write_text(&json_str).map_err(|e: /* Type */| e.to_string())?;
    |                                                     ++++++++++++

Some errors have detailed explanations: E0282, E0599.
For more information about an error, try `rustc --explain E0282`.
error: could not compile `tauri-app` (lib) due to 2 previous errors
        Info File src-tauri/src/commands.rs changed. Rebuilding application...
     Running DevCommand (`cargo  run --no-default-features --color always --`)
        Info File src-tauri/src/commands.rs changed. Rebuilding application...
     Running DevCommand (`cargo  run --no-default-features --color always --`)
   Compiling tauri-app v0.1.0 (/Users/Zhuanz/Downloads/aws-builder-automation/src-tauri)
warning: associated items `fetch_emails_via_imap` and `extract_verification_code` are never used
   --> src/imap_client.rs:71:18
    |
 14 | impl ImapClient {
    | --------------- associated items in this implementation
...
 71 |     pub async fn fetch_emails_via_imap(
    |                  ^^^^^^^^^^^^^^^^^^^^^
...
239 |     pub fn extract_verification_code(body: &str) -> Option<String> {
    |            ^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: method `wait_for_condition` is never used
   --> src/browser_automation.rs:261:18
    |
 14 | impl BrowserAutomation {
    | ---------------------- method in this implementation
...
261 |     pub async fn wait_for_condition(
    |                  ^^^^^^^^^^^^^^^^^^

warning: method `auto_authorize` is never used
   --> src/sso_auth.rs:389:18
    |
 96 | impl SsoAuth {
    | ------------ method in this implementation
...
389 |     pub async fn auto_authorize(
    |                  ^^^^^^^^^^^^^^

warning: `tauri-app` (lib) generated 3 warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.43s
     Running `target/debug/tauri-app`
11:02:04 PM [vite] (client) hmr update /src/components/AccountsTable.tsx
11:02:14 PM [vite] (client) hmr update /src/components/AccountsTable.tsx
        Info File src-tauri/src/commands.rs changed. Rebuilding application...
     Running DevCommand (`cargo  run --no-default-features --color always --`)
        Info File src-tauri/src/commands.rs changed. Rebuilding application...
     Running DevCommand (`cargo  run --no-default-features --color always --`)
   Compiling tauri-app v0.1.0 (/Users/Zhuanz/Downloads/aws-builder-automation/src-tauri)
warning: associated items `fetch_emails_via_imap` and `extract_verification_code` are never used
   --> src/imap_client.rs:71:18
    |
 14 | impl ImapClient {
    | --------------- associated items in this implementation
...
 71 |     pub async fn fetch_emails_via_imap(
    |                  ^^^^^^^^^^^^^^^^^^^^^
...
239 |     pub fn extract_verification_code(body: &str) -> Option<String> {
    |            ^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: method `wait_for_condition` is never used
   --> src/browser_automation.rs:261:18
    |
 14 | impl BrowserAutomation {
    | ---------------------- method in this implementation
...
261 |     pub async fn wait_for_condition(
    |                  ^^^^^^^^^^^^^^^^^^

warning: method `auto_authorize` is never used
   --> src/sso_auth.rs:389:18
    |
 96 | impl SsoAuth {
    | ------------ method in this implementation
...
389 |     pub async fn auto_authorize(
    |                  ^^^^^^^^^^^^^^

warning: `tauri-app` (lib) generated 3 warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.69s
     Running `target/debug/tauri-app`
        Info File src-tauri/src/commands.rs changed. Rebuilding application...
     Running DevCommand (`cargo  run --no-default-features --color always --`)
        Info File src-tauri/src/commands.rs changed. Rebuilding application...
     Running DevCommand (`cargo  run --no-default-features --color always --`)
   Compiling tauri-app v0.1.0 (/Users/Zhuanz/Downloads/aws-builder-automation/src-tauri)
warning: associated items `fetch_emails_via_imap` and `extract_verification_code` are never used
   --> src/imap_client.rs:71:18
    |
 14 | impl ImapClient {
    | --------------- associated items in this implementation
...
 71 |     pub async fn fetch_emails_via_imap(
    |                  ^^^^^^^^^^^^^^^^^^^^^
...
239 |     pub fn extract_verification_code(body: &str) -> Option<String> {
    |            ^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: method `wait_for_condition` is never used
   --> src/browser_automation.rs:261:18
    |
 14 | impl BrowserAutomation {
    | ---------------------- method in this implementation
...
261 |     pub async fn wait_for_condition(
    |                  ^^^^^^^^^^^^^^^^^^

warning: method `auto_authorize` is never used
   --> src/sso_auth.rs:389:18
    |
 96 | impl SsoAuth {
    | ------------ method in this implementation
...
389 |     pub async fn auto_authorize(
    |                  ^^^^^^^^^^^^^^

warning: `tauri-app` (lib) generated 3 warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.72s
     Running `target/debug/tauri-app`
        Info File src-tauri/src/commands.rs changed. Rebuilding application...
     Running DevCommand (`cargo  run --no-default-features --color always --`)
        Info File src-tauri/src/commands.rs changed. Rebuilding application...
     Running DevCommand (`cargo  run --no-default-features --color always --`)
   Compiling tauri-app v0.1.0 (/Users/Zhuanz/Downloads/aws-builder-automation/src-tauri)
warning: associated items `fetch_emails_via_imap` and `extract_verification_code` are never used
   --> src/imap_client.rs:71:18
    |
 14 | impl ImapClient {
    | --------------- associated items in this implementation
...
 71 |     pub async fn fetch_emails_via_imap(
    |                  ^^^^^^^^^^^^^^^^^^^^^
...
239 |     pub fn extract_verification_code(body: &str) -> Option<String> {
    |            ^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: method `wait_for_condition` is never used
   --> src/browser_automation.rs:261:18
    |
 14 | impl BrowserAutomation {
    | ---------------------- method in this implementation
...
261 |     pub async fn wait_for_condition(
    |                  ^^^^^^^^^^^^^^^^^^

warning: method `auto_authorize` is never used
   --> src/sso_auth.rs:389:18
    |
 96 | impl SsoAuth {
    | ------------ method in this implementation
...
389 |     pub async fn auto_authorize(
    |                  ^^^^^^^^^^^^^^

warning: `tauri-app` (lib) generated 3 warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.60s
     Running `target/debug/tauri-app`
        Info File src-tauri/src/commands.rs changed. Rebuilding application...
     Running DevCommand (`cargo  run --no-default-features --color always --`)
        Info File src-tauri/src/commands.rs changed. Rebuilding application...
     Running DevCommand (`cargo  run --no-default-features --color always --`)
   Compiling tauri-app v0.1.0 (/Users/Zhuanz/Downloads/aws-builder-automation/src-tauri)
warning: associated items `fetch_emails_via_imap` and `extract_verification_code` are never used
   --> src/imap_client.rs:71:18
    |
 14 | impl ImapClient {
    | --------------- associated items in this implementation
...
 71 |     pub async fn fetch_emails_via_imap(
    |                  ^^^^^^^^^^^^^^^^^^^^^
...
239 |     pub fn extract_verification_code(body: &str) -> Option<String> {
    |            ^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: method `wait_for_condition` is never used
   --> src/browser_automation.rs:261:18
    |
 14 | impl BrowserAutomation {
    | ---------------------- method in this implementation
...
261 |     pub async fn wait_for_condition(
    |                  ^^^^^^^^^^^^^^^^^^

warning: method `auto_authorize` is never used
   --> src/sso_auth.rs:389:18
    |
 96 | impl SsoAuth {
    | ------------ method in this implementation
...
389 |     pub async fn auto_authorize(
    |                  ^^^^^^^^^^^^^^

warning: `tauri-app` (lib) generated 3 warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.97s
     Running `target/debug/tauri-app`
11:18:18 PM [vite] (client) hmr update /src/components/AccountsTable.tsx
Waiting for password input page...
Waiting for login to complete and extracting token from cookies...
Token extraction attempt 1/10...
Token extraction attempt 2/10...
Token extraction attempt 3/10...
Token extraction attempt 4/10...
Token extraction attempt 5/10...
Token extraction attempt 6/10...
Token extraction attempt 7/10...
Token extraction attempt 8/10...
Token extraction attempt 9/10...
Token extraction attempt 10/10...
