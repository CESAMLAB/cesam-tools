//! Protocole **PROFIBUS DP-V0** simulé : codec des trames, calcul FCS, et machine
//! à états de l'esclave. **Source de vérité** du protocole (comme `namur.rs` pour
//! OSNE).
//!
//! # ⚠️ Portée et limites (lire avant tout usage)
//!
//! Ce module implémente un **sous-ensemble éducatif** de DP-V0 (paramétrage,
//! configuration, diagnostic, échange cyclique de données) suffisant pour illustrer
//! la structure du protocole et pour être testé de bout en bout **entre deux
//! instances logicielles**. Il ne vise **aucune conformité binaire stricte** aux
//! tables normatives (IEC 61158 / EN 50170) au-delà des éléments les plus
//! universellement documentés :
//!
//! - les délimiteurs de trame (`SD1`/`SD2`/`SD3`/`SD4`/`SC`) et la somme de
//!   contrôle FCS (somme modulo 256) sont conformes ;
//! - les numéros de SAP des services de paramétrage (`Slave_Diag` = 61,
//!   `Set_Prm` = 62, `Chk_Cfg` = 63) sont conformes ;
//! - en revanche, l'**encodage exact des bits de la fonction FC**, la disposition
//!   précise des octets de diagnostic, et la disposition des blocs d'entrées/
//!   sorties (`map.rs`) sont des **conventions propres à ce simulateur**, pas un
//!   profil GSD réel enregistré au PNO.
//!
//! Surtout : **aucun timing de bus réel n'est respecté** (slot time, `Tset`,
//! `Tsdr`…). Ce module ne sera **jamais reconnu par un vrai maître PROFIBUS DP**
//! (automate + carte maître matérielle) — voir `docs/fr/reference_profibus.md`.

use crate::regulator::{AutoManual, Command};
use mock_lib_control::ControllerKind;

// --- Délimiteurs de trame (conformes) ---
pub const SD1: u8 = 0x10;
pub const SD2: u8 = 0x68;
pub const SD3: u8 = 0xA2;
pub const SD4: u8 = 0xDC;
pub const SC: u8 = 0xE5;
pub const ED: u8 = 0x16;

// --- Numéros de SAP des services de paramétrage (conformes) ---
pub const SAP_SLAVE_DIAG: u8 = 61;
pub const SAP_SET_PRM: u8 = 62;
pub const SAP_CHK_CFG: u8 = 63;

/// Bit d'extension d'adresse (DAE) : présence d'un octet de SAP après `DA`.
/// Absent = échange de données par défaut (`Data_Exchange`).
const ADDR_EXT: u8 = 0x80;

// --- Bits du champ FC (convention de simulation, voir doc de module) ---
// Produits côté "maître" uniquement : utilitaires de test (synthèse de trames
// de requête), non utilisés par l'esclave lui-même qui ne fait qu'y répondre.
#[cfg(test)]
/// Direction de la trame : requête maître (mis) / réponse esclave (absent).
pub const FC_REQ: u8 = 0x80;
#[cfg(test)]
/// *Frame Count Valid* : le bit FCB est significatif.
pub const FC_FCV: u8 = 0x10;

/// Erreur de décodage d'une trame brute.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameError {
    TooShort,
    BadDelimiter,
    LengthMismatch,
    BadChecksum,
    MissingEndDelimiter,
    UnknownSap(u8),
}

fn checksum(bytes: &[u8]) -> u8 {
    bytes.iter().fold(0u8, |acc, b| acc.wrapping_add(*b))
}

/// Encode une trame **SD2** (longueur variable) : `DA SA FC [data...]`.
pub fn encode_sd2(da: u8, sa: u8, fc: u8, data: &[u8]) -> Vec<u8> {
    let l = (data.len() + 3) as u8;
    let mut out = Vec::with_capacity(data.len() + 6);
    out.push(SD2);
    out.push(l);
    out.push(l);
    out.push(SD2);
    out.push(da);
    out.push(sa);
    out.push(fc);
    out.extend_from_slice(data);
    out.push(checksum(&out[4..]));
    out.push(ED);
    out
}

