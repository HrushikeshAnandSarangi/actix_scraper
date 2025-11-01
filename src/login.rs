use crate::model::{LoginCredentials, CookieData};
use crate::config::{get_platform_config, PlatformConfig};
use chromiumoxide::{Page, cdp::browser_protocol::network::SetCookieParams};
use chromiumoxide::cdp::browser_protocol::page::NavigateParams;
use tokio::time::{sleep, Duration};
use std::error::Error;
use tracing::{info, warn, error, debug, instrument, info_span};

// --- Helper function to check page state ---
async fn log_page_state(page: &Page, context: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
    let url = page.url().await
        .ok()
        .flatten()
        .unwrap_or_else(|| "unknown".to_string());
    let title = page.evaluate("document.title").await
        .ok()
        .and_then(|v| v.into_value::<String>().ok())
        .unwrap_or_else(|| "unknown".to_string());
    
    debug!("[{}] URL: {}", context, url);
    debug!("[{}] Title: {}", context, title);
    
    // Log visible input fields for debugging
    let inputs: String = page.evaluate(
        r#"
        (() => {
            const inputs = document.querySelectorAll('input');
            return Array.from(inputs)
                .filter(i => i.offsetParent !== null)
                .slice(0, 10)
                .map(i => `${i.type}[name="${i.name}",id="${i.id}",placeholder="${i.placeholder}"]`)
                .join(' | ');
        })()
        "#
    ).await.ok().and_then(|v| v.into_value().ok()).unwrap_or_else(|| "none".to_string());
    
    if !inputs.is_empty() && inputs != "none" {
        debug!("[{}] Visible inputs: {}", context, inputs);
    }
    Ok(())
}

/// Enhanced stealth navigation with realistic behavior
async fn stealth_navigate(page: &Page, url: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
    info!("🌐 Navigating to: {}", url);
    
    // Set realistic headers and properties before navigation
    let _ = page.evaluate(
        r#"
        (() => {
            // Override navigator properties to appear more human
            Object.defineProperty(navigator, 'webdriver', {
                get: () => undefined
            });
            
            // Add realistic plugins
            Object.defineProperty(navigator, 'plugins', {
                get: () => [1, 2, 3, 4, 5]
            });
            
            // Override languages
            Object.defineProperty(navigator, 'languages', {
                get: () => ['en-US', 'en']
            });
            
            // Set realistic screen properties
            Object.defineProperty(screen, 'availWidth', {
                get: () => 1920
            });
            Object.defineProperty(screen, 'availHeight', {
                get: () => 1080
            });
        })()
        "#
    ).await;
    
    // Navigate with longer timeout
    page.goto(url).await?;
    
    // Random delay to appear more human
    let delay = 1500 + (rand::random::<u64>() % 1000);
    sleep(Duration::from_millis(delay)).await;
    
    Ok(())
}

/// Wait for any element with better error handling
async fn wait_for_any_element(
    page: &Page,
    selectors: &[String],
    timeout_ms: u64,
) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
    let check_interval_ms = 200;
    let mut elapsed = 0;

    info!("🔍 Searching for elements (timeout: {}s)", timeout_ms / 1000);
    
    while elapsed < timeout_ms {
        for selector in selectors {
            let is_visible: bool = page.evaluate(format!(
                r#"
                (() => {{
                    try {{
                        const el = document.querySelector('{}');
                        if (!el) return false;
                        
                        const style = window.getComputedStyle(el);
                        const rect = el.getBoundingClientRect();
                        
                        return el.offsetParent !== null &&
                               style.visibility !== 'hidden' && 
                               style.display !== 'none' && 
                               style.opacity !== '0' &&
                               rect.width > 0 && 
                               rect.height > 0 &&
                               rect.top >= 0;
                    }} catch(e) {{
                        return false;
                    }}
                }})()
                "#,
                selector.replace("'", "\\'").replace("\\", "\\\\")
            )).await.ok().and_then(|v| v.into_value().ok()).unwrap_or(false);

            if is_visible {
                info!("✅ Found visible element: {}", selector);
                return Ok(Some(selector.clone()));
            }
        }
        
        sleep(Duration::from_millis(check_interval_ms)).await;
        elapsed += check_interval_ms;
    }

    error!("❌ No element found after {}ms", timeout_ms);
    Ok(None)
}

