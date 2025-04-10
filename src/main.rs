mod kdb;
mod util;

use anyhow::Result;
use kdb::get_kdb_records_with_cache;
use std::fs::File;
use tracing_subscriber::EnvFilter;
use util::WriteJsonExt;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let courses = get_kdb_records_with_cache("dist/kdb.csv")?;
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
