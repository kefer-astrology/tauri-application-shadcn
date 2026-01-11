mod commands;
mod storage;
mod workspace;
use commands::default::{read, write};
use commands::storage::{init_storage, store_positions, query_positions, store_relation, query_aspects};
use commands::workspace::{load_workspace, compute_chart, open_folder_dialog, get_chart_details};

#[allow(clippy::missing_panics_doc)]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            read, 
            write,
            init_storage,
            store_positions,
            query_positions,
            store_relation,
            query_aspects,
            load_workspace,
            compute_chart,
            open_folder_dialog,
            get_chart_details,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
