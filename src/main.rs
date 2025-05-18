use std::{env, fs::File, io::BufReader, sync::{LazyLock, RwLock}};
use std::collections::HashMap;
use rust_decimal::{prelude::FromPrimitive, Decimal};

// For the exercise: A static hashmap, because we don't need much more
// for real world: probably a database, with proper sync impl
static TRANSACTIONS: LazyLock<RwLock<HashMap<u32, Transaction>>> = LazyLock::new(Default::default);

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
struct Transaction {
	#[serde(rename = "type")]
	kind: TransactionKind,
	client: u16,
	tx: u32,
	amount: Option<f64>,
	#[serde(skip)]
	under_dispute: bool,
}

impl Transaction {
	/// If transaction has an amount -> returns just it
	/// else we have a dispute/resolve/charback
	/// so we return the amount for it + the original transaction
	pub fn amount(&self) -> anyhow::Result<Decimal> {
		let amount = match self.amount {
			Some(amount) => {
				// Idk if ur gonna feed it bad data in ur tests so I'm being cautious
				if !amount.is_finite() { anyhow::bail!("Bad amount"); }
				amount
			}
			None => {
				let res = TRANSACTIONS.read().unwrap();

				let Some(Some(amount)) = res.get(&self.tx).map(|tx| tx.amount) else {
					anyhow::bail!("Missing tx/amount");
				};

				amount
			}
		};

		// round_dp defaults to banker's rounding
		Decimal::from_f64(amount)
			.map(|x| x.round_dp(4))
			.ok_or_else(|| anyhow::anyhow!("Missing amount"))
	}

	pub fn record(&self) -> anyhow::Result<()> {
		TRANSACTIONS.write()
			.and_then(|mut x| Ok(x.insert(self.tx, self.clone())))
			.map_err(|e| anyhow::anyhow!(format!("Error adding transaction: {e}")))?;
		Ok(())
	}
}

#[derive(Debug, Clone, Copy, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum TransactionKind {
	Deposit,
	Withdrawal,
	Dispute,
	Resolve,
	Chargeback,
}

#[derive(Debug, Clone, Default)]
struct Account {
	// You *should* probably have this in the account,
	// but you don't need to for this example
	// client_id: u16,
	available: Decimal,

	/// Total - available
	held: Decimal,

	locked: bool,
}

impl Account {
	fn deposit(&mut self, transaction: &Transaction) -> anyhow::Result<()> {
		if self.locked { return Ok(()) };

		let maybe_amount = transaction.amount();
		let Ok(amount) = maybe_amount else { anyhow::bail!(maybe_amount.unwrap_err()) };

		if let Err(e) = transaction.record() { anyhow::bail!("Could not save transaction!! {e:?}"); };

		self.available += amount;
		return Ok(());
	}

	fn withdraw(&mut self, transaction: &Transaction) -> anyhow::Result<()>{
		if self.locked { return Ok(()) };

		let maybe_amount = transaction.amount();
		let Ok(amount) = maybe_amount else { anyhow::bail!(maybe_amount.unwrap_err()) };

		if self.available < amount {
			eprintln!("Insufficient funds: {self:?} vs {transaction:?}");
			return Ok(());
		}

		if let Err(e) = transaction.record() { anyhow::bail!(e); };

		self.available -= amount;
		Ok(())
	}

	fn dispute(&mut self, transaction: &Transaction) -> anyhow::Result<()> {
		if self.locked { return Ok(()) };

		// The task mentions that disputes should decrease the amount,
		// regardless if it's disputing a deposit or a withdrawal
		// That makes life easier, so we'll do that

		let maybe_amount = transaction.amount();
		let Ok(amount) = maybe_amount else { anyhow::bail!(maybe_amount.unwrap_err()) };

		let mut reference = TRANSACTIONS.write().unwrap();
		let Some(reference) = reference.get_mut(&transaction.tx) else {
			anyhow::bail!("transaction under dispute does not exist")
		};

		// Overdraft OK on dispute
		reference.under_dispute = true;
		self.available -= amount;
		self.held += amount;

		Ok(())
	}

