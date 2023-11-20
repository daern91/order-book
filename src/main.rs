//main.rs
mod order;
mod order_book;
mod order_queue;
mod order_side;
mod side;
use order::TimeInForce;
use order_book::OrderBook;
use std::{
    env, thread,
    {collections::BTreeMap, error::Error, process},
    {io, sync::mpsc},
};

#[derive(Debug)]
struct Transaction {
    user_id: u32,
    symbol: String,
    price: usize,
    quantity: usize,
    side: side::Side,
    user_order_id: u32,
}

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let ignore_errors = args.len() == 2 && args[1] == "--ignore-errors";
    // Multi-producer, single-consumer queue messaging. We will use this
    // to have one producer thread per order book, and one consumer thread
    // to print to stdout.
    // Sharding the Order Book per Ticker one thread could sense when we need to persist?
    // Perhaps even an individual node per ticker?
    // If we don't persist at all, and store all in memory. Maybe look into having a
    // shadow node that might or might not be persisted to disk? Or simpy route
    // traffic to shadow if active goes down?
    // Look into TCP and UDP. Inbound traffic with TCP and outbound with UDP?
    // TCP is reliable and makes sure the order goes through. UDP is not reliable
    // but spits out completed orders faster.
    // Look into where to host this. Where do we get a single machine? Like AWS XXXXL?
    // Should be "bare metal" and not virtualized AWS EC2 instances.
    //
    //
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let mut amount_of_flushes = 0;
        let mut order_books: BTreeMap<String, OrderBook> = BTreeMap::new();
        // Build the CSV reader and iterate over each record.
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .comment(Some(b'#'))
            .flexible(true)
            .trim(csv::Trim::All)
            .from_reader(io::stdin());
        for result in rdr.records() {
            // The iterator yields Result<StringRecord, Error>, so we check the
            // error here.
            // check first column for transaction type
            let record = result.unwrap();
            match record.get(0) {
                Some("N") => {
                    let transaction = if ignore_errors {
                        Transaction {
                            user_id: record.get(1).unwrap_or("").parse::<u32>().unwrap_or(0),
                            symbol: record.get(2).unwrap_or("").to_string(),
                            price: record.get(3).unwrap_or("").parse::<usize>().unwrap_or(0),
                            quantity: record.get(4).unwrap_or("").parse::<usize>().unwrap_or(0),
                            side: match record.get(5).unwrap_or("X") {
                                "B" => side::Side::Buy,
                                "S" => side::Side::Sell,
                                _ => continue,
                            },
                            user_order_id: record.get(6).unwrap_or("").parse::<u32>().unwrap_or(0),
                        }
                    } else {
                        Transaction {
                            user_id: record.get(1).unwrap().parse::<u32>().unwrap(),
                            symbol: record.get(2).unwrap().to_string(),
                            price: record.get(3).unwrap().parse::<usize>().unwrap(),
                            quantity: record.get(4).unwrap().parse::<usize>().unwrap(),
                            side: match record.get(5).unwrap() {
                                "B" => side::Side::Buy,
                                "S" => side::Side::Sell,
                                _ => panic!("Invalid side"),
                            },
                            user_order_id: record.get(6).unwrap().parse::<u32>().unwrap(),
                        }
                    };
                    let trading_enabled = if amount_of_flushes >= 10 { true } else { false };
                    let order_book =
                        order_books
                            .entry(transaction.symbol.clone())
                            .or_insert(OrderBook::new(
                                transaction.symbol.clone(),
                                trading_enabled,
                                Some(tx.clone()), // creating multiple producer threads - one for each symbol
                            ));
                    if transaction.price > 0 {
                        order_book.add_limit_order(
                            transaction.side,
                            transaction.quantity,
                            transaction.price,
                            transaction.user_id,
                            transaction.user_order_id,
                            TimeInForce::GTC,
                        );
                    } else {
                        order_book.add_market_order(
                            transaction.side,
                            transaction.quantity,
                            transaction.user_id,
                            transaction.user_order_id,
                        );
                    }
                }
                Some("C") => {
                    // TODO: implement better cancel logic
                    if ignore_errors {
                        let user_id = record.get(1).unwrap_or("").parse::<u32>().unwrap_or(0);
                        let order_id = record.get(2).unwrap_or("").parse::<u32>().unwrap_or(0);
                        for (_, order_book) in order_books.iter_mut() {
                            order_book.cancel_order_user(user_id, order_id);
                        }
                    } else {
                        let user_id = record.get(1).unwrap().parse::<u32>().unwrap();
                        let order_id = record.get(2).unwrap().parse::<u32>().unwrap();
                        for (_, order_book) in order_books.iter_mut() {
                            order_book.cancel_order_user(user_id, order_id);
                        }
                    }
                }
                Some("F") => {
                    amount_of_flushes += 1;
                    order_books.clear();
                }
                _ => {
                    if ignore_errors {
                        continue;
                    };
                    eprintln!("Invalid transaction type");
                    process::exit(1);
                }
            };
        }
    });

    for received in rx {
        println!("{}", received);
    }
    Ok(())
}

fn main() {
    if let Err(err) = run() {
        println!("error running example: {}", err);
        process::exit(1);
    }
}
