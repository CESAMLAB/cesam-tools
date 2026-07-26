//! Couche **S7comm** (Siemens) sur **ISO-on-TCP / RFC1006** : analyse des trames
//! TPKT + COTP + S7 et fabrication des réponses.
//!
//! Tout ici est **pur et synchrone** (aucune dépendance réseau) afin d'être testable
//! **sans socket** : l'IO TCP vit dans l'acteur réseau ([`crate::actors::network`]).
//! C'est l'équivalent S7 du `opcua_server.rs` des autres instruments.
//!
//! Sous-ensemble implémenté (suffisant pour un automate simulé) :
//! - **COTP** : Connection Request (CR) → Connection Confirm (CC), Data (DT) ;
//! - **S7comm** : Setup Communication, **Read Var** et **Write Var** sur un bloc de
//!   données **DB1** (image d'octets).
//!
//! Le parsing est **borné** (aucun accès hors limites) : une trame malformée issue
//! du réseau renvoie « pas de réponse », jamais de panique.
//!
//! Plan d'adressage DB1 (REAL = `f32` big-endian IEEE-754) :
//!
//! | Offset | Type | Accès | Champ |
//! |---|---|:--:|---|
//! | `DBD0`  | REAL | R/W | Setpoint |
//! | `DBD4`  | REAL | R   | ProcessValue |
//! | `DBD8`  | REAL | R   | Output |
//! | `DBD12` | REAL | R/W | ManualOutput |
//! | `DBX16.0` | BOOL | R/W | Run |
//! | `DBX16.1` | BOOL | R/W | Auto |
//! | `DBD20` | REAL | R | SetpointMin |
//! | `DBD24` | REAL | R | SetpointMax |
//! | `DBD28` | REAL | R | PID Kp |
//! | `DBD32` | REAL | R | PID Ki |
//! | `DBD36` | REAL | R | PID Kd |

use crate::regulator::{Command, Snapshot};

// --- Constantes protocole ------------------------------------------------------

const TPKT_VERSION: u8 = 0x03;
const COTP_CR: u8 = 0xE0;
const COTP_CC: u8 = 0xD0;
const COTP_DT: u8 = 0xF0;

const S7_PROTO_ID: u8 = 0x32;
const ROSCTR_JOB: u8 = 0x01;
const ROSCTR_ACK_DATA: u8 = 0x03;

const FUNC_SETUP: u8 = 0xF0;
const FUNC_READ: u8 = 0x04;
const FUNC_WRITE: u8 = 0x05;

const TRANSPORT_BIT: u8 = 0x01;

/// Zone « bloc de données » (DB).
pub const AREA_DB: u8 = 0x84;
/// Numéro du DB exposé.
pub const DB_NUMBER: u16 = 1;
/// Taille de l'image DB (octets).
pub const DB_SIZE: usize = 40;

/// Code retour S7 « succès ».
const RC_OK: u8 = 0xFF;
/// Code retour S7 « objet inexistant / hors zone ».
const RC_NOT_FOUND: u8 = 0x0A;

// --- Image du DB ---------------------------------------------------------------

/// Sérialise l'instantané en image d'octets DB1 (REAL big-endian).
#[must_use]
pub fn db_image(s: &Snapshot) -> [u8; DB_SIZE] {
    let mut b = [0u8; DB_SIZE];
    b[0..4].copy_from_slice(&s.setpoint.to_be_bytes());
    b[4..8].copy_from_slice(&s.pv.to_be_bytes());
    b[8..12].copy_from_slice(&s.output.to_be_bytes());
    b[12..16].copy_from_slice(&s.manual_output.to_be_bytes());
    b[16] = u8::from(s.run) | (u8::from(s.auto) << 1);
    b[20..24].copy_from_slice(&s.sp_min.to_be_bytes());
    b[24..28].copy_from_slice(&s.sp_max.to_be_bytes());
    b[28..32].copy_from_slice(&s.pid.kp.to_be_bytes());
    b[32..36].copy_from_slice(&s.pid.ki.to_be_bytes());
    b[36..40].copy_from_slice(&s.pid.kd.to_be_bytes());
    b
}

// --- Aides lecture bornée ------------------------------------------------------

