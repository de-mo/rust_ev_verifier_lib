pub mod setup;

pub trait FromJson {
    fn from_json(s: &String) -> Self;
}
