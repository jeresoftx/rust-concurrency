use rust_concurrency::deadlocks::WaitForGraph;

fn main() {
    let mut graph = WaitForGraph::new();

    graph.add_wait("thread-a", "thread-b");
    graph.add_wait("thread-b", "thread-c");
    graph.add_wait("thread-c", "thread-a");

    println!("tiene ciclo: {}", graph.has_cycle());
    println!("ciclo: {:?}", graph.cycle_path());
}
