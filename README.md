# Unicontacts

Application Android de gestion unifiée des contacts, avec un cœur métier écrit en Rust.

## Principe

Un référentiel maître unique contient les contacts, gérés comme un seul bloc. Les sources (CardDAV, Google, SIM, stockage local…) ne sont que des cibles de synchronisation. Toute divergence non triviale est soumise à l'utilisateur.

## Spécification

Voir [docs/SPEC.md](docs/SPEC.md) — spécification fonctionnelle et technique v0.1 (2026-11-14).

## État

**Conception** — aucun code métier pour l'instant. Le dépôt contient uniquement le squelette du workspace Cargo et la spécification.

## Structure

```
unicontacts/
├── Cargo.toml            # workspace
├── crates/
│   └── core/             # unicontacts-core : modèle, stockage, fusion, décisions
├── docs/
│   └── SPEC.md
├── README.md
└── .gitignore
```
