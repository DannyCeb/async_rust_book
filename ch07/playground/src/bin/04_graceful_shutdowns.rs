use std::time::Duration;

async fn cleanup() {
    println!("Cleanup background tast started");
    let mut count = 0;
    loop {
        tokio::signal::ctrl_c().await.unwrap();
        println!("ctrl-c received!");
        count += 1;
        if count > 2 {
            println!("Has matado el proceso manualmente");
            std::process::exit(0);
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let _ = tokio::spawn(cleanup());

    for l in 1..=10 {
        println!(
            "El programa se terminará en {} segundos, puedes matar el proceso manualmente",
            11 - l
        );
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
