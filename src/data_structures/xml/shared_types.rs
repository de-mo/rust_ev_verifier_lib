use quick_xml::{
    events::{BytesStart, Event},
    reader, Reader,
};
use std::{
    fmt::Debug,
    fs::File,
    io::{BufRead, BufReader},
    marker::PhantomData,
    path::{Path, PathBuf},
};

use crate::data_structures::DataStructureError;

use super::xml_read_to_end_into_buffer;

#[derive(Clone, Debug)]
pub struct TagManyWithIterator<T>
where
    T: Clone + Debug,
{
    file_path: PathBuf,
    position_in_buffer: usize,
    tag_name: String,
    phantom_t: PhantomData<T>,
}

pub struct TagManyIter<T>
where
    T: Clone + Debug,
{
    reader: Reader<BufReader<File>>,
    tag_name: String,
    phantom: PhantomData<T>,
}

impl<T> TagManyWithIterator<T>
where
    T: Clone + Debug,
{
    pub fn new(path: &Path, position_in_buffer: usize, tag_name: &str) -> Self {
        Self {
            file_path: path.to_path_buf(),
            position_in_buffer,
            tag_name: tag_name.to_string(),
            phantom_t: PhantomData,
        }
    }

    pub fn reader(&self) -> Result<Reader<BufReader<File>>, DataStructureError> {
        let mut reader =
            Reader::from_file(&self.file_path).map_err(|e| DataStructureError::ParseQuickXML {
                msg: format!(
                    "Error creating xml reader for file {}",
                    self.file_path.to_str().unwrap()
                ),
                source: e,
            })?;
        reader.stream().consume(self.position_in_buffer);
        Ok(reader)
    }

    pub fn iter(&self) -> Result<TagManyIter<T>, DataStructureError> {
        Ok(TagManyIter {
            reader: self.reader()?,
            tag_name: self.tag_name.clone(),
            phantom: PhantomData,
        })
    }
}

impl<T> TagManyIter<T>
where
    T: Clone + Debug,
{
    pub fn reader(&mut self) -> &mut Reader<BufReader<File>> {
        &mut self.reader
    }

    pub fn tag_name(&self) -> &str {
        &self.tag_name
    }
}

macro_rules! impl_iterator_for_tag_many_iter {
    ($type: ident) => {
        impl Iterator for TagManyIter<$type> {
            type Item = $type;

            fn next(&mut self) -> Option<Self::Item> {
                let mut buf = Vec::new();
                let tag_name = self.tag_name().to_string();
                let mut reader = self.reader();
                loop {
                    match reader.read_event_into(&mut buf) {
                        Err(e) => return None,
                        Ok(Event::Eof) => return None,
                        Ok(Event::Start(e)) => {
                            if e == BytesStart::new(&tag_name) {
                                let mut buffer = vec![];
                                return match xml_read_to_end_into_buffer(
                                    &mut reader,
                                    &tag_name,
                                    &mut buffer,
                                ) {
                                    Ok(_) => {
                                        let s = String::from_utf8_lossy(&buffer);
                                        match quick_xml::de::from_str::<$type>(&s) {
                                            Ok(v) => Some(v),
                                            Err(_) => None,
                                        }
                                    }
                                    Err(_) => None,
                                };
                            }
                        }
                        // There are several other `Event`s we do not consider here
                        _ => (),
                    }
                    // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
                    buf.clear();
                }
            }
        }
    };
}
pub(crate) use impl_iterator_for_tag_many_iter;
