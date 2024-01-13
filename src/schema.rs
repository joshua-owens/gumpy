// @generated automatically by Diesel CLI.

diesel::table! {
    runs (id) {
        id -> Int4,
        distance -> Numeric,
        duration -> Interval,
        date -> Date,
        created_at -> Nullable<Timestamp>,
    }
}
