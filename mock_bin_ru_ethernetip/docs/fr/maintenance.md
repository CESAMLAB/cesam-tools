# Maintenance — Régulateur EtherNet/IP (OREE)

*🌍 **FR** · [EN](../en/maintenance.md) · [DE](../de/maintenance.md) · [ES](../es/maintenance.md) · [IT](../it/maintenance.md) · [PT](../pt/maintenance.md) · [NL](../nl/maintenance.md) · [PL](../pl/maintenance.md)*

---

## 1. Build & lancement

```bash
cargo run -p mock_bin_ru_ethernetip                        # IHM + adaptateur EtherNet/IP
cargo build -p mock_bin_ru_ethernetip --release            # exécutable IHM
cargo build -p mock_bin_ru_ethernetip --no-default-features # headless (sans IHM)
```

Features : `gui` (IHM `egui`, par défaut). `--no-default-features` produit un binaire
**headless** : adaptateur EtherNet/IP + simulation, sans IHM ni vérification de MAJ.
Le port 44818 ne nécessite **aucun privilège**.

## 2. Configuration

Fichier TOML `mock_ru_ethernetip.toml` (répertoire courant ; chemin surchargeable par
`MOCK_CONFIG`). Sections : `language`, `[network]` (`bind_ip`, `port`, `allowlist`),
`[process]`, `[regulation]`, `check_updates`. Toute valeur est **assainie** au
chargement.

## 3. Tests

```bash
cargo test -p mock_bin_ru_ethernetip      # unitaires + round-trip TCP local
```

- **Couche protocole** (`eip_server`, sans réseau) : RegisterSession, Read/Write Tag,
  écriture BOOL, tag inconnu (`0x05`), écriture d'un tag lecture seule, **non-panique**
  sur paquets malformés.
- **Acteur réseau** : bind/écoute et un **round-trip TCP réel** (RegisterSession,
  Write puis Read de la consigne) — sans dépendance à un client externe.

## 4. Dépannage

| Symptôme | Piste |
|---|---|
| Client refusé | liste blanche d'IP ; pare-feu ; IP/port (44818) |
| Tag introuvable | nom inexact (casse) ; voir la table de tags |
| Écriture sans effet | tag en lecture seule |
| Valeurs incohérentes | EtherNet/IP est **little-endian** (REAL = `f32` LE) |

## 5. Docker (headless)

Image headless via `scripts/build-prod.sh` (entrée
`mock_bin_ru_ethernetip:ru_eip:44818`, `EXPOSE 44818`). Monter un volume sur le
répertoire de travail pour fournir le `mock_ru_ethernetip.toml`.

## 6. Étendre la table de tags

La table de tags et le mapping des écritures sont la **source de vérité** dans
[`eip_server.rs`](../../src/eip_server.rs) (`read_tag` + `write_tag`). Pour ajouter un
tag : l'ajouter à `read_tag` (lecture) et, si pilotable, à `write_tag` (écriture →
`Command`), puis refléter ici et dans
[`reference_ethernetip.md`](reference_ethernetip.md). Ajouter un test dans le module.

## 7. Cross / Windows

Comme les autres instruments (cf. `Cross.toml`). Aucune dépendance native
particulière : la couche EtherNet/IP est 100 % Rust sur TCP standard.
