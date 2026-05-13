# rhythia-rp-finder

CLI Rust pour lister les maps **Rhythia** dont le Max RP se trouve dans une fourchette définie.

## Prérequis

- Rust stable (≥ 1.75) — [rustup.rs](https://rustup.rs)

## Installation

```bash
git clone <repo>
cd rhythia-rp-finder
cargo build --release
# Le binaire est dans ./target/release/rhythia-rp-finder
```

## Usage

```bash
rhythia-rp-finder --low 175 --high 225
rhythia-rp-finder --low 100 --high 150 --sort date
```

### Options

| Flag | Description | Défaut |
|------|-------------|--------|
| `--low <N>` | Borne basse du Max RP (inclusive) | obligatoire |
| `--high <N>` | Borne haute du Max RP (inclusive) | obligatoire |
| `--sort <plays\|date>` | Critère de tri | `plays` |

### Navigation interactive

Après affichage d'une page :

```
[n]ext  [p]rev  [q]uit
```

### Exemple de sortie

```
Page 1/4 — 87 maps trouvées (RP 175-225)

[#1] Song Title — Artist
     Mapper: NomDuMapper  |  Max RP: 213  |  ⭐ 9.2  |  Plays: 12 453
     Tags: jump, stream   |  Durée: 2:34  |  BPM: 220
     🔗 https://www.rhythia.com/maps/1234
```

## Architecture

```
src/
├── main.rs      — parsing args (clap), orchestration
├── api.rs       — fetch + désérialisation JSON (reqwest blocking)
├── cache.rs     — stockage en mémoire, filtrage, tri
├── display.rs   — formatage terminal, couleurs ANSI, pagination
└── models.rs    — structs Map, ApiPage, ApiMeta, ApiMap
```

Au démarrage, l'outil charge **toutes** les maps ranked en mémoire (Vec\<Map\>), puis filtre/trie localement. Le cache est en mémoire uniquement — aucun fichier n'est écrit sur le disque.

## Formule Max RP

Dérivée du dépôt [cunev/rhythia-web-utils](https://github.com/cunev/rhythia-web-utils) :

```
max_rp = round((star_rating × 50)² / 1000)
       = round(star_rating² × 2.5)
```

C'est le RP obtenu à 100 % de précision.

## Note sur l'API

Rhythia n'a pas d'API publique documentée. Ce projet suppose un endpoint `GET /api/maps?page=N&limit=50&ranked=true`. Si la structure JSON diffère, adapter les alias `serde` dans `src/models.rs` (champs `ApiMap`).

L'outil respecte les codes 429 (rate limit) avec un backoff exponentiel (500 ms × 2^n, jusqu'à 4 tentatives).

## Variables d'environnement

| Variable | Effet |
|----------|-------|
| `NO_COLOR` | Désactive les couleurs ANSI |
| `TERM=dumb` | Désactive les couleurs ANSI |
