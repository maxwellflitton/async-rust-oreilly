use std::ptr;

struct SelfReferential {
    data: String,
    self_pointer: *const String,
}


impl SelfReferential {
    fn new(data: String) -> SelfReferential {
        let mut sr = SelfReferential {
            data,
            self_pointer: ptr::null(),
        };
        sr.self_pointer = &sr.data as *const String;
        sr
    }

    fn print(&self) {
        unsafe {
            println!("{}", *self.self_pointer);
        }
    }
}


fn main() {
    let mut first = SelfReferential::new("first".to_string());
    let mut second = SelfReferential::new("second".to_string());
    unsafe {
        ptr::swap(&mut first, &mut second);
    }
    first.print();
}