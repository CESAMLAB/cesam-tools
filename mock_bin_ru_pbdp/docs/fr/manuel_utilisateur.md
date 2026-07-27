# Manuel utilisateur — Régulateur PROFIBUS DP simulé (ORPD)

*🌍 **FR** · [EN](../en/manuel_utilisateur.md) · [DE](../de/manuel_utilisateur.md) · [ES](../es/manuel_utilisateur.md) · [IT](../it/manuel_utilisateur.md) · [PT](../pt/manuel_utilisateur.md) · [NL](../nl/manuel_utilisateur.md) · [PL](../pl/manuel_utilisateur.md)*

> Crate : `mock_bin_ru_pbdp` · Exécutable : **ru_pbdp** · Marque : **ORPD**

---

## ⚠️ Avant de commencer : ce que ce simulateur n'est PAS

`ru_pbdp` **n'est pas** un esclave PROFIBUS DP conforme au matériel. PROFIBUS DP
est un bus à jeton dont le respect des fenêtres temporelles (*slot time*, *Tsdr*,
chien de garde) exige un circuit dédié (ASIC SPC3/VPC3, carte maître Hilscher/
Softing/Siemens CP). Un programme Tokio ordinaire, même relié à un vrai port
RS-485, **ne peut pas tenir ces contraintes** : un automate réel (Siemens S7 +
carte maître, par exemple) ne reconnaîtra **jamais** ce simulateur comme esclave
valide sur un vrai bus.

Ce que `ru_pbdp` fait réellement : il implémente, **en logiciel et sans
contrainte de temps réel**, la structure des trames et la machine à états d'un
esclave DP-V0 (paramétrage, configuration, diagnostic, échange cyclique). C'est
un outil pour **comprendre le protocole** et **tester un développement logiciel**
(codec, machine à états, outillage de test) — pas pour piloter du matériel de
terrain. Voir [reference_profibus.md](reference_profibus.md) §6 pour le détail
des limites.

---

## 1. À quoi sert ce simulateur

`ru_pbdp` simule un **régulateur de procédé** (boucle PID sur un procédé
thermique, modèle identique à ORME/Modbus) et l'expose via un jeu de trames
PROFIBUS DP-V0 simulé, sur une liaison série (RS-485/RS-232). L'interface
graphique permet de **piloter** la simulation et de **visualiser** la dynamique ;
le journal de trames affiche le trafic échangé en hexadécimal.

---

## 2. Prise en main

```bash
cargo run -p mock_bin_ru_pbdp          # IHM + liaison série PROFIBUS DP
```

Au lancement, le simulateur tente d'ouvrir le port série configuré (par défaut
`/dev/ttyUSB0` ou `COM3`, 500 kbit/s, adresse de station 3). Si le port
n'existe pas (cas courant en l'absence de matériel série), l'IHM affiche l'erreur
d'ouverture dans l'en-tête — la simulation du régulateur continue de tourner,
seule la liaison est indisponible. Réglez le **port série** dans *Paramètres*
pour pointer vers un pseudo-terminal ou un adaptateur USB-série disponible.

---

## 3. L'interface

### En-tête

- **Titre** et boutons **⚙ Paramètres** / **💾 Sauvegarder les réglages**.
- À droite : **état de l'appareil** (EN MARCHE / À L'ARRÊT), **état de la
  liaison** (`PROFIBUS ● <port> [<état>]` en vert si ouverte — l'état affiché
  est celui de la machine DP-V0 : `Power_On`/`Wait_Prm`/`Wait_Cfg`/
  `Data_Exchange`), et le **logo CESAM-Lab**.
