## Context

`pois` est conçu comme un compagnon IA personnel Rust multi-agent, mono-utilisateur, auto-hébergé. Il combine :

- une **CLI locale** pour jouer, diagnostiquer et lancer le serveur ;
- un **gateway HTTP** (dashboard SSR) pour la configuration à distance, accessible sur une URL publique Railway ;
- un **pool d'agents** long-running, chaque agent ayant sa propre *soul*, sa propre mémoire (via Honcho), son propre *heartbeat*, et son propre channel (Telegram ou WebSocket) — propre ou partagé avec un autre agent ;
- un **volume `/data/`** qui contient la totalité de l'état persistant.

Le projet s'inspire librement de `nanobot` (HKUDS) et d'`OpenClaw` pour les concepts (soul, heartbeat, skills/outils, routing de providers, MCP), mais ne vise pas la parité API et n'épingle aucune version amont.

Cette proposition couvre les **fondations** et pose un **squelette roulant** (binaire qui compile, Docker image qui démarre, `/health` répond derrière basic auth). Aucune logique métier d'agent, aucun channel, aucun provider, aucun MCP, aucune mémoire.

### Contraintes

- **Mono-utilisateur** : toute la conception peut sacrifier multi-tenancy et auth fine.
- **Déploiement Railway** : l'image doit respecter `PORT`, supporter `/data` comme volume monté, démarrer vite, être petite.
- **Self-host universel** : doit pouvoir tourner localement avec `cargo run` OU en docker quelconque, sans service externe obligatoire au boot.
- **Itération rapide** : je suis seul dev ; les conventions ne doivent pas freiner le prototypage.
- **Pas de CI encore** : la validation se fait à la main ; la CI arrivera dans une proposition dédiée une fois le squelette stabilisé.

### Stakeholders

Un unique utilisateur (moi) à ce stade. Pas de consommateur externe de bibliothèque. Pas de contrainte d'API publique stable.

## Goals / Non-Goals

**Goals**

- Figer toolchain, stack, conventions transverses, layout et cible de déploiement en requirements testables.
- Fournir un binaire qui **démarre** en mode gateway, répond à `/health`, protège le reste par basic auth, lit `/data/`.
- Fournir un `Dockerfile` qui produit une image déployable sur Railway en l'état.
- Poser la structure `src/` (modules stubs) de sorte que les propositions suivantes aient un squelette clair où brancher leur code.
- Rendre toute dérive future des fondations visible via OpenSpec.

**Non-Goals**

- Boucle d'agent, schéma `SOUL.md`, schéma `HEARTBEAT.md`, spawn d'agent — proposition `port-agent-loop` (ou équivalent).
- Client Honcho — proposition `integrate-honcho`.
- Client MCP (local stdio + remote HTTP) — proposition `integrate-mcp`.
- Channels Telegram / WebSocket — proposition `port-channels`.
- Provider OpenRouter avec `transforms` + `reasoning` — proposition `port-provider-openrouter`.
- Schéma `config.toml` global + local — proposition `port-config`.
- Routes UI du dashboard au-delà de `/health` et d'une page d'accueil vide — proposition `port-dashboard`.
- Mise en place de CI / release pipeline — proposition `bootstrap-ci`.
- Publication sur crates.io — hors sujet (projet perso).

## Decisions

### D1. Inspiration libre, pas de portage épinglé

**Choix** : nanobot et OpenClaw sont cités comme inspirations dans `project.md` et dans l'intro du spec, sans SHA ni tag épinglé. Les concepts qu'on reprend (soul, heartbeat, channels, providers, MCP, skills) seront adaptés à la surface Rust de `pois`, pas traduits ligne à ligne.

**Alternatives considérées** :
- *Portage fidèle avec SHA épinglé* : rejeté — nanobot pousse quotidiennement, la parité continue est coûteuse et sans valeur pour un outil perso.
- *Forker nanobot et wrapper* : rejeté — langages différents, architectures différentes.

### D2. Layout mono-crate

**Choix** : un unique crate `pois` (bibliothèque + binaire), avec modules sous `src/` :

