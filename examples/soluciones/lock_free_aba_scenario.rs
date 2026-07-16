use rust_concurrency::lock_free::AbaScenario;

fn main() {
    let scenario = AbaScenario::new(3, 1, 3);

    assert!(scenario.is_aba_risk());
    println!("{}", scenario.description());
}
