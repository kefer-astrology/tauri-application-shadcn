use crate::storage::models::{PositionData, PositionRow, AspectData, RelationData};
use duckdb::{Connection, Result, params, ToSql};
use std::collections::HashMap;

pub struct DuckDBStorage {
    conn: Connection,
}

impl DuckDBStorage {
    /// Create a new DuckDB storage instance
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        // Initialize schema
        conn.execute_batch(
            r#"
            -- Computed positions table with all physical properties
            CREATE TABLE IF NOT EXISTS computed_positions (
                chart_id TEXT NOT NULL,
                datetime TIMESTAMP NOT NULL,
                object_id TEXT NOT NULL,
                
                -- Ecliptic coordinates (always available)
                longitude REAL NOT NULL,
                latitude REAL,
                
                -- Equatorial coordinates (JPL/Skyfield - always computed when engine=JPL)
                declination REAL,
                right_ascension REAL,
                distance REAL,
                
                -- Topocentric coordinates (JPL/Skyfield - computed when location available)
                altitude REAL,
                azimuth REAL,
                
                -- Physical properties (JPL/Skyfield)
                apparent_magnitude REAL,
                phase_angle REAL,
                elongation REAL,
                light_time REAL,
                
                -- Motion properties
                speed REAL,
                retrograde BOOLEAN,
                
                -- Engine metadata
                engine TEXT,
                ephemeris_file TEXT,
                
                -- Flags for which columns are populated (for efficient querying)
                has_equatorial BOOLEAN DEFAULT FALSE,
                has_topocentric BOOLEAN DEFAULT FALSE,
                has_physical BOOLEAN DEFAULT FALSE,
                
                PRIMARY KEY (chart_id, datetime, object_id)
            );

            -- Indexes for common queries
            CREATE INDEX IF NOT EXISTS idx_positions_chart_datetime 
                ON computed_positions(chart_id, datetime);
            CREATE INDEX IF NOT EXISTS idx_positions_object 
                ON computed_positions(object_id);
            CREATE INDEX IF NOT EXISTS idx_positions_declination 
                ON computed_positions(declination) WHERE has_equatorial = TRUE;
            CREATE INDEX IF NOT EXISTS idx_positions_distance 
                ON computed_positions(distance) WHERE distance IS NOT NULL;
            CREATE INDEX IF NOT EXISTS idx_positions_altitude 
                ON computed_positions(altitude) WHERE has_topocentric = TRUE;

            -- Computed aspects table
            CREATE TABLE IF NOT EXISTS computed_aspects (
                relation_id TEXT NOT NULL,
                datetime TIMESTAMP NOT NULL,
                source_object TEXT NOT NULL,
                target_object TEXT NOT NULL,
                aspect_type TEXT NOT NULL,
                angle REAL NOT NULL,
                orb REAL NOT NULL,
                exact_datetime TIMESTAMP,
                PRIMARY KEY (relation_id, datetime, source_object, target_object, aspect_type)
            );

            CREATE INDEX IF NOT EXISTS idx_aspects_relation_datetime 
                ON computed_aspects(relation_id, datetime);
            CREATE INDEX IF NOT EXISTS idx_aspects_type 
                ON computed_aspects(aspect_type);

            -- Relations metadata table
            CREATE TABLE IF NOT EXISTS relations (
                relation_id TEXT PRIMARY KEY,
                relation_type TEXT NOT NULL,
                source_chart_id TEXT NOT NULL,
                target_chart_id TEXT,
                third_chart_id TEXT,
                method TEXT,
                time_span_start TIMESTAMP,
                time_span_end TIMESTAMP,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                source_config TEXT,
                target_config TEXT,
                included_objects TEXT,
                included_aspects TEXT,
                engine TEXT,
                ephemeris_file TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_relations_type 
                ON relations(relation_type);
            CREATE INDEX IF NOT EXISTS idx_relations_source 
                ON relations(source_chart_id);
            CREATE INDEX IF NOT EXISTS idx_relations_target 
                ON relations(target_chart_id);

            -- Computation jobs tracking
            CREATE TABLE IF NOT EXISTS computation_jobs (
                job_id TEXT PRIMARY KEY,
                relation_id TEXT NOT NULL,
                status TEXT NOT NULL,
                progress REAL DEFAULT 0.0,
                started_at TIMESTAMP,
                completed_at TIMESTAMP,
                error_message TEXT,
                FOREIGN KEY (relation_id) REFERENCES relations(relation_id)
            );
            "#,
        )?;

        Ok(Self { conn })
    }

