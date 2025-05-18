use std::env;

#[derive(Debug, Clone)]
struct Transaction {
	kind: TransactionKind,
	client_id: u16,
	/// Globally unique id
	tx: u32,
	amount: f64,
}

#[derive(Debug, Clone)]
enum TransactionKind {
	Deposit,
	Withdrawal,
	Dispute,
	Resolve,
	Chargeback,
}

#[derive(Debug, Clone)]
struct Account {
	client_id: u16,
	available: f64,

	/// Total - available
	// held: f64,
	/// Available + held
	// total: f64,
	locked: bool,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    println!("args: {args:#?}");
}
