use crate::schema::obsidian_file;
use diesel::prelude::*;
use rocket::serde::{Deserialize, Serialize};
use std::cmp::{Ord, Eq, PartialOrd, PartialEq};

// Queryable will generate the code needed to load the struct from an SQL statement
#[derive(Debug, Queryable, Serialize, Deserialize, Ord, Eq, PartialEq, PartialOrd)]
#[diesel(table_name = obsidian_file)]
pub struct ObsidianFile {
    pub id: i32,
    pub name: String,
    pub content: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    
}


#[derive(Debug, Insertable, Serialize, Deserialize)]
#[diesel(table_name = obsidian_file)]
pub struct NewObsidianFile {
    pub name: String,
    pub content: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}


pub fn create_obsidian_file_in_db(conn: &mut PgConnection, new_obsidian_file: NewObsidianFile) -> ObsidianFile {
    diesel::insert_into(obsidian_file::table)
        .values(&new_obsidian_file)
        .get_result(conn)
        .expect("Error saving new file content")
}

pub fn get_obsidian_file_by_id(conn: &mut PgConnection, file_id: i32) -> Option<ObsidianFile> {
    obsidian_file::table
        .find(file_id)
        .first(conn)
        .optional()
        .expect("Error loading file content")
}


pub fn update_or_create_obsidian_file_by_name(conn: &mut PgConnection, file_name: String, new_content: String) -> Option<ObsidianFile> {
    match diesel::update(obsidian_file::table.filter(obsidian_file::name.eq(&file_name)))
        .set((
            obsidian_file::content.eq(&new_content),
            obsidian_file::updated_at.eq(chrono::Local::now().naive_local())
        ))
        .get_result(conn) // Return the updated file
        .optional() // Return None if no file is found
    {
        Ok(Some(file)) => Some(file),
        Ok(None) => {
            let new_file = NewObsidianFile {
                name: file_name,
                content: new_content,
                created_at: chrono::Local::now().naive_local(),
                updated_at: chrono::Local::now().naive_local(),
            };
            Some(create_obsidian_file_in_db(conn, new_file))
        }
        Err(_) => None,
    }
}