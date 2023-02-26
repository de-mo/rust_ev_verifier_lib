use super::{DataStructureTrait, DeserializeError, VerifierDataType};

pub struct VerifierData<T: DataStructureTrait> {
    data_type: VerifierDataType,
    data: Option<Box<T>>,
}

pub trait VerifierDataTrait<T: DataStructureTrait> {
    fn new(data_type: VerifierDataType, data: Option<Box<T>>) -> Self;

    fn get_data(&self) -> &Option<Box<T>>;
    fn set_data(&mut self, data: Option<Box<T>>);

    fn is_some(&self) -> bool;

    fn is_none(&self) -> bool {
        return !self.is_some();
    }

    fn get_data_type(&self) -> &VerifierDataType;

    fn from_json(&mut self, s: &String) -> Result<&Self, DeserializeError>
    where
        Self: VerifierDataTrait<T>,
    {
        match T::from_json(s) {
            Ok(res) => self.set_data(Some(Box::new(res))),
            Err(e) => return Err(e),
        }
        Ok(self)
    }
}

impl<T: DataStructureTrait> VerifierDataTrait<T> for VerifierData<T> {
    fn new(data_type: VerifierDataType, data: Option<Box<T>>) -> Self {
        Self { data_type, data }
    }

    fn get_data(&self) -> &Option<Box<T>> {
        &self.data
    }
    fn set_data(&mut self, data: Option<Box<T>>) {
        self.data = data;
    }

    fn is_some(&self) -> bool {
        self.data.is_some()
    }

    fn get_data_type(&self) -> &VerifierDataType {
        &self.data_type
    }
}

pub trait VerifierDataTraitNew<T: DataStructureTrait> {
    fn new_without_data() -> Self;
}