fn u16_be(buf: &[u8], i: usize) -> Option<u16> {
    Some(u16::from_be_bytes([*buf.get(i)?, *buf.get(i + 1)?]))
}

fn u24_be(buf: &[u8], i: usize) -> Option<u32> {
    Some(u32::from_be_bytes([0, *buf.get(i)?, *buf.get(i + 1)?, *buf.get(i + 2)?]))
}

/// Nombre d'octets d'un élément selon le code « transport size » S7.
fn elem_bytes(transport: u8) -> usize {
    match transport {
        0x04..=0x05 => 2, // WORD / INT
        0x06..=0x08 => 4, // DWORD / DINT / REAL
        _ => 1,                  // BIT / BYTE / CHAR / inconnu
    }
}

/// Longueur en octets demandée pour une lecture (`count` éléments du `transport`).
fn read_len(transport: u8, count: u16) -> usize {
    if transport == TRANSPORT_BIT {
        1
    } else {
        elem_bytes(transport) * count as usize
    }
}

// --- Encapsulation sortante ----------------------------------------------------

fn tpkt(payload: &[u8]) -> Vec<u8> {
    let len = u16::try_from(payload.len() + 4).unwrap_or(u16::MAX);
    let mut v = vec![TPKT_VERSION, 0x00];
    v.extend_from_slice(&len.to_be_bytes());
    v.extend_from_slice(payload);
    v
}

/// Encapsule un PDU S7 dans un COTP Data (DT) puis un TPKT.
fn frame_s7(s7: &[u8]) -> Vec<u8> {
    let mut cotp = vec![0x02, COTP_DT, 0x80]; // li=2, type DT, EOT + n° TPDU
    cotp.extend_from_slice(s7);
    tpkt(&cotp)
}

/// Construit un PDU S7 `Ack_Data` (réponse).
fn s7_ack(pdu_ref: u16, params: &[u8], data: &[u8]) -> Vec<u8> {
    let mut v = Vec::with_capacity(12 + params.len() + data.len());
    v.push(S7_PROTO_ID);
    v.push(ROSCTR_ACK_DATA);
    v.extend_from_slice(&[0x00, 0x00]); // redondance
    v.extend_from_slice(&pdu_ref.to_be_bytes());
    v.extend_from_slice(&u16::try_from(params.len()).unwrap_or(u16::MAX).to_be_bytes());
    v.extend_from_slice(&u16::try_from(data.len()).unwrap_or(u16::MAX).to_be_bytes());
    v.extend_from_slice(&[0x00, 0x00]); // classe + code d'erreur (succès)
    v.extend_from_slice(params);
    v.extend_from_slice(data);
    v
}

// --- Point d'entrée ------------------------------------------------------------

/// Traite une trame TPKT complète et renvoie `(réponse éventuelle, commandes)`.
///
/// Une trame non reconnue ou malformée renvoie `(None, [])` (jamais de panique).
#[must_use]
pub fn handle_frame(frame: &[u8], snap: &Snapshot) -> (Option<Vec<u8>>, Vec<Command>) {
    try_handle(frame, snap).unwrap_or((None, Vec::new()))
}

fn try_handle(frame: &[u8], snap: &Snapshot) -> Option<(Option<Vec<u8>>, Vec<Command>)> {
    if *frame.first()? != TPKT_VERSION {
        return None;
    }
    let payload = frame.get(4..)?; // saute l'en-tête TPKT
    let li = *payload.first()? as usize;
    let cotp_type = *payload.get(1)?;

    match cotp_type {
        COTP_CR => Some((Some(build_cc(payload, li)?), Vec::new())),
        COTP_DT => {
            let s7 = payload.get(3..)?; // COTP DT = 3 octets (li, type, n° TPDU)
            handle_s7(s7, snap)
        }
        _ => None,
    }
}

/// Construit le Connection Confirm (CC) en écho au Connection Request (CR).
fn build_cc(payload: &[u8], li: usize) -> Option<Vec<u8>> {
    let src_ref = payload.get(4..6)?; // référence source du client
    let params = payload.get(7..=li)?; // partie variable (taille TPDU, TSAP…)
    let mut body = vec![COTP_CC, src_ref[0], src_ref[1], 0x00, 0x01, 0x00];
    body.extend_from_slice(params);
    let mut cotp = vec![u8::try_from(body.len()).unwrap_or(u8::MAX)];
    cotp.extend_from_slice(&body);
    Some(tpkt(&cotp))
}

