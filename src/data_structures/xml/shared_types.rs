// Copyright © 2025 Denis Morel
//
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version.
//
// This program is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
// details.
//
// You should have received a copy of the GNU General Public License and
// a copy of the GNU General Public License along with this program. If not, see
// <https://www.gnu.org/licenses/>.

use crate::data_structures::{DataStructureError, DataStructureErrorImpl};
use quick_xml::Reader;
use std::{
    fmt::Debug,
    fs::File,
    io::{BufRead, BufReader},
    marker::PhantomData,
    path::{Path, PathBuf},
    sync::Arc,
};

#[derive(Clone, Debug)]
/// Structure to iterate over tags
///
/// `tag_name` is the tag to iter over
/// `external_tag_name` is the tag containung the tag_name (to avoid collision)
pub struct TagManyWithIterator<T>
where
    T: Clone + Debug,
{
    data: Arc<String>,
    position_in_buffer: usize,
    tag_name: String,
    external_tag_name: String,
    phantom_t: PhantomData<T>,
}

pub struct TagManyIter<'a, T>
where
    T: Clone + Debug,
{
    reader: Reader<&'a [u8]>,
    tag_name: String,
    external_tag_name: String,
    phantom: PhantomData<T>,
}

impl<T> TagManyWithIterator<T>
where
    T: Clone + Debug,
{
    pub fn new(
        data: &Arc<String>,
        position_in_buffer: usize,
        tag_name: &str,
        external_tag_name: &str,
    ) -> Self {
        Self {
            data: data.clone(),
            position_in_buffer,
            tag_name: tag_name.to_string(),
            external_tag_name: external_tag_name.to_string(),
            phantom_t: PhantomData,
        }
    }

    pub fn reader(&self) -> Result<Reader<&[u8]>, DataStructureError> {
        let mut reader = Reader::from_str(&self.data);
        reader.stream().consume(self.position_in_buffer);
        Ok(reader)
    }

    pub fn iter(&self) -> Result<TagManyIter<T>, DataStructureError> {
        Ok(TagManyIter {
            reader: self.reader()?,
            tag_name: self.tag_name.clone(),
            external_tag_name: self.external_tag_name.clone(),
            phantom: PhantomData,
        })
    }
}

impl<'a, T> TagManyIter<'a, T>
where
    T: Clone + Debug,
{
    pub fn reader(&mut self) -> &mut Reader<&'a [u8]> {
        &mut self.reader
    }

    pub fn tag_name(&self) -> &str {
        &self.tag_name
    }

    pub fn external_tag_name(&self) -> &str {
        &self.external_tag_name
    }
}

macro_rules! impl_iterator_for_tag_many_iter {
    ($type: ident) => {
        impl<'a> Iterator for TagManyIter<'a, $type> {
            type Item = $type;

            fn next(&mut self) -> Option<Self::Item> {
                let mut buf = Vec::new();
                let tag_name = self.tag_name().to_string();
                let external_tag_name = self.external_tag_name().to_string();
                let mut reader = self.reader();
                loop {
                    match reader.read_event_into(&mut buf) {
                        Err(_) => return None,
                        Ok(Event::Eof) => return None,
                        Ok(Event::End(e)) => {
                            if e == BytesEnd::new(&external_tag_name) {
                                return None;
                            }
                        }
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
                                            Err(e) => {
                                                println!("{:?}", &s);
                                                error!("Error deserialize {} {:#?}", &tag_name, e);
                                                println!(
                                                    "Error deserialize {} {:#?}",
                                                    &tag_name, e
                                                );
                                                None
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        error!(
                                            "Error xml_read_to_end_into_buffer for {}: {:#?}",
                                            &tag_name, e
                                        );
                                        println!(
                                            "Error xml_read_to_end_into_buffer for {}: {:#?}",
                                            &tag_name, e
                                        );
                                        None
                                    }
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
