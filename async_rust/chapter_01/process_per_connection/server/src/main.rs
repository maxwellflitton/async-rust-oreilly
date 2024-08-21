use std::process::{Command, Output};


fn main() {
    // Replace `./path_to_your_binary` with the actual path to your compiled binary
    let output: Output = Command::new("./connection_bin")
        .output()
        .expect("Failed to execute command");

    if output.status.success() {
        // Capture the standard output and print it
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("Output: {}", stdout);
    } else {
        // Capture the standard error and print it if there's an error
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Error: {}", stderr);
    }
}
