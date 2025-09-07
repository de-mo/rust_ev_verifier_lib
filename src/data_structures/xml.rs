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

//! Module to manage the schemas used for the verifier
use roxmltree::{Children, Node};
use std::{
    fmt::Debug,
    sync::{Arc, RwLock},
};
#[allow(clippy::type_complexity)]
#[derive(Clone)]
pub struct XMLData<T, E>
where
    T: Clone + std::fmt::Debug + Send + Sync,
    E: Clone + std::fmt::Debug + Send + Sync,
{
    fn_decode: Arc<dyn Fn(&str) -> Result<T, E> + Send + Sync>,
    inner: Arc<RwLock<RawOrData<T>>>,
}

impl<T, E> XMLData<T, E>
where
    T: Clone + std::fmt::Debug + Send + Sync,
    E: Clone + std::fmt::Debug + Send + Sync,
{
    pub fn new(
        input: &str,
        fn_decode: impl Fn(&str) -> Result<T, E> + Send + Sync + 'static,
    ) -> Self {
        Self {
            fn_decode: Arc::new(fn_decode),
            inner: Arc::new(RwLock::new(RawOrData::new_raw(input))),
        }
    }

    pub fn get_data(&self) -> Result<Arc<T>, E> {
        if self.inner.read().unwrap().is_raw() {
            let s = self.inner.read().unwrap().unwrap_raw();
            let mut i = self.inner.write().unwrap();
            *i = RawOrData::new_data((self.fn_decode)(s.as_str())?);
        }
        Ok(self.inner.read().unwrap().unwrap_data())
    }

    pub fn get_raw(&self) -> Option<Arc<String>> {
        self.inner.read().unwrap().get_raw()
    }
}

impl<T, E> std::fmt::Debug for XMLData<T, E>
where
    T: Clone + std::fmt::Debug + Send + Sync,
    E: Clone + std::fmt::Debug + Send + Sync,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("XMLData")
            .field("inner", &self.inner)
            .finish()
    }
}

#[derive(Debug, Clone)]
enum RawOrData<T: Clone + std::fmt::Debug> {
    Raw(Arc<String>),
    Data(Arc<T>),
}

impl<T: Clone + std::fmt::Debug> RawOrData<T> {
    fn new_raw(s: &str) -> Self {
        Self::Raw(Arc::new(s.to_string()))
    }

    fn new_data(t: T) -> Self {
        Self::Data(Arc::new(t))
    }
    fn is_raw(&self) -> bool {
        match self {
            RawOrData::Raw(_) => true,
            RawOrData::Data(_) => false,
        }
    }

    #[allow(dead_code)]
    fn is_data(&self) -> bool {
        !self.is_raw()
    }

    fn get_raw(&self) -> Option<Arc<String>> {
        match self {
            RawOrData::Raw(s) => Some(s.clone()),
            RawOrData::Data(_) => None,
        }
    }

    fn get_data(&self) -> Option<Arc<T>> {
        match self {
            RawOrData::Raw(_) => None,
            RawOrData::Data(t) => Some(t.clone()),
        }
    }

    fn unwrap_raw(&self) -> Arc<String> {
        self.get_raw().unwrap()
    }

    fn unwrap_data(&self) -> Arc<T> {
        self.get_data().unwrap()
    }
}

pub struct ElementChildrenIter<'a, 'input> {
    children: Children<'a, 'input>,
}

impl<'a, 'input> Iterator for ElementChildrenIter<'a, 'input> {
    type Item = Node<'a, 'input>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.children.next() {
            Some(n) => match n.is_element() {
                true => Some(n),
                false => self.next(),
            },
            None => None,
        }
    }
}

pub trait ElementChildren<'a, 'input> {
    fn element_children(&self) -> ElementChildrenIter<'a, 'input>;
}

impl<'a, 'input> ElementChildren<'a, 'input> for Node<'a, 'input> {
    fn element_children(&self) -> ElementChildrenIter<'a, 'input> {
        ElementChildrenIter {
            children: self.children(),
        }
    }
}
