//! Couche **EtherNet/IP** (encapsulation) + **CIP** (messagerie explicite) côté
//! **adaptateur** (serveur). Analyse des paquets et fabrication des réponses.
//!
//! Tout ici est **pur et synchrone** (aucune dépendance réseau) afin d'être testable
//! **sans socket** : l'IO TCP vit dans l'acteur réseau ([`crate::actors::network`]).
//! C'est l'équivalent EtherNet/IP du `opcua_server.rs` des autres instruments.
//!
//! ⚠️ EtherNet/IP / CIP est **little-endian** (à l'inverse de Modbus/S7). Les REAL
//! sont des `f32` IEEE-754 little-endian.
//!
//! Sous-ensemble implémenté :
//! - **Encapsulation** : `RegisterSession` (0x0065), `UnRegisterSession` (0x0066),
//!   `SendRRData` (0x006F, messagerie explicite non connectée).
//! - **CIP** : `Read Tag` (0x4C) et `Write Tag` (0x4D) sur des **tags nommés**
//!   (segment symbolique ANSI).
//!
//! Le parsing est **borné** (aucun accès hors limites) : un paquet malformé venu du
//! réseau renvoie « pas de réponse », jamais de panique.
//!
//! Tags exposés (type CIP entre parenthèses) :
//!
//! | Tag | Type | Accès | Grandeur |
//! |---|---|:--:|---|
//! | `Setpoint` | REAL | R/W | consigne |
//! | `ProcessValue` | REAL | R | mesure |
//! | `Output` | REAL | R | sortie (%) |
//! | `ManualOutput` | REAL | R/W | sortie manuelle (%) |
//! | `Run` | BOOL | R/W | marche |
//! | `Auto` | BOOL | R/W | mode auto |
//! | `SetpointMin`/`SetpointMax` | REAL | R | bornes de consigne |
//! | `Kp`/`Ki`/`Kd` | REAL | R | gains PID |

use crate::regulator::{Command, Snapshot};

// --- Constantes encapsulation --------------------------------------------------

const ENCAP_HEADER_LEN: usize = 24;
const CMD_REGISTER_SESSION: u16 = 0x0065;
const CMD_UNREGISTER_SESSION: u16 = 0x0066;
const CMD_SEND_RR_DATA: u16 = 0x006F;

const ITEM_NULL_ADDRESS: u16 = 0x0000;
const ITEM_UNCONNECTED_DATA: u16 = 0x00B2;

// --- Constantes CIP ------------------------------------------------------------

const SVC_READ_TAG: u8 = 0x4C;
const SVC_WRITE_TAG: u8 = 0x4D;
const SVC_REPLY_MASK: u8 = 0x80;

const CIP_OK: u8 = 0x00;
const CIP_PATH_UNKNOWN: u8 = 0x05; // destination de chemin inconnue (tag inexistant)

const TYPE_BOOL: u16 = 0x00C1;
const TYPE_REAL: u16 = 0x00CA;

const SEGMENT_ANSI_SYMBOLIC: u8 = 0x91;

// --- Aides little-endian bornées ----------------------------------------------

fn le16(buf: &[u8], i: usize) -> Option<u16> {
    Some(u16::from_le_bytes([*buf.get(i)?, *buf.get(i + 1)?]))
}

fn le32(buf: &[u8], i: usize) -> Option<u32> {
    Some(u32::from_le_bytes([*buf.get(i)?, *buf.get(i + 1)?, *buf.get(i + 2)?, *buf.get(i + 3)?]))
}

// --- Encapsulation sortante ----------------------------------------------------

/// Construit un en-tête d'encapsulation (24 octets) + données.
fn encap(command: u16, session: u32, sender_ctx: &[u8], data: &[u8]) -> Vec<u8> {
    let mut v = Vec::with_capacity(ENCAP_HEADER_LEN + data.len());
    v.extend_from_slice(&command.to_le_bytes());
    v.extend_from_slice(&u16::try_from(data.len()).unwrap_or(u16::MAX).to_le_bytes());
    v.extend_from_slice(&session.to_le_bytes());
    v.extend_from_slice(&0u32.to_le_bytes()); // statut = succès
    let mut ctx = [0u8; 8];
    let n = sender_ctx.len().min(8);
    ctx[..n].copy_from_slice(&sender_ctx[..n]);
    v.extend_from_slice(&ctx);
    v.extend_from_slice(&0u32.to_le_bytes()); // options
    v.extend_from_slice(data);
    v
}

// --- Point d'entrée ------------------------------------------------------------

