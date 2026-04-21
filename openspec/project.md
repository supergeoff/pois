# Project: pois

## Purpose

Un équivalent de nanobot écrit en Rust.

## Tech stack

- **Langage** : Rust
- **Édition / toolchain** : à préciser (édition 2021 ou 2024, stable)
- **Framework(s)** : à préciser (runtime async, HTTP, MCP, etc.)
- **Build** : Cargo (workspace à confirmer selon la taille)

## Project structure

À définir après les premières propositions OpenSpec. Structure Rust
typique envisagée :

```
pois/
├── Cargo.toml
├── src/
│   ├── main.rs          # binaire
│   └── lib.rs           # bibliothèque principale
├── tests/               # tests d'intégration
└── openspec/            # specs et propositions de changement
```

## Conventions

À compléter au fil des premières propositions. Pistes de départ :

- **Erreurs** : `Result<T, E>` avec un type d'erreur dédié (thiserror / anyhow).
- **Formatage** : `cargo fmt` + `cargo clippy -- -D warnings` en CI.
- **Tests** : TDD via la skill Superpowers `test-driven-development`.
- **Async** : runtime à choisir (tokio probable) — à inscrire dans
  la première proposition architecturale.

## External context

- **nanobot** : projet de référence que ce dépôt vise à réimplémenter
  en Rust. À documenter précisément (URL, version, surface d'API
  reprise) dans la première proposition OpenSpec d'onboarding.

## Open questions

- Runtime async : `tokio` vs `async-std` vs autre ?
- Surface d'API : port 1-pour-1 de nanobot ou réinterprétation Rust-idiomatique ?
- Cible de distribution : binaire statique, crate publiée, conteneur ?
- Modèle de concurrence et de configuration attendu ?
