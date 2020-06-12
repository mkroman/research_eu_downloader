use std::fmt;

use reqwest::Client;

use quick_xml::de::{from_str, DeError};
use serde::Deserialize;

#[derive(Debug)]
pub enum Error {
    ReqwestError(reqwest::Error),
    XmlError(DeError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ReqwestError(err) => write!(f, "reqwest error: {}", err),
            Error::XmlError(err) => write!(f, "XML deserialization error: {}", err),
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
pub struct HeaderElement {
    #[serde(rename = "numHits")]
    num_hits: usize,
    #[serde(rename = "totalHits")]
    total_hits: usize,
    records: String,
}

#[derive(Deserialize)]
pub struct ResultElement {
    header: HeaderElement,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Identifiers {
    issn: String,
    #[serde(rename = "catalogueNumber")]
    catalogue_number: String,
    #[serde(rename = "cellarId")]
    cellar_id: String,
    issue: String,
}

impl Identifiers {
    pub fn issue(&self) -> &str {
        &self.issue
    }
}

#[derive(Deserialize, Debug)]
pub struct Article {
    title: String,
    relations: Vec<RelationsElement>,
    identifiers: Identifiers,
}

impl Article {
    pub fn identifiers(&self) -> &Identifiers {
        &self.identifiers
    }

    /// Returns a list of weblinks found in any of the associations
    pub fn weblinks(&self) -> Vec<&WebLinkElement> {
        self.relations
            .iter()
            .flat_map(|relation| &relation.associations)
            .flat_map(|association| &association.weblinks)
            .collect()
    }
}

#[derive(Deserialize, Debug)]
pub struct Association {
    #[serde(rename = "webLink")]
    weblinks: Vec<WebLinkElement>,
}

#[derive(Deserialize, Debug)]
pub struct RelationsElement {
    associations: Vec<Association>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct WebLinkElement {
    #[serde(rename = "type")]
    pub typ: String,
    title: String,
    id: String,
    language: String,
    #[serde(rename = "physUrl")]
    pub phys_url: String,
}

#[derive(Deserialize, Debug)]
pub struct Hit {
    score: f64,
    article: Article,
}

impl Hit {
    pub fn article(&self) -> &Article {
        &self.article
    }
}

#[derive(Deserialize, Debug)]
pub struct HitsElement {
    #[serde(rename = "hit")]
    hits: Vec<Hit>,
}

#[derive(Deserialize)]
pub struct SearchResponse {
    result: ResultElement,
    hits: HitsElement,
}

impl SearchResponse {
    pub fn total_hits(&self) -> usize {
        self.result.header.total_hits
    }

    /// Calculates the number of total pages to request with the current per-page limit to receive
    /// all the hits
    pub fn num_pages(&self) -> usize {
        let total_hits = self.total_hits();
        // FIXME: Don't use hardcoded size of 10!!!

        if total_hits % 10 == 0 {
            total_hits / 10
        } else {
            (total_hits / 10) + 1
        }
    }

    pub fn hits(&self) -> &Vec<Hit> {
        self.hits.hits.as_ref()
    }

    pub fn articles(&self) -> Vec<&Article> {
        self.hits().iter().map(|x| x.article()).collect()
    }
}

/// Searches the CORDIS repository for the given query, which is an SQL-like format
pub async fn search(query: &str, page: usize, limit: usize) -> Result<SearchResponse, Error> {
    let client = Client::new();

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
    let result: SearchResponse = from_str(&text)?;

    Ok(result)
}
