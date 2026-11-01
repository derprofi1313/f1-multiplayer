use std::path::PathBuf;

use swofty_core::baseline::{Identity, exe_version, lang_dir, original_lang};
use swofty_formats::lng::Lng;

#[test]
fn stock_language_files_are_extracted_once_per_build() {
    let Some(game) = std::env::var_os("F1_25_DIR").map(PathBuf::from) else {
        eprintln!("F1_25_DIR is not set; skipping");
        return;
    };
    if !game.join(swofty_core::GAME_EXE).is_file() {
        eprintln!("no {} under F1_25_DIR; skipping", swofty_core::GAME_EXE);
        return;
    }
    let data = std::env::temp_dir().join(format!("swofty-lang-{}", std::process::id()));
    let id = Identity {
        build_id: "test".into(),
        exe_sha256: "0123456789abcdef".into(),
    };
    let eng = original_lang(&data, &game, &id, "eng").unwrap();
    assert_eq!(eng, lang_dir(&data, &id).join("language_eng.lng"));
    let lng = Lng::parse(&std::fs::read(&eng).unwrap()).unwrap();
    assert!(lng.get("lng_1_stop").is_some());
    let count = std::fs::read_dir(lang_dir(&data, &id)).unwrap().count();
    assert!(count >= 11, "{count} language files");
    assert!(original_lang(&data, &game, &id, "xyz").is_err());
    let version = exe_version(&game).unwrap().unwrap();
    assert!(version.starts_with("1.0."), "{version}");
    std::fs::remove_dir_all(&data).ok();
}