```
pois/
├── Cargo.toml
├── Cargo.lock              (commité)
├── rust-toolchain.toml
├── Dockerfile
├── .dockerignore
├── .gitignore
├── src/
│   ├── main.rs             # clap + dispatch sous-commandes
│   ├── lib.rs              # re-exports publics (tests d'intégration)
│   ├── cli/                # sous-commandes CLI (stubs pour l'instant)
│   │   ├── mod.rs
│   │   └── gateway.rs      # sous-commande `pois gateway`
│   ├── gateway/            # serveur axum
│   │   ├── mod.rs
│   │   ├── auth.rs         # middleware basic auth
│   │   ├── health.rs       # route /health
│   │   └── views.rs        # page d'accueil askama
│   ├── config/             # chargement + schéma (stubs)
│   │   └── mod.rs
│   ├── data/               # gestion du répertoire /data/
│   │   └── mod.rs
│   └── errors.rs           # AppError (thiserror) si utile transverse
├── templates/              # askama
│   ├── base.html
│   └── index.html
├── static/                 # htmx + css éventuel
│   └── htmx.min.js         # vendorisé
└── openspec/
```

**Rationale** :

- Un seul utilisateur, un seul binaire, un seul processus. Multi-crate ajoute de la cérémonie Cargo sans valeur pour ce profil.
- Les frontières conceptuelles (`gateway`, `config`, `data`, futur `agent`, `channels`, `providers`, `mcp`) vivent comme modules. Le jour où l'un prend du poids et mériterait un crate indépendant, ce sera tranché par proposition dédiée.

**Alternatives** :
- *Workspace 2 crates (runtime/bin)* : repoussé — cérémonie sans bénéfice tant qu'il n'y a pas un second consommateur du runtime.
- *Workspace fin-grained (5+ crates)* : repoussé — encore plus de cérémonie, encore moins justifié.

### D3. Toolchain : édition 2024, MSRV 1.85, stable

**Choix** : `rust-toolchain.toml` épingle `channel = "1.85.0"`, `components = ["rustfmt", "clippy"]`, `profile = "minimal"`. Le `Cargo.toml` déclare `edition = "2024"` et `rust-version = "1.85.0"`.

**Rationale** : 1.85 a stabilisé l'édition 2024 en février 2025 ; 14 mois après, l'édition 2024 est mûre. Pas de raison de se priver.

### D4. Stack web : axum + askama + htmx, SSR

**Choix** : `axum` pour le serveur HTTP, `tower-http` pour middlewares (trace, basic auth via extractor custom ou `ValidateRequestHeaderLayer`), `askama` pour templates typés à la compilation, `htmx` vendorisé en `static/` pour l'interactivité côté client.

**Rationale** :

- Pas de build JS, pas de SPA, pas de bundler. Tout est Rust + HTML + un fichier JS de ~15 Ko.
- Images Docker petites.
- Feedback loop dev rapide : askama détecte les templates au compile time.

**Alternatives** :
- *JSON API + SPA Svelte/React* : rejeté — deux toolchains, deux pipelines de build, overkill mono-user.
- *Leptos / Dioxus full-stack* : rejeté — encore jeune, plus de complexité que htmx pour le gain visé.

### D5. Config : TOML via serde

**Choix** : format TOML partout (`/data/config.toml` global, `/data/agents/<id>/config.toml` local). Crate `toml` + `serde`. Le schéma exact est HORS scope ici.

**Rationale** : TOML est lisible à la main, standard Rust, supporté nativement par Cargo — cohérent avec l'ADN du projet.

### D6. Authentification du dashboard : HTTP Basic Auth

**Choix** : middleware axum qui vérifie `Authorization: Basic …` contre les env vars `POIS_ADMIN_USER` et `POIS_ADMIN_PASS`. Comparaison à temps constant via `subtle::ConstantTimeEq` (ou équivalent). Si les env vars sont absentes ou vides au boot, le binaire refuse de démarrer. `/health` est exclu du middleware.

**Rationale** :

