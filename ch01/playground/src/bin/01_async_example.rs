use reqwest::Error;
use std::time::Instant;

#[tokio::main()]
async fn main() -> Result<(), Error> {
    let url = "https://www.microsoft.com";

    let start_time = Instant::now();

    let req1 = reqwest::get(url);
    let req2 = reqwest::get(url);
    let req3 = reqwest::get(url);
    let req4 = reqwest::get(url);

    let _ = tokio::join!(req1, req2, req3, req4);
    let elapsed_time = start_time.elapsed();
    println!("Request took: {} ms", elapsed_time.as_millis());
    Ok(())
}