/// Extremely realistic typing simulation
async fn type_into_field(
    page: &Page,
    selector: &str,
    text: &str,
) -> Result<bool, Box<dyn Error + Send + Sync>> {
    info!("⌨️  Typing into: {}", selector);
    
    let result: bool = page.evaluate(format!(
        r#"
        (async () => {{
            try {{
                const field = document.querySelector('{}');
                if (!field || field.offsetParent === null) {{
                    console.error('Field not found or not visible');
                    return false;
                }}
                
                // Scroll into view naturally
                field.scrollIntoView({{ behavior: 'smooth', block: 'center' }});
                await new Promise(r => setTimeout(r, 300 + Math.random() * 200));
                
                // Focus with mouse-like behavior
                field.focus();
                await new Promise(r => setTimeout(r, 50 + Math.random() * 50));
                field.click();
                await new Promise(r => setTimeout(r, 100 + Math.random() * 100));
                
                // Clear existing value
                field.value = '';
                
                // Dispatch focus events
                field.dispatchEvent(new FocusEvent('focusin', {{ bubbles: true }}));
                field.dispatchEvent(new FocusEvent('focus', {{ bubbles: true }}));
                
                const text = '{}';
                
                // Type character by character with realistic delays
                for (let i = 0; i < text.length; i++) {{
                    const char = text.charAt(i);
                    
                    // Simulate realistic typing patterns
                    const baseDelay = 80;
                    const variance = 60;
                    const delay = baseDelay + Math.floor(Math.random() * variance);
                    
                    // Occasionally add longer pauses (thinking)
                    const thinkingPause = Math.random() < 0.1 ? 200 + Math.random() * 300 : 0;
                    
                    await new Promise(r => setTimeout(r, delay + thinkingPause));
                    
                    // Update value
                    const prevValue = field.value;
                    field.value = prevValue + char;
                    
                    // Create realistic keyboard events
                    const keyCode = char.charCodeAt(0);
                    
                    field.dispatchEvent(new KeyboardEvent('keydown', {{ 
                        key: char,
                        code: 'Key' + char.toUpperCase(),
                        keyCode: keyCode,
                        which: keyCode,
                        bubbles: true,
                        cancelable: true,
                        composed: true
                    }}));
                    
                    field.dispatchEvent(new KeyboardEvent('keypress', {{ 
                        key: char,
                        keyCode: keyCode,
                        which: keyCode,
                        bubbles: true,
                        cancelable: true
                    }}));
                    
                    // Most important for modern frameworks
                    field.dispatchEvent(new InputEvent('input', {{ 
                        data: char,
                        inputType: 'insertText',
                        bubbles: true,
                        cancelable: false,
                        composed: true
                    }}));
                    
                    field.dispatchEvent(new KeyboardEvent('keyup', {{ 
                        key: char,
                        code: 'Key' + char.toUpperCase(),
                        keyCode: keyCode,
                        which: keyCode,
                        bubbles: true,
                        cancelable: true
                    }}));
                }}
                
                // Wait before blur
                await new Promise(r => setTimeout(r, 200 + Math.random() * 200));
                
                // Trigger change and blur events
                field.dispatchEvent(new Event('change', {{ bubbles: true }}));
                field.dispatchEvent(new FocusEvent('blur', {{ bubbles: true }}));
                field.dispatchEvent(new FocusEvent('focusout', {{ bubbles: true }}));
                
                // Final stabilization
                await new Promise(r => setTimeout(r, 300));
                
                return true;
            }} catch(e) {{
                console.error('Typing error:', e);
                return false;
            }}
        }})()
        "#,
        selector.replace("'", "\\'").replace("\\", "\\\\"),
        text.replace("'", "\\'").replace("\\", "\\\\").replace("\n", "\\n")
    )).await?.into_value()?;

    if result {
        info!("✅ Successfully typed into field");
    } else {
        error!("❌ Failed to type into field");
    }
    
    Ok(result)
}