/// Traite un paquet EtherNet/IP complet et renvoie `(réponse éventuelle, commandes)`.
///
/// `assigned_handle` est le handle de session attribué à la connexion (renvoyé dans
/// la réponse `RegisterSession`). Un paquet non reconnu ou malformé renvoie
/// `(None, [])` (jamais de panique).
#[must_use]
pub fn handle_packet(packet: &[u8], assigned_handle: u32, snap: &Snapshot) -> (Option<Vec<u8>>, Vec<Command>) {
    try_handle(packet, assigned_handle, snap).unwrap_or((None, Vec::new()))
}

fn try_handle(packet: &[u8], assigned_handle: u32, snap: &Snapshot) -> Option<(Option<Vec<u8>>, Vec<Command>)> {
    let command = le16(packet, 0)?;
    let length = le16(packet, 2)? as usize;
    let session = le32(packet, 4)?;
    let sender_ctx = packet.get(12..20)?;
    let data = packet.get(ENCAP_HEADER_LEN..ENCAP_HEADER_LEN + length)?;

    match command {
        CMD_REGISTER_SESSION => {
            // Réponse : version (1) + options (0), nouveau handle de session.
            let body = [0x01, 0x00, 0x00, 0x00];
            Some((Some(encap(CMD_REGISTER_SESSION, assigned_handle, sender_ctx, &body)), Vec::new()))
        }
        CMD_UNREGISTER_SESSION => Some((None, Vec::new())),
        CMD_SEND_RR_DATA => {
            let (cip_resp, cmds) = handle_send_rr_data(data, snap)?;
            Some((Some(encap(CMD_SEND_RR_DATA, session, sender_ctx, &cip_resp)), cmds))
        }
        _ => None,
    }
}

/// Extrait la requête CIP d'un `SendRRData` et construit la réponse encapsulée CPF.
fn handle_send_rr_data(data: &[u8], snap: &Snapshot) -> Option<(Vec<u8>, Vec<Command>)> {
    // data : interface handle (u32) + timeout (u16) + CPF.
    let item_count = le16(data, 6)?;
    let mut off = 8;
    let mut cip_request: Option<&[u8]> = None;
    for _ in 0..item_count {
        let type_id = le16(data, off)?;
        let len = le16(data, off + 2)? as usize;
        let body = data.get(off + 4..off + 4 + len)?;
        if type_id == ITEM_UNCONNECTED_DATA {
            cip_request = Some(body);
        }
        off += 4 + len;
    }
    let cip_request = cip_request?;
    let (cip_response, commands) = handle_cip(cip_request, snap)?;

    // CPF de réponse : item adresse nul + item données non connecté.
    let mut cpf = Vec::new();
    cpf.extend_from_slice(&0u32.to_le_bytes()); // interface handle
    cpf.extend_from_slice(&0u16.to_le_bytes()); // timeout
    cpf.extend_from_slice(&2u16.to_le_bytes()); // item count
    cpf.extend_from_slice(&ITEM_NULL_ADDRESS.to_le_bytes());
    cpf.extend_from_slice(&0u16.to_le_bytes()); // longueur 0
    cpf.extend_from_slice(&ITEM_UNCONNECTED_DATA.to_le_bytes());
    cpf.extend_from_slice(&u16::try_from(cip_response.len()).unwrap_or(u16::MAX).to_le_bytes());
    cpf.extend_from_slice(&cip_response);
    Some((cpf, commands))
}

/// Traite une requête CIP `Read Tag` / `Write Tag`.
fn handle_cip(cip: &[u8], snap: &Snapshot) -> Option<(Vec<u8>, Vec<Command>)> {
    let service = *cip.first()?;
    let path_words = *cip.get(1)? as usize;
    let path = cip.get(2..2 + path_words * 2)?;
    let rest = cip.get(2 + path_words * 2..)?;
    let tag = parse_tag_name(path)?;
    let reply_service = service | SVC_REPLY_MASK;

    match service {
        SVC_READ_TAG => {
            if let Some((type_id, bytes)) = read_tag(&tag, snap) {
                let mut r = vec![reply_service, 0x00, CIP_OK, 0x00];
                r.extend_from_slice(&type_id.to_le_bytes());
                r.extend_from_slice(&bytes);
                Some((r, Vec::new()))
            } else {
                Some((vec![reply_service, 0x00, CIP_PATH_UNKNOWN, 0x00], Vec::new()))
            }
        }
        SVC_WRITE_TAG => {
            let type_id = le16(rest, 0)?;
            // rest : type (2) + nombre d'éléments (2) + données.
            let payload = rest.get(4..)?;
            let (status, cmds) = write_tag(&tag, type_id, payload);
            Some((vec![reply_service, 0x00, status, 0x00], cmds))
        }
        _ => None,
    }
}

