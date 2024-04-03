trait Greeting {
    fn greet(&self) -> String;
}

// Concrete Component
struct HelloWorld;
impl Greeting for HelloWorld {
    fn greet(&self) -> String {
        "Hello, World!".to_string()
    }
}

// Decorator Component
struct ExcitedGreeting<T: Greeting> {
    inner: T,
}

impl<T: Greeting> Greeting for ExcitedGreeting<T> {
    fn greet(&self) -> String {
        let mut greeting = self.inner.greet();
        greeting.push_str(" I'm so excited to be in Rust!");
        greeting
    }
}

// Usage
fn main() {

    #[cfg(feature = "logging_decorator")]
    let hello = ExcitedGreeting { inner: HelloWorld };

    #[cfg(not(feature = "logging_decorator"))]
    let hello = HelloWorld;

    println!("{}", hello.greet());
}
