use std::env;



fn main() {
    println!("Starting JASN");

    let args: Vec<String> = env::args().collect();
    println!("args: {:?}", args);

}
