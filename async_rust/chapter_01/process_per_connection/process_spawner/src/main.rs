use tokio::process::Command;
use tokio::join;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut handles = vec![];

    // Define the number of processes you want to run
    let num_processes = 4;

    // Spawn the processes in a loop
    for _ in 0..num_processes {
        let handle = tokio::spawn(async {
            let output = Command::new("../connection_bin")
                .output()
                .await;

            match output {
                Ok(output) => {
                    println!("Process completed with output: {}", String::from_utf8_lossy(&output.stdout));
                    Ok(output.status.code().unwrap_or(-1))  // Return the exit code
                }
                Err(e) => {
                    eprintln!("Failed to run process: {}", e);
                    Err(e)
                }
            }
        });
        handles.push(handle);
    }

    let mut results = Vec::with_capacity(handles.len());
    for handle in handles {
        results.push(handle.await.unwrap());
    }

    // Handle the results
    for (i, result) in results.into_iter().enumerate() {
        match result {
            Ok(exit_code) => println!("Process {} exited with code {}", i + 1, exit_code),
            Err(e) => eprintln!("Process {} failed: {}", i + 1, e),
        }
    }

    Ok(())
}
