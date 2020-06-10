use std::fmt;

use reqwest::Client;
use tokio::main;

use quick_xml::de::{from_str, DeError};
use serde::Deserialize;

#[derive(Debug)]
pub enum Error {
    ReqwestError(reqwest::Error),
    XmlError(DeError),
    ElementNotFoundError,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ReqwestError(err) => write!(f, "reqwest error: {}", err),
            Error::XmlError(err) => write!(f, "XML deserialization error: {}", err),
            Error::ElementNotFoundError => write!(f, "expected element not found"),
        }
    }
}

impl std::error::Error for Error {}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        Error::ReqwestError(err)
    }
}

impl From<DeError> for Error {
    fn from(err: DeError) -> Error {
        Error::XmlError(err)
    }
}

#[derive(Deserialize, Debug)]
struct HeaderElement {
    #[serde(rename = "numHits")]
    num_hits: usize,
    #[serde(rename = "totalHits")]
    total_hits: usize,
    records: String,
}

#[derive(Deserialize)]
struct ResultElement {
    header: HeaderElement,
}

#[derive(Deserialize, Debug)]
struct ArticleElement {
    title: String,
    relations: Vec<RelationsElement>,
}

#[derive(Deserialize, Debug)]
struct Association {
    #[serde(rename = "webLink")]
    weblinks: Vec<WebLinkElement>,
}

#[derive(Deserialize, Debug)]
struct RelationsElement {
    associations: Vec<Association>,
}

#[derive(Deserialize, Debug)]
struct WebLinkElement {
    #[serde(rename = "type")]
    typ: String,
    title: String,
    id: String,
    language: String,
    #[serde(rename = "physUrl")]
    phys_url: String,
}

#[derive(Deserialize, Debug)]
struct Hit {
    score: f64,
    article: ArticleElement,
}

#[derive(Deserialize, Debug)]
struct HitsElement {
    #[serde(rename = "hit")]
    hits: Vec<Hit>,
}

#[derive(Deserialize)]
struct SearchResponse {
    result: ResultElement,
    hits: HitsElement,
}

/// Searches the CORDIS repository for the given query, which is an SQL-like format
pub async fn search(query: &str, page: usize, limit: usize) -> Result<(), Error> {
    let client = reqwest::Client::new();

    let resp = client
        .get("https://cordis.europa.eu/search/en")
        .query(&[
            ("q", query),
            ("p", &page.to_string()),
            ("num", &limit.to_string()),
            ("srt", "/article/contentUpdateDate:decreasing"),
            ("format", "xml"),
        ])
        .send()
        .await?;

    let text = resp.text().await?;
    //let text = std::fs::read_to_string("data/page_1.xml").unwrap();

    let result: SearchResponse = from_str(&text)?;

    for hit in result.hits.hits {
        println!("result: {:?}", hit);
    }

    Ok(())
}
