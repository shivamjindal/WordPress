use chrono::NaiveDate;
use serde::Serialize;

use crate::content;

#[derive(Debug, Clone, Serialize)]
pub struct WpApiIndex {
    pub name: String,
    pub description: String,
    pub namespaces: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RenderedText {
    pub rendered: String,
    pub protected: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Guid {
    pub rendered: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WpPost {
    pub id: i64,
    pub date: String,
    pub date_gmt: String,
    pub guid: Guid,
    pub modified: String,
    pub modified_gmt: String,
    pub slug: String,
    pub status: String,
    #[serde(rename = "type")]
    pub post_type: String,
    pub link: String,
    pub title: RenderedText,
    pub content: RenderedText,
    pub excerpt: RenderedText,
    pub author: i64,
    pub featured_media: i64,
    pub comment_status: String,
    pub ping_status: String,
    pub sticky: bool,
    pub template: String,
    pub format: String,
    pub categories: Vec<i64>,
    pub tags: Vec<i64>,
}

pub fn wp_api_index(site: &content::SiteConfig) -> WpApiIndex {
    WpApiIndex {
        name: site.title.clone(),
        description: site.tagline.clone(),
        namespaces: vec!["wp/v2".to_string(), "oembed/1.0".to_string()],
    }
}

pub fn post_to_wp_post(post: &content::Post, base_url: &str) -> WpPost {
    let id = post.id.unwrap_or_else(|| stable_id_from_slug(&post.slug));

    // WordPress uses local time + GMT, but for this demo we treat date as UTC midnight.
    let date = iso_datetime_from_date(post.date);
    let date_gmt = date.clone();
    let modified = date.clone();
    let modified_gmt = date.clone();

    let link = format!("{}/{}/", base_url.trim_end_matches('/'), post.slug);

    WpPost {
        id,
        date,
        date_gmt,
        guid: Guid {
            rendered: link.clone(),
        },
        modified,
        modified_gmt,
        slug: post.slug.clone(),
        status: post.status.clone().unwrap_or_else(|| "publish".to_string()),
        post_type: "post".to_string(),
        link: link.clone(),
        title: RenderedText {
            rendered: post.title.clone(),
            protected: false,
        },
        content: RenderedText {
            rendered: post.content_html.clone(),
            protected: false,
        },
        excerpt: RenderedText {
            // WordPress typically wraps excerpts in <p>.
            rendered: format!("<p>{}</p>", html_escape_text(&post.excerpt)),
            protected: false,
        },
        author: 1,
        featured_media: 0,
        comment_status: "open".to_string(),
        ping_status: "open".to_string(),
        sticky: false,
        template: String::new(),
        format: "standard".to_string(),
        categories: vec![1],
        tags: vec![],
    }
}

fn iso_datetime_from_date(date: NaiveDate) -> String {
    format!("{}T00:00:00", date.format("%Y-%m-%d"))
}

fn stable_id_from_slug(slug: &str) -> i64 {
    // Deterministic, stable across runs; not cryptographic.
    let mut acc: u64 = 1469598103934665603;
    for b in slug.as_bytes() {
        acc ^= *b as u64;
        acc = acc.wrapping_mul(1099511628211);
    }
    // Ensure positive and keep it smallish.
    (acc % 10_000_000) as i64 + 1
}

fn html_escape_text(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
