mod content;
mod wp;

use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::{Host, Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::Response;
use axum::routing::get;
use axum::{Json, Router};
use clap::Parser;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::Level;

#[derive(Debug, Parser)]
#[command(
    name = "rustpress",
    about = "A tiny WordPress-like blog server in Rust"
)]
struct Args {
    /// Host interface to bind to.
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Port to listen on.
    #[arg(long, default_value_t = 3000)]
    port: u16,

    /// Content directory containing `site.toml` and `posts/`.
    #[arg(long, default_value = "content")]
    content_dir: PathBuf,
}

#[derive(Clone)]
struct AppState {
    site: Arc<content::SiteConfig>,
    posts: Arc<Vec<content::Post>>,
}

#[derive(Debug, serde::Deserialize)]
struct WpQuery {
    p: Option<i64>,
    name: Option<String>,
    s: Option<String>,
}

#[derive(Debug, Clone)]
struct SiteView {
    title: String,
    tagline: String,
}

#[derive(Debug, Clone)]
struct PostListItemView {
    title: String,
    slug: String,
    date: String,
    excerpt: String,
}

#[derive(Debug, Clone)]
struct PostView {
    title: String,
    date: String,
    excerpt: String,
    content_html: String,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    site: SiteView,
    posts: Vec<PostListItemView>,
}

#[derive(Template)]
#[template(path = "post.html")]
struct PostTemplate {
    site: SiteView,
    post: PostView,
}

#[derive(Template)]
#[template(path = "404.html")]
struct NotFoundTemplate {
    site: SiteView,
    path: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rustpress=info,tower_http=info".into()),
        )
        .init();

    let args = Args::parse();
    let host: IpAddr = args
        .host
        .parse()
        .with_context(|| format!("parse host ip {}", args.host))?;
    let addr = SocketAddr::from((host, args.port));

    let (site, posts) = content::load_site(&args.content_dir)?;
    let state = AppState {
        site: Arc::new(site),
        posts: Arc::new(posts),
    };

    let app = Router::new()
        // WordPress front controller equivalents.
        .route("/", get(index))
        .route("/index.php", get(index))
        .route("/healthz", get(healthz))
        .route("/robots.txt", get(robots_txt))
        .route("/feed", get(feed))
        .route("/feed/", get(feed))
        .route("/wp-json", get(wp_json_index))
        .route("/wp-json/", get(wp_json_index))
        .route("/wp-json/wp/v2/posts", get(wp_json_posts))
        .route("/wp-json/wp/v2/posts/", get(wp_json_posts))
        .route("/wp-json/wp/v2/posts/:id", get(wp_json_post_by_id))
        .route("/wp-json/wp/v2/posts/:id/", get(wp_json_post_by_id))
        .route("/wp-login.php", get(wp_login))
        .route("/wp-admin", get(wp_admin))
        .route("/wp-admin/", get(wp_admin))
        .route("/xmlrpc.php", get(xmlrpc_get))
        .route("/:slug", get(post))
        .route("/:slug/", get(post))
        .nest_service("/static", ServeDir::new("static"))
        // Keep these paths WordPress-compatible for assets, even if PHP isn't executed.
        .nest_service("/wp-content", ServeDir::new("wp-content"))
        .nest_service("/wp-includes", ServeDir::new("wp-includes"))
        .nest_service("/wp-admin", ServeDir::new("wp-admin"))
        .fallback(not_found)
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("bind {}", addr))?;

    tracing::info!("RustPress listening on http://{}", addr);
    axum::serve(listener, app).await.context("server error")?;

    Ok(())
}

async fn healthz() -> &'static str {
    "ok"
}

async fn index(State(state): State<AppState>, Query(q): Query<WpQuery>) -> Response {
    // WordPress also serves single posts via query string (e.g. /?p=1 or /?name=slug).
    if q.p.is_some() || q.name.is_some() {
        let slug = if let Some(name) = q.name.clone() {
            name
        } else if let Some(id) = q.p {
            state
                .posts
                .iter()
                .find(|p| p.id.unwrap_or(0) == id)
                .map(|p| p.slug.clone())
                .unwrap_or_else(|| "__missing__".to_string())
        } else {
            "__missing__".to_string()
        };

        if slug != "__missing__" {
            // Reuse the post handler logic.
            return post(State(state), Path(slug)).await;
        }
    }

    let site = SiteView {
        title: state.site.title.clone(),
        tagline: state.site.tagline.clone(),
    };

    let posts = state
        .posts
        .iter()
        .map(|p| PostListItemView {
            title: p.title.clone(),
            slug: p.slug.clone(),
            date: p.date.format("%b %e, %Y").to_string(),
            excerpt: p.excerpt.clone(),
        })
        .collect();

    IndexTemplate { site, posts }.into_response()
}

async fn wp_login(State(state): State<AppState>) -> Response {
    let site = SiteView {
        title: state.site.title.clone(),
        tagline: state.site.tagline.clone(),
    };

    // Minimal placeholder so common WordPress paths exist.
    let html = format!(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Log In ‹ {}</title>
    <link rel="stylesheet" href="/static/styles.css" />
  </head>
  <body>
    <main class="wrap">
      <section class="hero">
        <h1>Log In</h1>
        <p class="muted">This is a Rust implementation stub for <span class="code">/wp-login.php</span>.</p>
        <div class="card">
          <p>RustPress focuses on front-end rendering and WordPress-compatible APIs for this demo.</p>
          <p><a class="button" href="/">Back to site</a></p>
        </div>
      </section>
    </main>
  </body>
</html>"#,
        html_escape(&site.title)
    );

    (StatusCode::OK, html).into_response()
}

