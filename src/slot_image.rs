use crate::paths::AppPaths;
use anyhow::{bail, Context, Result};
use image::imageops::FilterType;
use image::ImageFormat;
use slint::Image;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Taille cible (px) — affichage 72×72, marge pour écrans nets.
pub const SLOT_IMAGE_PX: u32 = 144;

pub fn is_image_appearance(appearance: &str) -> bool {
    appearance == "image"
}

pub fn slot_photo_path(paths: &AppPaths, filename: &str) -> PathBuf {
    paths.image_file(filename)
}

pub fn load_slot_photo(paths: &AppPaths, filename: &str) -> Option<Image> {
    load_slot_photo_at_base(&paths.base, filename)
}

pub fn load_slot_photo_at_base(base: &Path, filename: &str) -> Option<Image> {
    let path = base.join("assets").join("images").join(filename);
    if path.is_file() {
        Image::load_from_path(&path).ok()
    } else {
        None
    }
}

pub fn import_slot_image_from_file(paths: &AppPaths, source: &Path) -> Result<String> {
    let ext = source
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if !matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp") {
        bail!("Format non supporté — utilisez PNG, JPG, GIF ou WebP");
    }
    let bytes = std::fs::read(source).with_context(|| format!("lecture {}", source.display()))?;
    let stem = source
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .filter(|s| !s.trim().is_empty());
    save_slot_image_bytes(paths, &bytes, stem.as_deref())
}

const HTTP_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36";

/// Nettoie une URL collée depuis le presse-papiers (espaces, guillemets typographiques).
pub fn normalize_image_url(url: &str) -> String {
    url.trim()
        .trim_matches(|c: char| {
            c.is_whitespace() || c == '"' || c == '\'' || c == '\u{201c}' || c == '\u{201d}'
        })
        .to_string()
}

/// Ajuste les URL CDN (Unsplash, etc.) pour obtenir JPEG/PNG plutôt qu'AVIF.
pub fn adjust_image_url_for_download(url: &str) -> String {
    let lower = url.to_ascii_lowercase();
    if !lower.contains("unsplash.com") {
        return url.to_string();
    }
    let mut out = url.to_string();
    if !out.contains("fm=") {
        let sep = if out.contains('?') { '&' } else { '?' };
        out.push_str(&format!("{sep}fm=jpg"));
    }
    out
}

pub fn import_slot_image_from_url(
    paths: &AppPaths,
    url: &str,
    custom_name: Option<&str>,
) -> Result<String> {
    let url = normalize_image_url(url);
    if url.is_empty() {
        bail!("URL vide");
    }
    if !url.starts_with("http://") && !url.starts_with("https://") {
        bail!("URL invalide — doit commencer par http:// ou https://");
    }
    let url = adjust_image_url_for_download(&url);
    let bytes = download_image_bytes(&url)?;
    let stem = custom_name
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(sanitize_image_stem)
        .or_else(|| stem_from_url(&url));
    save_slot_image_bytes(paths, &bytes, stem.as_deref())
}

fn download_image_bytes(url: &str) -> Result<Vec<u8>> {
    let lower = url.to_ascii_lowercase();
    let mut req = ureq::get(url)
        .set("User-Agent", HTTP_USER_AGENT)
        // Ne pas demander AVIF en priorité — le crate `image` ne le décode pas.
        .set("Accept", "image/jpeg,image/png,image/gif,image/webp,*/*;q=0.5");
    if lower.contains("unsplash.com") {
        req = req.set("Referer", "https://unsplash.com/");
    }
    let response = req
        .call()
        .map_err(|e| anyhow::anyhow!("téléchargement: {e}"))?;
    let status = response.status();
    if !(200..300).contains(&status) {
        bail!("HTTP {status}");
    }
    let mut bytes = Vec::new();
    std::io::copy(&mut response.into_reader(), &mut bytes).context("lecture réponse HTTP")?;
    if bytes.is_empty() {
        bail!("Réponse vide");
    }
    if bytes.len() > 8 * 1024 * 1024 {
        bail!("Image trop volumineuse (max 8 Mo)");
    }
    if looks_like_html(&bytes) {
        bail!(
            "L'URL renvoie une page web, pas un fichier image. \
             Utilisez le lien direct (clic droit sur l'image → « Copier l'adresse de l'image »)"
        );
    }
    Ok(bytes)
}

fn looks_like_html(bytes: &[u8]) -> bool {
    let head: Vec<u8> = bytes
        .iter()
        .copied()
        .filter(|b| !b.is_ascii_whitespace())
        .take(32)
        .collect();
    head.starts_with(b"<!")
        || head.starts_with(b"<html")
        || head.starts_with(b"<HTML")
        || head.starts_with(b"<?xml")
            && bytes
                .windows(5)
                .any(|w| w.eq_ignore_ascii_case(b"<html"))
}

