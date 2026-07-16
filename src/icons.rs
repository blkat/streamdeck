use rust_embed::RustEmbed;

use slint::Image;

use std::path::{Path, PathBuf};



#[derive(RustEmbed)]

#[folder = "ui/icons/pack/"]

struct EmbeddedIconPack;



pub struct IconEntry {

    pub id: &'static str,

    pub file: &'static str,

}



/// Catalogue d'icones SVG (embarquees dans l'exe, repli ui/icons en dev).

pub const ICON_CATALOG: &[IconEntry] = &[

    IconEntry {

        id: "note_musique",

        file: "icons/pack/note-de-musique.svg",

    },

    IconEntry {

        id: "musique_alt",

        file: "icons/pack/musique-alt.svg",

    },

    IconEntry {

        id: "musique",

        file: "icons/pack/music.svg",

    },

    IconEntry {

        id: "dossier",

        file: "icons/pack/dossier-ouvert.svg",

    },

    IconEntry {

        id: "cloche",

        file: "icons/pack/cloche.svg",

    },

    IconEntry {

        id: "crayon",

        file: "icons/pack/crayon.svg",

    },

    IconEntry {

        id: "code8",

        file: "icons/pack/code_8.svg",

    },

    IconEntry {

        id: "coding",

        file: "icons/pack/coding-svgrepo-com.svg",

    },

    IconEntry {

        id: "fleche",

        file: "icons/pack/fleche-petite-gauche.svg",

    },

    IconEntry {

        id: "star",

        file: "icons/pack/star.svg",

    },

    IconEntry {

        id: "mic",

        file: "icons/pack/mic.svg",

    },

    IconEntry {

        id: "bolt",

        file: "icons/pack/bolt.svg",

    },

    IconEntry {

        id: "game",

        file: "icons/pack/game.svg",

    },

    IconEntry {

        id: "cam",

        file: "icons/pack/cam.svg",

    },

    IconEntry {

        id: "code",

        file: "icons/pack/code.svg",

    },

    IconEntry {

        id: "bell",

        file: "icons/pack/bell.svg",

    },

    IconEntry {

        id: "aide",

        file: "icons/pack/aide.svg",

    },

];



fn icon_file_name(entry: &IconEntry) -> Option<&str> {

    Path::new(entry.file).file_name()?.to_str()

}



pub fn resolve_icon_path(base: &Path, icon_id: &str) -> Option<PathBuf> {

    let entry = ICON_CATALOG.iter().find(|e| e.id == icon_id)?;

    let path = base.join("ui").join(entry.file);

    if path.exists() {

        return Some(path);

    }

    let alt = Path::new(env!("CARGO_MANIFEST_DIR")).join("ui").join(entry.file);

    if alt.exists() {

        Some(alt)

    } else {

        None

    }

}



pub fn is_catalog_icon_id(id: &str) -> bool {
    let id = id.trim();
    ICON_CATALOG.iter().any(|e| e.id == id)
}

pub fn load_icon(base: &Path, icon_id: &str) -> Option<Image> {

    let entry = ICON_CATALOG.iter().find(|e| e.id == icon_id)?;

    if let Some(name) = icon_file_name(entry) {

        if let Some(file) = EmbeddedIconPack::get(name) {

            if let Ok(img) = Image::load_from_svg_data(file.data.as_ref()) {

                return Some(img);

            }

        }

    }

    let path = resolve_icon_path(base, icon_id)?;

    Image::load_from_path(&path).ok()

}



pub fn slot_display_icon(

    base: &Path,

    appearance: &str,

    image_path: &Option<String>,

) -> Option<Image> {

    if crate::slot_image::is_image_appearance(appearance) {

        return None;

    }

    let id = image_path.as_ref()?.trim();

    if id.is_empty() {

        return None;

    }

    load_icon(base, id)

}


