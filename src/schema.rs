// @generated automatically by Diesel CLI.

diesel::table! {
    obsidian_file (id) {
        id -> Int4,
        name -> Text,
        content -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
