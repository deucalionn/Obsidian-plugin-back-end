// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "vector"))]
    pub struct Vector;
}

diesel::table! {
    langchain_pg_collection (uuid) {
        name -> Nullable<Varchar>,
        cmetadata -> Nullable<Json>,
        uuid -> Text,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Vector;

    langchain_pg_embedding (uuid) {
        collection_id -> Nullable<Text>,
        embedding -> Nullable<Vector>,
        document -> Nullable<Varchar>,
        cmetadata -> Nullable<Json>,
        uuid -> Text,
    }
}

diesel::table! {
    obsidian_file (id) {
        id -> Int4,
        name -> Text,
        content -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        embed_id -> Nullable<Uuid>,
    }
}

diesel::joinable!(langchain_pg_embedding -> langchain_pg_collection (collection_id));

diesel::allow_tables_to_appear_in_same_query!(
    langchain_pg_collection,
    langchain_pg_embedding,
    obsidian_file,
);
