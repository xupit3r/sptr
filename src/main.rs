use std::collections::HashMap;
use std::string::{String, FromUtf8Error};
use std::process::Command;
use serde::{Deserialize, Serialize};
use reqwest;
use tokio;
use dotenv;

/// expected response for our authorization token
#[derive(Serialize, Deserialize, Debug)]
struct AuthToken {
    access_token: String,
    token_type: String,
    expires_in: i32
}

/// expected structure of a device retrieved from
/// Spotify API
#[derive(Serialize, Deserialize, Debug)]
struct Device {
    id: String,
    is_active: bool,
    is_private_session: bool,
    is_restricted: bool,
    name: String, 
    #[serde(rename = "type")]
    kind: String,
    volume_percent: i16
}

#[derive(Serialize, Deserialize, Debug)]
struct Devices {
    devices: Vec<Device>
}

/// retreives/returns a Spotify authorization token
async fn sp_token() -> Result<AuthToken, reqwest::Error> {
    let client_id: String = dotenv::var("CLIENT_ID").unwrap();
    let client_secret: String = dotenv::var("CLIENT_SECRET").unwrap();

    let mut params = HashMap::new();

    // form encoded params for request
    params.insert("grant_type", "client_credentials");
    params.insert("client_id", &client_id);
    params.insert("client_secret", &client_secret);

    let client = reqwest::Client::new();

    client.post("https://accounts.spotify.com/api/token")
          .form(&params)
          .send()
          .await?
          .json::<AuthToken>()
          .await
}

/// returns a future that resolves to a vector of available
/// devices or an error
async fn sp_devices(access_token: &str) -> Result<Devices, reqwest::Error> {
    let client = reqwest::Client::new();

    client.get("https://api.spotify.com/v1/me/player/devices")
        .bearer_auth(access_token)
        .send()
        .await?
        .json::<Devices>()
        .await
}

/// returns spotifyd process ID
fn sp_pid() -> String {
    String::from_utf8(
        Command::new("pgrep")
            .arg("spotifyd")
            .output()
            .expect("Failed to execute command")
            .stdout
    ).expect("get_pid -- invalid UTF-8")
}

/// returns spotifyd instance URI
fn sp_instance(pid: &str) -> String {
    format!("org.mpris.MediaPlayer2.spotifyd.instance{pid}")
}

/// returns a player method URI
fn sp_player(method: &str) -> String {
    format!("org.mpris.MediaPlayer2.Player.{method}")
}

/// returns a URI for the thing we want to do 
fn sp_thing(uri: &str) -> String {
    format!("string:spotify:{uri}")
}

/// sends a dbus messaage and returns the output
/// result of the command
fn dbus_message(instance: &str,
                method_uri: &str,
                thing: &str) -> Result<String, FromUtf8Error> {
    String::from_utf8(
        Command::new("dbus-send")
            .arg("--print-reply")
            .arg(format!("--dest={instance}"))
            .arg("/org/mpris/MediaPlayer2")
            .arg(method_uri)
            .arg(thing.clone())
            .output()
            .expect(format!("dbus-send failed for {thing}").as_str())
            .stdout
    )
}

#[tokio::main]
async fn main() {
    match sp_token().await {
        Ok(atkn) => {
            match sp_devices(&atkn.access_token).await {
                Ok(dvs) => {
                    println!("{:#?}", dvs)
                },
                Err(err) => println!("{:#?}", err)
            }
        },
        Err(err) => println!("{:#?}", err)
    }
}
