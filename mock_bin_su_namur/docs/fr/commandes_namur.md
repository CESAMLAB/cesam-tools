# Jeu de commandes NAMUR — Agitateur simulé (OSNE)

*🌍 **FR** · [EN](../en/commandes_namur.md) · [DE](../de/commandes_namur.md) · [ES](../es/commandes_namur.md) · [IT](../it/commandes_namur.md) · [PT](../pt/commandes_namur.md) · [NL](../nl/commandes_namur.md) · [PL](../pl/commandes_namur.md)*

> Crate : `mock_bin_su_namur` · Exécutable : **OSNE** · Protocole : **NAMUR** (ASCII, esclave)

Référence fonctionnelle du protocole. La **source de vérité technique** est
l'en-tête de [`src/namur.rs`](../../src/namur.rs).

---

## 1. Généralités

| Élément | Valeur |
|---------|--------|
| Transport | **TCP** (port `4001` par défaut) ou **série RS-232** (feature `serial`) |
| Rôle | **Esclave** (répond aux requêtes du maître) |
| Trame | une **ligne ASCII** par requête, terminée par `CR LF` |
| Lectures | `IN_*` → renvoient `valeur canal` (ex. `1200.0 4`) |
| Écritures / actions | `OUT_*`, `START_*`, `STOP_*`, `RESET` → **silencieuses** (pas de réponse) |
| Maîtres | **un seul à la fois** (point-à-point) ; en TCP un nouveau maître patiente jusqu'à la déconnexion du précédent |
| Filtrage | liste blanche d'IP optionnelle (TCP) |

> Réglage série NAMUR typique : **9600 bauds, 7 bits, parité paire, 1 stop (7E1)**.

### Canaux

| Canal | Grandeur | Unité |
|-------|----------|-------|
| `4` | Vitesse | tr/min |
| `5` | Couple | N·cm |

---

## 2. Commandes

| Commande | Type | Effet | Réponse |
|----------|------|-------|---------|
| `IN_NAME` | lecture | Nom de l'appareil | `CESAM-STIRRER` |
| `IN_TYPE` | lecture | Type d'appareil | `OSNE` |
| `IN_SW_VERSION` | lecture | Version du firmware simulé | ex. `0.1.0` |
| `IN_PV_4` | lecture | Vitesse **mesurée** | `<v> 4` |
| `IN_PV_5` | lecture | Couple **mesuré** | `<c> 5` |
| `IN_SP_4` | lecture | Consigne de vitesse | `<v> 4` |
| `OUT_SP_4 <v>` | écriture | **Régler** la consigne de vitesse (tr/min) | — |
| `START_4` | action | Démarrer le moteur | — |
| `STOP_4` | action | Arrêter le moteur | — |
| `RESET` | action | Arrêt + retour en commande locale | — |
| `OUT_WD1@<m>` | écriture | **Chien de garde** : arrêt sûr si aucune commande pendant `<m>` s | — |
| `OUT_WD2@<m>` | écriture | Chien de garde (idem v1 : arrêt sûr) | — |

> Toute commande inconnue ou argument invalide est **ignoré** (aucune réponse) et
> journalisé en `debug`.

### Chien de garde

Après `OUT_WD1@30`, si **aucune ligne** n'arrive pendant 30 s, le moteur est
**arrêté** (`STOP`) automatiquement — protection en cas de perte de communication
avec le superviseur. `OUT_WD1@0` désarme le chien de garde. Le compteur est
**réarmé à chaque commande reçue**.

---

## 3. Exemples (`nc` / netcat)

```text
$ nc 127.0.0.1 4001
IN_NAME
CESAM-STIRRER
OUT_SP_4 1200          (silencieux)
START_4                (silencieux)
IN_PV_4
1200.0 4
IN_PV_5
62.0 5
STOP_4                 (silencieux)
```

> Le **couple** lu croît avec la **viscosité** réglée (côté IHM) et la vitesse :
> `couple ≈ coeff_charge · viscosité · vitesse + frottement`. À forte viscosité, le
> couple sature au maximum moteur : la vitesse de consigne n'est plus atteinte
> (**surcharge**), comportement reproduisant un agitateur réel.
