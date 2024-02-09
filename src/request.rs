use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateStockItem {
    pub part_no: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub count: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AdjustStockItem {
    pub part_no: String,
    pub count: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteStockItem {
    pub part_no: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateGenericEvent {
    pub stream_name: String,
    pub stream_prefix: String,
    pub event_type: String,
    pub data: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GenericEvent {
    pub data: HashMap<String, String>,
}