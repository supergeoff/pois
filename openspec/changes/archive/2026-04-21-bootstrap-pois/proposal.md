## Why

Le dépôt `pois` doit devenir un compagnon IA personnel Rust multi-agent, contrôlable depuis une CLI locale et un dashboard web servable derrière une URL, déployable en une image Docker auto-hébergeable (cible Railway). Il s'inspire — librement, pas comme un portage fidèle — de [HKUDS/nanobot](https://github.com/HKUDS/nanobot) et d'[OpenClaw](https://github.com/openclaw/openclaw) pour les concepts de *soul*, *heartbeat*, *skills*, *channels* et *provider routing*.

Avant d'écrire toute logique d'agent, il faut (1) figer les fondations techniques (toolchain, stack, conventions, layout), et (2) poser un **squelette roulant** — binaire qui se lance, sert un endpoint `/health` protégé par basic auth, lit son volume `/data/` — pour valider de bout en bout la chaîne Docker/Railway/volume/auth avant de remplir le contenu métier.

## What Changes

- Figer la toolchain Rust (édition 2024, MSRV 1.95) via `rust-toolchain.toml`.
- Adopter un **layout mono-crate** (`pois`) avec modules plutôt qu'un workspace multi-crate — cohérent avec un outil personnel mono-binaire ; l'extraction en crates viendra par proposition dédiée si une frontière émerge.
- Figer la stack transverse : `tokio` (runtime unique), `axum` + `askama` + `htmx` (dashboard SSR), `serde` + `toml` (config), `clap` (CLI), `tracing` (logs).
- Figer la cible de distribution : binaire Linux statique empaqueté dans une image Docker, volume `/data/` monté, port via `PORT` (env), dashboard protégé par basic auth (`POIS_ADMIN_USER` / `POIS_ADMIN_PASS`).
- Documenter le **schéma de `/data/`** comme source de vérité d'état : `/data/config.toml` (global), `/data/agents/<id>/` (par agent : `config.toml`, `SOUL.md`, `HEARTBEAT.md`, `tools/`), `/data/honcho/`, `/data/logs/`.
- Trancher que la CLI et le gateway partagent le même binaire et la même arborescence `/data/` (pas d'IPC) : le réglage distant passe exclusivement par le dashboard web.
- Implémenter le **squelette roulant** : sous-commande `pois gateway` qui lance un serveur axum avec basic auth et une route `/health` ; les autres sous-commandes (et toute logique d'agent) restent stubs jusqu'à une proposition ultérieure.
- Produire un `Dockerfile` multi-stage minimal (build Rust → image distroless ou alpine), `ENTRYPOINT = ["/usr/local/bin/pois", "gateway"]`.
- Réécrire `openspec/project.md` pour remplacer la section « portage de nanobot » par la vision actuelle (inspiration libre, multi-agent, docker/railway).
- Introduire la capability `project-foundations` qui capture sous forme de requirements vérifiables les invariants techniques et produits du projet.

**Explicitement HORS scope** de cette proposition :
- boucle d'agent, gestion multi-agent réelle, channels (Telegram/WS), provider OpenRouter, client MCP, client Honcho, schéma `SOUL.md` / `HEARTBEAT.md`, routes UI du dashboard au-delà de `/health` et de la page d'accueil. Chacun aura sa propre proposition.

## Capabilities

### New Capabilities

- `project-foundations` : socle non-fonctionnel et produit du projet — identité produit (compagnon CLI+web Rust multi-agent), toolchain Rust, layout de crate, runtime async, conventions transverses, schéma du volume `/data/`, cible de déploiement Docker/Railway, authentification du dashboard, politique d'évolution via OpenSpec.

### Modified Capabilities

<!-- Aucune spec existante — openspec/specs/ est vide. -->

## Impact

- **Code produit** : binaire qui compile, démarre, sert `/health` derrière basic auth, lit son volume `/data/`. Aucune logique d'agent, de channel, de provider ou de MCP.
- **Fichiers créés** : `rust-toolchain.toml`, `Cargo.toml`, `Cargo.lock` (commité), `.gitignore`, `Dockerfile`, `.dockerignore`, `src/main.rs`, `src/lib.rs`, modules stubs (`src/cli/`, `src/gateway/`, `src/config/`, `src/data/`), `templates/` (askama, au moins `base.html` et `index.html`). Pas de répertoire `static/` au bootstrap : htmx et pico.css sont chargés depuis CDN, la bascule vers vendoring passera par une proposition dédiée.
- **Fichiers modifiés** : `openspec/project.md` réécrit intégralement.
- **Dépendances Rust introduites** : `tokio`, `axum`, `tower-http`, `askama`, `serde`, `serde_derive`, `toml`, `clap` (avec `derive`), `tracing`, `tracing-subscriber`, `thiserror`, `anyhow` (surface main uniquement). Versions précises arrêtées dans `design.md` / `tasks.md`.
- **Dépendances NON introduites** (explicitement reportées) : `teloxide`, `rmcp`, `reqwest`, `async-openai`, client Honcho, crate de base de données, `notify`. Chacune arrivera avec la proposition qui la justifie.
- **CI** : non couverte ici — la mise en place de GitHub Actions fera l'objet d'une proposition dédiée (`bootstrap-ci` ou équivalent) une fois le squelette validé localement.
- **Questions ouvertes tranchées par cette proposition** : les 4 du `project.md` d'origine (runtime async, surface d'API, cible de distribution, modèle de concurrence) sont tranchées dans `design.md`.
