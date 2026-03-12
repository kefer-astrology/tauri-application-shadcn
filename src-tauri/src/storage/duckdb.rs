use crate::storage::models::{
    AspectData, PositionData, PositionRow, RadixRelativeRow, RelationData,
};
use duckdb::{params, Connection, Result, ToSql};
use std::collections::HashMap;

pub struct DuckDBStorage {
    conn: Connection,
}

impl DuckDBStorage {
    /// Create a new DuckDB storage instance
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        // Drop old partial indexes if they exist (DuckDB doesn't support them)
        // Ignore errors if they don't exist
        let _ = conn.execute("DROP INDEX IF EXISTS idx_positions_declination", []);
        let _ = conn.execute("DROP INDEX IF EXISTS idx_positions_distance", []);
        let _ = conn.execute("DROP INDEX IF EXISTS idx_positions_altitude", []);

        // Initialize schema
        conn.execute_batch(
            r#"
            -- Computed positions table with all physical properties
            CREATE TABLE IF NOT EXISTS computed_positions (
                chart_id TEXT NOT NULL,
                datetime TIMESTAMP NOT NULL,
                object_id TEXT NOT NULL,
                
                -- Ecliptic coordinates (always available)
                -- Longitude is relative to vernal equinox (spring solstice = 0°)
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
                
                -- Radix/relation tracking
                radix_chart_id TEXT,              -- NULL for radix, chart_id for transits
                is_radix BOOLEAN DEFAULT TRUE,    -- TRUE for base charts, FALSE for transits
                
                -- Flags for which columns are populated (for efficient querying)
                has_equatorial BOOLEAN DEFAULT FALSE,
                has_topocentric BOOLEAN DEFAULT FALSE,
                has_physical BOOLEAN DEFAULT FALSE,
                
                PRIMARY KEY (chart_id, datetime, object_id)
            );
            
            -- Add radix columns if they don't exist (migration)
            -- Note: This will fail silently if columns already exist, which is fine
            PRAGMA table_info(computed_positions);
            -- We'll check and add columns in a separate migration step if needed

            -- Indexes for common queries
            CREATE INDEX IF NOT EXISTS idx_positions_chart_datetime 
                ON computed_positions(chart_id, datetime);
            CREATE INDEX IF NOT EXISTS idx_positions_object 
                ON computed_positions(object_id);
            -- Note: DuckDB doesn't support partial indexes (WHERE clauses)
            -- Create regular indexes instead
            CREATE INDEX IF NOT EXISTS idx_positions_declination 
                ON computed_positions(declination);
            CREATE INDEX IF NOT EXISTS idx_positions_distance 
                ON computed_positions(distance);
            CREATE INDEX IF NOT EXISTS idx_positions_altitude 
                ON computed_positions(altitude);

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
            let has_topocentric = pos_data.altitude.is_some() && pos_data.azimuth.is_some();
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

    /// Helper function to map position rows
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
            radix_chart_id: row.get::<_, Option<String>>(15).ok().flatten(),
            is_radix: row
                .get::<_, Option<bool>>(16)
                .unwrap_or(Some(true))
                .unwrap_or(true),
        })
    }

    /// Query positions for a chart in a time range
    /// Supports optional start/end datetime and Parquet file queries
    pub fn query_positions(
        &self,
        chart_id: &str,
        start: Option<&str>,
        end: Option<&str>,
        objects: Option<&[String]>,
        use_parquet: bool,
        parquet_dir: Option<&str>,
    ) -> Result<Vec<PositionRow>> {
        // If using Parquet and parquet_dir is provided, query Parquet files
        if use_parquet {
            if let Some(parquet_dir) = parquet_dir {
                return self.query_positions_parquet(chart_id, start, end, objects, parquet_dir);
            }
        }

        // Build base query with optional datetime filters
        // Cast TIMESTAMP to TEXT for compatibility with Rust bindings
        let mut query = String::from(
            r#"
            SELECT CAST(datetime AS TEXT) AS datetime, object_id, longitude, latitude,
                   declination, right_ascension, distance,
                   altitude, azimuth,
                   apparent_magnitude, phase_angle, elongation, light_time,
                   speed, retrograde, radix_chart_id, is_radix
            FROM computed_positions 
            WHERE chart_id = ?
        "#,
        );

        // Build query with optional filters using match to handle lifetimes
        match (start, end) {
            (Some(start_dt), Some(end_dt)) => {
                query.push_str(" AND datetime >= ? AND datetime <= ? ORDER BY datetime, object_id");
                let mut stmt = self.conn.prepare(&query)?;
                let rows_iter =
                    stmt.query_map(params![chart_id, start_dt, end_dt], Self::map_position_row)?;
                Ok(rows_iter.collect::<Result<Vec<_>, _>>()?)
            }
            (Some(start_dt), None) => {
                query.push_str(" AND datetime >= ? ORDER BY datetime, object_id");
                let mut stmt = self.conn.prepare(&query)?;
                let rows_iter =
                    stmt.query_map(params![chart_id, start_dt], Self::map_position_row)?;
                Ok(rows_iter.collect::<Result<Vec<_>, _>>()?)
            }
            (None, Some(end_dt)) => {
                query.push_str(" AND datetime <= ? ORDER BY datetime, object_id");
                let mut stmt = self.conn.prepare(&query)?;
                let rows_iter =
                    stmt.query_map(params![chart_id, end_dt], Self::map_position_row)?;
                Ok(rows_iter.collect::<Result<Vec<_>, _>>()?)
            }
            (None, None) => {
                query.push_str(" ORDER BY datetime, object_id");
                let mut stmt = self.conn.prepare(&query)?;
                let rows_iter = stmt.query_map(params![chart_id], Self::map_position_row)?;
                Ok(rows_iter.collect::<Result<Vec<_>, _>>()?)
            }
        }
    }

    /// Query distinct timestamps for a chart (fast - no position data)
    pub fn query_timestamps(&self, chart_id: &str) -> Result<Vec<String>> {
        let query = r#"
            SELECT DISTINCT CAST(datetime AS TEXT) AS datetime
            FROM computed_positions 
            WHERE chart_id = ?
            ORDER BY datetime
        "#;

        let mut stmt = self.conn.prepare(query)?;
        let rows = stmt.query_map(params![chart_id], |row| row.get::<_, String>(0))?;

        rows.collect::<Result<Vec<_>, _>>()
    }

    /// Query positions from Parquet files
    fn query_positions_parquet(
        &self,
        chart_id: &str,
        start: Option<&str>,
        end: Option<&str>,
        _objects: Option<&[String]>,
        parquet_dir: &str,
    ) -> Result<Vec<PositionRow>> {
        // DuckDB can query Parquet files directly using glob patterns
        // Escape single quotes in paths
        let escaped_dir = parquet_dir.replace("'", "''");
        let escaped_chart_id = chart_id.replace("'", "''");
        let pattern = format!("{}/{}_*.parquet", escaped_dir, escaped_chart_id);

        // Cast TIMESTAMP to TEXT for compatibility with Rust bindings
        let mut query = String::from(
            r#"
            SELECT CAST(datetime AS TEXT) AS datetime, object_id, longitude, latitude,
                   declination, right_ascension, distance,
                   altitude, azimuth,
                   apparent_magnitude, phase_angle, elongation, light_time,
                   speed, retrograde, radix_chart_id, is_radix
            FROM read_parquet(?)
            WHERE chart_id = ?
        "#,
        );

        // Use match to handle optional parameters with proper lifetimes
        match (start, end) {
            (Some(start_dt), Some(end_dt)) => {
                query.push_str(" AND datetime >= ? AND datetime <= ? ORDER BY datetime, object_id");
                let mut stmt = self.conn.prepare(&query)?;
                let rows_iter = stmt.query_map(
                    params![&pattern, chart_id, start_dt, end_dt],
                    Self::map_position_row,
                )?;
                Ok(rows_iter.collect::<Result<Vec<_>, _>>()?)
            }
            (Some(start_dt), None) => {
                query.push_str(" AND datetime >= ? ORDER BY datetime, object_id");
                let mut stmt = self.conn.prepare(&query)?;
                let rows_iter = stmt.query_map(
                    params![&pattern, chart_id, start_dt],
                    Self::map_position_row,
                )?;
                Ok(rows_iter.collect::<Result<Vec<_>, _>>()?)
            }
            (None, Some(end_dt)) => {
                query.push_str(" AND datetime <= ? ORDER BY datetime, object_id");
                let mut stmt = self.conn.prepare(&query)?;
                let rows_iter =
                    stmt.query_map(params![&pattern, chart_id, end_dt], Self::map_position_row)?;
                Ok(rows_iter.collect::<Result<Vec<_>, _>>()?)
            }
            (None, None) => {
                query.push_str(" ORDER BY datetime, object_id");
                let mut stmt = self.conn.prepare(&query)?;
                let rows_iter =
                    stmt.query_map(params![&pattern, chart_id], Self::map_position_row)?;
                Ok(rows_iter.collect::<Result<Vec<_>, _>>()?)
            }
        }
    }

    /// Compute aspects on-demand from positions
    pub fn compute_aspects(
        &self,
        chart_id: &str,
        datetime: &str,
        aspect_types: &[&str],
        max_orb: f64,
    ) -> Result<Vec<AspectData>> {
        // SQL query to compute aspects from positions (using parameterized query)
        let query = r#"
            WITH positions AS (
                SELECT object_id, longitude
                FROM computed_positions
                WHERE chart_id = ? AND datetime = ?
            )
            SELECT 
                p1.object_id AS from_object,
                p2.object_id AS to_object,
                CASE 
                    WHEN ABS(p1.longitude - p2.longitude) > 180.0 
                    THEN 360.0 - ABS(p1.longitude - p2.longitude)
                    ELSE ABS(p1.longitude - p2.longitude)
                END AS angle
            FROM positions p1
            CROSS JOIN positions p2
            WHERE p1.object_id < p2.object_id
        "#;

        // Define aspect types and their target angles
        let aspect_definitions: Vec<(f64, &str)> = vec![
            (0.0, "conjunction"),
            (60.0, "sextile"),
            (90.0, "square"),
            (120.0, "trine"),
            (180.0, "opposition"),
        ];

        let mut stmt = self.conn.prepare(query)?;
        let rows = stmt.query_map(params![chart_id, datetime], |row| {
            let from_object: String = row.get(0)?;
            let to_object: String = row.get(1)?;
            let angle: f64 = row.get(2)?;

            // Find matching aspect type
            let mut aspects = Vec::new();
            for (target_angle, aspect_name) in &aspect_definitions {
                if aspect_types.contains(aspect_name) {
                    let orb = (angle - target_angle).abs();
                    if orb <= max_orb {
                        aspects.push((aspect_name, orb));
                    }
                }
            }

            Ok((from_object, to_object, angle, aspects))
        })?;

        let mut result = Vec::new();
        for row_result in rows {
            let (from_object, to_object, angle, aspects) = row_result?;
            for (aspect_name, orb) in aspects {
                result.push(AspectData {
                    relation_id: format!("{}_aspects", chart_id),
                    datetime: datetime.to_string(),
                    source_object: from_object.clone(),
                    target_object: to_object.clone(),
                    aspect_type: aspect_name.to_string(),
                    angle,
                    orb,
                    exact_datetime: None,
                });
            }
        }

        Ok(result)
    }

    /// Helper function to map radix-relative rows
    fn map_radix_row(row: &duckdb::Row) -> Result<RadixRelativeRow, duckdb::Error> {
        Ok(RadixRelativeRow {
            datetime: row.get(0)?,
            object_id: row.get(1)?,
            transit_longitude: row.get(2)?,
            radix_longitude: row.get(3)?,
            longitude_diff: row.get(4)?,
            transit_declination: row.get::<_, Option<f64>>(5)?,
            radix_declination: row.get::<_, Option<f64>>(6)?,
            declination_diff: row.get::<_, Option<f64>>(7)?,
            transit_distance: row.get::<_, Option<f64>>(8)?,
            radix_distance: row.get::<_, Option<f64>>(9)?,
            distance_diff: row.get::<_, Option<f64>>(10)?,
        })
    }

    /// Query radix-relative positions (transits vs base chart)
    pub fn query_radix_relative(
        &self,
        transit_chart_id: &str,
        radix_chart_id: &str,
        start: Option<&str>,
        end: Option<&str>,
    ) -> Result<Vec<RadixRelativeRow>> {
        // Cast TIMESTAMP to TEXT for compatibility with Rust bindings
        let base_query = String::from(
            r#"
            SELECT 
                CAST(t.datetime AS TEXT) AS datetime,
                t.object_id,
                t.longitude AS transit_longitude,
                r.longitude AS radix_longitude,
                CASE 
                    WHEN ABS(t.longitude - r.longitude) > 180.0 
                    THEN (t.longitude - r.longitude) - SIGN(t.longitude - r.longitude) * 360.0
                    ELSE t.longitude - r.longitude
                END AS longitude_diff,
                t.declination AS transit_declination,
                r.declination AS radix_declination,
                t.declination - r.declination AS declination_diff,
                t.distance AS transit_distance,
                r.distance AS radix_distance,
                t.distance - r.distance AS distance_diff
            FROM computed_positions t
            JOIN computed_positions r
                ON t.object_id = r.object_id
            WHERE t.chart_id = ?
              AND r.chart_id = ?
              AND r.is_radix = TRUE
              AND r.datetime = (SELECT DISTINCT datetime FROM computed_positions WHERE chart_id = ? AND is_radix = TRUE LIMIT 1)
            "#,
        );

        // Use match to handle optional parameters with proper lifetimes
        match (start, end) {
            (Some(start_dt), Some(end_dt)) => {
                let query = format!(
                    "{} AND t.datetime >= ? AND t.datetime <= ? ORDER BY t.datetime, t.object_id",
                    base_query
                );
                let mut stmt = self.conn.prepare(&query)?;
                let rows_iter = stmt.query_map(
                    params![
                        transit_chart_id,
                        radix_chart_id,
                        radix_chart_id,
                        start_dt,
                        end_dt
                    ],
                    Self::map_radix_row,
                )?;
                Ok(rows_iter.collect::<Result<Vec<_>, _>>()?)
            }
            (Some(start_dt), None) => {
                let query = format!(
                    "{} AND t.datetime >= ? ORDER BY t.datetime, t.object_id",
                    base_query
                );
                let mut stmt = self.conn.prepare(&query)?;
                let rows_iter = stmt.query_map(
                    params![transit_chart_id, radix_chart_id, radix_chart_id, start_dt],
                    Self::map_radix_row,
                )?;
                Ok(rows_iter.collect::<Result<Vec<_>, _>>()?)
            }
            (None, Some(end_dt)) => {
                let query = format!(
                    "{} AND t.datetime <= ? ORDER BY t.datetime, t.object_id",
                    base_query
                );
                let mut stmt = self.conn.prepare(&query)?;
                let rows_iter = stmt.query_map(
                    params![transit_chart_id, radix_chart_id, radix_chart_id, end_dt],
                    Self::map_radix_row,
                )?;
                Ok(rows_iter.collect::<Result<Vec<_>, _>>()?)
            }
            (None, None) => {
                let query = format!("{} ORDER BY t.datetime, t.object_id", base_query);
                let mut stmt = self.conn.prepare(&query)?;
                let rows_iter = stmt.query_map(
                    params![transit_chart_id, radix_chart_id, radix_chart_id],
                    Self::map_radix_row,
                )?;
                Ok(rows_iter.collect::<Result<Vec<_>, _>>()?)
            }
        }
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
        // Cast TIMESTAMP to TEXT for compatibility with Rust bindings
        let base_query = r#"
            SELECT relation_id, CAST(datetime AS TEXT) AS datetime, source_object, target_object, 
                   aspect_type, angle, orb, CAST(exact_datetime AS TEXT) AS exact_datetime
            FROM computed_aspects 
            WHERE relation_id = ? AND datetime >= ? AND datetime <= ?
        "#;

        let query = if let Some(types) = aspect_types {
            if !types.is_empty() {
                let placeholders = types.iter().map(|_| "?").collect::<Vec<_>>().join(",");
                format!(
                    "{} AND aspect_type IN ({}) ORDER BY datetime",
                    base_query, placeholders
                )
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