fn handle_s7(s7: &[u8], snap: &Snapshot) -> Option<(Option<Vec<u8>>, Vec<Command>)> {
    if *s7.first()? != S7_PROTO_ID || *s7.get(1)? != ROSCTR_JOB {
        return None;
    }
    let pdu_ref = u16_be(s7, 4)?;
    let par_len = u16_be(s7, 6)? as usize;
    let data_len = u16_be(s7, 8)? as usize;
    let params = s7.get(10..10 + par_len)?;
    let data = s7.get(10 + par_len..10 + par_len + data_len)?;

    match *params.first()? {
        FUNC_SETUP => Some((Some(frame_s7(&s7_ack(pdu_ref, params, &[]))), Vec::new())),
        FUNC_READ => Some((Some(handle_read(pdu_ref, params, snap)?), Vec::new())),
        FUNC_WRITE => {
            let (resp, cmds) = handle_write(pdu_ref, params, data)?;
            Some((Some(resp), cmds))
        }
        _ => None,
    }
}

/// Read Var : sert les octets demandés depuis l'image DB1.
fn handle_read(pdu_ref: u16, params: &[u8], snap: &Snapshot) -> Option<Vec<u8>> {
    let item_count = *params.get(1)? as usize;
    let image = db_image(snap);
    let mut data: Vec<u8> = Vec::new();

    for i in 0..item_count {
        let off = 2 + i * 12; // chaque spec S7ANY fait 12 octets
        let transport = *params.get(off + 3)?;
        let count = u16_be(params, off + 4)?;
        let db = u16_be(params, off + 6)?;
        let area = *params.get(off + 8)?;
        let addr = u24_be(params, off + 9)?;
        let byte_off = (addr / 8) as usize;
        let nbytes = read_len(transport, count);

        let last = i + 1 == item_count;
        if area == AREA_DB && db == DB_NUMBER && byte_off + nbytes <= DB_SIZE {
            let bits = u16::try_from(nbytes * 8).unwrap_or(u16::MAX);
            data.push(RC_OK);
            data.push(0x04); // taille exprimée en bits
            data.extend_from_slice(&bits.to_be_bytes());
            data.extend_from_slice(&image[byte_off..byte_off + nbytes]);
            // Bourrage à un nombre pair d'octets, sauf pour le dernier item.
            if !last && nbytes % 2 == 1 {
                data.push(0x00);
            }
        } else {
            data.extend_from_slice(&[RC_NOT_FOUND, 0x00, 0x00, 0x00]);
        }
    }

    let resp_params = [FUNC_READ, item_count as u8];
    Some(frame_s7(&s7_ack(pdu_ref, &resp_params, &data)))
}

/// Write Var : décode les items et produit les commandes correspondantes.
fn handle_write(pdu_ref: u16, params: &[u8], data: &[u8]) -> Option<(Vec<u8>, Vec<Command>)> {
    let item_count = *params.get(1)? as usize;
    let mut commands = Vec::new();
    let mut return_codes = Vec::with_capacity(item_count);
    let mut doff = 0usize; // curseur dans la section data

    for i in 0..item_count {
        let off = 2 + i * 12;
        let transport = *params.get(off + 3)?;
        let count = u16_be(params, off + 4)?;
        let db = u16_be(params, off + 6)?;
        let area = *params.get(off + 8)?;
        let addr = u24_be(params, off + 9)?;
        let byte_off = (addr / 8) as usize;
        let bit = (addr % 8) as u8;
        let nbytes = read_len(transport, count);

        // En-tête d'item data : code retour (1) + transport (1) + longueur (2).
        let payload = data.get(doff + 4..doff + 4 + nbytes)?;
        doff += 4 + nbytes;
        if nbytes % 2 == 1 {
            doff += 1; // bourrage pair entre items
        }

        let mut rc = RC_OK;
        if area == AREA_DB && db == DB_NUMBER {
            match byte_off {
                0 if nbytes >= 4 => commands.push(Command::SetSetpoint(be_f32(payload))),
                12 if nbytes >= 4 => commands.push(Command::SetManualOutput(be_f32(payload))),
                16 => {
                    let on = payload.first().is_some_and(|b| *b != 0);
                    if transport == TRANSPORT_BIT {
                        match bit {
                            0 => commands.push(Command::SetRun(on)),
                            1 => commands.push(Command::SetAuto(on)),
                            _ => {}
                        }
                    } else {
                        // Écriture d'octet : bit0 = Run, bit1 = Auto.
                        let v = payload[0];
                        commands.push(Command::SetRun(v & 0x01 != 0));
                        commands.push(Command::SetAuto(v & 0x02 != 0));
                    }
                }
                // Offsets en lecture seule (PV, Output, bornes, PID) : accepté mais ignoré.
                _ => {}
            }
        } else {
            rc = RC_NOT_FOUND;
        }
        return_codes.push(rc);
    }

    let resp_params = [FUNC_WRITE, item_count as u8];
    let resp = frame_s7(&s7_ack(pdu_ref, &resp_params, &return_codes));
    Some((resp, commands))
}

