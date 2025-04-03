mod kdb;

use anyhow::Result;
use kdb::write_json;
use reqwest::blocking::Client;
use std::path::Path;

fn main() -> Result<()> {
    tracing_subscriber::fmt().init();

    let csv_file_path = Path::new("dist/kdb.csv");
    if !csv_file_path.exists() {
        let client = Client::builder().cookie_store(true).build().unwrap();
        let url = kdb::grant_session(&client)?;
        let url = kdb::search_courses(&client, url)?;
        kdb::download_courses_csv(&client, url, csv_file_path)?;
    }

    let courses = kdb::get_kdb_records_from_csv(csv_file_path)?;
    tracing::info!("all: {} courses", courses.len());

    let output_file_path = Path::new("dist/kdb.json");
    write_json(&courses, output_file_path, true)?;

    let output_min_file_path = Path::new("dist/kdb.min.json");
    write_json(&courses, output_min_file_path, false)?;

    let undergraduate_courses: Vec<_> = courses
        .iter()
        .filter(|course| !course.code.starts_with('0'))
        .cloned()
        .collect();
    tracing::info!("undergrad: {} courses", undergraduate_courses.len());

    let output_file_path = Path::new("dist/kdb_undergrad.json");
    write_json(&undergraduate_courses, output_file_path, true)?;

    let output_min_file_path = Path::new("dist/kdb_undergrad.min.json");
    write_json(&undergraduate_courses, output_min_file_path, false)?;

    let graduate_courses: Vec<_> = courses
        .iter()
        .filter(|course| course.code.starts_with('0'))
        .cloned()
        .collect();
    tracing::info!("grad: {} courses", graduate_courses.len());

    let output_file_path = Path::new("dist/kdb_grad.json");
    write_json(&graduate_courses, output_file_path, true)?;

    let output_min_file_path = Path::new("dist/kdb_grad.min.json");
    write_json(&graduate_courses, output_min_file_path, false)?;

    Ok(())
}