/// Décode une trame **SD2**. Renvoie `(da, sa, fc, data)`.
pub fn decode_sd2(bytes: &[u8]) -> Result<(u8, u8, u8, &[u8]), FrameError> {
    if bytes.len() < 6 {
        return Err(FrameError::TooShort);
    }
    if bytes[0] != SD2 || bytes[3] != SD2 {
        return Err(FrameError::BadDelimiter);
    }
    let l = bytes[1];
    if bytes[2] != l {
        return Err(FrameError::LengthMismatch);
    }
    let end = 4 + l as usize;
    if bytes.len() != end + 2 {
        return Err(FrameError::LengthMismatch);
    }
    if bytes[end + 1] != ED {
        return Err(FrameError::MissingEndDelimiter);
    }
    let fcs = checksum(&bytes[4..end]);
    if bytes[end] != fcs {
        return Err(FrameError::BadChecksum);
    }
    let da = bytes[4];
    let sa = bytes[5];
    let fc = bytes[6];
    let data = &bytes[7..end];
    Ok((da, sa, fc, data))
}

/// Encode une trame **SD1** (requête fixe sans données) : `DA SA FC`. Utilitaire
/// de test (synthèse d'une requête `Slave_Diag` par délimiteur court) : l'esclave
/// ne construit jamais de requête SD1, il ne fait que les décoder.
#[cfg(test)]
pub fn encode_sd1(da: u8, sa: u8, fc: u8) -> [u8; 6] {
    let fcs = checksum(&[da, sa, fc]);
    [SD1, da, sa, fc, fcs, ED]
}

/// Décode une trame **SD1**. Renvoie `(da, sa, fc)`.
pub fn decode_sd1(bytes: &[u8]) -> Result<(u8, u8, u8), FrameError> {
    if bytes.len() != 6 {
        return Err(FrameError::TooShort);
    }
    if bytes[0] != SD1 {
        return Err(FrameError::BadDelimiter);
    }
    if bytes[5] != ED {
        return Err(FrameError::MissingEndDelimiter);
    }
    let fcs = checksum(&bytes[1..4]);
    if bytes[4] != fcs {
        return Err(FrameError::BadChecksum);
    }
    Ok((bytes[1], bytes[2], bytes[3]))
}

/// Encode une trame **SD3** (données fixes, 8 octets) : `SD3 DA SA FC [8] FCS ED`
/// (14 octets au total). Ce simulateur privilégie **SD2** pour tous les échanges
/// `Data_Exchange` (voir doc de module) : SD3 n'est ni émis ni requis par
/// l'esclave — fourni pour la complétude du codec et ses tests.
#[cfg(test)]
pub fn encode_sd3(da: u8, sa: u8, fc: u8, data: &[u8; 8]) -> [u8; 14] {
    let mut out = [0u8; 14];
    out[0] = SD3;
    out[1] = da;
    out[2] = sa;
    out[3] = fc;
    out[4..12].copy_from_slice(data);
    out[12] = checksum(&out[1..12]);
    out[13] = ED;
    out
}

/// Décode une trame **SD3** (14 octets). Renvoie `(da, sa, fc, data)`.
#[cfg(test)]
pub fn decode_sd3(bytes: &[u8]) -> Result<(u8, u8, u8, [u8; 8]), FrameError> {
    if bytes.len() != 14 {
        return Err(FrameError::TooShort);
    }
    if bytes[0] != SD3 {
        return Err(FrameError::BadDelimiter);
    }
    if bytes[13] != ED {
        return Err(FrameError::MissingEndDelimiter);
    }
    let fcs = checksum(&bytes[1..12]);
    if bytes[12] != fcs {
        return Err(FrameError::BadChecksum);
    }
    let mut data = [0u8; 8];
    data.copy_from_slice(&bytes[4..12]);
    Ok((bytes[1], bytes[2], bytes[3], data))
}

/// Encode une trame **SD4** (jeton, 3 octets, sans FCS ni ED). Hors sujet pour un
/// esclave mono-maître simulé (le jeton ne concerne que la circulation entre
/// maîtres) ; fourni pour la complétude du codec et ses tests.
#[cfg(test)]
pub fn encode_sd4(da: u8, sa: u8) -> [u8; 3] {
    [SD4, da, sa]
}

/// Décode une trame **SD4**.
#[cfg(test)]
pub fn decode_sd4(bytes: &[u8]) -> Result<(u8, u8), FrameError> {
    if bytes.len() != 3 {
        return Err(FrameError::TooShort);
    }
    if bytes[0] != SD4 {
        return Err(FrameError::BadDelimiter);
    }
    Ok((bytes[1], bytes[2]))
}

