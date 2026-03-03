use std::collections::BTreeMap;

use serde::Serialize;

/// Represents a normalized front-end request used by content pipeline adapters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FrontRequest {
    pub path: String,
    pub query_string: String,
}

impl FrontRequest {
    pub fn from_parts(path: &str, query_string: &str) -> Self {
        let normalized_path = normalize_path(path);
        Self {
            path: normalized_path,
            query_string: query_string.trim().to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum FrontRouteKind {
    Home,
    Single,
    Page,
    Archive,
    Search,
    Feed,
    NotFound,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FrontRouteMatch {
    pub request: FrontRequest,
    pub kind: FrontRouteKind,
    pub query_vars: BTreeMap<String, String>,
    pub template_candidates: Vec<String>,
    pub canonical_redirect: Option<String>,
}

pub fn parse_front_route(path: &str, query_string: &str) -> FrontRouteMatch {
    let request = FrontRequest::from_parts(path, query_string);
    let query_pairs = parse_query_pairs(&request.query_string);
    let segments = split_path_segments(&request.path);
    let mut query_vars = BTreeMap::new();
    let mut kind = FrontRouteKind::NotFound;

    if request.path == "/" {
        kind = FrontRouteKind::Home;
    } else if request.path == "/feed" || request.path == "/feed/" {
        kind = FrontRouteKind::Feed;
        query_vars.insert("feed".to_string(), "rss2".to_string());
    } else if let Some(search_term) = query_pairs.get("s").filter(|value| !value.is_empty()) {
        kind = FrontRouteKind::Search;
        query_vars.insert("s".to_string(), search_term.clone());
    } else if let Some((slug, paged)) = taxonomy_archive_from_segments(&segments, "category") {
        kind = FrontRouteKind::Archive;
        query_vars.insert("category_name".to_string(), slug);
        if let Some(paged) = paged {
            query_vars.insert("paged".to_string(), paged.to_string());
        }
    } else if let Some((slug, paged)) = taxonomy_archive_from_segments(&segments, "tag") {
        kind = FrontRouteKind::Archive;
        query_vars.insert("tag".to_string(), slug);
        if let Some(paged) = paged {
            query_vars.insert("paged".to_string(), paged.to_string());
        }
    } else if let Some((slug, paged)) = taxonomy_archive_from_segments(&segments, "author") {
        kind = FrontRouteKind::Archive;
        query_vars.insert("author_name".to_string(), slug);
        if let Some(paged) = paged {
            query_vars.insert("paged".to_string(), paged.to_string());
        }
    } else {
        if segments.len() == 3
            && is_year(segments[0])
            && is_month(segments[1])
            && is_day(segments[2])
        {
            kind = FrontRouteKind::Archive;
            query_vars.insert("year".to_string(), segments[0].to_string());
            query_vars.insert("monthnum".to_string(), segments[1].to_string());
            query_vars.insert("day".to_string(), segments[2].to_string());
        } else if segments.len() == 3
            && is_year(segments[0])
            && is_month(segments[1])
            && !segments[2].is_empty()
        {
            kind = FrontRouteKind::Single;
            query_vars.insert("year".to_string(), segments[0].to_string());
            query_vars.insert("monthnum".to_string(), segments[1].to_string());
            query_vars.insert("name".to_string(), segments[2].to_string());
        } else if segments.len() == 2 && segments[0] == "page" && is_positive_integer(segments[1]) {
            kind = FrontRouteKind::Home;
            query_vars.insert("paged".to_string(), segments[1].to_string());
        } else if segments.len() == 1 && is_year(segments[0]) {
            kind = FrontRouteKind::Archive;
            query_vars.insert("year".to_string(), segments[0].to_string());
        } else if segments.len() == 2 && is_year(segments[0]) && is_month(segments[1]) {
            kind = FrontRouteKind::Archive;
            query_vars.insert("year".to_string(), segments[0].to_string());
            query_vars.insert("monthnum".to_string(), segments[1].to_string());
        } else if segments.len() == 1
            && segments[0].chars().all(|character| {
                character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
            })
        {
            kind = FrontRouteKind::Page;
            query_vars.insert("pagename".to_string(), segments[0].to_string());
        }
    }

    let template_candidates = template_candidates(kind, &query_vars);
    let canonical_redirect = canonical_redirect_target(path);

    FrontRouteMatch {
        request,
        kind,
        query_vars,
        template_candidates,
        canonical_redirect,
    }
}

pub fn extract_block_names(content: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut remaining = content;
    while let Some(start) = remaining.find("<!-- wp:") {
        let after = &remaining[start + "<!-- wp:".len()..];
        if let Some(end) = after.find("-->") {
            let mut marker = after[..end].trim();
            if marker.starts_with('/') {
                remaining = &after[end + 3..];
                continue;
            }
            if marker.ends_with('/') {
                marker = marker.trim_end_matches('/');
            }
            let name = marker.split_whitespace().next().unwrap_or_default();
            if !name.is_empty() {
                names.push(name.to_string());
            }
            remaining = &after[end + 3..];
        } else {
            break;
        }
    }
    names
}

fn template_candidates(kind: FrontRouteKind, query_vars: &BTreeMap<String, String>) -> Vec<String> {
    match kind {
        FrontRouteKind::Home => vec![
            "front-page.php".to_string(),
            "home.php".to_string(),
            "index.php".to_string(),
        ],
        FrontRouteKind::Single => vec![
            "single.php".to_string(),
            "singular.php".to_string(),
            "index.php".to_string(),
        ],
        FrontRouteKind::Page => {
            let mut templates = Vec::new();
            if let Some(slug) = query_vars.get("pagename") {
                templates.push(format!("page-{slug}.php"));
            }
            templates.push("page.php".to_string());
            templates.push("singular.php".to_string());
            templates.push("index.php".to_string());
            templates
        }
        FrontRouteKind::Archive => {
            let mut templates = Vec::new();
            if let Some(category) = query_vars.get("category_name") {
                templates.push(format!("category-{category}.php"));
            } else if let Some(tag) = query_vars.get("tag") {
                templates.push(format!("tag-{tag}.php"));
            } else if let Some(author_name) = query_vars.get("author_name") {
                templates.push(format!("author-{author_name}.php"));
                templates.push("author.php".to_string());
            } else if query_vars.contains_key("year") {
                templates.push("date.php".to_string());
            }
            templates.push("archive.php".to_string());
            templates.push("index.php".to_string());
            templates
        }
        FrontRouteKind::Search => vec!["search.php".to_string(), "index.php".to_string()],
        FrontRouteKind::Feed => vec![
            "feed-rss2.php".to_string(),
            "feed.php".to_string(),
            "index.php".to_string(),
        ],
        FrontRouteKind::NotFound => vec!["404.php".to_string(), "index.php".to_string()],
    }
}

fn canonical_redirect_target(path: &str) -> Option<String> {
    let normalized = normalize_path(path);
    if normalized == path {
        return None;
    }
    Some(normalized)
}

fn normalize_path(path: &str) -> String {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return "/".to_string();
    }

    let mut normalized = if trimmed.starts_with('/') {
        trimmed.to_string()
    } else {
        format!("/{trimmed}")
    };

    while normalized.contains("//") {
        normalized = normalized.replace("//", "/");
    }

    if normalized != "/" && !normalized.ends_with('/') && !normalized.contains('.') {
        normalized.push('/');
    }

    normalized
}

fn parse_query_pairs(query_string: &str) -> BTreeMap<String, String> {
    query_string
        .split('&')
        .filter_map(|entry| {
            let mut parts = entry.splitn(2, '=');
            let key = parts.next()?.trim();
            if key.is_empty() {
                return None;
            }
            let value = parts.next().unwrap_or_default().trim();
            Some((key.to_string(), value.to_string()))
        })
        .collect()
}

fn split_path_segments(path: &str) -> Vec<&str> {
    path.trim_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect()
}

fn is_year(value: &str) -> bool {
    value.len() == 4 && value.chars().all(|character| character.is_ascii_digit())
}

fn is_month(value: &str) -> bool {
    if value.len() != 2 || !value.chars().all(|character| character.is_ascii_digit()) {
        return false;
    }
    matches!(value.parse::<u8>(), Ok(month) if (1..=12).contains(&month))
}

fn is_day(value: &str) -> bool {
    if value.len() != 2 || !value.chars().all(|character| character.is_ascii_digit()) {
        return false;
    }
    matches!(value.parse::<u8>(), Ok(day) if (1..=31).contains(&day))
}

fn is_positive_integer(value: &str) -> bool {
    !value.is_empty() && matches!(value.parse::<u32>(), Ok(number) if number > 0)
}

fn taxonomy_archive_from_segments(segments: &[&str], base: &str) -> Option<(String, Option<u32>)> {
    if segments.len() == 2 && segments[0] == base && !segments[1].is_empty() {
        return Some((segments[1].to_string(), None));
    }
    if segments.len() == 4
        && segments[0] == base
        && !segments[1].is_empty()
        && segments[2] == "page"
        && is_positive_integer(segments[3])
    {
        return Some((segments[1].to_string(), segments[3].parse::<u32>().ok()));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_path_defaults_to_root() {
        let request = FrontRequest::from_parts("", "");
        assert_eq!(request.path, "/");
    }

    #[test]
    fn parses_single_post_route() {
        let matched = parse_front_route("/2025/02/hello-world", "");
        assert_eq!(matched.kind, FrontRouteKind::Single);
        assert_eq!(
            matched.query_vars.get("name"),
            Some(&"hello-world".to_string())
        );
        assert_eq!(
            matched.canonical_redirect,
            Some("/2025/02/hello-world/".to_string())
        );
    }

    #[test]
    fn parses_search_route() {
        let matched = parse_front_route("/search", "s=wordpress");
        assert_eq!(matched.kind, FrontRouteKind::Search);
        assert_eq!(matched.query_vars.get("s"), Some(&"wordpress".to_string()));
        assert_eq!(
            matched.template_candidates,
            vec!["search.php".to_string(), "index.php".to_string()]
        );
    }

    #[test]
    fn parses_archive_route() {
        let matched = parse_front_route("/category/news", "");
        assert_eq!(matched.kind, FrontRouteKind::Archive);
        assert_eq!(
            matched.query_vars.get("category_name"),
            Some(&"news".to_string())
        );
        assert!(matched
            .template_candidates
            .contains(&"category-news.php".to_string()));
    }

    #[test]
    fn parses_author_archive_route() {
        let matched = parse_front_route("/author/admin", "");
        assert_eq!(matched.kind, FrontRouteKind::Archive);
        assert_eq!(
            matched.query_vars.get("author_name"),
            Some(&"admin".to_string())
        );
        assert!(matched
            .template_candidates
            .contains(&"author-admin.php".to_string()));
        assert!(matched
            .template_candidates
            .contains(&"author.php".to_string()));
    }

    #[test]
    fn parses_year_archive_route() {
        let matched = parse_front_route("/2025", "");
        assert_eq!(matched.kind, FrontRouteKind::Archive);
        assert_eq!(matched.query_vars.get("year"), Some(&"2025".to_string()));
        assert_eq!(matched.canonical_redirect, Some("/2025/".to_string()));
    }

    #[test]
    fn parses_home_pagination_route() {
        let matched = parse_front_route("/page/2", "");
        assert_eq!(matched.kind, FrontRouteKind::Home);
        assert_eq!(matched.query_vars.get("paged"), Some(&"2".to_string()));
        assert_eq!(
            matched.template_candidates,
            vec![
                "front-page.php".to_string(),
                "home.php".to_string(),
                "index.php".to_string()
            ]
        );
    }

    #[test]
    fn parses_day_archive_route() {
        let matched = parse_front_route("/2025/02/03", "");
        assert_eq!(matched.kind, FrontRouteKind::Archive);
        assert_eq!(matched.query_vars.get("year"), Some(&"2025".to_string()));
        assert_eq!(matched.query_vars.get("monthnum"), Some(&"02".to_string()));
        assert_eq!(matched.query_vars.get("day"), Some(&"03".to_string()));
        assert!(matched
            .template_candidates
            .contains(&"date.php".to_string()));
    }

    #[test]
    fn parses_category_archive_pagination_route() {
        let matched = parse_front_route("/category/news/page/2", "");
        assert_eq!(matched.kind, FrontRouteKind::Archive);
        assert_eq!(
            matched.query_vars.get("category_name"),
            Some(&"news".to_string())
        );
        assert_eq!(matched.query_vars.get("paged"), Some(&"2".to_string()));
    }

    #[test]
    fn parses_author_archive_pagination_route() {
        let matched = parse_front_route("/author/admin/page/3", "");
        assert_eq!(matched.kind, FrontRouteKind::Archive);
        assert_eq!(
            matched.query_vars.get("author_name"),
            Some(&"admin".to_string())
        );
        assert_eq!(matched.query_vars.get("paged"), Some(&"3".to_string()));
    }

    #[test]
    fn parses_feed_route() {
        let matched = parse_front_route("/feed/", "");
        assert_eq!(matched.kind, FrontRouteKind::Feed);
        assert_eq!(matched.query_vars.get("feed"), Some(&"rss2".to_string()));
    }

    #[test]
    fn falls_back_to_404_for_unmatched_paths() {
        let matched = parse_front_route("/Admin", "");
        assert_eq!(matched.kind, FrontRouteKind::NotFound);
        assert_eq!(
            matched.template_candidates,
            vec!["404.php".to_string(), "index.php".to_string()]
        );
    }

    #[test]
    fn extracts_block_names_from_markup() {
        let content =
            "<!-- wp:paragraph --><p>Hello</p><!-- /wp:paragraph --><!-- wp:image {\"id\":1} /-->";
        let blocks = extract_block_names(content);
        assert_eq!(blocks, vec!["paragraph".to_string(), "image".to_string()]);
    }
}
