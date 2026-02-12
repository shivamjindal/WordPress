use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use chrono::NaiveDate;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct SiteConfig {
    pub title: String,
    pub tagline: String,
}

#[derive(Debug, Clone)]
pub struct Post {
    pub title: String,
    pub slug: String,
    pub date: NaiveDate,
    pub excerpt: String,
    pub content_html: String,
}

#[derive(Debug, Clone, Deserialize)]
struct Frontmatter {
    title: String,
    slug: Option<String>,
    date: NaiveDate,
    excerpt: Option<String>,
}

pub fn load_site(content_dir: &Path) -> Result<(SiteConfig, Vec<Post>)> {
    let site_path = content_dir.join("site.toml");
    let site_raw = fs::read_to_string(&site_path)
        .with_context(|| format!("read site config at {}", site_path.display()))?;
    let site: SiteConfig = toml::from_str(&site_raw)
        .with_context(|| format!("parse site config TOML at {}", site_path.display()))?;

    let posts_dir = content_dir.join("posts");
    let mut posts = load_posts_from_dir(&posts_dir)
        .with_context(|| format!("load posts from {}", posts_dir.display()))?;

    // Newest first, WordPress-like default.
    posts.sort_by(|a, b| b.date.cmp(&a.date));

    Ok((site, posts))
}

fn load_posts_from_dir(posts_dir: &Path) -> Result<Vec<Post>> {
    let mut posts = Vec::new();
    let entries = fs::read_dir(posts_dir)
        .with_context(|| format!("read_dir {}", posts_dir.display()))?;

    for entry in entries {
        let entry = entry.context("read_dir entry")?;
        let path = entry.path();
        if path.extension() != Some(OsStr::new("md")) {
            continue;
        }

        posts.push(load_post(&path).with_context(|| format!("load post {}", path.display()))?);
    }

    Ok(posts)
}

fn load_post(path: &Path) -> Result<Post> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let (frontmatter, body_markdown) = split_frontmatter(&raw)
        .with_context(|| format!("parse frontmatter in {}", path.display()))?;

    let title = frontmatter.title.trim().to_string();
    if title.is_empty() {
        return Err(anyhow!("frontmatter.title must not be empty"));
    }

    let slug = frontmatter
        .slug
        .unwrap_or_else(|| slugify(&title))
        .trim()
        .to_string();
    if slug.is_empty() {
        return Err(anyhow!("post slug must not be empty"));
    }

    let excerpt = frontmatter
        .excerpt
        .unwrap_or_else(|| infer_excerpt(&body_markdown));

    let content_html = markdown_to_html(&body_markdown);

    Ok(Post {
        title,
        slug,
        date: frontmatter.date,
        excerpt,
        content_html,
    })
}

fn split_frontmatter(raw: &str) -> Result<(Frontmatter, String)> {
    let raw = raw.replace("\r\n", "\n");
    let mut lines = raw.lines();
    let first = lines.next().unwrap_or_default();
    if first.trim() != "---" {
        return Err(anyhow!(
            "missing YAML frontmatter; expected first line to be `---`"
        ));
    }

    let mut fm_lines = Vec::new();
    for line in lines.by_ref() {
        if line.trim() == "---" {
            break;
        }
        fm_lines.push(line);
    }

    let fm_raw = fm_lines.join("\n");
    let frontmatter: Frontmatter =
        serde_yaml::from_str(&fm_raw).context("deserialize YAML frontmatter")?;

    let body = lines.collect::<Vec<_>>().join("\n").trim().to_string();
    Ok((frontmatter, body))
}

fn infer_excerpt(markdown: &str) -> String {
    let first_para = markdown
        .split("\n\n")
        .map(str::trim)
        .find(|p| !p.is_empty())
        .unwrap_or_default();

    // Keep it simple: strip leading markdown heading markers and truncate.
    let mut s = first_para.trim_start_matches('#').trim().to_string();
    if s.len() > 160 {
        s.truncate(157);
        s.push_str("...");
    }
    if s.is_empty() {
        s = "Post".to_string();
    }
    s
}

fn markdown_to_html(markdown: &str) -> String {
    use pulldown_cmark::{html, Options, Parser};

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(markdown, options);
    let mut out = String::new();
    html::push_html(&mut out, parser);
    out
}

fn slugify(input: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;

    for ch in input.chars() {
        let ch = ch.to_ascii_lowercase();
        let is_alnum = ch.is_ascii_alphanumeric();
        if is_alnum {
            out.push(ch);
            prev_dash = false;
        } else if !prev_dash && !out.is_empty() {
            out.push('-');
            prev_dash = true;
        }
    }

    out.trim_matches('-').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_basic() {
        assert_eq!(slugify("hello world"), "hello-world");
        assert_eq!(slugify("  Hello, World!!! "), "hello-world");
        assert_eq!(slugify("RustPress 2026"), "rustpress-2026");
    }

    #[test]
    fn split_frontmatter_parses_yaml_and_body() {
        let raw = r#"---
title: hello world
slug: hello-world
date: 2026-02-12
excerpt: My first post.
---

# Heading

Body text
"#;

        let (fm, body) = split_frontmatter(raw).expect("split_frontmatter");
        assert_eq!(fm.title, "hello world");
        assert_eq!(fm.slug.as_deref(), Some("hello-world"));
        assert_eq!(fm.date, NaiveDate::from_ymd_opt(2026, 2, 12).unwrap());
        assert_eq!(fm.excerpt.as_deref(), Some("My first post."));
        assert!(body.contains("Body text"));
    }
}

