# Manuel utilisateur — ORME (régulateur simulé Modbus)

*🌍 **FR** · [EN](../en/manuel_utilisateur.md) · [DE](../de/manuel_utilisateur.md) · [ES](../es/manuel_utilisateur.md) · [IT](../it/manuel_utilisateur.md) · [PT](../pt/manuel_utilisateur.md) · [NL](../nl/manuel_utilisateur.md) · [PL](../pl/manuel_utilisateur.md)*

> **ORME** — *Open Regulator Modbus Emulator* · binaire `mock_bin_ru_modbustcp` ·
> Licence MIT · Éditeur : **CESAM-Lab** · Identifiant appareil Modbus : **CESAM-Lab**
>
> *« Ouvrez le bus. »* Un régulateur de terrain qui n'existe que sur votre bus
> Modbus (TCP/RTU) — pour tester SCADA, automates et IHM sans matériel réel.

Ce manuel s'adresse à l'**utilisateur** du régulateur simulé : comment le lancer,
le piloter depuis l'interface, le paramétrer, et le raccorder en Modbus TCP.
Aucune connaissance de programmation n'est nécessaire.

---

## 1. À quoi sert ce logiciel ?

Il simule un **régulateur industriel** (type four ou bain thermostaté) :

- un **procédé physique** réaliste (la « mesure » monte/descend selon la commande) ;
- une **régulation** automatique ou manuelle, en **chaud** et/ou en **froid** ;
- un **serveur Modbus TCP** pour le piloter/superviser depuis un autre logiciel
  (automate, SCADA, passerelle…) ;
- une **interface graphique** de conduite et de visualisation.

C'est un outil de **test** : il permet de mettre au point et démontrer un
superviseur ou un automate **sans matériel réel**.

---

## 2. Démarrer le logiciel

Lancer l'exécutable correspondant à votre système :

| Système | Fichier |
|---------|---------|
| Windows | `orme-windows-x86_64.exe` (double-clic) |
| Linux PC | `./orme-linux-x86_64` |
| Raspberry Pi (écran) | `./orme-rpi-arm64` |

La fenêtre s'ouvre et le **serveur Modbus démarre automatiquement** (port `5502`
par défaut). L'en-tête indique l'état :

- **● EN MARCHE / ● À L'ARRÊT** : état de l'appareil ;
- **Modbus ● 0.0.0.0:5502** (vert) : serveur à l'écoute ; **✖ …** (rouge) en cas
  de problème réseau.

> Sans écran (serveur seul), voir le **§ 9 (Utilisation sans écran)**.

---

## 3. L'interface en un coup d'œil

La fenêtre comporte quatre zones :

```
┌───────────────────────────── En-tête : titre, ⚙ Paramètres, 💾 Sauvegarder, états ─────────────────────────────┐
├──────────────────┬─────────────────────────────────────────────────┬───────────────────────────────────────────┤
│  COMMANDES        │   SUPERVISION                                   │   TABLE D'ADRESSES MODBUS                 │
│  (gauche)         │   - valeurs instantanées (Mesure / Consigne /   │   (droite)                                │
│  Marche/Arrêt     │     Sortie)                                     │   liste live : désignation, table,        │
│  Auto/Manuel      │   - COURBE de tendance temps réel               │   adresse, valeur, accès                  │
│  Modes, consignes │                                                 │                                           │
│  réglages PID…    │                                                 │                                           │
└──────────────────┴─────────────────────────────────────────────────┴───────────────────────────────────────────┘
```

---

## 4. Piloter le régulateur (panneau de gauche)

### 4.1 Marche / Arrêt
Bouton **Marche / Arrêt**. À l'arrêt, la sortie est nulle et la mesure revient
doucement vers la valeur ambiante.

### 4.2 Auto / Manuel
- **Manuel** : *vous* imposez la sortie via la **consigne manuelle** (en %).
- **Auto** : le régulateur calcule la sortie pour atteindre la **consigne auto**.

### 4.3 Les consignes
Chaque consigne dispose d'un **champ numérique** (saisie précise au clavier) et
d'un **curseur**. Les deux sont toujours modifiables ; la consigne **active**
(selon le mode) est affichée en gras.

| Consigne | Unité | Rôle |
|----------|-------|------|
| **SP auto** | unité de mesure (ex. °C) | cible à atteindre en mode Auto |
| **SP manuel** | % de sortie, de −100 à +100 | sortie imposée en mode Manuel (**+** chaud / **−** froid) |

### 4.4 Modes de régulation — sens 1 (chaud) et sens 2 (froid)
Chaque sens se règle indépendamment :

- **Désactivé** — le sens n'agit pas ;
- **PID** — régulation continue (sortie 0…100 %), précise et douce ;
- **Tout-ou-rien (TOR)** — relais à hystérésis : sortie 0 % ou 100 %, simple mais
  oscillant autour de la consigne ;
