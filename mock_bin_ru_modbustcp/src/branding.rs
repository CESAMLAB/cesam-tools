//! Ressources de marque (logos) embarquées dans le binaire IHM.
//!
//! Les images sont incluses à la compilation (`include_bytes!`) depuis le dossier
//! `pic/` du workspace : le binaire reste autonome (aucun fichier à déployer à
//! côté). Compilé uniquement avec la feature `gui`.
//!
//! - [`ORME_ICON_PNG`] : icône du régulateur (cadran), aussi utilisée comme icône
//!   de fenêtre via [`window_icon`].
//! - [`CESAM_LOGO_PNG`] : logo CESAM-Lab (signature « éditeur » dans l'en-tête).

use eframe::egui;

/// Icône ORME (cadran de régulation) — voir `pic/orme-icon.svg` pour la source.
pub const ORME_ICON_PNG: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../pic/orme-icon.png"));

/// Logo CESAM-Lab (couleur).
pub const CESAM_LOGO_PNG: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../pic/Logo-CESAM-Couleur-vect.png"));

/// Décode un PNG embarqué en texture egui. En cas d'échec (ne devrait pas arriver
/// avec des assets internes), renvoie `None` : l'IHM se contente alors du texte.
pub fn load_texture(ctx: &egui::Context, name: &str, png: &[u8]) -> Option<egui::TextureHandle> {
    let image = image::load_from_memory(png).ok()?.into_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    let color = egui::ColorImage::from_rgba_unmultiplied(size, image.as_raw());
    Some(ctx.load_texture(name, color, egui::TextureOptions::LINEAR))
}

/// Construit l'icône de fenêtre à partir de l'icône ORME.
///
/// Utilisée par `with_icon` (X11, Windows, macOS). ⚠️ Sous **Wayland**, le
/// compositeur ignore cette icône embarquée : l'icône de la barre des tâches y
/// provient du fichier `orme.desktop` associé via l'`app_id` « orme » (voir
/// `packaging/` et la cible `install` de `scripts/`).
pub fn window_icon() -> Option<egui::IconData> {
    let image = image::load_from_memory(ORME_ICON_PNG).ok()?.into_rgba8();
    let (width, height) = image.dimensions();
    Some(egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    })
}
