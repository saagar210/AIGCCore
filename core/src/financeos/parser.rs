use serde::{Deserialize, Serialize};
use crate::error::{CoreError, CoreResult};
use sha2::{Digest, Sha256};

/// Transaction from financial statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub transaction_id: String,
    pub date: String,          // YYYY-MM-DD
    pub amount: f64,
    pub account: String,
    pub category: String,
    pub description: String,
}

/// Financial statement with transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialStatement {
    pub statement_id: String,
    pub period_start: String,
    pub period_end: String,
    pub transactions: Vec<Transaction>,
    pub summary: StatementSummary,
}

/// Statement summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatementSummary {
    pub total_amount: f64,
    pub transaction_count: usize,
    pub accounts: Vec<String>,
    pub categories: Vec<String>,
    pub date_range: (String, String),
}

/// Parse JSON financial statement
pub fn parse_financial_statement(json_str: &str) -> CoreResult<FinancialStatement> {
    let raw: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| CoreError::InvalidInput(format!("Failed to parse statement: {}", e)))?;

    let statement_id = raw
        .get("statement_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CoreError::InvalidInput("Missing statement_id".to_string()))?
        .to_string();

    let period_start = raw
        .get("period_start")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CoreError::InvalidInput("Missing period_start".to_string()))?
        .to_string();

    let period_end = raw
        .get("period_end")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CoreError::InvalidInput("Missing period_end".to_string()))?
        .to_string();

    let transactions_arr = raw
        .get("transactions")
        .and_then(|v| v.as_array())
        .ok_or_else(|| CoreError::InvalidInput("Missing transactions array".to_string()))?;

    let mut transactions = Vec::new();
    let mut total_amount = 0.0;
    let mut accounts_set = std::collections::HashSet::new();
    let mut categories_set = std::collections::HashSet::new();

    for (idx, tx) in transactions_arr.iter().enumerate() {
        let date = tx
            .get("date")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CoreError::InvalidInput(format!("Missing date at index {}", idx)))?;

        let amount: f64 = tx
            .get("amount")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| CoreError::InvalidInput(format!("Missing/invalid amount at index {}", idx)))?;

        let account = tx
            .get("account")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CoreError::InvalidInput(format!("Missing account at index {}", idx)))?
            .to_string();

        let category = tx
            .get("category")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CoreError::InvalidInput(format!("Missing category at index {}", idx)))?
            .to_string();

        let description = tx
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // Generate deterministic transaction ID
        let tx_id = generate_transaction_id(&statement_id, date, amount, &account, idx);

        transactions.push(Transaction {
            transaction_id: tx_id,
            date: date.to_string(),
            amount,
            account: account.clone(),
            category: category.clone(),
            description,
        });

        total_amount += amount;
        accounts_set.insert(account);
        categories_set.insert(category);
    }

    if transactions.is_empty() {
        return Err(CoreError::InvalidInput("No transactions found".to_string()));
    }

    // Sort by date
    transactions.sort_by(|a, b| a.date.cmp(&b.date));

    let mut accounts: Vec<String> = accounts_set.into_iter().collect();
    accounts.sort();

    let mut categories: Vec<String> = categories_set.into_iter().collect();
    categories.sort();

    let summary = StatementSummary {
        total_amount,
        transaction_count: transactions.len(),
        accounts,
        categories,
        date_range: (transactions[0].date.clone(), transactions[transactions.len() - 1].date.clone()),
    };

    Ok(FinancialStatement {
        statement_id,
        period_start,
        period_end,
        transactions,
        summary,
    })
}

/// Generate deterministic transaction ID
fn generate_transaction_id(statement_id: &str, date: &str, amount: f64, account: &str, idx: usize) -> String {
    let combined = format!("{}{}{}{}{}", statement_id, date, amount.to_bits(), account, idx);
    let mut hasher = Sha256::new();
    hasher.update(combined.as_bytes());
    let hash_bytes = hasher.finalize();
    let hash_hex = hex::encode(&hash_bytes[0..8]);

    format!("FINANCE_{}_{}", account.replace(' ', "_"), hash_hex)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_statement() -> &'static str {
        r#"{
            "statement_id": "STMT_2026_01",
            "period_start": "2026-01-01",
            "period_end": "2026-01-31",
            "transactions": [
                {
                    "date": "2026-01-05",
                    "amount": 1000.00,
                    "account": "checking",
                    "category": "salary",
                    "description": "Monthly salary deposit"
                },
                {
                    "date": "2026-01-10",
                    "amount": 50.00,
                    "account": "checking",
                    "category": "utilities",
                    "description": "Electric bill"
                },
                {
                    "date": "2026-01-15",
                    "amount": 250.00,
                    "account": "savings",
                    "category": "transfer",
                    "description": "Transfer to savings"
                }
            ]
        }"#
    }

    #[test]
    fn test_parse_valid_statement() {
        let stmt = parse_financial_statement(sample_statement()).unwrap();
        assert_eq!(stmt.transactions.len(), 3);
        assert_eq!(stmt.statement_id, "STMT_2026_01");
    }

    #[test]
    fn test_statement_summary() {
        let stmt = parse_financial_statement(sample_statement()).unwrap();
        assert_eq!(stmt.summary.transaction_count, 3);
        assert_eq!(stmt.summary.total_amount, 1300.0);
        assert_eq!(stmt.summary.accounts.len(), 2);
    }

    #[test]
    fn test_transactions_sorted_by_date() {
        let stmt = parse_financial_statement(sample_statement()).unwrap();
        for i in 1..stmt.transactions.len() {
            assert!(stmt.transactions[i - 1].date <= stmt.transactions[i].date);
        }
    }

    #[test]
    fn test_transaction_id_determinism() {
        let stmt1 = parse_financial_statement(sample_statement()).unwrap();
        let stmt2 = parse_financial_statement(sample_statement()).unwrap();

        for (tx1, tx2) in stmt1.transactions.iter().zip(stmt2.transactions.iter()) {
            assert_eq!(tx1.transaction_id, tx2.transaction_id);
        }
    }

    #[test]
    fn test_invalid_json() {
        let invalid = r#"{ invalid json }"#;
        let result = parse_financial_statement(invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_fields() {
        let incomplete = r#"{"statement_id": "STMT_001", "transactions": []}"#;
        let result = parse_financial_statement(incomplete);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_transactions() {
        let no_tx = r#"{
            "statement_id": "STMT_001",
            "period_start": "2026-01-01",
            "period_end": "2026-01-31",
            "transactions": []
        }"#;
        let result = parse_financial_statement(no_tx);
        assert!(result.is_err());
    }
}
