use crate::model::{LoginCredentials, CookieData};
use crate::config::{get_platform_config, PlatformConfig};
use chromiumoxide::{Page, cdp::browser_protocol::network::SetCookieParams};
use chromiumoxide::cdp::browser_protocol::emulation::{SetUserAgentOverrideParams, SetTimezoneOverrideParams};
use chromiumoxide::cdp::browser_protocol::page::SetWebLifecycleStateParams;
use tokio::time::{sleep, Duration};
use std::error::Error;
use tracing::{info, warn, error, debug, instrument, info_span};

async fn setup_stealth_mode(page: &Page) -> Result<(), Box<dyn Error + Send + Sync>> {
    let user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
    
    let ua_params = SetUserAgentOverrideParams {
        user_agent: user_agent.to_string(),
        accept_language: Some("en-US,en;q=0.9".to_string()),
        platform: Some("Win32".to_string()),
        user_agent_metadata: None,
    };
    page.execute(ua_params).await?;
    
    let tz_params = SetTimezoneOverrideParams {
        timezone_id: "America/New_York".to_string(),
    };
    page.execute(tz_params).await?;
    
    let stealth_script = r#"
        Object.defineProperty(navigator, 'webdriver', {
            get: () => undefined
        });
        
        const originalQuery = window.navigator.permissions.query;
        window.navigator.permissions.query = (parameters) => (
            parameters.name === 'notifications' ?
                Promise.resolve({ state: Notification.permission }) :
                originalQuery(parameters)
        );
        
        Object.defineProperty(navigator, 'plugins', {
            get: () => [
                {
                    0: {type: "application/x-google-chrome-pdf", suffixes: "pdf", description: "Portable Document Format"},
                    description: "Portable Document Format",
                    filename: "internal-pdf-viewer",
                    length: 1,
                    name: "Chrome PDF Plugin"
                },
                {
                    0: {type: "application/pdf", suffixes: "pdf", description: "Portable Document Format"},
                    description: "Portable Document Format",
                    filename: "mhjfbmdgcfjbbpaeojofohoefgiehjai",
                    length: 1,
                    name: "Chrome PDF Viewer"
                }
            ]
        });
        
        Object.defineProperty(navigator, 'languages', {
            get: () => ['en-US', 'en']
        });
        
        window.chrome = {
            runtime: {}
        };
        
        const originalToString = Function.prototype.toString;
        Function.prototype.toString = function() {
            if (this === Function.prototype.toString) {
                return 'function toString() { [native code] }';
            }
            return originalToString.call(this);
        };
        
        const originalContentWindowGetter = Object.getOwnPropertyDescriptor(HTMLIFrameElement.prototype, 'contentWindow').get;
        Object.defineProperty(HTMLIFrameElement.prototype, 'contentWindow', {
            get: function() {
                const win = originalContentWindowGetter.call(this);
                try {
                    if (win) {
                        win.navigator.webdriver = undefined;
                    }
                } catch (e) {}
                return win;
            }
        });
        
        const originalConsoleDebug = console.debug;
        console.debug = function(...args) {
            if (args.some(arg => typeof arg === 'string' && (arg.includes('webdriver') || arg.includes('automation')))) {
                return;
            }
            return originalConsoleDebug.apply(console, args);
        };
    "#;
    
    page.evaluate(stealth_script).await?;
    
    Ok(())
}

#[instrument(skip(page, cookies), fields(count = cookies.len()))]
pub async fn set_cookies(
    page: &Page,
    cookies: &[CookieData],
) -> Result<bool, Box<dyn Error + Send + Sync>> {
    info!("Setting {} cookies", cookies.len());
    
    let mut success_count = 0;
    
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
        } else {
            success_count += 1;
            debug!("Set cookie: {}", cookie.name);
        }
    }
    
    info!("Set {}/{} cookies successfully", success_count, cookies.len());
    sleep(Duration::from_millis(1000)).await;
    
    Ok(success_count > 0)
}

