use rust_concurrency::arc::Shared;

fn main() {
    let shared = Shared::new(String::from("Jeresoft Academy"));
    let cloned = shared.clone_shared();

    println!("valor: {}", shared.with_ref(|value| value.clone()));
    println!("strong_count: {}", shared.strong_count());

    drop(cloned);
    println!("strong_count después de drop: {}", shared.strong_count());
}
