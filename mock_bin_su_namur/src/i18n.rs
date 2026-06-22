//! Internationalisation (i18n) de l'IHM — catalogue de chaînes traduites (8 langues).
//!
//! Seules les chaînes **destinées à l'opérateur** sont traduites ; les logs et les
//! acronymes (NAMUR, PID, tr/min, N·cm) restent codés en dur. Le compilateur
//! garantit qu'aucune clé n'est oubliée (tableau de taille fixe).

use serde::{Deserialize, Serialize};

/// Langue de l'interface graphique. L'ordre des variantes **fixe** l'indexation
/// des tableaux de traduction (`Fr = 0, …, Pl = 7`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    #[default]
    Fr,
    En,
    De,
    Es,
    It,
    Pt,
    Nl,
    Pl,
}

impl Lang {
    pub const ALL: [Lang; 8] = [
        Lang::Fr,
        Lang::En,
        Lang::De,
        Lang::Es,
        Lang::It,
        Lang::Pt,
        Lang::Nl,
        Lang::Pl,
    ];

    #[inline]
    fn idx(self) -> usize {
        self as usize
    }

    #[must_use]
    pub fn native_name(self) -> &'static str {
        match self {
            Lang::Fr => "Français",
            Lang::En => "English",
            Lang::De => "Deutsch",
            Lang::Es => "Español",
            Lang::It => "Italiano",
            Lang::Pt => "Português",
            Lang::Nl => "Nederlands",
            Lang::Pl => "Polski",
        }
    }
}

/// Résout une clé de message dans la langue donnée.
#[must_use]
#[inline]
pub fn tr(lang: Lang, key: Msg) -> &'static str {
    key.entries()[lang.idx()]
}

/// Clés de message traduisibles de l'IHM.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Msg {
    AppSubtitle,
    SettingsBtn,
    SaveSettingsBtn,
    SettingsSaved,
    SaveFailed,
    DeviceRunning,
    DeviceStopped,
    Master,
    NoMaster,
    LinkActive,
    LinkIdle,
    SecurityExposed,
    // Panneau gauche
    Commands,
    OnOff,
    SpeedSetpoint,
    Viscosity,
    PidSettings,
    // Panneau central
    Speed,
    Torque,
    Overload,
    FramesTitle,
    ClearBtn,
    SendBtn,
    CmdRefTitle,
    CmdInsertHint,
    CmdIdentity,
    CmdReadSpeed,
    CmdReadTorque,
    CmdReadSetpoint,
    CmdSetSetpoint,
    CmdStart,
    CmdStop,
    CmdReset,
    CmdWatchdog,
    LegSpeed,
    LegSetpoint,
    LegTorque,
    AxisTime,
    // Modal paramètres
    SettingsTitle,
    Language,
    NamurTransport,
    BindIp,
    Port,
    AllowedIps,
    SerialPort,
    Baud,
    Parity,
    DataBits,
    StopBits,
    ParityNone,
    ParityEven,
    ParityOdd,
    SerialPointToPoint,
    #[cfg_attr(feature = "serial", allow(dead_code))]
    SerialNoFeature,
    MotorParams,
    Inertia,
    LoadCoeff,
    Friction,
    TorqueMax,
    SpeedBounds,
    SpeedMin,
    SpeedMax,
    ViscosityBounds,
    ViscMin,
    ViscMax,
    ApplyBtn,
    ResetBtn,
    CloseBtn,
    // Vérification de mise à jour
    CheckUpdates,
    CheckNow,
    UpdateAvailable,
    UpdateDownload,
    UpToDate,
    UpdateCheckFailed,
}

