// src/scraper.rs

use crate::errors::ScrapeError;
use crate::login::auto_login;
use crate::model::{ImageData, LinkData, LoginCredentials, ScrapedData};
use chromiumoxide::browser::{Browser, BrowserConfig, HeadlessMode};
use chromiumoxide::page::Page;
use chromiumoxide::cdp::browser_protocol::emulation::{
    SetDeviceMetricsOverrideParams, SetUserAgentOverrideParams,
};
use chromiumoxide::cdp::browser_protocol::page::AddScriptToEvaluateOnNewDocumentParams;
use futures::StreamExt;
use std::time::Duration;
use tokio::task;

/// A wrapper for the browser and page to ensure proper cleanup.
pub struct Scraper {
    browser: Option<Browser>,
    page: Page,
    _handler_handle: task::JoinHandle<()>,
}

impl Scraper {
    /// Creates a new Scraper instance, launching a headless browser.
    pub async fn new(headless: bool) -> Result<Self, ScrapeError> {
        let mut builder = BrowserConfig::builder()
            .request_timeout(Duration::from_secs(30))
            .no_sandbox()
            .arg("--disable-dev-shm-usage")
            .arg("--disable-blink-features=AutomationControlled")
            .arg("--disable-extensions")
            .arg("--disable-gpu")
            .arg("--disable-software-rasterizer");

        if headless {
            builder = builder.headless_mode(HeadlessMode::True);
        } else {
            builder = builder.headless_mode(HeadlessMode::False);
        }

        let (mut browser, mut handler) = Browser::launch(builder.build().unwrap())
            .await
            .map_err(|e| ScrapeError::BrowserLaunch(e.to_string()))?;

        // Keep the handler running and don't let it die
        let _handler_handle = task::spawn(async move {
            while handler.next().await.is_some() {
                // Keep processing events
            }
        });

        // Give browser time to fully start
        tokio::time::sleep(Duration::from_millis(500)).await;

        let page = browser
            .new_page("about:blank")
            .await
            .map_err(|e| ScrapeError::PageCreation(e.to_string()))?;

        // Give page time to initialize
        tokio::time::sleep(Duration::from_millis(500)).await;

        Self::setup_evasions(&page).await?;

        Ok(Self {
            browser: Some(browser),
            page,
            _handler_handle,
        })
    }

    /// Injects scripts to make the headless browser appear more like a real user's browser.
    async fn setup_evasions(page: &Page) -> Result<(), ScrapeError> {
        page.execute(SetUserAgentOverrideParams::new(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/129.0.0.0 Safari/537.36"
        ))
        .await
        .map_err(|e| ScrapeError::EvaluationFailed(format!("Set User Agent: {}", e)))?;

        page.execute(
            SetDeviceMetricsOverrideParams::builder()
                .width(1920)
                .height(1080)
                .device_scale_factor(1.0)
                .mobile(false)
                .build()
                .unwrap(),
        )
        .await
        .map_err(|e| ScrapeError::EvaluationFailed(format!("Set Viewport: {}", e)))?;

        let evasion_script = r#"
            Object.defineProperty(navigator, 'webdriver', { get: () => undefined });
            Object.defineProperty(navigator, 'plugins', { get: () => [1, 2, 3] });
            Object.defineProperty(navigator, 'languages', { get: () => ['en-US', 'en'] });
            const originalQuery = window.navigator.permissions.query;
            window.navigator.permissions.query = (parameters) => (
                parameters.name === 'notifications' ?
                Promise.resolve({ state: Notification.permission }) :
                originalQuery(parameters)
            );
            try {
                const getParameter = WebGLRenderingContext.prototype.getParameter;
                WebGLRenderingContext.prototype.getParameter = function(parameter) {
                    if (parameter === 37445) return 'Intel Open Source Technology Center';
                    if (parameter === 37446) return 'Mesa DRI Intel(R) HD Graphics 4000 (IVB GT2)';
                    return getParameter.call(this, parameter);
                };
            } catch (e) {}
        "#.to_string();

        page.execute(AddScriptToEvaluateOnNewDocumentParams {
            source: evasion_script,
            world_name: None,
            include_command_line_api: None,
            run_immediately: None,
        })
        .await
        .map_err(|e| ScrapeError::EvaluationFailed(format!("Add Evasion Script: {}", e)))?;

        Ok(())
    }

    /// Triggers lazy-loaded content by scrolling the page incrementally.
    async fn scroll_for_lazy_content(&self) -> Result<(), ScrapeError> {
        println!("📜 Scrolling to trigger lazy-loaded content...");
        let mut last_height: i64 = -1;
        for _ in 0..5 {
            let new_height = self
                .page
                .evaluate("window.scrollTo(0, document.body.scrollHeight); document.body.scrollHeight;")
                .await
                .map_err(|e| ScrapeError::EvaluationFailed(e.to_string()))?
                .into_value::<i64>()
                .unwrap_or(0);

            if new_height == last_height {
                break;
            }
            last_height = new_height;
            tokio::time::sleep(Duration::from_millis(1500)).await;
        }
        let _ = self.page.evaluate("window.scrollTo(0, 0);").await;
        tokio::time::sleep(Duration::from_millis(500)).await;
        Ok(())
    }

