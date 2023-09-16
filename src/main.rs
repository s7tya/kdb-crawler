mod kdb;

use anyhow::Result;
use std::path::Path;

fn main() -> Result<()> {
    let csv_file_path = Path::new("dist/kdb.csv");
    if !csv_file_path.exists() {
        let client = ureq::Agent::new();
        let url = kdb::grant_session(&client);
        let url = kdb::search_courses(&client, url);

        let _ = kdb::download_courses_csv(&client, url, &csv_file_path);
    }

    let output_file_path = Path::new("dist/kdb.json");

    let _ = kdb::convert_courses_csv_to_json(csv_file_path, output_file_path, true)?;

    Ok(())
}