/// Extrait le nom de tag d'un EPATH (segment symbolique ANSI `0x91`).
fn parse_tag_name(path: &[u8]) -> Option<String> {
    if *path.first()? != SEGMENT_ANSI_SYMBOLIC {
        return None;
    }
    let len = *path.get(1)? as usize;
    let name = path.get(2..2 + len)?;
    Some(String::from_utf8_lossy(name).into_owned())
}

/// Renvoie `(type CIP, octets little-endian)` pour un tag lisible.
fn read_tag(tag: &str, s: &Snapshot) -> Option<(u16, Vec<u8>)> {
    let real = |v: f32| Some((TYPE_REAL, v.to_le_bytes().to_vec()));
    let boolean = |b: bool| Some((TYPE_BOOL, vec![u8::from(b)]));
    match tag {
        "Setpoint" => real(s.setpoint),
        "ProcessValue" => real(s.pv),
        "Output" => real(s.output),
        "ManualOutput" => real(s.manual_output),
        "Run" => boolean(s.run),
        "Auto" => boolean(s.auto),
        "SetpointMin" => real(s.sp_min),
        "SetpointMax" => real(s.sp_max),
        "Kp" => real(s.pid.kp),
        "Ki" => real(s.pid.ki),
        "Kd" => real(s.pid.kd),
        _ => None,
    }
}

