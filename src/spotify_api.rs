use std::error::Error;
use reqwest::blocking::{Client, Response};
use serde::Deserialize;

use crate::config;

#[derive(Deserialize, Clone)]
pub struct Album {
    pub name: String,
    pub images: Vec<Image>,
}

#[derive(Deserialize, Clone)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub url: String,
}

#[derive(Deserialize)]
pub struct Artist {
    pub name: String,
}

#[derive(Deserialize)]
pub struct SearchResults {
    pub tracks: Option<SearchResult<Track>>,
}

#[derive(Deserialize)]
pub struct SearchResult<T> {
    pub href: String,
    pub items: Vec<T>,
}

#[derive(Deserialize)]
pub struct Track {
    pub album: Album,
    pub artists: Vec<Artist>,
    pub id: String,
    pub name: String,
}

#[derive(Deserialize)]
pub struct ExchangeCodeResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub token_type: String,
}

#[derive(Deserialize)]
pub struct RefreshTokenResponse {
    pub access_token: String,
    pub scope: String,
    pub expires_in: i64,
    pub token_type: String,
}

#[derive(Deserialize)]
struct SpotifyError {
    error_description: String
}

const SPOTIFY_TOKEN_ENDPOINT:       &str = "https://accounts.spotify.com/api/token";
const SPOTIFY_GET_ALBUM_ENDPOINT:   &str = "https://api.spotify.com/v1/albums/{id}";
const SPOTIFY_SEARCH_ENDPOINT:      &str = "https://api.spotify.com/v1/search";

fn map_error_result(response: Response) -> String {
    response.json::<SpotifyError>()
        .map(|e| e.error_description)
        .unwrap_or(String::from("Server returned an unknown error"))
}

fn map_response_json<T: for <'de> Deserialize<'de>>(response: Response)
  -> Result<T, Box<dyn Error>> {

    if response.status().is_success() {
        Ok(response.json::<T>()?)
    } else {
        Err(map_error_result(response))?
    }
}

fn substitute(url: &str, params: &[(&str, &str)]) -> String {
    let mut new_string = String::from(url);

    for (key, value) in params {
        new_string = new_string.replace(&format!("{{{}}}", key), value);
    }

    new_string
}

fn oauth_request(config: &config::Config, params: &[(&str, &str)])
  -> Result<Response, Box<dyn Error>> {

    let client = Client::new();

    let request = client.post(SPOTIFY_TOKEN_ENDPOINT)
        .basic_auth(&config.spotify.client_id,
                    Some(&config.spotify.client_secret))
        .form(&params)
        .build()?;

    Ok(client.execute(request)?)
}

pub fn exchange_code(config: &config::Config, redirect_uri: &str, code: &str)
  -> Result<ExchangeCodeResponse, Box<dyn Error>> {

    map_response_json(
        oauth_request(config, &[
            ("redirect_uri", redirect_uri),
            ("code", code),
            ("grant_type", "authorization_code")]
        )
    ?)
}

pub fn refresh_token(config: &config::Config)
  -> Result<RefreshTokenResponse, Box<dyn Error>> {

    map_response_json(
        oauth_request(config, &[
            ("refresh_token", config.spotify.refresh_token
                .as_ref()
                .expect("refresh token is not configured")
                .as_str()),
            ("grant_type", "refresh_token")]
        )
    ?)
}

pub fn get_album(config: &config::Config, id: &str)
  -> Result<Album, Box<dyn Error>> {

    let client = Client::new();

    let request = client.get(
            substitute(SPOTIFY_GET_ALBUM_ENDPOINT, &[("id", id)])
        ).bearer_auth(config.spotify.access_token
            .as_ref()
            .expect("access token is not configured"))
        .build()?;

    Ok(map_response_json(client.execute(request)?)?)
}

pub fn search(config: &config::Config, q: &str, type_: &str)
  -> Result<SearchResults, Box<dyn Error>> {

    let client = Client::new();

    let request = client.get(SPOTIFY_SEARCH_ENDPOINT)
        .query(&[("q", q), ("type", type_)])
        .bearer_auth(config.spotify.access_token
            .as_ref()
            .expect("access token is not configured"))
        .build()?;

    Ok(map_response_json(client.execute(request)?)?)
}