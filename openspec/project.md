# Project: pois

## Purpose

`pois` est un compagnon IA personnel Rust multi-agent, contrôlable via
CLI locale et dashboard web, auto-hébergeable en Docker (cible Railway).
Inspiré librement de [nanobot](https://github.com/HKUDS/nanobot)
(HKUDS) et [OpenClaw](https://github.com/openclaw/openclaw) — pas un
portage fidèle, pas de contrat de parité API.

Mono-utilisateur par instance : l'opérateur unique administre ses
agents via CLI sur sa machine et/ou via le dashboard protégé par basic
auth à distance.

## Tech stack

- **Langage** : Rust
- **Édition / MSRV** : `edition = "2024"`, `rust-version = "1.95.0"`
- **Runtime async** : `tokio` (unique, features explicites, pas de `full`)
- **Web** : `axum` + `askama` (templates typés à la compilation) + `htmx` (CDN) + `pico.css` (CDN)
- **Config** : `serde` + `toml` (TOML partout — global et par-agent)
- **CLI** : `clap` (derive, env)
- **Observabilité** : `tracing` + `tracing-subscriber` (JSON en prod via `POIS_LOG_FORMAT=json`, compact en dev)
- **Erreurs** : `thiserror` dans les modules runtime, `anyhow` toléré en `main` / init
- **Build / distribution** : Cargo (mono-crate, `[[bin]]` + `[lib]`), Dockerfile multi-stage (`rust:1.95-slim` → `debian:bookworm-slim`)

## Project structure

Mono-crate. Un unique package Cargo nommé `pois` avec bibliothèque et
binaire dans le même manifeste. Tout workspace multi-crate passe par
une proposition OpenSpec dédiée.

```
pois/
├── Cargo.toml
├── Cargo.lock              # commité
├── rust-toolchain.toml     # pinned channel 1.95.0
├── Dockerfile
├── .dockerignore
├── .gitignore
├── src/
│   ├── main.rs             # clap::parse + dispatch sous-commandes
│   ├── lib.rs              # re-exports publics
│   ├── cli/
│   │   ├── mod.rs          # Cli / Command / run
│   │   └── gateway.rs      # sous-commande `pois gateway`
│   ├── gateway/
│   │   ├── mod.rs          # router axum + serve()
│   │   ├── auth.rs         # middleware basic auth (subtle + base64)
│   │   ├── health.rs       # GET /health (public)
│   │   └── views.rs        # handlers askama
│   ├── config/
│   │   └── mod.rs          # GlobalConfig (stub — port-config)
│   ├── data/
│   │   └── mod.rs          # ensure_layout() de $POIS_DATA_DIR
│   └── errors.rs           # AppError (thiserror)
├── templates/              # askama, typés à la compilation
│   ├── base.html
│   └── index.html
└── openspec/               # propositions de changement et specs promues
```

## Conventions

- **Erreurs** : `Result<T, ModuleError>` via `thiserror` dans les
  modules runtime (`gateway`, `config`, `data`, futurs `agent`,
  `channels`, `providers`, `mcp`). `anyhow` autorisé uniquement dans
  `main` et le chemin d'init.
- **unwrap / expect** : tolérés uniquement accompagnés d'un commentaire
  `// SAFETY:` ou `// NOTE:` documentant l'invariant. Sinon, un retour
  d'erreur typé est exigé.
- **Style / lint** : `cargo fmt --check` et
  `cargo clippy --all-targets -- -D warnings` sont la référence.
  `unwrap_used = "deny"` N'EST PAS activé globalement à ce stade.
- **Tests** : TDD via la skill Superpowers `test-driven-development`
  quand une logique métier l'appelle ; aucun test de smoke n'est
  exigé tant que la surface se stabilise.
- **Dépendances tokio** : features explicites
  (`rt-multi-thread`, `macros`, `net`, `fs`, `signal`, `time`),
  jamais `full`. Aucune dépendance directe ou indirecte à `async-std`
  ou `smol` ne doit apparaître dans `cargo tree`.

## Persistence

Tout l'état persistant vit sous `$POIS_DATA_DIR` (défaut : `/data`).
Le runtime crée les sous-répertoires manquants au boot sans écraser
`config.toml` s'il existe déjà.

```
$POIS_DATA_DIR/
├── config.toml             # configuration globale (schéma : port-config)
├── agents/
│   └── <agent-id>/
│       ├── config.toml     # config locale agent
│       ├── SOUL.md         # persona / identité (port-agent-loop)
│       ├── HEARTBEAT.md    # mémoire rolling (port-agent-loop)
│       └── tools/          # outils spécifiques agent
├── honcho/                 # cache / tokens client Honcho (integrate-honcho)
└── logs/                   # traces runtime
```

Le schéma interne de chaque fichier (`config.toml` global et local,
`SOUL.md`, `HEARTBEAT.md`) est tranché par les propositions OpenSpec
dédiées citées ci-dessus.

## Deployment

- **Cible primaire** : Railway (PaaS). Tout PaaS compatible Docker
  (Fly, Render, …) fonctionne — le Dockerfile est standard.
- **Image** : multi-stage, base runtime `debian:bookworm-slim`,
  objectif < 100 Mo.
- **Port** : respecte la variable d'env `PORT` (défaut `8080`).
- **Volume** : `/data` déclaré `VOLUME` pour signaler le montage
  persistant attendu.
- **Authentification dashboard** : HTTP Basic Auth, credentials lus
  au boot dans `POIS_ADMIN_USER` / `POIS_ADMIN_PASS`. Absence/vide
  = refus de démarrer avec code non nul. `/health` reste public pour
  les probes.

## Inspirations (non contractuelles)

- **nanobot** (HKUDS) — <https://github.com/HKUDS/nanobot> : concepts
  de *soul*, *heartbeat*, *channels*, *provider routing*, MCP.
- **OpenClaw** — <https://github.com/openclaw/openclaw> : architecture
  d'agent, orchestration d'outils.

Ni l'un ni l'autre n'est porté ligne à ligne ; leurs SHA / tags ne sont
pas épinglés.
