use std::{
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
    pub fn from_csv(path: impl AsRef<Path>) -> Result<Self> {
        Self::from_reader(File::open(path)?)
    }

    pub fn from_reader<R: io::Read>(reader: R) -> Result<Self> {
        let records = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(reader)
            .into_records()
            .collect::<csv::Result<Vec<_>>>()?;
        assert!(records.len() > 1 && !records[0].is_empty());

        let mut data = (0..records[0].len())
            .map(|i| Series::new(&records[0][i]))
            .collect::<Vec<_>>();

        for (i, series) in data.iter_mut().enumerate() {
            if records.iter().skip(1).all(|r| f64::from_str(&r[i]).is_ok()) {
                series.data = SeriesData::F64(
                    records
                        .iter()
                        .skip(1)
                        .map(|r| f64::from_str(&r[i]).unwrap())
                        .collect(),
                );
            } else {
                series.data =
                    SeriesData::Str(records.iter().skip(1).map(|r| r[i].into()).collect());
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
            if include_tab { write!(tw, "\t").unwrap(); }
            write!(tw, "{}", match s.data() {
                SeriesData::F64(..) => "f64",
                SeriesData::Str(..) => "string",
            }).unwrap();
        };

        let mut series = self.data.iter();
        write_kind(series.next().unwrap(), false);
        for s in series {
            write_kind(s, true);
        }
        writeln!(tw).unwrap();

        for i in 0..std::cmp::min(10, self.data[0].len()) {
            let mut write_element = |s: &Series, include_tab: bool| {
                if include_tab { write!(tw, "\t").unwrap(); }
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
