//! Vérification de mise à jour logicielle.
//!
//! Interroge l'API GitHub (« dernière release » d'un dépôt) et compare le numéro
//! de version publié à la version courante du binaire. La logique de comparaison
//! ([`is_newer`]) est **synchrone et testable** sans accès réseau ; seul
//! [`check_blocking`] effectue une requête HTTPS (à exécuter hors du thread IHM).
//!
//! La requête est **bornée par un timeout** et toute erreur (hors-ligne, quota
//! GitHub, réponse inattendue) est remontée proprement : la vérification est une
//! commodité, jamais un point de défaillance.

use std::time::Duration;

use serde::Deserialize;

/// Erreur de vérification de mise à jour.
#[derive(Debug, thiserror::Error)]
pub enum UpdateError {
    /// Échec réseau / HTTP (hors-ligne, timeout, TLS, quota dépassé…).
    #[error("update check transport error: {0}")]
    Http(String),
    /// Réponse reçue mais inexploitable (JSON invalide, champ manquant…).
    #[error("unexpected update response: {0}")]
    Response(String),
}

/// Release publiée, telle qu'exposée à l'IHM.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Release {
    /// Numéro de version normalisé (sans le `v` de tête), p. ex. `0.2.0`.
    pub version: String,
    /// URL de la page de release (à ouvrir dans le navigateur).
    pub url: String,
}

/// Résultat d'une vérification réussie.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateStatus {
    /// La version courante est à jour (ou plus récente que la dernière release).
    UpToDate,
    /// Une version plus récente est disponible.
    Available(Release),
}

/// Forme minimale de la réponse `GET /repos/{repo}/releases/latest`.
#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    #[serde(default)]
    html_url: String,
}

/// Découpe une version `MAJEUR.MINEUR.CORRECTIF` en triplet numérique. Tolère un
/// `v`/`V` de tête et un suffixe de pré-version (`-rc1`, `+meta`) qui est ignoré.
/// Les composants manquants valent 0 (`"1.2"` → `(1, 2, 0)`).
fn parse_semver(s: &str) -> Option<(u64, u64, u64)> {
    let s = s.trim();
    let s = s.strip_prefix(['v', 'V']).unwrap_or(s);
    // On ne garde que le cœur numérique (avant un éventuel `-` ou `+`).
    let core = s.split(['-', '+']).next().unwrap_or(s);
    let mut it = core.split('.');
    let major = it.next()?.parse().ok()?;
    let minor = it.next().unwrap_or("0").parse().ok()?;
    let patch = it.next().unwrap_or("0").parse().ok()?;
    Some((major, minor, patch))
}

/// Vrai si `latest` désigne une version **strictement plus récente** que
/// `current`. Si l'une des deux versions est illisible, renvoie `false` (on
/// préfère ne rien signaler plutôt qu'une fausse alerte).
#[must_use]
pub fn is_newer(current: &str, latest: &str) -> bool {
    match (parse_semver(current), parse_semver(latest)) {
        (Some(c), Some(l)) => l > c,
        _ => false,
    }
}

/// Interroge la dernière release GitHub de `repo` (format `proprietaire/depot`)
/// et la compare à `current` (typiquement `env!("CARGO_PKG_VERSION")`).
///
/// **Bloquant** : à appeler depuis un thread dédié, jamais sur le thread IHM.
/// La requête est bornée par `timeout`.
///
/// # Errors
/// Renvoie [`UpdateError`] si la requête échoue ou si la réponse est inexploitable.
pub fn check_blocking(
    repo: &str,
    current: &str,
    timeout: Duration,
) -> Result<UpdateStatus, UpdateError> {
    let url = format!("https://api.github.com/repos/{repo}/releases/latest");
    let resp = match ureq::get(&url)
        // GitHub exige un User-Agent ; `Accept` fige la version d'API.
        .set("User-Agent", "cesam-tools-update-check")
        .set("Accept", "application/vnd.github+json")
        .timeout(timeout)
        .call()
    {
        Ok(resp) => resp,
        // Réponse HTTP non-2xx (quota dépassé, dépôt privé/inexistant…).
        Err(ureq::Error::Status(code, _)) => return Err(UpdateError::Http(format!("HTTP {code}"))),
        // Échec de transport (hors-ligne, DNS, TLS, timeout…).
        Err(e) => return Err(UpdateError::Http(e.to_string())),
    };

    let release: GithubRelease = resp
        .into_json()
        .map_err(|e| UpdateError::Response(e.to_string()))?;

    Ok(classify(current, &release.tag_name, &release.html_url))
}

/// Logique pure de classification (découplée du transport pour les tests).
fn classify(current: &str, tag: &str, url: &str) -> UpdateStatus {
    if is_newer(current, tag) {
        UpdateStatus::Available(Release {
            version: tag
                .trim()
                .strip_prefix(['v', 'V'])
                .unwrap_or(tag.trim())
                .to_string(),
            url: url.to_string(),
        })
    } else {
        UpdateStatus::UpToDate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_with_and_without_v_prefix() {
        assert_eq!(parse_semver("v1.2.3"), Some((1, 2, 3)));
        assert_eq!(parse_semver("1.2.3"), Some((1, 2, 3)));
        assert_eq!(parse_semver("0.1"), Some((0, 1, 0)));
        assert_eq!(parse_semver("2.0.0-rc1"), Some((2, 0, 0)));
        assert_eq!(parse_semver("1.4.0+build7"), Some((1, 4, 0)));
        assert_eq!(parse_semver("nightly"), None);
    }

    #[test]
    fn newer_is_strict_and_component_wise() {
        assert!(is_newer("0.1.0", "0.2.0"));
        assert!(is_newer("0.1.0", "v0.1.1"));
        assert!(is_newer("0.9.9", "1.0.0"));
        assert!(!is_newer("0.2.0", "0.1.0"));
        assert!(!is_newer("1.0.0", "1.0.0"));
        // 0.10 > 0.9 (comparaison numérique, pas lexicographique).
        assert!(is_newer("0.9.0", "0.10.0"));
    }

    #[test]
    fn unparseable_never_alerts() {
        assert!(!is_newer("garbage", "1.0.0"));
        assert!(!is_newer("1.0.0", "garbage"));
    }

    #[test]
    fn classify_strips_v_and_keeps_url() {
        let s = classify("0.1.0", "v0.2.0", "https://example/r");
        assert_eq!(
            s,
            UpdateStatus::Available(Release {
                version: "0.2.0".to_string(),
                url: "https://example/r".to_string(),
            })
        );
        assert_eq!(classify("0.2.0", "v0.1.0", "x"), UpdateStatus::UpToDate);
        assert_eq!(classify("0.1.0", "v0.1.0", "x"), UpdateStatus::UpToDate);
    }
}