impl Msg {
    #[rustfmt::skip]
    fn entries(self) -> [&'static str; 8] {
        use Msg::*;
        match self {
            AppSubtitle     => ["Agitateur NAMUR simulé", "Simulated NAMUR stirrer", "Simulierter NAMUR-Rührer", "Agitador NAMUR simulado", "Agitatore NAMUR simulato", "Agitador NAMUR simulado", "Gesimuleerde NAMUR-roerder", "Symulowane mieszadło NAMUR"],
            SettingsBtn     => ["Paramètres", "Settings", "Einstellungen", "Ajustes", "Impostazioni", "Definições", "Instellingen", "Ustawienia"],
            SaveSettingsBtn => ["Sauvegarder les réglages", "Save settings", "Einstellungen speichern", "Guardar ajustes", "Salva impostazioni", "Guardar definições", "Instellingen opslaan", "Zapisz ustawienia"],
            SettingsSaved   => ["Réglages sauvegardés", "Settings saved", "Einstellungen gespeichert", "Ajustes guardados", "Impostazioni salvate", "Definições guardadas", "Instellingen opgeslagen", "Ustawienia zapisane"],
            SaveFailed      => ["Échec de sauvegarde", "Save failed", "Speichern fehlgeschlagen", "Error al guardar", "Salvataggio non riuscito", "Falha ao guardar", "Opslaan mislukt", "Zapis nie powiódł się"],
            DeviceRunning   => ["EN MARCHE", "RUNNING", "IN BETRIEB", "EN MARCHA", "IN FUNZIONE", "EM FUNCIONAMENTO", "IN BEDRIJF", "PRACUJE"],
            DeviceStopped   => ["À L'ARRÊT", "STOPPED", "GESTOPPT", "DETENIDO", "FERMO", "PARADO", "GESTOPT", "ZATRZYMANY"],
            Master          => ["Maître", "Master", "Master", "Maestro", "Master", "Mestre", "Master", "Master"],
            NoMaster        => ["Aucun maître", "No master", "Kein Master", "Sin maestro", "Nessun master", "Sem mestre", "Geen master", "Brak mastera"],
            LinkActive      => ["Lien actif — trafic NAMUR récent", "Link active — recent NAMUR traffic", "Verbindung aktiv — kürzlich NAMUR-Verkehr", "Enlace activo — tráfico NAMUR reciente", "Collegamento attivo — traffico NAMUR recente", "Ligação ativa — tráfego NAMUR recente", "Verbinding actief — recent NAMUR-verkeer", "Łącze aktywne — niedawny ruch NAMUR"],
            LinkIdle        => ["Lien inactif — aucun trafic récent", "Link idle — no recent traffic", "Verbindung inaktiv — kein kürzlicher Verkehr", "Enlace inactivo — sin tráfico reciente", "Collegamento inattivo — nessun traffico recente", "Ligação inativa — sem tráfego recente", "Verbinding inactief — geen recent verkeer", "Łącze nieaktywne — brak ostatniego ruchu"],
            SecurityExposed => ["⚠ Serveur NAMUR exposé à tout le réseau (aucune liste blanche d'IP)", "⚠ NAMUR server exposed to the whole network (no IP allowlist)", "⚠ NAMUR-Server für das gesamte Netzwerk offen (keine IP-Whitelist)", "⚠ Servidor NAMUR expuesto a toda la red (sin lista blanca de IP)", "⚠ Server NAMUR esposto a tutta la rete (nessuna whitelist IP)", "⚠ Servidor NAMUR exposto a toda a rede (sem lista branca de IP)", "⚠ NAMUR-server blootgesteld aan het hele netwerk (geen IP-witlijst)", "⚠ Serwer NAMUR dostępny dla całej sieci (brak białej listy IP)"],
            Commands        => ["Commandes", "Commands", "Befehle", "Comandos", "Comandi", "Comandos", "Bediening", "Sterowanie"],
            OnOff           => ["Marche / Arrêt", "On / Off", "Ein / Aus", "Marcha / Paro", "Marcia / Arresto", "Ligar / Desligar", "Aan / Uit", "Wł. / Wył."],
            SpeedSetpoint   => ["Consigne de vitesse (tr/min)", "Speed setpoint (rpm)", "Drehzahl-Sollwert (U/min)", "Consigna de velocidad (rpm)", "Setpoint velocità (giri/min)", "Setpoint de velocidade (rpm)", "Toerental-setpoint (tpm)", "Zadane obroty (obr/min)"],
            Viscosity       => ["Viscosité (relative)", "Viscosity (relative)", "Viskosität (relativ)", "Viscosidad (relativa)", "Viscosità (relativa)", "Viscosidade (relativa)", "Viscositeit (relatief)", "Lepkość (względna)"],
            PidSettings     => ["Réglages PID de vitesse", "Speed PID settings", "Drehzahl-PID-Einstellungen", "Ajustes PID de velocidad", "Parametri PID velocità", "Parâmetros PID de velocidade", "Toerental-PID-instellingen", "Nastawy PID obrotów"],
            Speed           => ["Vitesse", "Speed", "Drehzahl", "Velocidad", "Velocità", "Velocidade", "Toerental", "Obroty"],
            Torque          => ["Couple", "Torque", "Drehmoment", "Par", "Coppia", "Binário", "Koppel", "Moment obrotowy"],
            Overload        => ["Surcharge", "Overload", "Überlast", "Sobrecarga", "Sovraccarico", "Sobrecarga", "Overbelasting", "Przeciążenie"],
            FramesTitle     => ["Trames NAMUR", "NAMUR frames", "NAMUR-Telegramme", "Tramas NAMUR", "Frame NAMUR", "Tramas NAMUR", "NAMUR-frames", "Ramki NAMUR"],
            ClearBtn        => ["Effacer", "Clear", "Löschen", "Borrar", "Cancella", "Limpar", "Wissen", "Wyczyść"],
            SendBtn         => ["Envoyer", "Send", "Senden", "Enviar", "Invia", "Enviar", "Verzenden", "Wyślij"],
            CmdRefTitle     => ["Protocole NAMUR", "NAMUR protocol", "NAMUR-Protokoll", "Protocolo NAMUR", "Protocollo NAMUR", "Protocolo NAMUR", "NAMUR-protocol", "Protokół NAMUR"],
            CmdInsertHint   => ["Cliquer une commande pour l'insérer", "Click a command to insert it", "Befehl zum Einfügen anklicken", "Clic en un comando para insertarlo", "Clic su un comando per inserirlo", "Clique num comando para inseri-lo", "Klik op een commando om in te voegen", "Kliknij polecenie, aby je wstawić"],
            CmdIdentity     => ["identité", "identity", "Identität", "identidad", "identità", "identidade", "identiteit", "tożsamość"],
            CmdReadSpeed    => ["vitesse mesurée", "measured speed", "gemessene Drehzahl", "velocidad medida", "velocità misurata", "velocidade medida", "gemeten toerental", "zmierzone obroty"],
            CmdReadTorque   => ["couple mesuré", "measured torque", "gemessenes Drehmoment", "par medido", "coppia misurata", "binário medido", "gemeten koppel", "zmierzony moment"],
            CmdReadSetpoint => ["consigne (lecture)", "setpoint (read)", "Sollwert (lesen)", "consigna (lectura)", "setpoint (lettura)", "setpoint (leitura)", "setpoint (lezen)", "wartość zadana (odczyt)"],
            CmdSetSetpoint  => ["régler la consigne", "set the setpoint", "Sollwert setzen", "fijar la consigna", "imposta il setpoint", "definir o setpoint", "setpoint instellen", "ustaw wartość zadaną"],
            CmdStart        => ["démarrer", "start", "starten", "arrancar", "avvia", "arrancar", "starten", "uruchom"],
            CmdStop         => ["arrêter", "stop", "stoppen", "parar", "ferma", "parar", "stoppen", "zatrzymaj"],
            CmdReset        => ["arrêt / local", "stop / local", "Stopp / lokal", "paro / local", "arresto / locale", "parar / local", "stop / lokaal", "stop / lokalnie"],
            CmdWatchdog     => ["chien de garde", "watchdog", "Watchdog", "watchdog", "watchdog", "watchdog", "watchdog", "watchdog"],
            LegSpeed        => ["Vitesse (tr/min)", "Speed (rpm)", "Drehzahl (U/min)", "Velocidad (rpm)", "Velocità (giri/min)", "Velocidade (rpm)", "Toerental (tpm)", "Obroty (obr/min)"],
            LegSetpoint     => ["Consigne (tr/min)", "Setpoint (rpm)", "Sollwert (U/min)", "Consigna (rpm)", "Setpoint (giri/min)", "Setpoint (rpm)", "Setpoint (tpm)", "Zadane (obr/min)"],
            LegTorque       => ["Couple (N·cm)", "Torque (N·cm)", "Drehmoment (N·cm)", "Par (N·cm)", "Coppia (N·cm)", "Binário (N·cm)", "Koppel (N·cm)", "Moment (N·cm)"],
            AxisTime        => ["temps (s)", "time (s)", "Zeit (s)", "tiempo (s)", "tempo (s)", "tempo (s)", "tijd (s)", "czas (s)"],
            SettingsTitle   => ["Paramètres", "Settings", "Einstellungen", "Ajustes", "Impostazioni", "Definições", "Instellingen", "Ustawienia"],
            Language        => ["Langue", "Language", "Sprache", "Idioma", "Lingua", "Idioma", "Taal", "Język"],
            NamurTransport  => ["Transport NAMUR", "NAMUR transport", "NAMUR-Transport", "Transporte NAMUR", "Trasporto NAMUR", "Transporte NAMUR", "NAMUR-transport", "Transport NAMUR"],
            BindIp          => ["IP d'écoute", "Listen IP", "Lausch-IP", "IP de escucha", "IP di ascolto", "IP de escuta", "Luister-IP", "IP nasłuchu"],
            Port            => ["Port", "Port", "Port", "Puerto", "Porta", "Porta", "Poort", "Port"],
            AllowedIps      => ["IP autorisées (une/ligne, jokers `*`, vide = toutes) :", "Allowed IPs (one/line, `*` wildcards, empty = all):", "Erlaubte IPs (eine/Zeile, Joker `*`, leer = alle):", "IP autorizadas (una/línea, comodines `*`, vacío = todas):", "IP autorizzate (una/riga, jolly `*`, vuoto = tutte):", "IP autorizadas (uma/linha, curingas `*`, vazio = todas):", "Toegestane IP's (één/regel, jokers `*`, leeg = alle):", "Dozwolone IP (jeden/linię, `*`, puste = wszystkie):"],
            SerialPort      => ["Port série", "Serial port", "Serielle Schnittstelle", "Puerto serie", "Porta seriale", "Porta série", "Seriële poort", "Port szeregowy"],
            Baud            => ["Baud", "Baud", "Baud", "Baudios", "Baud", "Baud", "Baud", "Baud"],
            Parity          => ["Parité", "Parity", "Parität", "Paridad", "Parità", "Paridade", "Pariteit", "Parzystość"],
            DataBits        => ["Bits de données", "Data bits", "Datenbits", "Bits de datos", "Bit di dati", "Bits de dados", "Databits", "Bity danych"],
            StopBits        => ["Bits de stop", "Stop bits", "Stoppbits", "Bits de parada", "Bit di stop", "Bits de paragem", "Stopbits", "Bity stopu"],
            ParityNone      => ["Aucune", "None", "Keine", "Ninguna", "Nessuna", "Nenhuma", "Geen", "Brak"],
            ParityEven      => ["Paire", "Even", "Gerade", "Par", "Pari", "Par", "Even", "Parzysta"],
            ParityOdd       => ["Impaire", "Odd", "Ungerade", "Impar", "Dispari", "Ímpar", "Oneven", "Nieparzysta"],
            SerialPointToPoint => ["Série : liaison point-à-point (un seul appareil).", "Serial: point-to-point link (single device).", "Seriell: Punkt-zu-Punkt-Verbindung (ein Gerät).", "Serie: enlace punto a punto (un solo equipo).", "Seriale: collegamento punto-punto (un solo dispositivo).", "Série: ligação ponto a ponto (um só equipamento).", "Serieel: punt-naar-punt-verbinding (één apparaat).", "Szeregowy: połączenie punkt-punkt (jedno urządzenie)."],
            SerialNoFeature => ["⚠ binaire compilé sans la feature `serial` : le transport série ne démarrera pas.", "⚠ binary built without the `serial` feature: the serial transport will not start.", "⚠ Binärdatei ohne Feature `serial`: serieller Transport startet nicht.", "⚠ binario compilado sin la feature `serial`: el transporte serie no arrancará.", "⚠ binario compilato senza la feature `serial`: il trasporto seriale non si avvierà.", "⚠ binário compilado sem a feature `serial`: o transporte série não arrancará.", "⚠ binary zonder de `serial`-feature: het seriële transport start niet.", "⚠ binarka bez funkcji `serial`: transport szeregowy nie wystartuje."],
            MotorParams     => ["Moteur (fonction de transfert)", "Motor (transfer function)", "Motor (Übertragungsfunktion)", "Motor (función de transferencia)", "Motore (funzione di trasferimento)", "Motor (função de transferência)", "Motor (overdrachtsfunctie)", "Silnik (transmitancja)"],
            Inertia         => ["Inertie (réactivité)", "Inertia (responsiveness)", "Trägheit (Reaktivität)", "Inercia (reactividad)", "Inerzia (reattività)", "Inércia (reatividade)", "Traagheid (reactiviteit)", "Bezwładność (reaktywność)"],
            LoadCoeff       => ["Coeff. de charge visqueuse", "Viscous load coeff.", "Viskose Lastkoeff.", "Coef. de carga viscosa", "Coeff. carico viscoso", "Coef. de carga viscosa", "Viskeuze-belastingscoëff.", "Wsp. obciążenia lepkiego"],
            Friction        => ["Frottement (N·cm)", "Friction (N·cm)", "Reibung (N·cm)", "Fricción (N·cm)", "Attrito (N·cm)", "Atrito (N·cm)", "Wrijving (N·cm)", "Tarcie (N·cm)"],
            TorqueMax       => ["Couple max (N·cm)", "Max torque (N·cm)", "Max. Drehmoment (N·cm)", "Par máx (N·cm)", "Coppia max (N·cm)", "Binário máx (N·cm)", "Max. koppel (N·cm)", "Maks. moment (N·cm)"],
            SpeedBounds     => ["Bornes de vitesse (tr/min)", "Speed bounds (rpm)", "Drehzahlgrenzen (U/min)", "Límites de velocidad (rpm)", "Limiti velocità (giri/min)", "Limites de velocidade (rpm)", "Toerentalgrenzen (tpm)", "Granice obrotów (obr/min)"],
            SpeedMin        => ["Vitesse min", "Speed min", "Drehzahl min", "Velocidad mín", "Velocità min", "Velocidade mín", "Toerental min", "Obroty min"],
            SpeedMax        => ["Vitesse max", "Speed max", "Drehzahl max", "Velocidad máx", "Velocità max", "Velocidade máx", "Toerental max", "Obroty maks"],
            ViscosityBounds => ["Bornes de viscosité", "Viscosity bounds", "Viskositätsgrenzen", "Límites de viscosidad", "Limiti viscosità", "Limites de viscosidade", "Viscositeitsgrenzen", "Granice lepkości"],
            ViscMin         => ["Visc. min", "Visc. min", "Visk. min", "Visc. mín", "Visc. min", "Visc. mín", "Visc. min", "Lepkość min"],
            ViscMax         => ["Visc. max", "Visc. max", "Visk. max", "Visc. máx", "Visc. max", "Visc. máx", "Visc. max", "Lepkość maks"],
            ApplyBtn        => ["Appliquer", "Apply", "Anwenden", "Aplicar", "Applica", "Aplicar", "Toepassen", "Zastosuj"],
            ResetBtn        => ["Réinitialiser par défaut", "Reset to defaults", "Auf Standard zurücksetzen", "Restablecer valores", "Ripristina predefiniti", "Repor predefinições", "Standaard herstellen", "Przywróć domyślne"],
            CloseBtn        => ["Fermer", "Close", "Schließen", "Cerrar", "Chiudi", "Fechar", "Sluiten", "Zamknij"],
            CheckUpdates    => ["Vérifier les mises à jour au démarrage", "Check for updates at startup", "Beim Start nach Updates suchen", "Buscar actualizaciones al iniciar", "Controlla aggiornamenti all'avvio", "Procurar atualizações ao iniciar", "Bij opstarten op updates controleren", "Sprawdzaj aktualizacje przy starcie"],
            CheckNow        => ["Vérifier maintenant", "Check now", "Jetzt prüfen", "Comprobar ahora", "Controlla ora", "Verificar agora", "Nu controleren", "Sprawdź teraz"],
            UpdateAvailable => ["🔔 Mise à jour disponible :", "🔔 Update available:", "🔔 Update verfügbar:", "🔔 Actualización disponible:", "🔔 Aggiornamento disponibile:", "🔔 Atualização disponível:", "🔔 Update beschikbaar:", "🔔 Dostępna aktualizacja:"],
            UpdateDownload  => ["Télécharger", "Download", "Herunterladen", "Descargar", "Scarica", "Transferir", "Downloaden", "Pobierz"],
            UpToDate        => ["Logiciel à jour", "Up to date", "Aktuell", "Actualizado", "Aggiornato", "Atualizado", "Up-to-date", "Aktualne"],
            UpdateCheckFailed => ["Vérification impossible (hors ligne ?)", "Check failed (offline?)", "Prüfung fehlgeschlagen (offline?)", "Comprobación fallida (¿sin conexión?)", "Controllo non riuscito (offline?)", "Verificação falhou (offline?)", "Controle mislukt (offline?)", "Sprawdzenie nie powiodło się (offline?)"],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_translation_non_empty() {
        for lang in Lang::ALL {
            for key in [Msg::AppSubtitle, Msg::Speed, Msg::Torque, Msg::Viscosity, Msg::CloseBtn] {
                assert!(!tr(lang, key).is_empty(), "{lang:?}/{key:?} vide");
            }
        }
    }

    #[test]
    fn lang_round_trips_through_toml() {
        for lang in Lang::ALL {
            #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
            struct W {
                lang: Lang,
            }
            let s = toml::to_string(&W { lang }).unwrap();
            assert_eq!(toml::from_str::<W>(&s).unwrap().lang, lang);
        }
    }
}
