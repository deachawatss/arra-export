# arra-export

`arra-export` creates portable knowledge artifacts from one running Arra Oracle instance. It is a consumer of the Oracle export API: it does not read Oracle databases directly.

Phase 1 includes a Rust/Axum API server and a SvelteKit static UI. Federation and mesh discovery are intentionally out of scope.

## Requirements

- Rust stable
- [Bun](https://bun.sh/)
- An Arra Oracle server with the export app endpoints available (normally `http://localhost:47778`)

## Run locally

Build the SvelteKit application first. The Rust server serves the resulting static files from `frontend/build`.

```sh
cd frontend
bun install
bun run build
cd ..

cargo run
```

Open [http://localhost:4778](http://localhost:4778). Use the connection form to test the target Oracle, load its collections, create an export, and download the artifact.

## Configuration

| Variable | Default | Purpose |
| --- | --- | --- |
| `ORACLE_URL` | `http://localhost:47778` | Initial Oracle HTTP API target. |
| `ARRA_EXPORT_PORT` | `4778` | Axum server port. |
| `ARRA_EXPORT_FRONTEND_DIST` | `frontend/build` | SvelteKit static output to serve. |

For example:

```sh
ORACLE_URL=http://localhost:47778 ARRA_EXPORT_PORT=4778 cargo run
```

The connection form may switch the in-memory target for the current process after a successful test. It accepts HTTP(S) origins only, does not permit credentials, disables redirects, and rejects user-supplied hosts that resolve to private/reserved addresses other than loopback.

## API

All endpoints are served by `cargo run` at the configured export port.

| Method | Path | Description |
| --- | --- | --- |
| `GET` | `/api/health` | Server readiness response. |
| `POST` | `/api/test-connection` | Tests an Oracle URL and makes it the current process target after success. |
| `GET` | `/api/collections` | Lists collections and supported formats from the current Oracle. |
| `POST` | `/api/export` | Creates an export job through the Oracle export app. |
| `GET` | `/api/export/:id/download` | Proxies a completed artifact download. |
| `GET` | `/api/export/history` | Lists export jobs created during the current process lifetime. |

Test a connection:

```sh
curl -X POST http://localhost:4778/api/test-connection \
  -H 'content-type: application/json' \
  -d '{"url":"http://localhost:47778"}'
```

Create an export:

```sh
curl -X POST http://localhost:4778/api/export \
  -H 'content-type: application/json' \
  -d '{"collection":"oracle_documents","format":"json","includeGraph":true}'
```

The export request supports `json`, `csv`, `markdown`, and `jsonl`. Arra Oracle's `/api/v1/export/app/run` endpoint creates the requested artifact, so Phase 1 preserves that generated representation while tracking and serving the job locally. A future local transformation engine needs a stable raw-record contract before it can safely convert between formats.

## Development checks

```sh
cargo test
cargo clippy --all-targets -- -D warnings
cargo build

cd frontend
bun run check
bun run build
```

## Architecture

```text
crates/arra-export/
  src/
    main.rs            Axum server and static asset host
    app.rs             Local REST API handlers
    oracle_client.rs   Oracle HTTP API bridge
    export.rs          Export job history and artifact model
    config.rs          Environment configuration
frontend/
  src/routes/          SvelteKit pages
  src/lib/             Typed API client and UI components
```
