# Manuel utilisateur — OSNE (agitateur de laboratoire simulé NAMUR)

*🌍 **FR** · [EN](../en/manuel_utilisateur.md) · [DE](../de/manuel_utilisateur.md) · [ES](../es/manuel_utilisateur.md) · [IT](../it/manuel_utilisateur.md) · [PT](../pt/manuel_utilisateur.md) · [NL](../nl/manuel_utilisateur.md) · [PL](../pl/manuel_utilisateur.md)*

> **OSNE** — *Open Stirrer NAMUR Emulator* · binaire `mock_bin_su_namur`
> (exécutable `osne`) · Licence MIT · Éditeur : **CESAM-Lab** · Identité NAMUR :
> nom `CESAM-STIRRER`, type `OSNE`.
>
> *Un agitateur de laboratoire (façon IKA) qui n'existe que sur votre liaison
> NAMUR — pour tester superviseurs, scripts et passerelles sans matériel réel.*

Ce manuel s'adresse à l'**utilisateur** de l'agitateur simulé : comment le lancer,
le piloter depuis l'interface, le paramétrer, et le raccorder en **NAMUR** (TCP ou
série RS-232). Aucune connaissance de programmation n'est nécessaire.

---

## 1. À quoi sert ce logiciel ?

Il simule un **agitateur de laboratoire** (agitateur de paillasse à hélice, façon
IKA) :

- un **moteur physique** réaliste : la vitesse monte/descend selon le couple
  appliqué, avec une **régulation de vitesse rapide** ;
- une **charge visqueuse réglable** : plus le milieu est visqueux, plus le couple
  nécessaire est élevé — jusqu'à la **surcharge** (consigne inatteignable) ;
- un **serveur NAMUR** (protocole série ASCII des appareils de labo) pour le
  piloter/superviser depuis un autre logiciel ou un script ;
- une **interface graphique** de conduite, de visualisation et de **test du
  protocole** (mini-terminal NAMUR intégré).

C'est un outil de **test** : il permet de mettre au point et de démontrer un
superviseur, un script d'acquisition ou une passerelle **sans matériel réel**.

---

## 2. Démarrer le logiciel

Lancer l'exécutable correspondant à votre système :

| Système | Fichier |
|---------|---------|
| Windows | `osne-windows-x86_64.exe` (double-clic) |
| Linux PC | `./osne-linux-x86_64` |
| Raspberry Pi (écran) | `./osne-rpi-arm64` |

La fenêtre s'ouvre et le **serveur NAMUR démarre automatiquement** (port `4001`
par défaut). L'en-tête indique l'état :

- **● EN MARCHE / ● À L'ARRÊT** : état du moteur ;
- **NAMUR ● 0.0.0.0:4001** (vert) : serveur à l'écoute ; **✖ …** (rouge) en cas
  de problème (port occupé, série indisponible…) ;
- un **voyant de connexion** : en TCP il affiche le maître connecté (ou « aucun
  maître »), en série un simple point. Il passe au **vert** lorsqu'une trame a été
  reçue récemment (lien actif), gris sinon.

> Sans écran (serveur seul), voir le **§ 9 (Utilisation sans écran)**.

---

## 3. L'interface en un coup d'œil

