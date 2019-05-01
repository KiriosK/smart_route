use crate::schema::tickets;
use diesel::sql_types::{Integer, Text};
use diesel::{Insertable, Queryable};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Insertable)]
#[table_name = "tickets"]
pub struct Ticket {
    pub id: String,
    pub departure_code: String,
    pub arrival_code: String,
    pub departure_time: i32,
    pub arrival_time: i32,
    pub price: i32,
}

#[derive(Queryable, Debug)]
pub struct OneTicketSolution {
    pub id: String,
    pub price: i32,
    pub departure_time: i32,
}

#[derive(QueryableByName, Debug)]
pub struct TwoTicketSolution {
    #[sql_type = "Text"]
    pub from_id: String,
    #[sql_type = "Text"]
    pub to_id: String,
    #[sql_type = "Integer"]
    pub price: i32,
    #[sql_type = "Integer"]
    pub departure_time: i32,
}
