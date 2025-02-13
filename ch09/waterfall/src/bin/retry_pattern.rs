use std::{error::Error, time::Duration};

async fn get_data() -> Result<String, Box<dyn Error>> {
    Err("Error".into())
}

async fn do_something() -> Result<(), Box<dyn Error>> {
    let mut milliseconds = 1000;
    let total_count = 5;
    let mut count = 0;
    let result: String;

    loop {
        match get_data().await {
            Ok(data) => {
                result = data;
                break;
            }
            Err(e) => {
                println!("Error: {}", e);
                count += 1;
                if count == total_count {
                    return Err(e);
                }
            }
        }

        tokio::time::sleep(Duration::from_millis(milliseconds)).await;
        milliseconds *= 2;
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let outcome = do_something().await;
    println!("{:?}", outcome);
}
