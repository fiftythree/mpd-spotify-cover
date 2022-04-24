use std::io;
use log::*;

use crate::{
    spotify_api,
    util,
    config
};

pub fn run(mut config: config::Config) {
    println!(
        "You are not authenticated. To generate an access token, visit the following URL: {}\
        \nReplace 'your-app-redirect-uri' with the configured URI.\
        \nAfter authenticating, copy the URL from your browser's address bar, and paste it here:",
        format!("https://accounts.spotify.com/authorize\
            ?client_id={}\
            &response_type=code\
            &redirect_uri=your-app-redirect-uri", config.spotify.client_id));

    let mut auth_url = String::new();
    io::stdin().read_line(&mut auth_url)
        .expect("unable to read from stdin");

    let opt_pair: Option<(&str, &str)> = try {
        let code_index      = auth_url.find("?code=")?;
        let redirect_uri    = auth_url.get(0..code_index)?;
        let code            = auth_url.get(code_index + "?code=".len()..auth_url.len())?.trim();

        (redirect_uri, code)
    };

    let parameters = opt_pair
        .expect("malformed url");
    let response = spotify_api::exchange_code(&config, parameters.0, parameters.1)
        .expect("unable to exchange authorization code");

    debug!("Updating configuration:\
            \n access_token: {} \
            \n refresh_token: {} \
            \n expires_in: {}",
            response.access_token, response.refresh_token, response.expires_in);

    config.spotify.access_token  = Some(response.access_token);
    config.spotify.refresh_token = Some(response.refresh_token);
    config.spotify.expires_at    = Some(util::offset_time_by_expiration(response.expires_in));

    config::save_config(&config)
        .expect("unable to save config");

    info!("Configuration updated");
}


