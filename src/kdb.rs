use anyhow::{Context, Result};
use encoding_rs_io::DecodeReaderBytesBuilder;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{BufReader, Read, Write},
    path::Path,
};

const KDB_URL: &str = "https://kdb.tsukuba.ac.jp";
const YEAR: i32 = 2025;

fn grant_session(client: &Client) -> Result<String> {
    let mut resp = client
        .get(KDB_URL)
        .send()
        .context("failed to grant a session")?;

    let mut body = String::new();
    resp.read_to_string(&mut body)?;
    if body.contains("sys-err-head") {
        panic!("KdB error");
    }

    Ok(resp.url().to_string())
}

fn search_courses(client: &Client, request_url: String) -> Result<String> {
    let mut resp = client
        .post(&request_url)
        .form(&[
            ("nendo", format!("{}", YEAR).as_str()),
            ("freeWord", ""),
            ("_eventId", "searchOpeningCourse"),
        ])
        .send()
        .context("failed to search courses")?;

    let mut body = String::new();
    resp.read_to_string(&mut body)?;
    if body.contains("sys-err-head") {
        panic!("KdB error");
    }

    Ok(resp.url().to_string())
}

fn get_courses_csv(client: &Client, request_url: String) -> Result<Vec<u8>> {
    let resp = client
        .post(&request_url)
        .form(&[
            ("_eventId", "output"),
            ("_outputFormat", "0"),
            ("nendo", &format!("{}", YEAR)),
        ])
        .send()?;

    let body = resp.bytes()?.to_vec();
    Ok(body)
}

fn download_csv<P: AsRef<Path>>(output_file_path: P) -> Result<()> {
    let output_file_path = output_file_path.as_ref();
    if output_file_path.exists() {
        return Err(anyhow::anyhow!("specified file name has already exist"));
    }

    let client = Client::builder()
        .cookie_store(true)
        .user_agent(concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION"),
        ))
        .build()
        .unwrap();
    let url = grant_session(&client)?;
    let url = search_courses(&client, url)?;
    let bytes = get_courses_csv(&client, url)?;

    fs::create_dir_all(
        output_file_path
            .parent()
            .context("failed to fetch parent dir")?,
    )?;

    let mut output_file = File::create(output_file_path)?;
    output_file.write_all(&bytes)?;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KdbRecord {
    #[serde(rename = "科目番号")]
    pub code: String,

    #[serde(rename = "科目名")]
    name: String,

    #[serde(rename = "授業方法")]
    instructional_type: String,

    #[serde(rename = "単位数")]
    credits: String,

    #[serde(rename = "標準履修年次")]
    standard_year: String,

    #[serde(rename = "実施学期")]
    module: String,

    #[serde(rename = "曜時限")]
    period: String,

    #[serde(rename = "教室")]
    classroom: String,

    #[serde(rename = "担当教員")]
    instructors: String,

    #[serde(rename = "授業概要")]
    overview: String,

    #[serde(rename = "備考")]
    remarks: String,

    #[serde(rename = "データ更新日")]
    updated_at: String,
}

fn get_kdb_records_from_csv<R: Read>(reader: R) -> Result<Vec<KdbRecord>> {
    let transcoded_reader = DecodeReaderBytesBuilder::new()
        .encoding(Some(encoding_rs::SHIFT_JIS))
        .build(reader);

    let mut csv_reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(transcoded_reader);

    let mut records: Vec<KdbRecord> = vec![];

    for result in csv_reader.deserialize::<KdbRecord>() {
        let record = result?;
        records.push(record);
    }

    Ok(records)
}

pub fn get_kdb_records_with_cache<P: AsRef<Path>>(path: P) -> Result<Vec<KdbRecord>> {
    let csv_file_path = path.as_ref();
    if csv_file_path.exists() {
        tracing::info!("kdb.csv is exist. download skipped.")
    } else {
        download_csv("dist/kdb.csv")?;
    }

    let csv_file = File::open(csv_file_path)?;
    let csv_reader = BufReader::new(csv_file);

    let courses = get_kdb_records_from_csv(csv_reader)?;

    Ok(courses)
}
