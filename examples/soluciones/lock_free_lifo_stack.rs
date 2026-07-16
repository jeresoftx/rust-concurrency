use rust_concurrency::lock_free::BoundedLockFreeStack;

fn main() {
    let stack = BoundedLockFreeStack::new(3);

    stack.push(10).unwrap();
    stack.push(20).unwrap();
    stack.push(30).unwrap();

    assert_eq!(stack.pop(), Some(30));
    assert_eq!(stack.pop(), Some(20));
    assert_eq!(stack.pop(), Some(10));
    assert_eq!(stack.pop(), None);
}
