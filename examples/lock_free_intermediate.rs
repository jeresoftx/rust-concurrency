use rust_concurrency::lock_free::{AbaScenario, ProgressGuarantee};

fn main() {
    let scenario = AbaScenario::new(3, 1, 3);

    println!("garantía: {:?}", ProgressGuarantee::LockFree);
    println!("riesgo ABA: {}", scenario.is_aba_risk());
    println!("{}", scenario.description());
}