/// Sépare l'éventuel octet de SAP porté par l'extension d'adresse (`DA` bit 7).
fn split_sap(da: u8, data: &[u8]) -> Result<(Option<u8>, &[u8]), FrameError> {
    if da & ADDR_EXT != 0 {
        let sap = *data.first().ok_or(FrameError::TooShort)?;
        Ok((Some(sap), &data[1..]))
    } else {
        Ok((None, data))
    }
}

/// Station d'adresse (0-125), sans le bit d'extension.
fn station_of(da: u8) -> u8 {
    da & !ADDR_EXT
}

/// Requête décodée (niveau service), indépendante de la trame porteuse.
#[derive(Debug, Clone, PartialEq)]
pub enum Request {
    /// Interrogation de diagnostic (`Slave_Diag`, SAP 61).
    SlaveDiag,
    /// Paramétrage (`Set_Prm`, SAP 62) : identifiant attendu + chien de garde.
    SetPrm {
        ident_number: u16,
        /// `None` = chien de garde désactivé par le maître.
        watchdog_ms: Option<u32>,
    },
    /// Vérification de configuration (`Chk_Cfg`, SAP 63) : longueurs I/O annoncées.
    ChkCfg { out_len: u8, in_len: u8 },
    /// Échange cyclique de données (pas de SAP = adresse par défaut).
    DataExchange { output: Vec<u8> },
}

/// Décode une requête maître à partir d'une trame `SD1`/`SD2` complète.
///
/// `SD3` n'est pas utilisé côté requête par ce simulateur (on privilégie `SD2`
/// pour toutes les tailles de bloc, y compris 8 octets) — voir doc de module.
pub fn decode_request(bytes: &[u8]) -> Result<(u8, u8, Request), FrameError> {
    if bytes.first() == Some(&SD1) {
        // Convention de simulation : une trame SD1 (sans champ de données) est
        // toujours une requête `Slave_Diag` — pas d'extension d'adresse possible
        // faute d'octet disponible pour porter un SAP.
        let (da, sa, _fc) = decode_sd1(bytes)?;
        return Ok((station_of(da), sa, Request::SlaveDiag));
    }
    let (da, sa, _fc, data) = decode_sd2(bytes)?;
    let (sap, rest) = split_sap(da, data)?;
    let station = station_of(da);
    match sap {
        Some(SAP_SLAVE_DIAG) => Ok((station, sa, Request::SlaveDiag)),
        Some(SAP_SET_PRM) => {
            if rest.len() < 4 {
                return Err(FrameError::TooShort);
            }
            let ident_number = u16::from_be_bytes([rest[0], rest[1]]);
            let wd1 = rest[2];
            let wd2 = rest[3];
            let watchdog_ms = if wd1 == 0 || wd2 == 0 {
                None
            } else {
                Some(u32::from(wd1) * u32::from(wd2) * 10)
            };
            Ok((
                station,
                sa,
                Request::SetPrm {
                    ident_number,
                    watchdog_ms,
                },
            ))
        }
        Some(SAP_CHK_CFG) => {
            if rest.len() < 2 {
                return Err(FrameError::TooShort);
            }
            Ok((
                station,
                sa,
                Request::ChkCfg {
                    out_len: rest[0],
                    in_len: rest[1],
                },
            ))
        }
        Some(s) => Err(FrameError::UnknownSap(s)),
        None => Ok((
            station,
            sa,
            Request::DataExchange {
                output: rest.to_vec(),
            },
        )),
    }
}

