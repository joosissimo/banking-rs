use std::str::FromStr;

use anyhow::Result;
use thiserror::Error;

use crate::account::{Account, Cents};

#[derive(Error, Debug, Clone, PartialEq)]
pub enum BankingSystemError {
    #[error("account with name {0} already exists")]
    DuplicateAccountName(String),
    #[error("account with name {0} not found")]
    AccountNotFound(String),
    #[error("invalid amount {0:?}, must be a non-negative number only containing digits up to two decimal places")]
    InvalidAmount(String),
    #[error("amount {0} would overflow")]
    AmountOverflow(String),
}

/// System to process user input and execute the specified command.
#[derive(Debug, Clone)]
pub struct BankingSystem(pub Vec<Account>);

impl BankingSystem {
    pub fn show(&self) {
        for account in self.0.iter() {
            println!("{account}");
        }
    }

    fn account_exists(&self, name: &str) -> bool {
        self.0.iter().any(|x| x.name == name)
    }

    fn get_account_mut(&mut self, name: &str) -> Result<&mut Account, BankingSystemError> {
        self.0
            .iter_mut()
            .find(|x| x.name == name)
            .ok_or(BankingSystemError::AccountNotFound(name.to_owned()))
    }

    pub fn create(&mut self, name: &str, balance: &str) -> Result<()> {
        if self.account_exists(name) {
            return Err(BankingSystemError::DuplicateAccountName(name.to_owned()).into());
        }

        match Account::new(name.to_owned(), balance.parse()?) {
            Ok(account) => {
                println!(
                    "Account created with name {} and balance {}",
                    account.name, account.balance
                );
                self.0.push(account);
                Ok(())
            },
            Err(account) => Err(account.into()),
        }
    }

    pub fn deposit(&mut self, name: &str, amount: &str) -> Result<()> {
        let account = self.get_account_mut(name)?;

        match account.deposit(amount.parse()?) {
            Ok(account) => {
                println!("Account balance is now {}", account.balance);
                Ok(())
            },
            Err(account) => Err(account.into()),
        }
    }

    pub fn withdraw(&mut self, name: &str, amount: &str) -> Result<()> {
        let account = self.get_account_mut(name)?;

        match account.withdraw(amount.parse()?) {
            Ok(account) => {
                println!("Account balance is now {}", account.balance);
                Ok(())
            },
            Err(account) => Err(account.into()),
        }
    }

