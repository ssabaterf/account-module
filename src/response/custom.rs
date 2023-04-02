use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Pagination<T>{
    pub skip: u64,
    pub limit: u64,
    pub count: u64,
    pub result: Vec<T>
}