- Un **bandeau orange permanent** rappelle la non-interopérabilité avec du
  matériel réel (voir l'avertissement ci-dessus).

### Mini-terminal (bas de fenêtre)

Journal en lecture seule des trames **reçues** (← RX) et **émises** (→ TX),
horodatées et affichées en hexadécimal. Bouton **Effacer** pour vider le journal.

### Panneau de commandes (gauche)

Identique à ORME : **Marche/Arrêt**, **Auto/Manuel**, modes de régulation
**sens 1 (chaud)** / **sens 2 (froid)** (Off/PID/TOR/PWM), **consignes**
(automatique et manuelle), **réglages PID** des deux sens, **hystérésis**,
**cycle minimal TOR**, **période PWM**.

### Panneau droit : blocs I/O PROFIBUS

Table en direct des blocs *Output* (maître→esclave) et *Input* (esclave→maître),
avec la disposition d'octets utilisée par ce simulateur — voir
[reference_profibus.md](reference_profibus.md) §3.

### Zone centrale

Cartes **Mesure**, **Consigne active**, **Sortie**, et courbe de tendance.

---

## 4. Paramètres (modal ⚙)

- **Langue** de l'interface (8 langues), persistée.
- **Vérifier les mises à jour au démarrage** + bouton **Vérifier maintenant**.
- **Port série**, **débit** (bauds — utiliser une valeur normalisée PROFIBUS DP :
  9600, 19200, 45450, 93750, 187500, 500000, 1500000, 3000000, 6000000 ou
  12000000), **adresse de station** (0-125).
- **Chien de garde protocolaire (autorisé)** : case à cocher — si décochée, le
  chien de garde demandé par le maître via `Set_Prm` est **ignoré** (jamais armé).
- **Fonction de transfert du procédé** : gain `K`, constante de temps `τ`, retard
  pur, valeur ambiante.
- **Bornes de consigne** : min / max (réordonnées automatiquement si inversées).
- **Appliquer** / **Réinitialiser par défaut** / **Fermer**.

Un changement de port/débit/adresse **referme et rouvre** la liaison série.
Les réglages sont sauvegardés dans `mock_ru_pbdp.toml` (répertoire courant ;
surchargeable via la variable d'environnement `MOCK_CONFIG`).

**Le format de trame (8E1) est fixé par la norme PROFIBUS DP** et n'est pas
réglable ici, contrairement à Modbus RTU ou NAMUR série.

---

## 5. Le mini-terminal comme outil pédagogique

Sans matériel PROFIBUS réel, le meilleur moyen d'observer le protocole est de
faire dialoguer **deux instances** de cet outil — ou d'écrire un petit script
qui rejoue une séquence `Slave_Diag` → `Set_Prm` → `Chk_Cfg` → `Data_Exchange`
sur un pseudo-terminal (`socat -d -d pty,raw,echo=0 pty,raw,echo=0`) — et de lire
le mini-terminal pour voir les trames échangées en hexadécimal, avec leur
décodage dans [reference_profibus.md](reference_profibus.md).

---

## 6. FAQ

**Puis-je relier ce simulateur à un vrai automate PROFIBUS DP ?** Non — voir
l'avertissement en tête de ce document et §6 de
[reference_profibus.md](reference_profibus.md).

**Le port série ne s'ouvre pas.** Le fichier/périphérique indiqué n'existe pas
ou les droits sont insuffisants (groupe `dialout` sous Linux). L'erreur exacte
est affichée dans l'en-tête de l'IHM.

**La liaison reste en `Wait_Prm`.** Le maître n'a pas encore envoyé de `Set_Prm`
avec l'identifiant attendu (`0xEE01`, identifiant **fictif**, non enregistré
PNO). Voir [reference_profibus.md](reference_profibus.md) §2.

**La liaison reste en `Wait_Cfg`.** Le `Chk_Cfg` reçu n'annonce pas les
longueurs I/O attendues (45 octets en sortie, 17 en entrée pour ce simulateur).

**L'appareil s'arrête tout seul.** Le chien de garde protocolaire (armé par le
maître via `Set_Prm`) s'est déclenché faute d'échange cyclique reçu à temps —
c'est l'état sûr attendu, pas un bug.

**Lancer sans interface graphique ?** Compilez en *headless* :
`cargo run -p mock_bin_ru_pbdp --no-default-features` — la liaison série et la
simulation tournent sans IHM.