    /// Store positions for a chart at a specific datetime
    pub fn store_positions(
        &mut self,
        chart_id: &str,
        datetime: &str,
        positions: &HashMap<String, PositionData>,
        engine: &str,
    ) -> Result<()> {
        // Use a transaction for better performance
        let tx = self.conn.transaction()?;

        let mut stmt = tx.prepare(
            r#"
            INSERT OR REPLACE INTO computed_positions 
            (chart_id, datetime, object_id, longitude, latitude, 
             declination, right_ascension, distance,
             altitude, azimuth,
             apparent_magnitude, phase_angle, elongation, light_time,
             speed, retrograde,
             has_equatorial, has_topocentric, has_physical, engine) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )?;

        for (object_id, pos_data) in positions {
            let has_equatorial = pos_data.declination.is_some()
                && pos_data.right_ascension.is_some()
                && pos_data.distance.is_some();
            let has_topocentric = pos_data.altitude.is_some()
                && pos_data.azimuth.is_some();
            let has_physical = pos_data.apparent_magnitude.is_some()
                || pos_data.phase_angle.is_some()
                || pos_data.elongation.is_some();

            stmt.execute(params![
                chart_id,
                datetime,
                object_id,
                pos_data.longitude,
                pos_data.latitude,
                pos_data.declination,
                pos_data.right_ascension,
                pos_data.distance,
                pos_data.altitude,
                pos_data.azimuth,
                pos_data.apparent_magnitude,
                pos_data.phase_angle,
                pos_data.elongation,
                pos_data.light_time,
                pos_data.speed,
                pos_data.retrograde,
                has_equatorial,
                has_topocentric,
                has_physical,
                engine,
            ])?;
        }

        drop(stmt);
        tx.commit()?;
        Ok(())
    }

    /// Query positions for a chart in a time range
    pub fn query_positions(
        &self,
        chart_id: &str,
        start: &str,
        end: &str,
        objects: Option<&[String]>,
    ) -> Result<Vec<PositionRow>> {
        let base_query = r#"
            SELECT datetime, object_id, longitude, latitude,
                   declination, right_ascension, distance,
                   altitude, azimuth,
                   apparent_magnitude, phase_angle, elongation, light_time,
                   speed, retrograde
            FROM computed_positions 
            WHERE chart_id = ? AND datetime >= ? AND datetime <= ?
        "#;

        let query = if let Some(obj_list) = objects {
            if !obj_list.is_empty() {
                let placeholders = obj_list.iter().map(|_| "?").collect::<Vec<_>>().join(",");
                format!("{} AND object_id IN ({}) ORDER BY datetime, object_id", base_query, placeholders)
            } else {
                format!("{} ORDER BY datetime, object_id", base_query)
            }
        } else {
            format!("{} ORDER BY datetime, object_id", base_query)
        };

        let mut stmt = self.conn.prepare(&query)?;

        // Helper function to map rows
        fn map_position_row(row: &duckdb::Row) -> Result<PositionRow, duckdb::Error> {
            Ok(PositionRow {
                datetime: row.get::<_, String>(0)?,
                object_id: row.get::<_, String>(1)?,
                data: PositionData {
                    longitude: row.get(2)?,
                    latitude: row.get::<_, Option<f64>>(3)?,
                    declination: row.get::<_, Option<f64>>(4)?,
                    right_ascension: row.get::<_, Option<f64>>(5)?,
                    distance: row.get::<_, Option<f64>>(6)?,
                    altitude: row.get::<_, Option<f64>>(7)?,
                    azimuth: row.get::<_, Option<f64>>(8)?,
                    apparent_magnitude: row.get::<_, Option<f64>>(9)?,
                    phase_angle: row.get::<_, Option<f64>>(10)?,
                    elongation: row.get::<_, Option<f64>>(11)?,
                    light_time: row.get::<_, Option<f64>>(12)?,
                    speed: row.get::<_, Option<f64>>(13)?,
                    retrograde: row.get::<_, Option<bool>>(14)?,
                },
            })
        }

        let mut rows = Vec::new();
        let rows_iter = if let Some(obj_list) = objects {
            if !obj_list.is_empty() {
                // Build parameter vector for dynamic IN clause
                let mut param_vec: Vec<&dyn ToSql> = vec![&chart_id, &start, &end];
                for obj in obj_list {
                    param_vec.push(obj);
                }
                stmt.query_map(param_vec.as_slice(), map_position_row)?
            } else {
                stmt.query_map(params![chart_id, start, end], map_position_row)?
            }
        } else {
            stmt.query_map(params![chart_id, start, end], map_position_row)?
        };

        for row_result in rows_iter {
            rows.push(row_result?);
        }

        Ok(rows)
    }

