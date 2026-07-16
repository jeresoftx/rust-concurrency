use rust_concurrency::arc::Shared;

fn main() {
    let weak = {
        let shared = Shared::new(42);
        let weak = shared.downgrade();
        assert!(weak.upgrade().is_some());
        weak
    };

    assert!(weak.upgrade().is_none());
    println!("weak ya no puede recuperar el valor");
}
