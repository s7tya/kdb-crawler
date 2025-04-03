use anyhow::{anyhow, Context, Result};
use encoding_rs_io::DecodeReaderBytesBuilder;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{BufReader, Write},
    path::Path,
};
use ureq::{Agent, ResponseExt};

const KDB_URL: &str = "https://kdb.tsukuba.ac.jp";
const YEAR: i32 = 2025;

pub fn grant_session(client: &Agent) -> String {
    let resp = client.get(KDB_URL).call();

    match resp {
        Ok(v) => v.get_uri().to_string(),
        Err(e) => panic!("failed to grant a session: {}", e),
    }
}

pub fn search_courses(client: &Agent, request_url: String) -> String {
    let resp = client.post(&request_url).send_form([
        ("index", ""),
        ("locale", ""),
        ("nendo", &format!("{}", YEAR)),
        ("termCode", ""),
        ("dayCode", ""),
        ("periodCode", ""),
        ("campusCode", ""),
        ("hierarchy1", ""),
        ("hierarchy2", ""),
        ("hierarchy3", ""),
        ("hierarchy4", ""),
        ("hierarchy5", ""),
        ("freeWord", ""),
        ("_orFlg", "1"),
        ("_andFlg", "1"),
        ("_gaiyoFlg", "1"),
        ("_risyuFlg", "1"),
        ("_excludeFukaikoFlg", "1"),
        ("_eventId", "searchOpeningCourse"),
    ]);

    match resp {
        Ok(v) => v.get_uri().to_string(),
        Err(e) => panic!("failed to search courses: {}", e),
    }
}

pub fn download_courses_csv(
    client: &Agent,
    request_url: String,
    output_file_path: &Path,
) -> Result<()> {
    let resp = client.post(&request_url).send_form([
        ("index", ""),
        ("locale", ""),
        ("nendo", &format!("{}", YEAR)),
        ("termCode", ""),
        ("dayCode", ""),
        ("periodCode", ""),
        ("campusCode", ""),
        ("hierarchy1", ""),
        ("hierarchy2", ""),
        ("hierarchy3", ""),
        ("hierarchy4", ""),
        ("hierarchy5", ""),
        ("freeWord", ""),
        ("_orFlg", "1"),
        ("_andFlg", "1"),
        ("_gaiyoFlg", "1"),
        ("_risyuFlg", "1"),
        ("_excludeFukaikoFlg", "1"),
        ("_eventId", "output"),
        ("_outputFormat", "0"),
    ])?;
    let mut body = resp.into_body();
    let mut reader = body.as_reader();

    if output_file_path.exists() {
        return Err(anyhow!("specified file name has already exist"));
    }

    fs::create_dir_all(
        output_file_path
            .parent()
            .context("failed to fetch parent dir")?,
    )?;

    let mut output_file = File::create(output_file_path)?;

    let _ = std::io::copy(&mut reader, &mut output_file)?;

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

pub fn get_kdb_records_from_csv(csv_file_path: &Path) -> Result<Vec<KdbRecord>> {
    let csv_file = File::open(csv_file_path)?;
    let transcoded_reader = DecodeReaderBytesBuilder::new()
        .encoding(Some(encoding_rs::SHIFT_JIS))
        .build(BufReader::new(csv_file));

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

pub fn write_json(
    records: &Vec<KdbRecord>,
    output_file_path: &Path,
    is_pretty: bool,
) -> Result<()> {
    let json_data = if is_pretty {
        serde_json::to_string_pretty(&records)?
    } else {
        serde_json::to_string(&records)?
    };

    let mut output_file = File::create(output_file_path)?;
    output_file.write_all(json_data.as_bytes())?;

    Ok(())
}
