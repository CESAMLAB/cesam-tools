//! Internationalisation (i18n) de l'IHM — catalogue de chaînes traduites.
//!
//! Seules les chaînes **destinées à l'opérateur** (interface graphique) sont
//! traduites. Les logs, messages d'erreur internes et commentaires restent en
//! **français** (cf. conventions du projet).
//!
//! # Principe
//!
//! - [`Lang`] énumère les langues disponibles (sélection dans le modal
//!   *Paramètres*, persistée dans le TOML via `AppConfig`).
//! - [`Msg`] énumère les **clés** de message ; chaque clé renvoie un tableau de
//!   8 traductions, dans l'ordre des variantes de [`Lang`].
//! - [`tr`] résout `(langue, clé) -> &'static str`.
//!
//! Le compilateur garantit qu'aucune clé n'est oubliée (match exhaustif) et que
//! chaque clé possède exactement 8 traductions (tableau de taille fixe).

use serde::{Deserialize, Serialize};

/// Langue de l'interface graphique.
///
/// L'ordre des variantes **fixe** l'indexation des tableaux de traduction de
/// [`Msg::entries`] : `Fr = 0, En = 1, …, Pl = 7`. Ne pas réordonner sans
/// adapter tous les tableaux.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    /// Français (langue source).
    #[default]
    Fr,
    /// Anglais.
    En,
    /// Allemand.
    De,
    /// Espagnol.
    Es,
    /// Italien.
    It,
    /// Portugais.
    Pt,
    /// Néerlandais.
    Nl,
    /// Polonais.
    Pl,
}

impl Lang {
    /// Toutes les langues, dans l'ordre d'affichage du sélecteur.
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

    /// Indice du tableau de traductions correspondant à la langue.
    #[inline]
    fn idx(self) -> usize {
        self as usize
    }

