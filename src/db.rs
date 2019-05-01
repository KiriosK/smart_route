use crate::models::{OneTicketSolution, Ticket, TwoTicketSolution};
use crate::schema;
use chrono::NaiveDate;
use diesel::prelude::*;
use futures::future::{self, Future};
use itertools::Itertools;
use std::env;

static DEFAULT_DATABASE_URL: &str = "postgres://postgres:secret@localhost/tickets";
static THREE_HOURS: i32 = 10800;
static TWENTY_FOUR_HOURS: i32 = 86400;

type FutureBox<T> = Box<Future<Item = T, Error = GenericError> + Send>;
type GenericError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Deserialize, Serialize, Debug)]
pub struct TicketList {
    tickets: Vec<Ticket>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Solutions {
    solutions: Vec<Solution>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Solution {
    ids: Vec<String>,
    price: i32,
}

#[derive(Deserialize, Debug)]
pub struct SearchRequest {
    departure_code: String,
    arrival_code: String,
    departure_date: String,
    limit: i64,
}

pub fn connect_to_db() -> Result<PgConnection, GenericError> {
    let database_url = env::var("DATABASE_URL").unwrap_or(String::from(DEFAULT_DATABASE_URL));
    match PgConnection::establish(&database_url) {
        Ok(connection) => Ok(connection),
        Err(e) => Err(GenericError::from(e)),
    }
}

pub fn add_tickets(ticket_list: TicketList, db_connection: &PgConnection) -> FutureBox<()> {
    let new_tickets = ticket_list.tickets.iter().map(|ticket| {
        format!(
            "('{}', '{}', '{}', {}, {}, {})",
            ticket.id,
            ticket.departure_code,
            ticket.arrival_code,
            ticket.departure_time,
            ticket.arrival_time,
            ticket.price
        )
    });
    let v: String = new_tickets.collect::<Vec<_>>().join(", ");

    // replace_into is not working for diesel PgConnection
    let query = format!(
    "INSERT INTO tickets (id, departure_code, arrival_code, departure_time, arrival_time, price)
         VALUES {} ON CONFLICT (id) DO UPDATE 
         SET 
         departure_code=EXCLUDED.departure_code, 
         arrival_code=EXCLUDED.arrival_code, 
         departure_time=EXCLUDED.departure_time,
         arrival_time=EXCLUDED.arrival_time,
         price=EXCLUDED.price",
    v
  );

    match db_connection.execute(&query) {
        Ok(_) => Box::new(future::ok(())),
        Err(e) => Box::new(future::err(GenericError::from(e))),
    }
}

pub fn search(
    search_request: SearchRequest,
    db_connection: PgConnection,
) -> impl Future<Item = Solutions, Error = GenericError> {
    use schema::tickets::dsl::*;

    let date = NaiveDate::parse_from_str(&search_request.departure_date, "%Y-%m-%d").unwrap();
    let left_bound = date.and_hms(0, 0, 0).timestamp() as i32;
    let right_bound = date.and_hms(23, 59, 59).timestamp() as i32;

    future::ok(db_connection).and_then(move |db_connection| {
        let query = format!(
          "SELECT 
              t_from.id AS from_id,
              t_to.id AS to_id,
              t_from.price + t_to.price as price,
              t_from.departure_time 
          FROM tickets AS t_from
          LEFT JOIN tickets AS t_to 
          ON 
              (t_from.arrival_code = t_to.departure_code) AND
              (t_to.departure_time - t_from.arrival_time BETWEEN {three_hours} AND {twenty_four_hours})

          WHERE 
              (t_from.departure_code = '{from}' AND t_to.arrival_code = '{to}' ) AND
              (t_from.departure_time BETWEEN {l} AND {r})
          ORDER BY 
              (price) ASC,
              t_from.departure_time ASC 
          LIMIT {limit}",
          three_hours = THREE_HOURS,
          twenty_four_hours = TWENTY_FOUR_HOURS,
          from = search_request.departure_code,
          to = search_request.arrival_code,
          l = left_bound,
          r = right_bound,
          limit = search_request.limit
      );

        let one_ticket_solutions = tickets
            .select((id, price, departure_time))
            .filter(departure_code.eq(&search_request.departure_code))
            .filter(arrival_code.eq(&search_request.arrival_code))
            .filter(departure_time.between(left_bound, right_bound))
            .limit(search_request.limit)
            .load::<OneTicketSolution>(&db_connection)?;

        let two_ticket_solutions = diesel::sql_query(query)
            .load::<TwoTicketSolution>(&db_connection)?;

        let it1 = one_ticket_solutions.into_iter().map(|solution| Solution {
            ids: vec![solution.id],
            price: solution.price,
        });

        let it2 = two_ticket_solutions.into_iter().map(|solution| Solution {
            ids: vec![solution.from_id, solution.to_id],
            price: solution.price,
        });

        let ans: Vec<Solution> = it1
            .chain(it2)
            .sorted_by_key(|solution| solution.price)
            .take(search_request.limit as usize)
            .collect();

        Ok(Solutions { solutions: ans })
    })
}