    /// Store a relation
    pub fn store_relation(&mut self, relation: &RelationData) -> Result<()> {
        let mut stmt = self.conn.prepare(
            r#"
            INSERT OR REPLACE INTO relations 
            (relation_id, relation_type, source_chart_id, target_chart_id, third_chart_id,
             method, time_span_start, time_span_end,
             source_config, target_config, included_objects, included_aspects,
             engine, ephemeris_file)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )?;

        let included_objects_str = relation
            .included_objects
            .as_ref()
            .map(|v| serde_json::to_string(v).unwrap_or_default())
            .unwrap_or_default();
        let included_aspects_str = relation
            .included_aspects
            .as_ref()
            .map(|v| serde_json::to_string(v).unwrap_or_default())
            .unwrap_or_default();

        stmt.execute(params![
            relation.relation_id,
            relation.relation_type,
            relation.source_chart_id,
            relation.target_chart_id.as_deref().unwrap_or(""),
            relation.third_chart_id.as_deref().unwrap_or(""),
            relation.method.as_deref().unwrap_or(""),
            relation.time_span_start.as_deref().unwrap_or(""),
            relation.time_span_end.as_deref().unwrap_or(""),
            relation.source_config.as_deref().unwrap_or(""),
            relation.target_config.as_deref().unwrap_or(""),
            included_objects_str,
            included_aspects_str,
            relation.engine.as_deref().unwrap_or(""),
            relation.ephemeris_file.as_deref().unwrap_or(""),
        ])?;

        Ok(())
    }

    /// Query aspects for a relation
    pub fn query_aspects(
        &self,
        relation_id: &str,
        start: &str,
        end: &str,
        aspect_types: Option<&[String]>,
    ) -> Result<Vec<AspectData>> {
        let base_query = r#"
            SELECT relation_id, datetime, source_object, target_object, 
                   aspect_type, angle, orb, exact_datetime
            FROM computed_aspects 
            WHERE relation_id = ? AND datetime >= ? AND datetime <= ?
        "#;

        let query = if let Some(types) = aspect_types {
            if !types.is_empty() {
                let placeholders = types.iter().map(|_| "?").collect::<Vec<_>>().join(",");
                format!("{} AND aspect_type IN ({}) ORDER BY datetime", base_query, placeholders)
            } else {
                format!("{} ORDER BY datetime", base_query)
            }
        } else {
            format!("{} ORDER BY datetime", base_query)
        };

        let mut stmt = self.conn.prepare(&query)?;

        // Helper function to map aspect rows
        fn map_aspect_row(row: &duckdb::Row) -> Result<AspectData, duckdb::Error> {
            Ok(AspectData {
                relation_id: row.get(0)?,
                datetime: row.get(1)?,
                source_object: row.get(2)?,
                target_object: row.get(3)?,
                aspect_type: row.get(4)?,
                angle: row.get(5)?,
                orb: row.get(6)?,
                exact_datetime: row.get::<_, Option<String>>(7)?,
            })
        }

        let mut rows = Vec::new();
        let rows_iter = if let Some(types) = aspect_types {
            if !types.is_empty() {
                // Build parameter vector for dynamic IN clause
                let mut param_vec: Vec<&dyn ToSql> = vec![&relation_id, &start, &end];
                for t in types {
                    param_vec.push(t);
                }
                stmt.query_map(param_vec.as_slice(), map_aspect_row)?
            } else {
                stmt.query_map(params![relation_id, start, end], map_aspect_row)?
            }
        } else {
            stmt.query_map(params![relation_id, start, end], map_aspect_row)?
        };

        for row_result in rows_iter {
            rows.push(row_result?);
        }

        Ok(rows)
    }
}
