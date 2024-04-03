

type WaterFallResult = Result<String, Box<dyn std::error::Error>>;

async fn task1() -> WaterFallResult {
    Ok("Task 1 completed".to_string())
}
async fn task2(input: String) -> WaterFallResult {
    Ok(format!("{} then Task 2 completed", input))
}
async fn task3(input: String) -> WaterFallResult {
    Ok(format!("{} and finally Task 3 completed", input))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output1 = task1().await?;
    let output2 = task2(output1).await?;
    let result = task3(output2).await?;
    println!("{}", result);
    Ok(())
}
