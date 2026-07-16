use rust_concurrency::arc::Shared;

fn main() {
    let weak = {
        let shared = Shared::new(String::from("temporal"));
        let weak = shared.downgrade();
        println!("upgrade mientras vive: {}", weak.upgrade().is_some());
        weak
    };

    println!("upgrade después de drop: {}", weak.upgrade().is_some());
    println!("observaciones desde weak: {:?}", weak.observations());
}
