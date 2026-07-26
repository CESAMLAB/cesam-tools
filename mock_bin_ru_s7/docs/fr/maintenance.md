# Maintenance — Régulateur S7 (ORSS)

*🌍 **FR** · [EN](../en/maintenance.md) · [DE](../de/maintenance.md) · [ES](../es/maintenance.md) · [IT](../it/maintenance.md) · [PT](../pt/maintenance.md) · [NL](../nl/maintenance.md) · [PL](../pl/maintenance.md)*

---

## 1. Build & lancement

```bash
cargo run -p mock_bin_ru_s7                        # IHM + serveur S7
cargo build -p mock_bin_ru_s7 --release            # exécutable IHM
cargo build -p mock_bin_ru_s7 --no-default-features # headless (sans IHM)
```

Features : `gui` (IHM `egui`, par défaut). `--no-default-features` produit un binaire
**headless** : serveur S7 + simulation, sans IHM ni vérification de MAJ.

⚠️ Le port **102** (S7 standard) est privilégié (< 1024) : exécuter avec les droits
adéquats ou choisir un port haut dans la configuration.

## 2. Configuration

Fichier TOML `mock_ru_s7.toml` (répertoire courant ; chemin surchargeable par
`MOCK_CONFIG`). Sections : `language`, `[network]` (`bind_ip`, `port`, `allowlist`),
`[process]`, `[regulation]`, `check_updates`. Toute valeur est **assainie** au
chargement.

## 3. Tests

```bash
cargo test -p mock_bin_ru_s7      # unitaires + round-trip TCP local
```

- **Couche protocole** (`s7_server`, sans réseau) : CR→CC, Setup, Read/Write Var,
  écriture de bit, code retour hors-zone, **non-panique** sur trames malformées,
  round-trip de l'image DB.
- **Acteur réseau** : bind/écoute, et un **round-trip TCP réel** (connexion COTP,
  écriture puis relecture de la consigne via trames S7 brutes) — sans dépendance à un
  client externe.

## 4. Dépannage

| Symptôme | Piste |
|---|---|
| Bind échoue (`permission denied`) | port 102 < 1024 → droits root ou port haut |
| Client refusé | liste blanche d'IP ; pare-feu ; IP/port |
| Pas de réponse | rack/slot (tester 0/1, 0/2) ; trames hors sous-ensemble ignorées |
| Écriture sans effet | offset en lecture seule (cf. plan d'adressage) |

## 5. Docker (headless)

Image headless via `scripts/build-prod.sh` (entrée `mock_bin_ru_s7:ru_s7:102`,
`EXPOSE 102`). Monter un volume sur le répertoire de travail pour fournir le
`mock_ru_s7.toml`. Le conteneur publie le port 102 ; mapper vers un port haut côté
hôte si nécessaire.

## 6. Étendre le plan d'adressage

Le plan DB1 et le mapping des écritures sont la **source de vérité** dans
[`s7_server.rs`](../../src/s7_server.rs) (`db_image` + `handle_write`). Pour ajouter
une grandeur : l'écrire dans `db_image` (lecture) et, si pilotable, l'ajouter au
`match` de `handle_write` (écriture → `Command`), puis refléter ici et dans
[`reference_s7.md`](reference_s7.md). Ajouter un test dans le module.

## 7. Cross / Windows

Comme les autres instruments (cf. `Cross.toml`). Aucune dépendance native
particulière : la couche S7 est 100 % Rust sur TCP standard.
