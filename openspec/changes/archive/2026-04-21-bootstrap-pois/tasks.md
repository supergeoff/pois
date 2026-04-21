## 1. Toolchain et hygiène de dépôt

- [x] 1.1 Créer `rust-toolchain.toml` à la racine : `channel = "1.95.0"`, `components = ["rustfmt", "clippy"]`, `profile = "minimal"`
- [x] 1.2 Créer `.gitignore` Rust standard : `/target`, `*.rs.bk`, `/.vscode` (optionnel), **ne pas** ignorer `Cargo.lock`
- [x] 1.3 Créer `.dockerignore` : `target/`, `.git/`, `openspec/changes/`, `*.md` (sauf README si existant), `.vscode/`
- [x] 1.4 Vérifier : `cargo --version` sélectionne 1.95.0

## 2. Manifeste Cargo et dépendances

- [x] 2.1 Créer `Cargo.toml` racine : `[package] name = "pois"`, `version = "0.1.0"`, `edition = "2024"`, `rust-version = "1.95.0"`, `license = "MIT"` (à confirmer), `publish = false`, `default-run = "pois"`
- [x] 2.2 Déclarer `[[bin]] name = "pois", path = "src/main.rs"` et `[lib] name = "pois", path = "src/lib.rs"`
- [x] 2.3 Ajouter les dépendances versions stables courantes :
  - `tokio` avec features explicites `["rt-multi-thread", "macros", "net", "fs", "signal", "time"]` (pas de `full`)
  - `axum`
  - `tower`, `tower-http` avec features `["trace", "validate-request"]`
  - `askama` (avec `askama_axum` ou rendu manuel selon la version disponible)
  - `serde` avec feature `["derive"]`
  - `toml`
  - `clap` avec features `["derive", "env"]`
  - `tracing`, `tracing-subscriber` avec features `["env-filter", "json", "fmt"]`
  - `thiserror`
  - `anyhow`
  - `subtle` (comparaison à temps constant basic auth)
- [x] 2.4 Configurer `[lints.clippy]` : `all = "deny"` et `pedantic = "warn"` (sans `unwrap_used = "deny"`), ou un ensemble équivalent plus conservateur si pedantic est trop bruyant
- [x] 2.5 Lancer `cargo fetch` pour résoudre le graphe et commiter `Cargo.lock`

## 3. Squelette source

- [x] 3.1 Créer `src/lib.rs` minimal : `pub mod cli; pub mod config; pub mod data; pub mod errors; pub mod gateway;` plus constantes `pub const VERSION: &str = env!("CARGO_PKG_VERSION");`
- [x] 3.2 Créer `src/errors.rs` avec un enum `AppError` (`thiserror::Error`) qui capture au moins : `MissingEnv(&'static str)`, `Io(#[from] std::io::Error)`, `Config(#[from] toml::de::Error)`, `Bind(std::io::Error)` — les variantes non utilisées peuvent rester en placeholder
- [x] 3.3 Créer `src/cli/mod.rs` : enum `Cli` via `clap::Parser` avec sous-commande unique `Gateway` pour l'instant, et fonction `pub async fn run(cli: Cli) -> anyhow::Result<()>` qui dispatche
- [x] 3.4 Créer `src/cli/gateway.rs` : struct `GatewayArgs` (port via `#[clap(long, env = "PORT", default_value = "8080")]`, data-dir via `#[clap(long, env = "POIS_DATA_DIR", default_value = "/data")]`, admin user/pass via `env`), plus fonction `pub async fn run(args: GatewayArgs) -> anyhow::Result<()>`
- [x] 3.5 Créer `src/config/mod.rs` : struct `GlobalConfig` avec `#[derive(Deserialize)]` vide + `impl GlobalConfig { pub fn load_or_default(path: &Path) -> Result<Self, AppError> { ... } }` — si le fichier n'existe pas, retourner `Self::default()`. Commentaire TODO pointant la proposition `port-config`.
- [x] 3.6 Créer `src/data/mod.rs` : fonction `pub fn ensure_layout(root: &Path) -> Result<(), AppError>` qui crée `agents/`, `honcho/`, `logs/` si absents (`fs::create_dir_all`), sans toucher `config.toml`

## 4. Module gateway — serveur axum + auth + /health

- [x] 4.1 Créer `src/gateway/mod.rs` qui construit le `axum::Router` : route `GET /health` publique, toutes les autres routes passent par la couche basic auth. Expose `pub async fn serve(bind_addr: SocketAddr, auth: BasicAuth, data_dir: PathBuf) -> Result<(), AppError>`
- [x] 4.2 Créer `src/gateway/auth.rs` : struct `BasicAuth { user: String, pass: String }` avec constructeur `from_env()` qui lit `POIS_ADMIN_USER` et `POIS_ADMIN_PASS`, renvoie `AppError::MissingEnv(...)` si absent/vide. Middleware axum qui :
  - lit le header `Authorization`
  - décode `Basic base64(user:pass)`
  - compare `user` et `pass` à temps constant via `subtle::ConstantTimeEq`
  - renvoie `401` avec header `WWW-Authenticate: Basic realm="pois"` si échec
- [x] 4.3 Créer `src/gateway/health.rs` : `async fn health() -> &'static str { "ok" }` + impl éventuel `IntoResponse`
- [x] 4.4 Créer `src/gateway/views.rs` : handler `async fn index() -> impl IntoResponse` qui rend le template askama `index.html`. Le template peut juste dire « pois — gateway up » pour l'instant.
- [x] 4.5 Wire-up dans `src/cli/gateway.rs::run` : initialiser tracing-subscriber, construire `BasicAuth::from_env()` (échec early si manquant), appeler `data::ensure_layout(&args.data_dir)`, construire le router, binder sur `0.0.0.0:{port}`, servir via `axum::serve`

