use std::cmp::Ordering;
use std::error::Error;
use std::fs::File;
use std::io;
use std::path::Path;
use datetime::Instant;
use log::*;

use crate::{
    config,
    mpd_api,
    spotify_api,
    util
};

pub fn run(mut config: config::Config) {
    let expires_at = config.spotify.expires_at
        .expect("token expiration time is not configured");

    let properties = mpd_api::get_playing_song_props(&config)
        .expect("unable to fetch currently playing song's properties");

    // Refresh the Spotify access token, if it's expired
    if Instant::now().cmp(&Instant::at(expires_at)) == Ordering::Greater {
        update_token(&mut config)
    }

    // Fetch the album metadata by ID, if the MPD server
    // is powered by Mopidy and the Mopidy-Spotify addon is installed.
    // Otherwise, search for the song and use the first result.

    let images = if let Some(id) = properties.get("X-AlbumUri") {
        if !id.starts_with("spotify:album:") {
            panic!("invalid id: {}", id);
        }

        let slice = id
            .get("spotify:album:".len()..id.len())
            .expect("unable to extract album id");

        debug!("Looking up album by album ID: {}", slice);
        lookup_images_by_album_id(&config, slice)
    } else {
        let artist = properties.get("Artist")
            .expect("no `Artist` in properties");
        let title = properties.get("Title")
            .expect("no `Title` in properties");

        debug!("Looking up song by name: {} - {}", artist, title);
        lookup_images_by_name(&config, artist, title)
    }.expect("unable to lookup the images");

    // parse size from config into width & height
    let size = {
        let size_parts: Vec<u32> = config.cover.preferred_size
            .split("x")
            .map(|e| e.parse::<u32>()
                .expect("unable to parse size part"))
            .take(2).collect();

        (size_parts[0], size_parts[1])
    };

    if let Some(image) = images.iter().find(|i| i.width == size.0 && i.height == size.1) {
        info!("Downloading image: {}", image.url);

        download_image(&image.url, &config.cover.output_path)
            .expect("unable to download image");

        info!("Image downloaded to {}", config.cover.output_path);
    } else {
        error!("Size {} is not available for album", config.cover.preferred_size);

        let sep = String::from(", ");

        error!("Available sizes: {}", images.iter()
            .map(|i| format!("{}x{}", i.width, i.height))
            .intersperse(sep)
            .collect::<String>());

        panic!("preferred size is not available");
    }
}

fn update_token(config: &mut config::Config) {
    debug!("Access token has expired, requesting a new one");

    let response = spotify_api::refresh_token(&config)
        .expect("unable to refresh token");

    trace!("Received token: {}", response.access_token);

    config.spotify.access_token = Some(response.access_token);
    config.spotify.expires_at   = Some(util::offset_time_by_expiration(response.expires_in));

    trace!("Updating configuration");

    config::save_config(&config)
        .expect("unable to save config");

    debug!("Configuration updated and written to disk");
}

fn lookup_images_by_name(config: &config::Config, artist: &str, title: &str)
  -> Result<Vec<spotify_api::Image>, Box<dyn Error>> {

    let results = spotify_api::search_track(
        config,
        /* query */ &format!("{} - {}", artist, title),
        /* type  */ "track")?;

    if let Some(tracks) = results.tracks {
        if tracks.items.is_empty() {
            return Err("No tracks found")?;
        }

        Ok(tracks.items
            .iter()
            .find(|t|
                util::either_contains(&t.name, title) &&
                t.artists.iter().any(|a| util::either_contains(&a.name, artist)))
            .map(|t| t.album.clone().images)
            .ok_or("No matching tracks found")?)
    } else {
        Err("Unable to perform search")?
    }
}

fn lookup_images_by_album_id(config: &config::Config, id: &str)
  -> Result<Vec<spotify_api::Image>, Box<dyn Error>> {

    spotify_api::get_album(config, id).and_then(|r| Ok(r.images))
}

fn download_image(url: &str, path: &str)
  -> Result<(), Box<dyn Error>> {

    let os_path = Path::new(path);
    let output_bytes = reqwest::blocking::get(url)?.bytes()?;

    File::create(os_path)
        .and_then(|mut f| {
            // copy output stream to the file
            io::copy(&mut output_bytes.as_ref(), &mut f)?; Ok(())
        })?;

    Ok(())
}