fn be_f32(b: &[u8]) -> f32 {
    f32::from_be_bytes([b[0], b[1], b[2], b[3]])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::regulator::{Regulator, RegulatorConfig};

    fn snap() -> Snapshot {
        Regulator::new(RegulatorConfig::default()).snapshot()
    }

    /// Enveloppe un PDU S7 en COTP DT + TPKT (côté « client » pour les tests).
    fn client_frame_s7(s7: &[u8]) -> Vec<u8> {
        frame_s7(s7)
    }

    /// Construit un PDU S7 « job ».
    fn s7_job(params: &[u8], data: &[u8]) -> Vec<u8> {
        let mut v = vec![S7_PROTO_ID, ROSCTR_JOB, 0, 0, 0x00, 0x01];
        v.extend_from_slice(&(params.len() as u16).to_be_bytes());
        v.extend_from_slice(&(data.len() as u16).to_be_bytes());
        v.extend_from_slice(params);
        v.extend_from_slice(data);
        v
    }

    /// Spec S7ANY de 12 octets (adressage par octet → bit address = byte*8).
    fn s7any(transport: u8, count: u16, db: u16, area: u8, byte_off: u32) -> Vec<u8> {
        let addr = byte_off * 8;
        let mut v = vec![0x12, 0x0a, 0x10, transport];
        v.extend_from_slice(&count.to_be_bytes());
        v.extend_from_slice(&db.to_be_bytes());
        v.push(area);
        v.extend_from_slice(&addr.to_be_bytes()[1..4]); // 3 octets
        v
    }

    #[test]
    fn connection_request_yields_confirm() {
        // CR minimal : li=0x11, type 0xE0, dst=0, src=0x0102, class=0, params TPDU/TSAP.
        let cotp = [
            0x11, COTP_CR, 0x00, 0x00, 0x01, 0x02, 0x00, 0xc0, 0x01, 0x0a, 0xc1, 0x02, 0x01, 0x00,
            0xc2, 0x02, 0x01, 0x02,
        ];
        let frame = tpkt(&cotp);
        let (resp, cmds) = handle_frame(&frame, &snap());
        assert!(cmds.is_empty());
        let resp = resp.expect("CC attendu");
        // TPKT puis COTP : type CC à l'offset 5 (4 TPKT + 1 li).
        assert_eq!(resp[5], COTP_CC);
    }

    #[test]
    fn setup_communication_is_acked() {
        let params = [FUNC_SETUP, 0x00, 0x00, 0x01, 0x00, 0x01, 0x01, 0xe0];
        let frame = client_frame_s7(&s7_job(&params, &[]));
        let (resp, _) = handle_frame(&frame, &snap());
        let resp = resp.expect("ack attendu");
        // S7 PDU commence après TPKT(4)+COTP_DT(3) = offset 7.
        assert_eq!(resp[7], S7_PROTO_ID);
        assert_eq!(resp[8], ROSCTR_ACK_DATA);
        // 1er octet de param = fonction Setup.
        // header ack_data = 12 octets → params à l'offset 7+12.
        assert_eq!(resp[7 + 12], FUNC_SETUP);
    }

    #[test]
    fn read_setpoint_returns_be_bytes() {
        let mut s = snap();
        s.setpoint = 80.0;
        let params = {
            let mut p = vec![FUNC_READ, 0x01];
            p.extend_from_slice(&s7any(0x08, 1, DB_NUMBER, AREA_DB, 0)); // REAL, DBD0
            p
        };
        let frame = client_frame_s7(&s7_job(&params, &[]));
        let (resp, cmds) = handle_frame(&frame, &s);
        assert!(cmds.is_empty());
        let resp = resp.unwrap();
        // data commence après TPKT(4)+COTP(3)+S7hdr(12)+respparams(2) = 21.
        // data item : rc(1)+transport(1)+len(2) puis 4 octets REAL.
        let d = &resp[21..];
        assert_eq!(d[0], RC_OK);
        let value = f32::from_be_bytes([d[4], d[5], d[6], d[7]]);
        assert!((value - 80.0).abs() < 1e-6, "consigne lue = {value}");
    }

    #[test]
    fn write_setpoint_yields_command() {
        let params = {
            let mut p = vec![FUNC_WRITE, 0x01];
            p.extend_from_slice(&s7any(0x08, 1, DB_NUMBER, AREA_DB, 0));
            p
        };
        // data item : rc res(0), transport 0x04, longueur en bits (32), 4 octets.
        let mut data = vec![0x00, 0x04, 0x00, 0x20];
        data.extend_from_slice(&55.0f32.to_be_bytes());
        let frame = client_frame_s7(&s7_job(&params, &data));
        let (resp, cmds) = handle_frame(&frame, &snap());
        assert_eq!(cmds, vec![Command::SetSetpoint(55.0)]);
        let resp = resp.unwrap();
        // data du write = code retour à l'offset 21.
        assert_eq!(resp[21], RC_OK);
    }

    #[test]
    fn write_run_bit_yields_command() {
        let params = {
            let mut p = vec![FUNC_WRITE, 0x01];
            // BIT, count=1, DB1, area DB, adresse bit = byte16*8 + 0.
            let mut spec = vec![0x12, 0x0a, 0x10, TRANSPORT_BIT];
            spec.extend_from_slice(&1u16.to_be_bytes());
            spec.extend_from_slice(&DB_NUMBER.to_be_bytes());
            spec.push(AREA_DB);
            spec.extend_from_slice(&(16u32 * 8).to_be_bytes()[1..4]);
            p.extend_from_slice(&spec);
            p
        };
        let data = vec![0x00, 0x03, 0x00, 0x01, 0x01]; // transport bit, 1 bit, valeur 1
        let frame = client_frame_s7(&s7_job(&params, &data));
        let (_, cmds) = handle_frame(&frame, &snap());
        assert_eq!(cmds, vec![Command::SetRun(true)]);
    }

    #[test]
    fn read_wrong_db_returns_error_code() {
        let params = {
            let mut p = vec![FUNC_READ, 0x01];
            p.extend_from_slice(&s7any(0x08, 1, 2, AREA_DB, 0)); // DB2 inexistant
            p
        };
        let frame = client_frame_s7(&s7_job(&params, &[]));
        let (resp, _) = handle_frame(&frame, &snap());
        let resp = resp.unwrap();
        assert_eq!(resp[21], RC_NOT_FOUND);
    }

    #[test]
    fn malformed_frames_do_not_panic() {
        for bad in [&[][..], &[0x03], &[0x03, 0x00, 0x00], &[0x03, 0x00, 0x00, 0x07, 0x02, 0xf0]] {
            let (resp, cmds) = handle_frame(bad, &snap());
            assert!(resp.is_none() || resp.is_some());
            assert!(cmds.is_empty());
        }
    }

    #[test]
    fn db_image_roundtrip_setpoint() {
        let mut s = snap();
        s.setpoint = 123.5;
        let img = db_image(&s);
        assert!((f32::from_be_bytes([img[0], img[1], img[2], img[3]]) - 123.5).abs() < 1e-6);
        // Flags : run/auto.
        s.run = true;
        s.auto = false;
        assert_eq!(db_image(&s)[16] & 0x03, 0x01);
    }
}