- **Relais à cycle (PWM)** — un PID calcule un rapport cyclique, *haché* sur une
  période fixe : la sortie physique reste tout-ou-rien (0/100 %), mais sa
  **moyenne** suit le PID. C'est le meilleur compromis pour piloter finement un
  organe qui ne sait que s'ouvrir ou se fermer (relais, vanne TOR).

> 👉 **Important — voir **§ 6 (Comprendre la régulation)**** : choisir
> PID/TOR/PWM pour le froid *arme* le froid, mais celui-ci ne **débite que lorsque
> la mesure dépasse la consigne**.

### 4.5 Réglages PID (Kp, Ki, Kd)
Pour chaque sens, trois gains ajustables en direct :

- **Kp** (proportionnel) : plus il est grand, plus la réaction est vive (risque d'oscillation) ;
- **Ki** (intégral) : annule l'écart résiduel dans le temps (trop fort → dépassement) ;
- **Kd** (dérivé) : amortit/anticipe (trop fort → sensible au bruit).

### 4.6 Réglages TOR / PWM
- **Hystérésis TOR** — largeur de la **zone morte** du mode Tout-ou-rien, centrée
  sur la consigne (`[SP − h/2, SP + h/2]`) : évite que la sortie ne claque sans
  arrêt. Plus elle est large, plus l'ondulation est grande mais les commutations
  espacées.
- **Cycle min. TOR (s)** — durée minimale pendant laquelle le relais reste dans un
  état avant de pouvoir recommuter (**anti-court-cycle**). Protège un actionneur
  réel (relais, compresseur) et lisse le comportement. `0` = désactivé.
- **Période PWM (s)** — durée d'un cycle du **relais à cycle**. Courte → moyenne
  plus fidèle mais commutations fréquentes ; longue → moins d'usure mais ondulation
  plus marquée. À choisir bien plus petite que la constante de temps du procédé.

---

## 5. Lire la courbe de tendance

La courbe (au centre) trace en temps réel trois grandeurs. La **légende, en haut
à gauche**, rappelle la couleur **et la dernière valeur** de chaque série :

| Couleur | Série | Signification |
|---------|-------|---------------|
| 🔵 bleu | **Consigne (SP)** | cible (en Auto) |
| 🔴 rouge | **Mesure (PV)** | valeur du procédé |
| 🟢 vert | **Sortie (%)** | commande appliquée (**+** chaud / **−** froid) |

Au-dessus de la courbe, trois cartes affichent les valeurs instantanées
(Mesure, Consigne active, Sortie). On peut zoomer/déplacer la courbe à la souris.

---

## 6. Comprendre la régulation (chaud / froid)

Le régulateur agit dans **un seul sens à la fois**, choisi selon l'écart
`Consigne − Mesure` :

| Situation | Sens qui agit | Sortie | Voyant |
|-----------|---------------|--------|--------|
| Mesure **< ** Consigne (il faut chauffer) | **Sens 1 (chaud)** | **positive** (0…+100 %) | **Chaud actif = 1** |
| Mesure **> ** Consigne (il faut refroidir) | **Sens 2 (froid)** | **négative** (−100…0 %) | **Froid actif = 1** |

Conséquences pratiques :

- Sélectionner **PID/TOR pour le froid** ne suffit pas à allumer « Froid actif » :
  il faut que **la mesure soit au-dessus de la consigne**. Tant que la mesure est
  en dessous, c'est le **chaud** qui travaille.
- Pour voir le froid débiter : en **Auto**, sens 2 en PID/TOR, **abaissez la
  consigne sous la mesure courante** (ou attendez un dépassement). La sortie
  devient négative et **Froid actif** passe à 1.
- En **TOR**, le relais bascule sur la **demi-hystérésis** de part et d'autre de la
  consigne (zone morte symétrique) et respecte le **cycle minimal** entre deux
  commutations. En **PWM**, la sortie hache à 0/100 % mais sa moyenne suit le PID.

---

## 7. Paramètres (bouton ⚙)

Le bouton **⚙ Paramètres** ouvre une fenêtre pour configurer :

### Transport Modbus
Choix du bus de communication — **un seul actif à la fois** :

**TCP (Ethernet)**
- **IP d'écoute** (`0.0.0.0` = toutes les interfaces) et **Port** (défaut 5502) ;
- **IP autorisées** : une par ligne, jokers `*` acceptés (ex. `192.168.1.*`).
  **Liste vide = toutes les IP autorisées.** Les autres sont refusées.

**RTU (RS485)** — nécessite un binaire compilé avec la feature `rtu`
- **Port série** : `/dev/ttyUSB0`, `/dev/ttyAMA0` (Raspberry Pi), `COM3` (Windows)… ;
- **Baud** (défaut 19200), **Parité** (défaut Paire), **Bits de données** (8),
  **Bits de stop** (1) — à accorder avec le maître ;
- **Adresse esclave** (1–247).

> ⚠️ **Un seul maître distant à la fois.** En TCP, la connexion d'un nouveau
> maître **déconnecte automatiquement** le précédent. L'IHM locale n'est **pas**
> un maître : elle reste toujours active. En RTU, privilégier une **liaison
> point-à-point** (l'appareil répond quelle que soit l'adresse demandée).

### Fonction de transfert (procédé)
Comportement physique simulé `G(s) = K·e^(−L·s) / (1 + T·s)` :
- **Gain K** : variation de mesure par % de sortie ;
- **Constante T** (s) : inertie/rapidité ;
- **Retard L** (s) : temps mort avant réaction ;
- **Ambiant** : valeur de repos.

### Bornes de consigne
Limites mini/maxi de la consigne auto.

Boutons : **Appliquer** (prend effet immédiatement **et** enregistre),
**Réinitialiser par défaut**, **Fermer**.

### Enregistrement des réglages
Les réglages sont **sauvegardés** dans un fichier `mock_ru_modbustcp.toml` (à côté
du logiciel) et **rechargés au prochain démarrage**. Le bouton **💾 Sauvegarder
réglages** de l'en-tête enregistre aussi les gains PID, l'hystérésis, le cycle
minimal TOR et la période PWM modifiés depuis le panneau de gauche.

---

## 8. Raccorder un client Modbus

Le logiciel est un **esclave Modbus** (TCP port 5502 par défaut, ou RTU série
selon le transport choisi au § 7). Un client (automate, SCADA, `mbpoll`…) peut
**lire** l'état et **écrire** les consignes/modes. Rappel : **un seul maître
distant à la fois** (en TCP, un nouveau venu déconnecte le précédent).

Repères principaux (adresses **base 0**) :

| Donnée | Table | Adresse | Type | Accès |
|--------|-------|---------|------|-------|
| Marche/Arrêt | Bobine | 0 | bit | L/É |
| Auto/Manuel | Bobine | 1 | bit | L/É |
| Mode sens 1 / sens 2 | Holding | 0 / 1 | 0=Off,1=PID,2=TOR,3=PWM | L/É |
| Consigne auto | Holding | 2–3 | flottant | L/É |
| Consigne manuelle | Holding | 4–5 | flottant | L/É |
| Cycle min. TOR (s) | Holding | 20–21 | flottant | L/É |
| Période PWM (s) | Holding | 22–23 | flottant | L/É |
| Mesure (PV) | Input | 0–1 | flottant | L |
| Sortie (%) | Input | 2–3 | flottant | L |
| Identifiant « CESAM-Lab » | Holding | 42–46 | texte ASCII | L |

> La **table complète** (gains PID, hystérésis, encodage des flottants, codes
> fonction, exemples `mbpoll`) est dans **[table_modbus.md](table_modbus.md)**.
> La même table est aussi visible **en direct** dans le panneau de droite de l'IHM.

---

## 9. Utilisation sans écran (« headless » / Docker)

Pour un déploiement en tâche de fond (Raspberry Pi sans écran, serveur), une
version **sans interface** existe : elle fait tourner la simulation et le serveur
Modbus, pilotables **uniquement par Modbus**.

```bash
# Image Docker (déployable n'importe où) :
docker run --rm -p 5502:5502 -v "$PWD/conf:/data" orme:headless
```

Le dossier monté sur `/data` permet de fournir/conserver `mock_ru_modbustcp.toml`.

---

## 10. Questions fréquentes

| Question / symptôme | Réponse |
|---------------------|---------|
| **« Froid actif » ne passe pas à 1 alors que j'ai mis PID/TOR.** | Normal : le froid ne débite que si **la mesure dépasse la consigne**. Abaissez la consigne sous la mesure (mode Auto). Voir **§ 6 (Comprendre la régulation)**. |
| La mesure ne bouge pas. | Vérifiez que l'appareil est **En marche**, et la consigne/sortie non nulles. |
| En manuel, changer les modes sens 1/2 ne fait rien. | Normal : les modes ne s'appliquent qu'en **Auto**. |
| L'en-tête affiche **Modbus ✖**. | Port déjà utilisé ou < 1024 sans droits : changez le **port** dans ⚙ Paramètres. |
| Mon client Modbus est refusé. | Son IP n'est pas dans la **liste blanche** : videz la liste ou ajoutez un motif (`192.168.1.*`). |
| Les flottants lus sont incohérents. | Problème d'**ordre des mots** côté client (mot de poids fort en premier). Voir table_modbus.md. |
| Une consigne écrite en Modbus est ignorée. | Un flottant occupe **2 registres** : écrivez-les **ensemble**. |
| Mes réglages ne sont pas conservés. | Cliquez **Appliquer** / **💾 Sauvegarder**. Le fichier `mock_ru_modbustcp.toml` doit être accessible en écriture. |

---

*Documentation technique associée : [conception.md](conception.md) ·
[table_modbus.md](table_modbus.md) · [maintenance.md](maintenance.md).*
