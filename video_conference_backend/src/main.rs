use video_conference_backend::config;
use video_conference_backend::signaling;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = config::get_signaling_server_addr();
    signaling::run_signaling_server(addr).await
}