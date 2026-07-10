mod desktop_ui;

use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

use eframe::egui;
use study_rpg::{CharacterClass, DesktopController, SqliteStore};

use crate::desktop_ui::StudyRpgDesktopApp;

fn main() -> Result<(), Box<dyn Error>> {
    std::fs::create_dir_all("data")?;
    let store = SqliteStore::open("data/study_rpg.sqlite3")?;
    let controller = DesktopController::load_or_create(
        store,
        "Player",
        CharacterClass::Scholar,
        current_epoch_seconds(),
    )?;

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1_180.0, 760.0])
            .with_min_inner_size([900.0, 600.0]),
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        "Study RPG",
        native_options,
        Box::new(move |creation_context| {
            Ok(Box::new(StudyRpgDesktopApp::new(
                controller,
                creation_context,
            )))
        }),
    )?;

    Ok(())
}

fn current_epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