/// Encode une requête maître (utilitaire de test / futur outil de simulation de
/// maître) vers la station `station` (0-125), avec l'adresse maître `master`.
#[cfg(test)]
pub fn encode_request(station: u8, master: u8, req: &Request) -> Vec<u8> {
    let fc = FC_REQ | FC_FCV;
    match req {
        Request::SlaveDiag => {
            let da = station | ADDR_EXT;
            encode_sd2(da, master, fc, &[SAP_SLAVE_DIAG])
        }
        Request::SetPrm {
            ident_number,
            watchdog_ms,
        } => {
            let da = station | ADDR_EXT;
            let mut data = vec![SAP_SET_PRM];
            data.extend_from_slice(&ident_number.to_be_bytes());
            let (wd1, wd2) = watchdog_ms
                .map(|ms| {
                    let total = (ms / 10).max(1);
                    // Factorisation simple : wd2 en dizaines, wd1 = reste (borné 1..=255).
                    let wd2 = total.clamp(1, 255);
                    (1u8, wd2 as u8)
                })
                .unwrap_or((0, 0));
            data.push(wd1);
            data.push(wd2);
            encode_sd2(da, master, fc, &data)
        }
        Request::ChkCfg { out_len, in_len } => {
            let da = station | ADDR_EXT;
            encode_sd2(da, master, fc, &[SAP_CHK_CFG, *out_len, *in_len])
        }
        Request::DataExchange { output } => encode_sd2(station, master, fc, output),
    }
}

/// Réponse esclave décodée.
#[derive(Debug, Clone, PartialEq)]
pub enum Response {
    /// Accusé de réception court (`SC`, un seul octet) : accepté, rien à renvoyer.
    ShortAck,
    /// Réponse au diagnostic (`Slave_Diag`).
    Diag(Vec<u8>),
    /// Réponse à l'échange cyclique : bloc d'entrées.
    DataExchange(Vec<u8>),
}

/// Encode une réponse esclave vers le maître `master`, depuis la station `station`.
pub fn encode_response(station: u8, master: u8, resp: &Response) -> Vec<u8> {
    match resp {
        Response::ShortAck => vec![SC],
        Response::Diag(bytes) => encode_sd2(master, station, 0, bytes),
        Response::DataExchange(bytes) => encode_sd2(master, station, 0, bytes),
    }
}

/// Décode une réponse esclave (utilitaire de test). La réponse ne porte aucun
/// marqueur de type sur le fil (comme le vrai DP-V0) : ce décodeur de test
/// distingue `Diag` de `Data_Exchange` par la **longueur** du bloc de données,
/// en s'appuyant sur le profil figé de ce simulateur (heuristique de test
/// uniquement — un vrai maître sait déjà quelle requête il a envoyée).
#[cfg(test)]
pub fn decode_response(bytes: &[u8]) -> Result<Response, FrameError> {
    if bytes == [SC] {
        return Ok(Response::ShortAck);
    }
    let (_da, _sa, _fc, data) = decode_sd2(bytes)?;
    if data.len() == 6 {
        Ok(Response::Diag(data.to_vec()))
    } else {
        Ok(Response::DataExchange(data.to_vec()))
    }
}

/// Bits de diagnostic `Stat_1` (octet 0 de la réponse `Slave_Diag`) — sous-ensemble
/// simulé, positions inspirées de la norme mais non garanties bit-exactes.
pub const STAT1_PRM_REQ: u8 = 0x01;
pub const STAT1_CFG_FAULT: u8 = 0x02;

/// État de la machine à états de l'esclave DP-V0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlaveState {
    /// Juste après la mise sous tension, avant la première interrogation de
    /// diagnostic du maître.
    PowerOn,
    /// En attente d'un `Set_Prm` valide (identifiant conforme).
    WaitPrm,
    /// Paramétré, en attente d'un `Chk_Cfg` valide (longueurs I/O conformes).
    WaitCfg,
    /// Paramétré et configuré : échange cyclique de données actif.
    DataExchange,
}

/// Profil figé de l'esclave simulé (identifiant + tailles des blocs I/O).
#[derive(Debug, Clone, Copy)]
pub struct SlaveProfile {
    /// Identifiant PROFIBUS **fictif**, non enregistré au PNO — voir doc de module.
    pub ident_number: u16,
    pub out_len: u8,
    pub in_len: u8,
}

/// Machine à états DP-V0 de l'esclave, plus décodage/encodage métier.
pub struct SlaveFsm {
    profile: SlaveProfile,
    state: SlaveState,
}

/// Résultat du traitement d'une requête par la machine à états.
pub struct Handled {
    pub response: Response,
    /// Commandes métier à appliquer (non vide seulement pour `Data_Exchange`).
    pub commands: Vec<Command>,
    /// Présent si `Set_Prm` a (re)configuré le chien de garde du protocole.
    pub watchdog_ms: Option<Option<u32>>,
}

