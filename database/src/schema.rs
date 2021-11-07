table! {
    days (id) {
        id -> Text,
        date -> Date,
        meals -> Text,
        menu_id -> Text,
    }
}

table! {
    menus (id) {
        id -> Text,
        title -> Text,
        updated_at -> Nullable<Timestamptz>,
    }
}

joinable!(days -> menus (menu_id));

allow_tables_to_appear_in_same_query!(
    days,
    menus,
);
