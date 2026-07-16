use rust_concurrency::arc::Shared;

fn main() {
    let shared = Shared::new("curso");
    let cloned = shared.clone_shared();

    println!("strong_count con clon: {}", shared.strong_count());
    drop(cloned);
    println!("strong_count final: {}", shared.strong_count());
}
