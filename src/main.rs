use std::{env, fs::File, io::BufReader, sync::{LazyLock, RwLock}};
use std::collections::HashMap;
use rust_decimal::{prelude::FromPrimitive, Decimal};

// For the exercise: An in memory static hashmap, because we don't need more
// (If anything, I think we only need "tx" -> "amount" + "deposit or withdrawal" to be more efficient)
// for real world: probably a database, with proper sync impl
static TRANSACTIONS: LazyLock<RwLock<HashMap<u32, Transaction>>> = LazyLock::new(Default::default);

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
struct Transaction {
	#[serde(rename = "type")]
	kind: TransactionKind,
	client: u16,
	/// Globally unique id
	tx: u32,
	amount: Option<f64>,
}

impl Transaction {
	pub fn amount(&self) -> anyhow::Result<Option<Decimal>> {
		if let Some(amount) = self.amount {
			if !amount.is_finite() {
				anyhow::bail!("Bad amount: {self:#?}");
			}
		}

		// round_dp defaults to banker's rounding
		Ok(self.amount
			.map(Decimal::from_f64).flatten()
			.and_then(|x| Some(x.round_dp(4))))

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
	/// Available + held
	total: Decimal,
	locked: bool,
}

impl Account {
	fn deposit(&mut self, transaction: &Transaction) -> anyhow::Result<()> {
		if self.locked { return Ok(()) };

		let amount = match transaction.amount() {
			Ok(Some(amnt)) => amnt,
			Ok(None) => anyhow::bail!("Missing amount"),
			Err(e) => anyhow::bail!(e),
		};

		if let Err(e) = transaction.record() { anyhow::bail!(e); };

		self.available += amount;
		return Ok(());
	}

	fn withdraw(&mut self, transaction: &Transaction) -> anyhow::Result<()>{
		if self.locked { return Ok(()) };

		let amount = match transaction.amount() {
			Ok(Some(amnt)) => amnt,
			Ok(None) => anyhow::bail!("Missing amount"),
			Err(e) => anyhow::bail!(e),
		};

		if self.available < amount {
			eprintln!("Insufficient funds: {self:?} vs {transaction:?}");
			return Ok(());
		}

		if let Err(e) = transaction.record() { anyhow::bail!(e); };

		self.available -= amount;
		Ok(())
	}

	fn dispute(&mut self, tx: u32) {

	}

	fn resolve() {}
	fn chargeback() {}
}

fn main() -> anyhow::Result<()> {
	let arg = env::args().nth(1);
	let Some(input_file_path) = arg else {
		anyhow::bail!("No input.");
	};
	println!("args: {input_file_path:#?}");

	let mut accounts: HashMap<u16, Account> = HashMap::new();

	let file = File::open(input_file_path)?;
	let buffer = BufReader::new(file);

	// NOTE: Your example in the pdf is comma AND space delimited
	// which I don't believe is supported by the csv crate currently
	// (https://github.com/BurntSushi/rust-csv/issues/210)
	// So I hope that was just for readibility
	let mut reader = csv::ReaderBuilder::new()
		.flexible(true)
		.from_reader(buffer);

	for transaction in reader.deserialize::<Transaction>() {
		let Ok(transaction @ Transaction { kind, client, tx, amount }) = transaction else {
			eprintln!("Broken record: {transaction:?}");
			continue;
		};

		println!("Client {client}, {kind:?} of {amount:?}");

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
			TransactionKind::Dispute => entry.dispute(tx),
			TransactionKind::Resolve => todo!(),
			TransactionKind::Chargeback => todo!(),
		}
	}

	Ok(())
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn simple_deposit_withdraw() {

	}
}
