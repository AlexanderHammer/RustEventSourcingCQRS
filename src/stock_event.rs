use std::fmt::Display;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum StockEvent {
    ADD,
    CREATE,
    SET,
    DELETE,
}

impl std::str::FromStr for StockEvent {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<StockEvent, Self::Err> {
        match s {
            "add-stock-item" => Ok(StockEvent::ADD),
            "create-stock-item" => Ok(StockEvent::CREATE),
            "set-stock-item" => Ok(StockEvent::SET),
            "delete-stock-item" => Ok(StockEvent::DELETE),
            _ => Err("Variant not found"),
        }
    }
}

impl Display for StockEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            StockEvent::ADD => "add-stock-item",
            StockEvent::CREATE => "create-stock-item",
            StockEvent::SET => "set-stock-item",
            StockEvent::DELETE => "delete-stock-item",
        };
        write!(f, "{}", str)
    }
}
