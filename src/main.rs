extern crate google_photos;

use google_photos::gphotos::{self};
use google_photos::WallflowerError;

fn main() -> Result<(), WallflowerError> {
    let client = gphotos::Client::new();
    let _ = client.load_access_token();
    Ok(())
}