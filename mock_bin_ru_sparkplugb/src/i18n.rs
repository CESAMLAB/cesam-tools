//! Internationalisation (i18n) de l'IHM — catalogue de chaînes traduites (8 langues).
//!
//! Seules les chaînes **destinées à l'opérateur** sont traduites ; les logs et les
//! acronymes (MQTT, Sparkplug B, PID, %) restent codés en dur. Le compilateur
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
    // Bandeau & statut connexion
    SecurityPlaintext,
    SecurityTls,
    Connected,
    Disconnected,
    // Panneau commandes
    Commands,
    RunStop,
    AutoMode,
    Setpoint,
    ManualOutput,
    PidSettings,
    // Panneau central
    ProcessValue,
    Output,
    LegSetpoint,
    LegPv,
    LegOutput,
    AxisTime,
    // Modal paramètres — broker / Sparkplug B
    SettingsTitle,
    Language,
    Broker,
    BrokerHost,
    BrokerPort,
    ClientId,
    GroupId,
    EdgeNodeId,
    Username,
    Password,
    UseTls,
    Keepalive,
    PublishOnChange,
    PublishPeriod,
    // Modal paramètres — procédé / régulation
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
            AppSubtitle    => ["Régulateur de procédé simulé (Sparkplug B)", "Simulated process regulator (Sparkplug B)", "Simulierter Prozessregler (Sparkplug B)", "Regulador de proceso simulado (Sparkplug B)", "Regolatore di processo simulato (Sparkplug B)", "Regulador de processo simulado (Sparkplug B)", "Gesimuleerde procesregelaar (Sparkplug B)", "Symulowany regulator procesu (Sparkplug B)"],
            SettingsBtn     => ["Paramètres", "Settings", "Einstellungen", "Ajustes", "Impostazioni", "Definições", "Instellingen", "Ustawienia"],
            SaveSettingsBtn => ["Sauvegarder les réglages", "Save settings", "Einstellungen speichern", "Guardar ajustes", "Salva impostazioni", "Guardar definições", "Instellingen opslaan", "Zapisz ustawienia"],
            SettingsSaved   => ["Réglages sauvegardés", "Settings saved", "Einstellungen gespeichert", "Ajustes guardados", "Impostazioni salvate", "Definições guardadas", "Instellingen opgeslagen", "Ustawienia zapisane"],
            SaveFailed      => ["Échec de sauvegarde", "Save failed", "Speichern fehlgeschlagen", "Error al guardar", "Salvataggio non riuscito", "Falha ao guardar", "Opslaan mislukt", "Zapis nie powiódł się"],
            DeviceRunning   => ["EN MARCHE", "RUNNING", "IN BETRIEB", "EN MARCHA", "IN FUNZIONE", "EM FUNCIONAMENTO", "IN BEDRIJF", "PRACUJE"],
            DeviceStopped   => ["À L'ARRÊT", "STOPPED", "GESTOPPT", "DETENIDO", "FERMO", "PARADO", "GESTOPT", "ZATRZYMANY"],
            SecurityPlaintext => ["⚠ MQTT en clair (pas de TLS) — réseau de confiance uniquement", "⚠ Plaintext MQTT (no TLS) — trusted network only", "⚠ MQTT unverschlüsselt (kein TLS) — nur vertrauenswürdiges Netzwerk", "⚠ MQTT sin cifrar (sin TLS) — solo red de confianza", "⚠ MQTT in chiaro (senza TLS) — solo rete attendibile", "⚠ MQTT em claro (sem TLS) — apenas rede de confiança", "⚠ MQTT zonder versleuteling (geen TLS) — alleen vertrouwd netwerk", "⚠ MQTT bez szyfrowania (bez TLS) — tylko zaufana sieć"],
            SecurityTls     => ["🔒 MQTT/TLS", "🔒 MQTT/TLS", "🔒 MQTT/TLS", "🔒 MQTT/TLS", "🔒 MQTT/TLS", "🔒 MQTT/TLS", "🔒 MQTT/TLS", "🔒 MQTT/TLS"],
            Connected       => ["Connecté", "Connected", "Verbunden", "Conectado", "Connesso", "Ligado", "Verbonden", "Połączono"],
            Disconnected    => ["Déconnecté", "Disconnected", "Getrennt", "Desconectado", "Disconnesso", "Desligado", "Verbroken", "Rozłączono"],
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
            Broker          => ["Broker MQTT / Sparkplug B", "MQTT broker / Sparkplug B", "MQTT-Broker / Sparkplug B", "Broker MQTT / Sparkplug B", "Broker MQTT / Sparkplug B", "Broker MQTT / Sparkplug B", "MQTT-broker / Sparkplug B", "Broker MQTT / Sparkplug B"],
            BrokerHost      => ["Hôte du broker", "Broker host", "Broker-Host", "Host del broker", "Host del broker", "Anfitrião do broker", "Broker-host", "Host brokera"],
            BrokerPort      => ["Port du broker", "Broker port", "Broker-Port", "Puerto del broker", "Porta del broker", "Porta do broker", "Broker-poort", "Port brokera"],
            ClientId        => ["Identifiant client", "Client ID", "Client-ID", "ID de cliente", "ID client", "ID de cliente", "Client-ID", "Identyfikator klienta"],
            GroupId         => ["Groupe (group_id)", "Group (group_id)", "Gruppe (group_id)", "Grupo (group_id)", "Gruppo (group_id)", "Grupo (group_id)", "Groep (group_id)", "Grupa (group_id)"],
            EdgeNodeId      => ["Nœud edge (edge_node_id)", "Edge node (edge_node_id)", "Edge-Knoten (edge_node_id)", "Nodo edge (edge_node_id)", "Nodo edge (edge_node_id)", "Nó edge (edge_node_id)", "Edge-node (edge_node_id)", "Węzeł edge (edge_node_id)"],
            Username        => ["Utilisateur", "Username", "Benutzername", "Usuario", "Nome utente", "Utilizador", "Gebruikersnaam", "Nazwa użytkownika"],
            Password        => ["Mot de passe", "Password", "Passwort", "Contraseña", "Password", "Palavra-passe", "Wachtwoord", "Hasło"],
            UseTls          => ["Chiffrement TLS", "TLS encryption", "TLS-Verschlüsselung", "Cifrado TLS", "Crittografia TLS", "Cifragem TLS", "TLS-versleuteling", "Szyfrowanie TLS"],
            Keepalive       => ["Keepalive (s)", "Keepalive (s)", "Keepalive (s)", "Keepalive (s)", "Keepalive (s)", "Keepalive (s)", "Keepalive (s)", "Keepalive (s)"],
            PublishOnChange => ["Publier sur changement", "Publish on change", "Bei Änderung senden", "Publicar al cambiar", "Pubblica al cambiamento", "Publicar ao mudar", "Publiceren bij wijziging", "Publikuj przy zmianie"],
            PublishPeriod   => ["Période de publication (s)", "Publish period (s)", "Sendeintervall (s)", "Periodo de publicación (s)", "Periodo di pubblicazione (s)", "Período de publicação (s)", "Publicatie-interval (s)", "Okres publikacji (s)"],
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
            for key in [Msg::AppSubtitle, Msg::Setpoint, Msg::ProcessValue, Msg::Output, Msg::Broker, Msg::CloseBtn] {
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