- HTTPS est fourni par Railway côté ingress, donc basic auth suffit.
- Aucune gestion de comptes, pas de base users, pas de sessions — cohérent mono-user.

**Alternatives** :
- *Token statique* (env var, envoyée en header custom ou cookie) : équivalent fonctionnellement mais moins intégré aux navigateurs (prompt basic auth natif, utile pour debug).
- *OAuth / SSO* : overkill.

### D7. Persistance : un unique répertoire `/data/`

**Choix** : la variable d'env `POIS_DATA_DIR` (défaut `/data`) désigne le répertoire racine. Schéma défini dans le spec. Au boot, le runtime crée les sous-répertoires manquants (`agents/`, `honcho/`, `logs/`) sans toucher `config.toml` global s'il existe.

**Rationale** : Railway monte un volume persistant à un chemin configurable — `/data/` est la convention la plus courante. Un seul point de vérité simplifie backups (tar) et rollback.

### D8. Provider OpenRouter : client thin maison, extensions `transforms` et `reasoning`

**Choix — décision à valeur de flag, pas implémentée ici** : quand arrivera la proposition de portage du provider, on implémentera un client HTTP `reqwest` minimal ciblant l'API OpenRouter directement, en modélisant explicitement les champs `transforms` (liste de transforms OR) et `reasoning` (config thinking tokens) en plus de l'API OpenAI chat/completions.

**Rationale** : `async-openai` modélise l'API OpenAI vanilla ; les extensions OR vivraient dans un wrapper. Un client thin `reqwest` + types serde est plus simple à maintenir que patcher `async-openai`.

**Alternatives** : utiliser `async-openai` + champs `serde_json::Value` pour les extras — rejeté, perd la vérification type.

### D9. Concurrence : un agent = une tâche tokio

**Choix** : chaque agent du pool tourne comme une `tokio::task::JoinHandle`. Les outils I/O sont naturellement async. Les outils shell bloquants passeront par `tokio::process::Command` ou `tokio::task::spawn_blocking`. Le pool est stocké dans un `DashMap<AgentId, AgentHandle>` (ou équivalent lock-free) ou un `RwLock<HashMap<…>>` suivant les contentions réelles — à trancher quand on aura le code.

**Rationale** : le modèle le plus léger de l'écosystème tokio. Si un agent devient CPU-bound (embedding local, etc.), on pourra migrer vers un thread dédié sans casser l'API publique.

### D10. Conventions relaxées

**Choix** :

- `cargo fmt --check` + `cargo clippy --all-targets -- -D warnings` sont la référence.
- `unwrap_used = "deny"` N'EST PAS activé à ce stade — trop de friction en prototypage mono-user.
- `thiserror` obligatoire pour les types d'erreurs exportés par des modules runtime ; `anyhow` autorisé dans `main.rs` et l'init.
- `unwrap()` / `expect()` tolérés si commentés `// SAFETY:` ou `// NOTE:`.

**Rationale** : on veut de la rigueur là où ça compte (API lib interne, frontières entre modules) et de la souplesse là où ça ne compte pas (main + init).

### D11. CLI et gateway partagent le binaire et `/data/`, pas d'IPC

**Choix** :

- Le binaire `pois` a plusieurs sous-commandes via `clap` : `pois gateway` (lance le serveur), `pois --help`, `pois --version`, et des sous-commandes locales futures qui opéreront directement sur `/data/` (ex. `pois agent spawn`, `pois config edit`).
- Le Dockerfile fait `ENTRYPOINT ["/usr/local/bin/pois", "gateway"]`.
- Il n'y a **pas d'IPC** entre une CLI locale et un gateway qui tournerait ailleurs. Le réglage à distance passe exclusivement par le dashboard web.

**Rationale** :

- Modèle simple : un fichier d'état (`/data/`), un binaire.
- Pas besoin de socket unix, d'API de management HTTP séparée, ni de protocole CLI↔server.
- Si un jour je veux contrôler une instance distante depuis mon laptop, j'utilise le dashboard. Suffisant.

