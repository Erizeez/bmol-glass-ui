//! Embeds UI fonts placed in `assets/fonts/` into the crate.
//!
//! Drop SF Pro (or any replacement) `.otf`/`.ttf` files into `assets/fonts/`
//! and they are compiled into the binary; the generated table records each
//! font's family name so `font::ui_fonts()` can reference it by name. When
//! the directory is empty the table is empty and callers fall back to
//! platform fonts.

use std::{env, fmt::Write as _, fs, path::Path};

fn main() {
    let fonts_dir = Path::new("assets/fonts");
    println!("cargo:rerun-if-changed={}", fonts_dir.display());

    let mut fonts = Vec::new();
    if let Ok(entries) = fs::read_dir(fonts_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let is_font =
                path.extension().is_some_and(|ext| ext == "otf" || ext == "ttf" || ext == "ttc");
            if !is_font {
                continue;
            }
            let bytes = fs::read(&path).expect("read font file");
            let family = family_name(&bytes).unwrap_or_else(|| {
                path.file_stem().expect("font file name").to_string_lossy().into_owned()
            });
            fonts.push((
                path.file_name().expect("font file name").to_string_lossy().into_owned(),
                family,
            ));
        }
    }
    fonts.sort();

    let mut generated = String::from(
        "/// (family name, font bytes) for every file found in `assets/fonts/`.\n\
         pub static EMBEDDED_FONTS: &[(&str, &[u8])] = &[\n",
    );
    for (file, family) in &fonts {
        writeln!(
            generated,
            "    ({family:?}, include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/assets/fonts/{file}\"))),"
        )
        .expect("write generated fonts");
    }
    generated.push_str("];\n");

    let out = Path::new(&env::var("OUT_DIR").expect("OUT_DIR")).join("embedded_fonts.rs");
    fs::write(out, generated).expect("write embedded_fonts.rs");
}

/// Reads the family name (nameID 16, falling back to 1) from an sfnt font.
fn family_name(bytes: &[u8]) -> Option<String> {
    let table = find_table(bytes, *b"name")?;
    let count = u16_at(table, 2)? as usize;
    let string_base = u16_at(table, 4)? as usize;

    let mut fallback = None;
    for index in 0..count {
        let record = table.get(6 + index * 12..6 + index * 12 + 12)?;
        let platform = u16::from_be_bytes(record[0..2].try_into().ok()?);
        let name_id = u16::from_be_bytes(record[6..8].try_into().ok()?);
        let length = u16::from_be_bytes(record[8..10].try_into().ok()?) as usize;
        let offset = u16::from_be_bytes(record[10..12].try_into().ok()?) as usize;
        if name_id != 16 && name_id != 1 {
            continue;
        }
        let raw = table.get(string_base + offset..string_base + offset + length)?;
        let name = if platform == 3 || platform == 0 {
            let units: Vec<u16> =
                raw.chunks_exact(2).map(|c| u16::from_be_bytes([c[0], c[1]])).collect();
            String::from_utf16_lossy(&units)
        } else {
            raw.iter().map(|&b| char::from(b)).collect()
        };
        if name_id == 16 {
            return Some(name);
        }
        fallback.get_or_insert(name);
    }
    fallback
}

/// Returns the bytes of an sfnt table directory entry.
fn find_table(bytes: &[u8], tag: [u8; 4]) -> Option<&[u8]> {
    // A TTC wraps several fonts; the first face's directory is enough for a
    // family name. Table offsets stay absolute from the start of the file in
    // both plain sfnt and TTC.
    let directory = if bytes.starts_with(b"ttcf") {
        let first_offset = u32_at(bytes, 12)? as usize;
        bytes.get(first_offset..)?
    } else {
        bytes
    };
    let num_tables = u16_at(directory, 4)? as usize;
    for index in 0..num_tables {
        let record = directory.get(12 + index * 16..12 + index * 16 + 16)?;
        if record[0..4] == tag[..] {
            let offset = u32::from_be_bytes(record[8..12].try_into().ok()?) as usize;
            let length = u32::from_be_bytes(record[12..16].try_into().ok()?) as usize;
            return bytes.get(offset..offset + length);
        }
    }
    None
}

fn u16_at(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_be_bytes(bytes.get(offset..offset + 2)?.try_into().ok()?))
}

fn u32_at(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_be_bytes(bytes.get(offset..offset + 4)?.try_into().ok()?))
}
