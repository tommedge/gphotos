extern crate reqwest;
extern crate serde;
extern crate serde_json;

use self::reqwest::Url;

use std::io;
use std::fs::{File};
use std::env;

use error::FlickrError;

const GPHOTOS_DATA_FILE: &str = ".gphotos-data.json";

#[derive(Debug)]
pub struct Client {

}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResult {
    access_token: String,
    refresh_token: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientDetails {
    client_id: String,
    client_secret: String
}

impl Client {
    pub fn new() -> Self {
        Client {

        }
    }

    pub fn load_access_token(self) -> Result<TokenResult, FlickrError> {
        match File::open(GPHOTOS_DATA_FILE) {
            Ok(file) => {
                let token_result: TokenResult = serde_json::from_reader(file)?;
                println!("Loaded token from file\n {:?}", token_result);
                Ok(token_result)
            }
            Err(e) => {
                println!("{:?}", e);
                let token_result = self.authenticate()?;

                // Save app data for using on the next run.
                let file = File::create(GPHOTOS_DATA_FILE)?;
                let _ = serde_json::to_writer_pretty(file, &token_result)?;

                Ok(token_result)
            }
        }
    }

    /// Perform an OAuth 1.0 authentication flow to obtain an access token
    fn authenticate(self) -> Result<TokenResult, FlickrError> {
        let client_id = env::var("GPHOTOS_CLIENT_ID").expect("GPHOTOS_CLIENT_ID must be set");
        let client_secret = env::var("GPHOTOS_CLIENT_SECRET").expect("GPHOTOS_CLIENT_SECRET must be set");
        let redirect_uri = "urn:ietf:wg:oauth:2.0:oob";
        let response_type = "code";
        let scope = "https://www.googleapis.com/auth/photoslibrary.readonly";

        let authorization_params = [
            ("client_id", client_id.to_string()),
            ("redirect_uri", redirect_uri.to_string()),
            ("response_type", response_type.to_string()),
            ("scope", scope.to_string())
        ];
        let authorization_url = Url::parse_with_params(
            "https://accounts.google.com/o/oauth2/auth",
            &authorization_params,
        ).expect("unable to parse authorization_url");

        println!(
            "Visit this url in your browser to authorize the application:\n\n{}",
            authorization_url
        );

        let mut verification_code = String::new();
        while verification_code.trim().is_empty() {
            print!("\nEnter the code: ");
            io::stdin()
                .read_line(&mut verification_code)
                .map_err(|_err| FlickrError::AuthenticationError)?;
        }

        let token_params = [
            ("code", verification_code),
            ("client_id", client_id.to_string()),
            ("client_secret", client_secret.to_string()),
            ("redirect_uri", redirect_uri.to_string()),
            ("grant_type", "authorization_code".to_string())
        ];

        let token_url = "https://www.googleapis.com/oauth2/v4/token";

        let client = reqwest::Client::new();
        let mut res = client.post(token_url).form(&token_params)
            .send().expect("failed to send request");
        //let body = res.text().expect("failed to deserialize body");

        println!("Status: {}", res.status());
        println!("Headers:\n{}", res.headers());
       // println!("Body:\n{}", body);

        // TODO persist the refresh token
        let result: TokenResult = res.json()?;

        println!("Result {:?}", result.access_token);

        let media_items_url = Url::parse_with_params(
            "https://photoslibrary.googleapis.com/v1/mediaItems",
            &[("access_token", result.access_token.to_string())],
        ).expect("unable to parse authorization_url");

        let mut media_items = client.get(media_items_url).send().expect("failed to retrieve media items");

        let body = media_items.text().expect("failed to deserialize body");

        println!("{}", body);

        Ok(result)
    }
}
