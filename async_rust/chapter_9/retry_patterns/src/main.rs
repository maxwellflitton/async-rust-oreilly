
async fn get_data() -> Result<String, Box<dyn std::error::Error>> {
    Err("Error".into())
}


async fn do_something() -> Result<(), Box<dyn std::error::Error>> {
    let mut miliseconds = 1000;
    let total_count = 5;
    let mut count = 0;
    let result: String;
    loop {
        match get_data().await {
            Ok(data) => {
                result = data;
                break;
            },
            Err(err) => {
                println!("Error: {}", err);
                count += 1;
                if count == total_count {
                    return Err(err);
                }
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(miliseconds)).await;
        miliseconds *= 2;
    }
    println!("{}", result);
    Ok(())
}


#[tokio::main]
async fn main() {
    let outcome = do_something().await;
    println!("Outcome: {:?}", outcome);
}
