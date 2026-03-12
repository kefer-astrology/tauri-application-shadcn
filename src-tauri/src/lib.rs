mod commands;
mod storage;
mod workspace;
use commands::default::{read, write};
use commands::storage::{
    compute_aspects, init_storage, query_aspects, query_positions, query_radix_relative,
    query_timestamps, store_positions, store_relation,
};
use commands::workspace::{
    compute_chart, compute_chart_from_data, compute_transit_series, create_chart, create_workspace,
    delete_chart, delete_workspace, get_chart_details, get_workspace_defaults, load_workspace,
    open_folder_dialog, save_workspace, update_chart,
};

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
            compute_aspects,
            query_radix_relative,
            query_timestamps,
            load_workspace,
            save_workspace,
            create_workspace,
            delete_workspace,
            create_chart,
            update_chart,
            delete_chart,
            get_workspace_defaults,
            compute_chart,
            compute_chart_from_data,
            compute_transit_series,
            open_folder_dialog,
            get_chart_details,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
