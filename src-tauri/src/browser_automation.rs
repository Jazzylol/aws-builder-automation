use crate::models::{BrowserConfig, BrowserMode};
use anyhow::{Result, Context};
use headless_chrome::{Browser, LaunchOptionsBuilder};
use headless_chrome::Tab;
use std::sync::Arc;
use std::time::Duration;
use std::ffi::OsStr;
use rand::Rng;

pub struct BrowserAutomation {
    config: BrowserConfig,
}

impl BrowserAutomation {
    pub fn new(config: BrowserConfig) -> Self {
        Self { config }
    }

    pub fn generate_random_window_size() -> (u32, u32) {
        let mut rng = rand::thread_rng();
        let width = rng.gen_range(800..=1920);
        let height = rng.gen_range(600..=1080);
        (width, height)
    }

    pub fn generate_random_os_version() -> String {
        let mut rng = rand::thread_rng();
        if rng.gen_bool(0.5) {
            "Windows 10".to_string()
        } else {
            "Windows 11".to_string()
        }
    }

    fn generate_fingerprint_script() -> String {
        let mut rng = rand::thread_rng();

        // 🔥 随机生成各种设备特征
        let canvas_noise = rng.gen_range(0..10);
        let rect_noise = rng.gen_range(0.0..0.2);
        let audio_noise = rng.gen_range(0.0001..0.001);
        
        // 🔥 随机 GPU 型号
        let gpus = vec![
            "Intel(R) UHD Graphics 630",
            "NVIDIA GeForce GTX 1060",
            "AMD Radeon RX 580",
            "Intel(R) Iris(R) Xe Graphics",
            "NVIDIA GeForce RTX 3060",
            "AMD Radeon RX 6600",
            "Intel(R) HD Graphics 620",
            "NVIDIA GeForce GTX 1650",
        ];
        let random_gpu = gpus[rng.gen_range(0..gpus.len())];
        
        // 🔥 随机硬件并发数（CPU 核心数）
        let hardware_concurrency = [4, 6, 8, 12, 16][rng.gen_range(0..5)];
        
        // 🔥 随机设备内存（GB）
        let device_memory = [4, 8, 16, 32][rng.gen_range(0..4)];
        
        // 🔥 随机屏幕分辨率
        let screen_width = rng.gen_range(1366..=1920);
        let screen_height = rng.gen_range(768..=1080);
        
        // 🔥 随机设备名称
        let device_id = rand::random::<u32>() % 10000;

        format!(
            r#"
            (function() {{
                // 🔥 Canvas fingerprint randomization (增强版)
                const originalGetContext = HTMLCanvasElement.prototype.getContext;
                HTMLCanvasElement.prototype.getContext = function(type, attributes) {{
                    const context = originalGetContext.call(this, type, attributes);
                    if (type === '2d') {{
                        const originalGetImageData = context.getImageData;
                        context.getImageData = function(...args) {{
                            const imageData = originalGetImageData.apply(this, args);
                            // 更复杂的随机化
                            for (let i = 0; i < imageData.data.length; i += 4) {{
                                const noise = Math.floor(Math.random() * {}) - {};
                                imageData.data[i] = Math.max(0, Math.min(255, imageData.data[i] + noise));
                            }}
                            return imageData;
                        }};
                    }}
                    return context;
                }};

                // 🔥 WebGL fingerprint randomization (增强版 - 随机 GPU)
                const getParameter = WebGLRenderingContext.prototype.getParameter;
                WebGLRenderingContext.prototype.getParameter = function(parameter) {{
                    if (parameter === 37445) {{
                        return 'Google Inc.';
                    }}
                    if (parameter === 37446) {{
                        return '{}';  // 随机 GPU
                    }}
                    return getParameter.call(this, parameter);
                }};

                // 🔥 WebGL2 fingerprint randomization
                if (window.WebGL2RenderingContext) {{
                    const getParameter2 = WebGL2RenderingContext.prototype.getParameter;
                    WebGL2RenderingContext.prototype.getParameter = function(parameter) {{
                        if (parameter === 37445) {{
                            return 'Google Inc.';
                        }}
                        if (parameter === 37446) {{
                            return '{}';  // 随机 GPU
                        }}
                        return getParameter2.call(this, parameter);
                    }};
                }}

                // 🔥 AudioContext fingerprint randomization (增强版)
                const audioContext = window.AudioContext || window.webkitAudioContext;
                if (audioContext) {{
                    const OriginalAnalyser = audioContext.prototype.createAnalyser;
                    audioContext.prototype.createAnalyser = function() {{
                        const analyser = OriginalAnalyser.call(this);
                        const originalGetFloatFrequencyData = analyser.getFloatFrequencyData;
                        analyser.getFloatFrequencyData = function(array) {{
                            originalGetFloatFrequencyData.call(this, array);
                            for (let i = 0; i < array.length; i++) {{
                                array[i] = array[i] + (Math.random() * {} - {});
                            }}
                        }};
                        return analyser;
                    }};
                }}

                // 🔥 ClientRects randomization (增强版)
                const originalGetClientRects = Element.prototype.getClientRects;
                Element.prototype.getClientRects = function() {{
                    const rects = originalGetClientRects.call(this);
                    const noise = {};
                    for (let i = 0; i < rects.length; i++) {{
                        rects[i].x += noise;
                        rects[i].y += noise;
                    }}
                    return rects;
                }};

                // 🔥 NEW: 硬件并发数随机化（CPU 核心数）
                Object.defineProperty(navigator, 'hardwareConcurrency', {{
                    get: function() {{ return {}; }}
                }});

                // 🔥 NEW: 设备内存随机化
                Object.defineProperty(navigator, 'deviceMemory', {{
                    get: function() {{ return {}; }}
                }});

                // 🔥 NEW: 屏幕分辨率随机化
                Object.defineProperty(screen, 'width', {{
                    get: function() {{ return {}; }}
                }});
                Object.defineProperty(screen, 'height', {{
                    get: function() {{ return {}; }}
                }});
                Object.defineProperty(screen, 'availWidth', {{
                    get: function() {{ return {}; }}
                }});
                Object.defineProperty(screen, 'availHeight', {{
                    get: function() {{ return {} - 40; }}  // 减去任务栏高度
                }});

                // 🔥 Media devices randomization (增强版)
                if (navigator.mediaDevices) {{
                    const enumerateDevices = navigator.mediaDevices.enumerateDevices;
                    navigator.mediaDevices.enumerateDevices = function() {{
                        return enumerateDevices.call(this).then(devices => {{
                            return devices.map((device, index) => ({{
                                ...device,
                                deviceId: 'device_{}_{}_' + Math.random().toString(36).substr(2, 9),
                                groupId: 'group_{}_{}_' + Math.random().toString(36).substr(2, 9)
                            }}));
                        }});
                    }};
                }}

                // Disable Do Not Track
                Object.defineProperty(navigator, 'doNotTrack', {{
                    get: function() {{ return null; }}
                }});

                // Speech voices randomization
                if (window.speechSynthesis) {{
                    const voices = [
                        {{ name: 'Microsoft Huihui - Chinese (Simplified, PRC)', lang: 'zh-CN' }},
                        {{ name: 'Microsoft Kangkang - Chinese (Simplified, PRC)', lang: 'zh-CN' }},
                        {{ name: 'Microsoft Yaoyao - Chinese (Simplified, PRC)', lang: 'zh-CN' }},
                    ];

                    const originalGetVoices = speechSynthesis.getVoices;
                    speechSynthesis.getVoices = function() {{
                        return voices;
                    }};
                }}

                // 🔥 WebRTC IP protection (增强版 - 完全禁用)
                delete window.RTCPeerConnection;
                delete window.RTCDataChannel;
                delete window.RTCSessionDescription;
                delete window.webkitRTCPeerConnection;
                delete window.mozRTCPeerConnection;

                // Port scan protection
                const originalFetch = window.fetch;
                window.fetch = function(url, ...args) {{
                    const urlObj = new URL(url, window.location.href);
                    if (urlObj.hostname === 'localhost' || urlObj.hostname === '127.0.0.1') {{
                        return Promise.reject(new Error('Port scan detected and blocked'));
                    }}
                    return originalFetch.call(this, url, ...args);
                }};

                // 🔥 NEW: 隐藏 webdriver 标识
                Object.defineProperty(navigator, 'webdriver', {{
                    get: function() {{ return undefined; }}
                }});

                // 🔥 NEW: 隐藏自动化特征
                delete window.cdc_adoQpoasnfa76pfcZLmcfl_Array;
                delete window.cdc_adoQpoasnfa76pfcZLmcfl_Promise;
                delete window.cdc_adoQpoasnfa76pfcZLmcfl_Symbol;
            }})();
            "#,
            canvas_noise * 2, canvas_noise,  // Canvas 噪音
            random_gpu,  // WebGL GPU
            random_gpu,  // WebGL2 GPU
            audio_noise * 2.0, audio_noise,  // Audio 噪音
            rect_noise,  // ClientRects 噪音
            hardware_concurrency,  // CPU 核心数
            device_memory,  // 设备内存
            screen_width,  // 屏幕宽度
            screen_height,  // 屏幕高度
            screen_width,  // 可用宽度
            screen_height,  // 可用高度
            device_id, rng.gen_range(0..1000),  // 设备 ID 1
            device_id, rng.gen_range(0..1000),  // 设备 ID 2
        )
    }