impl SlaveFsm {
    #[must_use]
    pub fn new(profile: SlaveProfile) -> Self {
        Self {
            profile,
            state: SlaveState::PowerOn,
        }
    }

    #[must_use]
    pub fn state(&self) -> SlaveState {
        self.state
    }

    fn stat1(&self) -> u8 {
        match self.state {
            SlaveState::PowerOn | SlaveState::WaitPrm => STAT1_PRM_REQ,
            SlaveState::WaitCfg => STAT1_CFG_FAULT,
            SlaveState::DataExchange => 0,
        }
    }

    fn diag_bytes(&self) -> Vec<u8> {
        // Stat_1, Stat_2, Stat_3, Master_Add, Ident_Number(2) — disposition simulée.
        vec![
            self.stat1(),
            0,
            0,
            0xFF, // aucun maître connu tant qu'aucun Set_Prm n'a été accepté
            (self.profile.ident_number >> 8) as u8,
            (self.profile.ident_number & 0xFF) as u8,
        ]
    }

    /// Traite une requête décodée et met à jour l'état interne.
    pub fn handle(&mut self, req: Request, snap: &crate::regulator::RegulatorSnapshot) -> Handled {
        match req {
            Request::SlaveDiag => {
                if self.state == SlaveState::PowerOn {
                    self.state = SlaveState::WaitPrm;
                }
                Handled {
                    response: Response::Diag(self.diag_bytes()),
                    commands: Vec::new(),
                    watchdog_ms: None,
                }
            }
            Request::SetPrm {
                ident_number,
                watchdog_ms,
            } => {
                if ident_number == self.profile.ident_number {
                    self.state = SlaveState::WaitCfg;
                    Handled {
                        response: Response::ShortAck,
                        commands: Vec::new(),
                        watchdog_ms: Some(watchdog_ms),
                    }
                } else {
                    // Identifiant refusé : reste en attente de paramétrage.
                    self.state = SlaveState::WaitPrm;
                    Handled {
                        response: Response::ShortAck,
                        commands: Vec::new(),
                        watchdog_ms: None,
                    }
                }
            }
            Request::ChkCfg { out_len, in_len } => {
                if self.state == SlaveState::WaitCfg
                    && out_len == self.profile.out_len
                    && in_len == self.profile.in_len
                {
                    self.state = SlaveState::DataExchange;
                }
                Handled {
                    response: Response::ShortAck,
                    commands: Vec::new(),
                    watchdog_ms: None,
                }
            }
            Request::DataExchange { output } => {
                if self.state != SlaveState::DataExchange {
                    // Le maître échange des données avant la fin du séquencement :
                    // on répond par le diagnostic courant plutôt que de planter.
                    return Handled {
                        response: Response::Diag(self.diag_bytes()),
                        commands: Vec::new(),
                        watchdog_ms: None,
                    };
                }
                let commands = crate::map::decode_output(&output);
                let input = crate::map::encode_input(snap);
                Handled {
                    response: Response::DataExchange(input),
                    commands,
                    watchdog_ms: None,
                }
            }
        }
    }

    /// Réaction au dépassement du chien de garde (silence prolongé du maître) :
    /// force la sortie en état sûr sans exiger un nouveau `Set_Prm`/`Chk_Cfg`
    /// (simplification documentée — voir `docs/fr/reference_profibus.md`).
    #[must_use]
    pub fn watchdog_expired(&self) -> Command {
        Command::SetOnOff(false)
    }
}

/// Décode le mode combiné marche/arrêt + auto/manuel + modes sens1/sens2 depuis
/// l'octet 0 du bloc de sortie (voir `map.rs`).
#[must_use]
pub fn decode_mode_byte(byte: u8) -> (bool, AutoManual, ControllerKind, ControllerKind) {
    let on = byte & 0x01 != 0;
    let auto = AutoManual::from_bool(byte & 0x02 != 0);
    let mode1 = ControllerKind::from_code(u16::from((byte >> 2) & 0x03));
    let mode2 = ControllerKind::from_code(u16::from((byte >> 4) & 0x03));
    (on, auto, mode1, mode2)
}

