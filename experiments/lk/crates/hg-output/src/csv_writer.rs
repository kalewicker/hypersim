//! Generic CSV writer: exactly one CSV per run, schema-version comment
//! line first, then a serde-derived header from `T`'s field names.

use serde::Serialize;
use std::fs::File;
use std::io::Write;
use std::marker::PhantomData;
use std::path::Path;

pub struct CsvWriter<T: Serialize> {
    inner: csv::Writer<File>,
    _row: PhantomData<T>,
}

impl<T: Serialize> CsvWriter<T> {
    /// Create the CSV and write the schema-version comment line. The `csv`
    /// crate's serde integration derives the header from `T`'s field names
    /// on the first `write_row`.
    pub fn create(
        path: &Path,
        schema_version: u32,
        run_id: &str,
        seed: u64,
        created: &str,
    ) -> std::io::Result<Self> {
        let mut file = File::create(path)?;
        writeln!(
            file,
            "#hypersim_csv_v={schema_version},run_id={run_id},seed={seed},created={created}"
        )?;
        let inner = csv::WriterBuilder::new().has_headers(true).from_writer(file);
        Ok(Self {
            inner,
            _row: PhantomData,
        })
    }

    /// Reopen an existing CSV for appending, without re-emitting the header
    /// or the provenance comment.
    ///
    /// Exists for campaign resume (`docs/campaign_output.md` §7): an
    /// interrupted campaign continues into the *same* `campaign.csv`, so the
    /// "exactly one CSV" invariant survives a crash. Creating a second file
    /// or truncating the first would both break it, in opposite directions.
    ///
    /// Errors if the file does not exist — appending to a missing CSV would
    /// produce one with no header, which every reader would then
    /// misinterpret as a headerless data row.
    pub fn append(path: &Path) -> std::io::Result<Self> {
        let file = std::fs::OpenOptions::new().append(true).open(path)?;
        let inner = csv::WriterBuilder::new().has_headers(false).from_writer(file);
        Ok(Self {
            inner,
            _row: PhantomData,
        })
    }

    pub fn write_row(&mut self, row: &T) -> std::io::Result<()> {
        self.inner.serialize(row).map_err(std::io::Error::other)?;
        Ok(())
    }

    pub fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}
