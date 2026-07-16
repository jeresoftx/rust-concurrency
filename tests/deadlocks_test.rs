use rust_concurrency::deadlocks::{
    BankAccounts, LockOrderTracker, LockOrderViolation, LockRank, WaitForGraph,
};

#[test]
fn lock_order_tracker_accepts_increasing_ranks() {
    let mut tracker = LockOrderTracker::new();
    let account = LockRank::new(10, "account");
    let ledger = LockRank::new(20, "ledger");

    assert_eq!(tracker.enter(account), Ok(()));
    assert_eq!(tracker.enter(ledger), Ok(()));
    assert_eq!(tracker.held_ranks(), vec![account, ledger]);
}

#[test]
fn lock_order_tracker_rejects_rank_inversion() {
    let mut tracker = LockOrderTracker::new();
    let account = LockRank::new(10, "account");
    let ledger = LockRank::new(20, "ledger");

    tracker.enter(ledger).unwrap();

    assert_eq!(
        tracker.enter(account),
        Err(LockOrderViolation {
            held: ledger,
            requested: account,
        }),
    );
}

#[test]
fn lock_order_tracker_allows_release_and_reenter() {
    let mut tracker = LockOrderTracker::new();
    let account = LockRank::new(10, "account");
    let ledger = LockRank::new(20, "ledger");

    tracker.enter(ledger).unwrap();
    assert_eq!(tracker.exit(ledger), Some(ledger));
    assert_eq!(tracker.enter(account), Ok(()));
    assert_eq!(tracker.held_ranks(), vec![account]);
}

#[test]
fn wait_for_graph_detects_cycle() {
    let mut graph = WaitForGraph::new();

    graph.add_wait("thread-a", "thread-b");
    graph.add_wait("thread-b", "thread-c");
    graph.add_wait("thread-c", "thread-a");

    assert!(graph.has_cycle());
    assert_eq!(
        graph.cycle_path().unwrap(),
        vec!["thread-a", "thread-b", "thread-c", "thread-a"],
    );
}

#[test]
fn wait_for_graph_reports_acyclic_graph() {
    let mut graph = WaitForGraph::new();

    graph.add_wait("thread-a", "thread-b");
    graph.add_wait("thread-b", "thread-c");

    assert!(!graph.has_cycle());
    assert_eq!(graph.cycle_path(), None);
}

#[test]
fn ordered_transfer_moves_money_without_lock_inversion() {
    let accounts = BankAccounts::new([100, 25, 0]);

    accounts.transfer_ordered(0, 2, 40).unwrap();
    accounts.transfer_ordered(1, 0, 10).unwrap();

    assert_eq!(accounts.balance(0), 70);
    assert_eq!(accounts.balance(1), 15);
    assert_eq!(accounts.balance(2), 40);
    assert_eq!(accounts.last_lock_order(), vec![0, 1]);
}

#[test]
fn ordered_transfer_rejects_insufficient_funds() {
    let accounts = BankAccounts::new([10, 0]);

    assert!(accounts.transfer_ordered(0, 1, 11).is_err());
    assert_eq!(accounts.balance(0), 10);
    assert_eq!(accounts.balance(1), 0);
}
