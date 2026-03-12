use crate::storage::duckdb::DuckDBStorage;
use crate::storage::models::{
    AspectData, PositionData, PositionRow, RadixRelativeRow, RelationData,
};
use std::collections::HashMap;
use std::path::PathBuf;

/// Initialize DuckDB storage for a workspace
#[tauri::command]
pub async fn init_storage(workspace_path: String) -> Result<String, String> {
    let workspace_dir = PathBuf::from(&workspace_path);
    let data_dir = workspace_dir.join("data");

    // Create data directory if it doesn't exist
    std::fs::create_dir_all(&data_dir)
        .map_err(|e| format!("Failed to create data directory: {}", e))?;

    let db_path = data_dir.join("workspace.db");
    let db_path_str = db_path.to_str().ok_or("Invalid database path")?;

    // Initialize storage (creates schema)
    DuckDBStorage::new(db_path_str).map_err(|e| format!("Failed to initialize storage: {}", e))?;

    Ok(db_path_str.to_string())
}

/// Store positions for a chart
#[tauri::command]
pub async fn store_positions(
    workspace_path: String,
    chart_id: String,
    datetime: String,
    positions: HashMap<String, PositionData>,
    engine: String,
) -> Result<(), String> {
    let workspace_dir = PathBuf::from(&workspace_path);
    let db_path = workspace_dir.join("data").join("workspace.db");
    let db_path_str = db_path.to_str().ok_or("Invalid database path")?;

    let mut storage =
        DuckDBStorage::new(db_path_str).map_err(|e| format!("Failed to open storage: {}", e))?;

    storage
        .store_positions(&chart_id, &datetime, &positions, &engine)
        .map_err(|e| format!("Failed to store positions: {}", e))?;

    Ok(())
}

/// Query positions for a chart
#[tauri::command]
pub async fn query_positions(
    workspace_path: String,
    chart_id: String,
    start_datetime: Option<String>,
    end_datetime: Option<String>,
    use_parquet: bool,
) -> Result<Vec<PositionRow>, String> {
    let workspace_dir = PathBuf::from(&workspace_path);
    let db_path = workspace_dir.join("data").join("workspace.db");
    let db_path_str = db_path.to_str().ok_or("Invalid database path")?;

    // Store parquet path to ensure it lives long enough
    let parquet_path = if use_parquet {
        Some(workspace_dir.join("data").join("parquet"))
    } else {
        None
    };
    let parquet_dir = parquet_path.as_ref().and_then(|p| p.to_str());

    let storage =
        DuckDBStorage::new(db_path_str).map_err(|e| format!("Failed to open storage: {}", e))?;

    let start_ref = start_datetime.as_deref();
    let end_ref = end_datetime.as_deref();
    storage
        .query_positions(
            &chart_id,
            start_ref,
            end_ref,
            None,
            use_parquet,
            parquet_dir,
        )
        .map_err(|e| format!("Failed to query positions: {}", e))
}

/// Store a relation
#[tauri::command]
pub async fn store_relation(workspace_path: String, relation: RelationData) -> Result<(), String> {
    let workspace_dir = PathBuf::from(&workspace_path);
    let db_path = workspace_dir.join("data").join("workspace.db");
    let db_path_str = db_path.to_str().ok_or("Invalid database path")?;

    let mut storage =
        DuckDBStorage::new(db_path_str).map_err(|e| format!("Failed to open storage: {}", e))?;

    storage
        .store_relation(&relation)
        .map_err(|e| format!("Failed to store relation: {}", e))?;

    Ok(())
}

/// Query aspects for a relation
#[tauri::command]
pub async fn query_aspects(
    workspace_path: String,
    relation_id: String,
    start: String,
    end: String,
    aspect_types: Option<Vec<String>>,
) -> Result<Vec<AspectData>, String> {
    let workspace_dir = PathBuf::from(&workspace_path);
    let db_path = workspace_dir.join("data").join("workspace.db");
    let db_path_str = db_path.to_str().ok_or("Invalid database path")?;

    let storage =
        DuckDBStorage::new(db_path_str).map_err(|e| format!("Failed to open storage: {}", e))?;

    let types_ref = aspect_types.as_deref();
    storage
        .query_aspects(&relation_id, &start, &end, types_ref)
        .map_err(|e| format!("Failed to query aspects: {}", e))
}

/// Compute aspects on-demand from positions
#[tauri::command]
pub async fn compute_aspects(
    workspace_path: String,
    chart_id: String,
    datetime: String,
    aspect_types: Vec<String>,
    max_orb: f64,
) -> Result<Vec<AspectData>, String> {
    let workspace_dir = PathBuf::from(&workspace_path);
    let db_path = workspace_dir.join("data").join("workspace.db");
    let db_path_str = db_path.to_str().ok_or("Invalid database path")?;

    let storage =
        DuckDBStorage::new(db_path_str).map_err(|e| format!("Failed to open storage: {}", e))?;

    let aspect_types_ref: Vec<&str> = aspect_types.iter().map(|s| s.as_str()).collect();
    storage
        .compute_aspects(&chart_id, &datetime, &aspect_types_ref, max_orb)
        .map_err(|e| format!("Failed to compute aspects: {}", e))
}

/// Query distinct timestamps for a chart (fast - optimized for navigation)
#[tauri::command]
pub async fn query_timestamps(
    workspace_path: String,
    chart_id: String,
) -> Result<Vec<String>, String> {
    let workspace_dir = PathBuf::from(&workspace_path);
    let db_path = workspace_dir.join("data").join("workspace.db");
    let db_path_str = db_path.to_str().ok_or("Invalid database path")?;

    let storage =
        DuckDBStorage::new(db_path_str).map_err(|e| format!("Failed to open storage: {}", e))?;

    storage
        .query_timestamps(&chart_id)
        .map_err(|e| format!("Failed to query timestamps: {}", e))
}

/// Query radix-relative positions (transits vs base chart)
#[tauri::command]
pub async fn query_radix_relative(
    workspace_path: String,
    transit_chart_id: String,
    radix_chart_id: String,
    start_datetime: Option<String>,
    end_datetime: Option<String>,
) -> Result<Vec<RadixRelativeRow>, String> {
    let workspace_dir = PathBuf::from(&workspace_path);
    let db_path = workspace_dir.join("data").join("workspace.db");
    let db_path_str = db_path.to_str().ok_or("Invalid database path")?;

    let storage =
        DuckDBStorage::new(db_path_str).map_err(|e| format!("Failed to open storage: {}", e))?;

    let start_ref = start_datetime.as_deref();
    let end_ref = end_datetime.as_deref();
    storage
        .query_radix_relative(&transit_chart_id, &radix_chart_id, start_ref, end_ref)
        .map_err(|e| format!("Failed to query radix-relative positions: {}", e))
}