#[instrument(skip(page, cookies), fields(count = cookies.len()))]
pub async fn set_cookies(
    page: &Page,
    cookies: &[CookieData],
) -> Result<(), Box<dyn Error + Send + Sync>> {
    info!("🍪 Setting {} cookies...", cookies.len());
    for cookie in cookies {
        let domain_clean = cookie.domain.trim_start_matches('.');
        let cookie_url = format!("https://{}{}", domain_clean, cookie.path.as_deref().unwrap_or("/"));
        
        let cookie_params = SetCookieParams {
            name: cookie.name.clone(),
            value: cookie.value.clone(),
            url: Some(cookie_url),
            domain: Some(cookie.domain.clone()),
            path: cookie.path.clone(),
            secure: Some(true),
            http_only: None,
            same_site: Some(chromiumoxide::cdp::browser_protocol::network::CookieSameSite::Lax),
            expires: None,
            priority: None,
            same_party: None,
            source_scheme: None,
            source_port: None,
            partition_key: None,
        };
        
        if let Err(e) = page.execute(cookie_params).await {
            warn!("Failed to set cookie {}: {:?}", cookie.name, e);
        }
    }
    
    sleep(Duration::from_millis(500)).await;
    Ok(())
}

async fn retry_action<F, Fut, T>(
    max_retries: u32,
    mut action: F,
) -> Result<T, Box<dyn Error + Send + Sync>>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, Box<dyn Error + Send + Sync>>> + Send,
{
    let mut last_error = None;
    for attempt in 1..=max_retries {
        match action().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                warn!("Attempt {}/{} failed: {}", attempt, max_retries, e);
                last_error = Some(e);
                if attempt < max_retries {
                    let delay = Duration::from_millis(1000 * attempt as u64);
                    sleep(delay).await;
                }
            }
        }
    }
    Err(last_error.unwrap_or_else(|| "Unknown retry failure".into()))
}

/// Platform-specific login URL resolver
fn get_login_url(platform: &str, target_url: &str) -> String {
    match platform {
        "google" => "https://accounts.google.com/signin".to_string(),
        "linkedin" => "https://www.linkedin.com/login".to_string(),
        "reddit" => "https://www.reddit.com/login/".to_string(),
        "github" => "https://github.com/login".to_string(),
        "facebook" => "https://www.facebook.com/login/".to_string(),
        "twitter" | "x" => "https://twitter.com/i/flow/login".to_string(),
        "instagram" => "https://www.instagram.com/accounts/login/".to_string(),
        "microsoft" => "https://login.live.com/".to_string(),
        _ => target_url.to_string(),
    }
}

/// Handle cookie consent banners more aggressively
async fn dismiss_cookie_banners(page: &Page) -> Result<(), Box<dyn Error + Send + Sync>> {
    info!("🍪 Dismissing cookie banners");
    
    for attempt in 0..3 {
        let clicked = page.evaluate(
            r#"
            (() => {
                const acceptTexts = [
                    'accept all', 'accept', 'agree', 'allow all', 'allow cookies',
                    'i agree', 'got it', 'ok', 'continue', 'agree and continue',
                    'accept cookies', 'agree & continue', 'agree and proceed'
                ];
                
                const rejectTexts = ['reject all', 'reject', 'decline', 'deny'];
                
                // Try accept buttons first
                const allButtons = document.querySelectorAll(
                    'button, a[role="button"], div[role="button"], [class*="cookie"] button, [id*="cookie"] button'
                );
                
                for (const btn of allButtons) {
                    const text = (btn.textContent || btn.innerText || btn.value || '').toLowerCase().trim();
                    const ariaLabel = (btn.getAttribute('aria-label') || '').toLowerCase();
                    const fullText = text + ' ' + ariaLabel;
                    
                    // Prefer accept buttons
                    if (acceptTexts.some(accept => fullText.includes(accept)) && 
                        fullText.length < 80 && btn.offsetParent !== null) {
                        console.log('Clicking accept:', text);
                        btn.scrollIntoView({ block: 'center' });
                        btn.click();
                        return true;
                    }
                }
                
                // If no accept found, try close buttons
                const closeButtons = document.querySelectorAll('[aria-label*="close" i], button[class*="close" i]');
                for (const btn of closeButtons) {
                    if (btn.offsetParent !== null) {
                        console.log('Clicking close button');
                        btn.click();
                        return true;
                    }
                }
                
                return false;
            })()
            "#
        ).await.ok().and_then(|v| v.into_value::<bool>().ok()).unwrap_or(false);
        
        if clicked {
            info!("✅ Dismissed banner (attempt {})", attempt + 1);
            sleep(Duration::from_millis(1000)).await;
        } else if attempt == 0 {
            sleep(Duration::from_millis(500)).await;
        } else {
            break;
        }
    }
    
    Ok(())
}

