// Main function
fn main() {
    println!("VOStuff - Stuff tracking application");
    println!("Use 'cargo run --bin schema-manager' to manage the database schema");

    println!("fibonacci(15) = {}", fibonacci(15));
}

fn fibonacci(n: usize) -> usize {
    n + 1
}
