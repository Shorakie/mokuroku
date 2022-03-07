use serde_json::json;
use tracing::{info, error};
use thiserror::Error;

use crate::graphql::{
    SEARCH_ANIME_QUERY,
    make_anime_query,
};
use crate::strings::anime::url::ANILIST_API;
use crate::models::anime::{Anime, SingleAnimeQueryResponse};

#[derive(Error, Debug)]
pub enum AnimeQueryError {
    #[error("Can not find `{0}` anime")]
    AnimeNotFoundError(String),
    #[error("Request error")]
    RequestError(#[from] reqwest::Error),
    #[error("Json deserialize error")]
    DeserializeError(#[from] serde_json::Error),
}

pub async fn find_anime(search_string: &str) -> Result<Anime, AnimeQueryError> {
    let client = reqwest::Client::new();

    // Define query and variables
    let query = json!({"query": SEARCH_ANIME_QUERY, "variables": {"search_string": search_string}});

    match client.post(ANILIST_API)
        .header("Accept", "application/json")
        .json(&query)
        .send()
        .await {
            Ok(response) => match response.json::<SingleAnimeQueryResponse>().await {
                Ok(query_response) => query_response.data
                    .ok_or(AnimeQueryError::AnimeNotFoundError(search_string.to_owned()))
                    .map(|data| data.anime),
                Err(err) => {
                    error!("failed to parse the graphql SingleAnimeQueryResponse: {:#?}", err);
                    Err(AnimeQueryError::RequestError(err))
                },
            },
            Err(reqwest_err) => {
                error!("failed to request single anime: {:#?}", reqwest_err);
                Err(AnimeQueryError::RequestError(reqwest_err))
            },
        }
}

pub async fn get_animes(anime_ids: &Vec<i64>) -> Option<serde_json::Value> {
    let client = reqwest::Client::new();

    // Define query and variables
    let query = json!({"query": make_anime_query(&anime_ids)});
    info!("{:#?}", query);
    match client.post(ANILIST_API)
        .header("Accept", "application/json")
        .json(&query)
        .send()
        .await {
            Ok(resp) => match resp.text().await {
                Ok(text) => match serde_json::from_str(&text) {
                    Ok(resp_json) => Some(resp_json),
                    Err(_) => None,
                },
                Err(_) => None,
            },
            Err(_) => None,
        }
}