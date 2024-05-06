use std::fmt::Display;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::banking_system::BankingSystemError;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum AccountError {
    #[error(
        "account {name} would overdraft if {withdraw_amount} was withdrawn from balance {balance}"
    )]
    AccountOverdraft {
        name: String,
        balance: Cents,
        withdraw_amount: Cents,
    },
    #[error("account {name} would have balance overflow if {deposit_amount} was deposited")]
    BalanceOverflow { name: String, deposit_amount: Cents },
    #[error("account name cannot not be empty")]
    EmptyAccountName,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cents(pub(crate) u64);

/// Display Cents as base currency unit.
impl Display for Cents {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "${}.{:02}", self.0 / 100, self.0 % 100)
    }
}

/// Parse a string into Cents. The string represents a non-negative number up to two decimal places.
/// The string must only contain digits and up to one period as a decimal separator.
impl FromStr for Cents {
    type Err = BankingSystemError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Has no decimal part
        if !s.contains('.') {
            let num = s
                .parse::<u64>()
                .map_err(|_| BankingSystemError::InvalidAmount(s.to_owned()))?
                .checked_mul(100)
                .ok_or(BankingSystemError::AmountOverflow(s.to_owned()))?;
            return Ok(Self(num));
        }

        let (int_part_str, dec_part_str) = s
            .rsplit_once('.')
            .expect("from_str string should have period");

        let integer_part = if int_part_str.chars().count() < 1 {
            0 // Number has no leading zero
        } else {
            int_part_str
                .parse::<u64>()
                .map_err(|_| BankingSystemError::InvalidAmount(s.to_owned()))?
        };

        // Invalid decimal part length
        if dec_part_str.chars().count() < 1 || dec_part_str.chars().count() > 2 {
            return Err(BankingSystemError::InvalidAmount(s.to_owned()));
        }
        let decimal_part = dec_part_str
            .parse::<u64>()
            .map_err(|_| BankingSystemError::InvalidAmount(s.to_owned()))?;
        let decimal_part = if dec_part_str.chars().count() == 1 {
            decimal_part * 10 // Decimal part only goes to the tenth place
        } else {
            decimal_part
        };

        let num = integer_part
            .checked_mul(100)
            .ok_or(BankingSystemError::AmountOverflow(s.to_owned()))?
            .checked_add(decimal_part)
            .ok_or(BankingSystemError::AmountOverflow(s.to_owned()))?;

        Ok(Self(num))
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Account {
    pub(crate) name: String,
    pub(crate) balance: Cents,
}

impl Display for Account {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "name: {}\tbalance: {}", self.name, self.balance)
    }
}

impl Account {
    pub(crate) fn new(name: String, balance: Cents) -> Result<Self, AccountError> {
        if name.chars().count() < 1 {
            return Err(AccountError::EmptyAccountName);
        }
        Ok(Self { name, balance })
    }

    pub(crate) fn deposit(&mut self, amount: Cents) -> Result<&mut Self, AccountError> {
        self.balance.0 =
            self.balance
                .0
                .checked_add(amount.0)
                .ok_or(AccountError::BalanceOverflow {
                    name: self.name.to_owned(),
                    deposit_amount: amount,
                })?;
        Ok(self)
    }

