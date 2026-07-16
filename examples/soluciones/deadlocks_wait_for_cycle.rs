use rust_concurrency::deadlocks::WaitForGraph;

fn main() {
    let mut graph = WaitForGraph::new();

    graph.add_wait("a", "b");
    graph.add_wait("b", "c");
    graph.add_wait("c", "a");

    assert!(graph.has_cycle());
    println!("ciclo detectado: {:?}", graph.cycle_path().unwrap());
}
