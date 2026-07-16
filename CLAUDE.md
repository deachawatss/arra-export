# arra-export

- The Rust workspace lives at the repository root; the Axum binary is `crates/arra-export`.
- The SvelteKit frontend is a static SPA in `frontend/`; build it before running the server so Axum can serve `frontend/build`.
- The backend is an Oracle HTTP API bridge. Do not add direct database reads or writes.
- Keep `ORACLE_URL` as the process-start target and validate user-entered replacement URLs at the API boundary.
- Verify backend changes with `cargo test`, `cargo clippy -- -D warnings`, and `cargo build`; verify frontend changes with `bun run check` and `bun run build`.
