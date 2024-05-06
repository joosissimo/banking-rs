# banking-rs

A simple banking system

## Building
Tested on Linux only
1. Install Rust stable (https://www.rust-lang.org/tools/install)
2. In the project root, run `cargo build --release`

## Running
All commands should be run in the project root. Data will be read from and written to `banking_system.csv` in the current working directory.

List all commands:

`target/release/banking-rs`

Show all accounts:

`target/release/banking-rs show`

Create account:

`target/release/banking-rs create -n user1 -a 10`

Deposit to account:

`target/release/banking-rs deposit -n user1 -a 0.01`

Withdraw from account:

`target/release/banking-rs withdraw -n user1 -a 0.2`

Transfer from one account to another:

`target/release/banking-rs transfer -f user1 -t user2 -a 10`