## 5. Templates askama et static

- [x] 5.1 Créer `templates/base.html` : squelette HTML5 avec balise `<title>pois</title>`, import CDN de `htmx.org` (v2) et `@picocss/pico` (v2), `{% block content %}{% endblock %}`
- [x] 5.2 Créer `templates/index.html` : extends `base.html`, contenu minimal qui confirme que le gateway tourne
- [x] 5.3 Centraliser les URLs CDN (htmx + pico.css) dans `base.html` ; noter les versions figées dans un commentaire HTML en tête de fichier. Pas de vendoring `static/`, pas de `ServeDir` pour l'instant.
- [x] 5.4 Vérifier que askama compile les templates sans erreur (`cargo check`)

## 6. Point d'entrée binaire

- [x] 6.1 Créer `src/main.rs` : `#[tokio::main(flavor = "multi_thread")] async fn main() -> anyhow::Result<()> { let cli = Cli::parse(); pois::cli::run(cli).await }`
- [x] 6.2 Vérifier : `cargo run -- --help` affiche la page d'aide clap avec la sous-commande `gateway`
- [x] 6.3 Vérifier : `POIS_ADMIN_USER=admin POIS_ADMIN_PASS=secret cargo run -- gateway` démarre un serveur sur `0.0.0.0:8080`
- [x] 6.4 Vérifier sans credentials : `cargo run -- gateway` échoue avec un message explicite sur stderr et code de sortie non nul

## 7. Dockerfile et packaging

- [x] 7.1 Créer `Dockerfile` multi-stage :
  - stage `builder` : `FROM rust:1.95-slim`, install `pkg-config` + `libssl-dev` si reqwest/openssl traîne (à voir selon deps), `WORKDIR /src`, copier sources, `cargo build --release --locked`
  - stage `runtime` : `FROM debian:bookworm-slim`, install `ca-certificates`, copier le binaire depuis `builder` vers `/usr/local/bin/pois`
  - `VOLUME /data`
  - `ENV POIS_DATA_DIR=/data`
  - `EXPOSE 8080`
  - `ENTRYPOINT ["/usr/local/bin/pois", "gateway"]`
- [x] 7.2 Vérifier : `docker build -t pois:dev .` réussit et l'image fait < 100 Mo
- [x] 7.3 Vérifier : `docker run --rm -e POIS_ADMIN_USER=u -e POIS_ADMIN_PASS=p -p 8080:8080 pois:dev` démarre et `curl -s localhost:8080/health` répond `ok`
- [x] 7.4 Vérifier : `curl -s -o /dev/null -w '%{http_code}\n' localhost:8080/` renvoie `401`
- [x] 7.5 Vérifier : `curl -s -u u:p localhost:8080/` renvoie `200` avec du HTML

## 8. Réécriture de `openspec/project.md`

- [x] 8.1 Remplacer la section « Purpose » : « `pois` est un compagnon IA personnel Rust multi-agent, contrôlable via CLI locale et dashboard web, auto-hébergeable en Docker (Railway). Inspiré librement de `nanobot` (HKUDS) et `OpenClaw` — pas un portage fidèle. »
- [x] 8.2 Remplacer la section « Tech stack » : Rust 1.95.0 édition 2024 / MSRV 1.95.0, `tokio`, `axum + askama + htmx`, `serde + toml`, `clap`, `tracing`, `thiserror`, Docker multi-stage
- [x] 8.3 Remplacer la section « Project structure » : le layout mono-crate figé ci-dessus
- [x] 8.4 Remplacer la section « Conventions » : `Result<T, Error>` via `thiserror` dans les modules runtime, `anyhow` permis en `main`, `unwrap`/`expect` justifiés ou interdits, `cargo fmt --check` + `cargo clippy --all-targets -- -D warnings`
- [x] 8.5 Ajouter une section « Persistence » : schéma `/data/` tel que décrit dans le spec
- [x] 8.6 Ajouter une section « Deployment » : Docker image, Railway, `PORT` env, volume `/data`, auth via `POIS_ADMIN_USER` / `POIS_ADMIN_PASS`
- [x] 8.7 Ajouter une section « Inspirations (non contractuelles) » : URL nanobot, URL OpenClaw
- [x] 8.8 Supprimer la section « Open questions » d'origine (toutes tranchées)

## 9. Validation transverse

- [x] 9.1 `cargo build` : succès
- [x] 9.2 `cargo fmt --check` : succès (appliquer `cargo fmt` si besoin)
- [x] 9.3 `cargo clippy --all-targets -- -D warnings` : succès
- [x] 9.4 `cargo test` : succès (aucun test métier requis à ce stade, sortie OK)
- [x] 9.5 `cargo tree -i async-std 2>&1` et `cargo tree -i smol 2>&1` : absents du graphe
- [x] 9.6 Tests manuels fumée : variantes `--help`, `gateway` avec/sans env vars, `/health`, `/` avec/sans basic auth, validés ci-dessus en 6.2-6.4 et 7.3-7.5

## 10. Validation OpenSpec

- [x] 10.1 `openspec validate bootstrap-pois --strict` : succès
- [x] 10.2 `openspec show bootstrap-pois --type change --deltas-only` : inspection visuelle du delta `project-foundations`
- [x] 10.3 Vérifier que `openspec/specs/` reste vide (spec promue à l'archivage)

## 11. Archivage (post-revue)

- [x] 11.1 Après acceptation, lancer `/opsx-archive-change bootstrap-pois` pour promouvoir `project-foundations` vers `openspec/specs/project-foundations/spec.md`
