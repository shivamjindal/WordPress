mod content;

use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use clap::Parser;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::Level;

#[derive(Debug, Parser)]
#[command(name = "rustpress", about = "A tiny WordPress-like blog server in Rust")]
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
        .route("/", get(index))
        .route("/healthz", get(healthz))
        .route("/:slug", get(post))
        .route("/:slug/", get(post))
        .nest_service("/static", ServeDir::new("static"))
        .fallback(not_found)
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("bind {}", addr))?;

    tracing::info!("RustPress listening on http://{}", addr);
    axum::serve(listener, app)
        .await
        .context("server error")?;

    Ok(())
}

async fn healthz() -> &'static str {
    "ok"
}

async fn index(State(state): State<AppState>) -> Response {
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