    pub fn launch_browser(&self) -> Result<Browser> {
        let (width, height) = if self.config.window_width > 0 && self.config.window_height > 0 {
            (self.config.window_width, self.config.window_height)
        } else {
            Self::generate_random_window_size()
        };

        let headless = self.config.mode == BrowserMode::Background;

        let mut launch_options = LaunchOptionsBuilder::default()
            .headless(headless)
            .window_size(Some((width, height)))
            .sandbox(false)
            .build()
            .context("Failed to build launch options")?;

        // 🔥 为每个浏览器实例创建独立的用户数据目录
        let user_data_dir = format!(
            "/tmp/chrome_profile_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );
        
        eprintln!("[浏览器] 使用独立配置文件: {}", user_data_dir);

        // Add additional Chrome arguments for fingerprint protection
        let window_size_arg = format!("--window-size={},{}", width, height);
        let user_data_arg = format!("--user-data-dir={}", user_data_dir);
        
        launch_options.args.push(OsStr::new(&window_size_arg));
        launch_options.args.push(OsStr::new(&user_data_arg));  // 🔥 独立配置文件
        launch_options.args.push(OsStr::new("--disable-blink-features=AutomationControlled"));
        launch_options.args.push(OsStr::new("--disable-features=IsolateOrigins,site-per-process"));
        launch_options.args.push(OsStr::new("--lang=zh-CN"));
        launch_options.args.push(OsStr::new("--disable-web-security"));
        launch_options.args.push(OsStr::new("--ignore-certificate-errors"));
        launch_options.args.push(OsStr::new("--disable-dev-shm-usage"));
        launch_options.args.push(OsStr::new("--no-first-run"));
        launch_options.args.push(OsStr::new("--no-default-browser-check"));
        
        // 🔥 额外的反检测参数
        launch_options.args.push(OsStr::new("--disable-blink-features=AutomationControlled"));
        launch_options.args.push(OsStr::new("--exclude-switches=enable-automation"));
        launch_options.args.push(OsStr::new("--disable-infobars"));

        let browser = Browser::new(launch_options)
            .context("Failed to launch browser")?;

        Ok(browser)
    }

    pub fn apply_fingerprint_protection(&self, tab: &Arc<Tab>) -> Result<()> {
        let script = Self::generate_fingerprint_script();
        tab.evaluate(&script, false)
            .context("Failed to apply fingerprint protection")?;
        Ok(())
    }

    pub async fn wait_for_element(
        &self,
        tab: &Arc<Tab>,
        xpath: &str,
        timeout_seconds: u64,
    ) -> Result<bool> {
        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(timeout_seconds);

        loop {
            if start.elapsed() > timeout {
                return Ok(false);
            }

            let script = format!(
                r#"
                (function() {{
                    const result = document.evaluate("{}", document, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null);
                    return result.singleNodeValue !== null;
                }})()
                "#,
                xpath
            );

            match tab.evaluate(&script, true) {
                Ok(result) => {
                    if let Some(value) = result.value {
                        if value.as_bool().unwrap_or(false) {
                            return Ok(true);
                        }
                    }
                }
                Err(_) => {}
            }

            std::thread::sleep(Duration::from_millis(500));
        }
    }

    pub async fn wait_for_condition(
        &self,
        tab: &Arc<Tab>,
        js_condition: &str,
        timeout_seconds: u64,
    ) -> Result<bool> {
        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(timeout_seconds);

        loop {
            if start.elapsed() > timeout {
                return Ok(false);
            }

            match tab.evaluate(js_condition, true) {
                Ok(result) => {
                    if let Some(value) = result.value {
                        if value.as_bool().unwrap_or(false) {
                            return Ok(true);
                        }
                    }
                }
                Err(_) => {}
            }

            std::thread::sleep(Duration::from_millis(500));
        }
    }

    pub fn click_element(&self, tab: &Arc<Tab>, xpath: &str) -> Result<()> {
        let script = format!(
            r#"
            (function() {{
                const result = document.evaluate("{}", document, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null);
                const element = result.singleNodeValue;
                if (element) {{
                    element.click();
                    return true;
                }}
                return false;
            }})()
            "#,
            xpath
        );

        tab.evaluate(&script, true)
            .context("Failed to click element")?;

        Ok(())
    }

    pub fn input_text(&self, tab: &Arc<Tab>, xpath: &str, text: &str) -> Result<()> {
        // Properly escape JavaScript string to prevent encoding issues
        let escaped_text = Self::escape_js_string(text);

        let script = format!(
            r#"
            (function() {{
                const result = document.evaluate("{}", document, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null);
                const element = result.singleNodeValue;
                if (element) {{
                    // Focus the element first
                    element.focus();

                    // Clear existing value
                    element.value = "";

                    // Set the value using multiple methods to ensure React detects it
                    const nativeInputValueSetter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, "value").set;
                    nativeInputValueSetter.call(element, "{}");

                    // Dispatch events in the correct order to simulate real user input
                    element.dispatchEvent(new Event('input', {{ bubbles: true, cancelable: true }}));
                    element.dispatchEvent(new Event('change', {{ bubbles: true, cancelable: true }}));
                    element.dispatchEvent(new Event('blur', {{ bubbles: true }}));

                    return true;
                }}
                return false;
            }})()
            "#,
            xpath,
            escaped_text
        );

        tab.evaluate(&script, true)
            .context("Failed to input text")?;

        Ok(())
    }

    /// Escape a string for safe use in JavaScript code
    fn escape_js_string(s: &str) -> String {
        s.chars()
            .map(|c| match c {
                '\\' => "\\\\".to_string(),
                '"' => "\\\"".to_string(),
                '\'' => "\\'".to_string(),
                '\n' => "\\n".to_string(),
                '\r' => "\\r".to_string(),
                '\t' => "\\t".to_string(),
                '\x08' => "\\b".to_string(),
                '\x0C' => "\\f".to_string(),
                c if c.is_control() => format!("\\u{:04x}", c as u32),
                c => c.to_string(),
            })
            .collect()
    }

    #[allow(dead_code)]
    pub fn wait_for_navigation(&self, tab: &Arc<Tab>, timeout_seconds: u64) -> Result<()> {
        std::thread::sleep(Duration::from_secs(timeout_seconds));
        tab.wait_until_navigated()
            .context("Navigation timeout")?;
        Ok(())
    }

    pub fn clear_browser_data(&self) -> Result<()> {
        // This will be handled by launching a new browser instance with incognito mode
        // The browser automatically clears data when closed
        Ok(())
    }
}
