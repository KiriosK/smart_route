table! {
    tickets (id) {
        id -> Varchar,
        departure_code -> Varchar,
        arrival_code -> Varchar,
        departure_time -> Int4,
        arrival_time -> Int4,
        price -> Int4,
    }
}
