mod kdb;
mod util;

use anyhow::Result;
use std::{fs::File, path::Path};
use tracing::info;
use tracing_subscriber::EnvFilter;
use util::WriteJsonExt;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let csv_file_path = Path::new("dist/kdb.csv");
    if csv_file_path.exists() {
        info!("kdb.csv is exist. download skipped.")
    } else {
        kdb::download_csv("dist/kdb.csv")?;
    }

    let courses = kdb::get_kdb_records_from_csv(csv_file_path)?;
    let (undergraduate_courses, graduate_courses): (Vec<_>, Vec<_>) = courses
        .iter()
        .partition(|course| course.code.starts_with('0'));

    tracing::info!("all: {} courses", courses.len());
    tracing::info!("undergrad: {} courses", undergraduate_courses.len());
    tracing::info!("grad: {} courses", graduate_courses.len());

    for (name, courses) in &[
        ("kdb", &courses.iter().collect::<Vec<_>>()),
        ("kdb_undergrad", &undergraduate_courses),
        ("kdb_grad", &graduate_courses),
    ] {
        let output_file_path = format!("dist/{}.json", name);
        let mut file = File::create(output_file_path)?;
        file.write_json(courses, true)?;

        let output_file_path = format!("dist/{}.min.json", name);
        let mut file = File::create(output_file_path)?;
        file.write_json(courses, false)?;
    }

    Ok(())
}