    pub(crate) fn withdraw(&mut self, amount: Cents) -> Result<&mut Self, AccountError> {
        self.balance.0 =
            self.balance
                .0
                .checked_sub(amount.0)
                .ok_or(AccountError::AccountOverdraft {
                    name: self.name.to_owned(),
                    balance: self.balance,
                    withdraw_amount: amount,
                })?;
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEFAULT_NAME: &str = "user";

    #[test]
    fn test_empty_account_name() {
        assert_eq!(
            Account::new(String::from(""), Cents(23)),
            Err(AccountError::EmptyAccountName)
        );
    }

    #[test]
    fn test_balance_overflow() {
        assert_eq!(
            Account::new(DEFAULT_NAME.to_owned(), Cents(u64::MAX - 1))
                .unwrap()
                .deposit(Cents(200)),
            Err(AccountError::BalanceOverflow {
                name: DEFAULT_NAME.to_owned(),
                deposit_amount: Cents(200)
            })
        );
    }

    #[test]
    fn test_account_overdraft() {
        assert_eq!(
            Account::new(DEFAULT_NAME.to_owned(), Cents(2))
                .unwrap()
                .withdraw(Cents(10)),
            Err(AccountError::AccountOverdraft {
                name: DEFAULT_NAME.to_owned(),
                balance: Cents(2),
                withdraw_amount: Cents(10)
            })
        );
    }

    #[test]
    fn test_deposit() {
        assert_eq!(
            Account::new(DEFAULT_NAME.to_owned(), Cents(20))
                .unwrap()
                .deposit(Cents(100))
                .unwrap()
                .balance,
            Cents(120)
        );
    }

    #[test]
    fn test_withdraw() {
        assert_eq!(
            Account::new(DEFAULT_NAME.to_owned(), Cents(120))
                .unwrap()
                .withdraw(Cents(100))
                .unwrap()
                .balance,
            Cents(20)
        );
    }

    #[test]
    fn test_parse_to_cents_no_decimal() {
        assert_eq!(Cents::from_str("0").unwrap(), Cents(0));
        assert_eq!(Cents::from_str("2").unwrap(), Cents(200));
        assert_eq!(Cents::from_str("30").unwrap(), Cents(3000));
        assert_eq!(
            Cents::from_str("-2"),
            Err(BankingSystemError::InvalidAmount("-2".to_owned()))
        );
        assert_eq!(
            Cents::from_str("2a"),
            Err(BankingSystemError::InvalidAmount("2a".to_owned()))
        );
        assert_eq!(
            Cents::from_str("wef"),
            Err(BankingSystemError::InvalidAmount("wef".to_owned()))
        );
        assert_eq!(
            Cents::from_str(u64::MAX.to_string().as_str()),
            Err(BankingSystemError::AmountOverflow(u64::MAX.to_string()))
        );
    }

    #[test]
    fn test_parse_to_cents_has_decimal() {
        assert_eq!(Cents::from_str(".0").unwrap(), Cents(0));
        assert_eq!(Cents::from_str(".02").unwrap(), Cents(2));
        assert_eq!(Cents::from_str(".2").unwrap(), Cents(20));
        assert_eq!(Cents::from_str("0.0").unwrap(), Cents(0));
        assert_eq!(Cents::from_str("0.00").unwrap(), Cents(0));
        assert_eq!(Cents::from_str("1.00").unwrap(), Cents(100));
        assert_eq!(Cents::from_str("1.02").unwrap(), Cents(102));
        assert_eq!(Cents::from_str("3.1").unwrap(), Cents(310));
        assert_eq!(Cents::from_str("30.2").unwrap(), Cents(3020));
        assert_eq!(Cents::from_str("40.02").unwrap(), Cents(4002));
        assert_eq!(Cents::from_str("40.12").unwrap(), Cents(4012));
        assert_eq!(Cents::from_str("40.20").unwrap(), Cents(4020));
        assert_eq!(Cents::from_str("50.99").unwrap(), Cents(5099));
        assert_eq!(Cents::from_str(".1").unwrap(), Cents(10));
        assert_eq!(
            Cents::from_str("-0.0"),
            Err(BankingSystemError::InvalidAmount("-0.0".to_owned()))
        );
        assert_eq!(
            Cents::from_str("-1.0"),
            Err(BankingSystemError::InvalidAmount("-1.0".to_owned()))
        );
        assert_eq!(
            Cents::from_str("1."),
            Err(BankingSystemError::InvalidAmount("1.".to_owned()))
        );
        assert_eq!(
            Cents::from_str("2.002"),
            Err(BankingSystemError::InvalidAmount("2.002".to_owned()))
        );
        assert_eq!(
            Cents::from_str(".002"),
            Err(BankingSystemError::InvalidAmount(".002".to_owned()))
        );
        assert_eq!(
            Cents::from_str("1.1.2"),
            Err(BankingSystemError::InvalidAmount("1.1.2".to_owned()))
        );
        assert_eq!(
            Cents::from_str(".1.2"),
            Err(BankingSystemError::InvalidAmount(".1.2".to_owned()))
        );
        assert_eq!(
            Cents::from_str(".1a"),
            Err(BankingSystemError::InvalidAmount(".1a".to_owned()))
        );
        assert_eq!(
            Cents::from_str("a.2"),
            Err(BankingSystemError::InvalidAmount("a.2".to_owned()))
        );
        assert_eq!(
            Cents::from_str("..2"),
            Err(BankingSystemError::InvalidAmount("..2".to_owned()))
        );
        let int_part_overflow = u64::MAX.to_string() + ".1";
        assert_eq!(
            Cents::from_str(int_part_overflow.as_str()),
            Err(BankingSystemError::AmountOverflow(int_part_overflow))
        );
        let dec_part_overflow = (u64::MAX - 1).to_string() + ".9";
        assert_eq!(
            Cents::from_str(dec_part_overflow.as_str()),
            Err(BankingSystemError::AmountOverflow(dec_part_overflow))
        );
    }

    #[test]
    fn test_display_cents() {
        assert_eq!(Cents(0).to_string(), "$0.00");
        assert_eq!(Cents(9).to_string(), "$0.09");
        assert_eq!(Cents(10).to_string(), "$0.10");
        assert_eq!(Cents(12).to_string(), "$0.12");
        assert_eq!(Cents(99).to_string(), "$0.99");
        assert_eq!(Cents(100).to_string(), "$1.00");
        assert_eq!(Cents(109).to_string(), "$1.09");
        assert_eq!(Cents(199).to_string(), "$1.99");
        assert_eq!(Cents(4023).to_string(), "$40.23");
        assert_eq!(Cents(5000).to_string(), "$50.00");
    }
}