/// Décode une écriture de tag → `(statut CIP, commandes)`.
///
/// Tags pilotables : `Setpoint`, `ManualOutput`, `Run`, `Auto`. Un tag connu en
/// lecture seule est **accepté** (statut succès) mais sans effet ; un tag inconnu
/// renvoie `CIP_PATH_UNKNOWN`.
fn write_tag(tag: &str, type_id: u16, data: &[u8]) -> (u8, Vec<Command>) {
    let as_f32 = || {
        if type_id == TYPE_REAL && data.len() >= 4 {
            Some(f32::from_le_bytes([data[0], data[1], data[2], data[3]]))
        } else {
            None
        }
    };
    let as_bool = || data.first().map(|b| *b != 0);

    match tag {
        "Setpoint" => match as_f32() {
            Some(v) => (CIP_OK, vec![Command::SetSetpoint(v)]),
            None => (CIP_OK, Vec::new()),
        },
        "ManualOutput" => match as_f32() {
            Some(v) => (CIP_OK, vec![Command::SetManualOutput(v)]),
            None => (CIP_OK, Vec::new()),
        },
        "Run" => match as_bool() {
            Some(b) => (CIP_OK, vec![Command::SetRun(b)]),
            None => (CIP_OK, Vec::new()),
        },
        "Auto" => match as_bool() {
            Some(b) => (CIP_OK, vec![Command::SetAuto(b)]),
            None => (CIP_OK, Vec::new()),
        },
        // Tags connus en lecture seule : acceptés sans effet.
        "ProcessValue" | "Output" | "SetpointMin" | "SetpointMax" | "Kp" | "Ki" | "Kd" => {
            (CIP_OK, Vec::new())
        }
        _ => (CIP_PATH_UNKNOWN, Vec::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::regulator::{Regulator, RegulatorConfig};

    fn snap() -> Snapshot {
        Regulator::new(RegulatorConfig::default()).snapshot()
    }

    /// Construit un EPATH segment symbolique ANSI pour un nom de tag.
    fn epath(tag: &str) -> Vec<u8> {
        let mut p = vec![SEGMENT_ANSI_SYMBOLIC, tag.len() as u8];
        p.extend_from_slice(tag.as_bytes());
        if p.len() % 2 == 1 {
            p.push(0x00); // bourrage pair
        }
        p
    }

    /// Encapsule une requête CIP dans un SendRRData complet.
    fn send_rr_data(cip: &[u8], session: u32) -> Vec<u8> {
        let mut cpf = Vec::new();
        cpf.extend_from_slice(&0u32.to_le_bytes()); // interface handle
        cpf.extend_from_slice(&0u16.to_le_bytes()); // timeout
        cpf.extend_from_slice(&2u16.to_le_bytes()); // item count
        cpf.extend_from_slice(&ITEM_NULL_ADDRESS.to_le_bytes());
        cpf.extend_from_slice(&0u16.to_le_bytes());
        cpf.extend_from_slice(&ITEM_UNCONNECTED_DATA.to_le_bytes());
        cpf.extend_from_slice(&(cip.len() as u16).to_le_bytes());
        cpf.extend_from_slice(cip);
        encap(CMD_SEND_RR_DATA, session, &[0; 8], &cpf)
    }

    fn read_tag_cip(tag: &str) -> Vec<u8> {
        let path = epath(tag);
        let mut cip = vec![SVC_READ_TAG, (path.len() / 2) as u8];
        cip.extend_from_slice(&path);
        cip.extend_from_slice(&1u16.to_le_bytes()); // nombre d'éléments
        cip
    }

    fn write_tag_cip(tag: &str, type_id: u16, data: &[u8]) -> Vec<u8> {
        let path = epath(tag);
        let mut cip = vec![SVC_WRITE_TAG, (path.len() / 2) as u8];
        cip.extend_from_slice(&path);
        cip.extend_from_slice(&type_id.to_le_bytes());
        cip.extend_from_slice(&1u16.to_le_bytes());
        cip.extend_from_slice(data);
        cip
    }

    #[test]
    fn register_session_returns_handle() {
        let pkt = encap(CMD_REGISTER_SESSION, 0, &[0; 8], &[0x01, 0x00, 0x00, 0x00]);
        let (resp, cmds) = handle_packet(&pkt, 0x1234_5678, &snap());
        assert!(cmds.is_empty());
        let resp = resp.unwrap();
        assert_eq!(le16(&resp, 0).unwrap(), CMD_REGISTER_SESSION);
        assert_eq!(le32(&resp, 4).unwrap(), 0x1234_5678, "handle attribué renvoyé");
    }

    #[test]
    fn read_setpoint_returns_real() {
        let mut s = snap();
        s.setpoint = 80.0;
        let pkt = send_rr_data(&read_tag_cip("Setpoint"), 1);
        let (resp, _) = handle_packet(&pkt, 1, &s);
        let resp = resp.unwrap();
        // CIP réponse dans le 2e item CPF. En-tête encap(24)+ifc(4)+timeout(2)+
        // count(2)+item0(4)+item1 header(4) = 40 → CIP commence à 40.
        let cip = &resp[40..];
        assert_eq!(cip[0], SVC_READ_TAG | SVC_REPLY_MASK);
        assert_eq!(cip[2], CIP_OK);
        assert_eq!(le16(cip, 4).unwrap(), TYPE_REAL);
        let v = f32::from_le_bytes([cip[6], cip[7], cip[8], cip[9]]);
        assert!((v - 80.0).abs() < 1e-6, "consigne lue = {v}");
    }

    #[test]
    fn write_setpoint_yields_command() {
        let pkt = send_rr_data(&write_tag_cip("Setpoint", TYPE_REAL, &55.0f32.to_le_bytes()), 1);
        let (resp, cmds) = handle_packet(&pkt, 1, &snap());
        assert_eq!(cmds, vec![Command::SetSetpoint(55.0)]);
        let cip = &resp.unwrap()[40..];
        assert_eq!(cip[0], SVC_WRITE_TAG | SVC_REPLY_MASK);
        assert_eq!(cip[2], CIP_OK);
    }

    #[test]
    fn write_run_bool_yields_command() {
        let pkt = send_rr_data(&write_tag_cip("Run", TYPE_BOOL, &[0x01]), 1);
        let (_, cmds) = handle_packet(&pkt, 1, &snap());
        assert_eq!(cmds, vec![Command::SetRun(true)]);
    }

    #[test]
    fn read_unknown_tag_returns_path_unknown() {
        let pkt = send_rr_data(&read_tag_cip("DoesNotExist"), 1);
        let (resp, _) = handle_packet(&pkt, 1, &snap());
        let cip = &resp.unwrap()[40..];
        assert_eq!(cip[2], CIP_PATH_UNKNOWN);
    }

    #[test]
    fn write_readonly_tag_is_accepted_without_command() {
        let pkt = send_rr_data(&write_tag_cip("ProcessValue", TYPE_REAL, &1.0f32.to_le_bytes()), 1);
        let (resp, cmds) = handle_packet(&pkt, 1, &snap());
        assert!(cmds.is_empty());
        assert_eq!(resp.unwrap()[40 + 2], CIP_OK);
    }

    #[test]
    fn unregister_session_has_no_response() {
        let pkt = encap(CMD_UNREGISTER_SESSION, 1, &[0; 8], &[]);
        let (resp, cmds) = handle_packet(&pkt, 1, &snap());
        assert!(resp.is_none() && cmds.is_empty());
    }

    #[test]
    fn malformed_packets_do_not_panic() {
        for bad in [&[][..], &[0x6f], &[0x6f, 0x00, 0xff, 0xff], &[0x65, 0x00, 0x04, 0x00]] {
            let (_, cmds) = handle_packet(bad, 1, &snap());
            assert!(cmds.is_empty());
        }
    }
}
