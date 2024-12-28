use langchain_rust::{
    chain::{Chain, ConversationalRetrieverChainBuilder}, embedding::ollama::ollama_embedder::OllamaEmbedder, llm::ollama::client::Ollama, memory::SimpleMemory, prompt::{PromptTemplate, TemplateFormat}, prompt_args, vectorstore::{pgvector::StoreBuilder, Retriever}
};

use std::env;


pub async fn search_in_note(
    search_query: String,
) -> String {
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
        Err(e) => {
            eprintln!("Error building store: {:?}", e);
            return "Error".to_string();
        }
    };

    let llm = Ollama::default().with_model("llama3.2");

    let prompt = PromptTemplate::new("Search in note {context} to answer the question : {question}".to_owned(), vec!["context".to_owned(), "question".to_owned()], TemplateFormat::FString);


    let chain = ConversationalRetrieverChainBuilder::new()
    .llm(llm)
    .rephrase_question(true)
    .memory(SimpleMemory::new().into())
    .retriever(Retriever::new(store, 5))
    .prompt(prompt)
    .build()
    .expect("Error building ConversationalChain");

    let input_variables = prompt_args! {
        "question" => search_query,
    };

    let result = chain.invoke(input_variables).await;
    if let Ok(result) = result {
        result
    }
    else {
        "Error".to_string()
    }
}
