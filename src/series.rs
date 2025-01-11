#[derive(Debug)]
pub enum SeriesData {
    F64(Vec<f64>),
    Str(Vec<String>),
}

impl SeriesData {
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        match self {
            Self::F64(fs) => fs.len(),
            Self::Str(ss) => ss.len(),
        }
    }
}

#[derive(Debug)]
pub struct Series {
    pub(crate) data: SeriesData,
    name: String,
}

impl Series {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            data: SeriesData::F64(Vec::new()),
            name: name.into(),
        }
    }

    pub fn data(&self) -> &SeriesData {
        &self.data
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