async fn wp_admin() -> Response {
    // Minimal placeholder so /wp-admin/ exists.
    (
        StatusCode::OK,
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Dashboard ‹ RustPress</title>
    <link rel="stylesheet" href="/static/styles.css" />
  </head>
  <body>
    <main class="wrap">
      <section class="hero">
        <h1>Dashboard</h1>
        <p class="muted">RustPress demo admin placeholder for <span class=\"code\">/wp-admin/</span>.</p>
        <div class="card">
          <p>Try the public site:</p>
          <p><a class="button" href="/">View site</a></p>
        </div>
      </section>
    </main>
  </body>
</html>"#,
    )
        .into_response()
}

async fn robots_txt() -> Response {
    (
        StatusCode::OK,
        "User-agent: *\nDisallow: /wp-admin/\nAllow: /wp-admin/admin-ajax.php\n",
    )
        .into_response()
}

async fn xmlrpc_get() -> Response {
    (
        StatusCode::METHOD_NOT_ALLOWED,
        "XML-RPC server accepts POST requests only.",
    )
        .into_response()
}

async fn post(State(state): State<AppState>, Path(slug): Path<String>) -> Response {
    let site = SiteView {
        title: state.site.title.clone(),
        tagline: state.site.tagline.clone(),
    };

    let Some(p) = state.posts.iter().find(|p| p.slug == slug) else {
        let tpl = NotFoundTemplate {
            site,
            path: format!("/{slug}/"),
        };
        return (StatusCode::NOT_FOUND, tpl).into_response();
    };

    let post = PostView {
        title: p.title.clone(),
        date: p.date.format("%b %e, %Y").to_string(),
        excerpt: p.excerpt.clone(),
        content_html: p.content_html.clone(),
    };

    PostTemplate { site, post }.into_response()
}

async fn wp_json_index(State(state): State<AppState>) -> Json<wp::WpApiIndex> {
    Json(wp::wp_api_index(&state.site))
}

async fn wp_json_posts(
    State(state): State<AppState>,
    Host(host): Host,
    Query(q): Query<WpQuery>,
) -> Response {
    let base_url = format!("http://{}", host);

    // Support a few common query patterns (`?p=1`, `?name=hello-world`, `?s=term`).
    let mut posts: Vec<&content::Post> = state.posts.iter().collect();

    if let Some(id) = q.p {
        posts.retain(|p| p.id.unwrap_or(0) == id);
    }
    if let Some(name) = q.name.as_deref() {
        posts.retain(|p| p.slug == name);
    }
    if let Some(search) = q.s.as_deref() {
        let s = search.to_ascii_lowercase();
        posts.retain(|p| {
            p.title.to_ascii_lowercase().contains(&s) || p.excerpt.to_ascii_lowercase().contains(&s)
        });
    }

    let out: Vec<wp::WpPost> = posts
        .into_iter()
        .map(|p| wp::post_to_wp_post(p, &base_url))
        .collect();

    Json(out).into_response()
}

async fn wp_json_post_by_id(
    State(state): State<AppState>,
    Host(host): Host,
    Path(id): Path<i64>,
) -> Response {
    let base_url = format!("http://{}", host);
    let Some(p) = state.posts.iter().find(|p| p.id.unwrap_or(0) == id) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    Json(wp::post_to_wp_post(p, &base_url)).into_response()
}

async fn feed(State(state): State<AppState>, Host(host): Host) -> Response {
    // Minimal RSS 2.0 feed, close to WordPress' shape.
    let base_url = format!("http://{}", host);
    let channel_title = xml_escape(&state.site.title);
    let channel_desc = xml_escape(&state.site.tagline);
    let channel_link = xml_escape(&base_url);

    let mut items = String::new();
    for p in state.posts.iter().take(20) {
        let link = format!("{}/{}/", base_url.trim_end_matches('/'), p.slug);
        let title = xml_escape(&p.title);
        let description = xml_escape(&p.excerpt);
        let pub_date = format!("{} 00:00:00 +0000", p.date.format("%Y-%m-%d"));

        items.push_str(&format!(
            r#"<item>
  <title>{}</title>
  <link>{}</link>
  <guid isPermaLink="true">{}</guid>
  <pubDate>{}</pubDate>
  <description>{}</description>
</item>
"#,
            title,
            xml_escape(&link),
            xml_escape(&link),
            xml_escape(&pub_date),
            description
        ));
    }

    let rss = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
<channel>
  <title>{}</title>
  <link>{}</link>
  <description>{}</description>
  {}
</channel>
</rss>
"#,
        channel_title, channel_link, channel_desc, items
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        "application/rss+xml; charset=utf-8".parse().unwrap(),
    );
    (StatusCode::OK, headers, rss).into_response()
}

async fn not_found(State(state): State<AppState>, uri: axum::http::Uri) -> Response {
    let site = SiteView {
        title: state.site.title.clone(),
        tagline: state.site.tagline.clone(),
    };

    (
        StatusCode::NOT_FOUND,
        NotFoundTemplate {
            site,
            path: uri.path().to_string(),
        },
    )
        .into_response()
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
