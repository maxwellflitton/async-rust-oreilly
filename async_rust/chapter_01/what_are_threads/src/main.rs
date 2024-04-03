use std::time::Instant;
use std::thread;


fn fibonacci(n: u64) -> u64 {
    if n == 0 || n == 1 {
        return n;
    }
    fibonacci(n-1) + fibonacci(n-2)
}


fn main() {
    let start = Instant::now();
    let mut handles = vec![];
    for _ in 0..4 {
        let handle = thread::spawn(|| {
            fibonacci(50)
        });
        handles.push(handle);
    }
    let duration = start.elapsed();
    println!("fibonacci(50) in {:?}", duration);

}
