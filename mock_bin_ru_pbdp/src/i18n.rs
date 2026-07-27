//! Internationalisation (i18n) de l'IHM — catalogue de chaînes traduites.
//!
//! Seules les chaînes **destinées à l'opérateur** (interface graphique) sont
//! traduites. Les logs, messages d'erreur internes et l'état protocolaire brut
//! (`Wait_Prm`, `Data_Exchange`…) restent en anglais/technique et ne sont pas
//! traduits (cf. conventions du projet).
//!
//! Le compilateur garantit qu'aucune clé n'est oubliée (match exhaustif) et que
//! chaque clé possède exactement 8 traductions (tableau de taille fixe).

use serde::{Deserialize, Serialize};

/// Langue de l'interface graphique.
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
    // --- Bandeau supérieur ---
    SettingsBtn,
    SaveSettingsBtn,
    AppSubtitle,
    DeviceRunning,
    DeviceStopped,
    SettingsSaved,
    SaveFailed,
    LinkActive,
    LinkIdle,
    /// Bandeau permanent : ce simulateur ne respecte pas le timing du bus réel.
    NonInterop,
    // --- Mini-terminal : trames PROFIBUS (hex) ---
    FramesTitle,
    ClearBtn,
    // --- Panneau gauche : commandes ---
    Commands,
    OnOff,
    ModeLabel,
    Manual,
    Auto,
    RegModes,
    Sens1Hot,
    Sens2Cold,
    Setpoints,
    SpAuto,
    SpManual,
    PidSens1,
    PidSens2,
    TorPwmSettings,
    HystSlider,
    TorMinCycleSlider,
    PwmPeriodSlider,
    HintAntiShortCycle,
    HintCyclicRelay,
    // --- Panneau droit : blocs I/O PROFIBUS ---
    IoTable,
    IoTableNote,
    ColName,
    ColBlock,
    ColOffset,
    ColValue,
    ColAccess,
    // --- Panneau central : supervision + courbe ---
    Measure,
    ActiveSetpoint,
    Output,
    OutputPct,
    LegSetpoint,
    LegMeasure,
    LegOutput,
    ManualDash,
    AxisTime,
    // --- Libellés de mode de régulation ---
    ModeOff,
    ModePid,
    ModeOnOff,
    ModePwm,
    // --- Modal Paramètres ---
    SettingsTitle,
    Language,
    SerialPort,
    Baud,
    StationAddress,
    WatchdogEnabled,
    ProcessTf,
    GainK,
    ConstT,
    DelayL,
    Ambient,
    SpBounds,
    SpMin,
    SpMax,
    ApplyBtn,
    ResetBtn,
    CloseBtn,
    // --- Noms de lignes de la table I/O (IHM) ---
    RowRunning,
    RowHeatingActive,
    RowCoolingActive,
    RowModeSens1,
    RowModeSens2,
    RowHysteresis,
    Dir1,
    Dir2,
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
            SettingsBtn        => ["Paramètres", "Settings", "Einstellungen", "Ajustes", "Impostazioni", "Definições", "Instellingen", "Ustawienia"],
            SaveSettingsBtn    => ["Sauvegarder les réglages", "Save settings", "Einstellungen speichern", "Guardar ajustes", "Salva impostazioni", "Guardar definições", "Instellingen opslaan", "Zapisz ustawienia"],
            AppSubtitle        => ["Régulateur PROFIBUS DP simulé (non interopérable)", "Simulated PROFIBUS DP controller (not interoperable)", "Simulierter PROFIBUS-DP-Regler (nicht interoperabel)", "Regulador PROFIBUS DP simulado (no interoperable)", "Regolatore PROFIBUS DP simulato (non interoperabile)", "Regulador PROFIBUS DP simulado (não interoperável)", "Gesimuleerde PROFIBUS DP-regelaar (niet interoperabel)", "Symulowany regulator PROFIBUS DP (niekompatybilny)"],
            DeviceRunning      => ["EN MARCHE", "RUNNING", "IN BETRIEB", "EN MARCHA", "IN FUNZIONE", "EM FUNCIONAMENTO", "IN BEDRIJF", "PRACUJE"],
            DeviceStopped      => ["À L'ARRÊT", "STOPPED", "GESTOPPT", "DETENIDO", "FERMO", "PARADO", "GESTOPT", "ZATRZYMANY"],
            SettingsSaved      => ["Réglages sauvegardés", "Settings saved", "Einstellungen gespeichert", "Ajustes guardados", "Impostazioni salvate", "Definições guardadas", "Instellingen opgeslagen", "Ustawienia zapisane"],
            SaveFailed         => ["Échec de sauvegarde", "Save failed", "Speichern fehlgeschlagen", "Error al guardar", "Salvataggio non riuscito", "Falha ao guardar", "Opslaan mislukt", "Zapis nie powiódł się"],
            LinkActive         => ["Lien actif — trafic PROFIBUS récent", "Link active — recent PROFIBUS traffic", "Verbindung aktiv — kürzlich PROFIBUS-Verkehr", "Enlace activo — tráfico PROFIBUS reciente", "Collegamento attivo — traffico PROFIBUS recente", "Ligação ativa — tráfego PROFIBUS recente", "Verbinding actief — recent PROFIBUS-verkeer", "Łącze aktywne — niedawny ruch PROFIBUS"],
            LinkIdle           => ["Lien inactif — aucun trafic récent", "Link idle — no recent traffic", "Verbindung inaktiv — kein kürzlicher Verkehr", "Enlace inactivo — sin tráfico reciente", "Collegamento inattivo — nessun traffico recente", "Ligação inativa — sem tráfego recente", "Verbinding inactief — geen recent verkeer", "Łącze nieaktywne — brak ostatniego ruchu"],
            NonInterop         => ["⚠ Simulateur logiciel : timing de bus réel non respecté — non interopérable avec un maître PROFIBUS DP matériel", "⚠ Software simulator: real bus timing not met — not interoperable with a hardware PROFIBUS DP master", "⚠ Software-Simulator: reales Bus-Timing wird nicht eingehalten — nicht interoperabel mit einem echten PROFIBUS-DP-Master", "⚠ Simulador por software: no cumple el timing real del bus — no interoperable con un maestro PROFIBUS DP real", "⚠ Simulatore software: non rispetta il timing reale del bus — non interoperabile con un master PROFIBUS DP reale", "⚠ Simulador por software: não cumpre o timing real do barramento — não interoperável com um mestre PROFIBUS DP real", "⚠ Softwaresimulator: voldoet niet aan de echte bustiming — niet interoperabel met een fysieke PROFIBUS DP-master", "⚠ Symulator programowy: nie spełnia rzeczywistego taktowania magistrali — niekompatybilny ze sprzętowym masterem PROFIBUS DP"],
            FramesTitle        => ["Trames PROFIBUS (hex)", "PROFIBUS frames (hex)", "PROFIBUS-Telegramme (hex)", "Tramas PROFIBUS (hex)", "Trame PROFIBUS (hex)", "Tramas PROFIBUS (hex)", "PROFIBUS-telegrammen (hex)", "Ramki PROFIBUS (hex)"],
            ClearBtn           => ["Effacer", "Clear", "Löschen", "Borrar", "Cancella", "Limpar", "Wissen", "Wyczyść"],
            Commands           => ["Commandes", "Commands", "Befehle", "Comandos", "Comandi", "Comandos", "Bediening", "Sterowanie"],
            OnOff              => ["Marche / Arrêt", "On / Off", "Ein / Aus", "Marcha / Paro", "Marcia / Arresto", "Ligar / Desligar", "Aan / Uit", "Wł. / Wył."],
            ModeLabel          => ["Mode :", "Mode:", "Modus:", "Modo:", "Modalità:", "Modo:", "Modus:", "Tryb:"],
            Manual             => ["Manuel", "Manual", "Manuell", "Manual", "Manuale", "Manual", "Handmatig", "Ręczny"],
            Auto               => ["Auto", "Auto", "Auto", "Auto", "Auto", "Auto", "Auto", "Auto"],
            RegModes           => ["Modes de régulation", "Control modes", "Regelungsarten", "Modos de regulación", "Modalità di regolazione", "Modos de regulação", "Regelmodi", "Tryby regulacji"],
            Sens1Hot           => ["Sens 1 (chaud) :", "Direction 1 (heating):", "Richtung 1 (Heizen):", "Sentido 1 (calor):", "Verso 1 (caldo):", "Sentido 1 (aquecer):", "Richting 1 (verwarmen):", "Kierunek 1 (grzanie):"],
            Sens2Cold          => ["Sens 2 (froid) :", "Direction 2 (cooling):", "Richtung 2 (Kühlen):", "Sentido 2 (frío):", "Verso 2 (freddo):", "Sentido 2 (arrefecer):", "Richting 2 (koelen):", "Kierunek 2 (chłodzenie):"],
            Setpoints          => ["Consignes", "Setpoints", "Sollwerte", "Consignas", "Setpoint", "Setpoints", "Setpoints", "Wartości zadane"],
            SpAuto             => ["SP auto", "SP auto", "SP auto", "SP auto", "SP auto", "SP auto", "SP auto", "SP auto"],
            SpManual           => ["SP manuel", "SP manual", "SP manuell", "SP manual", "SP manuale", "SP manual", "SP handmatig", "SP ręczny"],
            PidSens1           => ["Réglages PID sens 1 (chaud)", "PID settings, direction 1 (heating)", "PID-Einstellungen Richtung 1 (Heizen)", "Ajustes PID sentido 1 (calor)", "Parametri PID verso 1 (caldo)", "Parâmetros PID sentido 1 (aquecer)", "PID-instellingen richting 1 (verwarmen)", "Nastawy PID kierunek 1 (grzanie)"],
            PidSens2           => ["Réglages PID sens 2 (froid)", "PID settings, direction 2 (cooling)", "PID-Einstellungen Richtung 2 (Kühlen)", "Ajustes PID sentido 2 (frío)", "Parametri PID verso 2 (freddo)", "Parâmetros PID sentido 2 (arrefecer)", "PID-instellingen richting 2 (koelen)", "Nastawy PID kierunek 2 (chłodzenie)"],
            TorPwmSettings     => ["Réglages TOR / PWM", "On/off (TOR) / PWM settings", "Zweipunkt (TOR) / PWM-Einstellungen", "Ajustes TOR / PWM", "Impostazioni TOR / PWM", "Definições TOR / PWM", "TOR- / PWM-instellingen", "Ustawienia TOR / PWM"],
            HystSlider         => ["Hystérésis TOR", "Hysteresis (TOR)", "Hysterese (TOR)", "Histéresis (TOR)", "Isteresi (TOR)", "Histerese (TOR)", "Hysterese (TOR)", "Histereza (TOR)"],
            TorMinCycleSlider  => ["Cycle min. TOR (s)", "Min. cycle TOR (s)", "Min. Zyklus TOR (s)", "Ciclo mín. TOR (s)", "Ciclo min. TOR (s)", "Ciclo mín. TOR (s)", "Min. cyclus TOR (s)", "Min. cykl TOR (s)"],
            PwmPeriodSlider    => ["Période PWM (s)", "PWM period (s)", "PWM-Periode (s)", "Periodo PWM (s)", "Periodo PWM (s)", "Período PWM (s)", "PWM-periode (s)", "Okres PWM (s)"],
            HintAntiShortCycle => ["Anti-court-cycle", "Anti-short-cycle", "Taktschutz", "Anti ciclo corto", "Anti ciclo breve", "Anti ciclo curto", "Antikortcyclus", "Zabezp. krótkiego cyklu"],
            HintCyclicRelay    => ["Relais à cycle", "Time-proportioning relay", "Taktrelais", "Relé de ciclo", "Relè a ciclo", "Relé de ciclo", "Cyclusrelais", "Przekaźnik cykliczny"],
            IoTable            => ["Blocs d'entrées/sorties PROFIBUS", "PROFIBUS I/O blocks", "PROFIBUS-E/A-Blöcke", "Bloques de E/S PROFIBUS", "Blocchi I/O PROFIBUS", "Blocos de E/S PROFIBUS", "PROFIBUS I/O-blokken", "Bloki wejść/wyjść PROFIBUS"],
            IoTableNote        => ["f32 = 4 octets, big-endian", "f32 = 4 bytes, big-endian", "f32 = 4 Bytes, Big-Endian", "f32 = 4 bytes, big-endian", "f32 = 4 byte, big-endian", "f32 = 4 bytes, big-endian", "f32 = 4 bytes, big-endian", "f32 = 4 bajty, big-endian"],
            ColName            => ["Désignation", "Name", "Bezeichnung", "Designación", "Designazione", "Designação", "Naam", "Nazwa"],
            ColBlock           => ["Bloc", "Block", "Block", "Bloque", "Blocco", "Bloco", "Blok", "Blok"],
            ColOffset          => ["Octet(s)", "Byte(s)", "Byte(s)", "Byte(s)", "Byte", "Byte(s)", "Byte(s)", "Bajt(y)"],
            ColValue           => ["Valeur", "Value", "Wert", "Valor", "Valore", "Valor", "Waarde", "Wartość"],
            ColAccess          => ["Accès", "Access", "Zugriff", "Acceso", "Accesso", "Acesso", "Toegang", "Dostęp"],
            Measure            => ["Mesure (PV)", "Measurement (PV)", "Messwert (PV)", "Medida (PV)", "Misura (PV)", "Medição (PV)", "Meting (PV)", "Pomiar (PV)"],
            ActiveSetpoint     => ["Consigne active", "Active setpoint", "Aktiver Sollwert", "Consigna activa", "Setpoint attivo", "Setpoint ativo", "Actief setpoint", "Aktywna wartość zadana"],
            Output             => ["Sortie", "Output", "Ausgang", "Salida", "Uscita", "Saída", "Uitgang", "Wyjście"],
            OutputPct          => ["Sortie (%)", "Output (%)", "Ausgang (%)", "Salida (%)", "Uscita (%)", "Saída (%)", "Uitgang (%)", "Wyjście (%)"],
            LegSetpoint        => ["Consigne (SP)", "Setpoint (SP)", "Sollwert (SP)", "Consigna (SP)", "Setpoint (SP)", "Setpoint (SP)", "Setpoint (SP)", "Wartość zadana (SP)"],
            LegMeasure         => ["Mesure (PV)", "Measurement (PV)", "Messwert (PV)", "Medida (PV)", "Misura (PV)", "Medição (PV)", "Meting (PV)", "Pomiar (PV)"],
            LegOutput          => ["Sortie (%)", "Output (%)", "Ausgang (%)", "Salida (%)", "Uscita (%)", "Saída (%)", "Uitgang (%)", "Wyjście (%)"],
            ManualDash         => ["— (manuel)", "— (manual)", "— (manuell)", "— (manual)", "— (manuale)", "— (manual)", "— (handmatig)", "— (ręczny)"],
            AxisTime           => ["temps (s)", "time (s)", "Zeit (s)", "tiempo (s)", "tempo (s)", "tempo (s)", "tijd (s)", "czas (s)"],
            ModeOff            => ["Désactivé", "Disabled", "Deaktiviert", "Desactivado", "Disattivato", "Desativado", "Uitgeschakeld", "Wyłączony"],
            ModePid            => ["PID", "PID", "PID", "PID", "PID", "PID", "PID", "PID"],
            ModeOnOff          => ["Tout-ou-rien (TOR)", "On/off (TOR)", "Zweipunkt (TOR)", "Todo-nada (TOR)", "Tutto-niente (TOR)", "Tudo-ou-nada (TOR)", "Aan/uit (TOR)", "Dwustawny (TOR)"],
            ModePwm            => ["Relais à cycle (PWM)", "Time-proportioning relay (PWM)", "Taktrelais (PWM)", "Relé de ciclo (PWM)", "Relè a ciclo (PWM)", "Relé de ciclo (PWM)", "Cyclusrelais (PWM)", "Przekaźnik cykliczny (PWM)"],
            SettingsTitle      => ["Paramètres", "Settings", "Einstellungen", "Ajustes", "Impostazioni", "Definições", "Instellingen", "Ustawienia"],
            Language           => ["Langue", "Language", "Sprache", "Idioma", "Lingua", "Idioma", "Taal", "Język"],
            SerialPort         => ["Port série", "Serial port", "Serielle Schnittstelle", "Puerto serie", "Porta seriale", "Porta série", "Seriële poort", "Port szeregowy"],
            Baud               => ["Baud", "Baud", "Baud", "Baudios", "Baud", "Baud", "Baud", "Baud"],
            StationAddress     => ["Adresse de station", "Station address", "Stationsadresse", "Dirección de estación", "Indirizzo di stazione", "Endereço de estação", "Stationsadres", "Adres stacji"],
            WatchdogEnabled    => ["Chien de garde protocolaire (autorisé)", "Protocol watchdog (allowed)", "Protokoll-Watchdog (zulässig)", "Perro guardián de protocolo (permitido)", "Watchdog di protocollo (consentito)", "Watchdog de protocolo (permitido)", "Protocol-watchdog (toegestaan)", "Watchdog protokołu (dozwolony)"],
            ProcessTf          => ["Fonction de transfert (procédé)", "Transfer function (process)", "Übertragungsfunktion (Prozess)", "Función de transferencia (proceso)", "Funzione di trasferimento (processo)", "Função de transferência (processo)", "Overdrachtsfunctie (proces)", "Transmitancja (proces)"],
            GainK              => ["Gain K (u/%)", "Gain K (u/%)", "Verstärkung K (E/%)", "Ganancia K (u/%)", "Guadagno K (u/%)", "Ganho K (u/%)", "Versterking K (e/%)", "Wzmocnienie K (j./%)"],
            ConstT             => ["Constante T (s)", "Time constant T (s)", "Zeitkonstante T (s)", "Constante T (s)", "Costante T (s)", "Constante T (s)", "Tijdconstante T (s)", "Stała czasowa T (s)"],
            DelayL             => ["Retard L (s)", "Dead time L (s)", "Totzeit L (s)", "Retardo L (s)", "Ritardo L (s)", "Atraso L (s)", "Vertraging L (s)", "Opóźnienie L (s)"],
            Ambient            => ["Ambiant", "Ambient", "Umgebung", "Ambiente", "Ambiente", "Ambiente", "Omgeving", "Otoczenie"],
            SpBounds           => ["Bornes de consigne", "Setpoint bounds", "Sollwertgrenzen", "Límites de consigna", "Limiti del setpoint", "Limites do setpoint", "Setpointgrenzen", "Granice wartości zadanej"],
            SpMin              => ["SP min", "SP min", "SP min", "SP mín", "SP min", "SP mín", "SP min", "SP min"],
            SpMax              => ["SP max", "SP max", "SP max", "SP máx", "SP max", "SP máx", "SP max", "SP maks"],
            ApplyBtn           => ["Appliquer", "Apply", "Anwenden", "Aplicar", "Applica", "Aplicar", "Toepassen", "Zastosuj"],
            ResetBtn           => ["Réinitialiser par défaut", "Reset to defaults", "Auf Standard zurücksetzen", "Restablecer valores", "Ripristina predefiniti", "Repor predefinições", "Standaard herstellen", "Przywróć domyślne"],
            CloseBtn           => ["Fermer", "Close", "Schließen", "Cerrar", "Chiudi", "Fechar", "Sluiten", "Zamknij"],
            RowRunning         => ["En marche", "Running", "In Betrieb", "En marcha", "In funzione", "Em funcionamento", "In bedrijf", "Pracuje"],
            RowHeatingActive   => ["Chaud actif", "Heating active", "Heizen aktiv", "Calor activo", "Caldo attivo", "Aquecimento ativo", "Verwarmen actief", "Grzanie aktywne"],
            RowCoolingActive   => ["Froid actif", "Cooling active", "Kühlen aktiv", "Frío activo", "Freddo attivo", "Arrefecimento ativo", "Koelen actief", "Chłodzenie aktywne"],
            RowModeSens1       => ["Mode sens 1", "Mode direction 1", "Modus Richtung 1", "Modo sentido 1", "Modalità verso 1", "Modo sentido 1", "Modus richting 1", "Tryb kierunek 1"],
            RowModeSens2       => ["Mode sens 2", "Mode direction 2", "Modus Richtung 2", "Modo sentido 2", "Modalità verso 2", "Modo sentido 2", "Modus richting 2", "Tryb kierunek 2"],
            RowHysteresis      => ["Hystérésis", "Hysteresis", "Hysterese", "Histéresis", "Isteresi", "Histerese", "Hysterese", "Histereza"],
            Dir1               => ["sens 1", "dir. 1", "Richtung 1", "sentido 1", "verso 1", "sentido 1", "richting 1", "kier. 1"],
            Dir2               => ["sens 2", "dir. 2", "Richtung 2", "sentido 2", "verso 2", "sentido 2", "richting 2", "kier. 2"],
            CheckUpdates       => ["Vérifier les mises à jour au démarrage", "Check for updates at startup", "Beim Start nach Updates suchen", "Buscar actualizaciones al iniciar", "Controlla aggiornamenti all'avvio", "Procurar atualizações ao iniciar", "Bij opstarten op updates controleren", "Sprawdzaj aktualizacje przy starcie"],
            CheckNow           => ["Vérifier maintenant", "Check now", "Jetzt prüfen", "Comprobar ahora", "Controlla ora", "Verificar agora", "Nu controleren", "Sprawdź teraz"],
            UpdateAvailable    => ["🔔 Mise à jour disponible :", "🔔 Update available:", "🔔 Update verfügbar:", "🔔 Actualización disponible:", "🔔 Aggiornamento disponibile:", "🔔 Atualização disponível:", "🔔 Update beschikbaar:", "🔔 Dostępna aktualizacja:"],
            UpdateDownload     => ["Télécharger", "Download", "Herunterladen", "Descargar", "Scarica", "Transferir", "Downloaden", "Pobierz"],
            UpToDate           => ["Logiciel à jour", "Up to date", "Aktuell", "Actualizado", "Aggiornato", "Atualizado", "Up-to-date", "Aktualne"],
            UpdateCheckFailed  => ["Vérification impossible (hors ligne ?)", "Check failed (offline?)", "Prüfung fehlgeschlagen (offline?)", "Comprobación fallida (¿sin conexión?)", "Controllo non riuscito (offline?)", "Verificação falhou (offline?)", "Controle mislukt (offline?)", "Sprawdzenie nie powiodło się (offline?)"],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_has_eight_distinct_langs() {
        assert_eq!(Lang::ALL.len(), 8);
        assert_eq!(Lang::Fr.idx(), 0);
        assert_eq!(Lang::Pl.idx(), 7);
    }

    #[test]
    fn every_translation_is_non_empty() {
        let keys = [Msg::Commands, Msg::SettingsTitle, Msg::ModeOnOff, Msg::NonInterop, Msg::AxisTime, Msg::Dir2];
        for key in keys {
            for lang in Lang::ALL {
                assert!(!tr(lang, key).is_empty(), "{lang:?}/{key:?} vide");
            }
        }
    }

    #[test]
    fn native_names_are_set() {
        for lang in Lang::ALL {
            assert!(!lang.native_name().is_empty());
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
            let back: W = toml::from_str(&s).unwrap();
            assert_eq!(back.lang, lang);
        }
    }
}