async fn verify_authentication(page: &Page, platform: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
    info!("Verifying authentication status");
    
    let is_authenticated = page.evaluate(
        r#"
        (() => {
            const text = document.body.innerText.toLowerCase();
            const html = document.documentElement.outerHTML.toLowerCase();
            
            const loggedInIndicators = [
                'sign out', 'signout', 'log out', 'logout',
                'my account', 'profile', 'settings',
                'dashboard', 'notifications'
            ];
            
            const loggedOutIndicators = [
                'sign in', 'signin', 'log in', 'login',
                'create account', 'register', 'join now',
                'get started', 'sign up'
            ];
            
            const hasLoggedIn = loggedInIndicators.some(ind => text.includes(ind) || html.includes(ind));
            const hasLoggedOut = loggedOutIndicators.some(ind => text.includes(ind) || html.includes(ind));
            
            const userElements = document.querySelectorAll(
                '[aria-label*="profile" i], [aria-label*="account" i], [aria-label*="user menu" i], ' +
                '[data-testid*="user" i], [data-testid*="profile" i], ' +
                '.avatar, .user-avatar, img[alt*="avatar" i], img[alt*="profile" i]'
            );
            const hasUserElements = Array.from(userElements).some(el => el.offsetParent !== null);
            
            const hasAuthCookie = document.cookie.split(';').some(cookie => {
                const name = cookie.trim().split('=')[0].toLowerCase();
                return name.includes('session') || name.includes('auth') || 
                       name.includes('token') || name.includes('user');
            });
            
            if (hasUserElements) return true;
            if (hasAuthCookie && hasLoggedIn && !hasLoggedOut) return true;
            if (hasLoggedIn && !hasLoggedOut) return true;
            
            return false;
        })()
        "#
    ).await.ok().and_then(|v| v.into_value::<bool>().ok()).unwrap_or(false);
    
    if is_authenticated {
        info!("Authentication verified - user is logged in");
    } else {
        warn!("Authentication failed - user appears logged out");
    }
    
    Ok(is_authenticated)
}

async fn log_page_state(page: &Page, context: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
    let url = page.url().await.ok().flatten().unwrap_or_else(|| "unknown".to_string());
    let title = page.evaluate("document.title").await
        .ok()
        .and_then(|v| v.into_value::<String>().ok())
        .unwrap_or_else(|| "unknown".to_string());
    
    debug!("[{}] URL: {}", context, url);
    debug!("[{}] Title: {}", context, title);
    
    Ok(())
}

async fn wait_for_any_element(
    page: &Page,
    selectors: &[String],
    timeout_ms: u64,
) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
    let check_interval_ms = 300;
    let mut elapsed = 0;

    debug!("Searching for: {:?}", selectors);

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
                               parseFloat(style.opacity) > 0 &&
                               rect.width > 0 && 
                               rect.height > 0;
                    }} catch(e) {{
                        return false;
                    }}
                }})()
                "#,
                selector.replace("'", "\\'").replace("\\", "\\\\")
            )).await.ok().and_then(|v| v.into_value().ok()).unwrap_or(false);

            if is_visible {
                info!("Found element: {}", selector);
                return Ok(Some(selector.clone()));
            }
        }
        
        sleep(Duration::from_millis(check_interval_ms)).await;
        elapsed += check_interval_ms;
    }

    Ok(None)
}

async fn type_into_field(
    page: &Page,
    selector: &str,
    text: &str,
) -> Result<bool, Box<dyn Error + Send + Sync>> {
    info!("Typing into field: {}", selector);
    
    let base_delay = 60;
    let variance = 40;
    
    let result: bool = page.evaluate(format!(
        r#"
        (async () => {{
            try {{
                const field = document.querySelector('{}');
                if (!field || field.offsetParent === null) return false;
                
                field.scrollIntoView({{ behavior: 'smooth', block: 'center' }});
                await new Promise(r => setTimeout(r, 400));
                
                field.focus();
                field.click();
                await new Promise(r => setTimeout(r, 150));
                
                field.value = '';
                field.dispatchEvent(new Event('focus', {{ bubbles: true }}));
                
                const text = '{}';
                
                for (let i = 0; i < text.length; i++) {{
                    const char = text.charAt(i);
                    const delay = {} + Math.floor(Math.random() * {});
                    
                    await new Promise(r => setTimeout(r, delay));
                    
                    field.value += char;
                    
                    field.dispatchEvent(new InputEvent('input', {{ 
                        data: char,
                        inputType: 'insertText',
                        bubbles: true
                    }}));
                    
                    field.dispatchEvent(new KeyboardEvent('keydown', {{ 
                        key: char,
                        bubbles: true
                    }}));
                    
                    field.dispatchEvent(new KeyboardEvent('keyup', {{ 
                        key: char,
                        bubbles: true
                    }}));
                }}
                
                await new Promise(r => setTimeout(r, 250));
                field.dispatchEvent(new Event('change', {{ bubbles: true }}));
                field.dispatchEvent(new Event('blur', {{ bubbles: true }}));
                
                return true;
            }} catch(e) {{
                console.error('Type error:', e);
                return false;
            }}
        }})()
        "#,
        selector.replace("'", "\\'").replace("\\", "\\\\"),
        text.replace("'", "\\'").replace("\\", "\\\\").replace("\n", "\\n"),
        base_delay,
        variance
    )).await?.into_value()?;

    Ok(result)
}