**Conséquence architecturale** : le gateway doit tolérer des modifications de `/data/` faites hors-processus (par une CLI ou par un humain éditant des fichiers). Stratégie de base : relire `/data/` au démarrage uniquement ; hot-reload via `notify` sera introduit par proposition dédiée si le besoin se confirme.

### D12. Logging : tracing + tracing-subscriber

**Choix** : `tracing` comme façade ; `tracing-subscriber` avec formatter JSON en production (activé via env `POIS_LOG_FORMAT=json`) et formatter compact en dev (défaut). Niveau par défaut `info`, surchargable via `RUST_LOG`.

**Rationale** : standard Rust async, bien intégré avec axum/tokio/reqwest.

## Risks / Trade-offs

- **[Mono-crate dégénère en monolithe]** quand le code grossira, les modules `agent`, `channels`, `providers`, `mcp` vont se cross-importer et rendre le refactor coûteux. → *Mitigation* : la politique « ajouter un crate passe par proposition » exige qu'on pense l'extraction dès qu'une frontière devient claire. Le premier candidat sera probablement `providers` ou `mcp`.
- **[Basic auth perçue comme faible]** la tentation sera de « vite » ajouter un login page, OAuth, etc. → *Mitigation* : le spec borne ça. Tout changement demande une proposition qui justifie la valeur ajoutée.
- **[Schéma `/data/` gravé trop tôt]** on a figé la structure sans avoir écrit un seul agent — elle peut se révéler fausse. → *Mitigation* : le spec liste les sous-dossiers (`agents/`, `honcho/`, `logs/`) et dit explicitement que le schéma interne de chaque fichier est hors scope. Les propositions suivantes peuvent amender via `MODIFIED Requirements` quand l'usage montre un besoin.
- **[htmx vendorisé vieillit]** il faut mettre à jour la version périodiquement. → *Mitigation* : version écrite dans un commentaire de `static/htmx.min.js` ; upgrade = proposition dédiée.
- **[askama requiert recompilation à chaque changement de template]** peut ralentir le dev. → *Mitigation* : c'est le prix à payer pour la sûreté de type ; `cargo watch` + les templates petits limitent la gêne.
- **[Docker image trop grosse]** si on utilise `FROM rust` naïvement, l'image fait des centaines de Mo. → *Mitigation* : Dockerfile multi-stage — build dans `rust:1.85-slim`, copie du binaire dans `debian:bookworm-slim` ou `gcr.io/distroless/cc-debian12`.
- **[Railway incident / vendor lock-in]** si Railway disparaît, le spec mentionne Railway nommément. → *Mitigation* : le spec dit « compatible PaaS type Railway », et le Dockerfile est standard — tout autre PaaS (Fly, Render) marchera.
- **[MSRV 1.85 incompatible avec une future dep]** certaines crates récentes pourraient exiger 1.87+. → *Mitigation* : bump via proposition `bump-msrv`. Pas bloquant.

## Migration Plan

N/A — greenfield. Les fichiers créés par cette proposition sont tous nouveaux (sauf `openspec/project.md` qui est réécrit). Rollback = `git revert` de la PR.

## Open Questions

- **Faut-il un fichier `POIS.md` / `README.md` ?** Pour un outil perso, pas obligatoire. À trancher : je propose de ne pas en créer tant que le projet n'a pas un chemin d'usage stable. Un `README.md` naîtra naturellement avec la première proposition qui introduit une fonctionnalité visible pour un tiers.
- **`subtle` vs `constant_time_eq` vs comparaison naïve** pour basic auth : la différence est marginale sur un outil mono-user. Retenu : `subtle` (maintenu, audité, petit).
- **Port par défaut** : 8080 convient pour Railway. Confirmer en implémentation qu'il est overridable via `PORT`.
- **Politique de rotation des logs dans `/data/logs/`** : pas gérée ici — à traiter quand les logs deviendront un problème. Probablement via `tracing-appender` + roll quotidien, en proposition dédiée.
- **Licence du dépôt** : non décidée. MIT probable (alignement inspirations) mais hors scope ici.
- **Format de la commande `pois version`** : plate vs JSON — trivial, tranché en tasks.
