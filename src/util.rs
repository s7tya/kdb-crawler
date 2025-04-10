use anyhow::Result;
use serde::Serialize;
use std::io::Write;

pub trait WriteJsonExt {
    fn write_json<T: Serialize>(&mut self, value: &T, is_pretty: bool) -> Result<()>;
}

impl<W: Write> WriteJsonExt for W {
    fn write_json<T: Serialize>(&mut self, value: &T, is_pretty: bool) -> Result<()> {
        let json_data = if is_pretty {
            serde_json::to_string_pretty(&value)?
        } else {
            serde_json::to_string(&value)?
        };

        self.write_all(json_data.as_bytes())?;

        Ok(())
    }
}
