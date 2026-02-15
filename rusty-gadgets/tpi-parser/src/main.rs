use kernel_files::validate_context_type;

fn main() {
    let valid_context = "unconfined_u:unconfined_r:umrs_data_t:s0";
    let malformed_context = "unconfined_u:unconfined_r"; // Missing Type

    println!("Testing Valid Context...");
    match validate_context_type(valid_context) {
        Ok(t) => println!("Result: SUCCESS (Type: {})", t),
        Err(e) => println!("Result: FAILED ({})", e),
    }

    println!("\nTesting Malformed Context...");
    match validate_context_type(malformed_context) {
        Ok(t) => println!("Result: SUCCESS (Type: {})", t),
        Err(e) => println!("Result: FAILED ({})", e), // Both will fail here
    }
}
