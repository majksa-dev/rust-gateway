use essentials::tracing::info;

fn main() {
    essentials::install();
    let result = rust_lib::add(2, 2);
    info!("Result: {}", result);
}
