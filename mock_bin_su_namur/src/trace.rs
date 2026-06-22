//! Journal circulaire des trames NAMUR échangées, pour le mini-terminal de l'IHM.
//!
//! Le serveur y consigne chaque ligne **reçue** (`Rx`) et chaque réponse **émise**
//! (`Tx`) ; l'IHM le lit pour afficher le trafic. Borné à [`TRACE_CAP`] entrées
//! (les plus anciennes sont évincées).

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Nombre maximal de trames conservées dans le journal.
pub const TRACE_CAP: usize = 500;

/// Sens d'une trame du point de vue de l'appareil simulé.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Reçue du maître (commande / requête).
    Rx,
    /// Émise vers le maître (réponse).
    Tx,
}

/// Une trame horodatée.
#[derive(Debug, Clone)]
pub struct TraceEntry {
    pub at: Instant,
    pub dir: Direction,
    pub text: String,
}

/// Journal de trames partagé entre le serveur (écriture) et l'IHM (lecture).
pub type SharedTrace = Arc<Mutex<VecDeque<TraceEntry>>>;

/// Consigne une trame dans le journal (évince la plus ancienne au-delà de la borne).
pub fn record(trace: &SharedTrace, dir: Direction, text: impl Into<String>) {
    if let Ok(mut t) = trace.lock() {
        t.push_back(TraceEntry {
            at: Instant::now(),
            dir,
            text: text.into(),
        });
        while t.len() > TRACE_CAP {
            t.pop_front();
        }
    }
}
