use rust_concurrency::lock_free::BoundedLockFreeStack;

fn main() {
    let stack = BoundedLockFreeStack::new(4);

    stack.push(10).unwrap();
    stack.push(20).unwrap();

    println!("pop: {:?}", stack.pop());
    println!("pop: {:?}", stack.pop());
}