```
┌──────────────── En-tête : titre OSNE, ⚙ Paramètres, 💾 Sauvegarder, états & voyants ────────────────┐
├──────────────────┬──────────────────────────────────────────────────────────────────────────────────┤
│  COMMANDES        │   SUPERVISION                                                                      │
│  (gauche)         │   - cartes de valeurs (Vitesse / Couple / Viscosité / Surcharge)                  │
│  Marche/Arrêt     │   - COURBE de tendance temps réel (Consigne / Vitesse / Couple)                   │
│  Consigne vitesse │                                                                                   │
│  Viscosité        │                                                                                   │
│  Réglages PID     │                                                                                   │
├──────────────────┴──────────────────────────────────────────────────────────────────────────────────┤
│  ⇄ TRAMES NAMUR : mini-terminal (RX/TX) + ligne de commande + référence du protocole (à droite)       │
└───────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 4. Piloter l'agitateur (panneau de gauche)

### 4.1 Marche / Arrêt
Bouton **Marche / Arrêt**. À l'arrêt, le moteur décélère librement jusqu'à
l'immobilisation (frottement + charge), couple moteur nul.

### 4.2 Consigne de vitesse
Curseur **Consigne de vitesse** (en `tr/min`), borné par les limites min/max
réglées dans les *Paramètres*. C'est la même grandeur que la commande NAMUR
`OUT_SP_4` (canal 4). En marche, l'asservissement amène la vitesse mesurée vers
cette consigne.

### 4.3 Viscosité du milieu
Curseur **Viscosité** (échelle logarithmique). Il représente la **charge** du
milieu agité :

- viscosité **faible** → couple faible, la consigne est atteinte rapidement ;
- viscosité **élevée** → couple de charge important ; si le couple nécessaire
  dépasse le **couple moteur maximal**, la vitesse de consigne **n'est plus
  atteinte** → l'indicateur **Surcharge ⚠** s'allume (comportement d'un agitateur
  réel face à un milieu trop épais).

### 4.4 Réglages PID (Kp, Ki, Kd)
Les trois gains de l'asservissement de vitesse, ajustables en direct :

- **Kp** (proportionnel) : plus il est grand, plus la montée en vitesse est vive
  (risque de dépassement/oscillation) ;
- **Ki** (intégral) : annule l'écart résiduel de vitesse dans le temps ;
- **Kd** (dérivé) : amortit/anticipe (trop fort → sensible au bruit).

> Les gains par défaut sont volontairement « raides » : la sortie sature au couple
> maximal tant que l'erreur est grande (montée rapide), puis le terme intégral
> stabilise. La sortie du PID **est** le couple moteur, borné à `[0, couple_max]`.

---

## 5. Lire la courbe de tendance

La courbe (au centre) trace trois grandeurs en temps réel. La **légende, en haut à
gauche**, rappelle la couleur **et la dernière valeur** de chaque série :

| Couleur | Série | Signification |
|---------|-------|---------------|
| 🔵 bleu | **Consigne** | consigne de vitesse (en marche) |
| 🔴 rouge | **Vitesse** | vitesse mesurée (`tr/min`, axe de gauche) |
| 🟢 vert | **Couple** | couple mesuré (`N·cm`, **axe de droite**) |

> La courbe a **deux axes verticaux** : la **vitesse** (`tr/min`) à gauche, le
> **couple** (`N·cm`) à droite. Le couple est mis à l'échelle pour partager le
> graphe, mais l'axe de droite affiche bien des `N·cm`.

Au-dessus de la courbe, des **cartes** affichent les valeurs instantanées :
**Vitesse**, **Couple**, **Viscosité**, et **Surcharge ⚠** lorsque le moteur sature.
On peut zoomer/déplacer la courbe à la souris.

---

## 6. Le mini-terminal NAMUR (bas de fenêtre)

Le panneau **⇄ Trames NAMUR** permet de **tester le protocole** directement depuis
l'IHM, sans client externe :

- le **journal** affiche les trames **reçues** (`← RX`, bleu) et **émises**
  (`→ TX`, vert), horodatées ;
- la **ligne de commande** envoie une trame NAMUR au simulateur (touche **Entrée**
  ou bouton **▶ Envoyer**). Les flèches **↑/↓** rappellent les commandes
  précédentes (historique) ;
- la **référence du protocole** (panneau de droite) liste les commandes : un
  **clic** insère la commande dans la ligne de saisie ;
- le bouton **🗑 Effacer** vide le journal.

> Les trames tapées ici sont interprétées exactement comme celles d'un maître
> réseau : `OUT_SP_4 500` règle la consigne, `START_4`/`STOP_4` démarrent/arrêtent,
> `IN_PV_4` lit la vitesse, etc. Le **chien de garde** (`OUT_WD1@…`) n'a toutefois
> d'effet qu'au sein d'une vraie session réseau (cf. § 8).

---

## 7. Paramètres (bouton ⚙)

Le bouton **⚙ Paramètres** ouvre une fenêtre pour configurer :

### Langue de l'interface
Sélecteur en haut : **Français, English, Deutsch, Español, Italiano, Português,
Nederlands, Polski** (8 langues). La langue est persistée.

### Transport NAMUR
Choix de la liaison — **une seule active à la fois** :

**TCP (Ethernet)**
- **IP d'écoute** (`0.0.0.0` = toutes les interfaces) et **Port** (défaut 4001) ;
- **IP autorisées** : une par ligne, jokers `*` acceptés (ex. `192.168.1.*`).
  **Liste vide = toutes les IP autorisées.** Les autres sont refusées.

**Série (RS-232)** — nécessite un binaire compilé avec la feature `serial`
- **Port série** : `/dev/ttyUSB0` (Linux), `COM3` (Windows)… ;
- **Baud** (défaut 9600), **Parité** (défaut Paire), **Bits de données** (7),
  **Bits de stop** (1) — réglage NAMUR de labo typique : **9600 7E1**.

> ⚠️ **Un seul maître à la fois.** En TCP, un nouveau maître **patiente** jusqu'à la
> déconnexion du précédent (liaison point-à-point). L'IHM locale n'est **pas** un
> maître. En série, le bus *est* l'unique maître ; privilégier une **liaison
> point-à-point** (le serveur répond quelle que soit l'adresse demandée).

### Paramètres moteur
Comportement physique simulé `J·dω/dt = T − k·η·ω − frottement` :
- **Inertie** (`J`) : réactivité du moteur (petit ⇒ rapide) ;
- **Coefficient de charge** (`k`) : poids de la viscosité sur le couple ;
- **Frottement** (`N·cm`) : frottement sec résiduel ;
- **Couple max** (`N·cm`) : couple moteur maximal (plafond de la sortie PID).

### Bornes de vitesse
Limites mini/maxi de la consigne de vitesse (`tr/min`).

### Bornes de viscosité
Limites mini/maxi du curseur de viscosité.

Boutons : **Appliquer** (prend effet immédiatement **et** enregistre),
**Réinitialiser par défaut**, **Fermer**.

### Enregistrement des réglages
Les réglages sont **sauvegardés** dans un fichier `mock_su_namur.toml` (à côté du
logiciel) et **rechargés au prochain démarrage**. Le bouton **💾 Sauvegarder** de
l'en-tête enregistre aussi les gains PID et la viscosité modifiés depuis le
panneau de gauche.

---

## 8. Raccorder un client NAMUR

Le logiciel est un **esclave NAMUR** (TCP port 4001 par défaut, ou série selon le
transport choisi au § 7). Un client (script, terminal, passerelle) **envoie une
ligne ASCII par requête**, terminée par `CR LF`. Les **lectures** (`IN_*`)
renvoient une valeur ; les **écritures/actions** (`OUT_*`, `START_*`, `STOP_*`,
`RESET`) sont **silencieuses** (pas de réponse), conformément à l'usage NAMUR.

Repères principaux :

| Commande | Effet |
|----------|-------|
| `IN_NAME` / `IN_TYPE` | identité (`CESAM-STIRRER` / `OSNE`) |
| `IN_PV_4` / `IN_PV_5` | lire la vitesse (`tr/min`) / le couple (`N·cm`) |
| `IN_SP_4` | lire la consigne de vitesse |
| `OUT_SP_4 <v>` | **régler** la consigne de vitesse |
| `START_4` / `STOP_4` / `RESET` | démarrer / arrêter / réinitialiser |
| `OUT_WD1@<m>` | **chien de garde** : arrêt sûr si silence pendant `<m>` s |

Exemple avec `nc` (netcat) :

```text
$ nc 127.0.0.1 4001
IN_NAME
CESAM-STIRRER
OUT_SP_4 1200          (silencieux)
START_4                (silencieux)
IN_PV_4
1200.0 4
STOP_4                 (silencieux)
```

> Le **chien de garde** `OUT_WD1@30` arrête automatiquement le moteur si **aucune
> ligne** n'arrive pendant 30 s (protection en cas de perte de communication).
> `OUT_WD1@0` le désarme. Le compteur est réarmé à chaque commande reçue.

> La **référence complète du protocole** (canaux, encodage, exemples) est dans
> **[commandes_namur.md](commandes_namur.md)**. La même liste est rappelée **en
> direct** dans le panneau de droite du mini-terminal.

---

## 9. Utilisation sans écran (« headless » / Docker)

Pour un déploiement en tâche de fond (Raspberry Pi sans écran, serveur), une
version **sans interface** existe : elle fait tourner la simulation et le serveur
NAMUR, pilotables **uniquement par NAMUR**.

```bash
# Image Docker (déployable n'importe où) :
docker run --rm -p 4001:4001 -v "$PWD/conf:/data" osne:headless
```

Le dossier monté sur `/data` permet de fournir/conserver `mock_su_namur.toml`.

---

## 10. Questions fréquentes

| Question / symptôme | Réponse |
|---------------------|---------|
| **Surcharge ⚠** s'allume et la vitesse n'atteint pas la consigne. | Normal : la **viscosité** demande plus de couple que le moteur n'en fournit. Baissez la viscosité ou la consigne, ou augmentez le **couple max** (Paramètres). |
| La vitesse ne bouge pas. | Vérifiez que l'agitateur est **En marche** et la consigne non nulle. |
| L'en-tête affiche **NAMUR ✖**. | Port déjà utilisé ou < 1024 sans droits (TCP), ou port série indisponible : changez le réglage dans ⚙ Paramètres. |
| Mon client NAMUR/TCP est refusé. | Son IP n'est pas dans la **liste blanche** : videz la liste ou ajoutez un motif (`192.168.1.*`). |
| `OUT_SP_4 …` ne renvoie rien. | Normal : les écritures/actions NAMUR sont **silencieuses**. Lisez avec `IN_SP_4` / `IN_PV_4`. |
| Le moteur s'arrête tout seul. | Un **chien de garde** est armé (`OUT_WD1@…`) et aucune commande n'est arrivée à temps. Désarmez-le (`OUT_WD1@0`) ou envoyez des trames régulièrement. |
| La liaison série ne s'ouvre pas. | Binaire compilé **sans** la feature `serial`, ou port/permissions incorrects (groupe `dialout` sous Linux). |
| Mes réglages ne sont pas conservés. | Cliquez **Appliquer** / **💾 Sauvegarder**. Le fichier `mock_su_namur.toml` doit être accessible en écriture. |

---

*Documentation technique associée : [conception.md](conception.md) ·
[commandes_namur.md](commandes_namur.md) · [maintenance.md](maintenance.md).*
