// src/error.rs
use std::fmt;

#[derive(Debug)]
pub enum ScrapeError {
    BrowserLaunch(String),
    Navigation(String),
    PageCreation(String),
    EvaluationFailed(String),
    LoginFailed(String),
    TwoFactorAuthRequired,
    ContentExtraction(String),
}

impl fmt::Display for ScrapeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ScrapeError::BrowserLaunch(e) => write!(f, "Failed to launch browser: {}", e),
            ScrapeError::Navigation(e) => write!(f, "Navigation failed: {}", e),
            ScrapeError::PageCreation(e) => write!(f, "Failed to create new page: {}", e),
            ScrapeError::EvaluationFailed(e) => write!(f, "JavaScript evaluation failed: {}", e),
            ScrapeError::LoginFailed(e) => write!(f, "Automatic login failed: {}", e),
            ScrapeError::TwoFactorAuthRequired => write!(f, "2FA is required, cannot proceed automatically"),
            ScrapeError::ContentExtraction(e) => write!(f, "Failed to extract content: {}", e),
        }
    }
}

impl std::error::Error for ScrapeError {}