/// Encode l'octet de mode combiné (inverse de [`decode_mode_byte`]) — utilitaire
/// de test/outillage : l'esclave ne fait que décoder ce que le maître envoie.
#[cfg(test)]
#[must_use]
pub fn encode_mode_byte(on: bool, auto: AutoManual, mode1: ControllerKind, mode2: ControllerKind) -> u8 {
    let mut b = 0u8;
    if on {
        b |= 0x01;
    }
    if auto.is_auto() {
        b |= 0x02;
    }
    b |= (mode1.to_code() as u8 & 0x03) << 2;
    b |= (mode2.to_code() as u8 & 0x03) << 4;
    b
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::regulator::RegulatorSnapshot;

    #[test]
    fn sd1_round_trips() {
        let bytes = encode_sd1(0x2A, 0x03, 0x7C);
        assert_eq!(decode_sd1(&bytes), Ok((0x2A, 0x03, 0x7C)));
    }

    #[test]
    fn sd1_rejects_bad_checksum() {
        let mut bytes = encode_sd1(0x2A, 0x03, 0x7C);
        bytes[4] ^= 0xFF;
        assert_eq!(decode_sd1(&bytes), Err(FrameError::BadChecksum));
    }

    #[test]
    fn sd2_round_trips() {
        let data = [1, 2, 3, 4, 5];
        let bytes = encode_sd2(0x80 | 5, 0x03, 0xC0, &data);
        let (da, sa, fc, out) = decode_sd2(&bytes).unwrap();
        assert_eq!((da, sa, fc), (0x85, 0x03, 0xC0));
        assert_eq!(out, &data[..]);
    }

    #[test]
    fn sd2_rejects_length_mismatch() {
        let mut bytes = encode_sd2(5, 3, 0, &[1, 2, 3]);
        bytes[1] = 99; // LE corrompu, LEr encore correct
        assert_eq!(decode_sd2(&bytes), Err(FrameError::LengthMismatch));
    }

    #[test]
    fn sd3_round_trips() {
        let data = [1, 2, 3, 4, 5, 6, 7, 8];
        let bytes = encode_sd3(5, 3, 0x40, &data);
        let (da, sa, fc, out) = decode_sd3(&bytes).unwrap();
        assert_eq!((da, sa, fc, out), (5, 3, 0x40, data));
    }

    #[test]
    fn sd4_round_trips() {
        let bytes = encode_sd4(5, 3);
        assert_eq!(decode_sd4(&bytes), Ok((5, 3)));
    }

    #[test]
    fn slave_diag_request_round_trips() {
        let bytes = encode_request(5, 3, &Request::SlaveDiag);
        let (station, master, req) = decode_request(&bytes).unwrap();
        assert_eq!(station, 5);
        assert_eq!(master, 3);
        assert_eq!(req, Request::SlaveDiag);
    }

    #[test]
    fn set_prm_request_round_trips() {
        let req = Request::SetPrm {
            ident_number: 0xEE01,
            watchdog_ms: Some(500),
        };
        let bytes = encode_request(5, 3, &req);
        let (station, master, decoded) = decode_request(&bytes).unwrap();
        assert_eq!(station, 5);
        assert_eq!(master, 3);
        match decoded {
            Request::SetPrm {
                ident_number,
                watchdog_ms,
            } => {
                assert_eq!(ident_number, 0xEE01);
                assert!(watchdog_ms.is_some());
            }
            _ => panic!("expected SetPrm"),
        }
    }

    #[test]
    fn chk_cfg_request_round_trips() {
        let req = Request::ChkCfg {
            out_len: 45,
            in_len: 17,
        };
        let bytes = encode_request(5, 3, &req);
        let (_station, _master, decoded) = decode_request(&bytes).unwrap();
        assert_eq!(decoded, req);
    }

    #[test]
    fn data_exchange_request_round_trips() {
        let req = Request::DataExchange {
            output: vec![1, 2, 3],
        };
        let bytes = encode_request(5, 3, &req);
        let (station, master, decoded) = decode_request(&bytes).unwrap();
        assert_eq!(station, 5);
        assert_eq!(master, 3);
        assert_eq!(decoded, req);
    }

    fn profile() -> SlaveProfile {
        SlaveProfile {
            ident_number: 0xEE01,
            out_len: crate::map::OUTPUT_LEN as u8,
            in_len: crate::map::INPUT_LEN as u8,
        }
    }

    #[test]
    fn full_sequence_reaches_data_exchange() {
        let mut fsm = SlaveFsm::new(profile());
        let snap = RegulatorSnapshot {
            on: false,
            mode: AutoManual::Manual,
            mode_sens1: ControllerKind::Off,
            mode_sens2: ControllerKind::Off,
            sp_auto: 0.0,
            sp_manual: 0.0,
            pv: 20.0,
            output: 0.0,
            pid_heat: mock_lib_control::PidConfig::default(),
            pid_cool: mock_lib_control::PidConfig::default(),
            hysteresis: 2.0,
            tor_min_cycle: 5.0,
            pwm_period: 10.0,
            sp_min: 0.0,
            sp_max: 250.0,
            process_gain: 1.6,
            process_tau: 30.0,
            process_dead_time: 2.0,
            ambient: 20.0,
        };

        assert_eq!(fsm.state(), SlaveState::PowerOn);
        let h = fsm.handle(Request::SlaveDiag, &snap);
        assert!(matches!(h.response, Response::Diag(_)));
        assert_eq!(fsm.state(), SlaveState::WaitPrm);

        let h = fsm.handle(
            Request::SetPrm {
                ident_number: 0xEE01,
                watchdog_ms: Some(200),
            },
            &snap,
        );
        assert_eq!(h.response, Response::ShortAck);
        assert_eq!(h.watchdog_ms, Some(Some(200)));
        assert_eq!(fsm.state(), SlaveState::WaitCfg);

        let h = fsm.handle(
            Request::ChkCfg {
                out_len: crate::map::OUTPUT_LEN as u8,
                in_len: crate::map::INPUT_LEN as u8,
            },
            &snap,
        );
        assert_eq!(h.response, Response::ShortAck);
        assert_eq!(fsm.state(), SlaveState::DataExchange);

        let output = crate::map::encode_output_for_test(true, AutoManual::Auto, 120.0);
        let h = fsm.handle(Request::DataExchange { output }, &snap);
        assert!(matches!(h.response, Response::DataExchange(_)));
        assert!(h.commands.contains(&Command::SetOnOff(true)));
    }

    #[test]
    fn wrong_ident_number_stays_in_wait_prm() {
        let mut fsm = SlaveFsm::new(profile());
        fsm.handle(
            Request::SlaveDiag,
            &RegulatorSnapshot {
                on: false,
                mode: AutoManual::Manual,
                mode_sens1: ControllerKind::Off,
                mode_sens2: ControllerKind::Off,
                sp_auto: 0.0,
                sp_manual: 0.0,
                pv: 20.0,
                output: 0.0,
                pid_heat: mock_lib_control::PidConfig::default(),
                pid_cool: mock_lib_control::PidConfig::default(),
                hysteresis: 2.0,
                tor_min_cycle: 5.0,
                pwm_period: 10.0,
                sp_min: 0.0,
                sp_max: 250.0,
                process_gain: 1.6,
                process_tau: 30.0,
                process_dead_time: 2.0,
                ambient: 20.0,
            },
        );
        let snap = RegulatorSnapshot {
            on: false,
            mode: AutoManual::Manual,
            mode_sens1: ControllerKind::Off,
            mode_sens2: ControllerKind::Off,
            sp_auto: 0.0,
            sp_manual: 0.0,
            pv: 20.0,
            output: 0.0,
            pid_heat: mock_lib_control::PidConfig::default(),
            pid_cool: mock_lib_control::PidConfig::default(),
            hysteresis: 2.0,
            tor_min_cycle: 5.0,
            pwm_period: 10.0,
            sp_min: 0.0,
            sp_max: 250.0,
            process_gain: 1.6,
            process_tau: 30.0,
            process_dead_time: 2.0,
            ambient: 20.0,
        };
        let h = fsm.handle(
            Request::SetPrm {
                ident_number: 0x1234, // mauvais identifiant
                watchdog_ms: None,
            },
            &snap,
        );
        assert_eq!(h.watchdog_ms, None);
        assert_eq!(fsm.state(), SlaveState::WaitPrm);
    }

    #[test]
    fn mode_byte_round_trips() {
        let byte = encode_mode_byte(true, AutoManual::Auto, ControllerKind::Pid, ControllerKind::OnOff);
        let (on, auto, m1, m2) = decode_mode_byte(byte);
        assert!(on);
        assert!(auto.is_auto());
        assert_eq!(m1, ControllerKind::Pid);
        assert_eq!(m2, ControllerKind::OnOff);
    }
}