	fn resolve(&mut self, transaction: &Transaction) -> anyhow::Result<()> {
		if self.locked { return Ok(()) };

		let reference = TRANSACTIONS.read().unwrap();
		let Some(reference) = reference.get(&transaction.tx) else {
			anyhow::bail!("transaction under dispute does not exist")
		};

		if !reference.under_dispute { return Ok(()) };

		// The task mentions that disputes should decrease the amount,
		// regardless if it's disputing a deposit or a withdrawal
		// That makes life easier, so we'll do that

		let maybe_amount = transaction.amount();
		let Ok(amount) = maybe_amount else { anyhow::bail!(maybe_amount.unwrap_err()) };

		// Overdraft OK on dispute
		self.held -= amount;
		self.available += amount;

		Ok(())
	}

	fn chargeback(&mut self) -> anyhow::Result<()> {
		if self.locked { return Ok(()) };
		Ok(())
	}
}

fn main() -> anyhow::Result<()> {
	let Some(input_file_path) = env::args().nth(1) else { anyhow::bail!("No input."); };

	let _ = main_loop(&input_file_path)?;

	Ok(())
}

fn main_loop(file_path: &str) -> anyhow::Result<HashMap<u16, Account>> {
	let file = File::open(file_path)?;
	let buffer = BufReader::new(file);

	let mut accounts: HashMap<u16, Account> = HashMap::new();

	// NOTE: Your example in the pdf is comma AND space delimited
	// which I don't believe is supported by the csv crate currently
	// (https://github.com/BurntSushi/rust-csv/issues/210)
	// So I hope that was just for readibility
	let mut reader = csv::ReaderBuilder::new()
		.flexible(true)
		.from_reader(buffer);

	for transaction in reader.deserialize::<Transaction>() {
		let Ok(transaction @ Transaction { kind, client, amount, .. }) = transaction else {
			eprintln!("Broken record: {transaction:?}");
			continue;
		};

		let entry = accounts
			.entry(client)
			.or_insert(Account::default());

		println!("{entry:?}");

		match kind {
			// Safe unwraps
			TransactionKind::Deposit => {
				let res = entry.deposit(&transaction);
				if let Err(e) = res { eprintln!("{e}"); continue; }
			},
			TransactionKind::Withdrawal => {
				let res = entry.withdraw(&transaction);
				if let Err(e) = res { eprintln!("{e}"); continue; }
			},
			TransactionKind::Dispute => {
				let res = entry.dispute(&transaction);
				if let Err(e) = res { eprintln!("{e}"); continue; }
			},
			TransactionKind::Resolve => {
				let res = entry.resolve(&transaction);
				if let Err(e) = res { eprintln!("{e}"); continue; }
			},
			_ => continue,
			// TransactionKind::Chargeback => todo!(),
		}

		println!("Client {client}, {kind:?} of amount: {amount:?}");
	}

	Ok(accounts)
}

// Lazy tests
#[cfg(test)]
mod test {
	use rust_decimal::dec;

use super::*;
	#[test]
	fn simple_deposit_withdraw() {
		let accounts = main_loop("./csvs/deposit_withdraw.csv");
		let (_, account) = accounts.as_ref().unwrap().iter().next().unwrap();
		assert_eq!(account.available, dec!(0));
	}

	#[test]
	fn bad_withdrawal() {
		let accounts = main_loop("./csvs/bad_withdraw.csv");
		let (_, account) = accounts.as_ref().unwrap().iter().next().unwrap();
		assert_eq!(account.available, dec!(2));
	}

	#[test]
	fn simple_dispute() {
		let accounts = main_loop("./csvs/simple_dispute.csv");
		let (_, account) = accounts.as_ref().unwrap().iter().next().unwrap();
		assert_eq!(account.available, dec!(0));
		assert_eq!(account.held, dec!(5));
	}

	#[test]
	fn simple_resolve() {
		let accounts = main_loop("./csvs/simple_resolve.csv");
		let (_, account) = accounts.as_ref().unwrap().iter().next().unwrap();
		assert_eq!(account.available, dec!(0));
		assert_eq!(account.held, dec!(0));
	}

	#[test]
	fn large() {
		let accounts = main_loop("./csvs/large.csv");
		assert!(accounts.is_ok_and(|x| !x.is_empty()));
	}
}
