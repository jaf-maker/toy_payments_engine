use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::File;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

//Struct for the deserializer to parse the CSV input file
#[derive(Deserialize, Debug)]
struct OffChainTransaction {
    #[serde(rename = "type")]
    transaction_type: TransactionType,
    client: u16,
    #[serde(rename = "tx")]
    id: u32,
    amount: Option<f64>,
}

//Struct for an account
#[derive(Debug)]
struct Account {
    available: f64,
    held: f64,
    locked: bool,
    transactions: HashMap<u32, (f64, bool)>,
}

fn process_transaction(account: &mut Account, transaction: &OffChainTransaction) {
    // process transaction
    match transaction.transaction_type {
        TransactionType::Deposit => {
            // add transaction amount and log transaction
            if let Some(amount) = transaction.amount {
                account.available += amount;
                account.transactions.insert(transaction.id, (amount, false));
            }
        }
        TransactionType::Withdrawal => {
            // removes transaction amount if funds are enough
            if let Some(amount) = transaction.amount {
                if account.available >= amount {
                    account.available -= amount;
                }
            }
        }
        TransactionType::Dispute => {
            // move disputed amount to held balance and flags transaction
            if let Some((amount, disputed)) = account.transactions.get_mut(&transaction.id) {
                if !*disputed {
                    account.available -= *amount;
                    account.held += *amount;
                    *disputed = true;
                }
            }
        }
        TransactionType::Resolve => {
            // reverts transaction amount from held to available and removes the flag from transaction
            if let Some((amount, disputed)) = account.transactions.get_mut(&transaction.id) {
                if *disputed {
                    account.available += *amount;
                    account.held -= *amount;
                    *disputed = false;
                }
            }
        }
        TransactionType::Chargeback => {
            // removes dispute amount from held and locks the account
            if let Some((amount, disputed)) = account.transactions.get_mut(&transaction.id) {
                if *disputed {
                    account.held -= *amount;
                    account.locked = true;
                    *disputed = false;
                }
            }
        }
    }
}

pub fn process_transaction_file(file: std::fs::File) -> String {
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(file);

    let iter_reader = reader.deserialize::<OffChainTransaction>();
    let mut accounts: HashMap<u16, Account> = HashMap::new();

    // iterate csv file line by line
    for item in iter_reader {
        if let Ok(transaction) = item {
            // check if account exists, if not, it adds a new one
            if !accounts.contains_key(&transaction.client) {
                accounts.insert(
                    transaction.client,
                    Account {
                        available: 0.0,
                        held: 0.0,
                        locked: false,
                        transactions: HashMap::new(),
                    },
                );
            }
            let account = accounts.get_mut(&transaction.client).unwrap();

            // process transaction if not locked
            if !account.locked {
                process_transaction(account, &transaction)
            }
        } else {
            // log row that could not be deserialized
            eprintln!("Skipping invalid row: {:?}", item);
        }
    }

    // convert the keys to vector so that I can sort them in a predictale way
    // to validate the output using cargo test
    // generates a warning for keys being mutable
    let mut keys: Vec<_> = accounts.keys().cloned().collect();

    #[cfg(test)]
    keys.sort();

    // print final csv to cli
    let mut output = String::from("client,available,held,total,locked");

    for client in keys {
        let data = accounts.get(&client).unwrap();
        let total = data.held + data.available;
        output = format!(
            "{}\n{},{:.4},{:.4},{:.4},{}",
            output, client, data.available, data.held, total, data.locked
        );
    }
    output = format!("{}\n", output);
    return output;
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let file = File::open(&args[1])?;
    let output = process_transaction_file(file);
    print!("{}", output);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::process_transaction_file;
    use std::fs::File;

    #[test]
    fn default() {
        let correct_output = "client,available,held,total,locked
1,1.5000,0.0000,1.5000,false
2,2.0000,0.0000,2.0000,false\n";
        let file = File::open("./test_files/transactions_1.csv").unwrap();
        assert_eq!(process_transaction_file(file), correct_output);
    }

    #[test]
    fn longer_sequence() {
        let correct_output = "client,available,held,total,locked
1,70.0000,0.0000,70.0000,true
2,300.0000,0.0000,300.0000,false\n";
        let file = File::open("./test_files/transactions_2.csv").unwrap();
        assert_eq!(process_transaction_file(file), correct_output);
    }

    #[test]
    fn negative_balance() {
        let correct_output = "client,available,held,total,locked
1,20.0000,0.0000,20.0000,true
2,-100.0000,0.0000,-100.0000,true\n";
        let file = File::open("./test_files/transactions_3.csv").unwrap();
        assert_eq!(process_transaction_file(file), correct_output);
    }

    #[test]
    fn invalid_inputs() {
        let correct_output = "client,available,held,total,locked
1,50.0000,0.0000,50.0000,false
2,0.0000,0.0000,0.0000,false\n";
        let file = File::open("./test_files/transactions_4.csv").unwrap();
        assert_eq!(process_transaction_file(file), correct_output);
    }
}
