use std::{
    ffi::OsStr,
    fmt,
    fs::File,
    io::{self, Write},
    path::Path,
    str::FromStr,
};

use crate::{
    series::{Series, SeriesData},
    Result,
};

pub struct Frame {
    data: Vec<Series>,
}

impl Frame {
    pub fn from_csv(path: impl AsRef<Path>, has_headers: bool) -> Result<Self> {
        let path = path.as_ref();
        let f = File::open(path)?;

        if path
            .extension()
            .and_then(OsStr::to_str)
            .is_some_and(|x| x == "gz")
        {
            Self::from_reader(flate2::read::GzDecoder::new(f), has_headers)
        } else {
            Self::from_reader(f, has_headers)
        }
    }

    pub fn from_reader<R: io::Read>(reader: R, has_headers: bool) -> Result<Self> {
        let records = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(reader)
            .into_records()
            .collect::<csv::Result<Vec<_>>>()?;

        if has_headers {
            assert!(records.len() > 1);
        } else {
            assert!(!records.is_empty());
        }
        assert!(!records[0].is_empty());

        let mut data = (0..records[0].len())
            .map(|i| Series::new(if has_headers { &records[0][i] } else { "" }))
            .collect::<Vec<_>>();

        for (i, series) in data.iter_mut().enumerate() {
            if records
                .iter()
                .skip(if has_headers { 1 } else { 0 })
                .all(|r| f64::from_str(&r[i]).is_ok())
            {
                series.data = SeriesData::F64(
                    records
                        .iter()
                        .skip(if has_headers { 1 } else { 0 })
                        .map(|r| f64::from_str(&r[i]).unwrap())
                        .collect(),
                );
            } else {
                series.data = SeriesData::Str(
                    records
                        .iter()
                        .skip(if has_headers { 1 } else { 0 })
                        .map(|r| r[i].into())
                        .collect(),
                );
            }
        }

        Ok(Frame { data })
    }

    pub fn drop_column(&mut self, name: impl AsRef<str>) -> &mut Self {
        let name = name.as_ref();
        if let Some(i) = self.data.iter().position(|s| s.name() == name) {
            self.data.remove(i);
        }
        self
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.data[0].len()
    }

    pub fn map_parse(
        &mut self,
        name: impl AsRef<str>,
        mut f: impl FnMut(&str) -> f64,
    ) -> &mut Self {
        let name = name.as_ref();
        for series in self.data.iter_mut() {
            if series.name() == name {
                if let SeriesData::Str(old) = &series.data {
                    series.data = SeriesData::F64(old.iter().map(|s| f(s)).collect());
                }
                break;
            }
        }
        self
    }

    pub fn num_columns(&self) -> usize {
        self.data.len()
    }
}

impl fmt::Display for Frame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut tw = tabwriter::TabWriter::new(vec![]);

        let mut series = self.data.iter();
        write!(tw, "{}", series.next().unwrap().name()).unwrap();
        for s in series {
            write!(tw, "\t{}", s.name()).unwrap();
        }
        writeln!(tw).unwrap();

        let mut write_kind = |s: &Series, include_tab: bool| {
            if include_tab {
                write!(tw, "\t").unwrap();
            }
            write!(
                tw,
                "{}",
                match s.data() {
                    SeriesData::F64(..) => "f64",
                    SeriesData::Str(..) => "string",
                }
            )
            .unwrap();
        };

        let mut series = self.data.iter();
        write_kind(series.next().unwrap(), false);
        for s in series {
            write_kind(s, true);
        }
        writeln!(tw).unwrap();

        for i in 0..std::cmp::min(10, self.data[0].len()) {
            let mut write_element = |s: &Series, include_tab: bool| {
                if include_tab {
                    write!(tw, "\t").unwrap();
                }
                match s.data() {
                    SeriesData::F64(fs) => write!(tw, "{}", fs[i]).unwrap(),
                    SeriesData::Str(ss) => write!(tw, "{}", ss[i]).unwrap(),
                }
            };
            let mut series = self.data.iter();
            write_element(series.next().unwrap(), false);
            for s in series {
                write_element(s, true);
            }
            writeln!(tw).unwrap();
        }

        tw.flush().unwrap();

        write!(
            f,
            "{}",
            String::from_utf8(tw.into_inner().unwrap())
                .unwrap()
                .trim_end()
        )
    }
}
