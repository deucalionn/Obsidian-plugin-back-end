use crate::schema::obsidian_file;
use diesel::prelude::*;
use uuid::Uuid;
use rocket::serde::{Deserialize, Serialize};
use log::error;
use std::cmp::{Ord, Eq, PartialOrd, PartialEq};
use std::env;

use langchain_rust::{
    add_documents,
    delete_documents,
    embedding::ollama::ollama_embedder::OllamaEmbedder,
    schemas::Document,
    vectorstore::{pgvector::StoreBuilder, VectorStore},
};

// Queryable will generate the code needed to load the struct from an SQL statement
#[derive(Debug, Queryable, Serialize, Deserialize, Ord, Eq, PartialEq, PartialOrd)]
#[diesel(table_name = obsidian_file)]
pub struct ObsidianFile {
    pub id: i32,
    pub name: String,
    pub content: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub embed_id: Option<Uuid>,
    
}


#[derive(Debug, Insertable, Serialize, Deserialize)]
#[diesel(table_name = obsidian_file)]
pub struct NewObsidianFile {
    pub name: String,
    pub content: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub embed_id: Option<Uuid>,
}


pub fn create_obsidian_file_in_db(conn: &mut PgConnection, new_obsidian_file: NewObsidianFile) -> ObsidianFile {
    diesel::insert_into(obsidian_file::table)
        .values(new_obsidian_file)
        .get_result::<ObsidianFile>(conn)
        .expect("Error saving new file content")
}

pub fn get_obsidian_file_by_id(conn: &mut PgConnection, file_id: i32) -> Option<ObsidianFile> {
    obsidian_file::table
        .find(file_id)
        .first(conn)
        .optional()
        .expect("Error loading file content")
}



async fn add_content_to_vector_table(content: String) -> Option<Uuid> {
    eprintln!("Adding content to vector table");

    let database_url: String = env::var("DATABASE_URL")
    .expect("DATABASE_URL must be set in the environment");
    
    let embedder = OllamaEmbedder::default().with_model("llama3.2");

    let store = match StoreBuilder::new()
        .embedder(embedder)
        .connection_url(&database_url)
        .vector_dimensions(4096)
        .build()
        .await
    { 
        Ok(store) => store,
        Err(error) => {
            error!("Error building vector store: {:?}", error);
            return None;
        },
    };

    let document = Document::new(content);

    match add_documents!(&store, &[document]).await {
        Ok(result) => {
            let uuid_str = &result[0];
            match Uuid::parse_str(uuid_str) {
                Ok(uuid) => Some(uuid),
                Err(error) => {
                    error!("Error parsing UUID: {:?}", error);
                    None
                }
            }
        },
        Err(error) => {
            error!("Error adding document to vector table: {:?}", error);
            None
        },
    }
}



async fn delete_document_from_vector_table(embed_id: Uuid) -> bool {
    eprintln!("Deleting content to vector table");

    let database_url: String = env::var("DATABASE_URL")
    .expect("DATABASE_URL must be set in the environment");

    let embedder = OllamaEmbedder::default().with_model("llama3.2");

    let store = match StoreBuilder::new()
    .embedder(embedder)
    .connection_url(&database_url)
    .vector_dimensions(4096)
    .build()
    .await
    { 
        Ok(store) => store,
        Err(error) => {
            error!("Error building vector store: {:?}", error);
            return false
        },
    };

    match delete_documents!(&store, &[embed_id.to_string()]).await {
        Ok(_) => true,
        Err(error) => {
            error!("Error deleting document from vector table: {:?}", error);
            return false
        },
    }
}


pub async fn update_or_create_obsidian_file_by_name(
    conn: &mut PgConnection,
    file_name: String,
    new_content: String,
) -> Option<ObsidianFile> {
    // fetch file by name
    match obsidian_file::table
        .filter(obsidian_file::name.eq(&file_name))
        .get_result::<ObsidianFile>(conn)
        .optional()
    {
        Ok(Some(mut file)) => {
            // if file exists, update it by replacing the content
            file.content = new_content.clone();
            file.updated_at = chrono::Local::now().naive_local();

            // delete document associated with the file 
            if let Some(embed_id) = file.embed_id {
                delete_document_from_vector_table(embed_id).await;
            }

            // to upload the new content to the vector table
            file.embed_id = add_content_to_vector_table(new_content.clone()).await;

            // update the file in the database
            diesel::update(obsidian_file::table.filter(obsidian_file::name.eq(&file_name)))
                .set((
                    obsidian_file::content.eq(file.content.clone()),
                    obsidian_file::updated_at.eq(file.updated_at),
                    obsidian_file::embed_id.eq(file.embed_id),
                ))
                .execute(conn)
                .expect("Error updating file");

            Some(file)
        }
        Ok(None) => {
            // if no file exists, create a new file
            let embed_id = add_content_to_vector_table(new_content.clone()).await;

            let new_file = NewObsidianFile {
                name: file_name,
                content: new_content,
                embed_id,
                created_at: chrono::Local::now().naive_local(),
                updated_at: chrono::Local::now().naive_local(),
            };

            Some(create_obsidian_file_in_db(conn, new_file))
        }
        Err(error) => {
            error!("Error updating or creating file: {:?}", error);
            None
        }
    }
}
