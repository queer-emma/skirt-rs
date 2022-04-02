use std::{
    fs::File,
    io::{
        BufReader,
        Read,
    },
    path::Path,
};

use serde::Deserialize;
use zip::ZipArchive;

use crate::{
    error::Error,
    pattern::Template,
};

pub struct Reader {
    zip: ZipArchive<BufReader<File>>,
}

impl Reader {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, Error> {
        let reader = BufReader::new(File::open(path)?);
        let zip = ZipArchive::new(reader)?;

        log::debug!("zip file open");

        Ok(Self { zip })
    }

    fn template_specification(&mut self) -> Result<String, Error> {
        for path in self.zip.file_names() {
            let path = Path::new(path);
            if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                if file_name.ends_with("template_specification.json") {
                    return Ok(file_name.to_owned());
                }
            }
        }

        Err(Error::TemplatesNotFound)
    }

    pub fn template(&mut self) -> Result<Template, Error> {
        let path = self.template_specification()?;
        let file = self.zip.by_name(&path)?;
        Ok(json_deserialize(file)?)
    }
}

fn json_deserialize<T: for<'de> Deserialize<'de>, R: Read>(mut reader: R) -> Result<T, Error> {
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;

    Ok(serde_json::from_str(&buf).inspect_err(|e| {
        log::debug!("{}", e);

        for (i, line) in buf.lines().enumerate() {
            if i + 1 == e.line() {
                log::debug!("{:03} > {}", i + 1, line);
            }
            else if (i as i32 - e.line() as i32).abs() <= 10 {
                log::debug!("{:03}   {}", i + 1, line);
            }
        }
    })?)
}

pub fn from_file(path: impl AsRef<Path>) -> Result<Template, Error> {
    let mut reader = Reader::new(path)?;
    Ok(reader.template()?)
}
