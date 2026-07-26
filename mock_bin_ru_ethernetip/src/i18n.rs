//! Internationalisation (i18n) de l'IHM — catalogue de chaînes traduites (8 langues).
//!
//! Seules les chaînes **destinées à l'opérateur** sont traduites ; les logs et les
//! acronymes (EtherNet/IP, DB, PID, %) restent codés en dur. Le compilateur garantit qu'aucune
//! clé n'est oubliée (tableau de taille fixe).

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
    NetworkExposed,
    Commands,
    RunStop,
    AutoMode,
    Setpoint,
    ManualOutput,
    PidSettings,
    ProcessValue,
    Output,
    LegSetpoint,
    LegPv,
    LegOutput,
    AxisTime,
    SettingsTitle,
    Language,
    Network,
    BindIp,
    Port,
    Allowlist,
    ProcessParams,
    Gain,
    Tau,
    DeadTime,
    Ambient,
    SpBounds,
    SpMin,
    SpMax,
    ApplyBtn,
    ResetBtn,
    CloseBtn,
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
            AppSubtitle    => ["Régulateur de procédé simulé (EtherNet/IP)", "Simulated process regulator (EtherNet/IP)", "Simulierter Prozessregler (EtherNet/IP)", "Regulador de proceso simulado (EtherNet/IP)", "Regolatore di processo simulato (EtherNet/IP)", "Regulador de processo simulado (EtherNet/IP)", "Gesimuleerde procesregelaar (EtherNet/IP)", "Symulowany regulator procesu (EtherNet/IP)"],
            SettingsBtn     => ["Paramètres", "Settings", "Einstellungen", "Ajustes", "Impostazioni", "Definições", "Instellingen", "Ustawienia"],
            SaveSettingsBtn => ["Sauvegarder les réglages", "Save settings", "Einstellungen speichern", "Guardar ajustes", "Salva impostazioni", "Guardar definições", "Instellingen opslaan", "Zapisz ustawienia"],
            SettingsSaved   => ["Réglages sauvegardés", "Settings saved", "Einstellungen gespeichert", "Ajustes guardados", "Impostazioni salvate", "Definições guardadas", "Instellingen opgeslagen", "Ustawienia zapisane"],
            SaveFailed      => ["Échec de sauvegarde", "Save failed", "Speichern fehlgeschlagen", "Error al guardar", "Salvataggio non riuscito", "Falha ao guardar", "Opslaan mislukt", "Zapis nie powiódł się"],
            DeviceRunning   => ["EN MARCHE", "RUNNING", "IN BETRIEB", "EN MARCHA", "IN FUNZIONE", "EM FUNCIONAMENTO", "IN BEDRIJF", "PRACUJE"],
            DeviceStopped   => ["À L'ARRÊT", "STOPPED", "GESTOPPT", "DETENIDO", "FERMO", "PARADO", "GESTOPT", "ZATRZYMANY"],
            NetworkExposed  => ["⚠ Serveur EtherNet/IP exposé (toutes interfaces, liste blanche vide) — réseau de confiance uniquement", "⚠ EtherNet/IP server exposed (all interfaces, empty allowlist) — trusted network only", "⚠ EtherNet/IP-Server offen (alle Schnittstellen, leere Whitelist) — nur vertrauenswürdiges Netzwerk", "⚠ Servidor EtherNet/IP expuesto (todas las interfaces, lista blanca vacía) — solo red de confianza", "⚠ Server EtherNet/IP esposto (tutte le interfacce, whitelist vuota) — solo rete attendibile", "⚠ Servidor EtherNet/IP exposto (todas as interfaces, lista branca vazia) — apenas rede de confiança", "⚠ EtherNet/IP-server blootgesteld (alle interfaces, lege witte lijst) — alleen vertrouwd netwerk", "⚠ Serwer EtherNet/IP odsłonięty (wszystkie interfejsy, pusta biała lista) — tylko zaufana sieć"],
            Commands        => ["Commandes", "Commands", "Befehle", "Comandos", "Comandi", "Comandos", "Bediening", "Sterowanie"],
            RunStop         => ["Marche / Arrêt", "Run / Stop", "Start / Stopp", "Marcha / Paro", "Marcia / Arresto", "Ligar / Desligar", "Aan / Uit", "Praca / Stop"],
            AutoMode        => ["Mode automatique (PID)", "Automatic mode (PID)", "Automatikmodus (PID)", "Modo automático (PID)", "Modalità automatica (PID)", "Modo automático (PID)", "Automatische modus (PID)", "Tryb automatyczny (PID)"],
            Setpoint        => ["Consigne", "Setpoint", "Sollwert", "Consigna", "Setpoint", "Setpoint", "Setpoint", "Wartość zadana"],
            ManualOutput    => ["Sortie manuelle (%)", "Manual output (%)", "Manuelle Ausgabe (%)", "Salida manual (%)", "Uscita manuale (%)", "Saída manual (%)", "Handmatige uitgang (%)", "Wyjście ręczne (%)"],
            PidSettings     => ["Réglages PID", "PID settings", "PID-Einstellungen", "Ajustes PID", "Parametri PID", "Parâmetros PID", "PID-instellingen", "Nastawy PID"],
            ProcessValue    => ["Mesure", "Process value", "Messwert", "Medida", "Misura", "Medida", "Meetwaarde", "Wartość mierzona"],
            Output          => ["Sortie (%)", "Output (%)", "Ausgabe (%)", "Salida (%)", "Uscita (%)", "Saída (%)", "Uitgang (%)", "Wyjście (%)"],
            LegSetpoint     => ["Consigne", "Setpoint", "Sollwert", "Consigna", "Setpoint", "Setpoint", "Setpoint", "Wartość zadana"],
            LegPv           => ["Mesure", "Process value", "Messwert", "Medida", "Misura", "Medida", "Meetwaarde", "Wartość mierzona"],
            LegOutput       => ["Sortie (%)", "Output (%)", "Ausgabe (%)", "Salida (%)", "Uscita (%)", "Saída (%)", "Uitgang (%)", "Wyjście (%)"],
            AxisTime        => ["temps (s)", "time (s)", "Zeit (s)", "tiempo (s)", "tempo (s)", "tempo (s)", "tijd (s)", "czas (s)"],
            SettingsTitle   => ["Paramètres", "Settings", "Einstellungen", "Ajustes", "Impostazioni", "Definições", "Instellingen", "Ustawienia"],
            Language        => ["Langue", "Language", "Sprache", "Idioma", "Lingua", "Idioma", "Taal", "Język"],
            Network         => ["Réseau EtherNet/IP", "EtherNet/IP network", "EtherNet/IP-Netzwerk", "Red EtherNet/IP", "Rete EtherNet/IP", "Rede EtherNet/IP", "EtherNet/IP-netwerk", "Sieć EtherNet/IP"],
            BindIp          => ["IP d'écoute", "Listen IP", "Lausch-IP", "IP de escucha", "IP di ascolto", "IP de escuta", "Luister-IP", "IP nasłuchu"],
            Port            => ["Port", "Port", "Port", "Puerto", "Porta", "Porta", "Poort", "Port"],
            Allowlist       => ["Liste blanche d'IP (un motif par ligne, * = joker)", "IP allowlist (one pattern per line, * = wildcard)", "IP-Whitelist (ein Muster pro Zeile, * = Platzhalter)", "Lista blanca de IP (un patrón por línea, * = comodín)", "Whitelist IP (un pattern per riga, * = jolly)", "Lista branca de IP (um padrão por linha, * = curinga)", "IP-witte lijst (één patroon per regel, * = jokerteken)", "Biała lista IP (jeden wzorzec na wiersz, * = symbol wieloznaczny)"],
            ProcessParams   => ["Procédé (fonction de transfert)", "Process (transfer function)", "Prozess (Übertragungsfunktion)", "Proceso (función de transferencia)", "Processo (funzione di trasferimento)", "Processo (função de transferência)", "Proces (overdrachtsfunctie)", "Proces (transmitancja)"],
            Gain            => ["Gain statique (K)", "Static gain (K)", "Statische Verstärkung (K)", "Ganancia estática (K)", "Guadagno statico (K)", "Ganho estático (K)", "Statische versterking (K)", "Wzmocnienie statyczne (K)"],
            Tau             => ["Constante de temps τ (s)", "Time constant τ (s)", "Zeitkonstante τ (s)", "Constante de tiempo τ (s)", "Costante di tempo τ (s)", "Constante de tempo τ (s)", "Tijdconstante τ (s)", "Stała czasowa τ (s)"],
            DeadTime        => ["Retard pur (s)", "Dead time (s)", "Totzeit (s)", "Tiempo muerto (s)", "Tempo morto (s)", "Tempo morto (s)", "Dode tijd (s)", "Czas martwy (s)"],
            Ambient         => ["Valeur ambiante", "Ambient value", "Umgebungswert", "Valor ambiente", "Valore ambiente", "Valor ambiente", "Omgevingswaarde", "Wartość otoczenia"],
            SpBounds        => ["Bornes de consigne", "Setpoint bounds", "Sollwertgrenzen", "Límites de consigna", "Limiti setpoint", "Limites de setpoint", "Setpointgrenzen", "Granice wartości zadanej"],
            SpMin           => ["Consigne min", "Setpoint min", "Sollwert min", "Consigna mín", "Setpoint min", "Setpoint mín", "Setpoint min", "Zadana min"],
            SpMax           => ["Consigne max", "Setpoint max", "Sollwert max", "Consigna máx", "Setpoint max", "Setpoint máx", "Setpoint max", "Zadana maks"],
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
            for key in [Msg::AppSubtitle, Msg::Setpoint, Msg::ProcessValue, Msg::Allowlist, Msg::CloseBtn] {
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
