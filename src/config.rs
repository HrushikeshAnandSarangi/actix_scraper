#[derive(Clone)]
pub struct PlatformConfig {
    pub login_url: &'static str,
    pub email_selectors: Vec<&'static str>,
    pub password_selectors: Vec<&'static str>,
    pub submit_selectors: Vec<&'static str>,
    pub wait_after_login: u64,
    pub additional_checks: Option<Vec<&'static str>>,
}
pub fn get_platform_config(platform: &str) -> PlatformConfig {
    match platform.to_lowercase().as_str() {
        "linkedin" => PlatformConfig {
            login_url: "https://www.linkedin.com/login",
            email_selectors: vec![
                "#username",
                "input[name='session_key']",
                "input[id='username']",
            ],
            password_selectors: vec![
                "#password",
                "input[name='session_password']",
                "input[id='password']",
            ],
            submit_selectors: vec![
                "button[type='submit']",
                "button[data-litms-control-urn*='login-submit']",
                ".login__form_action_container button",
            ],
            wait_after_login: 8,
            additional_checks: Some(vec![
                ".global-nav__me",
                ".feed-identity-module",
            ]),
        },
        "facebook" => PlatformConfig {
            login_url: "https://www.facebook.com/login",
            email_selectors: vec![
                "#email",
                "input[name='email']",
                "input[type='text'][name='email']",
            ],
            password_selectors: vec![
                "#pass",
                "input[name='pass']",
                "input[type='password'][name='pass']",
            ],
            submit_selectors: vec![
                "button[name='login']",
                "button[type='submit']",
                "#loginbutton",
            ],
            wait_after_login: 6,
            additional_checks: Some(vec![
                "[aria-label='Your profile']",
                "[data-pagelet='LeftRail']",
            ]),
        },
        "twitter" | "x" => PlatformConfig {
            login_url: "https://twitter.com/i/flow/login",
            email_selectors: vec![
                "input[name='text']",
                "input[autocomplete='username']",
                "input[name='session[username_or_email]']",
            ],
            password_selectors: vec![
                "input[name='password']",
                "input[type='password']",
                "input[autocomplete='current-password']",
            ],
            submit_selectors: vec![
                "[role='button'][data-testid*='LoginForm_Login_Button']",
                "button[type='submit']",
                "[data-testid='LoginForm_Login_Button']",
            ],
            wait_after_login: 7,
            additional_checks: Some(vec![
                "[data-testid='SideNav_AccountSwitcher_Button']",
                "[aria-label='Home timeline']",
            ]),
        },
        "github" => PlatformConfig {
            login_url: "https://github.com/login",
            email_selectors: vec![
                "#login_field",
                "input[name='login']",
            ],
            password_selectors: vec![
                "#password",
                "input[name='password']",
            ],
            submit_selectors: vec![
                "input[type='submit'][value='Sign in']",
                "input[name='commit']",
            ],
            wait_after_login: 5,
            additional_checks: Some(vec![
                "[aria-label='Global navigation']",
                ".Header-link--user",
            ]),
        },
        "instagram" => PlatformConfig {
            login_url: "https://www.instagram.com/accounts/login/",
            email_selectors: vec![
                "input[name='username']",
                "input[aria-label='Phone number, username, or email']",
            ],
            password_selectors: vec![
                "input[name='password']",
                "input[type='password']",
            ],
            submit_selectors: vec![
                "button[type='submit']",
            ],
            wait_after_login: 6,
            additional_checks: Some(vec![
                "[aria-label='Home']",
                "svg[aria-label='Home']",
            ]),
        },
        "reddit" => PlatformConfig {
            login_url: "https://www.reddit.com/login/",
            email_selectors: vec![
                "#loginUsername",
                "input[name='username']",
            ],
            password_selectors: vec![
                "#loginPassword",
                "input[name='password']",
            ],
            submit_selectors: vec![
                "button[type='submit']",
                ".AnimatedForm__submitButton",
            ],
            wait_after_login: 5,
            additional_checks: Some(vec![
                "[id*='USER_DROPDOWN']",
                "button[aria-label*='User']",
            ]),
        },
        _ => PlatformConfig {
            login_url: "",
            email_selectors: vec![
                "input[type='email']",
                "input[name='email']",
                "input[id='email']",
                "input[name='username']",
                "input[id='username']",
                "input[placeholder*='email' i]",
                "input[placeholder*='username' i]",
            ],
            password_selectors: vec![
                "input[type='password']",
                "input[name='password']",
                "input[id='password']",
            ],
            submit_selectors: vec![
                "button[type='submit']",
                "input[type='submit']",
            ],
            wait_after_login: 5,
            additional_checks: None,
        },
    }
}