async fn dismiss_overlays(page: &Page) -> Result<(), Box<dyn Error + Send + Sync>> {
    for _ in 0..3 {
        let dismissed = page.evaluate(
            r#"
            (() => {
                const acceptTexts = ['accept', 'agree', 'allow', 'ok', 'got it', 'continue'];
                const closeTexts = ['close', 'dismiss', 'no thanks', 'reject'];
                
                const buttons = document.querySelectorAll('button, a[role="button"], div[role="button"]');
                
                for (const btn of buttons) {
                    const text = (btn.textContent || '').toLowerCase().trim();
                    const isVisible = btn.offsetParent !== null;
                    
                    if (isVisible && text.length < 50) {
                        if (acceptTexts.some(t => text.includes(t)) || closeTexts.some(t => text.includes(t))) {
                            btn.click();
                            return true;
                        }
                    }
                }
                
                return false;
            })()
            "#
        ).await.ok().and_then(|v| v.into_value::<bool>().ok()).unwrap_or(false);
        
        if dismissed {
            sleep(Duration::from_millis(1000)).await;
        } else {
            break;
        }
    }
    
    Ok(())
}

fn get_login_url(platform: &str, target_url: &str) -> String {
    match platform {
        "google" => "https://accounts.google.com/ServiceLogin".to_string(),
        "linkedin" => "https://www.linkedin.com/uas/login".to_string(),
        "reddit" => "https://www.reddit.com/login/".to_string(),
        "github" => "https://github.com/login".to_string(),
        "facebook" => "https://www.facebook.com/login/".to_string(),
        "twitter" | "x" => "https://twitter.com/i/flow/login".to_string(),
        "instagram" => "https://www.instagram.com/accounts/login/".to_string(),
        "microsoft" => "https://login.live.com/".to_string(),
        _ => target_url.to_string(),
    }
}