    /// The main scraping logic for a given URL.
    pub async fn scrape(
        &self,
        url: &str,
        login: Option<LoginCredentials>,
    ) -> Result<ScrapedData, ScrapeError> {
        // --- 1. Handle Login ---
        let (login_attempted, login_success, platform_detected, requires_2fa) =
            if let Some(credentials) = login {
                match auto_login(&self.page, &credentials, url).await {
                    Ok((success, platform, tfa)) => {
                        if tfa.unwrap_or(false) {
                            return Err(ScrapeError::TwoFactorAuthRequired);
                        }
                        if !success {
                            println!("⚠️ Login failed, continuing without authentication...");
                        }
                        (true, Some(success), platform, tfa)
                    }
                    Err(e) => {
                        println!("⚠️ Login error: {}, continuing without authentication...", e);
                        (true, Some(false), None, None)
                    }
                }
            } else {
                (false, None, None, None)
            };

        // --- 2. Navigate to Page ---
        let current_url = self
            .page
            .evaluate("window.location.href")
            .await
            .ok()
            .and_then(|v| v.into_value::<String>().ok())
            .unwrap_or_default();

        if !current_url.starts_with(url) {
            println!("🌐 Navigating to target URL: {}", url);
            
            // Navigate with better error handling
            let nav_result = self.page.goto(url).await;
            
            match nav_result {
                Ok(_) => {
                    // Give the page time to load
                    tokio::time::sleep(Duration::from_millis(2000)).await;
                }
                Err(e) => {
                    return Err(ScrapeError::Navigation(format!("Failed to navigate: {}", e)));
                }
            }
        }

        // --- 3. Wait for content and trigger lazy loading ---
        // Wait for DOM to be ready
        let wait_result = tokio::time::timeout(
            Duration::from_secs(10),
            async {
                for _ in 0..20 {
                    if self.page.find_element("body").await.is_ok() {
                        return Ok(());
                    }
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
                Err(ScrapeError::ContentExtraction("Body element not found".to_string()))
            }
        )
        .await
        .map_err(|_| ScrapeError::ContentExtraction("Timeout waiting for body element".to_string()))?;
        
        wait_result?;
        
        self.scroll_for_lazy_content().await?;

        println!("📊 Extracting page data...");
        // --- 4. Extract Data ---
        let title = self.page.get_title().await.ok().flatten();

        let description = self
            .page
            .evaluate(r#"
                (() => {
                    const meta = document.querySelector('meta[name="description"]');
                    return meta ? meta.getAttribute('content') : null;
                })()
            "#)
            .await
            .ok()
            .and_then(|v| v.into_value::<Option<String>>().ok())
            .flatten();

        let text = self
            .page
            .evaluate(
                r#"(() => {
                const clone = document.body.cloneNode(true);
                clone.querySelectorAll('script, style, noscript, nav, header, footer, svg, button, input').forEach(el => el.remove());
                let text = clone.innerText || clone.textContent || '';
                return text.replace(/\s\s+/g, ' ').trim().substring(0, 100000);
            })()"#,
            )
            .await
            .ok()
            .and_then(|v| v.into_value::<Option<String>>().ok())
            .flatten();

        let images = self
            .page
            .evaluate(
                r#"(() => {
                return Array.from(document.querySelectorAll('img')).map(img => {
                    let src = img.src || img.getAttribute('data-src') || '';
                    if (src && !src.startsWith('http') && !src.startsWith('data:')) {
                        try {
                            src = new URL(src, window.location.href).href;
                        } catch (e) {
                            src = '';
                        }
                    }
                    return { src, alt: img.alt || '' };
                }).filter(img => img.src.startsWith('http')).slice(0, 20);
            })()"#,
            )
            .await
            .ok()
            .and_then(|v| v.into_value::<Vec<ImageData>>().ok())
            .unwrap_or_default();

        let links = self
            .page
            .evaluate(
                r#"(() => {
                return Array.from(document.querySelectorAll('a[href]')).map(link => {
                    let href = link.href;
                    return { href, text: (link.innerText || '').trim().substring(0, 200) };
                }).filter(link => link.href.startsWith('http')).slice(0, 50);
            })()"#,
            )
            .await
            .ok()
            .and_then(|v| v.into_value::<Vec<LinkData>>().ok())
            .unwrap_or_default();

        println!("✅ Data extraction complete!");
        println!("   - Title: {}", title.as_deref().unwrap_or("N/A"));
        println!("   - Images: {}", images.len());
        println!("   - Links: {}", links.len());

        Ok(ScrapedData {
            title,
            description,
            text,
            images,
            links,
            login_attempted,
            login_success,
            platform_detected,
            requires_2fa,
        })
    }
}

impl Drop for Scraper {
    fn drop(&mut self) {
        if let Some(mut browser) = self.browser.take() {
            tokio::spawn(async move {
                let _ = browser.close().await;
                let _ = browser.wait().await;
            });
        }
    }
}

/// Public wrapper function to maintain a simple API.
pub async fn do_scrape(
    url: &str,
    login: Option<LoginCredentials>,
) -> Result<ScrapedData, ScrapeError> {
    let scraper = Scraper::new(true).await?;
    scraper.scrape(url, login).await
}