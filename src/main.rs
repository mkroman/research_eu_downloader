use tokio::main;
mod request_eu;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let query = "/article/relations/categories/collection/code='mag' AND language='en'";
    let result = request_eu::search(query, 1, 10).await?;

    println!("Hello, world!");

    Ok(())
}
