use vm::MyVM;

pub fn main() {
    let mut vm = match MyVM::new() {
        Ok(vm) => vm,
        Err(err) => {
            println!("Failed to create VM:\n\t{}", err);
            std::process::exit(1);
        }
    };
    
    if let Err(err) = vm.start() {
        println!("Failed to start VM:\n\t{}", err);
        std::process::exit(1);
    };
}
