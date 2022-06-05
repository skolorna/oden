table! {
    days (date, menu_id) {
        date -> Date,
        meals -> Bytea,
        menu_id -> Uuid,
    }
}

pub const DAYS_TABLE: &str = "days";

table! {
    menus (id) {
        id -> Uuid,
        title -> Text,
        slug -> Text,
        updated_at -> Nullable<Timestamptz>,
    }
}

pub const MENUS_TABLE: &str = "menus";

joinable!(days -> menus (menu_id));

allow_tables_to_appear_in_same_query!(days, menus,);
