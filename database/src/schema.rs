table! {
    meals (id) {
        id -> Int4,
        date -> Date,
        value -> Text,
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

joinable!(meals -> menus (menu_id));

allow_tables_to_appear_in_same_query!(meals, menus,);
