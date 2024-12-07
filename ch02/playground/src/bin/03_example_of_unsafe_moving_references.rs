struct SelfReferential {
    data: String,
    self_pointer: *const String,
}

impl SelfReferential {
    fn new(data: String) -> Self {
        let self_pointer = &data as *const String;
        Self { data, self_pointer }
    }

    fn print(&self) {
        unsafe { println!("{}", *self.self_pointer) }
    }
}

fn main() {
    let mut first = SelfReferential::new("Daniel".to_string());

    first.print();

    first.data = "Genaro".to_string();

    first.print();
}
