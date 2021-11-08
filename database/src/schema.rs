table! {
    days (id) {
        id -> Int4,
        date -> Date,
        meals -> Bytea,
        menu_id -> Int4,
    }
}

table! {
    menus (id) {
        id -> Int4,
        title -> Text,
        slug -> Text,
        updated_at -> Nullable<Timestamptz>,
    }
}

joinable!(days -> menus (menu_id));

allow_tables_to_appear_in_same_query!(days, menus,);