#[instrument(skip(page, credentials), fields(platform, target = target_url))]
pub async fn auto_login(
    page: &Page,
    credentials: &LoginCredentials,
    target_url: &str,
) -> Result<(bool, Option<String>, Option<bool>), Box<dyn Error + Send + Sync>> {
    info!("Starting authentication");
    
    setup_stealth_mode(page).await?;
    
    let platform = credentials.platform.as_deref().unwrap_or_else(|| {
        if target_url.contains("google.com") || target_url.contains("gmail.com") { "google" }
        else if target_url.contains("linkedin.com") { "linkedin" }
        else if target_url.contains("reddit.com") { "reddit" }
        else if target_url.contains("github.com") { "github" }
        else if target_url.contains("facebook.com") { "facebook" }
        else if target_url.contains("twitter.com") || target_url.contains("x.com") { "twitter" }
        else if target_url.contains("instagram.com") { "instagram" }
        else { "generic" }
    });
    info!("Platform: {}", platform);
    
    let config = get_platform_config(platform);
    
    if let Some(cookies) = &credentials.cookies {
        info!("Attempting cookie-based authentication");
        
        if set_cookies(page, cookies).await? {
            info!("Navigating to verify cookies: {}", target_url);
            page.goto(target_url).await?;
            sleep(Duration::from_millis(3000)).await;
            
            log_page_state(page, "after_cookies").await?;
            
            if verify_authentication(page, platform).await? {
                info!("Cookie authentication successful");
                return Ok((true, Some(platform.to_string()), Some(false)));
            } else {
                warn!("Cookies did not authenticate, falling back to form login");
            }
        }
    }
    
    info!("Starting form-based login");
    
    let login_url = credentials.login_url.as_deref()
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            if !config.login_url.is_empty() {
                config.login_url.clone().to_string()
            } else {
                get_login_url(platform, target_url)
            }
        });
    
    info!("Navigating to: {}", login_url);
    page.goto(&login_url).await?;
    sleep(Duration::from_millis(2500)).await;
    
    log_page_state(page, "login_page").await?;
    dismiss_overlays(page).await?;
    
    let email_selectors: Vec<String> = if let Some(sel) = &credentials.email_selector {
        vec![sel.clone()]
    } else if !config.email_selectors.is_empty() {
        config.email_selectors.iter().map(|s| s.to_string()).collect()
    } else {
        match platform {
            "github" => vec!["#login_field".to_string(), "input[name=\"login\"]".to_string()],
            "linkedin" => vec!["#username".to_string(), "input[name=\"session_key\"]".to_string()],
            "reddit" => vec!["#loginUsername".to_string(), "input[name=\"username\"]".to_string()],
            _ => vec!["input[type=\"email\"]".to_string(), "input[name=\"email\"]".to_string()],
        }
    };
    
    let password_selectors: Vec<String> = if let Some(sel) = &credentials.password_selector {
        vec![sel.clone()]
    } else if !config.password_selectors.is_empty() {
        config.password_selectors.iter().map(|s| s.to_string()).collect()
    } else {
        match platform {
            "github" => vec!["#password".to_string(), "input[name=\"password\"]".to_string()],
            "linkedin" => vec!["#password".to_string(), "input[name=\"session_password\"]".to_string()],
            "reddit" => vec!["#loginPassword".to_string(), "input[name=\"password\"]".to_string()],
            _ => vec!["input[type=\"password\"]".to_string()],
        }
    };
    
    info!("Entering email/username");
    let email_sel = wait_for_any_element(page, &email_selectors, 15000).await?;
    if let Some(sel) = email_sel {
        if !type_into_field(page, &sel, &credentials.email).await? {
            return Err("Failed to enter email".into());
        }
        sleep(Duration::from_millis(600)).await;
    } else {
        error!("Email field not found");
        return Err("Email field not found".into());
    }
    
    let is_multi_step = ["google", "linkedin", "twitter", "facebook"].contains(&platform);
    if is_multi_step {
        let pass_visible = wait_for_any_element(page, &password_selectors, 2000).await?.is_some();
        if !pass_visible {
            info!("Multi-step detected, clicking Next");
            let _ = page.evaluate(
                r#"
                (() => {
                    const buttons = document.querySelectorAll('button, input[type="submit"]');
                    for (const btn of buttons) {
                        const text = (btn.textContent || btn.value || '').toLowerCase();
                        if ((text.includes('next') || text.includes('continue')) && btn.offsetParent !== null) {
                            btn.click();
                            return true;
                        }
                    }
                    const emailInput = document.querySelector('input[type="email"]');
                    if (emailInput) {
                        emailInput.dispatchEvent(new KeyboardEvent('keydown', {key: 'Enter', keyCode: 13, bubbles: true}));
                        return true;
                    }
                    return false;
                })()
                "#
            ).await;
            sleep(Duration::from_millis(3000)).await;
        }
    }
    
    info!("Entering password");
    let pass_sel = wait_for_any_element(page, &password_selectors, 15000).await?;
    if let Some(sel) = pass_sel {
        if !type_into_field(page, &sel, &credentials.password).await? {
            return Err("Failed to enter password".into());
        }
        sleep(Duration::from_millis(600)).await;
    } else {
        error!("Password field not found");
        return Err("Password field not found".into());
    }
    
    info!("Submitting form");
    let submitted = page.evaluate(
        r#"
        (() => {
            const submitButtons = document.querySelectorAll('button[type="submit"], input[type="submit"]');
            for (const btn of submitButtons) {
                if (btn.offsetParent !== null) {
                    btn.click();
                    return true;
                }
            }
            
            const buttons = document.querySelectorAll('button');
            for (const btn of buttons) {
                const text = (btn.textContent || '').toLowerCase();
                if ((text.includes('sign in') || text.includes('log in') || text.includes('login')) && 
                    btn.offsetParent !== null) {
                    btn.click();
                    return true;
                }
            }
            
            const passField = document.querySelector('input[type="password"]');
            if (passField) {
                passField.dispatchEvent(new KeyboardEvent('keydown', {key: 'Enter', keyCode: 13, bubbles: true}));
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
    ).await.ok().and_then(|v| v.into_value::<bool>().ok()).unwrap_or(false);
    
    if !submitted {
        warn!("Could not find submit button");
    }
    
    info!("Waiting for response");
    sleep(Duration::from_secs(5)).await;
    log_page_state(page, "after_submit").await?;
    
    let requires_2fa = page.evaluate(
        "document.body.innerText.toLowerCase().includes('verification') || document.body.innerText.toLowerCase().includes('two-factor') || document.body.innerText.toLowerCase().includes('code')"
    ).await.ok().and_then(|v| v.into_value::<bool>().ok()).unwrap_or(false);
    
    if requires_2fa {
        warn!("2FA required");
        return Ok((false, Some(platform.to_string()), Some(true)));
    }
    
    let has_error = page.evaluate(
        "document.body.innerText.toLowerCase().includes('incorrect') || document.body.innerText.toLowerCase().includes('invalid') || document.body.innerText.toLowerCase().includes('wrong')"
    ).await.ok().and_then(|v| v.into_value::<bool>().ok()).unwrap_or(false);
    
    if has_error {
        error!("Login error detected");
        return Ok((false, Some(platform.to_string()), Some(false)));
    }
    
    let is_authenticated = verify_authentication(page, platform).await?;
    
    if is_authenticated {
        info!("Login successful");
        
        let current_url = page.url().await.ok().flatten().unwrap_or_default();
        if !target_url.contains(&current_url) && target_url != &login_url {
            info!("Navigating to target: {}", target_url);
            page.goto(target_url).await?;
            sleep(Duration::from_millis(2000)).await;
        }
    } else {
        warn!("Login status unclear");
    }
    
    Ok((is_authenticated, Some(platform.to_string()), Some(false)))
}