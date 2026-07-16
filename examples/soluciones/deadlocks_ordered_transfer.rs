use rust_concurrency::deadlocks::BankAccounts;

fn main() {
    let accounts = BankAccounts::new([50, 10]);

    accounts.transfer_ordered(0, 1, 15).unwrap();

    assert_eq!(accounts.balance(0), 35);
    assert_eq!(accounts.balance(1), 25);
    println!("orden de locks: {:?}", accounts.last_lock_order());
}
