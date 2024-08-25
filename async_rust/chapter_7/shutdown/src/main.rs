


async fn cleanup() {
    println!("cleanup background task started");
    let mut count = 0;
    loop {
        std::thread::sleep(std::time::Duration::from_secs(5));
        tokio::signal::ctrl_c().await.unwrap();
        println!("ctrl-c received!");
        count += 1;
        if count > 2 {
            std::process::exit(0);
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    std::thread::spawn(|| {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        runtime.block_on(async {
            println!("Hello, world!");
        });
    });
    let mut count = 0;
    loop {
        tokio::signal::ctrl_c().await.unwrap();
        println!("ctrl-c received!");
        count += 1;
        if count > 2 {
            std::process::exit(0);
        }
    }
}