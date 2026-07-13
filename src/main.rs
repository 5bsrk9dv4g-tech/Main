use std::hash::{BuildHasher, Hasher, RandomState};

use tokio_stream::StreamExt;
use wreq::Client;
use wreq_util::{Emulation, Platform, Profile};

const ANCHOR: &[u8] = b"type=\"text/javascript\"  src=\"";
const WINDOW: usize = 8192;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let candidates = [
        (Profile::Chrome149, Platform::Windows),
        (Profile::Chrome149, Platform::MacOS),
        (Profile::Chrome148, Platform::Windows),
        (Profile::Chrome148, Platform::MacOS),
    ];
    let pick = RandomState::new().build_hasher().finish() as usize % candidates.len();
    let (profile, platform) = candidates[pick];

    let client = Client::builder()
        .emulation(Emulation::builder().profile(profile).platform(platform).build())
        .build()?;

    let response = client
        .get("https://www.fedex.com/register/contact")
        .send()
        .await?;
    let mut stream = std::pin::pin!(response.bytes_stream());
    let mut tail = Vec::new();
    while let Some(chunk) = stream.next().await {
        tail.extend_from_slice(&chunk?);
        if tail.len() > WINDOW {
            tail.drain(..tail.len() - WINDOW);
        }
    }
    let start = memchr::memmem::rfind(&tail, ANCHOR).ok_or("anchor")? + ANCHOR.len();
    let end = memchr::memchr(b'"', &tail[start..]).ok_or("delimiter")?;
    let path = std::str::from_utf8(&tail[start..start + end])?;
    println!("{path}");

    Ok(())
}