    /// Nom de la langue dans la langue elle-même (pour le sélecteur).
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
///
/// Les chaînes purement techniques (acronymes Modbus `Coil`/`DI`/`HR`/`IR`,
/// codes d'accès `R`/`R/W`, suffixes d'unité, gabarits d'adresses, formules)
/// ne sont **pas** traduites et restent codées en dur à l'appel.
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
    // Voyant de connexion (maître Modbus / activité du lien).
    Master,
    NoMaster,
    LinkActive,
    LinkIdle,
    /// Avertissement de sécurité : serveur exposé sans filtrage d'IP.
    SecurityExposed,
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
    // --- Panneau droit : table Modbus ---
    ModbusTable,
    ModbusTableNote,
    ColName,
    ColTable,
    ColAddr,
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
    // --- Libellés de parité ---
    ParityNone,
    ParityEven,
    ParityOdd,
    // --- Modal Paramètres ---
    SettingsTitle,
    Language,
    ModbusTransport,
    BindIp,
    Port,
    AllowedIps,
    SerialPort,
    Baud,
    Parity,
    DataBits,
    StopBits,
    SlaveId,
    RtuPointToPoint,
    /// Affiché uniquement dans un binaire compilé **sans** la feature `rtu` ;
    /// donc « jamais construit » quand `rtu` est active (cas par défaut).
    #[cfg_attr(feature = "rtu", allow(dead_code))]
    RtuNoFeature,
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
    // --- Noms de lignes de la table Modbus (IHM) ---
    RowRunning,
    RowHeatingActive,
    RowCoolingActive,
    RowModeSens1,
    RowModeSens2,
    RowHysteresis,
    RowIdent,
    /// Mot composé en suffixe pour les recopies en lecture seule (« SP auto (recopie) »).
    Readback,
    // Mots réutilisés pour composer les libellés des gains PID.
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
    /// Tableau des 8 traductions, dans l'ordre des variantes de [`Lang`]
    /// (`Fr, En, De, Es, It, Pt, Nl, Pl`).
    #[rustfmt::skip]
    fn entries(self) -> [&'static str; 8] {
        use Msg::*;
        match self {
            // --- Bandeau supérieur ---
            SettingsBtn        => ["Paramètres", "Settings", "Einstellungen", "Ajustes", "Impostazioni", "Definições", "Instellingen", "Ustawienia"],
            SaveSettingsBtn    => ["Sauvegarder les réglages", "Save settings", "Einstellungen speichern", "Guardar ajustes", "Salva impostazioni", "Guardar definições", "Instellingen opslaan", "Zapisz ustawienia"],
            AppSubtitle        => ["Régulateur Modbus simulé", "Simulated Modbus controller", "Simulierter Modbus-Regler", "Regulador Modbus simulado", "Regolatore Modbus simulato", "Regulador Modbus simulado", "Gesimuleerde Modbus-regelaar", "Symulowany regulator Modbus"],
            DeviceRunning      => ["EN MARCHE", "RUNNING", "IN BETRIEB", "EN MARCHA", "IN FUNZIONE", "EM FUNCIONAMENTO", "IN BEDRIJF", "PRACUJE"],
            DeviceStopped      => ["À L'ARRÊT", "STOPPED", "GESTOPPT", "DETENIDO", "FERMO", "PARADO", "GESTOPT", "ZATRZYMANY"],
            SettingsSaved      => ["Réglages sauvegardés", "Settings saved", "Einstellungen gespeichert", "Ajustes guardados", "Impostazioni salvate", "Definições guardadas", "Instellingen opgeslagen", "Ustawienia zapisane"],
            SaveFailed         => ["Échec de sauvegarde", "Save failed", "Speichern fehlgeschlagen", "Error al guardar", "Salvataggio non riuscito", "Falha ao guardar", "Opslaan mislukt", "Zapis nie powiódł się"],
            Master             => ["Maître", "Master", "Master", "Maestro", "Master", "Mestre", "Master", "Master"],
            NoMaster           => ["Aucun maître", "No master", "Kein Master", "Sin maestro", "Nessun master", "Sem mestre", "Geen master", "Brak mastera"],
            LinkActive         => ["Lien actif — trafic Modbus récent", "Link active — recent Modbus traffic", "Verbindung aktiv — kürzlich Modbus-Verkehr", "Enlace activo — tráfico Modbus reciente", "Collegamento attivo — traffico Modbus recente", "Ligação ativa — tráfego Modbus recente", "Verbinding actief — recent Modbus-verkeer", "Łącze aktywne — niedawny ruch Modbus"],
            LinkIdle           => ["Lien inactif — aucun trafic récent", "Link idle — no recent traffic", "Verbindung inaktiv — kein kürzlicher Verkehr", "Enlace inactivo — sin tráfico reciente", "Collegamento inattivo — nessun traffico recente", "Ligação inativa — sem tráfego recente", "Verbinding inactief — geen recent verkeer", "Łącze nieaktywne — brak ostatniego ruchu"],
            SecurityExposed    => ["⚠ Serveur Modbus exposé à tout le réseau (aucune liste blanche d'IP)", "⚠ Modbus server exposed to the whole network (no IP allowlist)", "⚠ Modbus-Server für das gesamte Netzwerk offen (keine IP-Whitelist)", "⚠ Servidor Modbus expuesto a toda la red (sin lista blanca de IP)", "⚠ Server Modbus esposto a tutta la rete (nessuna whitelist IP)", "⚠ Servidor Modbus exposto a toda a rede (sem lista branca de IP)", "⚠ Modbus-server blootgesteld aan het hele netwerk (geen IP-witlijst)", "⚠ Serwer Modbus dostępny dla całej sieci (brak białej listy IP)"],
            // --- Panneau gauche : commandes ---
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
            // --- Panneau droit : table Modbus ---
            ModbusTable        => ["Table d'adresses Modbus", "Modbus address map", "Modbus-Adresstabelle", "Mapa de direcciones Modbus", "Mappa indirizzi Modbus", "Mapa de endereços Modbus", "Modbus-adrestabel", "Tabela adresów Modbus"],
            ModbusTableNote    => ["f32 = 2 registres, big-endian (mot fort en tête)", "f32 = 2 registers, big-endian (high word first)", "f32 = 2 Register, Big-Endian (High-Word zuerst)", "f32 = 2 registros, big-endian (palabra alta primero)", "f32 = 2 registri, big-endian (parola alta per prima)", "f32 = 2 registos, big-endian (palavra alta primeiro)", "f32 = 2 registers, big-endian (hoog woord eerst)", "f32 = 2 rejestry, big-endian (starsze słowo pierwsze)"],
            ColName            => ["Désignation", "Name", "Bezeichnung", "Designación", "Designazione", "Designação", "Naam", "Nazwa"],
            ColTable           => ["Table", "Table", "Tabelle", "Tabla", "Tabella", "Tabela", "Tabel", "Tabela"],
            ColAddr            => ["Adr.", "Addr.", "Adr.", "Dir.", "Ind.", "End.", "Adr.", "Adr."],
            ColValue           => ["Valeur", "Value", "Wert", "Valor", "Valore", "Valor", "Waarde", "Wartość"],
            ColAccess          => ["Accès", "Access", "Zugriff", "Acceso", "Accesso", "Acesso", "Toegang", "Dostęp"],
            // --- Panneau central : supervision + courbe ---
            Measure            => ["Mesure (PV)", "Measurement (PV)", "Messwert (PV)", "Medida (PV)", "Misura (PV)", "Medição (PV)", "Meting (PV)", "Pomiar (PV)"],
            ActiveSetpoint     => ["Consigne active", "Active setpoint", "Aktiver Sollwert", "Consigna activa", "Setpoint attivo", "Setpoint ativo", "Actief setpoint", "Aktywna wartość zadana"],
            Output             => ["Sortie", "Output", "Ausgang", "Salida", "Uscita", "Saída", "Uitgang", "Wyjście"],
            OutputPct          => ["Sortie (%)", "Output (%)", "Ausgang (%)", "Salida (%)", "Uscita (%)", "Saída (%)", "Uitgang (%)", "Wyjście (%)"],
            LegSetpoint        => ["Consigne (SP)", "Setpoint (SP)", "Sollwert (SP)", "Consigna (SP)", "Setpoint (SP)", "Setpoint (SP)", "Setpoint (SP)", "Wartość zadana (SP)"],
            LegMeasure         => ["Mesure (PV)", "Measurement (PV)", "Messwert (PV)", "Medida (PV)", "Misura (PV)", "Medição (PV)", "Meting (PV)", "Pomiar (PV)"],
            LegOutput          => ["Sortie (%)", "Output (%)", "Ausgang (%)", "Salida (%)", "Uscita (%)", "Saída (%)", "Uitgang (%)", "Wyjście (%)"],
            ManualDash         => ["— (manuel)", "— (manual)", "— (manuell)", "— (manual)", "— (manuale)", "— (manual)", "— (handmatig)", "— (ręczny)"],
            AxisTime           => ["temps (s)", "time (s)", "Zeit (s)", "tiempo (s)", "tempo (s)", "tempo (s)", "tijd (s)", "czas (s)"],
            // --- Libellés de mode de régulation ---
            ModeOff            => ["Désactivé", "Disabled", "Deaktiviert", "Desactivado", "Disattivato", "Desativado", "Uitgeschakeld", "Wyłączony"],
            ModePid            => ["PID", "PID", "PID", "PID", "PID", "PID", "PID", "PID"],
            ModeOnOff          => ["Tout-ou-rien (TOR)", "On/off (TOR)", "Zweipunkt (TOR)", "Todo-nada (TOR)", "Tutto-niente (TOR)", "Tudo-ou-nada (TOR)", "Aan/uit (TOR)", "Dwustawny (TOR)"],
            ModePwm            => ["Relais à cycle (PWM)", "Time-proportioning relay (PWM)", "Taktrelais (PWM)", "Relé de ciclo (PWM)", "Relè a ciclo (PWM)", "Relé de ciclo (PWM)", "Cyclusrelais (PWM)", "Przekaźnik cykliczny (PWM)"],
            // --- Libellés de parité ---
            ParityNone         => ["Aucune", "None", "Keine", "Ninguna", "Nessuna", "Nenhuma", "Geen", "Brak"],
            ParityEven         => ["Paire", "Even", "Gerade", "Par", "Pari", "Par", "Even", "Parzysta"],
            ParityOdd          => ["Impaire", "Odd", "Ungerade", "Impar", "Dispari", "Ímpar", "Oneven", "Nieparzysta"],
            // --- Modal Paramètres ---
            SettingsTitle      => ["Paramètres", "Settings", "Einstellungen", "Ajustes", "Impostazioni", "Definições", "Instellingen", "Ustawienia"],
            Language           => ["Langue", "Language", "Sprache", "Idioma", "Lingua", "Idioma", "Taal", "Język"],
            ModbusTransport    => ["Transport Modbus", "Modbus transport", "Modbus-Transport", "Transporte Modbus", "Trasporto Modbus", "Transporte Modbus", "Modbus-transport", "Transport Modbus"],
            BindIp             => ["IP d'écoute", "Listen IP", "Lausch-IP", "IP de escucha", "IP di ascolto", "IP de escuta", "Luister-IP", "IP nasłuchu"],
            Port               => ["Port", "Port", "Port", "Puerto", "Porta", "Porta", "Poort", "Port"],
            AllowedIps         => ["IP autorisées (une par ligne, jokers `*`, vide = toutes) :", "Allowed IPs (one per line, `*` wildcards, empty = all):", "Erlaubte IPs (eine pro Zeile, Joker `*`, leer = alle):", "IP autorizadas (una por línea, comodines `*`, vacío = todas):", "IP autorizzate (una per riga, jolly `*`, vuoto = tutte):", "IP autorizadas (uma por linha, curingas `*`, vazio = todas):", "Toegestane IP's (één per regel, jokertekens `*`, leeg = alle):", "Dozwolone IP (jeden na linię, znaki `*`, puste = wszystkie):"],
            SerialPort         => ["Port série", "Serial port", "Serielle Schnittstelle", "Puerto serie", "Porta seriale", "Porta série", "Seriële poort", "Port szeregowy"],
            Baud               => ["Baud", "Baud", "Baud", "Baudios", "Baud", "Baud", "Baud", "Baud"],
            Parity             => ["Parité", "Parity", "Parität", "Paridad", "Parità", "Paridade", "Pariteit", "Parzystość"],
            DataBits           => ["Bits de données", "Data bits", "Datenbits", "Bits de datos", "Bit di dati", "Bits de dados", "Databits", "Bity danych"],
            StopBits           => ["Bits de stop", "Stop bits", "Stoppbits", "Bits de parada", "Bit di stop", "Bits de paragem", "Stopbits", "Bity stopu"],
            SlaveId            => ["Adresse esclave", "Slave address", "Slave-Adresse", "Dirección esclavo", "Indirizzo slave", "Endereço escravo", "Slave-adres", "Adres slave"],
            RtuPointToPoint    => ["RTU : liaison point-à-point recommandée (réponse quelle que soit l'adresse).", "RTU: point-to-point link recommended (responds to any address).", "RTU: Punkt-zu-Punkt-Verbindung empfohlen (antwortet auf jede Adresse).", "RTU: enlace punto a punto recomendado (responde a cualquier dirección).", "RTU: collegamento punto-punto consigliato (risponde a qualsiasi indirizzo).", "RTU: ligação ponto a ponto recomendada (responde a qualquer endereço).", "RTU: punt-naar-punt-verbinding aanbevolen (antwoordt op elk adres).", "RTU: zalecane połączenie punkt-punkt (odpowiada na każdy adres)."],
            RtuNoFeature       => ["⚠ binaire compilé sans la feature `rtu` : le transport RTU ne démarrera pas.", "⚠ binary built without the `rtu` feature: the RTU transport will not start.", "⚠ Binärdatei ohne Feature `rtu` kompiliert: RTU-Transport startet nicht.", "⚠ binario compilado sin la feature `rtu`: el transporte RTU no arrancará.", "⚠ binario compilato senza la feature `rtu`: il trasporto RTU non si avvierà.", "⚠ binário compilado sem a feature `rtu`: o transporte RTU não arrancará.", "⚠ binary gebouwd zonder de `rtu`-feature: het RTU-transport start niet.", "⚠ binarka skompilowana bez funkcji `rtu`: transport RTU nie wystartuje."],
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
            // --- Noms de lignes de la table Modbus (IHM) ---
            RowRunning         => ["En marche", "Running", "In Betrieb", "En marcha", "In funzione", "Em funcionamento", "In bedrijf", "Pracuje"],
            RowHeatingActive   => ["Chaud actif", "Heating active", "Heizen aktiv", "Calor activo", "Caldo attivo", "Aquecimento ativo", "Verwarmen actief", "Grzanie aktywne"],
            RowCoolingActive   => ["Froid actif", "Cooling active", "Kühlen aktiv", "Frío activo", "Freddo attivo", "Arrefecimento ativo", "Koelen actief", "Chłodzenie aktywne"],
            RowModeSens1       => ["Mode sens 1", "Mode direction 1", "Modus Richtung 1", "Modo sentido 1", "Modalità verso 1", "Modo sentido 1", "Modus richting 1", "Tryb kierunek 1"],
            RowModeSens2       => ["Mode sens 2", "Mode direction 2", "Modus Richtung 2", "Modo sentido 2", "Modalità verso 2", "Modo sentido 2", "Modus richting 2", "Tryb kierunek 2"],
            RowHysteresis      => ["Hystérésis", "Hysteresis", "Hysterese", "Histéresis", "Isteresi", "Histerese", "Hysterese", "Histereza"],
            RowIdent           => ["Identifiant (ASCII)", "Identifier (ASCII)", "Kennung (ASCII)", "Identificador (ASCII)", "Identificatore (ASCII)", "Identificador (ASCII)", "Identificatie (ASCII)", "Identyfikator (ASCII)"],
            Readback           => ["recopie", "readback", "Rückmeldung", "lectura", "lettura", "leitura", "uitlezing", "odczyt"],
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
        // Indices alignés sur l'ordre de déclaration.
        assert_eq!(Lang::Fr.idx(), 0);
        assert_eq!(Lang::Pl.idx(), 7);
    }

    #[test]
    fn every_translation_is_non_empty() {
        // Échantillon de clés couvrant toutes les sections.
        let keys = [
            Msg::Commands,
            Msg::SettingsTitle,
            Msg::ModeOnOff,
            Msg::RowIdent,
            Msg::AxisTime,
            Msg::Dir2,
        ];
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
