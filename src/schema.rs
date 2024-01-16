// @generated automatically by Diesel CLI.

diesel::table! {
    runs (id) {
        id -> Int4,
        distance -> Numeric,
        duration -> Interval,
        created_at -> Timestamp,
    }
}
