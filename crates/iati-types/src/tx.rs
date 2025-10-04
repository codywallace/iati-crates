use crate::OrgRef;
use crate::money::{CurrencyCode, Money};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// IATI Transaction Type (from Transaction/@type).
/// See https://iatistandard.org/en/iati-standard/activities/transaction/#transaction-type
/// Unknown/extension codes are preserved via 'Unknown(u16)' variant.
/// Note that 'Transaction/@type' is distinct from 'Transaction/transaction-type'.
/// The latter is a child element with its own codelist (see TxType enum).
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TxType {
    IncomingFunds,      // 1
    OutgoingCommitment, // 2
    Disbursement,       // 3
    Expenditure,        // 4
    InterestPayment,    // 5
    LoanRepayment,      // 6
    Reimbursement,      // 7
    PurchaseOfEquity,   // 8
    SaleOfEquity,       // 9
    CreditGuarantee,    // 10
    IncomingCommitment, // 11
    OutgoingPledge,     // 12
    IncomingPledge,     // 13
    Unknown(u16),
}

impl TxType {
    /// Return the IATI code for this TxType.
    pub fn code(self) -> u16 {
        use TxType::*;
        match self {
            IncomingFunds => 1,
            OutgoingCommitment => 2,
            Disbursement => 3,
            Expenditure => 4,
            InterestPayment => 5,
            LoanRepayment => 6,
            Reimbursement => 7,
            PurchaseOfEquity => 8,
            SaleOfEquity => 9,
            CreditGuarantee => 10,
            IncomingCommitment => 11,
            OutgoingPledge => 12,
            IncomingPledge => 13,
            Unknown(c) => c,
        }
    }
}

impl std::fmt::Display for TxType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code())
    }
}

impl From<u16> for TxType {
    fn from(code: u16) -> Self {
        match code {
            1 => TxType::IncomingFunds,
            2 => TxType::OutgoingCommitment,
            3 => TxType::Disbursement,
            4 => TxType::Expenditure,
            5 => TxType::InterestPayment,
            6 => TxType::LoanRepayment,
            7 => TxType::Reimbursement,
            8 => TxType::PurchaseOfEquity,
            9 => TxType::SaleOfEquity,
            10 => TxType::CreditGuarantee,
            11 => TxType::IncomingCommitment,
            12 => TxType::OutgoingPledge,
            13 => TxType::IncomingPledge,
            c => TxType::Unknown(c),
        }
    }
}

impl FromStr for TxType {
    type Err = std::num::ParseIntError; // parsing from string to u16 may fail
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let n: u16 = s.trim().parse()?;
        Ok(TxType::from(n))
    }
}

/// Minimal transaction spine for rollups & FX.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Transaction {
    pub tx_type: TxType, // from transaction-type/@code
    pub date: NaiveDate, // from 'transaction-date/@iso-date'
    pub value: Money,    // from <value> body + attributes
    pub provider_org: Option<OrgRef>,
    pub receiver_org: Option<OrgRef>,
    /// optional hint for a resolved currency if caller has done FX lookup
    pub currency_hint: Option<CurrencyCode>,
}

impl Transaction {
    pub fn new(tx_type: TxType, date: NaiveDate, value: Money) -> Self {
        Self {
            tx_type,
            date,
            value,
            provider_org: None,
            receiver_org: None,
            currency_hint: None,
        }
    }

    /// Set provider organisation (with builder helpers).
    pub fn with_provider(mut self, org: OrgRef) -> Self {
        self.provider_org = Some(org);
        self
    }

    /// Set receiver organisation (with builder helpers).
    pub fn with_receiver(mut self, org: OrgRef) -> Self {
        self.receiver_org = Some(org);
        self
    }

    /// Set a pre-resolved currency hint (builder-style).
    pub fn with_currency_hint(mut self, code: CurrencyCode) -> Self {
        self.currency_hint = Some(code);
        self
    }
}
