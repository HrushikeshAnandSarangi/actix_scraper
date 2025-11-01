use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Clone)]
pub struct ScrapeRequest {
    pub url: String,
    
    #[serde(default)]
    pub login: Option<LoginCredentials>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct LoginCredentials {
    pub email: String,
    pub password: String,
    
    #[serde(default)]
    pub platform: Option<String>,
    
    #[serde(default)]
    pub login_url: Option<String>,
    #[serde(default)]
    pub email_selector: Option<String>,
    #[serde(default)]
    pub password_selector: Option<String>,
    #[serde(default)]
    pub submit_selector: Option<String>,
    
    #[serde(default)]
    pub wait_after_login_secs: Option<u64>,
    #[serde(default)]
    pub cookies: Option<Vec<CookieData>>,
}

#[derive(Deserialize, Clone, Serialize, Debug)] // Added Debug
pub struct CookieData {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)] // Added Debug
pub struct ImageData {
    pub src: String,
    pub alt: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LinkData {
    pub href: String,
    pub text: String,
}

#[derive(Serialize)]
pub struct ScrapeResponse {
    pub title: Option<String>,
    pub description: Option<String>,
    pub url: String,
    pub text: Option<String>,
    pub images: Vec<ImageData>,
    pub links: Vec<LinkData>,
    pub success: bool,
    pub error: Option<String>,
    pub login_attempted: bool,
    pub login_success: Option<bool>,
    pub platform_detected: Option<String>,
    pub requires_2fa: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct ScrapedData {
    pub title: Option<String>,
    pub description: Option<String>,
    pub text: Option<String>,
    pub images: Vec<ImageData>,
    pub links: Vec<LinkData>,
    pub login_attempted: bool,
    pub login_success: Option<bool>,
    pub platform_detected: Option<String>,
    pub requires_2fa: Option<bool>,
}
