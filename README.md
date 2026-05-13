# rhythia-rp-finder

Interface TUI pour trouver les maps **Rhythia** dont le Max RP se trouve dans une fourchette donnée. Les maps ranked sont chargées depuis le site, puis tu filtres en temps réel sans relancer le programme.

## Installation rapide (binaire précompilé)

Télécharge le binaire depuis la [page Releases](../../releases/latest), rends-le exécutable et lance-le :

```bash
chmod +x rhythia-rp-finder
./rhythia-rp-finder
```

## Compiler depuis les sources

Prérequis : Rust stable ≥ 1.75 — [rustup.rs](https://rustup.rs)

```bash
git clone https://github.com/N3koNoTsuki/Rhythia_rp_finder
cd Rhythia_rp_finder
cargo build --release
./target/release/rhythia-rp-finder
```

## Utilisation

Au lancement, le programme charge toutes les maps ranked (barre de progression). Une fois chargé, l'interface interactive s'affiche :

```
┌─ Min RP ──────────┐┌─ Max RP ──────────┐┌─ Tri ─────────────┐
│ 150▌               ││ 300               ││ ◀ Plays ▶         │
└───────────────────┘└───────────────────┘└───────────────────┘
 87 maps trouvées  (sur 475 chargées)
──────────────────────────────────────────────────────────────
▶ [#1] Jack Black - Steve's Lava Chicken
    Mapper: chidodou  │  RP: 124  │  ⭐ 4.98  │  Plays: 8 645  │  0:30
    https://www.rhythia.com/maps/9891

  [#2] Hiiragi Magnetite - Tetoris (ft. Kasane Teto)
    Mapper: worstghostplayer  │  RP: 189  │  ⭐ 6.15  │  Plays: 6 065  │  2:20
    https://www.rhythia.com/maps/8293
```

### Raccourcis clavier

| Touche | Action |
|--------|--------|
| **Tab** / **Shift+Tab** | Changer de champ actif |
| **0–9** | Saisir une valeur dans Min RP ou Max RP |
| **Backspace** | Effacer le dernier chiffre |
| **←** / **→** ou **Enter** | Changer le tri (Plays ↔ Date) — quand Tri est actif |
| **↑** / **↓** ou **k** / **j** | Naviguer dans la liste de résultats |
| **Page Up** / **Page Down** | Naviguer par blocs de 10 |
| **q** ou **Esc** | Quitter |

### Champs de filtre

- **Min RP** — borne basse inclusive (vide = 0)
- **Max RP** — borne haute inclusive (vide = illimité)
- **Tri** — `Plays` (plus joué en premier) ou `Date` (plus récent en premier)

Le champ actif est surligné en jaune. Les résultats se mettent à jour à chaque frappe.

## Formule Max RP

```
max_rp = round(star_rating² × 5)
```

C'est le RP obtenu à 100 % de précision.

## Architecture

```
src/
├── main.rs    — TUI (ratatui), état, événements, rendu
├── api.rs     — fetch paginé via POST /api/getBeatmaps (reqwest blocking)
└── models.rs  — structs Map, ApiPage, ApiMap
```

Toutes les maps ranked sont chargées en mémoire au démarrage. Le filtrage et le tri sont faits localement — aucune requête supplémentaire n'est envoyée pendant la navigation.
