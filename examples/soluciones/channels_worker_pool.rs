use rust_concurrency::channels::WorkerPool;

fn main() {
    let pool = WorkerPool::new(3, |value: i32| value * value);

    for value in 1..=5 {
        pool.submit(value).unwrap();
    }

    let mut outputs = pool.shutdown();
    outputs.sort();

    assert_eq!(outputs, vec![1, 4, 9, 16, 25]);
}
