//! Ressources de marque embarquées (feature `gui`). Le logo est inclus à la
//! compilation depuis le dossier `pic/` du workspace : le binaire reste autonome.

use eframe::egui;

/// Logo CESAM-Lab (signature « éditeur » dans l'en-tête de l'IHM).
pub const CESAM_LOGO_PNG: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../pic/Logo-CESAM-Couleur-vect.png"));

/// Icône OSNE (agitateur) — icône de fenêtre / barre des tâches. Générée par
/// `pic/osne-logo.gen.py`.
pub const OSNE_ICON_PNG: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../pic/osne-icon.png"));

/// Décode un PNG embarqué en texture egui. `None` en cas d'échec (l'IHM se
/// rabat alors sur le texte seul).
pub fn load_texture(ctx: &egui::Context, name: &str, png: &[u8]) -> Option<egui::TextureHandle> {
    let image = image::load_from_memory(png).ok()?.into_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    let color = egui::ColorImage::from_rgba_unmultiplied(size, image.as_raw());
    Some(ctx.load_texture(name, color, egui::TextureOptions::LINEAR))
}

/// Construit l'icône de fenêtre à partir de l'icône OSNE.
pub fn window_icon() -> Option<egui::IconData> {
    let image = image::load_from_memory(OSNE_ICON_PNG).ok()?.into_rgba8();
    let (width, height) = image.dimensions();
    Some(egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    })
}
