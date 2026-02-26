-- Minimal deterministic seed for compatibility captures.
-- This script assumes WordPress core tables already exist.

SET NAMES utf8mb4;

INSERT INTO wp_options (option_name, option_value, autoload)
VALUES
  ('blogname', 'Rust Migration Fixture', 'yes'),
  ('blogdescription', 'Deterministic baseline fixture', 'yes'),
  ('siteurl', 'http://localhost:8080', 'yes'),
  ('home', 'http://localhost:8080', 'yes')
ON DUPLICATE KEY UPDATE option_value = VALUES(option_value), autoload = VALUES(autoload);
