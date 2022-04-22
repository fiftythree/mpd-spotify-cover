# mpd-spotify-cover

This is a tool to fetch the album cover for the current playing song in MPD via Spotify API.

![image](https://user-images.githubusercontent.com/34306421/164785503-f2f402fb-3837-4384-ab8a-4b297dd58b6d.png)

## How it works

The app connects to to the MPD server provided in the config, retrieves the current song's options
and downloads the album cover.

If you're using Mopidy and have Mopidy-Spotify installed, the app would use `X-AlbumUri` property,
and would look up the cover directly. Otherwise it would perform a search based on the song 
artist and the song title.

## Setup

1. Create an application at https://developer.spotify.com/dashboard/
2. Add a Redirect URL in the app's settings. Any URL is valid, for example: `http://localhost:8888`
3. Create a `config.toml` file and fill the `client_id` and `client_secret` fields
4. Run the app. It will prompt you to authenticate. Copy the URL that was printed, replace `your-app-redirect-uri` 
   with the Redirect URL that was configured earlier, and login. Once you got redirected, copy the URL and paste
   it in the console.

## Configuration

Example config:

```
[spotify]
client_id = "your-app's-client-id"
client_secret = "your-app's-client-secret"

[mpd]
address = "127.0.0.1"
port = 6600

[cover]
preferred_size = "300x300" # usually, the ones available are: 600x600, 300x300, 64x64
output_path = "/tmp/mpd-spotify-cover.png"
```
