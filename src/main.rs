use essentials::info;

fn main() {
    essentials::install();
    let result = gateway::add(2, 2);
    info!("Result: {}", result);
}
