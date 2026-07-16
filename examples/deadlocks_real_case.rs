use rust_concurrency::deadlocks::BankAccounts;

fn main() {
    let accounts = BankAccounts::new([100, 25, 0]);

    accounts.transfer_ordered(0, 2, 40).unwrap();
    accounts.transfer_ordered(1, 0, 10).unwrap();

    println!("cuenta 0: {}", accounts.balance(0));
    println!("cuenta 1: {}", accounts.balance(1));
    println!("cuenta 2: {}", accounts.balance(2));
    println!("último orden de locks: {:?}", accounts.last_lock_order());
}
