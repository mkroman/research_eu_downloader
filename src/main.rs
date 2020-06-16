use std::collections::VecDeque;
use std::path::Path;

use indicatif::{ProgressBar, ProgressStyle};
use log::debug;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

mod request_eu;

struct Article {
    issue: String,
    url: String,
}

async fn download_article_pdf<P: AsRef<Path>>(
    article: Article,
    output_path: P,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut res = reqwest::get(&article.url).await?;
    let mut file = File::create(&output_path).await?;

    while let Some(chunk) = res.chunk().await? {
        file.write_all(&chunk).await?;
    }

    Ok(())
}

const ENGLISH_MAGAZINE_QUERY: &str =
    "/article/relations/categories/collection/code='mag' AND language='en'";

async fn get_magazines_page(page: usize) -> Result<request_eu::SearchResponse, request_eu::Error> {
    request_eu::search(ENGLISH_MAGAZINE_QUERY, page, 10).await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let output_dir = Path::new("./magazines");

    std::fs::create_dir(output_dir)?;

    let pb = ProgressBar::new(0);
    let sty =
        ProgressStyle::default_bar().template("[{elapsed_precise}] {bar} {pos:>7}/{len:7} {msg}");

    pb.set_style(sty.clone());
    pb.set_message("Downloading search pages");

    let mut page = 1;
    let mut result = get_magazines_page(page).await?;
    let num_pages = result.num_pages();
    let mut queue: VecDeque<Article> = VecDeque::new();

    pb.set_length(num_pages as u64);
    pb.inc(1);

    while page < num_pages {
        pb.inc(1);

        // Process the articles
        for article in result.articles().iter() {
            let weblinks = article.weblinks();

            if let Some(pdf_web_link) = weblinks.iter().find(|x| x.typ == "formatPdf") {
                queue.push_back(Article {
                    issue: article.identifiers().issue().to_owned(),
                    url: pdf_web_link.phys_url.clone(),
                });
            }
        }

        page += 1;
        debug!("Querying page {}", page);
        result = get_magazines_page(page).await?;
    }

    pb.finish_with_message("Done. Downloading files.");
    pb.set_position(0);
    pb.set_length(queue.len() as u64);

    while let Some(article) = queue.pop_front() {
        let output_path = output_dir.join(&format!("{}.pdf", article.issue));

        pb.inc(1);
        pb.set_message(output_path.to_str().unwrap());

        if output_path.exists() {
            continue;
        }

        debug!(
            "Downloading {} to `{}'",
            article.url,
            output_path.to_str().unwrap()
        );

        download_article_pdf(article, output_path).await?;
    }

    pb.finish_with_message("Done");

    Ok(())
}
