use std::sync::{Arc, Mutex};

use sorted_vec::SortedSet;

use super::Transaction;

#[derive(Debug, Clone, Default)]
pub struct TransactionPool {
    transactions: Arc<Mutex<SortedSet<Transaction>>>,
}

impl TransactionPool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_transaction(&self, transaction: Transaction) {
        if transaction.validate() {
            self.transactions.try_lock().unwrap().push(transaction);
        }
    }

    pub fn remove_transaction(&self, transaction: &Transaction) {
        self.transactions
            .try_lock()
            .unwrap()
            .remove_item(transaction);
    }
    pub fn tx_count(&self) -> usize {
        self.transactions.try_lock().unwrap().len()
    }
    pub fn get_top_transaction(&self) -> Transaction {
        self.transactions
            .try_lock()
            .unwrap()
            .drain(..1)
            .next()
            .unwrap()
    }

    pub fn get_transactions(&self, count: usize) -> Vec<Transaction> {
        self.transactions
            .try_lock()
            .unwrap()
            .drain(..count)
            .collect()
    }
}