    pub fn transfer(&mut self, from: &str, to: &str, amount: &str) -> Result<()> {
        let mut cloned_system = self.clone();
        let cloned_from = cloned_system.get_account_mut(from)?;
        let orig_to = self.get_account_mut(to)?;
        let amount = Cents::from_str(amount)?;

        // HACK: To perform transfer atomically without needing two mutable references to self
        // If withdrawal on cloned and actual deposit are successful, perform actual withdrawal on orig.
        cloned_from.withdraw(amount)?;
        orig_to.deposit(amount)?;

        let from_balance = cloned_from.balance;
        let to_balance = orig_to.balance;

        self.get_account_mut(from)
            .expect("from account should be found")
            .withdraw(amount)
            .expect("transfer withdrawal should succeed");

        println!("{from} balance is now {from_balance}, {to} balance is now {to_balance}");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account::AccountError;

    const DEFAULT_NAME: &str = "user";

    #[test]
    fn test_account_exists() {
        let bs = BankingSystem(Vec::from([Account {
            name: DEFAULT_NAME.to_owned(),
            balance: Cents(20),
        }]));

        assert!(bs.account_exists(DEFAULT_NAME));
        assert!(!bs.account_exists("user1"));
    }

    #[test]
    fn test_get_account_mut() {
        let account = Account {
            name: DEFAULT_NAME.to_owned(),
            balance: Cents(20),
        };
        let mut bs = BankingSystem(Vec::from([account]));

        assert_eq!(bs.get_account_mut(DEFAULT_NAME).unwrap().name, DEFAULT_NAME);
        assert_eq!(
            bs.get_account_mut("user1"),
            Err(BankingSystemError::AccountNotFound(String::from("user1")))
        );
    }

    #[test]
    fn test_create_duplicate_account_name() {
        let mut bs = BankingSystem(Vec::from([Account {
            name: DEFAULT_NAME.to_owned(),
            balance: Cents(20),
        }]));

        assert_eq!(
            bs.create(DEFAULT_NAME, "1000")
                .unwrap_err()
                .downcast::<BankingSystemError>()
                .unwrap(),
            BankingSystemError::DuplicateAccountName(DEFAULT_NAME.to_owned())
        );
    }

    #[test]
    fn test_create_account_success() {
        let mut bs = BankingSystem(Vec::new());
        bs.create(DEFAULT_NAME, "20").unwrap();

        assert!(bs.account_exists(DEFAULT_NAME));
    }

    #[test]
    fn test_create_account_failure() {
        let mut bs = BankingSystem(Vec::new());

        assert_eq!(
            bs.create("", "20")
                .unwrap_err()
                .downcast::<AccountError>()
                .unwrap(),
            AccountError::EmptyAccountName
        );
    }

    #[test]
    fn test_deposit_success() {
        let mut bs = BankingSystem(Vec::from([Account {
            name: DEFAULT_NAME.to_owned(),
            balance: Cents(20),
        }]));
        bs.deposit(DEFAULT_NAME, "20").unwrap();

        assert_eq!(
            bs.get_account_mut(DEFAULT_NAME).unwrap().balance,
            Cents(2020)
        );
    }

    #[test]
    fn test_deposit_failure() {
        let mut bs = BankingSystem(Vec::from([Account {
            name: DEFAULT_NAME.to_owned(),
            balance: Cents(u64::MAX),
        }]));

        assert_eq!(
            bs.deposit(DEFAULT_NAME, "2")
                .unwrap_err()
                .downcast::<AccountError>()
                .unwrap(),
            AccountError::BalanceOverflow {
                name: DEFAULT_NAME.to_owned(),
                deposit_amount: Cents(200)
            }
        );
    }

    #[test]
    fn test_withdraw_success() {
        let mut bs = BankingSystem(Vec::from([Account {
            name: DEFAULT_NAME.to_owned(),
            balance: Cents(2000),
        }]));
        bs.withdraw(DEFAULT_NAME, "20").unwrap();

        assert_eq!(bs.get_account_mut(DEFAULT_NAME).unwrap().balance, Cents(0));
    }

    #[test]
    fn test_withdraw_failure() {
        let mut bs = BankingSystem(Vec::from([Account {
            name: DEFAULT_NAME.to_owned(),
            balance: Cents(2),
        }]));

        assert_eq!(
            bs.withdraw(DEFAULT_NAME, "2")
                .unwrap_err()
                .downcast::<AccountError>()
                .unwrap(),
            AccountError::AccountOverdraft {
                name: DEFAULT_NAME.to_owned(),
                balance: Cents(2),
                withdraw_amount: Cents(200)
            }
        );
    }

    #[test]
    fn test_transfer_success() {
        let mut bs = BankingSystem(Vec::from([
            Account {
                name: String::from("user1"),
                balance: Cents(2000),
            },
            Account {
                name: String::from("user2"),
                balance: Cents(1000),
            },
        ]));
        bs.transfer("user1", "user2", "10").unwrap();

        assert_eq!(bs.get_account_mut("user1").unwrap().balance, Cents(1000));
        assert_eq!(bs.get_account_mut("user2").unwrap().balance, Cents(2000));
    }

    #[test]
    fn test_transfer_failure() {
        let mut bs = BankingSystem(Vec::from([
            Account {
                name: String::from("user1"),
                balance: Cents(2000),
            },
            Account {
                name: String::from("user2"),
                balance: Cents(u64::MAX),
            },
        ]));

        // test failed withdrawal
        assert_eq!(
            bs.transfer("user1", "user2", "30")
                .unwrap_err()
                .downcast::<AccountError>()
                .unwrap(),
            AccountError::AccountOverdraft {
                name: String::from("user1"),
                balance: Cents(2000),
                withdraw_amount: Cents(3000)
            }
        );
        assert_eq!(bs.get_account_mut("user1").unwrap().balance, Cents(2000));
        assert_eq!(
            bs.get_account_mut("user2").unwrap().balance,
            Cents(u64::MAX)
        );

        // test failed deposit
        assert_eq!(
            bs.transfer("user1", "user2", "10")
                .unwrap_err()
                .downcast::<AccountError>()
                .unwrap(),
            AccountError::BalanceOverflow {
                name: String::from("user2"),
                deposit_amount: Cents(1000)
            }
        );
        assert_eq!(bs.get_account_mut("user1").unwrap().balance, Cents(2000));
        assert_eq!(
            bs.get_account_mut("user2").unwrap().balance,
            Cents(u64::MAX)
        );
    }
}
