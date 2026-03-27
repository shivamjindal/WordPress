# AGENTS.md

## Cursor Cloud specific instructions

This is a **WordPress 7.0-alpha** (build 61604) core repository — a monolithic PHP application requiring PHP + MariaDB/MySQL.

### Services

| Service | How to start | Notes |
|---------|-------------|-------|
| MariaDB | `sudo service mariadb start` | Database: `wordpress`, user: `wpuser`, password: `wppass123` |
| PHP dev server | `cd /workspace && php -S localhost:8080` | Serves WordPress on port 8080 |

### Running the app

1. Start MariaDB: `sudo service mariadb start`
2. Start PHP built-in server: `cd /workspace && php -S localhost:8080`
3. Browse to `http://localhost:8080/` (admin: `admin` / `admin123!@#`)

### Lint

- PHP syntax check (all files): `find /workspace -name "*.php" | xargs -P4 php -l`
- There is no PHPUnit test suite or PHPCS config in this core repo checkout.

### Key caveats

- `wp-config.php` is git-ignored and auto-generated during setup. It points to local MariaDB with `DB_NAME=wordpress`, `DB_USER=wpuser`, `DB_PASSWORD=wppass123`.
- The REST API uses query-parameter routing by default (`?rest_route=/wp/v2/...`), not pretty permalinks.
- `WP_DEBUG`, `WP_DEBUG_LOG`, and `WP_DEBUG_DISPLAY` are enabled in `wp-config.php`.
- The PHP built-in server does not support `.htaccess` rewrites; pretty permalinks require Apache or Nginx.
