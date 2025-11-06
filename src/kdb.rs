use anyhow::{Context, Result};
use encoding_rs_io::DecodeReaderBytesBuilder;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::{
    env,
    fs::{self, File},
    io::{BufReader, Read, Write},
    path::Path,
};

const YEAR: i32 = 2025;

fn download_csv<P: AsRef<Path>>(output_file_path: P) -> Result<()> {
    let output_file_path = output_file_path.as_ref();
    if output_file_path.exists() {
        return Err(anyhow::anyhow!("specified file name has already exist"));
    }

    let client = Client::builder()
        .user_agent(concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION"),
        ))
        .build()
        .unwrap();

    let resp = client
        .post(&env::var("KDB_URL")?)
        .header("Accept-Language", "ja,ja-JP;q=0.9,en;q=0.8")
        .form(&[
            ("pageId", "SB0070"),
            ("action", "downloadList"),
            ("hdnFy", format!("{YEAR}").as_str()),
            ("hdnTermCode", ""),
            ("hdnDayCode", ""),
            ("hdnPeriodCode", ""),
            ("hdnAgentName", ""),
            ("hdnOrg", ""),
            ("hdnIsManager", ""),
            ("hdnReq", ""),
            ("hdnFac", ""),
            ("hdnDepth", ""),
            ("hdnChkSyllabi", "false"),
            ("hdnChkAuditor", "false"),
            ("hdnChkExchangeStudent", "false"),
            ("hdnChkConductedInEnglish", "false"),
            ("hdnCourse", ""),
            ("hdnKeywords", ""),
            ("hdnFullname", ""),
            ("hdnDispDay", ""),
            ("hdnDispPeriod", ""),
            ("hdnOrgName", ""),
            ("hdnReqName", ""),
            ("cmbDwldtype", "csv"),
        ])
        .send()?;
    let bytes = resp.bytes()?.to_vec();

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
        .has_headers(false)
        .trim(csv::Trim::All)
        .from_reader(transcoded_reader);

    let mut records: Vec<KdbRecord> = vec![];

    for result in csv_reader.records() {
        let record = result?;
        records.push(KdbRecord {
            code: record.get(0).unwrap().to_string(),
            name: record.get(1).unwrap().to_string(),
            instructional_type: record.get(2).unwrap().to_string(),
            credits: record.get(3).unwrap().to_string(),
            standard_year: record.get(4).unwrap().to_string(),
            module: record.get(5).unwrap().to_string(),
            period: record.get(6).unwrap().to_string(),
            instructors: record.get(7).unwrap().to_string(),
            overview: record.get(8).unwrap().to_string(),
            remarks: record.get(9).unwrap().to_string(),
            updated_at: record.get(17).unwrap().to_string(),
        });
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
