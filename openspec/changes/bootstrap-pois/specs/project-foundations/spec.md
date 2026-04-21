## ADDED Requirements

### Requirement: Product shape is documented

Le projet SHALL documenter, dans `openspec/project.md`, son identité produit : un compagnon IA personnel Rust multi-agent contrôlable par CLI locale et dashboard web. Cette documentation MUST préciser : les inspirations (`nanobot` et `OpenClaw`, citées en URL, sans contrat de parité API), la cible de déploiement (binaire empaqueté en image Docker, auto-hébergement type Railway), la nature mono-utilisateur (un seul opérateur par instance), et les éléments NON couverts au jour du cadrage (boucle d'agent, channels, providers, MCP, Honcho).

#### Scenario: project.md décrit le produit

- **WHEN** un contributeur lit `openspec/project.md`
- **THEN** il y trouve une section identifiant `pois` comme compagnon CLI + dashboard web Rust multi-agent, citant `nanobot` et `OpenClaw` comme inspirations non contractuelles, et précisant Docker/Railway comme cible

### Requirement: Rust toolchain is pinned

Le dépôt SHALL fournir, à la racine, un fichier `rust-toolchain.toml` qui épingle le canal `stable` à la version `1.85.0` (MSRV) et inclut les composants `rustfmt` et `clippy`. Le crate `pois` MUST déclarer `edition = "2024"` dans son manifeste.

#### Scenario: rust-toolchain.toml existe et active la toolchain

- **WHEN** un contributeur clone le dépôt avec `rustup` installé et lance `cargo --version`
- **THEN** la toolchain `1.85.0` stable est sélectionnée automatiquement

#### Scenario: Le crate cible l'édition 2024

- **WHEN** un contributeur inspecte `Cargo.toml`
- **THEN** le champ `package.edition` vaut `"2024"` et `package.rust-version` vaut `"1.85.0"`

### Requirement: Single-crate layout

Le dépôt SHALL être un unique crate Cargo nommé `pois`, avec une bibliothèque `src/lib.rs` et un binaire `src/main.rs`. Aucun workspace `[workspace]` ne DOIT exister à la racine tant qu'une frontière de dépendance n'a pas été formellement identifiée et actée par une proposition OpenSpec dédiée.

#### Scenario: Cargo.toml déclare un seul package nommé pois

- **WHEN** un outil analyse `Cargo.toml` racine
- **THEN** il trouve exactement une table `[package]` avec `name = "pois"`, et aucune table `[workspace]`

#### Scenario: Introduire un second crate exige une proposition

- **WHEN** un contributeur veut scinder `pois` en plusieurs crates
- **THEN** il ouvre une proposition OpenSpec qui amende cette exigence via un delta `MODIFIED Requirements`

### Requirement: Async runtime is tokio-only

Le crate `pois` SHALL dépendre de `tokio` comme unique runtime asynchrone. Toute dépendance transitive directe ou indirecte à `async-std` ou `smol` MUST être interdite au niveau du code applicatif du crate. Les features tokio nécessaires (`rt-multi-thread`, `macros`, `net`, `fs`, `signal`, `time`) MUST être activées explicitement plutôt que via `full`.

#### Scenario: tokio est déclaré et configuré

- **WHEN** un contributeur inspecte `Cargo.toml`
- **THEN** il trouve `tokio` avec un ensemble explicite de features, sans la feature `full`

#### Scenario: Aucun autre runtime async n'est introduit

- **WHEN** un contributeur exécute `cargo tree -i async-std` puis `cargo tree -i smol`
- **THEN** les deux commandes signalent que le crate cherché n'est pas dans le graphe de dépendances

### Requirement: Error handling convention

Le code du crate SHALL utiliser `thiserror` pour définir les types d'erreurs exportés par les modules de runtime (gateway, data, config, futur agent/channels/providers/mcp). Le point d'entrée `main` et le chemin d'initialisation MAY utiliser `anyhow` pour agréger les erreurs jusqu'à l'utilisateur. Les appels `unwrap()` et `expect()` MUST être justifiés par un commentaire `// SAFETY:` ou `// NOTE:` documentant l'invariant ; faute de quoi, ils MUST être remplacés par un retour d'erreur typé.

#### Scenario: Un module runtime expose un type d'erreur dédié

- **WHEN** un contributeur inspecte l'API publique d'un module runtime (par exemple `gateway` ou `config`)
- **THEN** les fonctions publiques qui peuvent échouer retournent `Result<T, ModuleError>` où `ModuleError` est un enum dérivant `thiserror::Error`

#### Scenario: Un unwrap non documenté est visible en revue

- **WHEN** un contributeur introduit `.unwrap()` sans commentaire `// SAFETY:` ou `// NOTE:` dans du code non-test
- **THEN** la revue exige soit le commentaire, soit le remplacement par un `?` ou une erreur typée

### Requirement: Formatting and lint policy

Le dépôt SHALL considérer `cargo fmt --check` et `cargo clippy --all-targets -- -D warnings` comme commandes de référence de style et de lint. La règle clippy `unwrap_used = "deny"` MUST NOT être activée globalement à ce stade afin de préserver la fluidité de prototypage ; l'activation viendra par proposition dédiée quand la surface se stabilisera.

#### Scenario: cargo fmt --check passe sur un dépôt propre

- **WHEN** un contributeur exécute `cargo fmt --check` sur une branche sans modification
- **THEN** la commande retourne un code 0

#### Scenario: Un warning clippy bloque la commande de référence

- **WHEN** du code introduit un warning clippy
- **THEN** `cargo clippy --all-targets -- -D warnings` retourne un code non nul

### Requirement: /data persistence layout

Le runtime SHALL considérer un répertoire racine `$POIS_DATA_DIR` (défaut : `/data`) comme source de vérité unique pour l'état persistant. Ce répertoire MUST avoir la structure suivante :

```
$POIS_DATA_DIR/
├── config.toml         # configuration globale (TOML)
├── agents/             # un sous-dossier par agent
│   └── <agent-id>/
│       ├── config.toml
│       ├── SOUL.md
│       ├── HEARTBEAT.md
│       └── tools/
├── honcho/             # état client Honcho (cache, tokens)
└── logs/               # traces runtime
```

Au démarrage, le binaire MUST créer les sous-répertoires manquants (`agents/`, `honcho/`, `logs/`) s'ils n'existent pas, sans toucher à `config.toml` s'il existe déjà. Le schéma détaillé des `config.toml`, de `SOUL.md` et de `HEARTBEAT.md` est HORS scope de cette capability et sera défini par des propositions dédiées.

#### Scenario: Le gateway crée la structure manquante au boot

- **WHEN** `pois gateway` démarre avec `POIS_DATA_DIR` pointant sur un répertoire vide
- **THEN** après le boot, les sous-répertoires `agents/`, `honcho/` et `logs/` existent, et `config.toml` n'est pas créé automatiquement

#### Scenario: POIS_DATA_DIR override le défaut

- **WHEN** le binaire démarre avec `POIS_DATA_DIR=/tmp/pois-dev`
- **THEN** toute lecture/écriture d'état se fait sous `/tmp/pois-dev`, jamais sous `/data`

### Requirement: Deployment target is Docker / Railway

Le dépôt SHALL fournir, à la racine, un `Dockerfile` et un `.dockerignore` qui produisent une image Linux contenant le binaire `pois`. L'image MUST :

- définir `ENTRYPOINT` sur une invocation qui lance `pois gateway` ;
- respecter la variable d'environnement `PORT` (défaut : `8080`) pour le port d'écoute du gateway, afin d'être compatible avec Railway et les PaaS similaires ;
- déclarer via `VOLUME` le chemin `/data` pour signaler que ce répertoire est destiné à être monté sur un volume persistant.

#### Scenario: docker build produit une image fonctionnelle

- **WHEN** un contributeur exécute `docker build -t pois .` à la racine
- **THEN** l'image se construit sans erreur et `docker run --rm pois --help` affiche la page d'aide CLI

#### Scenario: PORT est respecté

- **WHEN** `docker run -e PORT=3000 -e POIS_ADMIN_USER=u -e POIS_ADMIN_PASS=p pois` tourne
- **THEN** le gateway écoute sur le port 3000

### Requirement: Dashboard basic authentication

Le gateway SHALL protéger TOUTES les routes du dashboard web par HTTP Basic Authentication. Les credentials MUST être lus au démarrage depuis les variables d'environnement `POIS_ADMIN_USER` et `POIS_ADMIN_PASS`. Si l'une ou l'autre est absente ou vide, le binaire MUST refuser de démarrer avec un message d'erreur explicite vers stderr. La route `/health` MAY rester publique pour permettre les probes Railway.

#### Scenario: Le serveur refuse de démarrer sans credentials

- **WHEN** `pois gateway` est lancé sans `POIS_ADMIN_USER` ou sans `POIS_ADMIN_PASS`
- **THEN** le binaire écrit sur stderr un message indiquant les variables manquantes et quitte avec un code de sortie non nul

#### Scenario: Une requête dashboard sans credentials est rejetée

- **WHEN** un client HTTP appelle la route racine `/` sans header `Authorization`
- **THEN** le serveur répond `401 Unauthorized` avec un header `WWW-Authenticate: Basic realm="pois"`

#### Scenario: La route /health reste publique

- **WHEN** un client HTTP appelle `/health` sans credentials
- **THEN** le serveur répond `200 OK`

### Requirement: Foundation invariants evolve via OpenSpec

Toute modification d'un des invariants ci-dessus (forme produit, toolchain, layout de crate, runtime async, conventions d'erreurs, politique de lint, schéma `/data/`, cible de déploiement, authentification du dashboard) SHALL faire l'objet d'une proposition OpenSpec qui amende ce spec via un bloc `MODIFIED Requirements` ou `REMOVED Requirements`. Les modifications silencieuses de `openspec/project.md`, `rust-toolchain.toml`, `Cargo.toml`, `Dockerfile` touchant ces invariants MUST NOT être mergées sans proposition associée.

#### Scenario: Un PR touche un fichier d'invariant sans proposition

- **WHEN** un contributeur soumet un PR qui modifie `rust-toolchain.toml`, les sections d'invariant de `openspec/project.md`, ou les sections auth du `Dockerfile` sans proposition OpenSpec associée
- **THEN** la revue exige l'ouverture d'une proposition avant fusion