/// Enhanced click function with retry
async fn click_element(page: &Page, selector: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
    let clicked = page.evaluate(format!(
        r#"
        (() => {{
            try {{
                const el = document.querySelector('{}');
                if (!el || el.offsetParent === null) return false;
                
                el.scrollIntoView({{ behavior: 'smooth', block: 'center' }});
                
                // Wait a bit for scroll
                setTimeout(() => {{
                    el.focus();
                    el.click();
                }}, 200);
                
                return true;
            }} catch(e) {{
                return false;
            }}
        }})()
        "#,
        selector.replace("'", "\\'")
    )).await?.into_value()?;
    
    if clicked {
        sleep(Duration::from_millis(500)).await;
    }
    
    Ok(clicked)
}

#[instrument(skip(page, credentials), fields(platform, target = target_url))]
pub async fn auto_login(
    page: &Page,
    credentials: &LoginCredentials,
    target_url: &str,
) -> Result<(bool, Option<String>, Option<bool>), Box<dyn Error + Send + Sync>> {
    info!("🚀 Starting auto-login attempt");
    
    // 1. Platform Detection
    let platform = credentials.platform.as_deref().unwrap_or_else(|| {
        if target_url.contains("google.com") || target_url.contains("gmail.com") || 
           target_url.contains("accounts.google.com") { "google" }
        else if target_url.contains("linkedin.com") { "linkedin" }
        else if target_url.contains("reddit.com") { "reddit" }
        else if target_url.contains("facebook.com") { "facebook" }
        else if target_url.contains("twitter.com") || target_url.contains("x.com") { "twitter" }
        else if target_url.contains("github.com") { "github" }
        else if target_url.contains("instagram.com") { "instagram" }
        else if target_url.contains("microsoft.com") || target_url.contains("login.live.com") { "microsoft" }
        else { "generic" }
    });
    info!("🎯 Platform: {}", platform);
    
    let config = get_platform_config(platform);
    
    // 2. Set cookies if provided
    if let Some(cookies) = &credentials.cookies {
        set_cookies(page, cookies).await?;
    }
    
    // 3. Navigate to login page
    let login_url = credentials.login_url.as_deref()
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            if !config.login_url.is_empty() {
                config.login_url.clone().to_string()
            } else {
                get_login_url(platform, target_url)
            }
        });
    
    stealth_navigate(page, &login_url).await?;
    sleep(Duration::from_millis(2000)).await;
    log_page_state(page, "initial_load").await?;
    
    // 4. Dismiss cookie banners
    dismiss_cookie_banners(page).await?;
    sleep(Duration::from_millis(1000)).await;
    
    // 5. Get selectors with platform-specific fallbacks
    let email_selectors: Vec<String> = if let Some(sel) = &credentials.email_selector {
        vec![sel.clone()]
    } else if !config.email_selectors.is_empty() {
        config.email_selectors.iter().map(|s| s.to_string()).collect()
    } else {
        // Platform-specific selectors
        match platform {
            "google" => vec![
                "#identifierId".to_string(),
                "input[type=\"email\"]".to_string(),
                "input[name=\"identifier\"]".to_string(),
            ],
            "linkedin" => vec![
                "#username".to_string(),
                "input[name=\"session_key\"]".to_string(),
                "input[autocomplete=\"username\"]".to_string(),
            ],
            "reddit" => vec![
                "#loginUsername".to_string(),
                "input[name=\"username\"]".to_string(),
                "input[placeholder*=\"Username\" i]".to_string(),
            ],
            "github" => vec![
                "#login_field".to_string(),
                "input[name=\"login\"]".to_string(),
            ],
            _ => vec![
                "input[type=\"email\"]".to_string(),
                "input[name=\"email\"]".to_string(),
                "input[autocomplete=\"email\"]".to_string(),
            ],
        }
    };
    
    let password_selectors: Vec<String> = if let Some(sel) = &credentials.password_selector {
        vec![sel.clone()]
    } else if !config.password_selectors.is_empty() {
        config.password_selectors.iter().map(|s| s.to_string()).collect()
    } else {
        match platform {
            "google" => vec![
                "input[type=\"password\"]".to_string(),
                "input[name=\"Passwd\"]".to_string(),
            ],
            "linkedin" => vec![
                "#password".to_string(),
                "input[name=\"session_password\"]".to_string(),
            ],
            "reddit" => vec![
                "#loginPassword".to_string(),
                "input[name=\"password\"]".to_string(),
                "input[placeholder*=\"Password\" i]".to_string(),
            ],
            _ => vec![
                "input[type=\"password\"]".to_string(),
                "input[name=\"password\"]".to_string(),
            ],
        }
    };
    
    // 6. Fill email/username
    info!("📧 Step 1: Entering email/username");
    log_page_state(page, "before_email").await?;
    
    let email_found = wait_for_any_element(page, &email_selectors, 20000).await?;
    if let Some(email_sel) = email_found {
        if !type_into_field(page, &email_sel, &credentials.email).await? {
            error!("❌ Failed to type email");
            return Err("Failed to enter email".into());
        }
        sleep(Duration::from_millis(800)).await;
    } else {
        error!("❌ Email field not found");
        log_page_state(page, "email_not_found").await?;
        return Err("Email field not found".into());
    }
    
    // 7. Handle multi-step login (check if password field is visible)
    let is_multi_step = ["google", "linkedin", "twitter", "x", "facebook", "microsoft"].contains(&platform);
    
    if is_multi_step {
        info!("🔄 Checking for multi-step login");
        let password_visible = wait_for_any_element(page, &password_selectors, 2000).await?.is_some();
        
        if !password_visible {
            info!("🔘 Clicking 'Next' button");
            
            // Try multiple strategies to proceed
            let proceeded = page.evaluate(
                r#"
                (() => {
                    // Strategy 1: Known button IDs/classes
                    const knownButtons = [
                        '#identifierNext', '#passwordNext', 'button[type="submit"]',
                        'button[data-testid="submit"]', 'button[id*="next" i]'
                    ];
                    
                    for (const sel of knownButtons) {
                        const btn = document.querySelector(sel);
                        if (btn && btn.offsetParent !== null) {
                            console.log('Clicking known button:', sel);
                            btn.click();
                            return true;
                        }
                    }
                    
                    // Strategy 2: Text-based search
                    const buttons = document.querySelectorAll('button, div[role="button"], a[role="button"]');
                    const nextTexts = ['next', 'continue', 'weiter', 'suivant', 'continuar'];
                    
                    for (const btn of buttons) {
                        const text = (btn.textContent || btn.innerText || '').toLowerCase().trim();
                        if (nextTexts.some(t => text === t || (text.includes(t) && text.length < 20)) && 
                            btn.offsetParent !== null) {
                            console.log('Clicking text-matched button:', text);
                            btn.scrollIntoView({ block: 'center' });
                            btn.click();
                            return true;
                        }
                    }
                    
                    // Strategy 3: Press Enter on email field
                    const emailFields = document.querySelectorAll('input[type="email"], input[name*="user" i], input[name*="email" i]');
                    for (const field of emailFields) {
                        if (field.offsetParent !== null && field.value.length > 0) {
                            field.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', keyCode: 13, bubbles: true }));
                            field.dispatchEvent(new KeyboardEvent('keyup', { key: 'Enter', keyCode: 13, bubbles: true }));
                            return true;
                        }
                    }
                    
                    return false;
                })()
                "#
            ).await.ok().and_then(|v| v.into_value::<bool>().ok()).unwrap_or(false);
            
            if proceeded {
                info!("✅ Proceeded to next step");
                sleep(Duration::from_millis(3000)).await;
            } else {
                warn!("⚠️  Could not find next button, continuing anyway");
                sleep(Duration::from_millis(2000)).await;
            }
        }
    }
    
    // 8. Fill password
    info!("🔒 Step 2: Entering password");
    log_page_state(page, "before_password").await?;
    
    let password_found = wait_for_any_element(page, &password_selectors, 20000).await?;
    if let Some(pass_sel) = password_found {
        if !type_into_field(page, &pass_sel, &credentials.password).await? {
            error!("❌ Failed to type password");
            return Err("Failed to enter password".into());
        }
        sleep(Duration::from_millis(800)).await;
    } else {
        error!("❌ Password field not found");
        log_page_state(page, "password_not_found").await?;
        return Err("Password field not found".into());
    }
    
    // 9. Submit the form
    info!("📤 Step 3: Submitting form");
    
    let submit_selectors: Vec<String> = if let Some(sel) = &credentials.submit_selector {
        vec![sel.clone()]
    } else if !config.submit_selectors.is_empty() {
        config.submit_selectors.iter().map(|s| s.to_string()).collect()
    } else {
        vec![
            "button[type=\"submit\"]".to_string(),
            "input[type=\"submit\"]".to_string(),
            "button[id*=\"submit\" i]".to_string(),
        ]
    };
    
    let mut submitted = false;
    if let Some(submit_sel) = wait_for_any_element(page, &submit_selectors, 5000).await? {
        submitted = click_element(page, &submit_sel).await?;
    }
    
    if !submitted {
        info!("⚠️  Submit button not found, trying Enter key");
        let _ = page.evaluate(
            r#"
            (() => {
                const passField = document.querySelector('input[type="password"]');
                if (passField) {
                    passField.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', keyCode: 13, bubbles: true }));
                    passField.dispatchEvent(new KeyboardEvent('keyup', { key: 'Enter', keyCode: 13, bubbles: true }));
                    return true;
                }
                
                const form = document.querySelector('form');
                if (form) {
                    form.submit();
                    return true;
                }
                
                return false;
            })()
            "#
        ).await;
        submitted = true;
    }
    
    if !submitted {
        return Err("Could not submit form".into());
    }
    
    // 10. Wait for navigation
    info!("⏳ Waiting for post-login navigation");
    let wait_time = credentials.wait_after_login_secs.unwrap_or(config.wait_after_login).max(8);
    
    let nav_fut = page.wait_for_navigation();
    let timeout = sleep(Duration::from_secs(wait_time as u64));
    
    tokio::select! {
        res = nav_fut => {
            if let Err(e) = res {
                warn!("Navigation event: {}", e);
            } else {
                info!("✅ Navigation completed");
            }
        },
        _ = timeout => {
            info!("Navigation timeout after {}s", wait_time);
        },
    }
    
    sleep(Duration::from_millis(3000)).await;
    log_page_state(page, "after_login").await?;
    
    // 11. Check for post-login prompts
    for _ in 0..2 {
        let dismissed = page.evaluate(
            r#"
            (() => {
                const skipTexts = ['skip', 'not now', 'maybe later', 'no thanks', 'later', 'dismiss', 'remind me later'];
                const buttons = document.querySelectorAll('button, div[role="button"], a[role="button"]');
                
                for (const btn of buttons) {
                    const text = (btn.textContent || '').toLowerCase().trim();
                    if (skipTexts.some(s => text === s || (text.includes(s) && text.length < 30)) && 
                        btn.offsetParent !== null) {
                        console.log('Dismissing prompt:', text);
                        btn.click();
                        return true;
                    }
                }
                return false;
            })()
            "#
        ).await.ok().and_then(|v| v.into_value::<bool>().ok()).unwrap_or(false);
        
        if dismissed {
            sleep(Duration::from_millis(1500)).await;
        } else {
            break;
        }
    }
    
    // 12. Final checks
    let requires_captcha = page.evaluate(
        r#"
        (() => {
            const text = document.body.innerText.toLowerCase();
            return text.includes('captcha') || text.includes('recaptcha') || 
                   text.includes('verify you\'re human') ||
                   document.querySelectorAll('.g-recaptcha, iframe[src*="recaptcha"], [data-sitekey]').length > 0;
        })()
        "#
    ).await.ok().and_then(|v| v.into_value::<bool>().ok()).unwrap_or(false);
    
    if requires_captcha {
        warn!("🤖 Captcha detected");
        return Ok((false, Some(platform.to_string()), Some(true)));
    }
    
    let requires_2fa = page.evaluate(
        r#"
        (() => {
            const text = document.body.innerText.toLowerCase();
            const twoFaKeywords = ['two-factor', '2fa', 'verification code', 'enter code', 
                                   'security code', 'authenticator', 'phone number ending'];
            if (twoFaKeywords.some(k => text.includes(k))) return true;
            
            const codeInputs = document.querySelectorAll(
                'input[autocomplete="one-time-code"], input[type="tel"][maxlength="6"], ' +
                'input[inputmode="numeric"][maxlength="6"]'
            );
            return codeInputs.length > 0;
        })()
        "#
    ).await.ok().and_then(|v| v.into_value::<bool>().ok()).unwrap_or(false);
    
    if requires_2fa {
        warn!("🔐 2FA detected");
        return Ok((false, Some(platform.to_string()), Some(true)));
    }
    
    let has_error = page.evaluate(
        r#"
        (() => {
            const text = document.body.innerText.toLowerCase();
            const errorPatterns = [
                'wrong password', 'incorrect password', 'invalid', 
                'couldn\'t sign you in', 'authentication failed',
                'couldn\'t find your', 'check your email', 'try again'
            ];
            
            if (errorPatterns.some(err => text.includes(err))) return true;
            
            const errorElements = document.querySelectorAll(
                '[role="alert"], .error, .alert-danger, [class*="error" i], [id*="error" i]'
            );
            
            return Array.from(errorElements).some(el => {
                if (el.offsetParent === null) return false;
                const elText = (el.textContent || '').toLowerCase();
                return errorPatterns.some(err => elText.includes(err)) || elText.length > 10;
            });
        })()
        "#
    ).await.ok().and_then(|v| v.into_value::<bool>().ok()).unwrap_or(false);
    
    if has_error {
        error!("❌ Login error detected on page");
        return Ok((false, Some(platform.to_string()), Some(false)));
    }
    
    // Check for success
    let mut login_success = false;
    
    // Method 1: Check configured success indicators
    if let Some(checks) = &config.additional_checks {
        for check in checks {
            if wait_for_any_element(page, &[check.to_string()], 3000).await.is_ok_and(|opt| opt.is_some()) {
                login_success = true;
                info!("✅ Success indicator found: {}", check);
                break;
            }
        }
    }
    
    // Method 2: No login forms visible + user indicators present
    if !login_success {
        let success_indicators = page.evaluate(
            r#"
            (() => {
                // Check no login forms visible
                const loginInputs = Array.from(document.querySelectorAll(
                    'input[type="email"], input[type="password"], ' +
                    'input[name*="user" i], input[name*="pass" i], input[name*="login" i]'
                ));
                const visibleLogins = loginInputs.some(el => el.offsetParent !== null);
                
                if (visibleLogins) return false;
                
                // Check for user indicators
                const userIndicators = Array.from(document.querySelectorAll(
                    '[aria-label*="account" i], [aria-label*="profile" i], [data-testid*="user" i], ' +
                    '.user-avatar, .profile-pic, img[alt*="profile" i], [class*="avatar" i], ' +
                    '[href*="settings" i], [href*="logout" i], [href*="signout" i]'
                ));
                
                const hasUserIndicators = userIndicators.some(el => el.offsetParent !== null);
                
                // Also check URL changed from login page
                const urlChanged = !window.location.href.includes('/login') && 
                                   !window.location.href.includes('/signin') &&
                                   !window.location.href.includes('/authenticate');
                
                return hasUserIndicators || urlChanged;
            })()
            "#
        ).await.ok().and_then(|v| v.into_value::<bool>().ok()).unwrap_or(false);
        
        if success_indicators {
            login_success = true;
            info!("✅ Login successful (verified via indicators)");
        }
    }
    
    // Navigate to target URL if successful
    if login_success {
        info!("🎉 Login successful!");
        
        // Only navigate if target is different from current page
        let current_url = page.url().await.ok().flatten().unwrap_or_default();
        if !current_url.is_empty() && !target_url.contains(&current_url) && target_url != &login_url {
            info!("🌐 Navigating to target: {}", target_url);
            
            if let Err(e) = stealth_navigate(page, target_url).await {
                warn!("Failed to navigate to target: {}", e);
            } else {
                sleep(Duration::from_millis(2000)).await;
                log_page_state(page, "final_target").await?;
            }
        } else {
            info!("Already at target page");
        }
    } else {
        warn!("⚠️  Login status inconclusive - no clear success or error");
    }
    
    Ok((login_success, Some(platform.to_string()), Some(false)))
}