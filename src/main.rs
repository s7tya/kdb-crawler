mod kdb;

use anyhow::Result;
use std::{path::Path, sync::Arc};

fn main() -> Result<()> {
    tracing_subscriber::fmt().init();

    let csv_file_path = Path::new("dist/kdb.csv");
    if !csv_file_path.exists() {
        let client = Arc::new(ureq::Agent::new());
        let url = kdb::grant_session(&client);
        let url = kdb::search_courses(&client, url);

        let _ = kdb::download_courses_csv(&client, url, csv_file_path);
    }

    let output_file_path = Path::new("dist/kdb.json");
    kdb::convert_courses_csv_to_json(csv_file_path, output_file_path, true)?;

    let output_min_file_path = Path::new("dist/kdb.min.json");
    let _ = kdb::convert_courses_csv_to_json(csv_file_path, output_min_file_path, false);

    Ok(())
}