fn save_slot_image_bytes(
    paths: &AppPaths,
    bytes: &[u8],
    preferred_stem: Option<&str>,
) -> Result<String> {
    let img = image::load_from_memory(bytes).with_context(|| {
        "décodage image impossible — vérifiez que l'URL pointe vers un PNG, JPG, GIF ou WebP"
    })?;
    let resized = img.resize_to_fill(SLOT_IMAGE_PX, SLOT_IMAGE_PX, FilterType::Triangle);
    let name = unique_image_filename(paths, preferred_stem);
    let out = paths.image_file(&name);
    resized
        .write_to(&mut std::fs::File::create(&out)?, ImageFormat::Png)
        .with_context(|| format!("écriture {}", out.display()))?;
    Ok(name)
}

fn sanitize_image_stem(name: &str) -> String {
    let mut out = String::new();
    for c in name.trim().chars() {
        if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ' ' {
            out.push(c);
        } else if c == '.' {
            out.push('_');
        }
    }
    let trimmed = out.trim();
    if trimmed.is_empty() {
        "image".to_string()
    } else {
        trimmed.to_string()
    }
}

fn stem_from_url(url: &str) -> Option<String> {
    let path = url.split('?').next()?.split('#').next()?;
    let file = path.rsplit('/').next()?;
    let stem = file.rsplit_once('.').map(|(s, _)| s).unwrap_or(file);
    let clean = sanitize_image_stem(stem);
    if clean == "image" && stem.is_empty() {
        None
    } else {
        Some(clean)
    }
}

fn unique_image_filename(paths: &AppPaths, preferred_stem: Option<&str>) -> String {
    let base = preferred_stem
        .map(sanitize_image_stem)
        .filter(|s| s != "image")
        .unwrap_or_else(|| format!("image_{}", &Uuid::new_v4().simple().to_string()[..8]));
    let first = format!("{base}.png");
    if !paths.image_file(&first).exists() {
        return first;
    }
    for n in 2..1000 {
        let candidate = format!("{base}_{n}.png");
        if !paths.image_file(&candidate).exists() {
            return candidate;
        }
    }
    format!("{base}_{}.png", Uuid::new_v4().simple())
}

pub fn delete_slot_image_file(paths: &AppPaths, filename: &str) {
    let path = slot_photo_path(paths, filename);
    let _ = std::fs::remove_file(path);
}

/// Fichiers image stockés dans `assets/images/` (triés par nom).
pub fn list_slot_images(paths: &AppPaths) -> Result<Vec<String>> {
    let dir = &paths.images;
    let mut names = Vec::new();
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            let lower = name.to_ascii_lowercase();
            if lower.ends_with(".png")
                || lower.ends_with(".jpg")
                || lower.ends_with(".jpeg")
                || lower.ends_with(".gif")
                || lower.ends_with(".webp")
                || lower.ends_with(".bmp")
            {
                names.push(name.to_string());
            }
        }
    }
    names.sort();
    Ok(names)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn test_paths() -> AppPaths {
        let base = tempfile::tempdir().unwrap().keep();
        let paths = AppPaths {
            base: base.clone(),
            db: base.join("soundboard.db"),
            sounds: base.join("assets").join("sounds"),
            images: base.join("assets").join("images"),
            logs: base.join("logs"),
            temp: base.join("temp"),
        };
        paths.ensure_dirs().unwrap();
        paths
    }

    #[test]
    fn sanitize_stem() {
        assert_eq!(sanitize_image_stem("  Mon Image!  "), "Mon Image");
    }

    #[test]
    fn download_image_url_png() {
        let paths = test_paths();
        let name = import_slot_image_from_url(&paths, "https://placehold.co/64x64.png", None)
            .unwrap();
        assert!(paths.image_file(&name).is_file());
    }

    #[test]
    fn download_unsplash_cdn_url() {
        let paths = test_paths();
        let url = "https://plus.unsplash.com/premium_photo-1666672388644-2d99f3feb9f1?q=80&w=1470&auto=format&fit=crop&ixlib=rb-4.1.0&ixid=M3wxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8fA%3D%3D";
        let name = import_slot_image_from_url(&paths, url, Some("unsplash_test")).unwrap();
        assert!(paths.image_file(&name).is_file());
    }

    #[test]
    fn normalize_strips_quotes() {
        assert_eq!(
            normalize_image_url("  \"https://example.com/a.png\"  "),
            "https://example.com/a.png"
        );
    }

    #[test]
    fn unsplash_url_gets_fm_jpg() {
        let raw = "https://plus.unsplash.com/photo-1?q=80&auto=format";
        let adj = adjust_image_url_for_download(raw);
        assert!(adj.contains("fm=jpg"));
    }
}

pub fn image_library_label(filename: &str) -> String {
    filename
        .rsplit_once('.')
        .map(|(s, _)| s)
        .unwrap_or(filename)
        .to_string()
}
