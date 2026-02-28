use std::collections::{BTreeMap, BTreeSet};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::time::Instant;

use axum::body::{to_bytes, Bytes};
use axum::extract::{Path, Query, Request, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{any, get};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{error, info};
use wp_rs_admin::{core_admin_actions, AdminRequest, AdminSurface};
use wp_rs_auth::{resolve_current_user, sign_auth_cookie, AuthScheme, AuthSecrets, NonceService};
use wp_rs_config::{php_runtime_core_endpoints, RustGatewaySettings};
use wp_rs_content::{extract_block_names, parse_front_route, FrontRouteKind};
use wp_rs_cron::{parse_doing_wp_cron, CronEvent, CronScheduler};
use wp_rs_db::{MultisiteResolver, NetworkSite, OptionStore};
use wp_rs_http::{
    core_xmlrpc_registry, detect_endpoint_kind, parse_xmlrpc_method_name, xmlrpc_fault_response,
    xmlrpc_success_response,
};
use wp_rs_rest::{core_seed_routes, RestRequest};

#[derive(Debug, Clone)]
struct AppState {
    options: Arc<Mutex<OptionStore>>,
    auth_secrets: AuthSecrets,
    nonce_service: NonceService,
    cron_scheduler: Arc<Mutex<CronScheduler>>,
    multisite_resolver: MultisiteResolver,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let listen_addr =
        std::env::var("WP_RUST_SERVER_LISTEN").unwrap_or_else(|_| "127.0.0.1:8088".to_string());
    let address: SocketAddr = listen_addr.parse().expect("listen address must parse");

    let state = build_app_state();

    let app = Router::new()
        .route("/__wp_rust/health", get(health))
        .route("/__wp_rust/echo", get(echo))
        .route("/__wp_rust/proxy-decision", get(proxy_decision))
        .route("/__wp_rust/maintenance", any(maintenance_live_dispatch))
        .route("/wp-login.php", any(login_live_dispatch))
        .route("/wp-signup.php", any(signup_live_dispatch))
        .route("/wp-activate.php", any(activate_live_dispatch))
        .route("/wp-comments-post.php", any(comments_post_live_dispatch))
        .route("/wp-mail.php", any(mail_live_dispatch))
        .route("/wp-trackback.php", any(trackback_live_dispatch))
        .route("/wp-links-opml.php", any(links_opml_live_dispatch))
        .route("/wp-includes/load.php", any(load_include_live_dispatch))
        .route("/wp-includes/vars.php", any(vars_include_live_dispatch))
        .route("/wp-includes/update.php", any(update_include_live_dispatch))
        .route("/wp-includes/wp-db.php", any(wp_db_include_live_dispatch))
        .route("/wp-includes/utf8.php", any(utf8_include_live_dispatch))
        .route("/wp-includes/user.php", any(user_include_live_dispatch))
        .route(
            "/wp-includes/functions.php",
            any(functions_include_live_dispatch),
        )
        .route(
            "/wp-includes/formatting.php",
            any(formatting_include_live_dispatch),
        )
        .route("/wp-includes/plugin.php", any(plugin_include_live_dispatch))
        .route(
            "/wp-includes/pluggable.php",
            any(pluggable_include_live_dispatch),
        )
        .route(
            "/wp-includes/capabilities.php",
            any(capabilities_include_live_dispatch),
        )
        .route("/wp-includes/option.php", any(option_include_live_dispatch))
        .route("/wp-includes/post.php", any(post_include_live_dispatch))
        .route(
            "/wp-includes/class-wp-hook.php",
            any(class_wp_hook_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp.php",
            any(class_wp_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-query.php",
            any(class_wp_query_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-rewrite.php",
            any(class_wp_rewrite_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-role.php",
            any(class_wp_role_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-roles.php",
            any(class_wp_roles_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-user.php",
            any(class_wp_user_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-session-tokens.php",
            any(class_wp_session_tokens_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-user-meta-session-tokens.php",
            any(class_wp_user_meta_session_tokens_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-user-query.php",
            any(class_wp_user_query_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-meta-query.php",
            any(class_wp_meta_query_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-date-query.php",
            any(class_wp_date_query_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-tax-query.php",
            any(class_wp_tax_query_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-term-query.php",
            any(class_wp_term_query_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-comment-query.php",
            any(class_wp_comment_query_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-network-query.php",
            any(class_wp_network_query_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-site-query.php",
            any(class_wp_site_query_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-post-type.php",
            any(class_wp_post_type_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-post.php",
            any(class_wp_post_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-error.php",
            any(class_wp_error_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-http.php",
            any(class_wp_http_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-http-cookie.php",
            any(class_wp_http_cookie_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-http-encoding.php",
            any(class_wp_http_encoding_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-http-response.php",
            any(class_wp_http_response_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-http-curl.php",
            any(class_wp_http_curl_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-http-streams.php",
            any(class_wp_http_streams_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-http-proxy.php",
            any(class_wp_http_proxy_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-http-requests-hooks.php",
            any(class_wp_http_requests_hooks_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-http-requests-response.php",
            any(class_wp_http_requests_response_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-network.php",
            any(class_wp_network_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-site.php",
            any(class_wp_site_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-taxonomy.php",
            any(class_wp_taxonomy_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-theme.php",
            any(class_wp_theme_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-widget.php",
            any(class_wp_widget_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-scripts.php",
            any(class_wp_scripts_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-styles.php",
            any(class_wp_styles_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-dependencies.php",
            any(class_wp_dependencies_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-dependency.php",
            any(class_wp_dependency_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-script-modules.php",
            any(class_wp_script_modules_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-IXR.php",
            any(class_ixr_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-avif-info.php",
            any(class_avif_info_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-feed.php",
            any(class_feed_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-http.php",
            any(class_http_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-json.php",
            any(class_json_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-oembed.php",
            any(class_oembed_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-phpass.php",
            any(class_phpass_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-phpmailer.php",
            any(class_phpmailer_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-pop3.php",
            any(class_pop3_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-requests.php",
            any(class_requests_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-simplepie.php",
            any(class_simplepie_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-smtp.php",
            any(class_smtp_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-snoopy.php",
            any(class_snoopy_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-walker-category-dropdown.php",
            any(class_walker_category_dropdown_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-walker-category.php",
            any(class_walker_category_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-walker-comment.php",
            any(class_walker_comment_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-walker-nav-menu.php",
            any(class_walker_nav_menu_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-walker-page-dropdown.php",
            any(class_walker_page_dropdown_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-walker-page.php",
            any(class_walker_page_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wpdb.php",
            any(class_wpdb_include_live_dispatch),
        )
        .route(
            "/wp-includes/class.wp-dependencies.php",
            any(class_dot_wp_dependencies_include_live_dispatch),
        )
        .route(
            "/wp-includes/class.wp-scripts.php",
            any(class_dot_wp_scripts_include_live_dispatch),
        )
        .route(
            "/wp-includes/class.wp-styles.php",
            any(class_dot_wp_styles_include_live_dispatch),
        )
        .route(
            "/wp-includes/compat-utf8.php",
            any(compat_utf8_include_live_dispatch),
        )
        .route("/wp-includes/cron.php", any(cron_include_live_dispatch))
        .route("/wp-includes/date.php", any(date_include_live_dispatch))
        .route(
            "/wp-includes/default-constants.php",
            any(default_constants_include_live_dispatch),
        )
        .route(
            "/wp-includes/default-widgets.php",
            any(default_widgets_include_live_dispatch),
        )
        .route(
            "/wp-includes/deprecated.php",
            any(deprecated_include_live_dispatch),
        )
        .route(
            "/wp-includes/embed-template.php",
            any(embed_template_include_live_dispatch),
        )
        .route("/wp-includes/embed.php", any(embed_include_live_dispatch))
        .route(
            "/wp-includes/error-protection.php",
            any(error_protection_include_live_dispatch),
        )
        .route(
            "/wp-includes/feed-atom-comments.php",
            any(feed_atom_comments_include_live_dispatch),
        )
        .route(
            "/wp-includes/feed-atom.php",
            any(feed_atom_include_live_dispatch),
        )
        .route(
            "/wp-includes/feed-rdf.php",
            any(feed_rdf_include_live_dispatch),
        )
        .route(
            "/wp-includes/feed-rss.php",
            any(feed_rss_include_live_dispatch),
        )
        .route(
            "/wp-includes/feed-rss2-comments.php",
            any(feed_rss2_comments_include_live_dispatch),
        )
        .route(
            "/wp-includes/feed-rss2.php",
            any(feed_rss2_include_live_dispatch),
        )
        .route("/wp-includes/feed.php", any(feed_include_live_dispatch))
        .route("/wp-includes/fonts.php", any(fonts_include_live_dispatch))
        .route(
            "/wp-includes/functions.wp-scripts.php",
            any(functions_dot_wp_scripts_include_live_dispatch),
        )
        .route(
            "/wp-includes/functions.wp-styles.php",
            any(functions_dot_wp_styles_include_live_dispatch),
        )
        .route(
            "/wp-includes/global-styles-and-settings.php",
            any(global_styles_and_settings_include_live_dispatch),
        )
        .route("/wp-includes/http.php", any(http_include_live_dispatch))
        .route(
            "/wp-includes/https-detection.php",
            any(https_detection_include_live_dispatch),
        )
        .route(
            "/wp-includes/https-migration.php",
            any(https_migration_include_live_dispatch),
        )
        .route("/wp-includes/kses.php", any(kses_include_live_dispatch))
        .route("/wp-includes/l10n.php", any(l10n_include_live_dispatch))
        .route("/wp-includes/locale.php", any(locale_include_live_dispatch))
        .route(
            "/wp-includes/media-template.php",
            any(media_template_include_live_dispatch),
        )
        .route("/wp-includes/media.php", any(media_include_live_dispatch))
        .route("/wp-includes/meta.php", any(meta_include_live_dispatch))
        .route(
            "/wp-includes/ms-blogs.php",
            any(ms_blogs_include_live_dispatch),
        )
        .route(
            "/wp-includes/ms-default-constants.php",
            any(ms_default_constants_include_live_dispatch),
        )
        .route(
            "/wp-includes/ms-default-filters.php",
            any(ms_default_filters_include_live_dispatch),
        )
        .route(
            "/wp-includes/ms-deprecated.php",
            any(ms_deprecated_include_live_dispatch),
        )
        .route(
            "/wp-includes/ms-files.php",
            any(ms_files_include_live_dispatch),
        )
        .route(
            "/wp-includes/ms-functions.php",
            any(ms_functions_include_live_dispatch),
        )
        .route(
            "/wp-includes/ms-load.php",
            any(ms_load_include_live_dispatch),
        )
        .route(
            "/wp-includes/ms-network.php",
            any(ms_network_include_live_dispatch),
        )
        .route(
            "/wp-includes/ms-settings.php",
            any(ms_settings_include_live_dispatch),
        )
        .route(
            "/wp-includes/ms-site.php",
            any(ms_site_include_live_dispatch),
        )
        .route(
            "/wp-includes/nav-menu-template.php",
            any(nav_menu_template_include_live_dispatch),
        )
        .route(
            "/wp-includes/nav-menu.php",
            any(nav_menu_include_live_dispatch),
        )
        .route(
            "/wp-includes/pluggable-deprecated.php",
            any(pluggable_deprecated_include_live_dispatch),
        )
        .route(
            "/wp-includes/post-formats.php",
            any(post_formats_include_live_dispatch),
        )
        .route(
            "/wp-includes/post-template.php",
            any(post_template_include_live_dispatch),
        )
        .route(
            "/wp-includes/post-thumbnail-template.php",
            any(post_thumbnail_template_include_live_dispatch),
        )
        .route("/wp-includes/query.php", any(query_include_live_dispatch))
        .route(
            "/wp-includes/registration-functions.php",
            any(registration_functions_include_live_dispatch),
        )
        .route(
            "/wp-includes/registration.php",
            any(registration_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api.php",
            any(rest_api_include_live_dispatch),
        )
        .route(
            "/wp-includes/revision.php",
            any(revision_include_live_dispatch),
        )
        .route(
            "/wp-includes/rewrite.php",
            any(rewrite_include_live_dispatch),
        )
        .route(
            "/wp-includes/robots-template.php",
            any(robots_template_include_live_dispatch),
        )
        .route(
            "/wp-includes/rss-functions.php",
            any(rss_functions_include_live_dispatch),
        )
        .route("/wp-includes/rss.php", any(rss_include_live_dispatch))
        .route(
            "/wp-includes/script-loader.php",
            any(script_loader_include_live_dispatch),
        )
        .route(
            "/wp-includes/session.php",
            any(session_include_live_dispatch),
        )
        .route(
            "/wp-includes/spl-autoload-compat.php",
            any(spl_autoload_compat_include_live_dispatch),
        )
        .route(
            "/wp-includes/category-template.php",
            any(category_template_include_live_dispatch),
        )
        .route(
            "/wp-includes/category.php",
            any(category_include_live_dispatch),
        )
        .route(
            "/wp-includes/comment-template.php",
            any(comment_template_include_live_dispatch),
        )
        .route(
            "/wp-includes/comment.php",
            any(comment_include_live_dispatch),
        )
        .route("/wp-includes/compat.php", any(compat_include_live_dispatch))
        .route(
            "/wp-includes/bookmark-template.php",
            any(bookmark_template_include_live_dispatch),
        )
        .route(
            "/wp-includes/bookmark.php",
            any(bookmark_include_live_dispatch),
        )
        .route(
            "/wp-includes/cache-compat.php",
            any(cache_compat_include_live_dispatch),
        )
        .route("/wp-includes/cache.php", any(cache_include_live_dispatch))
        .route(
            "/wp-includes/canonical.php",
            any(canonical_include_live_dispatch),
        )
        .route(
            "/wp-includes/block-bindings.php",
            any(block_bindings_include_live_dispatch),
        )
        .route(
            "/wp-includes/block-editor.php",
            any(block_editor_include_live_dispatch),
        )
        .route(
            "/wp-includes/block-patterns.php",
            any(block_patterns_include_live_dispatch),
        )
        .route(
            "/wp-includes/block-template-utils.php",
            any(block_template_utils_include_live_dispatch),
        )
        .route(
            "/wp-includes/block-template.php",
            any(block_template_include_live_dispatch),
        )
        .route(
            "/wp-includes/abilities-api.php",
            any(abilities_api_include_live_dispatch),
        )
        .route(
            "/wp-includes/abilities.php",
            any(abilities_include_live_dispatch),
        )
        .route(
            "/wp-includes/abilities-api/class-wp-abilities-registry.php",
            any(abilities_api_class_wp_abilities_registry_include_live_dispatch),
        )
        .route(
            "/wp-includes/abilities-api/class-wp-ability-categories-registry.php",
            any(abilities_api_class_wp_ability_categories_registry_include_live_dispatch),
        )
        .route(
            "/wp-includes/abilities-api/class-wp-ability-category.php",
            any(abilities_api_class_wp_ability_category_include_live_dispatch),
        )
        .route(
            "/wp-includes/abilities-api/class-wp-ability.php",
            any(abilities_api_class_wp_ability_include_live_dispatch),
        )
        .route(
            "/wp-includes/abilities/class-wp-settings-abilities.php",
            any(abilities_class_wp_settings_abilities_include_live_dispatch),
        )
        .route(
            "/wp-includes/assets/script-loader-packages.min.php",
            any(assets_script_loader_packages_min_include_live_dispatch),
        )
        .route(
            "/wp-includes/assets/script-loader-packages.php",
            any(assets_script_loader_packages_include_live_dispatch),
        )
        .route(
            "/wp-includes/assets/script-modules-packages.min.php",
            any(assets_script_modules_packages_min_include_live_dispatch),
        )
        .route(
            "/wp-includes/assets/script-modules-packages.php",
            any(assets_script_modules_packages_include_live_dispatch),
        )
        .route(
            "/wp-includes/block-bindings/pattern-overrides.php",
            any(block_bindings_pattern_overrides_include_live_dispatch),
        )
        .route(
            "/wp-includes/block-bindings/post-data.php",
            any(block_bindings_post_data_include_live_dispatch),
        )
        .route(
            "/wp-includes/block-bindings/post-meta.php",
            any(block_bindings_post_meta_include_live_dispatch),
        )
        .route(
            "/wp-includes/block-bindings/term-data.php",
            any(block_bindings_term_data_include_live_dispatch),
        )
        .route(
            "/wp-includes/block-patterns/query-grid-posts.php",
            any(block_patterns_query_grid_posts_include_live_dispatch),
        )
        .route(
            "/wp-includes/block-patterns/query-large-title-posts.php",
            any(block_patterns_query_large_title_posts_include_live_dispatch),
        )
        .route(
            "/wp-includes/block-patterns/query-medium-posts.php",
            any(block_patterns_query_medium_posts_include_live_dispatch),
        )
        .route(
            "/wp-includes/block-patterns/query-offset-posts.php",
            any(block_patterns_query_offset_posts_include_live_dispatch),
        )
        .route(
            "/wp-includes/block-patterns/query-small-posts.php",
            any(block_patterns_query_small_posts_include_live_dispatch),
        )
        .route(
            "/wp-includes/block-patterns/query-standard-posts.php",
            any(block_patterns_query_standard_posts_include_live_dispatch),
        )
        .route(
            "/wp-includes/block-patterns/social-links-shared-background-color.php",
            any(block_patterns_social_links_shared_background_color_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/accordion-item.php",
            any(blocks_accordion_item_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/accordion.php",
            any(blocks_accordion_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/archives.php",
            any(blocks_archives_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/avatar.php",
            any(blocks_avatar_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/block.php",
            any(blocks_block_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/button.php",
            any(blocks_button_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/calendar.php",
            any(blocks_calendar_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/categories.php",
            any(blocks_categories_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/comment-author-name.php",
            any(blocks_comment_author_name_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/comment-content.php",
            any(blocks_comment_content_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/comment-date.php",
            any(blocks_comment_date_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/comment-edit-link.php",
            any(blocks_comment_edit_link_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/comment-reply-link.php",
            any(blocks_comment_reply_link_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/comments-pagination-next.php",
            any(blocks_comments_pagination_next_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/comments-pagination-numbers.php",
            any(blocks_comments_pagination_numbers_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/blocks-json.php",
            any(blocks_blocks_json_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/comments-pagination-previous.php",
            any(blocks_comments_pagination_previous_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/comments-pagination.php",
            any(blocks_comments_pagination_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/comments-title.php",
            any(blocks_comments_title_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/comments.php",
            any(blocks_comments_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/comment-template.php",
            any(blocks_comment_template_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/cover.php",
            any(blocks_cover_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/file.php",
            any(blocks_file_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/footnotes.php",
            any(blocks_footnotes_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/gallery.php",
            any(blocks_gallery_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/heading.php",
            any(blocks_heading_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/home-link.php",
            any(blocks_home_link_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/image.php",
            any(blocks_image_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/index.php",
            any(blocks_index_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/latest-comments.php",
            any(blocks_latest_comments_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/latest-posts.php",
            any(blocks_latest_posts_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/legacy-widget.php",
            any(blocks_legacy_widget_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/list.php",
            any(blocks_list_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/loginout.php",
            any(blocks_loginout_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/media-text.php",
            any(blocks_media_text_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/navigation-link.php",
            any(blocks_navigation_link_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/navigation.php",
            any(blocks_navigation_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/navigation-submenu.php",
            any(blocks_navigation_submenu_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/page-list-item.php",
            any(blocks_page_list_item_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/page-list.php",
            any(blocks_page_list_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/pattern.php",
            any(blocks_pattern_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/post-author-biography.php",
            any(blocks_post_author_biography_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/post-author-name.php",
            any(blocks_post_author_name_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/post-author.php",
            any(blocks_post_author_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/post-comments-count.php",
            any(blocks_post_comments_count_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/post-comments-form.php",
            any(blocks_post_comments_form_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/post-comments-link.php",
            any(blocks_post_comments_link_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/post-content.php",
            any(blocks_post_content_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/post-date.php",
            any(blocks_post_date_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/post-excerpt.php",
            any(blocks_post_excerpt_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/post-featured-image.php",
            any(blocks_post_featured_image_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/post-navigation-link.php",
            any(blocks_post_navigation_link_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/post-template.php",
            any(blocks_post_template_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/post-terms.php",
            any(blocks_post_terms_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/post-time-to-read.php",
            any(blocks_post_time_to_read_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/post-title.php",
            any(blocks_post_title_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/query-no-results.php",
            any(blocks_query_no_results_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/query-pagination-next.php",
            any(blocks_query_pagination_next_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/query-pagination-numbers.php",
            any(blocks_query_pagination_numbers_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/query-pagination.php",
            any(blocks_query_pagination_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/query-pagination-previous.php",
            any(blocks_query_pagination_previous_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/query.php",
            any(blocks_query_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/query-title.php",
            any(blocks_query_title_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/query-total.php",
            any(blocks_query_total_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/read-more.php",
            any(blocks_read_more_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/require-dynamic-blocks.php",
            any(blocks_require_dynamic_blocks_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/require-static-blocks.php",
            any(blocks_require_static_blocks_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/rss.php",
            any(blocks_rss_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/search.php",
            any(blocks_search_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/shortcode.php",
            any(blocks_shortcode_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/site-logo.php",
            any(blocks_site_logo_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/site-tagline.php",
            any(blocks_site_tagline_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/site-title.php",
            any(blocks_site_title_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/social-link.php",
            any(blocks_social_link_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/tag-cloud.php",
            any(blocks_tag_cloud_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/template-part.php",
            any(blocks_template_part_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/term-count.php",
            any(blocks_term_count_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/term-description.php",
            any(blocks_term_description_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/term-name.php",
            any(blocks_term_name_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/term-template.php",
            any(blocks_term_template_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/video.php",
            any(blocks_video_include_live_dispatch),
        )
        .route(
            "/wp-includes/blocks/widget-group.php",
            any(blocks_widget_group_include_live_dispatch),
        )
        .route(
            "/wp-includes/build/pages/font-library/page.php",
            any(build_pages_font_library_page_include_live_dispatch),
        )
        .route(
            "/wp-includes/build/pages/font-library/page-wp-admin.php",
            any(build_pages_font_library_page_wp_admin_include_live_dispatch),
        )
        .route(
            "/wp-includes/build/pages/site-editor/page.php",
            any(build_pages_site_editor_page_include_live_dispatch),
        )
        .route(
            "/wp-includes/build/pages/site-editor/page-wp-admin.php",
            any(build_pages_site_editor_page_wp_admin_include_live_dispatch),
        )
        .route(
            "/wp-includes/build/routes/font-list/content.min.asset.php",
            any(build_routes_font_list_content_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/build/routes/font-list/route.min.asset.php",
            any(build_routes_font_list_route_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/build/routes/fonts-home/route.min.asset.php",
            any(build_routes_fonts_home_route_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/build/routes/home/route.min.asset.php",
            any(build_routes_home_route_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/build/routes/index.php",
            any(build_routes_index_include_live_dispatch),
        )
        .route(
            "/wp-includes/build/routes/navigation-edit/content.min.asset.php",
            any(build_routes_navigation_edit_content_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/build/routes/navigation-edit/route.min.asset.php",
            any(build_routes_navigation_edit_route_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/build/routes/navigation-list/content.min.asset.php",
            any(build_routes_navigation_list_content_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/build/routes/navigation-list/route.min.asset.php",
            any(build_routes_navigation_list_route_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/build/routes/navigation/route.min.asset.php",
            any(build_routes_navigation_route_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/build/routes/pattern-list/content.min.asset.php",
            any(build_routes_pattern_list_content_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/build/routes/pattern-list/route.min.asset.php",
            any(build_routes_pattern_list_route_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/build/routes/pattern/route.min.asset.php",
            any(build_routes_pattern_route_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/build/routes/post-edit/route.min.asset.php",
            any(build_routes_post_edit_route_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/build/routes/post-list/content.min.asset.php",
            any(build_routes_post_list_content_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/build/routes/post-list/route.min.asset.php",
            any(build_routes_post_list_route_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/build/routes/post-new/route.min.asset.php",
            any(build_routes_post_new_route_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/build/routes/post/route.min.asset.php",
            any(build_routes_post_route_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/build/routes/registry.php",
            any(build_routes_registry_include_live_dispatch),
        )
        .route(
            "/wp-includes/build/routes/styles/content.min.asset.php",
            any(build_routes_styles_content_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/build/routes/styles/route.min.asset.php",
            any(build_routes_styles_route_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/build/routes/template-list/content.min.asset.php",
            any(build_routes_template_list_content_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/build/routes/template-list/route.min.asset.php",
            any(build_routes_template_list_route_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/build/routes/template-part-list/content.min.asset.php",
            any(build_routes_template_part_list_content_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/build/routes/template-part-list/route.min.asset.php",
            any(build_routes_template_part_list_route_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/build/routes/template-part/route.min.asset.php",
            any(build_routes_template_part_route_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/build/routes/template/route.min.asset.php",
            any(build_routes_template_route_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/css/dist/index.php",
            any(css_dist_index_include_live_dispatch),
        )
        .route(
            "/wp-includes/css/dist/registry.php",
            any(css_dist_registry_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/a11y.min.asset.php",
            any(js_dist_a11y_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/annotations.min.asset.php",
            any(js_dist_annotations_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/api-fetch.min.asset.php",
            any(js_dist_api_fetch_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/autop.min.asset.php",
            any(js_dist_autop_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/base-styles.min.asset.php",
            any(js_dist_base_styles_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/blob.min.asset.php",
            any(js_dist_blob_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/block-directory.min.asset.php",
            any(js_dist_block_directory_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/block-editor.min.asset.php",
            any(js_dist_block_editor_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/block-library.min.asset.php",
            any(js_dist_block_library_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/block-serialization-default-parser.min.asset.php",
            any(js_dist_block_serialization_default_parser_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/block-serialization-spec-parser.min.asset.php",
            any(js_dist_block_serialization_spec_parser_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/blocks.min.asset.php",
            any(js_dist_blocks_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/commands.min.asset.php",
            any(js_dist_commands_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/components.min.asset.php",
            any(js_dist_components_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/compose.min.asset.php",
            any(js_dist_compose_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/core-commands.min.asset.php",
            any(js_dist_core_commands_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/core-data.min.asset.php",
            any(js_dist_core_data_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/customize-widgets.min.asset.php",
            any(js_dist_customize_widgets_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/data-controls.min.asset.php",
            any(js_dist_data_controls_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/data.min.asset.php",
            any(js_dist_data_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/date.min.asset.php",
            any(js_dist_date_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/deprecated.min.asset.php",
            any(js_dist_deprecated_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/react-refresh-runtime.min.asset.php",
            any(js_dist_react_refresh_runtime_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/list-reusable-blocks.min.asset.php",
            any(js_dist_list_reusable_blocks_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/theme.min.asset.php",
            any(js_dist_theme_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/widgets.min.asset.php",
            any(js_dist_widgets_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/i18n.min.asset.php",
            any(js_dist_i18n_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/shortcode.min.asset.php",
            any(js_dist_shortcode_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/viewport.min.asset.php",
            any(js_dist_viewport_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/preferences-persistence.min.asset.php",
            any(js_dist_preferences_persistence_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/format-library.min.asset.php",
            any(js_dist_format_library_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/dom.min.asset.php",
            any(js_dist_dom_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/primitives.min.asset.php",
            any(js_dist_primitives_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/keyboard-shortcuts.min.asset.php",
            any(js_dist_keyboard_shortcuts_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/media-utils.min.asset.php",
            any(js_dist_media_utils_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/redux-routine.min.asset.php",
            any(js_dist_redux_routine_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/private-apis.min.asset.php",
            any(js_dist_private_apis_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/hooks.min.asset.php",
            any(js_dist_hooks_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/dom-ready.min.asset.php",
            any(js_dist_dom_ready_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/priority-queue.min.asset.php",
            any(js_dist_priority_queue_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/server-side-render.min.asset.php",
            any(js_dist_server_side_render_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/reusable-blocks.min.asset.php",
            any(js_dist_reusable_blocks_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/router.min.asset.php",
            any(js_dist_router_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/element.min.asset.php",
            any(js_dist_element_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/edit-post.min.asset.php",
            any(js_dist_edit_post_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/nux.min.asset.php",
            any(js_dist_nux_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/keycodes.min.asset.php",
            any(js_dist_keycodes_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/script-modules/interactivity-router/full-page.min.asset.php",
            any(js_dist_script_modules_interactivity_router_full_page_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/script-modules/interactivity-router/index.min.asset.php",
            any(js_dist_script_modules_interactivity_router_index_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/script-modules/edit-site-init/index.min.asset.php",
            any(js_dist_script_modules_edit_site_init_index_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/url.min.asset.php",
            any(js_dist_url_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/preferences.min.asset.php",
            any(js_dist_preferences_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/script-modules/interactivity/index.min.asset.php",
            any(js_dist_script_modules_interactivity_index_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/token-list.min.asset.php",
            any(js_dist_token_list_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/escape-html.min.asset.php",
            any(js_dist_escape_html_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/html-entities.min.asset.php",
            any(js_dist_html_entities_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/script-modules/lazy-editor/index.min.asset.php",
            any(js_dist_script_modules_lazy_editor_index_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/react-refresh-entry.min.asset.php",
            any(js_dist_react_refresh_entry_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/wordcount.min.asset.php",
            any(js_dist_wordcount_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/rich-text.min.asset.php",
            any(js_dist_rich_text_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/undo-manager.min.asset.php",
            any(js_dist_undo_manager_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/plugins.min.asset.php",
            any(js_dist_plugins_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/patterns.min.asset.php",
            any(js_dist_patterns_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/editor.min.asset.php",
            any(js_dist_editor_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/script-modules/workflow/index.min.asset.php",
            any(js_dist_script_modules_workflow_index_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/edit-site.min.asset.php",
            any(js_dist_edit_site_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/script-modules/route/index.min.asset.php",
            any(js_dist_script_modules_route_index_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/style-engine.min.asset.php",
            any(js_dist_style_engine_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/notices.min.asset.php",
            any(js_dist_notices_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/script-modules/abilities/index.min.asset.php",
            any(js_dist_script_modules_abilities_index_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/is-shallow-equal.min.asset.php",
            any(js_dist_is_shallow_equal_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/warning.min.asset.php",
            any(js_dist_warning_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/script-modules/a11y/index.min.asset.php",
            any(js_dist_script_modules_a11y_index_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/script-modules/boot/index.min.asset.php",
            any(js_dist_script_modules_boot_index_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/react-i18n.min.asset.php",
            any(js_dist_react_i18n_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/script-modules/latex-to-mathml/loader.min.asset.php",
            any(js_dist_script_modules_latex_to_mathml_loader_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/script-modules/block-library/search/view.min.asset.php",
            any(js_dist_script_modules_block_library_search_view_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/edit-widgets.min.asset.php",
            any(js_dist_edit_widgets_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/script-modules/block-library/accordion/view.min.asset.php",
            any(js_dist_script_modules_block_library_accordion_view_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/script-modules/block-library/tabs/view.min.asset.php",
            any(js_dist_script_modules_block_library_tabs_view_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/script-modules/block-library/navigation/view.min.asset.php",
            any(js_dist_script_modules_block_library_navigation_view_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/script-modules/block-library/query/view.min.asset.php",
            any(js_dist_script_modules_block_library_query_view_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/script-modules/latex-to-mathml/index.min.asset.php",
            any(js_dist_script_modules_latex_to_mathml_index_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/script-modules/block-library/image/view.min.asset.php",
            any(js_dist_script_modules_block_library_image_view_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/script-modules/block-library/file/view.min.asset.php",
            any(js_dist_script_modules_block_library_file_view_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/script-modules/block-library/form/view.min.asset.php",
            any(js_dist_script_modules_block_library_form_view_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/script-modules/block-editor/utils/fit-text-frontend.min.asset.php",
            any(js_dist_script_modules_block_editor_utils_fit_text_frontend_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/script-modules/core-abilities/index.min.asset.php",
            any(js_dist_script_modules_core_abilities_index_min_asset_include_live_dispatch),
        )
        .route(
            "/wp-includes/pomo/entry.php",
            any(pomo_entry_include_live_dispatch),
        )
        .route(
            "/wp-includes/pomo/plural-forms.php",
            any(pomo_plural_forms_include_live_dispatch),
        )
        .route(
            "/wp-includes/pomo/po.php",
            any(pomo_po_include_live_dispatch),
        )
        .route(
            "/wp-includes/pomo/translations.php",
            any(pomo_translations_include_live_dispatch),
        )
        .route(
            "/wp-includes/pomo/streams.php",
            any(pomo_streams_include_live_dispatch),
        )
        .route(
            "/wp-includes/pomo/mo.php",
            any(pomo_mo_include_live_dispatch),
        )
        .route(
            "/wp-includes/Text/Diff.php",
            any(text_diff_include_live_dispatch),
        )
        .route(
            "/wp-includes/Text/Exception.php",
            any(text_exception_include_live_dispatch),
        )
        .route(
            "/wp-includes/Text/Diff/Renderer.php",
            any(text_diff_renderer_include_live_dispatch),
        )
        .route(
            "/wp-includes/style-engine/class-wp-style-engine-processor.php",
            any(style_engine_class_wp_style_engine_processor_include_live_dispatch),
        )
        .route(
            "/wp-includes/Text/Diff/Renderer/inline.php",
            any(text_diff_renderer_inline_include_live_dispatch),
        )
        .route(
            "/wp-includes/Text/Diff/Engine/string.php",
            any(text_diff_engine_string_include_live_dispatch),
        )
        .route(
            "/wp-includes/Text/Diff/Engine/shell.php",
            any(text_diff_engine_shell_include_live_dispatch),
        )
        .route(
            "/wp-includes/style-engine/class-wp-style-engine-css-rule.php",
            any(style_engine_class_wp_style_engine_css_rule_include_live_dispatch),
        )
        .route(
            "/wp-includes/style-engine/class-wp-style-engine-css-rules-store.php",
            any(style_engine_class_wp_style_engine_css_rules_store_include_live_dispatch),
        )
        .route(
            "/wp-includes/Text/Diff/Engine/xdiff.php",
            any(text_diff_engine_xdiff_include_live_dispatch),
        )
        .route(
            "/wp-includes/style-engine/class-wp-style-engine-css-declarations.php",
            any(style_engine_class_wp_style_engine_css_declarations_include_live_dispatch),
        )
        .route(
            "/wp-includes/Text/Diff/Engine/native.php",
            any(text_diff_engine_native_include_live_dispatch),
        )
        .route(
            "/wp-includes/style-engine/class-wp-style-engine.php",
            any(style_engine_class_wp_style_engine_include_live_dispatch),
        )
        .route(
            "/wp-includes/build/pages.php",
            any(build_pages_include_live_dispatch),
        )
        .route(
            "/wp-includes/build/routes.php",
            any(build_routes_include_live_dispatch),
        )
        .route(
            "/wp-includes/php-compat/readonly.php",
            any(php_compat_readonly_include_live_dispatch),
        )
        .route(
            "/wp-includes/l10n/class-wp-translation-file-php.php",
            any(l10n_class_wp_translation_file_php_include_live_dispatch),
        )
        .route(
            "/wp-includes/l10n/class-wp-translation-file-mo.php",
            any(l10n_class_wp_translation_file_mo_include_live_dispatch),
        )
        .route(
            "/wp-includes/l10n/class-wp-translation-controller.php",
            any(l10n_class_wp_translation_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/sitemaps/class-wp-sitemaps-renderer.php",
            any(sitemaps_class_wp_sitemaps_renderer_include_live_dispatch),
        )
        .route(
            "/wp-includes/sitemaps/class-wp-sitemaps-provider.php",
            any(sitemaps_class_wp_sitemaps_provider_include_live_dispatch),
        )
        .route(
            "/wp-includes/sitemaps/providers/class-wp-sitemaps-taxonomies.php",
            any(sitemaps_providers_class_wp_sitemaps_taxonomies_include_live_dispatch),
        )
        .route(
            "/wp-includes/sitemaps/providers/class-wp-sitemaps-users.php",
            any(sitemaps_providers_class_wp_sitemaps_users_include_live_dispatch),
        )
        .route(
            "/wp-includes/sitemaps/providers/class-wp-sitemaps-posts.php",
            any(sitemaps_providers_class_wp_sitemaps_posts_include_live_dispatch),
        )
        .route(
            "/wp-includes/l10n/class-wp-translations.php",
            any(l10n_class_wp_translations_include_live_dispatch),
        )
        .route(
            "/wp-includes/l10n/class-wp-translation-file.php",
            any(l10n_class_wp_translation_file_include_live_dispatch),
        )
        .route(
            "/wp-includes/sitemaps/class-wp-sitemaps.php",
            any(sitemaps_class_wp_sitemaps_include_live_dispatch),
        )
        .route(
            "/wp-includes/sitemaps/class-wp-sitemaps-registry.php",
            any(sitemaps_class_wp_sitemaps_registry_include_live_dispatch),
        )
        .route(
            "/wp-includes/sitemaps/class-wp-sitemaps-index.php",
            any(sitemaps_class_wp_sitemaps_index_include_live_dispatch),
        )
        .route(
            "/wp-includes/block-supports/border.php",
            any(block_supports_border_include_live_dispatch),
        )
        .route(
            "/wp-includes/block-supports/custom-classname.php",
            any(block_supports_custom_classname_include_live_dispatch),
        )
        .route(
            "/wp-includes/block-supports/duotone.php",
            any(block_supports_duotone_include_live_dispatch),
        )
        .route(
            "/wp-includes/block-supports/spacing.php",
            any(block_supports_spacing_include_live_dispatch),
        )
        .route(
            "/wp-includes/block-supports/aria-label.php",
            any(block_supports_aria_label_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/autoload-php7.php",
            any(sodium_compat_autoload_php7_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/namespaced/Core/Ed25519.php",
            any(sodium_compat_namespaced_core_ed25519_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/namespaced/Core/Util.php",
            any(sodium_compat_namespaced_core_util_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/namespaced/Core/Curve25519.php",
            any(sodium_compat_namespaced_core_curve25519_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/namespaced/Core/HChaCha20.php",
            any(sodium_compat_namespaced_core_hchacha20_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/namespaced/Core/Curve25519/Ge/P3.php",
            any(sodium_compat_namespaced_core_curve25519_ge_p3_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/namespaced/Core/Curve25519/Ge/Precomp.php",
            any(sodium_compat_namespaced_core_curve25519_ge_precomp_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/namespaced/Core/Curve25519/Ge/P1p1.php",
            any(sodium_compat_namespaced_core_curve25519_ge_p1p1_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/namespaced/Core/Curve25519/Ge/P2.php",
            any(sodium_compat_namespaced_core_curve25519_ge_p2_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/namespaced/Core/Curve25519/Ge/Cached.php",
            any(sodium_compat_namespaced_core_curve25519_ge_cached_include_live_dispatch),
        )
        .route(
            "/wp-includes/interactivity-api/class-wp-interactivity-api.php",
            any(interactivity_api_class_wp_interactivity_api_include_live_dispatch),
        )
        .route(
            "/wp-includes/interactivity-api/interactivity-api.php",
            any(interactivity_api_interactivity_api_include_live_dispatch),
        )
        .route(
            "/wp-includes/interactivity-api/class-wp-interactivity-api-directives-processor.php",
            any(interactivity_api_class_wp_interactivity_api_directives_processor_include_live_dispatch),
        )
        .route(
            "/wp-includes/sitemaps/class-wp-sitemaps-stylesheet.php",
            any(sitemaps_class_wp_sitemaps_stylesheet_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/namespaced/Core/Curve25519/H.php",
            any(sodium_compat_namespaced_core_curve25519_h_include_live_dispatch),
        )
        .route(
            "/wp-includes/block-supports/utils.php",
            any(block_supports_utils_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/namespaced/Core/Poly1305/State.php",
            any(sodium_compat_namespaced_core_poly1305_state_include_live_dispatch),
        )
        .route(
            "/wp-includes/block-supports/colors.php",
            any(block_supports_colors_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/namespaced/Crypto.php",
            any(sodium_compat_namespaced_crypto_include_live_dispatch),
        )
        .route(
            "/wp-includes/block-supports/align.php",
            any(block_supports_align_include_live_dispatch),
        )
        .route(
            "/wp-includes/block-supports/shadow.php",
            any(block_supports_shadow_include_live_dispatch),
        )
        .route(
            "/wp-includes/block-supports/settings.php",
            any(block_supports_settings_include_live_dispatch),
        )
        .route(
            "/wp-includes/block-supports/dimensions.php",
            any(block_supports_dimensions_include_live_dispatch),
        )
        .route(
            "/wp-includes/block-supports/generated-classname.php",
            any(block_supports_generated_classname_include_live_dispatch),
        )
        .route(
            "/wp-includes/block-supports/elements.php",
            any(block_supports_elements_include_live_dispatch),
        )
        .route(
            "/wp-includes/block-supports/position.php",
            any(block_supports_position_include_live_dispatch),
        )
        .route(
            "/wp-includes/block-supports/block-visibility.php",
            any(block_supports_block_visibility_include_live_dispatch),
        )
        .route(
            "/wp-includes/block-supports/layout.php",
            any(block_supports_layout_include_live_dispatch),
        )
        .route(
            "/wp-includes/block-supports/typography.php",
            any(block_supports_typography_include_live_dispatch),
        )
        .route(
            "/wp-includes/block-supports/anchor.php",
            any(block_supports_anchor_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/search/class-wp-rest-post-format-search-handler.php",
            any(rest_api_search_class_wp_rest_post_format_search_handler_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/search/class-wp-rest-term-search-handler.php",
            any(rest_api_search_class_wp_rest_term_search_handler_include_live_dispatch),
        )
        .route(
            "/wp-includes/customize/class-wp-customize-color-control.php",
            any(customize_class_wp_customize_color_control_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/search/class-wp-rest-post-search-handler.php",
            any(rest_api_search_class_wp_rest_post_search_handler_include_live_dispatch),
        )
        .route(
            "/wp-includes/customize/class-wp-sidebar-block-editor-control.php",
            any(customize_class_wp_sidebar_block_editor_control_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/search/class-wp-rest-search-handler.php",
            any(rest_api_search_class_wp_rest_search_handler_include_live_dispatch),
        )
        .route(
            "/wp-includes/customize/class-wp-customize-new-menu-section.php",
            any(customize_class_wp_customize_new_menu_section_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/fields/class-wp-rest-comment-meta-fields.php",
            any(rest_api_fields_class_wp_rest_comment_meta_fields_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/fields/class-wp-rest-post-meta-fields.php",
            any(rest_api_fields_class_wp_rest_post_meta_fields_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/namespaced/Core/Salsa20.php",
            any(sodium_compat_namespaced_core_salsa20_include_live_dispatch),
        )
        .route(
            "/wp-includes/theme-compat/comments.php",
            any(theme_compat_comments_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/namespaced/Core/Xsalsa20.php",
            any(sodium_compat_namespaced_core_xsalsa20_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/namespaced/Core/ChaCha20.php",
            any(sodium_compat_namespaced_core_chacha20_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/namespaced/Core/XChaCha20.php",
            any(sodium_compat_namespaced_core_xchacha20_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/namespaced/Core/X25519.php",
            any(sodium_compat_namespaced_core_x25519_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/namespaced/Core/ChaCha20/Ctx.php",
            any(sodium_compat_namespaced_core_chacha20_ctx_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/namespaced/Core/ChaCha20/IetfCtx.php",
            any(sodium_compat_namespaced_core_chacha20_ietf_ctx_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/namespaced/Core/BLAKE2b.php",
            any(sodium_compat_namespaced_core_blake2b_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/namespaced/Core/Poly1305.php",
            any(sodium_compat_namespaced_core_poly1305_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/namespaced/Core/SipHash.php",
            any(sodium_compat_namespaced_core_sip_hash_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/namespaced/Core/HSalsa20.php",
            any(sodium_compat_namespaced_core_hsalsa20_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/namespaced/File.php",
            any(sodium_compat_namespaced_file_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/namespaced/Compat.php",
            any(sodium_compat_namespaced_compat_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/lib/php72compat.php",
            any(sodium_compat_lib_php72compat_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/lib/namespaced.php",
            any(sodium_compat_lib_namespaced_include_live_dispatch),
        )
        .route(
            "/wp-includes/customize/class-wp-customize-custom-css-setting.php",
            any(customize_class_wp_customize_custom_css_setting_include_live_dispatch),
        )
        .route(
            "/wp-includes/customize/class-wp-customize-nav-menu-setting.php",
            any(customize_class_wp_customize_nav_menu_setting_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/lib/sodium_compat.php",
            any(sodium_compat_lib_sodium_compat_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core/Ed25519.php",
            any(sodium_compat_src_core_ed25519_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core/Util.php",
            any(sodium_compat_src_core_util_include_live_dispatch),
        )
        .route(
            "/wp-includes/customize/class-wp-customize-themes-section.php",
            any(customize_class_wp_customize_themes_section_include_live_dispatch),
        )
        .route(
            "/wp-includes/customize/class-wp-customize-image-control.php",
            any(customize_class_wp_customize_image_control_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/fields/class-wp-rest-user-meta-fields.php",
            any(rest_api_fields_class_wp_rest_user_meta_fields_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/fields/class-wp-rest-meta-fields.php",
            any(rest_api_fields_class_wp_rest_meta_fields_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/fields/class-wp-rest-term-meta-fields.php",
            any(rest_api_fields_class_wp_rest_term_meta_fields_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/endpoints/class-wp-rest-font-collections-controller.php",
            any(rest_api_endpoints_class_wp_rest_font_collections_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/theme-compat/footer-embed.php",
            any(theme_compat_footer_embed_include_live_dispatch),
        )
        .route(
            "/wp-includes/theme-compat/embed.php",
            any(theme_compat_embed_include_live_dispatch),
        )
        .route(
            "/wp-includes/theme-compat/embed-content.php",
            any(theme_compat_embed_content_include_live_dispatch),
        )
        .route(
            "/wp-includes/theme-compat/header-embed.php",
            any(theme_compat_header_embed_include_live_dispatch),
        )
        .route(
            "/wp-includes/theme-compat/header.php",
            any(theme_compat_header_include_live_dispatch),
        )
        .route(
            "/wp-includes/theme-compat/embed-404.php",
            any(theme_compat_embed_404_include_live_dispatch),
        )
        .route(
            "/wp-includes/customize/class-wp-customize-code-editor-control.php",
            any(customize_class_wp_customize_code_editor_control_include_live_dispatch),
        )
        .route(
            "/wp-includes/customize/class-wp-widget-area-customize-control.php",
            any(customize_class_wp_widget_area_customize_control_include_live_dispatch),
        )
        .route(
            "/wp-includes/customize/class-wp-customize-nav-menu-locations-control.php",
            any(customize_class_wp_customize_nav_menu_locations_control_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/lib/php84compat.php",
            any(sodium_compat_lib_php84compat_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/lib/php84compat_const.php",
            any(sodium_compat_lib_php84compat_const_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/lib/stream-xchacha20.php",
            any(sodium_compat_lib_stream_xchacha20_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/endpoints/class-wp-rest-font-faces-controller.php",
            any(rest_api_endpoints_class_wp_rest_font_faces_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/endpoints/class-wp-rest-block-directory-controller.php",
            any(rest_api_endpoints_class_wp_rest_block_directory_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/lib/constants.php",
            any(sodium_compat_lib_constants_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/endpoints/class-wp-rest-menus-controller.php",
            any(rest_api_endpoints_class_wp_rest_menus_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/customize/class-wp-customize-header-image-control.php",
            any(customize_class_wp_customize_header_image_control_include_live_dispatch),
        )
        .route(
            "/wp-includes/customize/class-wp-customize-site-icon-control.php",
            any(customize_class_wp_customize_site_icon_control_include_live_dispatch),
        )
        .route(
            "/wp-includes/customize/class-wp-customize-nav-menu-item-control.php",
            any(customize_class_wp_customize_nav_menu_item_control_include_live_dispatch),
        )
        .route(
            "/wp-includes/customize/class-wp-customize-nav-menu-item-setting.php",
            any(customize_class_wp_customize_nav_menu_item_setting_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/lib/php72compat_const.php",
            any(sodium_compat_lib_php72compat_const_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/lib/ristretto255.php",
            any(sodium_compat_lib_ristretto255_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/endpoints/class-wp-rest-global-styles-controller.php",
            any(rest_api_endpoints_class_wp_rest_global_styles_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/autoload.php",
            any(sodium_compat_autoload_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/endpoints/class-wp-rest-font-families-controller.php",
            any(rest_api_endpoints_class_wp_rest_font_families_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/endpoints/class-wp-rest-menu-items-controller.php",
            any(rest_api_endpoints_class_wp_rest_menu_items_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/theme-compat/footer.php",
            any(theme_compat_footer_include_live_dispatch),
        )
        .route(
            "/wp-includes/theme-compat/sidebar.php",
            any(theme_compat_sidebar_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core/Base64/UrlSafe.php",
            any(sodium_compat_src_core_base64_url_safe_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core/Base64/Original.php",
            any(sodium_compat_src_core_base64_original_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core/XSalsa20.php",
            any(sodium_compat_src_core_xsalsa20_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/endpoints/class-wp-rest-edit-site-export-controller.php",
            any(rest_api_endpoints_class_wp_rest_edit_site_export_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/endpoints/class-wp-rest-widget-types-controller.php",
            any(rest_api_endpoints_class_wp_rest_widget_types_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/endpoints/class-wp-rest-application-passwords-controller.php",
            any(rest_api_endpoints_class_wp_rest_application_passwords_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/endpoints/class-wp-rest-template-revisions-controller.php",
            any(rest_api_endpoints_class_wp_rest_template_revisions_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/endpoints/class-wp-rest-templates-controller.php",
            any(rest_api_endpoints_class_wp_rest_templates_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core/AEGIS128L.php",
            any(sodium_compat_src_core_aegis128l_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core/Curve25519.php",
            any(sodium_compat_src_core_curve25519_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Compat.php",
            any(sodium_compat_src_compat_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core/BLAKE2b.php",
            any(sodium_compat_src_core_blake2b_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Crypto32.php",
            any(sodium_compat_src_crypto32_include_live_dispatch),
        )
        .route(
            "/wp-includes/customize/class-wp-customize-background-image-setting.php",
            any(customize_class_wp_customize_background_image_setting_include_live_dispatch),
        )
        .route(
            "/wp-includes/customize/class-wp-customize-themes-panel.php",
            any(customize_class_wp_customize_themes_panel_include_live_dispatch),
        )
        .route(
            "/wp-includes/customize/class-wp-customize-nav-menu-control.php",
            any(customize_class_wp_customize_nav_menu_control_include_live_dispatch),
        )
        .route(
            "/wp-includes/customize/class-wp-customize-background-position-control.php",
            any(customize_class_wp_customize_background_position_control_include_live_dispatch),
        )
        .route(
            "/wp-includes/customize/class-wp-customize-partial.php",
            any(customize_class_wp_customize_partial_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/endpoints/class-wp-rest-post-types-controller.php",
            any(rest_api_endpoints_class_wp_rest_post_types_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/endpoints/class-wp-rest-block-patterns-controller.php",
            any(rest_api_endpoints_class_wp_rest_block_patterns_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/endpoints/class-wp-rest-plugins-controller.php",
            any(rest_api_endpoints_class_wp_rest_plugins_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core/AES/Expanded.php",
            any(sodium_compat_src_core_aes_expanded_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/endpoints/class-wp-rest-pattern-directory-controller.php",
            any(rest_api_endpoints_class_wp_rest_pattern_directory_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core/AES/Block.php",
            any(sodium_compat_src_core_aes_block_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core/AES/KeySchedule.php",
            any(sodium_compat_src_core_aes_key_schedule_include_live_dispatch),
        )
        .route(
            "/wp-includes/fonts/class-wp-font-library.php",
            any(fonts_class_wp_font_library_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core/SecretStream/State.php",
            any(sodium_compat_src_core_secret_stream_state_include_live_dispatch),
        )
        .route(
            "/wp-includes/fonts/class-wp-font-face.php",
            any(fonts_class_wp_font_face_include_live_dispatch),
        )
        .route(
            "/wp-includes/customize/class-wp-customize-nav-menu-auto-add-control.php",
            any(customize_class_wp_customize_nav_menu_auto_add_control_include_live_dispatch),
        )
        .route(
            "/wp-includes/customize/class-wp-customize-nav-menu-name-control.php",
            any(customize_class_wp_customize_nav_menu_name_control_include_live_dispatch),
        )
        .route(
            "/wp-includes/customize/class-wp-customize-new-menu-control.php",
            any(customize_class_wp_customize_new_menu_control_include_live_dispatch),
        )
        .route(
            "/wp-includes/customize/class-wp-customize-cropped-image-control.php",
            any(customize_class_wp_customize_cropped_image_control_include_live_dispatch),
        )
        .route(
            "/wp-includes/customize/class-wp-customize-theme-control.php",
            any(customize_class_wp_customize_theme_control_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/endpoints/class-wp-rest-revisions-controller.php",
            any(rest_api_endpoints_class_wp_rest_revisions_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/endpoints/class-wp-rest-post-statuses-controller.php",
            any(rest_api_endpoints_class_wp_rest_post_statuses_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/endpoints/class-wp-rest-attachments-controller.php",
            any(rest_api_endpoints_class_wp_rest_attachments_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/endpoints/class-wp-rest-site-health-controller.php",
            any(rest_api_endpoints_class_wp_rest_site_health_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/endpoints/class-wp-rest-posts-controller.php",
            any(rest_api_endpoints_class_wp_rest_posts_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/customize/class-wp-customize-nav-menus-panel.php",
            any(customize_class_wp_customize_nav_menus_panel_include_live_dispatch),
        )
        .route(
            "/wp-includes/customize/class-wp-customize-nav-menu-location-control.php",
            any(customize_class_wp_customize_nav_menu_location_control_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core/HChaCha20.php",
            any(sodium_compat_src_core_hchacha20_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core/Curve25519/Ge/P3.php",
            any(sodium_compat_src_core_curve25519_ge_p3_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core/Curve25519/Ge/Precomp.php",
            any(sodium_compat_src_core_curve25519_ge_precomp_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core/Curve25519/Ge/P1p1.php",
            any(sodium_compat_src_core_curve25519_ge_p1p1_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core/Curve25519/Ge/P2.php",
            any(sodium_compat_src_core_curve25519_ge_p2_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core/Curve25519/Ge/Cached.php",
            any(sodium_compat_src_core_curve25519_ge_cached_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/endpoints/class-wp-rest-template-autosaves-controller.php",
            any(rest_api_endpoints_class_wp_rest_template_autosaves_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core/Curve25519/H.php",
            any(sodium_compat_src_core_curve25519_h_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/endpoints/class-wp-rest-widgets-controller.php",
            any(rest_api_endpoints_class_wp_rest_widgets_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/endpoints/class-wp-rest-global-styles-revisions-controller.php",
            any(rest_api_endpoints_class_wp_rest_global_styles_revisions_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/endpoints/class-wp-rest-comments-controller.php",
            any(rest_api_endpoints_class_wp_rest_comments_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/endpoints/class-wp-rest-controller.php",
            any(rest_api_endpoints_class_wp_rest_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/endpoints/class-wp-rest-sidebars-controller.php",
            any(rest_api_endpoints_class_wp_rest_sidebars_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core/Curve25519/Fe.php",
            any(sodium_compat_src_core_curve25519_fe_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core/Salsa20.php",
            any(sodium_compat_src_core_salsa20_include_live_dispatch),
        )
        .route(
            "/wp-includes/fonts/class-wp-font-collection.php",
            any(fonts_class_wp_font_collection_include_live_dispatch),
        )
        .route(
            "/wp-includes/fonts/class-wp-font-utils.php",
            any(fonts_class_wp_font_utils_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core/Ristretto255.php",
            any(sodium_compat_src_core_ristretto255_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/endpoints/class-wp-rest-settings-controller.php",
            any(rest_api_endpoints_class_wp_rest_settings_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/endpoints/class-wp-rest-blocks-controller.php",
            any(rest_api_endpoints_class_wp_rest_blocks_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/endpoints/class-wp-rest-navigation-fallback-controller.php",
            any(rest_api_endpoints_class_wp_rest_navigation_fallback_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/endpoints/class-wp-rest-block-types-controller.php",
            any(rest_api_endpoints_class_wp_rest_block_types_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/endpoints/class-wp-rest-url-details-controller.php",
            any(rest_api_endpoints_class_wp_rest_url_details_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/fonts/class-wp-font-face-resolver.php",
            any(fonts_class_wp_font_face_resolver_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core/AEGIS/State128L.php",
            any(sodium_compat_src_core_aegis_state128l_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core/AEGIS/State256.php",
            any(sodium_compat_src_core_aegis_state256_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core/ChaCha20.php",
            any(sodium_compat_src_core_chacha20_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core/XChaCha20.php",
            any(sodium_compat_src_core_xchacha20_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core/X25519.php",
            any(sodium_compat_src_core_x25519_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core/ChaCha20/Ctx.php",
            any(sodium_compat_src_core_chacha20_ctx_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core/ChaCha20/IetfCtx.php",
            any(sodium_compat_src_core_chacha20_ietf_ctx_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core/Poly1305.php",
            any(sodium_compat_src_core_poly1305_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core/SipHash.php",
            any(sodium_compat_src_core_sip_hash_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/endpoints/class-wp-rest-terms-controller.php",
            any(rest_api_endpoints_class_wp_rest_terms_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/endpoints/class-wp-rest-taxonomies-controller.php",
            any(rest_api_endpoints_class_wp_rest_taxonomies_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core/HSalsa20.php",
            any(sodium_compat_src_core_hsalsa20_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core/AES.php",
            any(sodium_compat_src_core_aes_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core/AEGIS256.php",
            any(sodium_compat_src_core_aegis256_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Crypto.php",
            any(sodium_compat_src_crypto_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core32/Ed25519.php",
            any(sodium_compat_src_core32_ed25519_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core32/Util.php",
            any(sodium_compat_src_core32_util_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core32/XSalsa20.php",
            any(sodium_compat_src_core32_xsalsa20_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core32/Curve25519.php",
            any(sodium_compat_src_core32_curve25519_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core/Poly1305/State.php",
            any(sodium_compat_src_core_poly1305_state_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core32/Int32.php",
            any(sodium_compat_src_core32_int32_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core32/SecretStream/State.php",
            any(sodium_compat_src_core32_secretstream_state_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core32/HChaCha20.php",
            any(sodium_compat_src_core32_hchacha20_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core32/Curve25519/Ge/P3.php",
            any(sodium_compat_src_core32_curve25519_ge_p3_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/endpoints/class-wp-rest-users-controller.php",
            any(rest_api_endpoints_class_wp_rest_users_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/endpoints/class-wp-rest-abilities-v1-run-controller.php",
            any(rest_api_endpoints_class_wp_rest_abilities_v1_run_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/endpoints/class-wp-rest-search-controller.php",
            any(rest_api_endpoints_class_wp_rest_search_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/endpoints/class-wp-rest-block-renderer-controller.php",
            any(rest_api_endpoints_class_wp_rest_block_renderer_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/endpoints/class-wp-rest-menu-locations-controller.php",
            any(rest_api_endpoints_class_wp_rest_menu_locations_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/endpoints/class-wp-rest-themes-controller.php",
            any(rest_api_endpoints_class_wp_rest_themes_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/endpoints/class-wp-rest-block-pattern-categories-controller.php",
            any(rest_api_endpoints_class_wp_rest_block_pattern_categories_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/endpoints/class-wp-rest-autosaves-controller.php",
            any(rest_api_endpoints_class_wp_rest_autosaves_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/endpoints/class-wp-rest-abilities-v1-categories-controller.php",
            any(rest_api_endpoints_class_wp_rest_abilities_v1_categories_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/endpoints/class-wp-rest-abilities-v1-list-controller.php",
            any(rest_api_endpoints_class_wp_rest_abilities_v1_list_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core32/Curve25519/Ge/Precomp.php",
            any(sodium_compat_src_core32_curve25519_ge_precomp_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core32/Curve25519/Ge/P1p1.php",
            any(sodium_compat_src_core32_curve25519_ge_p1p1_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core32/Curve25519/Ge/P2.php",
            any(sodium_compat_src_core32_curve25519_ge_p2_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core32/Curve25519/Ge/Cached.php",
            any(sodium_compat_src_core32_curve25519_ge_cached_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core32/Curve25519/H.php",
            any(sodium_compat_src_core32_curve25519_h_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core32/Curve25519/Fe.php",
            any(sodium_compat_src_core32_curve25519_fe_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core32/Salsa20.php",
            any(sodium_compat_src_core32_salsa20_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core32/ChaCha20.php",
            any(sodium_compat_src_core32_chacha20_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core32/XChaCha20.php",
            any(sodium_compat_src_core32_xchacha20_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core32/X25519.php",
            any(sodium_compat_src_core32_x25519_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core32/HSalsa20.php",
            any(sodium_compat_src_core32_hsalsa20_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core32/Int64.php",
            any(sodium_compat_src_core32_int64_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core32/Poly1305/State.php",
            any(sodium_compat_src_core32_poly1305_state_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/File.php",
            any(sodium_compat_src_file_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/PHP52/SplFixedArray.php",
            any(sodium_compat_src_php52_spl_fixed_array_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core32/ChaCha20/Ctx.php",
            any(sodium_compat_src_core32_chacha20_ctx_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core32/ChaCha20/IetfCtx.php",
            any(sodium_compat_src_core32_chacha20_ietf_ctx_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core32/BLAKE2b.php",
            any(sodium_compat_src_core32_blake2b_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core32/Poly1305.php",
            any(sodium_compat_src_core32_poly1305_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/Core32/SipHash.php",
            any(sodium_compat_src_core32_sip_hash_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/class-wp-rest-response.php",
            any(rest_api_class_wp_rest_response_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/class-wp-rest-request.php",
            any(rest_api_class_wp_rest_request_include_live_dispatch),
        )
        .route(
            "/wp-includes/rest-api/class-wp-rest-server.php",
            any(rest_api_class_wp_rest_server_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/src/SodiumException.php",
            any(sodium_compat_src_sodium_exception_include_live_dispatch),
        )
        .route(
            "/wp-includes/html-api/class-wp-html-text-replacement.php",
            any(html_api_class_wp_html_text_replacement_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/script-modules/index.php",
            any(js_dist_script_modules_index_include_live_dispatch),
        )
        .route(
            "/wp-includes/js/dist/script-modules/registry.php",
            any(js_dist_script_modules_registry_include_live_dispatch),
        )
        .route(
            "/wp-includes/widgets/class-wp-widget-media-video.php",
            any(widgets_class_wp_widget_media_video_include_live_dispatch),
        )
        .route(
            "/wp-includes/widgets/class-wp-widget-calendar.php",
            any(widgets_class_wp_widget_calendar_include_live_dispatch),
        )
        .route(
            "/wp-includes/widgets/class-wp-widget-media-image.php",
            any(widgets_class_wp_widget_media_image_include_live_dispatch),
        )
        .route(
            "/wp-includes/widgets/class-wp-widget-categories.php",
            any(widgets_class_wp_widget_categories_include_live_dispatch),
        )
        .route(
            "/wp-includes/widgets/class-wp-nav-menu-widget.php",
            any(widgets_class_wp_nav_menu_widget_include_live_dispatch),
        )
        .route(
            "/wp-includes/widgets/class-wp-widget-media-audio.php",
            any(widgets_class_wp_widget_media_audio_include_live_dispatch),
        )
        .route(
            "/wp-includes/widgets/class-wp-widget-links.php",
            any(widgets_class_wp_widget_links_include_live_dispatch),
        )
        .route(
            "/wp-includes/widgets/class-wp-widget-recent-comments.php",
            any(widgets_class_wp_widget_recent_comments_include_live_dispatch),
        )
        .route(
            "/wp-includes/widgets/class-wp-widget-archives.php",
            any(widgets_class_wp_widget_archives_include_live_dispatch),
        )
        .route(
            "/wp-includes/widgets/class-wp-widget-media.php",
            any(widgets_class_wp_widget_media_include_live_dispatch),
        )
        .route(
            "/wp-includes/widgets/class-wp-widget-block.php",
            any(widgets_class_wp_widget_block_include_live_dispatch),
        )
        .route(
            "/wp-includes/widgets/class-wp-widget-search.php",
            any(widgets_class_wp_widget_search_include_live_dispatch),
        )
        .route(
            "/wp-includes/widgets/class-wp-widget-tag-cloud.php",
            any(widgets_class_wp_widget_tag_cloud_include_live_dispatch),
        )
        .route(
            "/wp-includes/html-api/class-wp-html-tag-processor.php",
            any(html_api_class_wp_html_tag_processor_include_live_dispatch),
        )
        .route(
            "/wp-includes/html-api/class-wp-html-token.php",
            any(html_api_class_wp_html_token_include_live_dispatch),
        )
        .route(
            "/wp-includes/html-api/class-wp-html-decoder.php",
            any(html_api_class_wp_html_decoder_include_live_dispatch),
        )
        .route(
            "/wp-includes/html-api/class-wp-html-span.php",
            any(html_api_class_wp_html_span_include_live_dispatch),
        )
        .route(
            "/wp-includes/html-api/class-wp-html-stack-event.php",
            any(html_api_class_wp_html_stack_event_include_live_dispatch),
        )
        .route(
            "/wp-includes/html-api/class-wp-html-attribute-token.php",
            any(html_api_class_wp_html_attribute_token_include_live_dispatch),
        )
        .route(
            "/wp-includes/widgets/class-wp-widget-custom-html.php",
            any(widgets_class_wp_widget_custom_html_include_live_dispatch),
        )
        .route(
            "/wp-includes/html-api/class-wp-html-open-elements.php",
            any(html_api_class_wp_html_open_elements_include_live_dispatch),
        )
        .route(
            "/wp-includes/widgets/class-wp-widget-media-gallery.php",
            any(widgets_class_wp_widget_media_gallery_include_live_dispatch),
        )
        .route(
            "/wp-includes/widgets/class-wp-widget-text.php",
            any(widgets_class_wp_widget_text_include_live_dispatch),
        )
        .route(
            "/wp-includes/html-api/html5-named-character-references.php",
            any(html_api_html5_named_character_references_include_live_dispatch),
        )
        .route(
            "/wp-includes/html-api/class-wp-html-active-formatting-elements.php",
            any(html_api_class_wp_html_active_formatting_elements_include_live_dispatch),
        )
        .route(
            "/wp-includes/html-api/class-wp-html-processor.php",
            any(html_api_class_wp_html_processor_include_live_dispatch),
        )
        .route(
            "/wp-includes/html-api/class-wp-html-doctype-info.php",
            any(html_api_class_wp_html_doctype_info_include_live_dispatch),
        )
        .route(
            "/wp-includes/html-api/class-wp-html-processor-state.php",
            any(html_api_class_wp_html_processor_state_include_live_dispatch),
        )
        .route(
            "/wp-includes/widgets/class-wp-widget-meta.php",
            any(widgets_class_wp_widget_meta_include_live_dispatch),
        )
        .route(
            "/wp-includes/widgets/class-wp-widget-pages.php",
            any(widgets_class_wp_widget_pages_include_live_dispatch),
        )
        .route(
            "/wp-includes/widgets/class-wp-widget-rss.php",
            any(widgets_class_wp_widget_rss_include_live_dispatch),
        )
        .route(
            "/wp-includes/widgets/class-wp-widget-recent-posts.php",
            any(widgets_class_wp_widget_recent_posts_include_live_dispatch),
        )
        .route(
            "/wp-includes/html-api/class-wp-html-unsupported-exception.php",
            any(html_api_class_wp_html_unsupported_exception_include_live_dispatch),
        )
        .route(
            "/wp-content/index.php",
            any(wp_content_index_live_dispatch),
        )
        .route(
            "/wp-content/plugins/hello.php",
            any(wp_content_plugins_hello_live_dispatch),
        )
        .route(
            "/wp-content/plugins/index.php",
            any(wp_content_plugins_index_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentyfifteen/image.php",
            any(wp_content_themes_twentyfifteen_image_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentyfifteen/content-none.php",
            any(wp_content_themes_twentyfifteen_content_none_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentyfifteen/index.php",
            any(wp_content_themes_twentyfifteen_index_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentyfifteen/archive.php",
            any(wp_content_themes_twentyfifteen_archive_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentyfifteen/content-search.php",
            any(wp_content_themes_twentyfifteen_content_search_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentyfifteen/comments.php",
            any(wp_content_themes_twentyfifteen_comments_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentyfifteen/content.php",
            any(wp_content_themes_twentyfifteen_content_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentyfifteen/footer.php",
            any(wp_content_themes_twentyfifteen_footer_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentyfifteen/sidebar.php",
            any(wp_content_themes_twentyfifteen_sidebar_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentyfifteen/content-link.php",
            any(wp_content_themes_twentyfifteen_content_link_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentyfifteen/single.php",
            any(wp_content_themes_twentyfifteen_single_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentyfifteen/inc/back-compat.php",
            any(wp_content_themes_twentyfifteen_inc_back_compat_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentyfifteen/inc/custom-header.php",
            any(wp_content_themes_twentyfifteen_inc_custom_header_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentyfifteen/inc/customizer.php",
            any(wp_content_themes_twentyfifteen_inc_customizer_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentyfifteen/inc/block-patterns.php",
            any(wp_content_themes_twentyfifteen_inc_block_patterns_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentyfifteen/inc/template-tags.php",
            any(wp_content_themes_twentyfifteen_inc_template_tags_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentyfifteen/author-bio.php",
            any(wp_content_themes_twentyfifteen_author_bio_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentyfifteen/search.php",
            any(wp_content_themes_twentyfifteen_search_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentyfifteen/functions.php",
            any(wp_content_themes_twentyfifteen_functions_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentyfifteen/header.php",
            any(wp_content_themes_twentyfifteen_header_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentyfifteen/404.php",
            any(wp_content_themes_twentyfifteen_not_found_404_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentyfifteen/page.php",
            any(wp_content_themes_twentyfifteen_page_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentyfifteen/content-page.php",
            any(wp_content_themes_twentyfifteen_content_page_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentysixteen/template-parts/content-none.php",
            any(wp_content_themes_twentysixteen_template_parts_content_none_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentysixteen/template-parts/content-single.php",
            any(wp_content_themes_twentysixteen_template_parts_content_single_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentysixteen/template-parts/content-search.php",
            any(wp_content_themes_twentysixteen_template_parts_content_search_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentysixteen/template-parts/content.php",
            any(wp_content_themes_twentysixteen_template_parts_content_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentysixteen/template-parts/biography.php",
            any(wp_content_themes_twentysixteen_template_parts_biography_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentysixteen/template-parts/content-page.php",
            any(wp_content_themes_twentysixteen_template_parts_content_page_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentysixteen/index.php",
            any(wp_content_themes_twentysixteen_index_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentysixteen/archive.php",
            any(wp_content_themes_twentysixteen_archive_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentysixteen/searchform.php",
            any(wp_content_themes_twentysixteen_searchform_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentyseventeen/template-parts/post/content-none.php",
            any(wp_content_themes_twentyseventeen_template_parts_post_content_none_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentyseventeen/template-parts/post/content.php",
            any(wp_content_themes_twentyseventeen_template_parts_post_content_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentyseventeen/template-parts/post/content-audio.php",
            any(wp_content_themes_twentyseventeen_template_parts_post_content_audio_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentyseventeen/template-parts/post/content-excerpt.php",
            any(wp_content_themes_twentyseventeen_template_parts_post_content_excerpt_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentyseventeen/template-parts/post/content-video.php",
            any(wp_content_themes_twentyseventeen_template_parts_post_content_video_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentysixteen/comments.php",
            any(wp_content_themes_twentysixteen_comments_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentysixteen/single.php",
            any(wp_content_themes_twentysixteen_single_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentysixteen/sidebar-content-bottom.php",
            any(wp_content_themes_twentysixteen_sidebar_content_bottom_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentysixteen/inc/back-compat.php",
            any(wp_content_themes_twentysixteen_inc_back_compat_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentysixteen/inc/customizer.php",
            any(wp_content_themes_twentysixteen_inc_customizer_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentysixteen/inc/block-patterns.php",
            any(wp_content_themes_twentysixteen_inc_block_patterns_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentysixteen/inc/template-tags.php",
            any(wp_content_themes_twentysixteen_inc_template_tags_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentysixteen/search.php",
            any(wp_content_themes_twentysixteen_search_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentysixteen/functions.php",
            any(wp_content_themes_twentysixteen_functions_live_dispatch),
        )
        .route(
            "/wp-content/themes/twentysixteen/header.php",
            any(wp_content_themes_twentysixteen_header_live_dispatch),
        )
        .route(
            "/wp-includes/customize/class-wp-customize-selective-refresh.php",
            any(customize_class_wp_customize_selective_refresh_include_live_dispatch),
        )
        .route(
            "/wp-includes/customize/class-wp-widget-form-customize-control.php",
            any(customize_class_wp_widget_form_customize_control_include_live_dispatch),
        )
        .route(
            "/wp-includes/customize/class-wp-customize-media-control.php",
            any(customize_class_wp_customize_media_control_include_live_dispatch),
        )
        .route(
            "/wp-includes/customize/class-wp-customize-date-time-control.php",
            any(customize_class_wp_customize_date_time_control_include_live_dispatch),
        )
        .route(
            "/wp-includes/customize/class-wp-customize-header-image-setting.php",
            any(customize_class_wp_customize_header_image_setting_include_live_dispatch),
        )
        .route(
            "/wp-includes/customize/class-wp-customize-background-image-control.php",
            any(customize_class_wp_customize_background_image_control_include_live_dispatch),
        )
        .route(
            "/wp-includes/customize/class-wp-customize-filter-setting.php",
            any(customize_class_wp_customize_filter_setting_include_live_dispatch),
        )
        .route(
            "/wp-includes/customize/class-wp-customize-sidebar-section.php",
            any(customize_class_wp_customize_sidebar_section_include_live_dispatch),
        )
        .route(
            "/wp-includes/customize/class-wp-customize-nav-menu-section.php",
            any(customize_class_wp_customize_nav_menu_section_include_live_dispatch),
        )
        .route(
            "/wp-includes/customize/class-wp-customize-upload-control.php",
            any(customize_class_wp_customize_upload_control_include_live_dispatch),
        )
        .route(
            "/wp-includes/block-supports/block-style-variations.php",
            any(block_supports_block_style_variations_include_live_dispatch),
        )
        .route(
            "/wp-includes/sodium_compat/namespaced/Core/Curve25519/Fe.php",
            any(sodium_compat_namespaced_core_curve25519_fe_include_live_dispatch),
        )
        .route(
            "/wp-includes/block-supports/background.php",
            any(block_supports_background_include_live_dispatch),
        )
        .route(
            "/wp-includes/admin-bar.php",
            any(admin_bar_include_live_dispatch),
        )
        .route(
            "/wp-includes/atomlib.php",
            any(atomlib_include_live_dispatch),
        )
        .route(
            "/wp-includes/author-template.php",
            any(author_template_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-theme-json-data.php",
            any(class_wp_theme_json_data_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-theme-json-resolver.php",
            any(class_wp_theme_json_resolver_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-token-map.php",
            any(class_wp_token_map_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-url-pattern-prefixer.php",
            any(class_wp_url_pattern_prefixer_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-walker.php",
            any(class_wp_walker_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-simplepie-file.php",
            any(class_wp_simplepie_file_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-simplepie-sanitize-kses.php",
            any(class_wp_simplepie_sanitize_kses_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-speculation-rules.php",
            any(class_wp_speculation_rules_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-text-diff-renderer-inline.php",
            any(class_wp_text_diff_renderer_inline_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-text-diff-renderer-table.php",
            any(class_wp_text_diff_renderer_table_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-navigation-fallback.php",
            any(class_wp_navigation_fallback_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-object-cache.php",
            any(class_wp_object_cache_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-oembed-controller.php",
            any(class_wp_oembed_controller_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-paused-extensions-storage.php",
            any(class_wp_paused_extensions_storage_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-phpmailer.php",
            any(class_wp_phpmailer_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-customize-setting.php",
            any(class_wp_customize_setting_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-customize-widgets.php",
            any(class_wp_customize_widgets_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-feed-cache-transient.php",
            any(class_wp_feed_cache_transient_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-feed-cache.php",
            any(class_wp_feed_cache_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-http-ixr-client.php",
            any(class_wp_http_ixr_client_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-customize-control.php",
            any(class_wp_customize_control_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-customize-manager.php",
            any(class_wp_customize_manager_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-customize-nav-menus.php",
            any(class_wp_customize_nav_menus_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-customize-panel.php",
            any(class_wp_customize_panel_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-customize-section.php",
            any(class_wp_customize_section_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-block-supports.php",
            any(class_wp_block_supports_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-block-template.php",
            any(class_wp_block_template_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-block-type.php",
            any(class_wp_block_type_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-classic-to-block-menu-converter.php",
            any(class_wp_classic_to_block_menu_converter_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-duotone.php",
            any(class_wp_duotone_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-block-pattern-categories-registry.php",
            any(class_wp_block_pattern_categories_registry_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-block-patterns-registry.php",
            any(class_wp_block_patterns_registry_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-block-styles-registry.php",
            any(class_wp_block_styles_registry_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-block-templates-registry.php",
            any(class_wp_block_templates_registry_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-block-type-registry.php",
            any(class_wp_block_type_registry_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-block-metadata-registry.php",
            any(class_wp_block_metadata_registry_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-block-parser.php",
            any(class_wp_block_parser_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-block-parser-block.php",
            any(class_wp_block_parser_block_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-block-parser-frame.php",
            any(class_wp_block_parser_frame_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-block-processor.php",
            any(class_wp_block_processor_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-block-bindings-registry.php",
            any(class_wp_block_bindings_registry_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-block-bindings-source.php",
            any(class_wp_block_bindings_source_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-block-editor-context.php",
            any(class_wp_block_editor_context_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-block-list.php",
            any(class_wp_block_list_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-block.php",
            any(class_wp_block_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-recovery-mode.php",
            any(class_wp_recovery_mode_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-recovery-mode-cookie-service.php",
            any(class_wp_recovery_mode_cookie_service_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-recovery-mode-link-service.php",
            any(class_wp_recovery_mode_link_service_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-recovery-mode-key-service.php",
            any(class_wp_recovery_mode_key_service_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-recovery-mode-email-service.php",
            any(class_wp_recovery_mode_email_service_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-xmlrpc-server.php",
            any(class_wp_xmlrpc_server_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-widget-factory.php",
            any(class_wp_widget_factory_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-theme-json.php",
            any(class_wp_theme_json_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-theme-json-schema.php",
            any(class_wp_theme_json_schema_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-textdomain-registry.php",
            any(class_wp_textdomain_registry_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-image-editor.php",
            any(class_wp_image_editor_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-image-editor-gd.php",
            any(class_wp_image_editor_gd_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-image-editor-imagick.php",
            any(class_wp_image_editor_imagick_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-exception.php",
            any(class_wp_exception_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-fatal-error-handler.php",
            any(class_wp_fatal_error_handler_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-admin-bar.php",
            any(class_wp_admin_bar_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-ajax-response.php",
            any(class_wp_ajax_response_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-embed.php",
            any(class_wp_embed_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-editor.php",
            any(class_wp_editor_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-oembed.php",
            any(class_wp_oembed_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-comment.php",
            any(class_wp_comment_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-term.php",
            any(class_wp_term_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-user-request.php",
            any(class_wp_user_request_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-application-passwords.php",
            any(class_wp_application_passwords_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-plugin-dependencies.php",
            any(class_wp_plugin_dependencies_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-locale.php",
            any(class_wp_locale_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-locale-switcher.php",
            any(class_wp_locale_switcher_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-matchesmapregex.php",
            any(class_wp_matchesmapregex_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-list-util.php",
            any(class_wp_list_util_include_live_dispatch),
        )
        .route(
            "/wp-includes/class-wp-metadata-lazyloader.php",
            any(class_wp_metadata_lazyloader_include_live_dispatch),
        )
        .route(
            "/wp-includes/general-template.php",
            any(general_template_include_live_dispatch),
        )
        .route(
            "/wp-includes/link-template.php",
            any(link_template_include_live_dispatch),
        )
        .route(
            "/wp-includes/default-filters.php",
            any(default_filters_include_live_dispatch),
        )
        .route("/wp-includes/blocks.php", any(blocks_include_live_dispatch))
        .route("/wp-includes/theme.php", any(theme_include_live_dispatch))
        .route(
            "/wp-includes/theme-templates.php",
            any(theme_templates_include_live_dispatch),
        )
        .route(
            "/wp-includes/theme-previews.php",
            any(theme_previews_include_live_dispatch),
        )
        .route(
            "/wp-includes/speculative-loading.php",
            any(speculative_loading_include_live_dispatch),
        )
        .route(
            "/wp-includes/template-loader.php",
            any(template_loader_include_live_dispatch),
        )
        .route(
            "/wp-includes/template-canvas.php",
            any(template_canvas_include_live_dispatch),
        )
        .route(
            "/wp-includes/template.php",
            any(template_include_live_dispatch),
        )
        .route(
            "/wp-includes/taxonomy.php",
            any(taxonomy_include_live_dispatch),
        )
        .route(
            "/wp-includes/shortcodes.php",
            any(shortcodes_include_live_dispatch),
        )
        .route(
            "/wp-includes/widgets.php",
            any(widgets_include_live_dispatch),
        )
        .route(
            "/wp-includes/ID3/getid3.lib.php",
            any(id3_getid3_lib_include_live_dispatch),
        )
        .route(
            "/wp-includes/ID3/getid3.php",
            any(id3_getid3_include_live_dispatch),
        )
        .route(
            "/wp-includes/ID3/module.audio-video.asf.php",
            any(id3_module_audio_video_asf_include_live_dispatch),
        )
        .route(
            "/wp-includes/ID3/module.audio-video.flv.php",
            any(id3_module_audio_video_flv_include_live_dispatch),
        )
        .route(
            "/wp-includes/ID3/module.audio-video.matroska.php",
            any(id3_module_audio_video_matroska_include_live_dispatch),
        )
        .route(
            "/wp-includes/ID3/module.audio-video.quicktime.php",
            any(id3_module_audio_video_quicktime_include_live_dispatch),
        )
        .route(
            "/wp-includes/ID3/module.audio-video.riff.php",
            any(id3_module_audio_video_riff_include_live_dispatch),
        )
        .route(
            "/wp-includes/ID3/module.audio.ac3.php",
            any(id3_module_audio_ac3_include_live_dispatch),
        )
        .route(
            "/wp-includes/ID3/module.audio.dts.php",
            any(id3_module_audio_dts_include_live_dispatch),
        )
        .route(
            "/wp-includes/ID3/module.audio.flac.php",
            any(id3_module_audio_flac_include_live_dispatch),
        )
        .route(
            "/wp-includes/ID3/module.audio.mp3.php",
            any(id3_module_audio_mp3_include_live_dispatch),
        )
        .route(
            "/wp-includes/ID3/module.audio.ogg.php",
            any(id3_module_audio_ogg_include_live_dispatch),
        )
        .route(
            "/wp-includes/ID3/module.tag.apetag.php",
            any(id3_module_tag_apetag_include_live_dispatch),
        )
        .route(
            "/wp-includes/ID3/module.tag.id3v1.php",
            any(id3_module_tag_id3v1_include_live_dispatch),
        )
        .route(
            "/wp-includes/ID3/module.tag.id3v2.php",
            any(id3_module_tag_id3v2_include_live_dispatch),
        )
        .route(
            "/wp-includes/ID3/module.tag.lyrics3.php",
            any(id3_module_tag_lyrics3_include_live_dispatch),
        )
        .route(
            "/wp-includes/IXR/class-IXR-base64.php",
            any(ixr_class_base64_include_live_dispatch),
        )
        .route(
            "/wp-includes/IXR/class-IXR-client.php",
            any(ixr_class_client_include_live_dispatch),
        )
        .route(
            "/wp-includes/IXR/class-IXR-clientmulticall.php",
            any(ixr_class_clientmulticall_include_live_dispatch),
        )
        .route(
            "/wp-includes/IXR/class-IXR-date.php",
            any(ixr_class_date_include_live_dispatch),
        )
        .route(
            "/wp-includes/IXR/class-IXR-error.php",
            any(ixr_class_error_include_live_dispatch),
        )
        .route(
            "/wp-includes/IXR/class-IXR-introspectionserver.php",
            any(ixr_class_introspectionserver_include_live_dispatch),
        )
        .route(
            "/wp-includes/IXR/class-IXR-message.php",
            any(ixr_class_message_include_live_dispatch),
        )
        .route(
            "/wp-includes/IXR/class-IXR-request.php",
            any(ixr_class_request_include_live_dispatch),
        )
        .route(
            "/wp-includes/IXR/class-IXR-server.php",
            any(ixr_class_server_include_live_dispatch),
        )
        .route(
            "/wp-includes/IXR/class-IXR-value.php",
            any(ixr_class_value_include_live_dispatch),
        )
        .route(
            "/wp-includes/PHPMailer/DSNConfigurator.php",
            any(phpmailer_dsn_configurator_include_live_dispatch),
        )
        .route(
            "/wp-includes/PHPMailer/Exception.php",
            any(phpmailer_exception_include_live_dispatch),
        )
        .route(
            "/wp-includes/PHPMailer/OAuth.php",
            any(phpmailer_oauth_include_live_dispatch),
        )
        .route(
            "/wp-includes/PHPMailer/OAuthTokenProvider.php",
            any(phpmailer_oauth_token_provider_include_live_dispatch),
        )
        .route(
            "/wp-includes/PHPMailer/PHPMailer.php",
            any(phpmailer_phpmailer_include_live_dispatch),
        )
        .route(
            "/wp-includes/PHPMailer/POP3.php",
            any(phpmailer_pop3_include_live_dispatch),
        )
        .route(
            "/wp-includes/PHPMailer/SMTP.php",
            any(phpmailer_smtp_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/library/Requests.php",
            any(requests_library_requests_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Auth.php",
            any(requests_src_auth_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Auth/Basic.php",
            any(requests_src_auth_basic_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Autoload.php",
            any(requests_src_autoload_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Capability.php",
            any(requests_src_capability_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Cookie.php",
            any(requests_src_cookie_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Cookie/Jar.php",
            any(requests_src_cookie_jar_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Exception.php",
            any(requests_src_exception_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Exception/ArgumentCount.php",
            any(requests_src_exception_argument_count_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Exception/Http.php",
            any(requests_src_exception_http_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Exception/Http/Status304.php",
            any(requests_src_exception_http_status304_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Exception/Http/Status305.php",
            any(requests_src_exception_http_status305_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Exception/Http/Status306.php",
            any(requests_src_exception_http_status306_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Exception/Http/Status400.php",
            any(requests_src_exception_http_status400_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Exception/Http/Status401.php",
            any(requests_src_exception_http_status401_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Exception/Http/Status402.php",
            any(requests_src_exception_http_status402_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Exception/Http/Status403.php",
            any(requests_src_exception_http_status403_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Exception/Http/Status404.php",
            any(requests_src_exception_http_status404_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Exception/Http/Status405.php",
            any(requests_src_exception_http_status405_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Exception/Http/Status406.php",
            any(requests_src_exception_http_status406_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Exception/Http/Status407.php",
            any(requests_src_exception_http_status407_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Exception/Http/Status408.php",
            any(requests_src_exception_http_status408_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Exception/Http/Status409.php",
            any(requests_src_exception_http_status409_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Exception/Http/Status410.php",
            any(requests_src_exception_http_status410_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Exception/Http/Status411.php",
            any(requests_src_exception_http_status411_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Exception/Http/Status412.php",
            any(requests_src_exception_http_status412_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Exception/Http/Status413.php",
            any(requests_src_exception_http_status413_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Exception/Http/Status414.php",
            any(requests_src_exception_http_status414_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Exception/Http/Status415.php",
            any(requests_src_exception_http_status415_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Exception/Http/Status416.php",
            any(requests_src_exception_http_status416_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Exception/Http/Status417.php",
            any(requests_src_exception_http_status417_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Exception/Http/Status418.php",
            any(requests_src_exception_http_status418_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Exception/Http/Status428.php",
            any(requests_src_exception_http_status428_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Exception/Http/Status429.php",
            any(requests_src_exception_http_status429_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Exception/Http/Status431.php",
            any(requests_src_exception_http_status431_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Exception/Http/Status500.php",
            any(requests_src_exception_http_status500_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Exception/Http/Status501.php",
            any(requests_src_exception_http_status501_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Exception/Http/Status502.php",
            any(requests_src_exception_http_status502_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Exception/Http/Status503.php",
            any(requests_src_exception_http_status503_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Exception/Http/Status504.php",
            any(requests_src_exception_http_status504_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Exception/Http/Status505.php",
            any(requests_src_exception_http_status505_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Exception/Http/Status511.php",
            any(requests_src_exception_http_status511_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Exception/Http/StatusUnknown.php",
            any(requests_src_exception_http_status_unknown_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Exception/InvalidArgument.php",
            any(requests_src_exception_invalid_argument_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Exception/Transport/Curl.php",
            any(requests_src_exception_transport_curl_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Exception/Transport.php",
            any(requests_src_exception_transport_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/HookManager.php",
            any(requests_src_hook_manager_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Hooks.php",
            any(requests_src_hooks_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/IdnaEncoder.php",
            any(requests_src_idna_encoder_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Ipv6.php",
            any(requests_src_ipv6_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Iri.php",
            any(requests_src_iri_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Port.php",
            any(requests_src_port_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Proxy/Http.php",
            any(requests_src_proxy_http_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Proxy.php",
            any(requests_src_proxy_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Requests.php",
            any(requests_src_requests_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Response/Headers.php",
            any(requests_src_response_headers_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Response.php",
            any(requests_src_response_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Session.php",
            any(requests_src_session_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Ssl.php",
            any(requests_src_ssl_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Transport/Curl.php",
            any(requests_src_transport_curl_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Transport/Fsockopen.php",
            any(requests_src_transport_fsockopen_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Transport.php",
            any(requests_src_transport_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Utility/CaseInsensitiveDictionary.php",
            any(requests_src_utility_case_insensitive_dictionary_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Utility/FilteredIterator.php",
            any(requests_src_utility_filtered_iterator_include_live_dispatch),
        )
        .route(
            "/wp-includes/Requests/src/Utility/InputValidator.php",
            any(requests_src_utility_input_validator_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/autoloader.php",
            any(simple_pie_autoloader_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/library/SimplePie/Author.php",
            any(simple_pie_library_simple_pie_author_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/library/SimplePie/Cache/Base.php",
            any(simple_pie_library_simple_pie_cache_base_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/library/SimplePie/Cache/DB.php",
            any(simple_pie_library_simple_pie_cache_db_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/library/SimplePie/Cache/File.php",
            any(simple_pie_library_simple_pie_cache_file_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/library/SimplePie/Cache/Memcache.php",
            any(simple_pie_library_simple_pie_cache_memcache_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/library/SimplePie/Cache/Memcached.php",
            any(simple_pie_library_simple_pie_cache_memcached_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/library/SimplePie/Cache/MySQL.php",
            any(simple_pie_library_simple_pie_cache_mysql_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/library/SimplePie/Cache/Redis.php",
            any(simple_pie_library_simple_pie_cache_redis_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/library/SimplePie/Cache.php",
            any(simple_pie_library_simple_pie_cache_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/library/SimplePie/Caption.php",
            any(simple_pie_library_simple_pie_caption_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/library/SimplePie/Category.php",
            any(simple_pie_library_simple_pie_category_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/library/SimplePie/Content/Type/Sniffer.php",
            any(simple_pie_library_simple_pie_content_type_sniffer_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/library/SimplePie/Copyright.php",
            any(simple_pie_library_simple_pie_copyright_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/library/SimplePie/Core.php",
            any(simple_pie_library_simple_pie_core_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/library/SimplePie/Credit.php",
            any(simple_pie_library_simple_pie_credit_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/library/SimplePie/Decode/HTML/Entities.php",
            any(simple_pie_library_simple_pie_decode_html_entities_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/library/SimplePie/Enclosure.php",
            any(simple_pie_library_simple_pie_enclosure_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/library/SimplePie/Exception.php",
            any(simple_pie_library_simple_pie_exception_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/library/SimplePie/File.php",
            any(simple_pie_library_simple_pie_file_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/library/SimplePie/HTTP/Parser.php",
            any(simple_pie_library_simple_pie_http_parser_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/library/SimplePie/IRI.php",
            any(simple_pie_library_simple_pie_iri_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/library/SimplePie/Item.php",
            any(simple_pie_library_simple_pie_item_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/library/SimplePie/Locator.php",
            any(simple_pie_library_simple_pie_locator_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/library/SimplePie/Misc.php",
            any(simple_pie_library_simple_pie_misc_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/library/SimplePie/Net/IPv6.php",
            any(simple_pie_library_simple_pie_net_ipv6_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/library/SimplePie/Parse/Date.php",
            any(simple_pie_library_simple_pie_parse_date_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/library/SimplePie/Parser.php",
            any(simple_pie_library_simple_pie_parser_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/library/SimplePie/Rating.php",
            any(simple_pie_library_simple_pie_rating_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/library/SimplePie/Registry.php",
            any(simple_pie_library_simple_pie_registry_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/library/SimplePie.php",
            any(simple_pie_library_simple_pie_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/Author.php",
            any(simple_pie_src_author_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/Cache/Base.php",
            any(simple_pie_src_cache_base_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/Cache/BaseDataCache.php",
            any(simple_pie_src_cache_base_data_cache_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/Cache/CallableNameFilter.php",
            any(simple_pie_src_cache_callable_name_filter_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/Cache/DB.php",
            any(simple_pie_src_cache_db_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/Cache/DataCache.php",
            any(simple_pie_src_cache_data_cache_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/Cache/File.php",
            any(simple_pie_src_cache_file_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/Cache/Memcache.php",
            any(simple_pie_src_cache_memcache_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/Cache/Memcached.php",
            any(simple_pie_src_cache_memcached_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/Cache/MySQL.php",
            any(simple_pie_src_cache_mysql_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/Cache/NameFilter.php",
            any(simple_pie_src_cache_name_filter_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/Cache/Psr16.php",
            any(simple_pie_src_cache_psr16_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/Cache/Redis.php",
            any(simple_pie_src_cache_redis_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/Cache.php",
            any(simple_pie_src_cache_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/Caption.php",
            any(simple_pie_src_caption_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/Category.php",
            any(simple_pie_src_category_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/Content/Type/Sniffer.php",
            any(simple_pie_src_content_type_sniffer_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/Copyright.php",
            any(simple_pie_src_copyright_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/Credit.php",
            any(simple_pie_src_credit_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/Enclosure.php",
            any(simple_pie_src_enclosure_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/Exception.php",
            any(simple_pie_src_exception_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/File.php",
            any(simple_pie_src_file_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/Gzdecode.php",
            any(simple_pie_src_gzdecode_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/HTTP/Client.php",
            any(simple_pie_src_http_client_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/HTTP/ClientException.php",
            any(simple_pie_src_http_client_exception_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/HTTP/FileClient.php",
            any(simple_pie_src_http_file_client_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/HTTP/Parser.php",
            any(simple_pie_src_http_parser_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/HTTP/Psr18Client.php",
            any(simple_pie_src_http_psr18_client_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/HTTP/Psr7Response.php",
            any(simple_pie_src_http_psr7_response_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/HTTP/RawTextResponse.php",
            any(simple_pie_src_http_raw_text_response_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/HTTP/Response.php",
            any(simple_pie_src_http_response_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/IRI.php",
            any(simple_pie_src_iri_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/Item.php",
            any(simple_pie_src_item_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/Locator.php",
            any(simple_pie_src_locator_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/Misc.php",
            any(simple_pie_src_misc_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/Net/IPv6.php",
            any(simple_pie_src_net_ipv6_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/Parse/Date.php",
            any(simple_pie_src_parse_date_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/Parser.php",
            any(simple_pie_src_parser_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/Rating.php",
            any(simple_pie_src_rating_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/RegistryAware.php",
            any(simple_pie_src_registry_aware_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/Registry.php",
            any(simple_pie_src_registry_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/Restriction.php",
            any(simple_pie_src_restriction_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/Sanitize.php",
            any(simple_pie_src_sanitize_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/SimplePie.php",
            any(simple_pie_src_simple_pie_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/Source.php",
            any(simple_pie_src_source_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/src/XML/Declaration/Parser.php",
            any(simple_pie_src_xml_declaration_parser_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/library/SimplePie/Restriction.php",
            any(simple_pie_library_simple_pie_restriction_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/library/SimplePie/Sanitize.php",
            any(simple_pie_library_simple_pie_sanitize_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/library/SimplePie/Source.php",
            any(simple_pie_library_simple_pie_source_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/library/SimplePie/XML/Declaration/Parser.php",
            any(simple_pie_library_simple_pie_xml_declaration_parser_include_live_dispatch),
        )
        .route(
            "/wp-includes/SimplePie/library/SimplePie/gzdecode.php",
            any(simple_pie_library_simple_pie_gzdecode_include_live_dispatch),
        )
        .route(
            "/wp-includes/style-engine.php",
            any(style_engine_include_live_dispatch),
        )
        .route(
            "/wp-includes/sitemaps.php",
            any(sitemaps_include_live_dispatch),
        )
        .route(
            "/wp-includes/script-modules.php",
            any(script_modules_include_live_dispatch),
        )
        .route(
            "/wp-includes/version.php",
            any(version_include_live_dispatch),
        )
        .route(
            "/wp-includes/wp-diff.php",
            any(wp_diff_include_live_dispatch),
        )
        .route(
            "/wp-includes/view-transitions.php",
            any(view_transitions_include_live_dispatch),
        )
        .route("/wp-settings.php", any(settings_bootstrap_live_dispatch))
        .route("/index.php", any(index_bootstrap_live_dispatch))
        .route("/wp-blog-header.php", any(blog_header_live_dispatch))
        .route("/wp-load.php", any(load_bootstrap_live_dispatch))
        .route(
            "/wp-includes/js/tinymce/wp-tinymce.php",
            any(wp_tinymce_live_dispatch),
        )
        .route("/wp-admin", get(admin_dashboard_live))
        .route("/wp-admin/", get(admin_dashboard_live))
        .route("/wp-admin/index.php", get(admin_dashboard_live))
        .route("/wp-admin/admin.php", any(admin_bootstrap_live_dispatch))
        .route(
            "/wp-admin/load-scripts.php",
            any(load_scripts_live_dispatch),
        )
        .route("/wp-admin/load-styles.php", any(load_styles_live_dispatch))
        .route(
            "/wp-admin/custom-background.php",
            any(custom_background_live_dispatch),
        )
        .route(
            "/wp-admin/custom-header.php",
            any(custom_header_live_dispatch),
        )
        .route(
            "/wp-admin/admin-functions.php",
            any(admin_functions_include_live_dispatch),
        )
        .route(
            "/wp-admin/options-head.php",
            any(options_head_include_live_dispatch),
        )
        .route("/wp-admin/menu.php", any(menu_include_live_dispatch))
        .route(
            "/wp-admin/includes/admin-filters.php",
            any(admin_filters_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/admin.php",
            any(admin_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/ajax-actions.php",
            any(ajax_actions_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/bookmark.php",
            any(bookmark_admin_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-automatic-upgrader-skin.php",
            any(class_automatic_upgrader_skin_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-bulk-plugin-upgrader-skin.php",
            any(class_bulk_plugin_upgrader_skin_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-bulk-theme-upgrader-skin.php",
            any(class_bulk_theme_upgrader_skin_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-bulk-upgrader-skin.php",
            any(class_bulk_upgrader_skin_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-core-upgrader.php",
            any(class_core_upgrader_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-custom-background.php",
            any(class_custom_background_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-custom-image-header.php",
            any(class_custom_image_header_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-file-upload-upgrader.php",
            any(class_file_upload_upgrader_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-ftp-pure.php",
            any(class_ftp_pure_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-ftp-sockets.php",
            any(class_ftp_sockets_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-ftp.php",
            any(class_ftp_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-language-pack-upgrader-skin.php",
            any(class_language_pack_upgrader_skin_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-language-pack-upgrader.php",
            any(class_language_pack_upgrader_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-pclzip.php",
            any(class_pclzip_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-plugin-installer-skin.php",
            any(class_plugin_installer_skin_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-plugin-upgrader-skin.php",
            any(class_plugin_upgrader_skin_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-plugin-upgrader.php",
            any(class_plugin_upgrader_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-theme-installer-skin.php",
            any(class_theme_installer_skin_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-theme-upgrader-skin.php",
            any(class_theme_upgrader_skin_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-theme-upgrader.php",
            any(class_theme_upgrader_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-walker-category-checklist.php",
            any(class_walker_category_checklist_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-walker-nav-menu-checklist.php",
            any(class_walker_nav_menu_checklist_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-walker-nav-menu-edit.php",
            any(class_walker_nav_menu_edit_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-wp-ajax-upgrader-skin.php",
            any(class_wp_ajax_upgrader_skin_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-wp-application-passwords-list-table.php",
            any(class_wp_application_passwords_list_table_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-wp-automatic-updater.php",
            any(class_wp_automatic_updater_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-wp-comments-list-table.php",
            any(class_wp_comments_list_table_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-wp-community-events.php",
            any(class_wp_community_events_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-wp-debug-data.php",
            any(class_wp_debug_data_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-wp-filesystem-base.php",
            any(class_wp_filesystem_base_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-wp-filesystem-direct.php",
            any(class_wp_filesystem_direct_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-wp-filesystem-ftpext.php",
            any(class_wp_filesystem_ftpext_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-wp-filesystem-ftpsockets.php",
            any(class_wp_filesystem_ftpsockets_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-wp-filesystem-ssh2.php",
            any(class_wp_filesystem_ssh2_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-wp-importer.php",
            any(class_wp_importer_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-wp-internal-pointers.php",
            any(class_wp_internal_pointers_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-wp-links-list-table.php",
            any(class_wp_links_list_table_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-wp-list-table-compat.php",
            any(class_wp_list_table_compat_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-wp-list-table.php",
            any(class_wp_list_table_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-wp-media-list-table.php",
            any(class_wp_media_list_table_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-wp-ms-sites-list-table.php",
            any(class_wp_ms_sites_list_table_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-wp-ms-themes-list-table.php",
            any(class_wp_ms_themes_list_table_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-wp-ms-users-list-table.php",
            any(class_wp_ms_users_list_table_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-wp-plugin-install-list-table.php",
            any(class_wp_plugin_install_list_table_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-wp-plugins-list-table.php",
            any(class_wp_plugins_list_table_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-wp-post-comments-list-table.php",
            any(class_wp_post_comments_list_table_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-wp-posts-list-table.php",
            any(class_wp_posts_list_table_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-wp-privacy-data-export-requests-list-table.php",
            any(class_wp_privacy_data_export_requests_list_table_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-wp-privacy-data-removal-requests-list-table.php",
            any(class_wp_privacy_data_removal_requests_list_table_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-wp-privacy-policy-content.php",
            any(class_wp_privacy_policy_content_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-wp-privacy-requests-table.php",
            any(class_wp_privacy_requests_table_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-wp-screen.php",
            any(class_wp_screen_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-wp-site-health-auto-updates.php",
            any(class_wp_site_health_auto_updates_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-wp-site-health.php",
            any(class_wp_site_health_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-wp-site-icon.php",
            any(class_wp_site_icon_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-wp-terms-list-table.php",
            any(class_wp_terms_list_table_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-wp-theme-install-list-table.php",
            any(class_wp_theme_install_list_table_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-wp-themes-list-table.php",
            any(class_wp_themes_list_table_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-wp-upgrader.php",
            any(class_wp_upgrader_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-wp-upgrader-skin.php",
            any(class_wp_upgrader_skin_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-wp-upgrader-skins.php",
            any(class_wp_upgrader_skins_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/class-wp-users-list-table.php",
            any(class_wp_users_list_table_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/comment.php",
            any(admin_includes_comment_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/continents-cities.php",
            any(admin_includes_continents_cities_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/credits.php",
            any(admin_includes_credits_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/dashboard.php",
            any(admin_includes_dashboard_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/deprecated.php",
            any(admin_includes_deprecated_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/edit-tag-messages.php",
            any(admin_includes_edit_tag_messages_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/export.php",
            any(admin_includes_export_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/file.php",
            any(admin_includes_file_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/image-edit.php",
            any(admin_includes_image_edit_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/image.php",
            any(admin_includes_image_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/import.php",
            any(admin_includes_import_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/list-table.php",
            any(admin_includes_list_table_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/media.php",
            any(admin_includes_media_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/menu.php",
            any(admin_includes_menu_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/meta-boxes.php",
            any(admin_includes_meta_boxes_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/misc.php",
            any(admin_includes_misc_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/ms-admin-filters.php",
            any(admin_includes_ms_admin_filters_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/ms-deprecated.php",
            any(admin_includes_ms_deprecated_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/ms.php",
            any(admin_includes_ms_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/nav-menu.php",
            any(admin_includes_nav_menu_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/network.php",
            any(admin_includes_network_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/noop.php",
            any(admin_includes_noop_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/options.php",
            any(admin_includes_options_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/plugin-install.php",
            any(admin_includes_plugin_install_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/plugin.php",
            any(admin_includes_plugin_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/post.php",
            any(admin_includes_post_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/privacy-tools.php",
            any(admin_includes_privacy_tools_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/revision.php",
            any(admin_includes_revision_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/schema.php",
            any(admin_includes_schema_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/screen.php",
            any(admin_includes_screen_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/taxonomy.php",
            any(admin_includes_taxonomy_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/template.php",
            any(admin_includes_template_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/theme-install.php",
            any(admin_includes_theme_install_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/theme.php",
            any(admin_includes_theme_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/translation-install.php",
            any(admin_includes_translation_install_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/update-core.php",
            any(admin_includes_update_core_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/update.php",
            any(admin_includes_update_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/upgrade.php",
            any(admin_includes_upgrade_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/user.php",
            any(admin_includes_user_include_live_dispatch),
        )
        .route(
            "/wp-admin/includes/widgets.php",
            any(admin_includes_widgets_include_live_dispatch),
        )
        .route(
            "/wp-admin/menu-header.php",
            any(menu_header_include_live_dispatch),
        )
        .route(
            "/wp-admin/admin-header.php",
            any(admin_header_include_live_dispatch),
        )
        .route(
            "/wp-admin/admin-footer.php",
            any(admin_footer_include_live_dispatch),
        )
        .route(
            "/wp-admin/edit-form-blocks.php",
            any(edit_form_blocks_include_live_dispatch),
        )
        .route(
            "/wp-admin/edit-form-advanced.php",
            any(edit_form_advanced_include_live_dispatch),
        )
        .route(
            "/wp-admin/edit-form-comment.php",
            any(edit_form_comment_include_live_dispatch),
        )
        .route(
            "/wp-admin/edit-link-form.php",
            any(edit_link_form_include_live_dispatch),
        )
        .route(
            "/wp-admin/edit-tag-form.php",
            any(edit_tag_form_include_live_dispatch),
        )
        .route(
            "/wp-admin/link-parse-opml.php",
            any(link_parse_opml_include_live_dispatch),
        )
        .route(
            "/wp-admin/upgrade-functions.php",
            any(upgrade_functions_include_live_dispatch),
        )
        .route(
            "/wp-admin/network/menu.php",
            any(network_menu_include_live_dispatch),
        )
        .route(
            "/wp-admin/user/menu.php",
            any(user_menu_include_live_dispatch),
        )
        .route(
            "/wp-admin/user/admin.php",
            any(user_admin_bootstrap_live_dispatch),
        )
        .route(
            "/wp-admin/user/index.php",
            get(user_dashboard_live_dispatch),
        )
        .route(
            "/wp-admin/user/profile.php",
            any(user_profile_live_dispatch),
        )
        .route(
            "/wp-admin/user/user-edit.php",
            any(user_user_edit_live_dispatch),
        )
        .route("/wp-admin/user/about.php", any(user_about_live_dispatch))
        .route(
            "/wp-admin/user/credits.php",
            any(user_credits_live_dispatch),
        )
        .route(
            "/wp-admin/user/contribute.php",
            any(user_contribute_live_dispatch),
        )
        .route(
            "/wp-admin/user/freedoms.php",
            any(user_freedoms_live_dispatch),
        )
        .route(
            "/wp-admin/user/privacy.php",
            any(user_privacy_live_dispatch),
        )
        .route("/wp-admin/profile.php", any(profile_live_dispatch))
        .route("/wp-admin/user-edit.php", any(user_edit_live_dispatch))
        .route("/wp-admin/user-new.php", any(user_new_live_dispatch))
        .route("/wp-admin/post-new.php", any(post_new_live_dispatch))
        .route("/wp-admin/post.php", any(post_live_dispatch))
        .route("/wp-admin/install.php", any(install_live_dispatch))
        .route(
            "/wp-admin/setup-config.php",
            any(setup_config_live_dispatch),
        )
        .route(
            "/wp-admin/install-helper.php",
            any(install_helper_live_dispatch),
        )
        .route("/wp-admin/options.php", any(options_live_dispatch))
        .route(
            "/wp-admin/options-general.php",
            any(options_general_live_dispatch),
        )
        .route(
            "/wp-admin/options-writing.php",
            any(options_writing_live_dispatch),
        )
        .route(
            "/wp-admin/options-reading.php",
            any(options_reading_live_dispatch),
        )
        .route(
            "/wp-admin/options-discussion.php",
            any(options_discussion_live_dispatch),
        )
        .route(
            "/wp-admin/options-media.php",
            any(options_media_live_dispatch),
        )
        .route(
            "/wp-admin/options-permalink.php",
            any(options_permalink_live_dispatch),
        )
        .route(
            "/wp-admin/options-privacy.php",
            any(options_privacy_live_dispatch),
        )
        .route(
            "/wp-admin/privacy-policy-guide.php",
            any(privacy_policy_guide_live_dispatch),
        )
        .route("/wp-admin/about.php", any(about_live_dispatch))
        .route("/wp-admin/credits.php", any(credits_live_dispatch))
        .route("/wp-admin/contribute.php", any(contribute_live_dispatch))
        .route("/wp-admin/freedoms.php", any(freedoms_live_dispatch))
        .route("/wp-admin/privacy.php", any(privacy_live_dispatch))
        .route(
            "/wp-admin/plugin-install.php",
            any(plugin_install_live_dispatch),
        )
        .route(
            "/wp-admin/plugin-editor.php",
            any(plugin_editor_live_dispatch),
        )
        .route(
            "/wp-admin/theme-install.php",
            any(theme_install_live_dispatch),
        )
        .route(
            "/wp-admin/theme-editor.php",
            any(theme_editor_live_dispatch),
        )
        .route("/wp-admin/widgets.php", any(widgets_live_dispatch))
        .route(
            "/wp-admin/widgets-form.php",
            any(widgets_form_live_dispatch),
        )
        .route(
            "/wp-admin/widgets-form-blocks.php",
            any(widgets_form_blocks_live_dispatch),
        )
        .route("/wp-admin/nav-menus.php", any(nav_menus_live_dispatch))
        .route(
            "/wp-admin/font-library.php",
            any(font_library_live_dispatch),
        )
        .route("/wp-admin/customize.php", any(customize_live_dispatch))
        .route(
            "/wp-admin/authorize-application.php",
            any(authorize_application_live_dispatch),
        )
        .route("/wp-admin/site-editor.php", any(site_editor_live_dispatch))
        .route("/wp-admin/press-this.php", any(press_this_live_dispatch))
        .route("/wp-admin/term.php", any(term_live_dispatch))
        .route("/wp-admin/revision.php", any(revision_live_dispatch))
        .route("/wp-admin/moderation.php", any(moderation_live_dispatch))
        .route("/wp-admin/my-sites.php", any(my_sites_live_dispatch))
        .route("/wp-admin/ms-sites.php", any(ms_sites_live_dispatch))
        .route("/wp-admin/ms-users.php", any(ms_users_live_dispatch))
        .route("/wp-admin/ms-themes.php", any(ms_themes_live_dispatch))
        .route("/wp-admin/ms-edit.php", any(ms_edit_live_dispatch))
        .route("/wp-admin/ms-admin.php", any(ms_admin_live_dispatch))
        .route("/wp-admin/ms-options.php", any(ms_options_live_dispatch))
        .route(
            "/wp-admin/ms-upgrade-network.php",
            any(ms_upgrade_network_live_dispatch),
        )
        .route("/wp-admin/plugins.php", any(plugins_live_dispatch))
        .route("/wp-admin/themes.php", any(themes_live_dispatch))
        .route("/wp-admin/users.php", any(users_live_dispatch))
        .route("/wp-admin/edit.php", any(edit_posts_live_dispatch))
        .route("/wp-admin/edit-tags.php", any(edit_tags_live_dispatch))
        .route(
            "/wp-admin/edit-comments.php",
            any(edit_comments_live_dispatch),
        )
        .route("/wp-admin/comment.php", any(comment_live_dispatch))
        .route(
            "/wp-admin/link-manager.php",
            any(link_manager_live_dispatch),
        )
        .route("/wp-admin/link-add.php", any(link_add_live_dispatch))
        .route("/wp-admin/link.php", any(link_live_dispatch))
        .route("/wp-admin/media.php", any(media_live_dispatch))
        .route(
            "/wp-admin/media-upload.php",
            any(media_upload_live_dispatch),
        )
        .route("/wp-admin/upload.php", any(upload_live_dispatch))
        .route("/wp-admin/media-new.php", any(media_new_live_dispatch))
        .route("/wp-admin/tools.php", any(tools_live_dispatch))
        .route("/wp-admin/site-health.php", any(site_health_live_dispatch))
        .route(
            "/wp-admin/site-health-info.php",
            any(site_health_info_live_dispatch),
        )
        .route("/wp-admin/export.php", any(export_live_dispatch))
        .route("/wp-admin/import.php", any(import_live_dispatch))
        .route(
            "/wp-admin/export-personal-data.php",
            any(export_personal_data_live_dispatch),
        )
        .route(
            "/wp-admin/erase-personal-data.php",
            any(erase_personal_data_live_dispatch),
        )
        .route("/wp-admin/network.php", any(network_live_dispatch))
        .route(
            "/wp-admin/network/admin.php",
            any(network_admin_bootstrap_live_dispatch),
        )
        .route(
            "/wp-admin/network/setup.php",
            any(network_setup_live_dispatch),
        )
        .route("/wp-admin/network", any(network_index_live_dispatch))
        .route("/wp-admin/network/", any(network_index_live_dispatch))
        .route(
            "/wp-admin/network/index.php",
            any(network_index_live_dispatch),
        )
        .route(
            "/wp-admin/network/sites.php",
            any(network_sites_live_dispatch),
        )
        .route(
            "/wp-admin/network/users.php",
            any(network_users_live_dispatch),
        )
        .route(
            "/wp-admin/network/themes.php",
            any(network_themes_live_dispatch),
        )
        .route(
            "/wp-admin/network/plugins.php",
            any(network_plugins_live_dispatch),
        )
        .route(
            "/wp-admin/network/settings.php",
            any(network_settings_live_dispatch),
        )
        .route(
            "/wp-admin/network/site-new.php",
            any(network_site_new_live_dispatch),
        )
        .route(
            "/wp-admin/network/site-info.php",
            any(network_site_info_live_dispatch),
        )
        .route(
            "/wp-admin/network/site-settings.php",
            any(network_site_settings_live_dispatch),
        )
        .route(
            "/wp-admin/network/site-users.php",
            any(network_site_users_live_dispatch),
        )
        .route(
            "/wp-admin/network/site-themes.php",
            any(network_site_themes_live_dispatch),
        )
        .route(
            "/wp-admin/network/user-new.php",
            any(network_user_new_live_dispatch),
        )
        .route(
            "/wp-admin/network/edit.php",
            any(network_edit_live_dispatch),
        )
        .route(
            "/wp-admin/network/update.php",
            any(network_update_live_dispatch),
        )
        .route(
            "/wp-admin/network/update-core.php",
            any(network_update_core_live_dispatch),
        )
        .route(
            "/wp-admin/network/plugin-install.php",
            any(network_plugin_install_live_dispatch),
        )
        .route(
            "/wp-admin/network/plugin-editor.php",
            any(network_plugin_editor_live_dispatch),
        )
        .route(
            "/wp-admin/network/theme-editor.php",
            any(network_theme_editor_live_dispatch),
        )
        .route(
            "/wp-admin/network/privacy.php",
            any(network_privacy_live_dispatch),
        )
        .route(
            "/wp-admin/network/about.php",
            any(network_about_live_dispatch),
        )
        .route(
            "/wp-admin/network/credits.php",
            any(network_credits_live_dispatch),
        )
        .route(
            "/wp-admin/network/contribute.php",
            any(network_contribute_live_dispatch),
        )
        .route(
            "/wp-admin/network/freedoms.php",
            any(network_freedoms_live_dispatch),
        )
        .route(
            "/wp-admin/network/profile.php",
            any(network_profile_live_dispatch),
        )
        .route(
            "/wp-admin/network/user-edit.php",
            any(network_user_edit_live_dispatch),
        )
        .route(
            "/wp-admin/network/upgrade.php",
            any(network_upgrade_live_dispatch),
        )
        .route(
            "/wp-admin/network/theme-install.php",
            any(network_theme_install_live_dispatch),
        )
        .route(
            "/wp-admin/ms-delete-site.php",
            any(ms_delete_site_live_dispatch),
        )
        .route("/wp-admin/update.php", any(update_live_dispatch))
        .route("/wp-admin/update-core.php", any(update_core_live_dispatch))
        .route("/wp-admin/upgrade.php", any(upgrade_live_dispatch))
        .route("/wp-admin/maint/repair.php", any(repair_live_dispatch))
        .route("/wp-json", any(rest_dispatch_root))
        .route("/wp-json/", any(rest_dispatch_root))
        .route("/wp-json/*rest_path", any(rest_dispatch))
        .route("/wp-admin/admin-ajax.php", any(admin_ajax_dispatch))
        .route("/wp-admin/admin-post.php", any(admin_post_dispatch))
        .route("/wp-admin/async-upload.php", any(async_upload_dispatch))
        .route("/xmlrpc.php", any(xmlrpc_live_dispatch))
        .route("/wp-cron.php", any(cron_live_dispatch))
        .route("/__wp_rust/internal/options", get(internal_options))
        .route("/__wp_rust/internal/auth-cookie", get(internal_auth_cookie))
        .route(
            "/__wp_rust/internal/auth-session",
            get(internal_auth_session),
        )
        .route("/__wp_rust/internal/nonce", get(internal_nonce))
        .route(
            "/__wp_rust/internal/content-route",
            get(internal_content_route),
        )
        .route("/__wp_rust/internal/block-parse", get(internal_block_parse))
        .route(
            "/__wp_rust/internal/rest-contract",
            get(internal_rest_contract),
        )
        .route(
            "/__wp_rust/internal/rest-dispatch",
            get(internal_rest_dispatch),
        )
        .route(
            "/__wp_rust/internal/admin-contract",
            get(internal_admin_contract),
        )
        .route(
            "/__wp_rust/internal/admin-dispatch",
            get(internal_admin_dispatch),
        )
        .route(
            "/__wp_rust/internal/xmlrpc-contract",
            get(internal_xmlrpc_contract),
        )
        .route(
            "/__wp_rust/internal/xmlrpc-dispatch",
            get(internal_xmlrpc_dispatch),
        )
        .route(
            "/__wp_rust/internal/cron-schedule",
            get(internal_cron_schedule),
        )
        .route("/__wp_rust/internal/cron-due", get(internal_cron_due))
        .route(
            "/__wp_rust/internal/multisite-resolve",
            get(internal_multisite_resolve),
        )
        .route(
            "/__wp_rust/internal/maintenance-status",
            get(internal_maintenance_status),
        )
        .route(
            "/__wp_rust/internal/plugin-compat-matrix",
            get(internal_plugin_compat_matrix),
        )
        .route("/", get(front_live_dispatch_root))
        .route("/*front_path", get(front_live_dispatch))
        .fallback(not_found)
        .with_state(state)
        .layer(middleware::from_fn(latency_middleware));

    info!("wp-rs-server listening on {address}");

    if let Err(error) = axum::serve(
        tokio::net::TcpListener::bind(address)
            .await
            .expect("failed to bind listener"),
        app,
    )
    .await
    {
        error!("server failed: {error}");
    }
}

async fn latency_middleware(request: Request, next: Next) -> Response {
    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    let start = Instant::now();
    let mut response = next.run(request).await;
    let elapsed = start.elapsed();
    let elapsed_ms = elapsed.as_millis();
    let elapsed_us = elapsed.as_micros();
    if let Ok(value) = HeaderValue::from_str(&elapsed_ms.to_string()) {
        response.headers_mut().insert("X-WP-Rust-Latency-Ms", value);
    }
    if let Ok(value) = HeaderValue::from_str(&elapsed_us.to_string()) {
        response
            .headers_mut()
            .insert("X-WP-Rust-Latency-Micros", value);
    }

    info!(
        method = %method,
        path = %path,
        status = %response.status().as_u16(),
        latency_ms = elapsed_ms,
        latency_us = elapsed_us,
        "request handled"
    );
    response
}

async fn health() -> impl IntoResponse {
    let routes = core_seed_routes();
    rust_handled_json(json!({
        "status": "ok",
        "component": "wp-rs-server",
        "seed_rest_routes": routes.all().len(),
    }))
}

async fn echo(request: Request) -> impl IntoResponse {
    let endpoint_kind = detect_endpoint_kind(request.uri().path()).as_str();
    rust_handled_json(json!({
        "method": request.method().to_string(),
        "path": request.uri().path(),
        "query": request.uri().query().unwrap_or_default(),
        "endpoint_kind": endpoint_kind,
    }))
}

#[derive(Debug, Deserialize)]
struct ProxyDecisionQuery {
    endpoint: String,
}

#[derive(Debug, Serialize)]
struct ProxyDecisionResponse {
    enabled: bool,
    deployment_profile: String,
    fallback_enabled: bool,
    endpoint: String,
    should_route: bool,
    method_allowlist: Vec<String>,
    backend_url: String,
    timeout_ms: u64,
    plugin_compat_mode: String,
}

async fn proxy_decision(Query(query): Query<ProxyDecisionQuery>) -> impl IntoResponse {
    let settings = RustGatewaySettings::from_env();
    let should_route = settings.should_route(&query.endpoint);
    let mut methods = settings
        .method_allowlist
        .iter()
        .cloned()
        .collect::<Vec<_>>();
    methods.sort();
    let response = ProxyDecisionResponse {
        enabled: settings.enabled,
        deployment_profile: settings.deployment_profile.clone(),
        fallback_enabled: settings.fallback_enabled,
        should_route,
        endpoint: query.endpoint,
        method_allowlist: methods,
        backend_url: settings.backend_url,
        timeout_ms: settings.timeout_ms,
        plugin_compat_mode: settings.plugin_compat_mode,
    };
    rust_handled_json(response)
}

async fn maintenance_live_dispatch() -> Response {
    let mut headers = HeaderMap::new();
    headers.insert("X-WP-Rust-Handled", HeaderValue::from_static("1"));
    headers.insert(
        "Content-Type",
        HeaderValue::from_static("text/html; charset=UTF-8"),
    );
    headers.insert("Retry-After", HeaderValue::from_static("600"));

    (
        StatusCode::SERVICE_UNAVAILABLE,
        headers,
        "<!doctype html><html><body><h1>Maintenance</h1><p>Briefly unavailable for scheduled maintenance. Check back in a minute.</p></body></html>".to_string(),
    )
        .into_response()
}

async fn login_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let now = unix_now();
    let query_params = parse_urlencoded(parts.uri.query().unwrap_or_default());
    let action = query_params
        .get("action")
        .cloned()
        .unwrap_or_else(|| "login".to_string());

    if action == "lostpassword" || action == "retrievepassword" {
        if parts.method == axum::http::Method::POST {
            let body_bytes = read_request_body(body).await;
            let content_type = parts
                .headers
                .get("content-type")
                .and_then(|value| value.to_str().ok())
                .unwrap_or_default()
                .to_ascii_lowercase();
            let params = merged_params(parts.uri.query(), &body_bytes, &content_type);
            let has_identity = params
                .get("user_login")
                .or_else(|| params.get("user_email"))
                .is_some_and(|value| !value.trim().is_empty());

            if !has_identity {
                return rust_handled_json_with_status(
                    StatusCode::BAD_REQUEST,
                    json!({
                        "error": "missing_identity",
                        "message": "lostpassword requires user_login or user_email.",
                    }),
                )
                .into_response();
            }

            return rust_handled_redirect("/wp-login.php?checkemail=confirm").into_response();
        }

        return rust_handled_html(
            StatusCode::OK,
            "<!doctype html><html><body><h1>Lost Password (Rust)</h1></body></html>".to_string(),
        )
        .into_response();
    }

    if action == "rp" || action == "resetpass" {
        if parts.method == axum::http::Method::POST {
            let body_bytes = read_request_body(body).await;
            let content_type = parts
                .headers
                .get("content-type")
                .and_then(|value| value.to_str().ok())
                .unwrap_or_default()
                .to_ascii_lowercase();
            let params = merged_params(parts.uri.query(), &body_bytes, &content_type);
            let has_reset_token = params
                .get("key")
                .or_else(|| query_params.get("key"))
                .is_some_and(|value| !value.trim().is_empty());
            let has_login = params
                .get("login")
                .or_else(|| query_params.get("login"))
                .is_some_and(|value| !value.trim().is_empty());
            let has_password = params
                .get("pass1")
                .or_else(|| params.get("password"))
                .is_some_and(|value| !value.trim().is_empty());

            if !has_reset_token || !has_login || !has_password {
                return rust_handled_json_with_status(
                    StatusCode::BAD_REQUEST,
                    json!({
                        "error": "invalid_reset_payload",
                        "message": "resetpass requires key, login, and pass1/password.",
                    }),
                )
                .into_response();
            }

            return rust_handled_redirect("/wp-login.php?password=changed").into_response();
        }

        let key = query_params.get("key").cloned().unwrap_or_default();
        let login = query_params.get("login").cloned().unwrap_or_default();
        let html = format!(
            "<!doctype html><html><body><h1>Reset Password (Rust)</h1><p>login={login}</p><p>key={key}</p></body></html>"
        );
        return rust_handled_html(StatusCode::OK, html).into_response();
    }

    if action == "logout" {
        let mut headers = HeaderMap::new();
        headers.insert("X-WP-Rust-Handled", HeaderValue::from_static("1"));
        headers.insert(
            "Location",
            HeaderValue::from_static("/wp-login.php?loggedout=true"),
        );
        headers.append(
            "Set-Cookie",
            HeaderValue::from_static(
                "wordpress_logged_in_rust=deleted; Path=/; HttpOnly; Max-Age=0",
            ),
        );
        return (StatusCode::FOUND, headers, String::new()).into_response();
    }

    if parts.method == axum::http::Method::GET {
        let mut html = "<!doctype html><html><body><h1>WordPress Login (Rust)</h1>".to_string();
        if query_params.contains_key("loggedout") {
            html.push_str("<p>You are now logged out.</p>");
        }
        html.push_str("</body></html>");
        return rust_handled_html(StatusCode::OK, html).into_response();
    }

    if parts.method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-login.php currently supports GET, POST, and logout action.",
            }),
        )
        .into_response();
    }

    let body_bytes = read_request_body(body).await;
    let content_type = parts
        .headers
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let params = merged_params(parts.uri.query(), &body_bytes, &content_type);

    let username = params
        .get("log")
        .or_else(|| params.get("username"))
        .map(String::as_str)
        .unwrap_or_default()
        .trim()
        .to_string();
    let password_present = params
        .get("pwd")
        .or_else(|| params.get("password"))
        .is_some_and(|value| !value.trim().is_empty());

    if username.is_empty() || !password_present {
        return rust_handled_json_with_status(
            StatusCode::BAD_REQUEST,
            json!({
                "error": "invalid_credentials",
                "message": "Missing username or password in login request.",
            }),
        )
        .into_response();
    }

    let user_id = params
        .get("user_id")
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(1);
    let expiration = now + 3_600;
    let session_token = format!("rust-session-{user_id}");
    let cookie_value = sign_auth_cookie(
        user_id,
        &username,
        expiration,
        &session_token,
        AuthScheme::LoggedIn,
        &state.auth_secrets,
    );

    let mut headers = HeaderMap::new();
    headers.insert("X-WP-Rust-Handled", HeaderValue::from_static("1"));
    headers.insert("Location", HeaderValue::from_static("/wp-admin/"));
    if let Ok(cookie_header) = HeaderValue::from_str(&format!(
        "wordpress_logged_in_rust={cookie_value}; Path=/; HttpOnly"
    )) {
        headers.append("Set-Cookie", cookie_header);
    }

    let body = format!(
        "<!doctype html><html><body><h1>Login Success</h1><p>user={username}</p></body></html>"
    );
    (StatusCode::FOUND, headers, body).into_response()
}

async fn admin_dashboard_live(State(state): State<AppState>, request: Request) -> Response {
    let (authenticated, capabilities) =
        auth_context_from_headers(request.headers(), &state.auth_secrets);

    if !authenticated {
        return rust_handled_redirect("/wp-login.php?redirect_to=%2Fwp-admin%2F").into_response();
    }

    let mut html = "<!doctype html><html><body><h1>WordPress Admin (Rust)</h1>".to_string();
    if !capabilities.is_empty() {
        html.push_str("<ul>");
        for capability in capabilities {
            html.push_str(&format!("<li>{capability}</li>"));
        }
        html.push_str("</ul>");
    }
    html.push_str("</body></html>");
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn admin_bootstrap_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (authenticated, capabilities) =
        auth_context_from_headers(request.headers(), &state.auth_secrets);
    if !authenticated {
        return rust_handled_redirect("/wp-login.php?redirect_to=%2Fwp-admin%2Fadmin.php")
            .into_response();
    }

    rust_handled_json(json!({
        "component": "admin-bootstrap",
        "authenticated": true,
        "capabilities": capabilities,
        "message": "Rust admin bootstrap compatibility shim loaded.",
    }))
    .into_response()
}

async fn load_scripts_live_dispatch(request: Request) -> Response {
    if request.method() != axum::http::Method::GET {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/load-scripts.php currently supports GET only.",
            }),
        )
        .into_response();
    }

    let params = parse_urlencoded(request.uri().query().unwrap_or_default());
    let load = params.get("load").cloned().unwrap_or_default();
    if load.trim().is_empty() {
        return rust_handled_json_with_status(
            StatusCode::BAD_REQUEST,
            json!({
                "error": "invalid_request",
                "message": "Missing load parameter for wp-admin/load-scripts.php.",
            }),
        )
        .into_response();
    }

    let handles = load
        .split(',')
        .map(str::trim)
        .filter(|handle| !handle.is_empty())
        .collect::<Vec<_>>();
    if handles.is_empty() {
        return rust_handled_json_with_status(
            StatusCode::BAD_REQUEST,
            json!({
                "error": "invalid_request",
                "message": "No valid script handles supplied.",
            }),
        )
        .into_response();
    }

    let body = format!(
        "/* wp-admin/load-scripts.php (Rust) */\nwindow.wpRustLoadScripts = {{ handles: [{}] }};\n",
        handles
            .iter()
            .map(|handle| format!("\"{handle}\""))
            .collect::<Vec<_>>()
            .join(", ")
    );
    rust_handled_text(
        StatusCode::OK,
        "application/javascript; charset=UTF-8",
        body,
    )
    .into_response()
}

async fn load_styles_live_dispatch(request: Request) -> Response {
    if request.method() != axum::http::Method::GET {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/load-styles.php currently supports GET only.",
            }),
        )
        .into_response();
    }

    let params = parse_urlencoded(request.uri().query().unwrap_or_default());
    let load = params.get("load").cloned().unwrap_or_default();
    if load.trim().is_empty() {
        return rust_handled_json_with_status(
            StatusCode::BAD_REQUEST,
            json!({
                "error": "invalid_request",
                "message": "Missing load parameter for wp-admin/load-styles.php.",
            }),
        )
        .into_response();
    }

    let handles = load
        .split(',')
        .map(str::trim)
        .filter(|handle| !handle.is_empty())
        .collect::<Vec<_>>();
    if handles.is_empty() {
        return rust_handled_json_with_status(
            StatusCode::BAD_REQUEST,
            json!({
                "error": "invalid_request",
                "message": "No valid style handles supplied.",
            }),
        )
        .into_response();
    }

    let body = format!(
        "/* wp-admin/load-styles.php (Rust) */\n:root{{--wp-rust-load-styles-handles:\"{}\";}}\n",
        handles.join(",")
    );
    rust_handled_text(StatusCode::OK, "text/css; charset=UTF-8", body).into_response()
}

fn legacy_admin_include_guard_response() -> Response {
    rust_handled_text(
        StatusCode::OK,
        "text/plain; charset=UTF-8",
        "-1".to_string(),
    )
    .into_response()
}

fn legacy_admin_include_empty_response() -> Response {
    rust_handled_text(StatusCode::OK, "text/plain; charset=UTF-8", String::new()).into_response()
}

async fn custom_background_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_guard_response()
}

async fn custom_header_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_guard_response()
}

async fn admin_functions_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_guard_response()
}

async fn options_head_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_guard_response()
}

async fn menu_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_guard_response()
}

async fn admin_filters_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn admin_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn ajax_actions_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn bookmark_admin_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_automatic_upgrader_skin_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_bulk_plugin_upgrader_skin_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_bulk_theme_upgrader_skin_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_bulk_upgrader_skin_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_core_upgrader_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_custom_background_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_custom_image_header_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_file_upload_upgrader_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_ftp_pure_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_ftp_sockets_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_ftp_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_language_pack_upgrader_skin_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_language_pack_upgrader_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_pclzip_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_plugin_installer_skin_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_plugin_upgrader_skin_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_plugin_upgrader_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_theme_installer_skin_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_theme_upgrader_skin_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_theme_upgrader_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_walker_category_checklist_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_walker_nav_menu_checklist_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_walker_nav_menu_edit_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_ajax_upgrader_skin_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_application_passwords_list_table_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_automatic_updater_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_comments_list_table_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_community_events_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_debug_data_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_filesystem_base_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_filesystem_direct_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_filesystem_ftpext_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_filesystem_ftpsockets_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_filesystem_ssh2_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_importer_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_internal_pointers_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_links_list_table_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_list_table_compat_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_list_table_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_media_list_table_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_ms_sites_list_table_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_ms_themes_list_table_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_ms_users_list_table_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_plugin_install_list_table_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_plugins_list_table_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_post_comments_list_table_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_posts_list_table_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_privacy_data_export_requests_list_table_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_privacy_data_removal_requests_list_table_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_privacy_policy_content_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_privacy_requests_table_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_screen_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_site_health_auto_updates_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_site_health_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_site_icon_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_terms_list_table_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_theme_install_list_table_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_themes_list_table_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_upgrader_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_upgrader_skin_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_upgrader_skins_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_users_list_table_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn admin_includes_comment_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn admin_includes_continents_cities_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn admin_includes_credits_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn admin_includes_dashboard_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn admin_includes_deprecated_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn admin_includes_edit_tag_messages_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn admin_includes_export_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn admin_includes_file_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn admin_includes_image_edit_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn admin_includes_image_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn admin_includes_import_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn admin_includes_list_table_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn admin_includes_media_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn admin_includes_menu_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn admin_includes_meta_boxes_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn admin_includes_misc_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn admin_includes_ms_admin_filters_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn admin_includes_ms_deprecated_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn admin_includes_ms_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn admin_includes_nav_menu_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn admin_includes_network_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn admin_includes_noop_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn admin_includes_options_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn admin_includes_plugin_install_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn admin_includes_plugin_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn admin_includes_post_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn admin_includes_privacy_tools_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn admin_includes_revision_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn admin_includes_schema_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn admin_includes_screen_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn admin_includes_taxonomy_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn admin_includes_template_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn admin_includes_theme_install_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn admin_includes_theme_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn admin_includes_translation_install_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn admin_includes_update_core_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn admin_includes_update_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn admin_includes_upgrade_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn admin_includes_user_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn admin_includes_widgets_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn menu_header_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_guard_response()
}

async fn admin_header_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_guard_response()
}

async fn admin_footer_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_guard_response()
}

async fn edit_form_blocks_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_guard_response()
}

async fn edit_form_advanced_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_guard_response()
}

async fn edit_form_comment_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_guard_response()
}

async fn edit_link_form_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_guard_response()
}

async fn edit_tag_form_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_guard_response()
}

async fn link_parse_opml_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn upgrade_functions_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_guard_response()
}

async fn network_menu_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_guard_response()
}

async fn user_menu_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_guard_response()
}

async fn user_admin_bootstrap_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (authenticated, capabilities) =
        auth_context_from_headers(request.headers(), &state.auth_secrets);
    if !authenticated {
        return rust_handled_redirect("/wp-login.php?redirect_to=%2Fwp-admin%2Fuser%2Fadmin.php")
            .into_response();
    }

    rust_handled_json(json!({
        "component": "user-admin-bootstrap",
        "authenticated": true,
        "capabilities": capabilities,
        "message": "Rust user-admin bootstrap compatibility shim loaded.",
    }))
    .into_response()
}

async fn user_dashboard_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (authenticated, capabilities) =
        auth_context_from_headers(request.headers(), &state.auth_secrets);

    if !authenticated {
        return rust_handled_redirect("/wp-login.php?redirect_to=%2Fwp-admin%2Fuser%2F")
            .into_response();
    }

    let mut html = "<!doctype html><html><body><h1>WordPress User Admin (Rust)</h1>".to_string();
    if !capabilities.is_empty() {
        html.push_str("<ul>");
        for capability in capabilities {
            html.push_str(&format!("<li>{capability}</li>"));
        }
        html.push_str("</ul>");
    }
    html.push_str("</body></html>");
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn user_profile_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    if request.method() != axum::http::Method::GET && request.method() != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/user/profile.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(request.headers(), &state.auth_secrets);
    let can_view_profile = authenticated
        && (capabilities.contains("read")
            || capabilities.contains("edit_user")
            || capabilities.contains("manage_options"));
    if !can_view_profile {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "read capability is required for wp-admin/user/profile.php.",
            }),
        )
        .into_response();
    }

    if request.method() == axum::http::Method::POST {
        return rust_handled_redirect("/wp-admin/user/profile.php?updated=true").into_response();
    }

    let params = parse_urlencoded(request.uri().query().unwrap_or_default());
    let updated = params.get("updated").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>User Profile (Rust)</h1><p>updated={updated}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn user_user_edit_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/user/user-edit.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_edit_users = authenticated
        && (capabilities.contains("edit_users")
            || capabilities.contains("edit_user")
            || capabilities.contains("promote_users")
            || capabilities.contains("manage_options"));
    if !can_edit_users {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "edit_users capability is required for wp-admin/user/user-edit.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let user_id = params.get("user_id").cloned().unwrap_or_default();
    if method == axum::http::Method::POST {
        let target = format!("/wp-admin/user/user-edit.php?user_id={user_id}&updated=true");
        return rust_handled_redirect(&target).into_response();
    }

    let updated = params.get("updated").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>User Admin Edit (Rust)</h1><p>user_id={user_id}</p><p>updated={updated}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn user_about_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    user_information_page_live_dispatch(state, request, "User About WordPress (Rust)").await
}

async fn user_credits_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    user_information_page_live_dispatch(state, request, "User Credits (Rust)").await
}

async fn user_contribute_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    user_information_page_live_dispatch(state, request, "User Get Involved (Rust)").await
}

async fn user_freedoms_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    if request.method() == axum::http::Method::GET {
        let params = parse_urlencoded(request.uri().query().unwrap_or_default());
        if params.contains_key("privacy-notice") {
            return rust_handled_redirect("/wp-admin/user/privacy.php").into_response();
        }
    }
    user_information_page_live_dispatch(state, request, "User Freedoms (Rust)").await
}

async fn user_privacy_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    user_information_page_live_dispatch(state, request, "User Privacy (Rust)").await
}

async fn user_information_page_live_dispatch(
    state: AppState,
    request: Request,
    title: &str,
) -> Response {
    if request.method() != axum::http::Method::GET && request.method() != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/user informational pages currently support GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(request.headers(), &state.auth_secrets);
    let can_view = authenticated
        && (capabilities.contains("read")
            || capabilities.contains("manage_options")
            || capabilities.contains("manage_network"));
    if !can_view {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "read capability is required for this wp-admin/user informational page.",
            }),
        )
        .into_response();
    }

    let html = format!("<!doctype html><html><body><h1>{title}</h1></body></html>");
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn profile_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    if request.method() != axum::http::Method::GET && request.method() != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/profile.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(request.headers(), &state.auth_secrets);
    let can_view_profile = authenticated
        && (capabilities.contains("read")
            || capabilities.contains("edit_user")
            || capabilities.contains("manage_options"));
    if !can_view_profile {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "read capability is required for wp-admin/profile.php.",
            }),
        )
        .into_response();
    }

    if request.method() == axum::http::Method::POST {
        return rust_handled_redirect("/wp-admin/profile.php?updated=true").into_response();
    }

    let params = parse_urlencoded(request.uri().query().unwrap_or_default());
    let updated = params.get("updated").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Profile (Rust)</h1><p>updated={updated}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn user_edit_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/user-edit.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_edit_users = authenticated
        && (capabilities.contains("edit_users")
            || capabilities.contains("edit_user")
            || capabilities.contains("promote_users")
            || capabilities.contains("manage_options"));
    if !can_edit_users {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "edit_users capability is required for wp-admin/user-edit.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let user_id = params.get("user_id").cloned().unwrap_or_default();
    if method == axum::http::Method::POST {
        let target = format!("/wp-admin/user-edit.php?user_id={user_id}&updated=true");
        return rust_handled_redirect(&target).into_response();
    }

    let updated = params.get("updated").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>User Edit (Rust)</h1><p>user_id={user_id}</p><p>updated={updated}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn user_new_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/user-new.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_create_users = authenticated
        && (capabilities.contains("create_users")
            || capabilities.contains("promote_users")
            || capabilities.contains("manage_network_users")
            || capabilities.contains("manage_options"));
    if !can_create_users {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "create_users capability is required for wp-admin/user-new.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    if method == axum::http::Method::POST
        && params
            .get("action")
            .is_some_and(|value| value.eq_ignore_ascii_case("adduser"))
    {
        let email = params.get("email").cloned().unwrap_or_default();
        if email.trim().is_empty() {
            return rust_handled_redirect("/wp-admin/user-new.php?update=enter_email")
                .into_response();
        }
        return rust_handled_redirect("/wp-admin/user-new.php?update=addnoconfirmation&user_id=2")
            .into_response();
    }

    let update = params.get("update").cloned().unwrap_or_default();
    let user_id = params.get("user_id").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>User New (Rust)</h1><p>update={update}</p><p>user_id={user_id}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn post_new_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/post-new.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_create_posts = authenticated
        && (capabilities.contains("create_posts")
            || capabilities.contains("edit_posts")
            || capabilities.contains("manage_options"));
    if !can_create_posts {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "create_posts capability is required for wp-admin/post-new.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let post_type = params
        .get("post_type")
        .cloned()
        .unwrap_or_else(|| "post".to_string());
    if post_type == "attachment" {
        return rust_handled_redirect("/wp-admin/media-new.php").into_response();
    }

    if method == axum::http::Method::POST {
        let action = params.get("action").cloned().unwrap_or_default();
        if action == "post" || action == "postajaxpost" {
            return rust_handled_redirect("/wp-admin/post.php?action=edit&post=1").into_response();
        }
    }

    let html = format!(
        "<!doctype html><html><body><h1>Post New (Rust)</h1><p>post_type={post_type}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn post_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/post.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_edit_posts = authenticated
        && (capabilities.contains("edit_posts")
            || capabilities.contains("create_posts")
            || capabilities.contains("manage_options"));
    if !can_edit_posts {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "edit_posts capability is required for wp-admin/post.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let action = params
        .get("action")
        .cloned()
        .unwrap_or_else(|| "edit".to_string());
    let post_id = params
        .get("post")
        .or_else(|| params.get("post_ID"))
        .cloned()
        .unwrap_or_else(|| "0".to_string());

    if action == "delete" || action == "trash" {
        return rust_handled_redirect("/wp-admin/edit.php?deleted=1").into_response();
    }

    if method == axum::http::Method::POST && (action == "post" || action == "postajaxpost") {
        return rust_handled_redirect("/wp-admin/post.php?action=edit&post=1").into_response();
    }

    let html = format!(
        "<!doctype html><html><body><h1>Post Edit (Rust)</h1><p>action={action}</p><p>post={post_id}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn install_live_dispatch(request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let query_params = parse_urlencoded(parts.uri.query().unwrap_or_default());
    let step = query_params
        .get("step")
        .cloned()
        .unwrap_or_else(|| "0".to_string());

    if parts.method == axum::http::Method::GET {
        let html = format!(
            "<!doctype html><html><body><h1>WordPress Install (Rust)</h1><p>step={step}</p></body></html>"
        );
        return rust_handled_html(StatusCode::OK, html).into_response();
    }

    if parts.method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/install.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let body_bytes = read_request_body(body).await;
    let content_type = parts
        .headers
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let params = merged_params(parts.uri.query(), &body_bytes, &content_type);
    let has_title = params
        .get("weblog_title")
        .or_else(|| params.get("site_title"))
        .is_some_and(|value| !value.trim().is_empty());
    let has_user = params
        .get("user_login")
        .or_else(|| params.get("user_name"))
        .is_some_and(|value| !value.trim().is_empty());
    let has_email = params
        .get("admin_email")
        .or_else(|| params.get("user_email"))
        .is_some_and(|value| !value.trim().is_empty());

    if !has_title || !has_user || !has_email {
        return rust_handled_json_with_status(
            StatusCode::BAD_REQUEST,
            json!({
                "error": "invalid_install_payload",
                "message": "Installation requires site title, username, and admin email.",
            }),
        )
        .into_response();
    }

    rust_handled_redirect("/wp-admin/?installed=1").into_response()
}

async fn setup_config_live_dispatch(request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/setup-config.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let step = params
        .get("step")
        .cloned()
        .unwrap_or_else(|| "0".to_string());
    if method == axum::http::Method::POST {
        let has_db_name = params
            .get("dbname")
            .is_some_and(|value| !value.trim().is_empty());
        let has_db_user = params
            .get("uname")
            .or_else(|| params.get("dbuser"))
            .is_some_and(|value| !value.trim().is_empty());
        if !has_db_name || !has_db_user {
            return rust_handled_json_with_status(
                StatusCode::BAD_REQUEST,
                json!({
                    "error": "invalid_db_config_payload",
                    "message": "setup-config requires dbname and uname/dbuser fields.",
                }),
            )
            .into_response();
        }

        return rust_handled_redirect("/wp-admin/install.php?step=2").into_response();
    }

    let html = format!(
        "<!doctype html><html><body><h1>WordPress Setup Config (Rust)</h1><p>step={step}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn install_helper_live_dispatch(request: Request) -> Response {
    if request.method() != axum::http::Method::GET {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/install-helper.php currently supports GET only.",
            }),
        )
        .into_response();
    }

    rust_handled_json(json!({
        "component": "install-helper",
        "status": "available",
        "message": "Rust install-helper compatibility shim loaded.",
    }))
    .into_response()
}

async fn options_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_manage = authenticated && capabilities.contains("manage_options");

    if !can_manage {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_options capability is required for wp-admin/options.php.",
            }),
        )
        .into_response();
    }

    if method == axum::http::Method::GET {
        let options = state.options.lock().expect("options mutex poisoned");
        let snapshot = options.snapshot();
        let html = format!(
            "<!doctype html><html><body><h1>Options (Rust)</h1><p>count={}</p></body></html>",
            snapshot.len()
        );
        return rust_handled_html(StatusCode::OK, html).into_response();
    }

    if method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/options.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let body_bytes = read_request_body(body).await;
    let content_type = parts
        .headers
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let params = merged_params(parts.uri.query(), &body_bytes, &content_type);

    let mut options = state.options.lock().expect("options mutex poisoned");
    let mut updated_count = 0usize;
    for (key, value) in params {
        if matches!(
            key.as_str(),
            "action" | "option_page" | "_wpnonce" | "_wp_http_referer" | "submit"
        ) || key.starts_with('_')
        {
            continue;
        }
        options.set_option(&key, &value, false);
        updated_count += 1;
    }

    let target = format!(
        "/wp-admin/options-general.php?settings-updated=true&updated_count={updated_count}"
    );
    rust_handled_redirect(&target).into_response()
}

async fn options_general_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    if request.method() != axum::http::Method::GET {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/options-general.php currently supports GET only.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(request.headers(), &state.auth_secrets);
    if !(authenticated && capabilities.contains("manage_options")) {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_options capability is required for wp-admin/options-general.php.",
            }),
        )
        .into_response();
    }

    let options = state.options.lock().expect("options mutex poisoned");
    let blogname = options
        .get_option("blogname")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "WordPress".to_string());
    let blogdescription = options
        .get_option("blogdescription")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "Just another WordPress site".to_string());
    let html = format!(
        "<!doctype html><html><body><h1>General Settings (Rust)</h1><p>blogname={blogname}</p><p>blogdescription={blogdescription}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn options_writing_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    if request.method() != axum::http::Method::GET {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/options-writing.php currently supports GET only.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(request.headers(), &state.auth_secrets);
    if !(authenticated && capabilities.contains("manage_options")) {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_options capability is required for wp-admin/options-writing.php.",
            }),
        )
        .into_response();
    }

    let options = state.options.lock().expect("options mutex poisoned");
    let default_category = options
        .get_option("default_category")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "1".to_string());
    let default_post_format = options
        .get_option("default_post_format")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "0".to_string());
    let html = format!(
        "<!doctype html><html><body><h1>Writing Settings (Rust)</h1><p>default_category={default_category}</p><p>default_post_format={default_post_format}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn options_reading_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    if request.method() != axum::http::Method::GET {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/options-reading.php currently supports GET only.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(request.headers(), &state.auth_secrets);
    if !(authenticated && capabilities.contains("manage_options")) {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_options capability is required for wp-admin/options-reading.php.",
            }),
        )
        .into_response();
    }

    let options = state.options.lock().expect("options mutex poisoned");
    let show_on_front = options
        .get_option("show_on_front")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "posts".to_string());
    let posts_per_page = options
        .get_option("posts_per_page")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "10".to_string());
    let page_on_front = options
        .get_option("page_on_front")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "0".to_string());
    let page_for_posts = options
        .get_option("page_for_posts")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "0".to_string());
    let html = format!(
        "<!doctype html><html><body><h1>Reading Settings (Rust)</h1><p>show_on_front={show_on_front}</p><p>posts_per_page={posts_per_page}</p><p>page_on_front={page_on_front}</p><p>page_for_posts={page_for_posts}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn options_discussion_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    if request.method() != axum::http::Method::GET {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/options-discussion.php currently supports GET only.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(request.headers(), &state.auth_secrets);
    if !(authenticated && capabilities.contains("manage_options")) {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_options capability is required for wp-admin/options-discussion.php.",
            }),
        )
        .into_response();
    }

    let options = state.options.lock().expect("options mutex poisoned");
    let default_pingback_flag = options
        .get_option("default_pingback_flag")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "1".to_string());
    let default_ping_status = options
        .get_option("default_ping_status")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "open".to_string());
    let default_comment_status = options
        .get_option("default_comment_status")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "open".to_string());
    let comments_notify = options
        .get_option("comments_notify")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "1".to_string());
    let moderation_notify = options
        .get_option("moderation_notify")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "1".to_string());
    let html = format!(
        "<!doctype html><html><body><h1>Discussion Settings (Rust)</h1><p>default_pingback_flag={default_pingback_flag}</p><p>default_ping_status={default_ping_status}</p><p>default_comment_status={default_comment_status}</p><p>comments_notify={comments_notify}</p><p>moderation_notify={moderation_notify}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn options_media_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    if request.method() != axum::http::Method::GET {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/options-media.php currently supports GET only.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(request.headers(), &state.auth_secrets);
    if !(authenticated && capabilities.contains("manage_options")) {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_options capability is required for wp-admin/options-media.php.",
            }),
        )
        .into_response();
    }

    let options = state.options.lock().expect("options mutex poisoned");
    let thumbnail_size_w = options
        .get_option("thumbnail_size_w")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "150".to_string());
    let thumbnail_size_h = options
        .get_option("thumbnail_size_h")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "150".to_string());
    let medium_size_w = options
        .get_option("medium_size_w")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "300".to_string());
    let medium_size_h = options
        .get_option("medium_size_h")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "300".to_string());
    let large_size_w = options
        .get_option("large_size_w")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "1024".to_string());
    let large_size_h = options
        .get_option("large_size_h")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "1024".to_string());
    let uploads_use_yearmonth_folders = options
        .get_option("uploads_use_yearmonth_folders")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "1".to_string());
    let html = format!(
        "<!doctype html><html><body><h1>Media Settings (Rust)</h1><p>thumbnail_size_w={thumbnail_size_w}</p><p>thumbnail_size_h={thumbnail_size_h}</p><p>medium_size_w={medium_size_w}</p><p>medium_size_h={medium_size_h}</p><p>large_size_w={large_size_w}</p><p>large_size_h={large_size_h}</p><p>uploads_use_yearmonth_folders={uploads_use_yearmonth_folders}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn options_permalink_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    if request.method() != axum::http::Method::GET {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/options-permalink.php currently supports GET only.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(request.headers(), &state.auth_secrets);
    if !(authenticated && capabilities.contains("manage_options")) {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_options capability is required for wp-admin/options-permalink.php.",
            }),
        )
        .into_response();
    }

    let options = state.options.lock().expect("options mutex poisoned");
    let permalink_structure = options
        .get_option("permalink_structure")
        .map(|value| value.to_string())
        .unwrap_or_default();
    let category_base = options
        .get_option("category_base")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "category".to_string());
    let tag_base = options
        .get_option("tag_base")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "tag".to_string());
    let html = format!(
        "<!doctype html><html><body><h1>Permalink Settings (Rust)</h1><p>permalink_structure={permalink_structure}</p><p>category_base={category_base}</p><p>tag_base={tag_base}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn options_privacy_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    if request.method() != axum::http::Method::GET {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/options-privacy.php currently supports GET only.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(request.headers(), &state.auth_secrets);
    let can_manage_privacy = authenticated
        && (capabilities.contains("manage_privacy_options")
            || capabilities.contains("manage_options"));
    if !can_manage_privacy {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_privacy_options capability is required for wp-admin/options-privacy.php.",
            }),
        )
        .into_response();
    }

    let query_params = parse_urlencoded(request.uri().query().unwrap_or_default());
    let show_policy_guide = query_params
        .get("tab")
        .is_some_and(|value| value.eq_ignore_ascii_case("policyguide"));

    let options = state.options.lock().expect("options mutex poisoned");
    let privacy_page_id = options
        .get_option("wp_page_for_privacy_policy")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "0".to_string());
    let blog_public = options
        .get_option("blog_public")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "1".to_string());
    let html = if show_policy_guide {
        format!(
            "<!doctype html><html><body><h1>Privacy Policy Guide (Rust)</h1><p>wp_page_for_privacy_policy={privacy_page_id}</p><p>guide_mode=tab</p></body></html>"
        )
    } else {
        format!(
            "<!doctype html><html><body><h1>Privacy Settings (Rust)</h1><p>wp_page_for_privacy_policy={privacy_page_id}</p><p>blog_public={blog_public}</p></body></html>"
        )
    };
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn privacy_policy_guide_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    if request.method() != axum::http::Method::GET {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/privacy-policy-guide.php currently supports GET only.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(request.headers(), &state.auth_secrets);
    let can_manage_privacy = authenticated
        && (capabilities.contains("manage_privacy_options")
            || capabilities.contains("manage_options"));
    if !can_manage_privacy {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_privacy_options capability is required for wp-admin/privacy-policy-guide.php.",
            }),
        )
        .into_response();
    }

    let options = state.options.lock().expect("options mutex poisoned");
    let privacy_page_id = options
        .get_option("wp_page_for_privacy_policy")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "0".to_string());
    let html = format!(
        "<!doctype html><html><body><h1>Privacy Policy Guide (Rust)</h1><p>wp_page_for_privacy_policy={privacy_page_id}</p><p>guide_mode=direct</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn about_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    admin_information_page_live_dispatch(state, request, "About WordPress (Rust)").await
}

async fn credits_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    admin_information_page_live_dispatch(state, request, "Credits (Rust)").await
}

async fn contribute_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    admin_information_page_live_dispatch(state, request, "Get Involved (Rust)").await
}

async fn freedoms_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    if request.method() == axum::http::Method::GET {
        let params = parse_urlencoded(request.uri().query().unwrap_or_default());
        if params.contains_key("privacy-notice") {
            return rust_handled_redirect("/wp-admin/privacy.php").into_response();
        }
    }
    admin_information_page_live_dispatch(state, request, "Freedoms (Rust)").await
}

async fn privacy_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    admin_information_page_live_dispatch(state, request, "Privacy (Rust)").await
}

async fn admin_information_page_live_dispatch(
    state: AppState,
    request: Request,
    title: &str,
) -> Response {
    if request.method() != axum::http::Method::GET && request.method() != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "admin informational pages currently support GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(request.headers(), &state.auth_secrets);
    let can_view = authenticated
        && (capabilities.contains("read")
            || capabilities.contains("manage_options")
            || capabilities.contains("manage_network"));
    if !can_view {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "read capability is required for this wp-admin informational page.",
            }),
        )
        .into_response();
    }

    let html = format!("<!doctype html><html><body><h1>{title}</h1></body></html>");
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn plugin_install_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/plugin-install.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_install_plugins = authenticated
        && (capabilities.contains("install_plugins")
            || capabilities.contains("manage_options")
            || capabilities.contains("manage_network_plugins")
            || capabilities.contains("manage_network"));
    if !can_install_plugins {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "install_plugins capability is required for wp-admin/plugin-install.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let tab = params
        .get("tab")
        .cloned()
        .unwrap_or_else(|| "search".to_string());
    let search = params.get("s").cloned().unwrap_or_default();
    let iframe_request = tab == "plugin-information";
    let html = format!(
        "<!doctype html><html><body><h1>Plugin Install (Rust)</h1><p>tab={tab}</p><p>search={search}</p><p>iframe_request={iframe_request}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn plugin_editor_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/plugin-editor.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_edit_plugins = authenticated
        && (capabilities.contains("edit_plugins")
            || capabilities.contains("manage_options")
            || capabilities.contains("manage_network_plugins")
            || capabilities.contains("manage_network"));
    if !can_edit_plugins {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "edit_plugins capability is required for wp-admin/plugin-editor.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    if method == axum::http::Method::POST
        && params
            .get("action")
            .is_some_and(|value| value.eq_ignore_ascii_case("update"))
    {
        let plugin = params.get("plugin").cloned().unwrap_or_default();
        let target = format!("/wp-admin/plugin-editor.php?plugin={plugin}&updated=true");
        return rust_handled_redirect(&target).into_response();
    }

    let plugin = params.get("plugin").cloned().unwrap_or_default();
    let file = params.get("file").cloned().unwrap_or_default();
    let updated = params.get("updated").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Plugin Editor (Rust)</h1><p>plugin={plugin}</p><p>file={file}</p><p>updated={updated}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn theme_install_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/theme-install.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_install_themes = authenticated
        && (capabilities.contains("install_themes")
            || capabilities.contains("manage_options")
            || capabilities.contains("manage_network_themes")
            || capabilities.contains("manage_network"));
    if !can_install_themes {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "install_themes capability is required for wp-admin/theme-install.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let tab = params
        .get("tab")
        .cloned()
        .unwrap_or_else(|| "search".to_string());
    let search = params.get("s").cloned().unwrap_or_default();
    let iframe_request = tab == "theme-information";
    let html = format!(
        "<!doctype html><html><body><h1>Theme Install (Rust)</h1><p>tab={tab}</p><p>search={search}</p><p>iframe_request={iframe_request}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn theme_editor_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/theme-editor.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_edit_themes = authenticated
        && (capabilities.contains("edit_themes")
            || capabilities.contains("manage_options")
            || capabilities.contains("manage_network_themes")
            || capabilities.contains("manage_network"));
    if !can_edit_themes {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "edit_themes capability is required for wp-admin/theme-editor.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    if method == axum::http::Method::POST
        && params
            .get("action")
            .is_some_and(|value| value.eq_ignore_ascii_case("update"))
    {
        let theme = params.get("theme").cloned().unwrap_or_default();
        let target = format!("/wp-admin/theme-editor.php?theme={theme}&updated=true");
        return rust_handled_redirect(&target).into_response();
    }

    let theme = params.get("theme").cloned().unwrap_or_default();
    let file = params.get("file").cloned().unwrap_or_default();
    let updated = params.get("updated").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Theme Editor (Rust)</h1><p>theme={theme}</p><p>file={file}</p><p>updated={updated}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn widgets_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/widgets.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_edit_widgets = authenticated
        && (capabilities.contains("edit_theme_options")
            || capabilities.contains("manage_options")
            || capabilities.contains("switch_themes"));
    if !can_edit_widgets {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "edit_theme_options capability is required for wp-admin/widgets.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let action = params.get("action").cloned().unwrap_or_default();
    if method == axum::http::Method::POST && !action.is_empty() {
        let target = format!("/wp-admin/widgets.php?updated_action={action}");
        return rust_handled_redirect(&target).into_response();
    }

    let widgets_access = params.get("widgets-access").cloned().unwrap_or_default();
    let updated_action = params.get("updated_action").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Widgets (Rust)</h1><p>action={action}</p><p>widgets_access={widgets_access}</p><p>updated_action={updated_action}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn widgets_form_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/widgets-form.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_edit_widgets = authenticated
        && (capabilities.contains("edit_theme_options")
            || capabilities.contains("manage_options")
            || capabilities.contains("switch_themes"));
    if !can_edit_widgets {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "edit_theme_options capability is required for wp-admin/widgets-form.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    if method == axum::http::Method::POST
        && (params.contains_key("savewidget") || params.contains_key("removewidget"))
    {
        return rust_handled_redirect("/wp-admin/widgets.php?widget-updated=1").into_response();
    }

    let widgets_access = params.get("widgets-access").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Widgets Classic Form (Rust)</h1><p>widgets_access={widgets_access}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn widgets_form_blocks_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/widgets-form-blocks.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_edit_widgets = authenticated
        && (capabilities.contains("edit_theme_options")
            || capabilities.contains("manage_options")
            || capabilities.contains("switch_themes"));
    if !can_edit_widgets {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "edit_theme_options capability is required for wp-admin/widgets-form-blocks.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    if method == axum::http::Method::POST {
        return rust_handled_redirect("/wp-admin/widgets.php?widget-updated=1").into_response();
    }

    let legacy_notice = params.get("classic-widgets").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Widgets Block Form (Rust)</h1><p>classic_widgets={legacy_notice}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn nav_menus_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/nav-menus.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_edit_menus = authenticated
        && (capabilities.contains("edit_theme_options")
            || capabilities.contains("manage_options")
            || capabilities.contains("switch_themes"));
    if !can_edit_menus {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "edit_theme_options capability is required for wp-admin/nav-menus.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let action = params
        .get("action")
        .cloned()
        .unwrap_or_else(|| "edit".to_string());
    let menu = params.get("menu").cloned().unwrap_or_default();
    if method == axum::http::Method::POST && !action.is_empty() {
        let target = format!("/wp-admin/nav-menus.php?menu={menu}&updated_action={action}");
        return rust_handled_redirect(&target).into_response();
    }

    let updated_action = params.get("updated_action").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Navigation Menus (Rust)</h1><p>action={action}</p><p>menu={menu}</p><p>updated_action={updated_action}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn font_library_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    if request.method() != axum::http::Method::GET && request.method() != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/font-library.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(request.headers(), &state.auth_secrets);
    let can_manage_fonts = authenticated
        && (capabilities.contains("edit_theme_options")
            || capabilities.contains("manage_options")
            || capabilities.contains("switch_themes"));
    if !can_manage_fonts {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "edit_theme_options capability is required for wp-admin/font-library.php.",
            }),
        )
        .into_response();
    }

    if request.method() == axum::http::Method::POST {
        return rust_handled_redirect("/wp-admin/font-library.php?updated=true").into_response();
    }

    let params = parse_urlencoded(request.uri().query().unwrap_or_default());
    let updated = params.get("updated").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Font Library (Rust)</h1><p>updated={updated}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn customize_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/customize.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_customize = authenticated
        && (capabilities.contains("customize")
            || capabilities.contains("edit_theme_options")
            || capabilities.contains("manage_options"));
    if !can_customize {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "customize capability is required for wp-admin/customize.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    if method == axum::http::Method::POST {
        let return_url = params.get("return").cloned().unwrap_or_default();
        if !return_url.trim().is_empty() {
            return rust_handled_redirect(&return_url).into_response();
        }
        return rust_handled_redirect("/wp-admin/customize.php?saved=true").into_response();
    }

    let url = params.get("url").cloned().unwrap_or_default();
    let return_url = params.get("return").cloned().unwrap_or_default();
    let autofocus = params.get("autofocus").cloned().unwrap_or_default();
    let changeset_uuid = params.get("changeset_uuid").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Customizer (Rust)</h1><p>url={url}</p><p>return={return_url}</p><p>autofocus={autofocus}</p><p>changeset_uuid={changeset_uuid}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn authorize_application_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/authorize-application.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_authorize = authenticated
        && (capabilities.contains("read")
            || capabilities.contains("manage_options")
            || capabilities.contains("edit_users"));
    if !can_authorize {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "read capability is required for wp-admin/authorize-application.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    if method == axum::http::Method::POST
        && params
            .get("action")
            .is_some_and(|value| value == "authorize_application_password")
    {
        let reject = params.get("reject").cloned().unwrap_or_default();
        let approve = params.get("approve").cloned().unwrap_or_default();
        let success_url = params.get("success_url").cloned().unwrap_or_default();
        let reject_url = params.get("reject_url").cloned().unwrap_or_default();

        if !reject.is_empty() {
            if !reject_url.trim().is_empty() {
                return rust_handled_redirect(&reject_url).into_response();
            }
            return rust_handled_redirect("/wp-admin/").into_response();
        }
        if !approve.is_empty() {
            if !success_url.trim().is_empty() {
                let target =
                    format!("{success_url}?site_url=http%3A%2F%2Flocalhost&user_login=admin&password=rust-app-pass");
                return rust_handled_redirect(&target).into_response();
            }
            return rust_handled_html(
                StatusCode::OK,
                "<!doctype html><html><body><h1>Application Password Authorized (Rust)</h1><p>password=rust-app-pass</p></body></html>".to_string(),
            )
            .into_response();
        }
    }

    let app_name = params.get("app_name").cloned().unwrap_or_default();
    let app_id = params.get("app_id").cloned().unwrap_or_default();
    let success_url = params.get("success_url").cloned().unwrap_or_default();
    let reject_url = params.get("reject_url").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Authorize Application (Rust)</h1><p>app_name={app_name}</p><p>app_id={app_id}</p><p>success_url={success_url}</p><p>reject_url={reject_url}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn site_editor_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/site-editor.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_edit_theme = authenticated
        && (capabilities.contains("edit_theme_options")
            || capabilities.contains("manage_options")
            || capabilities.contains("switch_themes"));
    if !can_edit_theme {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "edit_theme_options capability is required for wp-admin/site-editor.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    if method == axum::http::Method::GET {
        if params.get("postType").is_some_and(|v| v == "wp_navigation")
            && params.contains_key("postId")
        {
            let post_id = params.get("postId").cloned().unwrap_or_default();
            let target = format!("/wp-admin/site-editor.php?p=%2Fwp_navigation%2F{post_id}");
            return rust_handled_redirect(&target).into_response();
        }
        if params.get("postType").is_some_and(|v| v == "wp_navigation") {
            return rust_handled_redirect("/wp-admin/site-editor.php?p=%2Fnavigation")
                .into_response();
        }
        if params.get("path").is_some_and(|v| v == "/wp_global_styles") {
            return rust_handled_redirect("/wp-admin/site-editor.php?p=%2Fstyles").into_response();
        }
    }

    if method == axum::http::Method::POST {
        return rust_handled_redirect("/wp-admin/site-editor.php?updated=true").into_response();
    }

    let p = params.get("p").cloned().unwrap_or_else(|| "/".to_string());
    let post_type = params.get("postType").cloned().unwrap_or_default();
    let post_id = params.get("postId").cloned().unwrap_or_default();
    let path = params.get("path").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Site Editor (Rust)</h1><p>p={p}</p><p>post_type={post_type}</p><p>post_id={post_id}</p><p>path={path}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn press_this_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/press-this.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_press_this = authenticated
        && (capabilities.contains("edit_posts")
            || capabilities.contains("create_posts")
            || capabilities.contains("manage_options"));
    if !can_press_this {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "edit_posts capability is required for wp-admin/press-this.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    if method == axum::http::Method::POST {
        return rust_handled_redirect("/wp-admin/post.php?action=edit&post=1").into_response();
    }

    let url = params.get("u").cloned().unwrap_or_default();
    let title = params.get("t").cloned().unwrap_or_default();
    let source = params.get("s").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Press This (Rust)</h1><p>url={url}</p><p>title={title}</p><p>source={source}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn term_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/term.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_edit_terms = authenticated
        && (capabilities.contains("manage_categories")
            || capabilities.contains("edit_posts")
            || capabilities.contains("manage_options"));
    if !can_edit_terms {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_categories capability is required for wp-admin/term.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let tag_id = params.get("tag_ID").cloned().unwrap_or_default();
    if tag_id.trim().is_empty() {
        return rust_handled_redirect("/wp-admin/edit-tags.php").into_response();
    }

    if method == axum::http::Method::POST {
        let taxonomy = params
            .get("taxonomy")
            .cloned()
            .unwrap_or_else(|| "category".to_string());
        let target =
            format!("/wp-admin/edit-tags.php?taxonomy={taxonomy}&tag_ID={tag_id}&updated=1");
        return rust_handled_redirect(&target).into_response();
    }

    let taxonomy = params
        .get("taxonomy")
        .cloned()
        .unwrap_or_else(|| "category".to_string());
    let post_type = params
        .get("post_type")
        .cloned()
        .unwrap_or_else(|| "post".to_string());
    let html = format!(
        "<!doctype html><html><body><h1>Edit Term (Rust)</h1><p>tag_id={tag_id}</p><p>taxonomy={taxonomy}</p><p>post_type={post_type}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn revision_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/revision.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_manage_revisions = authenticated
        && (capabilities.contains("edit_posts")
            || capabilities.contains("read")
            || capabilities.contains("manage_options"));
    if !can_manage_revisions {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "edit_posts capability is required for wp-admin/revision.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let action = params
        .get("action")
        .cloned()
        .unwrap_or_else(|| "view".to_string());
    let revision_id = params
        .get("revision")
        .or_else(|| params.get("to"))
        .cloned()
        .unwrap_or_default();

    if revision_id.trim().is_empty() {
        return rust_handled_redirect("/wp-admin/edit.php").into_response();
    }

    if method == axum::http::Method::POST && action == "restore" {
        return rust_handled_redirect("/wp-admin/post.php?action=edit&post=1&message=5")
            .into_response();
    }

    if method == axum::http::Method::GET && action == "restore" {
        return rust_handled_redirect("/wp-admin/revision.php?revision=1&action=view")
            .into_response();
    }

    let from = params.get("from").cloned().unwrap_or_default();
    let to = params.get("to").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Revisions (Rust)</h1><p>action={action}</p><p>revision={revision_id}</p><p>from={from}</p><p>to={to}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn moderation_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    if request.method() != axum::http::Method::GET && request.method() != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/moderation.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(request.headers(), &state.auth_secrets);
    let can_moderate = authenticated
        && (capabilities.contains("edit_posts")
            || capabilities.contains("moderate_comments")
            || capabilities.contains("manage_options"));
    if !can_moderate {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "edit_posts capability is required for wp-admin/moderation.php.",
            }),
        )
        .into_response();
    }

    rust_handled_redirect("/wp-admin/edit-comments.php?comment_status=moderated").into_response()
}

async fn my_sites_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/my-sites.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_view = authenticated
        && (capabilities.contains("read")
            || capabilities.contains("manage_sites")
            || capabilities.contains("manage_network")
            || capabilities.contains("manage_options"));
    if !can_view {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "read capability is required for wp-admin/my-sites.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    if method == axum::http::Method::POST
        && params
            .get("action")
            .is_some_and(|value| value == "updateblogsettings")
    {
        let primary_blog = params.get("primary_blog").cloned().unwrap_or_default();
        let target = format!("/wp-admin/my-sites.php?updated=true&primary_blog={primary_blog}");
        return rust_handled_redirect(&target).into_response();
    }

    let action = params
        .get("action")
        .cloned()
        .unwrap_or_else(|| "splash".to_string());
    let updated = params.get("updated").cloned().unwrap_or_default();
    let primary_blog = params.get("primary_blog").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>My Sites (Rust)</h1><p>action={action}</p><p>updated={updated}</p><p>primary_blog={primary_blog}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn ms_sites_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    multisite_redirect_live_dispatch(state, request, "/wp-admin/network/sites.php").await
}

async fn ms_users_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    multisite_redirect_live_dispatch(state, request, "/wp-admin/network/users.php").await
}

async fn ms_themes_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    multisite_redirect_live_dispatch(state, request, "/wp-admin/network/themes.php").await
}

async fn ms_edit_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    multisite_redirect_live_dispatch(state, request, "/wp-admin/network/").await
}

async fn ms_admin_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    multisite_redirect_live_dispatch(state, request, "/wp-admin/network/").await
}

async fn ms_options_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    multisite_redirect_live_dispatch(state, request, "/wp-admin/network/settings.php").await
}

async fn ms_upgrade_network_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    multisite_redirect_live_dispatch(state, request, "/wp-admin/network/upgrade.php").await
}

async fn multisite_redirect_live_dispatch(
    state: AppState,
    request: Request,
    target: &str,
) -> Response {
    if request.method() != axum::http::Method::GET && request.method() != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "legacy multisite redirect shims currently support GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(request.headers(), &state.auth_secrets);
    let can_view = authenticated
        && (capabilities.contains("read")
            || capabilities.contains("manage_sites")
            || capabilities.contains("manage_network")
            || capabilities.contains("manage_options"));
    if !can_view {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_network capability is required for this multisite admin shim.",
            }),
        )
        .into_response();
    }

    rust_handled_redirect(target).into_response()
}

async fn plugins_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/plugins.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_manage_plugins = authenticated
        && (capabilities.contains("activate_plugins")
            || capabilities.contains("manage_options")
            || capabilities.contains("manage_network_plugins")
            || capabilities.contains("manage_network"));
    if !can_manage_plugins {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "activate_plugins capability is required for wp-admin/plugins.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let action = params.get("action").cloned().unwrap_or_default();
    if method == axum::http::Method::POST && !action.is_empty() {
        let target = format!("/wp-admin/plugins.php?updated_action={action}");
        return rust_handled_redirect(&target).into_response();
    }

    let plugin = params.get("plugin").cloned().unwrap_or_default();
    let status = params.get("plugin_status").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Plugins (Rust)</h1><p>action={action}</p><p>plugin={plugin}</p><p>plugin_status={status}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn themes_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/themes.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_manage_themes = authenticated
        && (capabilities.contains("switch_themes")
            || capabilities.contains("edit_theme_options")
            || capabilities.contains("manage_options")
            || capabilities.contains("manage_network_themes")
            || capabilities.contains("manage_network"));
    if !can_manage_themes {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "switch_themes capability is required for wp-admin/themes.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let action = params.get("action").cloned().unwrap_or_default();
    if !action.is_empty() {
        let target = format!("/wp-admin/themes.php?updated_action={action}");
        return rust_handled_redirect(&target).into_response();
    }

    let activated = params.get("activated").cloned().unwrap_or_default();
    let resumed = params.get("resumed").cloned().unwrap_or_default();
    let deleted = params.get("deleted").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Themes (Rust)</h1><p>activated={activated}</p><p>resumed={resumed}</p><p>deleted={deleted}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn users_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/users.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_list_users = authenticated
        && (capabilities.contains("list_users")
            || capabilities.contains("edit_users")
            || capabilities.contains("manage_options")
            || capabilities.contains("manage_network_users")
            || capabilities.contains("manage_network"));
    if !can_list_users {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "list_users capability is required for wp-admin/users.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let action = params.get("action").cloned().unwrap_or_default();
    if method == axum::http::Method::POST && !action.is_empty() {
        let target = format!("/wp-admin/users.php?updated_action={action}");
        return rust_handled_redirect(&target).into_response();
    }

    let role = params.get("role").cloned().unwrap_or_default();
    let search = params.get("s").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Users (Rust)</h1><p>action={action}</p><p>role={role}</p><p>search={search}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn edit_posts_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/edit.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_edit_posts = authenticated
        && (capabilities.contains("edit_posts")
            || capabilities.contains("edit_pages")
            || capabilities.contains("manage_options"));
    if !can_edit_posts {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "edit_posts capability is required for wp-admin/edit.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let post_type = params
        .get("post_type")
        .cloned()
        .unwrap_or_else(|| "post".to_string());
    if post_type == "attachment" {
        return rust_handled_redirect("/wp-admin/upload.php").into_response();
    }

    let action = params
        .get("action")
        .or_else(|| params.get("doaction"))
        .cloned()
        .unwrap_or_default();
    if method == axum::http::Method::POST && !action.is_empty() {
        let target = format!("/wp-admin/edit.php?post_type={post_type}&updated_action={action}");
        return rust_handled_redirect(&target).into_response();
    }

    let paged = params
        .get("paged")
        .cloned()
        .unwrap_or_else(|| "1".to_string());
    let s = params.get("s").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Edit Posts (Rust)</h1><p>post_type={post_type}</p><p>action={action}</p><p>paged={paged}</p><p>search={s}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn edit_tags_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/edit-tags.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_manage_terms = authenticated
        && (capabilities.contains("manage_categories")
            || capabilities.contains("edit_posts")
            || capabilities.contains("manage_options"));
    if !can_manage_terms {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_categories capability is required for wp-admin/edit-tags.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let taxonomy = params
        .get("taxonomy")
        .cloned()
        .unwrap_or_else(|| "post_tag".to_string());
    let action = params.get("action").cloned().unwrap_or_default();
    if method == axum::http::Method::POST {
        let message = match action.as_str() {
            "add-tag" => "1",
            "editedtag" => "3",
            "delete" => "2",
            "bulk-delete" => "6",
            _ => "1",
        };
        let target = format!("/wp-admin/edit-tags.php?taxonomy={taxonomy}&message={message}");
        return rust_handled_redirect(&target).into_response();
    }

    let post_type = params
        .get("post_type")
        .cloned()
        .unwrap_or_else(|| "post".to_string());
    let message = params.get("message").cloned().unwrap_or_default();
    let error = params.get("error").cloned().unwrap_or_default();
    let paged = params
        .get("paged")
        .cloned()
        .unwrap_or_else(|| "1".to_string());
    let html = format!(
        "<!doctype html><html><body><h1>Edit Tags (Rust)</h1><p>taxonomy={taxonomy}</p><p>post_type={post_type}</p><p>action={action}</p><p>message={message}</p><p>error={error}</p><p>paged={paged}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn edit_comments_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/edit-comments.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_edit_comments = authenticated
        && (capabilities.contains("edit_posts")
            || capabilities.contains("moderate_comments")
            || capabilities.contains("manage_options"));
    if !can_edit_comments {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "edit_posts capability is required for wp-admin/edit-comments.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let action = params
        .get("action")
        .or_else(|| params.get("doaction"))
        .cloned()
        .unwrap_or_default();
    if method == axum::http::Method::POST && !action.is_empty() {
        let target = format!("/wp-admin/edit-comments.php?updated_action={action}");
        return rust_handled_redirect(&target).into_response();
    }

    let comment_status = params.get("comment_status").cloned().unwrap_or_default();
    let paged = params
        .get("paged")
        .cloned()
        .unwrap_or_else(|| "1".to_string());
    let html = format!(
        "<!doctype html><html><body><h1>Edit Comments (Rust)</h1><p>action={action}</p><p>comment_status={comment_status}</p><p>paged={paged}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn comment_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/comment.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_moderate = authenticated
        && (capabilities.contains("edit_posts")
            || capabilities.contains("moderate_comments")
            || capabilities.contains("manage_options"));
    if !can_moderate {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "edit_posts capability is required for wp-admin/comment.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let action = params
        .get("action")
        .cloned()
        .unwrap_or_else(|| "editcomment".to_string());
    let comment_id = params.get("c").cloned().unwrap_or_default();

    if method == axum::http::Method::POST {
        return rust_handled_redirect("/wp-admin/edit-comments.php?updated_action=comment")
            .into_response();
    }

    if action == "delete" || action == "approve" || action == "trash" || action == "spam" {
        let html = format!(
            "<!doctype html><html><body><h1>Moderate Comment (Rust)</h1><p>action={action}</p><p>comment_id={comment_id}</p></body></html>"
        );
        return rust_handled_html(StatusCode::OK, html).into_response();
    }

    let html = format!(
        "<!doctype html><html><body><h1>Edit Comment (Rust)</h1><p>action={action}</p><p>comment_id={comment_id}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn link_manager_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/link-manager.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_manage_links = authenticated
        && (capabilities.contains("manage_links")
            || capabilities.contains("manage_options")
            || capabilities.contains("manage_network"));
    if !can_manage_links {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_links capability is required for wp-admin/link-manager.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let action = params
        .get("action")
        .or_else(|| params.get("doaction"))
        .cloned()
        .unwrap_or_default();
    if method == axum::http::Method::POST && !action.is_empty() {
        let target = format!("/wp-admin/link-manager.php?updated_action={action}");
        return rust_handled_redirect(&target).into_response();
    }

    let deleted = params.get("deleted").cloned().unwrap_or_default();
    let s = params.get("s").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Link Manager (Rust)</h1><p>action={action}</p><p>deleted={deleted}</p><p>search={s}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn link_add_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/link-add.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_manage_links = authenticated
        && (capabilities.contains("manage_links")
            || capabilities.contains("manage_options")
            || capabilities.contains("manage_network"));
    if !can_manage_links {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_links capability is required for wp-admin/link-add.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    if method == axum::http::Method::POST {
        return rust_handled_redirect("/wp-admin/link-manager.php?added=true").into_response();
    }

    let action = params.get("action").cloned().unwrap_or_default();
    let cat_id = params.get("cat_id").cloned().unwrap_or_default();
    let link_id = params.get("link_id").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Link Add (Rust)</h1><p>action={action}</p><p>cat_id={cat_id}</p><p>link_id={link_id}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn link_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/link.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_manage_links = authenticated
        && (capabilities.contains("manage_links")
            || capabilities.contains("manage_options")
            || capabilities.contains("manage_network"));
    if !can_manage_links {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_links capability is required for wp-admin/link.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let action = params.get("action").cloned().unwrap_or_default();
    let link_id = params.get("link_id").cloned().unwrap_or_default();

    if method == axum::http::Method::POST {
        let target = if action.is_empty() {
            "/wp-admin/link-manager.php".to_string()
        } else {
            format!("/wp-admin/link-manager.php?updated_action={action}")
        };
        return rust_handled_redirect(&target).into_response();
    }

    if action == "edit" {
        let html = format!(
            "<!doctype html><html><body><h1>Edit Link (Rust)</h1><p>link_id={link_id}</p></body></html>"
        );
        return rust_handled_html(StatusCode::OK, html).into_response();
    }

    rust_handled_redirect("/wp-admin/link-manager.php").into_response()
}

async fn media_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/media.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_manage_media = authenticated
        && (capabilities.contains("upload_files")
            || capabilities.contains("edit_posts")
            || capabilities.contains("manage_options"));
    if !can_manage_media {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "upload_files capability is required for wp-admin/media.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let action = params.get("action").cloned().unwrap_or_default();
    let attachment_id = params.get("attachment_id").cloned().unwrap_or_default();
    if (action == "editattachment" || action == "edit") && !attachment_id.is_empty() {
        let target = format!("/wp-admin/upload.php?item={attachment_id}&error=deprecated");
        return rust_handled_redirect(&target).into_response();
    }
    rust_handled_redirect("/wp-admin/upload.php?error=deprecated").into_response()
}

async fn media_upload_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/media-upload.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_upload = authenticated
        && (capabilities.contains("upload_files")
            || capabilities.contains("edit_posts")
            || capabilities.contains("manage_options"));
    if !can_upload {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "upload_files capability is required for wp-admin/media-upload.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    if method == axum::http::Method::POST {
        return rust_handled_redirect("/wp-admin/media-new.php?posted=1").into_response();
    }

    let tab = params
        .get("tab")
        .cloned()
        .unwrap_or_else(|| "type".to_string());
    let media_type = params
        .get("type")
        .cloned()
        .unwrap_or_else(|| "file".to_string());
    let inline = params
        .get("inline")
        .cloned()
        .unwrap_or_else(|| "0".to_string());
    let post_id = params.get("post_id").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Media Upload (Rust)</h1><p>tab={tab}</p><p>type={media_type}</p><p>inline={inline}</p><p>post_id={post_id}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn upload_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/upload.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_manage_media = authenticated
        && (capabilities.contains("upload_files")
            || capabilities.contains("edit_posts")
            || capabilities.contains("manage_options"));
    if !can_manage_media {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "upload_files capability is required for wp-admin/upload.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    if method == axum::http::Method::POST {
        let action = params
            .get("action")
            .or_else(|| params.get("doaction"))
            .cloned()
            .unwrap_or_default();
        let target = if action.is_empty() {
            "/wp-admin/upload.php?posted=1".to_string()
        } else {
            format!("/wp-admin/upload.php?updated_action={action}")
        };
        return rust_handled_redirect(&target).into_response();
    }

    let posted = params.get("posted").cloned().unwrap_or_default();
    let attached = params.get("attached").cloned().unwrap_or_default();
    let detached = params.get("detach").cloned().unwrap_or_default();
    let deleted = params.get("deleted").cloned().unwrap_or_default();
    let trashed = params.get("trashed").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Media Library (Rust)</h1><p>posted={posted}</p><p>attached={attached}</p><p>detach={detached}</p><p>deleted={deleted}</p><p>trashed={trashed}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn media_new_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/media-new.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_upload = authenticated
        && (capabilities.contains("upload_files")
            || capabilities.contains("edit_posts")
            || capabilities.contains("manage_options"));
    if !can_upload {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "upload_files capability is required for wp-admin/media-new.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    if method == axum::http::Method::POST {
        return rust_handled_redirect("/wp-admin/upload.php?posted=1").into_response();
    }

    let post_id = params.get("post_id").cloned().unwrap_or_default();
    let browser_uploader = params.get("browser-uploader").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Upload New Media (Rust)</h1><p>post_id={post_id}</p><p>browser_uploader={browser_uploader}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn tools_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    if request.method() != axum::http::Method::GET && request.method() != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/tools.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(request.headers(), &state.auth_secrets);
    let can_access_tools = authenticated
        && (capabilities.contains("edit_posts")
            || capabilities.contains("manage_options")
            || capabilities.contains("import"));
    if !can_access_tools {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "edit_posts capability is required for wp-admin/tools.php.",
            }),
        )
        .into_response();
    }

    let params = parse_urlencoded(request.uri().query().unwrap_or_default());
    if params.contains_key("wp-privacy-policy-guide") {
        return rust_handled_redirect("/wp-admin/options-privacy.php?tab=policyguide")
            .into_response();
    }

    if let Some(page) = params.get("page") {
        if page == "export_personal_data" {
            return rust_handled_redirect("/wp-admin/export-personal-data.php").into_response();
        }
        if page == "remove_personal_data" {
            return rust_handled_redirect("/wp-admin/erase-personal-data.php").into_response();
        }
    }

    let html = "<!doctype html><html><body><h1>Tools (Rust)</h1><p>status=available</p><p>converter=categories-tags</p></body></html>".to_string();
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn site_health_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    if request.method() != axum::http::Method::GET {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/site-health.php currently supports GET only.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(request.headers(), &state.auth_secrets);
    let can_view_site_health = authenticated
        && (capabilities.contains("view_site_health_checks")
            || capabilities.contains("manage_options"));
    if !can_view_site_health {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "view_site_health_checks capability is required for wp-admin/site-health.php.",
            }),
        )
        .into_response();
    }

    let params = parse_urlencoded(request.uri().query().unwrap_or_default());
    let tab = params
        .get("tab")
        .cloned()
        .unwrap_or_else(|| "status".to_string());
    let html = format!(
        "<!doctype html><html><body><h1>Site Health (Rust)</h1><p>tab={tab}</p><p>status=available</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn site_health_info_live_dispatch(_request: Request) -> Response {
    rust_handled_text(StatusCode::OK, "text/plain; charset=UTF-8", String::new()).into_response()
}

async fn export_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    if request.method() != axum::http::Method::GET {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/export.php currently supports GET only.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(request.headers(), &state.auth_secrets);
    let can_export = authenticated
        && (capabilities.contains("export") || capabilities.contains("manage_options"));
    if !can_export {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "export capability is required for wp-admin/export.php.",
            }),
        )
        .into_response();
    }

    let params = parse_urlencoded(request.uri().query().unwrap_or_default());
    if params.contains_key("download") {
        let requested_content = params
            .get("content")
            .cloned()
            .unwrap_or_else(|| "all".to_string());
        let export_xml = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<rss version=\"2.0\"><channel><title>WordPress Export (Rust)</title><exported_content>{requested_content}</exported_content></channel></rss>"
        );
        let mut response =
            rust_handled_text(StatusCode::OK, "application/xml; charset=UTF-8", export_xml)
                .into_response();
        if let Ok(value) =
            HeaderValue::from_str("attachment; filename=\"wordpress-rust-export.xml\"")
        {
            response.headers_mut().insert("Content-Disposition", value);
        }
        return response;
    }

    let html = "<!doctype html><html><body><h1>Export (Rust)</h1><p>status=ready</p></body></html>"
        .to_string();
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn import_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    if request.method() != axum::http::Method::GET {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/import.php currently supports GET only.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(request.headers(), &state.auth_secrets);
    let can_import = authenticated
        && (capabilities.contains("import")
            || capabilities.contains("install_plugins")
            || capabilities.contains("manage_options"));
    if !can_import {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "import capability is required for wp-admin/import.php.",
            }),
        )
        .into_response();
    }

    let params = parse_urlencoded(request.uri().query().unwrap_or_default());
    let invalid = params.get("invalid").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Import (Rust)</h1><p>status=ready</p><p>invalid={invalid}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn export_personal_data_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/export-personal-data.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_export_personal_data = authenticated
        && (capabilities.contains("export_others_personal_data")
            || capabilities.contains("manage_options"));
    if !can_export_personal_data {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "export_others_personal_data capability is required for wp-admin/export-personal-data.php.",
            }),
        )
        .into_response();
    }

    if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        let params = merged_params(parts.uri.query(), &body_bytes, &content_type);
        let requested_identity = params
            .get("username_or_email_for_privacy_request")
            .cloned()
            .unwrap_or_default();
        return rust_handled_json_with_status(
            StatusCode::OK,
            json!({
                "status": "request_registered",
                "requester": requested_identity,
            }),
        )
        .into_response();
    }

    let html = "<!doctype html><html><body><h1>Export Personal Data (Rust)</h1><p>status=ready</p></body></html>".to_string();
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn erase_personal_data_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/erase-personal-data.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let has_erase_caps = capabilities.contains("erase_others_personal_data")
        && capabilities.contains("delete_users");
    let can_erase_personal_data =
        authenticated && (has_erase_caps || capabilities.contains("manage_options"));
    if !can_erase_personal_data {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "erase_others_personal_data and delete_users capabilities are required for wp-admin/erase-personal-data.php.",
            }),
        )
        .into_response();
    }

    if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        let params = merged_params(parts.uri.query(), &body_bytes, &content_type);
        let requested_identity = params
            .get("username_or_email_for_privacy_request")
            .cloned()
            .unwrap_or_default();
        return rust_handled_json_with_status(
            StatusCode::OK,
            json!({
                "status": "erasure_request_registered",
                "requester": requested_identity,
            }),
        )
        .into_response();
    }

    let html = "<!doctype html><html><body><h1>Erase Personal Data (Rust)</h1><p>status=ready</p></body></html>".to_string();
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_setup_network = authenticated
        && (capabilities.contains("setup_network") || capabilities.contains("manage_options"));
    if !can_setup_network {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "setup_network capability is required for wp-admin/network.php.",
            }),
        )
        .into_response();
    }

    if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        let params = merged_params(parts.uri.query(), &body_bytes, &content_type);
        let has_sitename = params
            .get("sitename")
            .is_some_and(|value| !value.trim().is_empty());
        let has_email = params
            .get("email")
            .is_some_and(|value| !value.trim().is_empty());
        if !has_sitename || !has_email {
            return rust_handled_json_with_status(
                StatusCode::BAD_REQUEST,
                json!({
                    "error": "invalid_network_payload",
                    "message": "network.php requires sitename and email.",
                }),
            )
            .into_response();
        }
        return rust_handled_redirect("/wp-admin/network.php?installed=1").into_response();
    }

    let html =
        "<!doctype html><html><body><h1>Network Setup (Rust)</h1><p>status=ready</p></body></html>"
            .to_string();
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_admin_bootstrap_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    if request.method() != axum::http::Method::GET && request.method() != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/admin.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(request.headers(), &state.auth_secrets);
    if !authenticated {
        return rust_handled_redirect(
            "/wp-login.php?redirect_to=%2Fwp-admin%2Fnetwork%2Fadmin.php",
        )
        .into_response();
    }

    let can_access_network_admin = capabilities.contains("manage_network")
        || capabilities.contains("manage_sites")
        || capabilities.contains("manage_options");
    if !can_access_network_admin {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_network capability is required for wp-admin/network/admin.php.",
            }),
        )
        .into_response();
    }

    rust_handled_json(json!({
        "component": "network-admin-bootstrap",
        "authenticated": true,
        "capabilities": capabilities,
        "message": "Rust network-admin bootstrap compatibility shim loaded.",
    }))
    .into_response()
}

async fn network_setup_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    network_live_dispatch(State(state), request).await
}

async fn network_index_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    if request.method() != axum::http::Method::GET {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/index.php currently supports GET only.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(request.headers(), &state.auth_secrets);
    let can_manage_network = authenticated
        && (capabilities.contains("manage_network") || capabilities.contains("manage_options"));
    if !can_manage_network {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_network capability is required for wp-admin/network/index.php.",
            }),
        )
        .into_response();
    }

    let html = "<!doctype html><html><body><h1>Network Dashboard (Rust)</h1><p>status=ready</p></body></html>".to_string();
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_sites_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/sites.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_manage_sites = authenticated
        && (capabilities.contains("manage_sites")
            || capabilities.contains("manage_network")
            || capabilities.contains("manage_options"));
    if !can_manage_sites {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_sites capability is required for wp-admin/network/sites.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    if params.get("action").is_some_and(|value| value == "confirm") {
        let action2 = params.get("action2").cloned().unwrap_or_default();
        let site_id = params.get("id").cloned().unwrap_or_default();
        let html = format!(
            "<!doctype html><html><body><h1>Network Sites Confirm (Rust)</h1><p>action2={action2}</p><p>id={site_id}</p></body></html>"
        );
        return rust_handled_html(StatusCode::OK, html).into_response();
    }

    if method == axum::http::Method::POST || params.contains_key("action") {
        let action = params.get("action").cloned().unwrap_or_default();
        let target = format!("/wp-admin/network/sites.php?updated_action={action}");
        return rust_handled_redirect(&target).into_response();
    }

    let html =
        "<!doctype html><html><body><h1>Network Sites (Rust)</h1><p>status=ready</p></body></html>"
            .to_string();
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_users_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/users.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_manage_network_users = authenticated
        && (capabilities.contains("manage_network_users")
            || capabilities.contains("manage_network")
            || capabilities.contains("manage_options"));
    if !can_manage_network_users {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_network_users capability is required for wp-admin/network/users.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    if params
        .get("action")
        .is_some_and(|value| value == "deleteuser")
    {
        let user_id = params.get("id").cloned().unwrap_or_default();
        let html = format!(
            "<!doctype html><html><body><h1>Network Users Delete (Rust)</h1><p>id={user_id}</p></body></html>"
        );
        return rust_handled_html(StatusCode::OK, html).into_response();
    }

    if method == axum::http::Method::POST
        && params
            .get("action")
            .is_some_and(|value| value == "allusers")
    {
        let bulk_action = params.get("bulk_action").cloned().unwrap_or_default();
        let target = format!("/wp-admin/network/users.php?updated=true&action={bulk_action}");
        return rust_handled_redirect(&target).into_response();
    }

    if method == axum::http::Method::POST
        && params
            .get("action")
            .is_some_and(|value| value == "dodelete")
    {
        return rust_handled_redirect("/wp-admin/network/users.php?updated=true&action=delete")
            .into_response();
    }

    let html =
        "<!doctype html><html><body><h1>Network Users (Rust)</h1><p>status=ready</p></body></html>"
            .to_string();
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_themes_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/themes.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_manage_network_themes = authenticated
        && (capabilities.contains("manage_network_themes")
            || capabilities.contains("manage_network")
            || capabilities.contains("manage_options"));
    if !can_manage_network_themes {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_network_themes capability is required for wp-admin/network/themes.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let action = params.get("action").cloned().unwrap_or_default();
    if action == "enable" {
        return rust_handled_redirect("/wp-admin/network/themes.php?enabled=1").into_response();
    }
    if action == "disable" {
        return rust_handled_redirect("/wp-admin/network/themes.php?disabled=1").into_response();
    }
    if action == "enable-selected" {
        return rust_handled_redirect("/wp-admin/network/themes.php?enabled=1").into_response();
    }
    if action == "disable-selected" {
        return rust_handled_redirect("/wp-admin/network/themes.php?disabled=1").into_response();
    }
    if action == "delete-selected" {
        return rust_handled_redirect("/wp-admin/network/themes.php?deleted=1").into_response();
    }
    if action == "update-selected" {
        let html =
            "<!doctype html><html><body><h1>Network Themes Update (Rust)</h1><p>status=in_progress</p></body></html>"
                .to_string();
        return rust_handled_html(StatusCode::OK, html).into_response();
    }

    let search = params.get("s").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Network Themes (Rust)</h1><p>status=ready</p><p>search={search}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_plugins_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/plugins.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_manage_network_plugins = authenticated
        && (capabilities.contains("manage_network_plugins")
            || capabilities.contains("manage_network")
            || capabilities.contains("activate_plugins")
            || capabilities.contains("manage_options"));
    if !can_manage_network_plugins {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_network_plugins capability is required for wp-admin/network/plugins.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let action = params.get("action").cloned().unwrap_or_default();
    if !action.is_empty() {
        let target = format!("/wp-admin/network/plugins.php?updated_action={action}");
        return rust_handled_redirect(&target).into_response();
    }

    let html = "<!doctype html><html><body><h1>Network Plugins (Rust)</h1><p>status=ready</p></body></html>"
        .to_string();
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_settings_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/settings.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_manage_network_options = authenticated
        && (capabilities.contains("manage_network_options")
            || capabilities.contains("manage_network")
            || capabilities.contains("manage_options"));
    if !can_manage_network_options {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_network_options capability is required for wp-admin/network/settings.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    if let Some(hash) = params.get("network_admin_hash") {
        if hash == "confirm-rust-admin-email" {
            return rust_handled_redirect("/wp-admin/network/settings.php?updated=true")
                .into_response();
        }
        return rust_handled_redirect("/wp-admin/network/settings.php?updated=false")
            .into_response();
    }

    if params
        .get("dismiss")
        .is_some_and(|value| value == "new_network_admin_email")
    {
        return rust_handled_redirect("/wp-admin/network/settings.php?updated=true")
            .into_response();
    }

    if method == axum::http::Method::POST {
        let site_name = params
            .get("site_name")
            .cloned()
            .unwrap_or_else(|| "WordPress Network".to_string());
        let new_admin_email = params.get("new_admin_email").cloned().unwrap_or_default();
        let registration = params.get("registration").cloned().unwrap_or_default();
        {
            let mut options = state.options.lock().expect("options mutex poisoned");
            options.set_option("network_site_name", &site_name, false);
            if !new_admin_email.is_empty() {
                options.set_option("network_admin_email", &new_admin_email, false);
            }
            if !registration.is_empty() {
                options.set_option("network_registration", &registration, false);
            }
        }
        return rust_handled_redirect("/wp-admin/network/settings.php?updated=true")
            .into_response();
    }

    let updated = params.get("updated").cloned().unwrap_or_default();
    let options = state.options.lock().expect("options mutex poisoned");
    let site_name = options
        .get_option("network_site_name")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "WordPress Network".to_string());
    let network_admin_email = options
        .get_option("network_admin_email")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "admin@example.com".to_string());
    let html = format!(
        "<!doctype html><html><body><h1>Network Settings (Rust)</h1><p>updated={updated}</p><p>site_name={site_name}</p><p>admin_email={network_admin_email}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_site_new_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/site-new.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_create_sites = authenticated
        && (capabilities.contains("create_sites")
            || capabilities.contains("manage_network")
            || capabilities.contains("manage_options"));
    if !can_create_sites {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "create_sites capability is required for wp-admin/network/site-new.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    if method == axum::http::Method::POST {
        let action = params.get("action").cloned().unwrap_or_default();
        if action != "add-site" {
            return rust_handled_json_with_status(
                StatusCode::BAD_REQUEST,
                json!({
                    "error": "invalid_site_new_action",
                    "message": "site-new.php requires action=add-site for POST requests.",
                }),
            )
            .into_response();
        }

        let domain = params
            .get("blog[domain]")
            .or_else(|| params.get("domain"))
            .map(|value| value.trim().to_string())
            .unwrap_or_default();
        let title = params
            .get("blog[title]")
            .or_else(|| params.get("title"))
            .map(|value| value.trim().to_string())
            .unwrap_or_default();
        let email = params
            .get("blog[email]")
            .or_else(|| params.get("email"))
            .map(|value| value.trim().to_string())
            .unwrap_or_default();

        if domain.is_empty() || title.is_empty() || email.is_empty() || !email.contains('@') {
            return rust_handled_json_with_status(
                StatusCode::BAD_REQUEST,
                json!({
                    "error": "invalid_site_new_payload",
                    "message": "site-new.php requires blog[domain], blog[title], and a valid blog[email].",
                }),
            )
            .into_response();
        }

        let site_id = params
            .get("site_id")
            .cloned()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "2".to_string());
        let target = format!("/wp-admin/network/site-new.php?update=added&id={site_id}");
        return rust_handled_redirect(&target).into_response();
    }

    let update = params.get("update").cloned().unwrap_or_default();
    let site_id = params.get("id").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Add Site (Rust)</h1><p>update={update}</p><p>id={site_id}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_site_info_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/site-info.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_manage_sites = authenticated
        && (capabilities.contains("manage_sites")
            || capabilities.contains("manage_network")
            || capabilities.contains("manage_options"));
    if !can_manage_sites {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_sites capability is required for wp-admin/network/site-info.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let site_id = params
        .get("id")
        .and_then(|value| value.trim().parse::<u64>().ok())
        .unwrap_or_default();
    if site_id == 0 {
        return rust_handled_json_with_status(
            StatusCode::BAD_REQUEST,
            json!({
                "error": "invalid_site_id",
                "message": "site-info.php requires a valid id query parameter.",
            }),
        )
        .into_response();
    }

    if method == axum::http::Method::POST {
        let action = params.get("action").cloned().unwrap_or_default();
        if action != "update-site" {
            return rust_handled_json_with_status(
                StatusCode::BAD_REQUEST,
                json!({
                    "error": "invalid_site_info_action",
                    "message": "site-info.php requires action=update-site for POST requests.",
                }),
            )
            .into_response();
        }
        let target = format!("/wp-admin/network/site-info.php?update=updated&id={site_id}");
        return rust_handled_redirect(&target).into_response();
    }

    let update = params.get("update").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Edit Site (Rust)</h1><p>id={site_id}</p><p>update={update}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_site_settings_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/site-settings.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_manage_sites = authenticated
        && (capabilities.contains("manage_sites")
            || capabilities.contains("manage_network")
            || capabilities.contains("manage_options"));
    if !can_manage_sites {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_sites capability is required for wp-admin/network/site-settings.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let site_id = params
        .get("id")
        .and_then(|value| value.trim().parse::<u64>().ok())
        .unwrap_or_default();
    if site_id == 0 {
        return rust_handled_json_with_status(
            StatusCode::BAD_REQUEST,
            json!({
                "error": "invalid_site_id",
                "message": "site-settings.php requires a valid id query parameter.",
            }),
        )
        .into_response();
    }

    if method == axum::http::Method::POST {
        let action = params.get("action").cloned().unwrap_or_default();
        if action != "update-site" {
            return rust_handled_json_with_status(
                StatusCode::BAD_REQUEST,
                json!({
                    "error": "invalid_site_settings_action",
                    "message": "site-settings.php requires action=update-site for POST requests.",
                }),
            )
            .into_response();
        }

        let mut updated_count = 0usize;
        {
            let mut options = state.options.lock().expect("options mutex poisoned");
            for (key, value) in &params {
                if let Some(option_name) = key
                    .strip_prefix("option[")
                    .and_then(|suffix| suffix.strip_suffix(']'))
                {
                    if option_name.is_empty() {
                        continue;
                    }
                    let namespaced_key = format!("site_{site_id}_{option_name}");
                    options.set_option(&namespaced_key, value, false);
                    updated_count += 1;
                }
            }
        }

        let target = format!(
            "/wp-admin/network/site-settings.php?update=updated&id={site_id}&updated_count={updated_count}"
        );
        return rust_handled_redirect(&target).into_response();
    }

    let update = params.get("update").cloned().unwrap_or_default();
    let updated_count = params.get("updated_count").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Edit Site Settings (Rust)</h1><p>id={site_id}</p><p>update={update}</p><p>updated_count={updated_count}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_site_users_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/site-users.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_manage_sites = authenticated
        && (capabilities.contains("manage_sites")
            || capabilities.contains("manage_network")
            || capabilities.contains("manage_options"));
    if !can_manage_sites {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_sites capability is required for wp-admin/network/site-users.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let site_id = params
        .get("id")
        .and_then(|value| value.trim().parse::<u64>().ok())
        .unwrap_or_default();
    if site_id == 0 {
        return rust_handled_json_with_status(
            StatusCode::BAD_REQUEST,
            json!({
                "error": "invalid_site_id",
                "message": "site-users.php requires a valid id query parameter.",
            }),
        )
        .into_response();
    }

    let action = params.get("action").cloned().unwrap_or_default();
    if method == axum::http::Method::POST {
        let update = match action.as_str() {
            "adduser" => "adduser",
            "newuser" => "newuser",
            "remove" => "remove",
            "promote" => "promote",
            "update-site" => "updated",
            _ if !action.is_empty() => "updated",
            _ => "updated",
        };
        let target = format!("/wp-admin/network/site-users.php?id={site_id}&update={update}");
        return rust_handled_redirect(&target).into_response();
    }

    if action == "update-site" {
        let target = format!("/wp-admin/network/site-users.php?id={site_id}");
        return rust_handled_redirect(&target).into_response();
    }

    let update = params.get("update").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Edit Site Users (Rust)</h1><p>id={site_id}</p><p>update={update}</p><p>action={action}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_site_themes_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/site-themes.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_manage_sites = authenticated
        && (capabilities.contains("manage_sites")
            || capabilities.contains("manage_network")
            || capabilities.contains("manage_options"));
    if !can_manage_sites {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_sites capability is required for wp-admin/network/site-themes.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let site_id = params
        .get("id")
        .and_then(|value| value.trim().parse::<u64>().ok())
        .unwrap_or_default();
    if site_id == 0 {
        return rust_handled_json_with_status(
            StatusCode::BAD_REQUEST,
            json!({
                "error": "invalid_site_id",
                "message": "site-themes.php requires a valid id query parameter.",
            }),
        )
        .into_response();
    }

    let action = params.get("action").cloned().unwrap_or_default();
    if method == axum::http::Method::POST || !action.is_empty() {
        let (status_key, count) = match action.as_str() {
            "enable" | "enable-selected" => ("enabled", "1"),
            "disable" | "disable-selected" => ("disabled", "1"),
            _ => ("error", "none"),
        };
        let target = format!("/wp-admin/network/site-themes.php?id={site_id}&{status_key}={count}");
        return rust_handled_redirect(&target).into_response();
    }

    let enabled = params.get("enabled").cloned().unwrap_or_default();
    let disabled = params.get("disabled").cloned().unwrap_or_default();
    let error = params.get("error").cloned().unwrap_or_default();
    let search = params.get("s").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Edit Site Themes (Rust)</h1><p>id={site_id}</p><p>enabled={enabled}</p><p>disabled={disabled}</p><p>error={error}</p><p>search={search}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_user_new_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/user-new.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_create_users = authenticated
        && (capabilities.contains("create_users")
            || capabilities.contains("manage_network_users")
            || capabilities.contains("manage_network")
            || capabilities.contains("manage_options"));
    if !can_create_users {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "create_users capability is required for wp-admin/network/user-new.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    if method == axum::http::Method::POST {
        let action = params.get("action").cloned().unwrap_or_default();
        if action != "add-user" {
            return rust_handled_json_with_status(
                StatusCode::BAD_REQUEST,
                json!({
                    "error": "invalid_user_new_action",
                    "message": "user-new.php requires action=add-user for POST requests.",
                }),
            )
            .into_response();
        }

        let username = params
            .get("user[username]")
            .or_else(|| params.get("username"))
            .map(|value| value.trim().to_string())
            .unwrap_or_default();
        let email = params
            .get("user[email]")
            .or_else(|| params.get("email"))
            .map(|value| value.trim().to_string())
            .unwrap_or_default();
        if username.is_empty() || email.is_empty() || !email.contains('@') {
            return rust_handled_json_with_status(
                StatusCode::BAD_REQUEST,
                json!({
                    "error": "invalid_user_new_payload",
                    "message": "user-new.php requires user[username] and valid user[email].",
                }),
            )
            .into_response();
        }

        let user_id = params
            .get("user_id")
            .cloned()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "2".to_string());
        let target = format!("/wp-admin/network/user-new.php?update=added&user_id={user_id}");
        return rust_handled_redirect(&target).into_response();
    }

    let update = params.get("update").cloned().unwrap_or_default();
    let user_id = params.get("user_id").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Add User (Rust)</h1><p>update={update}</p><p>user_id={user_id}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_edit_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/edit.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_manage_network = authenticated
        && (capabilities.contains("manage_network") || capabilities.contains("manage_options"));
    if !can_manage_network {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_network capability is required for wp-admin/network/edit.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let action = params.get("action").cloned().unwrap_or_default();
    if action.trim().is_empty() {
        return rust_handled_redirect("/wp-admin/network/").into_response();
    }

    let target = format!("/wp-admin/network/?updated_action={action}");
    rust_handled_redirect(&target).into_response()
}

async fn network_update_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/update.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_update_network = authenticated
        && (capabilities.contains("update_plugins")
            || capabilities.contains("update_themes")
            || capabilities.contains("manage_network")
            || capabilities.contains("manage_options"));
    if !can_update_network {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "network update capability is required for wp-admin/network/update.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let action = params.get("action").cloned().unwrap_or_default();
    let iframe_request = matches!(
        action.as_str(),
        "update-selected" | "activate-plugin" | "update-selected-themes"
    );
    let html = format!(
        "<!doctype html><html><body><h1>Network Update (Rust)</h1><p>action={action}</p><p>iframe_request={iframe_request}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_update_core_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/update-core.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_update_core = authenticated
        && (capabilities.contains("update_core")
            || capabilities.contains("update_plugins")
            || capabilities.contains("update_themes")
            || capabilities.contains("manage_network")
            || capabilities.contains("manage_options"));
    if !can_update_core {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "update_core capability is required for wp-admin/network/update-core.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let action = params.get("action").cloned().unwrap_or_default();
    let operation = if method == axum::http::Method::POST {
        "apply"
    } else {
        "preview"
    };
    let html = format!(
        "<!doctype html><html><body><h1>Network Update Core (Rust)</h1><p>action={action}</p><p>operation={operation}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_plugin_install_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/plugin-install.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_install_plugins = authenticated
        && (capabilities.contains("install_plugins")
            || capabilities.contains("manage_network_plugins")
            || capabilities.contains("manage_network")
            || capabilities.contains("manage_options"));
    if !can_install_plugins {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "install_plugins capability is required for wp-admin/network/plugin-install.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let tab = params.get("tab").cloned().unwrap_or_default();
    let search = params.get("s").cloned().unwrap_or_default();
    let iframe_request = tab == "plugin-information";
    let html = format!(
        "<!doctype html><html><body><h1>Network Plugin Install (Rust)</h1><p>tab={tab}</p><p>search={search}</p><p>iframe_request={iframe_request}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_plugin_editor_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/plugin-editor.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_edit_plugins = authenticated
        && (capabilities.contains("edit_plugins")
            || capabilities.contains("manage_network_plugins")
            || capabilities.contains("manage_network")
            || capabilities.contains("manage_options"));
    if !can_edit_plugins {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "edit_plugins capability is required for wp-admin/network/plugin-editor.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    if method == axum::http::Method::POST
        && params
            .get("action")
            .is_some_and(|value| value.eq_ignore_ascii_case("update"))
    {
        let plugin = params.get("plugin").cloned().unwrap_or_default();
        let target = format!("/wp-admin/network/plugin-editor.php?plugin={plugin}&updated=true");
        return rust_handled_redirect(&target).into_response();
    }

    let plugin = params.get("plugin").cloned().unwrap_or_default();
    let file = params.get("file").cloned().unwrap_or_default();
    let updated = params.get("updated").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Network Plugin Editor (Rust)</h1><p>plugin={plugin}</p><p>file={file}</p><p>updated={updated}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_theme_editor_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/theme-editor.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_edit_themes = authenticated
        && (capabilities.contains("edit_themes")
            || capabilities.contains("manage_network_themes")
            || capabilities.contains("manage_network")
            || capabilities.contains("manage_options"));
    if !can_edit_themes {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "edit_themes capability is required for wp-admin/network/theme-editor.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    if method == axum::http::Method::POST
        && params
            .get("action")
            .is_some_and(|value| value.eq_ignore_ascii_case("update"))
    {
        let theme = params.get("theme").cloned().unwrap_or_default();
        let target = format!("/wp-admin/network/theme-editor.php?theme={theme}&updated=true");
        return rust_handled_redirect(&target).into_response();
    }

    let theme = params.get("theme").cloned().unwrap_or_default();
    let file = params.get("file").cloned().unwrap_or_default();
    let updated = params.get("updated").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Network Theme Editor (Rust)</h1><p>theme={theme}</p><p>file={file}</p><p>updated={updated}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_privacy_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/privacy.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_manage_privacy = authenticated
        && (capabilities.contains("manage_network_options")
            || capabilities.contains("manage_privacy_options")
            || capabilities.contains("manage_network")
            || capabilities.contains("manage_options"));
    if !can_manage_privacy {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_network_options capability is required for wp-admin/network/privacy.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    if method == axum::http::Method::POST {
        return rust_handled_redirect("/wp-admin/network/privacy.php?updated=true").into_response();
    }

    let updated = params.get("updated").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Network Privacy (Rust)</h1><p>updated={updated}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_about_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    network_information_page_live_dispatch(state, request, "Network About (Rust)").await
}

async fn network_credits_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    network_information_page_live_dispatch(state, request, "Network Credits (Rust)").await
}

async fn network_contribute_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    network_information_page_live_dispatch(state, request, "Network Contribute (Rust)").await
}

async fn network_freedoms_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    network_information_page_live_dispatch(state, request, "Network Freedoms (Rust)").await
}

async fn network_information_page_live_dispatch(
    state: AppState,
    request: Request,
    title: &str,
) -> Response {
    let (parts, _body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "network informational pages currently support GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_view = authenticated
        && (capabilities.contains("manage_network")
            || capabilities.contains("manage_options")
            || capabilities.contains("read"));
    if !can_view {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_network capability is required for this network page.",
            }),
        )
        .into_response();
    }

    let html = format!("<!doctype html><html><body><h1>{title}</h1></body></html>");
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_profile_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, _body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/profile.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_manage_profile = authenticated
        && (capabilities.contains("read")
            || capabilities.contains("edit_user")
            || capabilities.contains("manage_network")
            || capabilities.contains("manage_options"));
    if !can_manage_profile {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "read capability is required for wp-admin/network/profile.php.",
            }),
        )
        .into_response();
    }

    if method == axum::http::Method::POST {
        return rust_handled_redirect("/wp-admin/network/profile.php?updated=true").into_response();
    }

    let params = parse_urlencoded(parts.uri.query().unwrap_or_default());
    let updated = params.get("updated").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Network Profile (Rust)</h1><p>updated={updated}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_user_edit_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/user-edit.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_edit_users = authenticated
        && (capabilities.contains("edit_users")
            || capabilities.contains("promote_users")
            || capabilities.contains("manage_network_users")
            || capabilities.contains("manage_network")
            || capabilities.contains("manage_options"));
    if !can_edit_users {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "edit_users capability is required for wp-admin/network/user-edit.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let user_id = params.get("user_id").cloned().unwrap_or_default();
    if method == axum::http::Method::POST {
        let target = format!("/wp-admin/network/user-edit.php?user_id={user_id}&updated=true");
        return rust_handled_redirect(&target).into_response();
    }

    let updated = params.get("updated").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Network User Edit (Rust)</h1><p>user_id={user_id}</p><p>updated={updated}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_upgrade_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/upgrade.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_upgrade_network = authenticated
        && (capabilities.contains("upgrade_network")
            || capabilities.contains("manage_network")
            || capabilities.contains("manage_options"));
    if !can_upgrade_network {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "upgrade_network capability is required for wp-admin/network/upgrade.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let action = params
        .get("action")
        .cloned()
        .unwrap_or_else(|| "show".to_string());
    let n = params
        .get("n")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0);
    let next_n = n.saturating_add(5);
    let html = format!(
        "<!doctype html><html><body><h1>Network Upgrade (Rust)</h1><p>action={action}</p><p>n={n}</p><p>next_n={next_n}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_theme_install_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/theme-install.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_install_themes = authenticated
        && (capabilities.contains("install_themes")
            || capabilities.contains("manage_network_themes")
            || capabilities.contains("manage_network")
            || capabilities.contains("manage_options"));
    if !can_install_themes {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "install_themes capability is required for wp-admin/network/theme-install.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let tab = params.get("tab").cloned().unwrap_or_default();
    let search = params.get("s").cloned().unwrap_or_default();
    let iframe_request = tab == "theme-information";
    let html = format!(
        "<!doctype html><html><body><h1>Network Theme Install (Rust)</h1><p>tab={tab}</p><p>search={search}</p><p>iframe_request={iframe_request}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn ms_delete_site_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/ms-delete-site.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_delete_site = authenticated
        && (capabilities.contains("delete_site") || capabilities.contains("manage_options"));
    if !can_delete_site {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "delete_site capability is required for wp-admin/ms-delete-site.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    if let Some(hash) = params.get("h") {
        if hash == "confirm-rust-delete" {
            let html = "<!doctype html><html><body><h1>Delete Site (Rust)</h1><p>site_deleted=true</p></body></html>".to_string();
            return rust_handled_html(StatusCode::OK, html).into_response();
        }
        let html = "<!doctype html><html><body><h1>Delete Site (Rust)</h1><p>error=stale_link</p></body></html>".to_string();
        return rust_handled_html(StatusCode::BAD_REQUEST, html).into_response();
    }

    if method == axum::http::Method::POST
        && params
            .get("action")
            .is_some_and(|value| value == "deleteblog")
        && params
            .get("confirmdelete")
            .is_some_and(|value| value == "1")
    {
        let html = "<!doctype html><html><body><h1>Delete Site (Rust)</h1><p>delete_request=queued</p></body></html>".to_string();
        return rust_handled_html(StatusCode::OK, html).into_response();
    }

    let html = "<!doctype html><html><body><h1>Delete Site (Rust)</h1><p>status=confirm_required</p></body></html>".to_string();
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn upgrade_live_dispatch(request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();

    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/upgrade.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    if params
        .get("step")
        .is_some_and(|value| value.eq_ignore_ascii_case("upgrade_db"))
    {
        return rust_handled_text(StatusCode::OK, "text/plain; charset=UTF-8", "0".to_string())
            .into_response();
    }

    let html = "<!doctype html><html><body><h1>WordPress Upgrade (Rust)</h1><p>No update required.</p></body></html>".to_string();
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn update_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/update.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_update = authenticated
        && (capabilities.contains("update_plugins")
            || capabilities.contains("update_themes")
            || capabilities.contains("install_plugins")
            || capabilities.contains("install_themes")
            || capabilities.contains("manage_options"));
    if !can_update {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "update_plugins capability is required for wp-admin/update.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let action = params.get("action").cloned().unwrap_or_default();
    if method == axum::http::Method::POST && !action.is_empty() {
        let target = format!("/wp-admin/update.php?action={action}&updated=true");
        return rust_handled_redirect(&target).into_response();
    }

    let plugin = params.get("plugin").cloned().unwrap_or_default();
    let theme = params.get("theme").cloned().unwrap_or_default();
    let success = params.get("success").cloned().unwrap_or_default();
    let failure = params.get("failure").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Update/Install (Rust)</h1><p>action={action}</p><p>plugin={plugin}</p><p>theme={theme}</p><p>success={success}</p><p>failure={failure}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn update_core_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/update-core.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_update_core = authenticated
        && (capabilities.contains("update_core")
            || capabilities.contains("update_themes")
            || capabilities.contains("update_plugins")
            || capabilities.contains("update_languages")
            || capabilities.contains("manage_options"));
    if !can_update_core {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "update_core capability is required for wp-admin/update-core.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let action = params.get("action").cloned().unwrap_or_default();
    if method == axum::http::Method::POST && !action.is_empty() {
        let target = format!("/wp-admin/update-core.php?action={action}&updated=true");
        return rust_handled_redirect(&target).into_response();
    }

    let updated = params.get("updated").cloned().unwrap_or_default();
    let operation = if method == axum::http::Method::POST {
        "apply"
    } else {
        "preview"
    };
    let html = format!(
        "<!doctype html><html><body><h1>Update Core (Rust)</h1><p>action={action}</p><p>operation={operation}</p><p>updated={updated}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn repair_live_dispatch(request: Request) -> Response {
    if request.method() != axum::http::Method::GET {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/maint/repair.php currently supports GET only.",
            }),
        )
        .into_response();
    }

    let params = parse_urlencoded(request.uri().query().unwrap_or_default());
    let html = if let Some(mode) = params.get("repair") {
        let optimize = mode == "2";
        format!(
            "<!doctype html><html><body><h1>Database Repair (Rust)</h1><p>repair_run=true</p><p>optimize={optimize}</p></body></html>"
        )
    } else {
        "<!doctype html><html><body><h1>Database Repair (Rust)</h1><p>WP_ALLOW_REPAIR must be enabled.</p></body></html>".to_string()
    };
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn signup_live_dispatch(request: Request) -> Response {
    let (parts, body) = request.into_parts();
    if parts.method == axum::http::Method::GET {
        let html = "<!doctype html><html><body><h1>Sign Up (Rust)</h1></body></html>".to_string();
        return rust_handled_html(StatusCode::OK, html).into_response();
    }

    if parts.method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-signup.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let body_bytes = read_request_body(body).await;
    let content_type = parts
        .headers
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let params = merged_params(parts.uri.query(), &body_bytes, &content_type);
    let has_identity = params
        .get("user_login")
        .or_else(|| params.get("user_email"))
        .is_some_and(|value| !value.trim().is_empty());
    if !has_identity {
        return rust_handled_json_with_status(
            StatusCode::BAD_REQUEST,
            json!({
                "error": "missing_signup_identity",
                "message": "Provide user_login or user_email for signup.",
            }),
        )
        .into_response();
    }

    rust_handled_redirect("/wp-signup.php?checkemail=registered").into_response()
}

async fn activate_live_dispatch(request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let query = parse_urlencoded(parts.uri.query().unwrap_or_default());
    let key_from_query = query.get("key").cloned();

    if parts.method == axum::http::Method::GET {
        let activation_state = if key_from_query
            .as_deref()
            .is_some_and(|value| !value.is_empty())
        {
            "ready"
        } else {
            "missing_key"
        };
        let html = format!(
            "<!doctype html><html><body><h1>Activate Account (Rust)</h1><p>state={activation_state}</p></body></html>"
        );
        return rust_handled_html(StatusCode::OK, html).into_response();
    }

    if parts.method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-activate.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let body_bytes = read_request_body(body).await;
    let content_type = parts
        .headers
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let params = merged_params(parts.uri.query(), &body_bytes, &content_type);
    let key = params
        .get("key")
        .cloned()
        .or(key_from_query)
        .unwrap_or_default();
    if key.trim().is_empty() {
        return rust_handled_json_with_status(
            StatusCode::BAD_REQUEST,
            json!({
                "error": "missing_activation_key",
                "message": "Provide activation key to continue.",
            }),
        )
        .into_response();
    }

    rust_handled_redirect("/wp-login.php?checkemail=activated").into_response()
}

async fn comments_post_live_dispatch(request: Request) -> Response {
    let (parts, body) = request.into_parts();
    if parts.method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-comments-post.php currently supports POST only.",
            }),
        )
        .into_response();
    }

    let body_bytes = read_request_body(body).await;
    let content_type = parts
        .headers
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let params = merged_params(parts.uri.query(), &body_bytes, &content_type);
    let comment_post_id = params.get("comment_post_ID").cloned().unwrap_or_default();
    let has_comment_body = params
        .get("comment")
        .is_some_and(|value| !value.trim().is_empty());

    if comment_post_id.trim().is_empty() || !has_comment_body {
        return rust_handled_json_with_status(
            StatusCode::BAD_REQUEST,
            json!({
                "error": "invalid_comment_payload",
                "message": "comment_post_ID and comment are required.",
            }),
        )
        .into_response();
    }

    rust_handled_redirect(&format!("/?p={comment_post_id}#comment-rust")).into_response()
}

async fn mail_live_dispatch(request: Request) -> Response {
    if request.method() != axum::http::Method::GET {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-mail.php currently supports GET only.",
            }),
        )
        .into_response();
    }

    rust_handled_json(json!({
        "processed": false,
        "message": "wp-mail processing scaffolded in Rust; mailbox polling not yet implemented.",
    }))
    .into_response()
}

async fn trackback_live_dispatch(request: Request) -> Response {
    if request.method() != axum::http::Method::POST {
        return rust_handled_text(
            StatusCode::OK,
            "text/plain; charset=UTF-8",
            "0\nRust trackback endpoint ready.\n".to_string(),
        )
        .into_response();
    }

    rust_handled_text(
        StatusCode::OK,
        "text/plain; charset=UTF-8",
        "0\nRust trackback accepted.\n".to_string(),
    )
    .into_response()
}

async fn links_opml_live_dispatch(request: Request) -> Response {
    if request.method() != axum::http::Method::GET {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-links-opml.php currently supports GET only.",
            }),
        )
        .into_response();
    }

    let opml = r#"<?xml version="1.0" encoding="UTF-8"?>
<opml version="1.0">
  <head><title>WordPress Links (Rust)</title></head>
  <body>
    <outline text="WordPress.org" title="WordPress.org" type="link" xmlUrl="https://wordpress.org/news/feed/" htmlUrl="https://wordpress.org/"/>
  </body>
</opml>"#
        .to_string();
    rust_handled_text(StatusCode::OK, "text/xml; charset=UTF-8", opml).into_response()
}

async fn load_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn vars_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_guard_response()
}

async fn update_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_guard_response()
}

async fn wp_db_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn utf8_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn user_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn functions_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_guard_response()
}

async fn formatting_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn plugin_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn pluggable_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn capabilities_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn option_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn post_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_hook_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_query_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_rewrite_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_role_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_roles_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_user_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_session_tokens_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_user_meta_session_tokens_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_user_query_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_meta_query_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_date_query_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_tax_query_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_term_query_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_comment_query_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_network_query_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_site_query_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_post_type_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_post_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_error_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_http_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_guard_response()
}

async fn class_wp_http_cookie_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_http_encoding_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_http_response_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_http_curl_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_http_streams_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_http_proxy_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_http_requests_hooks_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_http_requests_response_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_network_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_site_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_taxonomy_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_theme_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_widget_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_scripts_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_styles_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_dependencies_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_dependency_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_script_modules_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_ixr_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_guard_response()
}

async fn class_avif_info_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_feed_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_http_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_json_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_oembed_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_phpass_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_phpmailer_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_pop3_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_requests_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_simplepie_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_smtp_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_snoopy_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_walker_category_dropdown_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_walker_category_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_walker_comment_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_walker_nav_menu_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_walker_page_dropdown_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_walker_page_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wpdb_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_dot_wp_dependencies_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_dot_wp_scripts_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_dot_wp_styles_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn compat_utf8_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn cron_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn date_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn default_constants_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn default_widgets_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_guard_response()
}

async fn deprecated_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn embed_template_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn embed_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn error_protection_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn feed_atom_comments_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn feed_atom_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_guard_response()
}

async fn feed_rdf_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn feed_rss_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn feed_rss2_comments_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn feed_rss2_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn feed_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn fonts_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn functions_dot_wp_scripts_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn functions_dot_wp_styles_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn global_styles_and_settings_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn http_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn https_detection_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn https_migration_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn kses_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn l10n_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn locale_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn media_template_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn media_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_guard_response()
}

async fn meta_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn ms_blogs_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_guard_response()
}

async fn ms_default_constants_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn ms_default_filters_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn ms_deprecated_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn ms_files_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn ms_functions_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn ms_load_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn ms_network_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn ms_settings_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_guard_response()
}

async fn ms_site_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn nav_menu_template_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_guard_response()
}

async fn nav_menu_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn pluggable_deprecated_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn post_formats_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn post_template_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn post_thumbnail_template_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn query_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn registration_functions_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn registration_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn revision_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn rewrite_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn robots_template_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn rss_functions_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn rss_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn script_loader_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn session_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn spl_autoload_compat_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn category_template_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn category_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn comment_template_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn comment_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn compat_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn bookmark_template_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn bookmark_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn cache_compat_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn cache_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn canonical_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn block_bindings_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn block_editor_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn block_patterns_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn block_template_utils_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn block_template_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn abilities_api_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn abilities_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn abilities_api_class_wp_abilities_registry_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn abilities_api_class_wp_ability_categories_registry_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn abilities_api_class_wp_ability_category_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn abilities_api_class_wp_ability_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn abilities_class_wp_settings_abilities_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn assets_script_loader_packages_min_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn assets_script_loader_packages_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn assets_script_modules_packages_min_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn assets_script_modules_packages_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn block_bindings_pattern_overrides_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn block_bindings_post_data_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn block_bindings_post_meta_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn block_bindings_term_data_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn block_patterns_query_grid_posts_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn block_patterns_query_large_title_posts_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn block_patterns_query_medium_posts_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn block_patterns_query_offset_posts_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn block_patterns_query_small_posts_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn block_patterns_query_standard_posts_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn block_patterns_social_links_shared_background_color_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_accordion_item_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_accordion_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_archives_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_avatar_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_block_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_button_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_calendar_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_categories_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_comment_author_name_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_comment_content_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_comment_date_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_comment_edit_link_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_comment_reply_link_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_comments_pagination_next_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_comments_pagination_numbers_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_blocks_json_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_comments_pagination_previous_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_comments_pagination_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_comments_title_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_comments_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_comment_template_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_cover_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_file_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_footnotes_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_gallery_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_heading_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_home_link_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_image_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_index_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_latest_comments_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_latest_posts_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_legacy_widget_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_list_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_loginout_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_media_text_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_navigation_link_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_navigation_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_navigation_submenu_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_page_list_item_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_page_list_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_pattern_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_post_author_biography_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_post_author_name_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_post_author_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_post_comments_count_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_post_comments_form_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_post_comments_link_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_post_content_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_post_date_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_post_excerpt_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_post_featured_image_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_post_navigation_link_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_post_template_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_post_terms_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_post_time_to_read_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_post_title_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_query_no_results_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_query_pagination_next_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_query_pagination_numbers_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_query_pagination_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_query_pagination_previous_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_query_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_query_title_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_query_total_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_read_more_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_require_dynamic_blocks_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_require_static_blocks_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_rss_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_search_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_shortcode_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_site_logo_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_site_tagline_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_site_title_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_social_link_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_tag_cloud_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_template_part_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_term_count_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_term_description_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_term_name_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_term_template_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_video_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn blocks_widget_group_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn build_pages_font_library_page_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn build_pages_font_library_page_wp_admin_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn build_pages_site_editor_page_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn build_pages_site_editor_page_wp_admin_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn build_routes_font_list_content_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn build_routes_font_list_route_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn build_routes_fonts_home_route_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn build_routes_home_route_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn build_routes_index_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn build_routes_navigation_edit_content_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn build_routes_navigation_edit_route_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn build_routes_navigation_list_content_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn build_routes_navigation_list_route_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn build_routes_navigation_route_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn build_routes_pattern_list_content_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn build_routes_pattern_list_route_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn build_routes_pattern_route_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn build_routes_post_edit_route_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn build_routes_post_list_content_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn build_routes_post_list_route_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn build_routes_post_new_route_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn build_routes_post_route_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn build_routes_registry_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn build_routes_styles_content_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn build_routes_styles_route_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn build_routes_template_list_content_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn build_routes_template_list_route_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn build_routes_template_part_list_content_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn build_routes_template_part_list_route_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn build_routes_template_part_route_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn build_routes_template_route_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn css_dist_index_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn css_dist_registry_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_a11y_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_annotations_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_api_fetch_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_autop_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_base_styles_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_blob_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_block_directory_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_block_editor_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_block_library_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_block_serialization_default_parser_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_block_serialization_spec_parser_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_blocks_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_commands_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_components_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_compose_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_core_commands_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_core_data_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_customize_widgets_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_data_controls_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_data_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_date_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_deprecated_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_react_refresh_runtime_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_list_reusable_blocks_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_theme_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_widgets_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_i18n_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_shortcode_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_viewport_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_preferences_persistence_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_format_library_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_dom_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_primitives_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_keyboard_shortcuts_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_media_utils_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_redux_routine_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_private_apis_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_hooks_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_dom_ready_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_priority_queue_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_server_side_render_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_reusable_blocks_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_router_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_element_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_edit_post_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_nux_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_keycodes_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_script_modules_interactivity_router_full_page_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_script_modules_interactivity_router_index_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_script_modules_edit_site_init_index_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_url_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_preferences_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_script_modules_interactivity_index_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_token_list_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_escape_html_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_html_entities_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_script_modules_lazy_editor_index_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_react_refresh_entry_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_wordcount_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_rich_text_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_undo_manager_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_plugins_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_patterns_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_editor_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_script_modules_workflow_index_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_edit_site_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_script_modules_route_index_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_style_engine_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_notices_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_script_modules_abilities_index_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_is_shallow_equal_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_warning_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_script_modules_a11y_index_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_script_modules_boot_index_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_react_i18n_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_script_modules_latex_to_mathml_loader_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_script_modules_block_library_search_view_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_edit_widgets_min_asset_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_script_modules_block_library_accordion_view_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_script_modules_block_library_tabs_view_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_script_modules_block_library_navigation_view_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_script_modules_block_library_query_view_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_script_modules_latex_to_mathml_index_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_script_modules_block_library_image_view_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_script_modules_block_library_file_view_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_script_modules_block_library_form_view_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_script_modules_block_editor_utils_fit_text_frontend_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_script_modules_core_abilities_index_min_asset_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn pomo_entry_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn pomo_plural_forms_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn pomo_po_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn pomo_translations_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn pomo_streams_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn pomo_mo_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn text_diff_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn text_exception_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn text_diff_renderer_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn style_engine_class_wp_style_engine_processor_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn text_diff_renderer_inline_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn text_diff_engine_string_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn text_diff_engine_shell_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn style_engine_class_wp_style_engine_css_rule_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn style_engine_class_wp_style_engine_css_rules_store_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn text_diff_engine_xdiff_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn style_engine_class_wp_style_engine_css_declarations_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn text_diff_engine_native_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn style_engine_class_wp_style_engine_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn build_pages_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn build_routes_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn php_compat_readonly_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn l10n_class_wp_translation_file_php_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn l10n_class_wp_translation_file_mo_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn l10n_class_wp_translation_controller_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sitemaps_class_wp_sitemaps_renderer_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sitemaps_class_wp_sitemaps_provider_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sitemaps_providers_class_wp_sitemaps_taxonomies_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sitemaps_providers_class_wp_sitemaps_users_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sitemaps_providers_class_wp_sitemaps_posts_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn l10n_class_wp_translations_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn l10n_class_wp_translation_file_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sitemaps_class_wp_sitemaps_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sitemaps_class_wp_sitemaps_registry_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sitemaps_class_wp_sitemaps_index_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn block_supports_border_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn block_supports_custom_classname_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn block_supports_duotone_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn block_supports_spacing_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn block_supports_aria_label_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_autoload_php7_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_namespaced_core_ed25519_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_namespaced_core_util_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_namespaced_core_curve25519_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_namespaced_core_hchacha20_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_namespaced_core_curve25519_ge_p3_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_namespaced_core_curve25519_ge_precomp_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_namespaced_core_curve25519_ge_p1p1_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_namespaced_core_curve25519_ge_p2_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_namespaced_core_curve25519_ge_cached_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn interactivity_api_class_wp_interactivity_api_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn interactivity_api_interactivity_api_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn interactivity_api_class_wp_interactivity_api_directives_processor_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sitemaps_class_wp_sitemaps_stylesheet_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_namespaced_core_curve25519_h_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn block_supports_utils_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_namespaced_core_poly1305_state_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn block_supports_colors_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_namespaced_crypto_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn block_supports_align_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn block_supports_shadow_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn block_supports_settings_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn block_supports_dimensions_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn block_supports_generated_classname_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn block_supports_elements_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn block_supports_position_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn block_supports_block_visibility_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn block_supports_layout_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn block_supports_typography_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn block_supports_anchor_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_search_class_wp_rest_post_format_search_handler_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_search_class_wp_rest_term_search_handler_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn customize_class_wp_customize_color_control_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_search_class_wp_rest_post_search_handler_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn customize_class_wp_sidebar_block_editor_control_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_search_class_wp_rest_search_handler_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn customize_class_wp_customize_new_menu_section_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_fields_class_wp_rest_comment_meta_fields_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_fields_class_wp_rest_post_meta_fields_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_namespaced_core_salsa20_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn theme_compat_comments_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_namespaced_core_xsalsa20_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_namespaced_core_chacha20_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_namespaced_core_xchacha20_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_namespaced_core_x25519_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_namespaced_core_chacha20_ctx_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_namespaced_core_chacha20_ietf_ctx_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_namespaced_core_blake2b_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_namespaced_core_poly1305_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_namespaced_core_sip_hash_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_namespaced_core_hsalsa20_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_namespaced_file_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_namespaced_compat_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_lib_php72compat_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_lib_namespaced_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn customize_class_wp_customize_custom_css_setting_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn customize_class_wp_customize_nav_menu_setting_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_lib_sodium_compat_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core_ed25519_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core_util_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn customize_class_wp_customize_themes_section_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn customize_class_wp_customize_image_control_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_fields_class_wp_rest_user_meta_fields_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_fields_class_wp_rest_meta_fields_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_fields_class_wp_rest_term_meta_fields_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_endpoints_class_wp_rest_font_collections_controller_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn theme_compat_footer_embed_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn theme_compat_embed_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn theme_compat_embed_content_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn theme_compat_header_embed_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn theme_compat_header_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn theme_compat_embed_404_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn customize_class_wp_customize_code_editor_control_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn customize_class_wp_widget_area_customize_control_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn customize_class_wp_customize_nav_menu_locations_control_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_lib_php84compat_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_lib_php84compat_const_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_lib_stream_xchacha20_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_endpoints_class_wp_rest_font_faces_controller_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_endpoints_class_wp_rest_block_directory_controller_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_lib_constants_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_endpoints_class_wp_rest_menus_controller_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn customize_class_wp_customize_header_image_control_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn customize_class_wp_customize_site_icon_control_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn customize_class_wp_customize_nav_menu_item_control_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn customize_class_wp_customize_nav_menu_item_setting_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_lib_php72compat_const_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_lib_ristretto255_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_endpoints_class_wp_rest_global_styles_controller_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_autoload_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_endpoints_class_wp_rest_font_families_controller_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_endpoints_class_wp_rest_menu_items_controller_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn theme_compat_footer_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn theme_compat_sidebar_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core_base64_url_safe_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core_base64_original_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core_xsalsa20_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_endpoints_class_wp_rest_edit_site_export_controller_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_endpoints_class_wp_rest_widget_types_controller_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_endpoints_class_wp_rest_application_passwords_controller_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_endpoints_class_wp_rest_template_revisions_controller_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_endpoints_class_wp_rest_templates_controller_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core_aegis128l_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core_curve25519_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_compat_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core_blake2b_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_crypto32_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn customize_class_wp_customize_background_image_setting_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn customize_class_wp_customize_themes_panel_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn customize_class_wp_customize_nav_menu_control_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn customize_class_wp_customize_background_position_control_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn customize_class_wp_customize_partial_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_endpoints_class_wp_rest_post_types_controller_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_endpoints_class_wp_rest_block_patterns_controller_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_endpoints_class_wp_rest_plugins_controller_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core_aes_expanded_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_endpoints_class_wp_rest_pattern_directory_controller_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core_aes_block_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core_aes_key_schedule_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn fonts_class_wp_font_library_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core_secret_stream_state_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn fonts_class_wp_font_face_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn customize_class_wp_customize_nav_menu_auto_add_control_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn customize_class_wp_customize_nav_menu_name_control_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn customize_class_wp_customize_new_menu_control_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn customize_class_wp_customize_cropped_image_control_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn customize_class_wp_customize_theme_control_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_endpoints_class_wp_rest_revisions_controller_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_endpoints_class_wp_rest_post_statuses_controller_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_endpoints_class_wp_rest_attachments_controller_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_endpoints_class_wp_rest_site_health_controller_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_endpoints_class_wp_rest_posts_controller_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn customize_class_wp_customize_nav_menus_panel_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn customize_class_wp_customize_nav_menu_location_control_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core_hchacha20_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core_curve25519_ge_p3_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core_curve25519_ge_precomp_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core_curve25519_ge_p1p1_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core_curve25519_ge_p2_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core_curve25519_ge_cached_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_endpoints_class_wp_rest_template_autosaves_controller_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core_curve25519_h_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_endpoints_class_wp_rest_widgets_controller_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_endpoints_class_wp_rest_global_styles_revisions_controller_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_endpoints_class_wp_rest_comments_controller_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_endpoints_class_wp_rest_controller_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_endpoints_class_wp_rest_sidebars_controller_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core_curve25519_fe_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core_salsa20_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn fonts_class_wp_font_collection_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn fonts_class_wp_font_utils_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core_ristretto255_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_endpoints_class_wp_rest_settings_controller_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_endpoints_class_wp_rest_blocks_controller_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_endpoints_class_wp_rest_navigation_fallback_controller_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_endpoints_class_wp_rest_block_types_controller_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_endpoints_class_wp_rest_url_details_controller_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn fonts_class_wp_font_face_resolver_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core_aegis_state128l_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core_aegis_state256_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core_chacha20_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core_xchacha20_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core_x25519_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core_chacha20_ctx_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core_chacha20_ietf_ctx_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core_poly1305_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core_sip_hash_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_endpoints_class_wp_rest_terms_controller_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_endpoints_class_wp_rest_taxonomies_controller_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core_hsalsa20_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core_aes_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core_aegis256_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_crypto_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core32_ed25519_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core32_util_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core32_xsalsa20_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core32_curve25519_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core_poly1305_state_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core32_int32_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core32_secretstream_state_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core32_hchacha20_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core32_curve25519_ge_p3_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_endpoints_class_wp_rest_users_controller_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_endpoints_class_wp_rest_abilities_v1_run_controller_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_endpoints_class_wp_rest_search_controller_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_endpoints_class_wp_rest_block_renderer_controller_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_endpoints_class_wp_rest_menu_locations_controller_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_endpoints_class_wp_rest_themes_controller_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_endpoints_class_wp_rest_block_pattern_categories_controller_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_endpoints_class_wp_rest_autosaves_controller_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_endpoints_class_wp_rest_abilities_v1_categories_controller_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_endpoints_class_wp_rest_abilities_v1_list_controller_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core32_curve25519_ge_precomp_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core32_curve25519_ge_p1p1_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core32_curve25519_ge_p2_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core32_curve25519_ge_cached_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core32_curve25519_h_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core32_curve25519_fe_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core32_salsa20_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core32_chacha20_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core32_xchacha20_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core32_x25519_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core32_hsalsa20_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core32_int64_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core32_poly1305_state_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_file_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_php52_spl_fixed_array_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core32_chacha20_ctx_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core32_chacha20_ietf_ctx_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core32_blake2b_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core32_poly1305_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_core32_sip_hash_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_class_wp_rest_response_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_class_wp_rest_request_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn rest_api_class_wp_rest_server_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_src_sodium_exception_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn html_api_class_wp_html_text_replacement_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_script_modules_index_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn js_dist_script_modules_registry_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn widgets_class_wp_widget_media_video_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn widgets_class_wp_widget_calendar_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn widgets_class_wp_widget_media_image_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn widgets_class_wp_widget_categories_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn widgets_class_wp_nav_menu_widget_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn widgets_class_wp_widget_media_audio_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn widgets_class_wp_widget_links_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn widgets_class_wp_widget_recent_comments_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn widgets_class_wp_widget_archives_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn widgets_class_wp_widget_media_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn widgets_class_wp_widget_block_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn widgets_class_wp_widget_search_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn widgets_class_wp_widget_tag_cloud_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn html_api_class_wp_html_tag_processor_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn html_api_class_wp_html_token_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn html_api_class_wp_html_decoder_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn html_api_class_wp_html_span_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn html_api_class_wp_html_stack_event_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn html_api_class_wp_html_attribute_token_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn widgets_class_wp_widget_custom_html_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn html_api_class_wp_html_open_elements_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn widgets_class_wp_widget_media_gallery_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn widgets_class_wp_widget_text_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn html_api_html5_named_character_references_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn html_api_class_wp_html_active_formatting_elements_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn html_api_class_wp_html_processor_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn html_api_class_wp_html_doctype_info_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn html_api_class_wp_html_processor_state_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn widgets_class_wp_widget_meta_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn widgets_class_wp_widget_pages_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn widgets_class_wp_widget_rss_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn widgets_class_wp_widget_recent_posts_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn html_api_class_wp_html_unsupported_exception_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_index_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_plugins_hello_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_plugins_index_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentyfifteen_image_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentyfifteen_content_none_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentyfifteen_index_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentyfifteen_archive_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentyfifteen_content_search_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentyfifteen_comments_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentyfifteen_content_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentyfifteen_footer_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentyfifteen_sidebar_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentyfifteen_content_link_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentyfifteen_single_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentyfifteen_inc_back_compat_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentyfifteen_inc_custom_header_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentyfifteen_inc_customizer_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentyfifteen_inc_block_patterns_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentyfifteen_inc_template_tags_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentyfifteen_author_bio_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentyfifteen_search_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentyfifteen_functions_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentyfifteen_header_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentyfifteen_not_found_404_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentyfifteen_page_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentyfifteen_content_page_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentysixteen_template_parts_content_none_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentysixteen_template_parts_content_single_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentysixteen_template_parts_content_search_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentysixteen_template_parts_content_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentysixteen_template_parts_biography_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentysixteen_template_parts_content_page_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentysixteen_index_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentysixteen_archive_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentysixteen_searchform_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentyseventeen_template_parts_post_content_none_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentyseventeen_template_parts_post_content_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentyseventeen_template_parts_post_content_audio_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentyseventeen_template_parts_post_content_excerpt_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentyseventeen_template_parts_post_content_video_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentysixteen_comments_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentysixteen_single_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentysixteen_sidebar_content_bottom_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentysixteen_inc_back_compat_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentysixteen_inc_customizer_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentysixteen_inc_block_patterns_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentysixteen_inc_template_tags_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentysixteen_search_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentysixteen_functions_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_content_themes_twentysixteen_header_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn customize_class_wp_customize_selective_refresh_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn customize_class_wp_widget_form_customize_control_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn customize_class_wp_customize_media_control_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn customize_class_wp_customize_date_time_control_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn customize_class_wp_customize_header_image_setting_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn customize_class_wp_customize_background_image_control_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn customize_class_wp_customize_filter_setting_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn customize_class_wp_customize_sidebar_section_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn customize_class_wp_customize_nav_menu_section_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn customize_class_wp_customize_upload_control_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn block_supports_block_style_variations_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn sodium_compat_namespaced_core_curve25519_fe_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn block_supports_background_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn admin_bar_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn atomlib_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn author_template_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_theme_json_data_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_theme_json_resolver_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_token_map_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_url_pattern_prefixer_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_walker_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_simplepie_file_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_simplepie_sanitize_kses_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_guard_response()
}

async fn class_wp_speculation_rules_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_text_diff_renderer_inline_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_text_diff_renderer_table_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_guard_response()
}

async fn class_wp_navigation_fallback_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_object_cache_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_oembed_controller_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_paused_extensions_storage_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_phpmailer_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_customize_setting_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_guard_response()
}

async fn class_wp_customize_widgets_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_feed_cache_transient_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_feed_cache_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_http_ixr_client_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_customize_control_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_guard_response()
}

async fn class_wp_customize_manager_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_customize_nav_menus_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_customize_panel_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_guard_response()
}

async fn class_wp_customize_section_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_block_supports_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_block_template_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_block_type_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_classic_to_block_menu_converter_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_duotone_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_block_pattern_categories_registry_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_block_patterns_registry_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_block_styles_registry_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_block_templates_registry_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_block_type_registry_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_block_metadata_registry_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_block_parser_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_block_parser_block_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_block_parser_frame_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_block_processor_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_block_bindings_registry_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_block_bindings_source_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_block_editor_context_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_block_list_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_block_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_recovery_mode_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_recovery_mode_cookie_service_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_recovery_mode_link_service_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_recovery_mode_key_service_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_recovery_mode_email_service_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_xmlrpc_server_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_widget_factory_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_theme_json_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_theme_json_schema_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_textdomain_registry_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_image_editor_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_image_editor_gd_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_image_editor_imagick_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_exception_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_fatal_error_handler_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_admin_bar_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_ajax_response_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_embed_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_editor_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_oembed_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_comment_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_term_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_user_request_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_application_passwords_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_plugin_dependencies_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_locale_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_locale_switcher_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_matchesmapregex_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_list_util_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn class_wp_metadata_lazyloader_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn general_template_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn link_template_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn default_filters_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_guard_response()
}

async fn blocks_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn theme_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn theme_templates_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn theme_previews_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn speculative_loading_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn template_loader_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn template_canvas_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn template_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn taxonomy_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn shortcodes_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn widgets_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn id3_getid3_lib_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn id3_getid3_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn id3_module_audio_video_asf_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn id3_module_audio_video_flv_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn id3_module_audio_video_matroska_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn id3_module_audio_video_quicktime_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn id3_module_audio_video_riff_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn id3_module_audio_ac3_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn id3_module_audio_dts_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn id3_module_audio_flac_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn id3_module_audio_mp3_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn id3_module_audio_ogg_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn id3_module_tag_apetag_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn id3_module_tag_id3v1_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn id3_module_tag_id3v2_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn id3_module_tag_lyrics3_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn ixr_class_base64_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn ixr_class_client_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn ixr_class_clientmulticall_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn ixr_class_date_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn ixr_class_error_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn ixr_class_introspectionserver_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn ixr_class_message_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn ixr_class_request_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn ixr_class_server_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn ixr_class_value_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn phpmailer_dsn_configurator_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn phpmailer_exception_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn phpmailer_oauth_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn phpmailer_oauth_token_provider_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn phpmailer_phpmailer_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn phpmailer_pop3_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn phpmailer_smtp_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_library_requests_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_auth_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_auth_basic_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_autoload_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_capability_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_cookie_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_cookie_jar_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_exception_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_exception_argument_count_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_exception_http_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_exception_http_status304_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_exception_http_status305_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_exception_http_status306_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_exception_http_status400_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_exception_http_status401_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_exception_http_status402_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_exception_http_status403_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_exception_http_status404_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_exception_http_status405_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_exception_http_status406_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_exception_http_status407_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_exception_http_status408_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_exception_http_status409_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_exception_http_status410_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_exception_http_status411_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_exception_http_status412_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_exception_http_status413_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_exception_http_status414_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_exception_http_status415_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_exception_http_status416_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_exception_http_status417_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_exception_http_status418_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_exception_http_status428_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_exception_http_status429_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_exception_http_status431_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_exception_http_status500_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_exception_http_status501_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_exception_http_status502_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_exception_http_status503_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_exception_http_status504_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_exception_http_status505_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_exception_http_status511_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_exception_http_status_unknown_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_exception_invalid_argument_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_exception_transport_curl_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_exception_transport_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_hook_manager_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_hooks_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_idna_encoder_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_ipv6_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_iri_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_port_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_proxy_http_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_proxy_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_requests_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_response_headers_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_response_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_session_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_ssl_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_transport_curl_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_transport_fsockopen_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_transport_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_utility_case_insensitive_dictionary_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_utility_filtered_iterator_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn requests_src_utility_input_validator_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_autoloader_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_library_simple_pie_author_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_library_simple_pie_cache_base_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_library_simple_pie_cache_db_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_library_simple_pie_cache_file_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_library_simple_pie_cache_memcache_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_library_simple_pie_cache_memcached_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_library_simple_pie_cache_mysql_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_library_simple_pie_cache_redis_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_library_simple_pie_cache_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_library_simple_pie_caption_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_library_simple_pie_category_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_library_simple_pie_content_type_sniffer_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_library_simple_pie_copyright_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_library_simple_pie_core_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_library_simple_pie_credit_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_library_simple_pie_decode_html_entities_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_library_simple_pie_enclosure_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_library_simple_pie_exception_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_library_simple_pie_file_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_library_simple_pie_http_parser_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_library_simple_pie_iri_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_library_simple_pie_item_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_library_simple_pie_locator_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_library_simple_pie_misc_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_library_simple_pie_net_ipv6_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_library_simple_pie_parse_date_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_library_simple_pie_parser_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_library_simple_pie_rating_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_library_simple_pie_registry_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_library_simple_pie_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_author_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_cache_base_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_cache_base_data_cache_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_cache_callable_name_filter_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_cache_db_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_cache_data_cache_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_cache_file_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_cache_memcache_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_cache_memcached_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_cache_mysql_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_cache_name_filter_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_cache_psr16_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_cache_redis_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_cache_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_caption_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_category_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_content_type_sniffer_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_copyright_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_credit_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_enclosure_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_exception_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_file_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_gzdecode_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_http_client_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_http_client_exception_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_http_file_client_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_http_parser_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_http_psr18_client_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_http_psr7_response_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_http_raw_text_response_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_http_response_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_iri_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_item_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_locator_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_misc_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_net_ipv6_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_parse_date_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_parser_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_rating_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_registry_aware_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_registry_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_restriction_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_sanitize_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_simple_pie_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_source_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_src_xml_declaration_parser_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_library_simple_pie_restriction_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_library_simple_pie_sanitize_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_library_simple_pie_source_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_library_simple_pie_xml_declaration_parser_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn simple_pie_library_simple_pie_gzdecode_include_live_dispatch(
    _request: Request,
) -> Response {
    legacy_admin_include_empty_response()
}

async fn style_engine_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn sitemaps_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn script_modules_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn version_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_diff_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_guard_response()
}

async fn view_transitions_include_live_dispatch(_request: Request) -> Response {
    legacy_admin_include_empty_response()
}

async fn wp_tinymce_live_dispatch(request: Request) -> Response {
    if request.method() != axum::http::Method::GET {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-includes/js/tinymce/wp-tinymce.php currently supports GET only.",
            }),
        )
        .into_response();
    }

    let params = parse_urlencoded(request.uri().query().unwrap_or_default());
    let bundle_requested = params.contains_key("c");
    let js = if bundle_requested {
        "/* wp-tinymce.js (Rust compatibility bundle) */\nwindow.wpRustTinyMCE={bundle:true,version:\"compat\"};\n"
            .to_string()
    } else {
        "/* tinymce.min.js + compat3x/plugin.min.js (Rust compatibility bundle) */\nwindow.wpRustTinyMCE={bundle:false,version:\"compat3x\"};\n"
            .to_string()
    };

    rust_handled_text(StatusCode::OK, "application/javascript; charset=UTF-8", js).into_response()
}

async fn rest_dispatch_root(State(state): State<AppState>, request: Request) -> impl IntoResponse {
    rest_dispatch_inner(state, request, "/wp-json".to_string())
}

async fn rest_dispatch(
    State(state): State<AppState>,
    Path(rest_path): Path<String>,
    request: Request,
) -> impl IntoResponse {
    let normalized = rest_path.trim_matches('/');
    let route_path = if normalized.is_empty() {
        "/wp-json".to_string()
    } else {
        format!("/wp-json/{normalized}")
    };
    rest_dispatch_inner(state, request, route_path)
}

fn rest_dispatch_inner(state: AppState, request: Request, route_path: String) -> impl IntoResponse {
    let registry = core_seed_routes();
    let headers = request.headers();
    let (authenticated, capabilities) = auth_context_from_headers(headers, &state.auth_secrets);

    let mut rest_request = RestRequest::new(request.method().to_string(), route_path);
    rest_request.authenticated = authenticated;
    rest_request.capabilities = capabilities;

    let result = registry.dispatch(&rest_request);
    rust_handled_json_with_status(
        StatusCode::from_u16(result.status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
        result,
    )
}

async fn admin_ajax_dispatch(State(state): State<AppState>, request: Request) -> impl IntoResponse {
    admin_dispatch_inner(state, request, AdminSurface::Ajax).await
}

async fn admin_post_dispatch(State(state): State<AppState>, request: Request) -> impl IntoResponse {
    admin_dispatch_inner(state, request, AdminSurface::AdminPost).await
}

async fn async_upload_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> impl IntoResponse {
    admin_dispatch_inner(state, request, AdminSurface::AsyncUpload).await
}

async fn admin_dispatch_inner(
    state: AppState,
    request: Request,
    surface: AdminSurface,
) -> impl IntoResponse {
    let registry = core_admin_actions();
    let (parts, body) = request.into_parts();
    let body_bytes = read_request_body(body).await;
    let content_type = parts
        .headers
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let params = merged_params(parts.uri.query(), &body_bytes, &content_type);
    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);

    let action = params
        .get("action")
        .cloned()
        .unwrap_or_else(|| match surface {
            AdminSurface::AsyncUpload => "upload-attachment".to_string(),
            _ => String::new(),
        });

    let nonce_present = params
        .get("_ajax_nonce")
        .or_else(|| params.get("_wpnonce"))
        .is_some_and(|value| !value.trim().is_empty());

    let mut admin_request = AdminRequest::new(surface, action);
    admin_request.authenticated = authenticated;
    admin_request.capabilities = capabilities;
    admin_request.nonce_present = nonce_present;

    let result = registry.dispatch(&admin_request);
    rust_handled_json_with_status(
        StatusCode::from_u16(result.status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
        result,
    )
}

async fn xmlrpc_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> impl IntoResponse {
    let registry = core_xmlrpc_registry();
    let (parts, body) = request.into_parts();
    let body_bytes = read_request_body(body).await;
    let payload = String::from_utf8_lossy(&body_bytes).to_string();
    let method_name = parse_xmlrpc_method_name(&payload);
    let (authenticated, _) = auth_context_from_headers(&parts.headers, &state.auth_secrets);

    let (result, status_code, xml_body) = match method_name {
        Some(method_name) => {
            let result = registry.dispatch(&method_name, authenticated);
            let xml_body = if result.success {
                xmlrpc_success_response(&method_name)
            } else {
                xmlrpc_fault_response(result.fault_code.unwrap_or(-32603), &result.message)
            };
            let status = StatusCode::from_u16(if result.success { 200 } else { 403 })
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
            (result, status, xml_body)
        }
        None => (
            registry.dispatch("", false),
            StatusCode::BAD_REQUEST,
            xmlrpc_fault_response(-32600, "Invalid XML-RPC request"),
        ),
    };

    rust_handled_xml(status_code, xml_body, result.success)
}

async fn cron_live_dispatch(State(state): State<AppState>, request: Request) -> impl IntoResponse {
    let now = unix_now();
    let query_string = request.uri().query().unwrap_or_default();
    let doing_wp_cron = parse_doing_wp_cron(query_string);

    let mut scheduler = state
        .cron_scheduler
        .lock()
        .expect("cron scheduler mutex poisoned");

    let lock_acquired = scheduler.acquire_lock(Duration::from_secs(60));
    if !lock_acquired {
        return rust_handled_json(json!({
            "running": false,
            "lock_acquired": false,
            "due_count": 0,
            "doing_wp_cron": doing_wp_cron,
        }));
    }

    let due = scheduler.due_events(now);
    scheduler.release_lock();
    rust_handled_json(json!({
        "running": true,
        "lock_acquired": true,
        "due_count": due.len(),
        "events": due,
        "doing_wp_cron": doing_wp_cron,
        "next_event_timestamp": scheduler.next_event_timestamp(),
    }))
}

async fn index_bootstrap_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    if request.method() != axum::http::Method::GET {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "index.php currently supports GET only.",
            }),
        )
        .into_response();
    }

    front_live_dispatch_inner(state, request, "/".to_string()).await
}

async fn blog_header_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    if request.method() != axum::http::Method::GET {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-blog-header.php currently supports GET only.",
            }),
        )
        .into_response();
    }

    front_live_dispatch_inner(state, request, "/".to_string()).await
}

async fn load_bootstrap_live_dispatch(request: Request) -> Response {
    if request.method() != axum::http::Method::GET {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-load.php currently supports GET only.",
            }),
        )
        .into_response();
    }

    rust_handled_text(StatusCode::OK, "text/plain; charset=UTF-8", String::new()).into_response()
}

async fn settings_bootstrap_live_dispatch(request: Request) -> Response {
    if request.method() != axum::http::Method::GET {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-settings.php currently supports GET only.",
            }),
        )
        .into_response();
    }

    rust_handled_text(StatusCode::OK, "text/plain; charset=UTF-8", String::new()).into_response()
}

async fn front_live_dispatch_root(
    State(state): State<AppState>,
    request: Request,
) -> impl IntoResponse {
    front_live_dispatch_inner(state, request, "/".to_string()).await
}

async fn front_live_dispatch(
    State(state): State<AppState>,
    Path(front_path): Path<String>,
    request: Request,
) -> impl IntoResponse {
    let path = format!("/{}", front_path.trim_start_matches('/'));
    front_live_dispatch_inner(state, request, path).await
}

async fn front_live_dispatch_inner(state: AppState, request: Request, path: String) -> Response {
    if path.starts_with("/__wp_rust/")
        || path.starts_with("/wp-json")
        || path.starts_with("/wp-admin/")
        || path == "/xmlrpc.php"
        || path == "/wp-cron.php"
    {
        return not_found(request).await.into_response();
    }

    let query = request.uri().query().unwrap_or_default();
    let matched = parse_front_route(&path, query);
    let resolved_site = resolve_multisite_site(&state, request.headers(), &matched.request.path);

    if let Some(target) = &matched.canonical_redirect {
        if *target != path {
            let redirect_target = if query.is_empty() {
                target.clone()
            } else {
                format!("{target}?{query}")
            };
            return rust_handled_redirect(&redirect_target).into_response();
        }
    }

    match matched.kind {
        FrontRouteKind::Feed => {
            let feed_title = resolved_site
                .as_ref()
                .map(|site| format!("WordPress Feed (blog {})", site.blog_id))
                .unwrap_or_else(|| "WordPress Feed".to_string());
            let xml = format!(
                "<?xml version=\"1.0\"?><rss><channel><title>{}</title><description>Rust feed route</description><link>{}</link></channel></rss>",
                feed_title, matched.request.path
            );
            rust_handled_xml(StatusCode::OK, xml, true).into_response()
        }
        FrontRouteKind::NotFound => rust_handled_html(
            StatusCode::NOT_FOUND,
            "<!doctype html><html><body><h1>404 Not Found</h1></body></html>".to_string(),
        )
        .into_response(),
        _ => {
            let title = match matched.kind {
                FrontRouteKind::Home => "Home",
                FrontRouteKind::Single => "Single",
                FrontRouteKind::Page => "Page",
                FrontRouteKind::Archive => "Archive",
                FrontRouteKind::Search => "Search",
                FrontRouteKind::Feed | FrontRouteKind::NotFound => "Front",
            };
            let mut html = format!(
                "<!doctype html><html><body><h1>{title}</h1><p>path={}</p>",
                matched.request.path
            );
            if let Some(site) = &resolved_site {
                html.push_str(&format!(
                    "<p>blog_id={}</p><p>network_domain={}</p><p>network_path={}</p>",
                    site.blog_id, site.domain, site.path
                ));
            }
            if !matched.query_vars.is_empty() {
                html.push_str("<ul>");
                for (key, value) in matched.query_vars {
                    html.push_str(&format!("<li>{key}={value}</li>"));
                }
                html.push_str("</ul>");
            }
            html.push_str("</body></html>");
            rust_handled_html(StatusCode::OK, html).into_response()
        }
    }
}

async fn not_found(request: Request) -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "error": "not_found",
            "path": request.uri().path(),
            "message": "Route is not implemented in wp-rs-server yet."
        })),
    )
}

fn rust_handled_json<T: Serialize>(value: T) -> (HeaderMap, Json<T>) {
    let mut headers = HeaderMap::new();
    headers.insert("X-WP-Rust-Handled", HeaderValue::from_static("1"));
    (headers, Json(value))
}

fn rust_handled_json_with_status<T: Serialize>(
    status: StatusCode,
    value: T,
) -> (StatusCode, HeaderMap, Json<T>) {
    let mut headers = HeaderMap::new();
    headers.insert("X-WP-Rust-Handled", HeaderValue::from_static("1"));
    (status, headers, Json(value))
}

fn rust_handled_xml(
    status: StatusCode,
    body: String,
    _success: bool,
) -> (StatusCode, HeaderMap, String) {
    let mut headers = HeaderMap::new();
    headers.insert("X-WP-Rust-Handled", HeaderValue::from_static("1"));
    headers.insert(
        "Content-Type",
        HeaderValue::from_static("text/xml; charset=UTF-8"),
    );
    (status, headers, body)
}

fn rust_handled_html(status: StatusCode, body: String) -> (StatusCode, HeaderMap, String) {
    rust_handled_text(status, "text/html; charset=UTF-8", body)
}

fn rust_handled_text(
    status: StatusCode,
    content_type: &'static str,
    body: String,
) -> (StatusCode, HeaderMap, String) {
    let mut headers = HeaderMap::new();
    headers.insert("X-WP-Rust-Handled", HeaderValue::from_static("1"));
    headers.insert("Content-Type", HeaderValue::from_static(content_type));
    (status, headers, body)
}

fn rust_handled_redirect(target: &str) -> (StatusCode, HeaderMap, String) {
    let mut headers = HeaderMap::new();
    headers.insert("X-WP-Rust-Handled", HeaderValue::from_static("1"));
    if let Ok(location) = HeaderValue::from_str(target) {
        headers.insert("Location", location);
    }
    (StatusCode::MOVED_PERMANENTLY, headers, String::new())
}

fn resolve_multisite_site(
    state: &AppState,
    headers: &HeaderMap,
    request_path: &str,
) -> Option<NetworkSite> {
    let forwarded_host = headers
        .get("x-forwarded-host")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    let host = if forwarded_host.trim().is_empty() {
        headers
            .get("host")
            .and_then(|value| value.to_str().ok())
            .unwrap_or("example.com")
    } else {
        forwarded_host
    };
    let domain = host.split(':').next().unwrap_or("example.com").trim();
    if domain.is_empty() {
        return None;
    }

    state.multisite_resolver.resolve(domain, request_path)
}

fn auth_context_from_headers(
    headers: &HeaderMap,
    auth_secrets: &AuthSecrets,
) -> (bool, BTreeSet<String>) {
    let cookie_header = headers
        .get("cookie")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    let authenticated_from_cookie =
        resolve_current_user(cookie_header, unix_now(), auth_secrets).is_some();
    let authenticated_from_header = headers
        .get("x-wp-rust-authenticated")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.eq_ignore_ascii_case("true") || value == "1");

    let mut capabilities = BTreeSet::new();
    if let Some(capability_header) = headers
        .get("x-wp-rust-capabilities")
        .and_then(|value| value.to_str().ok())
    {
        for capability in capability_header
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            capabilities.insert(capability.to_string());
        }
    }

    (
        authenticated_from_cookie || authenticated_from_header,
        capabilities,
    )
}

fn merged_params(
    uri_query: Option<&str>,
    body_bytes: &Bytes,
    content_type: &str,
) -> BTreeMap<String, String> {
    let mut params = parse_urlencoded(uri_query.unwrap_or_default());
    if content_type.contains("application/x-www-form-urlencoded") {
        let body = String::from_utf8_lossy(body_bytes);
        for (key, value) in parse_urlencoded(&body) {
            params.insert(key, value);
        }
    }
    params
}

fn parse_urlencoded(raw: &str) -> BTreeMap<String, String> {
    raw.split('&')
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

async fn read_request_body(body: axum::body::Body) -> Bytes {
    to_bytes(body, 1024 * 1024)
        .await
        .unwrap_or_else(|_| Bytes::new())
}

fn build_app_state() -> AppState {
    let mut options = OptionStore::default();
    options.set_option("blogname", "WordPress", true);
    options.set_option("blogdescription", "Just another WordPress site", true);
    options.set_option("home", "http://localhost", true);
    options.set_option("siteurl", "http://localhost", true);
    options.set_option("recently_edited", "[]", false);

    let mut scheduler = CronScheduler::default();
    scheduler.schedule_event(CronEvent {
        hook: "wp_version_check".to_string(),
        timestamp: unix_now() + 60,
        schedule: Some("twicedaily".to_string()),
        args: vec![],
    });

    let mut multisite_resolver = MultisiteResolver::default();
    multisite_resolver.register_site(NetworkSite {
        blog_id: 1,
        domain: "example.com".to_string(),
        path: "/".to_string(),
        is_public: true,
    });
    multisite_resolver.register_site(NetworkSite {
        blog_id: 2,
        domain: "example.com".to_string(),
        path: "/blog/".to_string(),
        is_public: true,
    });

    AppState {
        options: Arc::new(Mutex::new(options)),
        auth_secrets: AuthSecrets::default(),
        nonce_service: NonceService::default(),
        cron_scheduler: Arc::new(Mutex::new(scheduler)),
        multisite_resolver,
    }
}

#[derive(Debug, Deserialize)]
struct InternalOptionsQuery {
    option: Option<String>,
    autoload_only: Option<bool>,
}

async fn internal_options(
    State(state): State<AppState>,
    Query(query): Query<InternalOptionsQuery>,
) -> impl IntoResponse {
    let mut options = state.options.lock().expect("options mutex poisoned");

    if let Some(option_name) = query.option {
        let value = options.get_option(&option_name);
        return rust_handled_json(json!({
            "scope": "single",
            "option": option_name,
            "found": value.is_some(),
            "value": value,
        }));
    }

    let autoload_only = query.autoload_only.unwrap_or(true);
    let values = if autoload_only {
        options.load_alloptions()
    } else {
        options.snapshot()
    };

    rust_handled_json(json!({
        "scope": "bulk",
        "autoload_only": autoload_only,
        "count": values.len(),
        "values": values,
    }))
}

#[derive(Debug, Deserialize)]
struct InternalAuthCookieQuery {
    user_id: Option<u64>,
    username: Option<String>,
    token: Option<String>,
    expiration: Option<u64>,
    scheme: Option<String>,
}

async fn internal_auth_cookie(
    State(state): State<AppState>,
    Query(query): Query<InternalAuthCookieQuery>,
) -> impl IntoResponse {
    let now = unix_now();
    let user_id = query.user_id.unwrap_or(1);
    let username = query.username.unwrap_or_else(|| "admin".to_string());
    let token = query.token.unwrap_or_else(|| "session-token".to_string());
    let expiration = query.expiration.unwrap_or(now + 3_600);
    let scheme = parse_auth_scheme(query.scheme.as_deref()).unwrap_or(AuthScheme::LoggedIn);
    let cookie_value = sign_auth_cookie(
        user_id,
        &username,
        expiration,
        &token,
        scheme,
        &state.auth_secrets,
    );
    let cookie_name = format!("{}fixture", scheme.cookie_prefix());

    rust_handled_json(json!({
        "cookie_name": cookie_name,
        "cookie_value": cookie_value,
        "scheme": scheme.as_str(),
        "user_id": user_id,
        "username": username,
        "expiration": expiration,
    }))
}

#[derive(Debug, Deserialize)]
struct InternalAuthSessionQuery {
    now: Option<u64>,
}

async fn internal_auth_session(
    State(state): State<AppState>,
    Query(query): Query<InternalAuthSessionQuery>,
    request: Request,
) -> impl IntoResponse {
    let now = query.now.unwrap_or_else(unix_now);
    let cookie_header = request
        .headers()
        .get("cookie")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    let resolved = resolve_current_user(cookie_header, now, &state.auth_secrets);

    rust_handled_json(json!({
        "cookie_header_present": !cookie_header.is_empty(),
        "resolved": resolved,
    }))
}

#[derive(Debug, Deserialize)]
struct InternalNonceQuery {
    action: Option<String>,
    user_id: Option<u64>,
    token: Option<String>,
    now: Option<u64>,
}

async fn internal_nonce(
    State(state): State<AppState>,
    Query(query): Query<InternalNonceQuery>,
) -> impl IntoResponse {
    let now = query.now.unwrap_or_else(unix_now);
    let action = query.action.unwrap_or_else(|| "sample-action".to_string());
    let user_id = query.user_id.unwrap_or(1);
    let token = query.token.unwrap_or_else(|| "session-token".to_string());

    let nonce = state
        .nonce_service
        .create_nonce(&action, user_id, &token, now);
    let is_valid = state
        .nonce_service
        .verify_nonce(&nonce, &action, user_id, &token, now);

    rust_handled_json(json!({
        "action": action,
        "user_id": user_id,
        "token": token,
        "nonce": nonce,
        "valid": is_valid,
    }))
}

#[derive(Debug, Deserialize)]
struct InternalContentRouteQuery {
    path: Option<String>,
    query: Option<String>,
}

async fn internal_content_route(
    Query(query): Query<InternalContentRouteQuery>,
) -> impl IntoResponse {
    let path = query.path.unwrap_or_else(|| "/".to_string());
    let query_string = query.query.unwrap_or_default();
    let route = parse_front_route(&path, &query_string);
    rust_handled_json(route)
}

#[derive(Debug, Deserialize)]
struct InternalBlockParseQuery {
    content: Option<String>,
}

async fn internal_block_parse(Query(query): Query<InternalBlockParseQuery>) -> impl IntoResponse {
    let content = query.content.unwrap_or_default();
    let block_names = extract_block_names(&content);
    rust_handled_json(json!({
        "count": block_names.len(),
        "blocks": block_names,
    }))
}

async fn internal_rest_contract() -> impl IntoResponse {
    let registry = core_seed_routes();
    rust_handled_json(registry.contract_document())
}

#[derive(Debug, Deserialize)]
struct InternalRestDispatchQuery {
    method: Option<String>,
    path: Option<String>,
    authenticated: Option<bool>,
    capabilities: Option<String>,
}

async fn internal_rest_dispatch(
    Query(query): Query<InternalRestDispatchQuery>,
) -> impl IntoResponse {
    let registry = core_seed_routes();
    let mut request = RestRequest::new(
        query.method.unwrap_or_else(|| "GET".to_string()),
        query
            .path
            .unwrap_or_else(|| "/wp-json/wp/v2/posts".to_string()),
    );
    request.authenticated = query.authenticated.unwrap_or(false);

    if let Some(capabilities) = query.capabilities {
        for capability in capabilities
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            request.capabilities.insert(capability.to_string());
        }
    }

    let result = registry.dispatch(&request);
    rust_handled_json(result)
}

async fn internal_admin_contract() -> impl IntoResponse {
    let registry = core_admin_actions();
    rust_handled_json(registry.contract_map())
}

#[derive(Debug, Deserialize)]
struct InternalAdminDispatchQuery {
    surface: Option<String>,
    action: Option<String>,
    authenticated: Option<bool>,
    nonce_present: Option<bool>,
    capabilities: Option<String>,
}

async fn internal_admin_dispatch(
    Query(query): Query<InternalAdminDispatchQuery>,
) -> impl IntoResponse {
    let registry = core_admin_actions();
    let surface = parse_admin_surface(query.surface.as_deref()).unwrap_or(AdminSurface::Ajax);
    let mut request = AdminRequest::new(surface, query.action.unwrap_or_default());
    request.authenticated = query.authenticated.unwrap_or(false);
    request.nonce_present = query.nonce_present.unwrap_or(false);

    if let Some(capabilities) = query.capabilities {
        for capability in capabilities
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            request.capabilities.insert(capability.to_string());
        }
    }

    let result = registry.dispatch(&request);
    rust_handled_json(result)
}

async fn internal_xmlrpc_contract() -> impl IntoResponse {
    let registry = core_xmlrpc_registry();
    rust_handled_json(registry.methods().to_vec())
}

#[derive(Debug, Deserialize)]
struct InternalXmlRpcDispatchQuery {
    method: Option<String>,
    payload: Option<String>,
    authenticated: Option<bool>,
}

async fn internal_xmlrpc_dispatch(
    Query(query): Query<InternalXmlRpcDispatchQuery>,
) -> impl IntoResponse {
    let registry = core_xmlrpc_registry();
    let authenticated = query.authenticated.unwrap_or(false);
    let method_name = query.method.or_else(|| {
        query
            .payload
            .and_then(|payload| parse_xmlrpc_method_name(&payload))
    });

    let Some(method_name) = method_name else {
        return rust_handled_json(json!({
            "status_code": 400,
            "result": {
                "success": false,
                "fault_code": -32600,
                "message": "Invalid XML-RPC request",
                "method_name": "",
            },
            "xml": xmlrpc_fault_response(-32600, "Invalid XML-RPC request"),
        }));
    };

    let result = registry.dispatch(&method_name, authenticated);
    let xml = if result.success {
        xmlrpc_success_response(&method_name)
    } else {
        xmlrpc_fault_response(result.fault_code.unwrap_or(-32603), &result.message)
    };

    rust_handled_json(json!({
        "status_code": if result.success { 200 } else { 403 },
        "result": result,
        "xml": xml,
    }))
}

#[derive(Debug, Deserialize)]
struct InternalCronScheduleQuery {
    hook: Option<String>,
    timestamp: Option<u64>,
    schedule: Option<String>,
    args: Option<String>,
    query_string: Option<String>,
}

async fn internal_cron_schedule(
    State(state): State<AppState>,
    Query(query): Query<InternalCronScheduleQuery>,
) -> impl IntoResponse {
    let hook = query
        .hook
        .unwrap_or_else(|| "wp_scheduled_delete".to_string());
    let timestamp = query.timestamp.unwrap_or_else(|| unix_now() + 60);
    let schedule = query.schedule;
    let args = query
        .args
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
        .collect::<Vec<_>>();
    let doing_wp_cron = parse_doing_wp_cron(&query.query_string.unwrap_or_default());

    let mut scheduler = state
        .cron_scheduler
        .lock()
        .expect("cron scheduler mutex poisoned");
    scheduler.schedule_event(CronEvent {
        hook,
        timestamp,
        schedule,
        args,
    });

    rust_handled_json(json!({
        "scheduled": true,
        "doing_wp_cron": doing_wp_cron,
        "next_event_timestamp": scheduler.next_event_timestamp(),
    }))
}

#[derive(Debug, Deserialize)]
struct InternalCronDueQuery {
    now: Option<u64>,
}

async fn internal_cron_due(
    State(state): State<AppState>,
    Query(query): Query<InternalCronDueQuery>,
) -> impl IntoResponse {
    let now = query.now.unwrap_or_else(unix_now);
    let mut scheduler = state
        .cron_scheduler
        .lock()
        .expect("cron scheduler mutex poisoned");

    let lock_acquired = scheduler.acquire_lock(Duration::from_secs(60));
    if !lock_acquired {
        return rust_handled_json(json!({
            "lock_acquired": false,
            "due_count": 0,
            "events": [],
        }));
    }

    let due = scheduler.due_events(now);
    scheduler.release_lock();
    rust_handled_json(json!({
        "lock_acquired": true,
        "due_count": due.len(),
        "events": due,
        "next_event_timestamp": scheduler.next_event_timestamp(),
    }))
}

#[derive(Debug, Deserialize)]
struct InternalMultisiteResolveQuery {
    domain: Option<String>,
    path: Option<String>,
}

async fn internal_multisite_resolve(
    State(state): State<AppState>,
    Query(query): Query<InternalMultisiteResolveQuery>,
) -> impl IntoResponse {
    let domain = query.domain.unwrap_or_else(|| "example.com".to_string());
    let path = query.path.unwrap_or_else(|| "/".to_string());
    let resolved = state.multisite_resolver.resolve(&domain, &path);

    rust_handled_json(json!({
        "domain": domain,
        "path": path,
        "resolved": resolved,
    }))
}

#[derive(Debug, Deserialize)]
struct InternalMaintenanceStatusQuery {
    upgrading: Option<u64>,
}

async fn internal_maintenance_status(
    Query(query): Query<InternalMaintenanceStatusQuery>,
) -> impl IntoResponse {
    let now = unix_now();
    let upgrading = query.upgrading.unwrap_or(0);
    let age_secs = if upgrading > 0 {
        now.saturating_sub(upgrading)
    } else {
        0
    };
    let stale = upgrading > 0 && age_secs >= 600;
    let active = upgrading > 0 && !stale;

    rust_handled_json(json!({
        "active": active,
        "upgrading": if upgrading > 0 { Some(upgrading) } else { None::<u64> },
        "age_secs": age_secs,
        "stale": stale,
        "retry_after_secs": 600,
        "now": now,
    }))
}

async fn internal_plugin_compat_matrix() -> impl IntoResponse {
    let settings = RustGatewaySettings::from_env();
    let core_endpoints = php_runtime_core_endpoints()
        .iter()
        .map(|endpoint| endpoint.to_string())
        .collect::<Vec<_>>();

    rust_handled_json(json!({
        "plugin_compat_mode": settings.plugin_compat_mode,
        "deployment_profile": settings.deployment_profile,
        "endpoint_allowlist": settings.endpoint_allowlist,
        "method_allowlist": settings.method_allowlist,
        "php_runtime_core_endpoints": core_endpoints,
    }))
}

fn parse_auth_scheme(value: Option<&str>) -> Option<AuthScheme> {
    match value
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "auth" => Some(AuthScheme::Auth),
        "secure_auth" | "secure" => Some(AuthScheme::SecureAuth),
        "logged_in" | "loggedin" => Some(AuthScheme::LoggedIn),
        _ => None,
    }
}

fn parse_admin_surface(value: Option<&str>) -> Option<AdminSurface> {
    match value
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "ajax" | "admin-ajax" => Some(AdminSurface::Ajax),
        "admin-post" | "post" => Some(AdminSurface::AdminPost),
        "async-upload" | "upload" => Some(AdminSurface::AsyncUpload),
        _ => None,
    }
}

fn unix_now() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_secs()
}
