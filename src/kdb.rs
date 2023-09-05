use anyhow::{anyhow, Context, Result};
use encoding_rs_io::DecodeReaderBytesBuilder;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{BufReader, Write},
    path::Path,
};
use ureq::Agent;

const KDB_URL: &str = "https://kdb.tsukuba.ac.jp";
const YEAR: i32 = 2023;

pub fn grant_session(client: &Agent) -> String {
    let res = client.get(KDB_URL).call();

    match res {
        Ok(v) => v.get_url().to_string(),
        Err(e) => panic!("failed to grant a session: {}", e),
    }
}

pub fn search_courses(client: &Agent, request_url: String) -> String {
    let res = client.post(&request_url).send_form(&[
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

    match res {
        Ok(v) => v.get_url().to_string(),
        Err(e) => panic!("failed to search courses: {}", e),
    }
}

pub fn download_csv(client: &Agent, request_url: String, output_file_path: &Path) -> Result<()> {
    let resp = client.post(&request_url).send_form(&[
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

    if output_file_path.exists() {
        return Err(anyhow!("specified file name has already exist"));
    }

    fs::create_dir_all(
        output_file_path
            .parent()
            .context("failed to fetch parent dir")?,
    )?;

    let mut output_file = File::create(output_file_path)?;

    let _ = std::io::copy(&mut resp.into_reader(), &mut output_file)?;

    Ok(())
}

pub fn convert_csv_to_json(
    csv_file_path: &Path,
    output_file_path: &Path,
    is_pretty: bool,
) -> Result<()> {
    #[derive(Debug, Serialize, Deserialize)]
    struct Record {
        #[serde(rename = "科目番号")]
        code: String,

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

    let csv_file = File::open(csv_file_path)?;
    let transcoded_reader = DecodeReaderBytesBuilder::new()
        .encoding(Some(&encoding_rs::SHIFT_JIS))
        .build(BufReader::new(csv_file));

    let mut csv_reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(transcoded_reader);

    let mut output_file = File::create(output_file_path)?;
    let mut records: Vec<Record> = vec![];

    for result in csv_reader.deserialize::<Record>() {
        let record = result?;
        records.push(record);
    }

    let json_data = if is_pretty {
        serde_json::to_string_pretty(&records)?
    } else {
        serde_json::to_string(&records)?
    };

    output_file.write_all(json_data.as_bytes())?;

    Ok(())
}
