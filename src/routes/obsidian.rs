use crate::schema::obsidian_file;
use crate::models::obsidian::{NewObsidianFile, ObsidianFile,
    create_obsidian_file_in_db, 
    add_content_to_vector_table, 
    delete_document_from_vector_table};
use diesel::PgConnection;
use diesel::prelude::*;
use log::error;




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
