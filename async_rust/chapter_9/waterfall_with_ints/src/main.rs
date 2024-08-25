

type WaterFallResult = Result<i32, Box<dyn std::error::Error>>;

async fn task1() -> WaterFallResult {
    Ok(5)
}
async fn task2(input: i32) -> WaterFallResult {
    Ok(input * 2)
}
async fn task3(input: i32) -> WaterFallResult {
    Ok(input - 1)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output1 = task1().await?;
    let output2: i32;
    if output1 > 10 {
        output2 = task2(output1).await?;
    } else {
        output2 = task3(output1).await?;
    }
    println!("{}", output2);
    Ok(())
}
