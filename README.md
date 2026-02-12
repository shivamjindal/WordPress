# RustPress (Rust WordPress-like demo)

This repo now includes a Rust implementation of a minimal WordPress-like blog server called **RustPress**.

It serves:
- Home page listing posts
- Individual post pages by slug (e.g. `/hello-world/`)
- Static assets at `/static/*`

## Quickstart

Requirements: Rust toolchain (`cargo`).

Run the server:

- `cargo run`

Open:

- `http://127.0.0.1:3000/`
- `http://127.0.0.1:3000/hello-world/`

## Content

- Site config: `content/site.toml`
- Posts: `content/posts/*.md` (YAML frontmatter + Markdown body)

Example post included: `content/posts/hello-world.md`

## Notes

The original PHP WordPress codebase is still present in the repository, but RustPress is the intended runtime for the migrated demo.

