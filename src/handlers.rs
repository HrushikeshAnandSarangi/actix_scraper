use actix_web::{HttpResponse, web, Responder};
use crate::model::{ScrapeRequest, ScrapeResponse};
use crate::scraper::do_scrape;

pub async fn health() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

pub async fn scrape(req: web::Json<ScrapeRequest>) -> impl Responder {
    let url = req.url.clone();
    let login = req.login.clone();
    
    match do_scrape(&url, login).await {
        Ok(data) => HttpResponse::Ok().json(ScrapeResponse {
            title: data.title,
            description: data.description,
            url: url.clone(),
            text: data.text,
            images: data.images,
            links: data.links,
            success: true,
            error: None,
            login_attempted: data.login_attempted,
            login_success: data.login_success,
            platform_detected: data.platform_detected,
            requires_2fa: data.requires_2fa,
        }),
        Err(e) => HttpResponse::InternalServerError().json(ScrapeResponse {
            title: None,
            description: None,
            url: url.clone(),
            text: None,
            images: Vec::new(),
            links: Vec::new(),
            success: false,
            error: Some(e.to_string()),
            login_attempted: false,
            login_success: None,
            platform_detected: None,
            requires_2fa: None,
        })
    }
}