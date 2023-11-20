use criterion::{black_box, criterion_group, criterion_main, Criterion};
use orderbook::{order::TimeInForce, order_book::OrderBook, side::Side};
use rand::thread_rng;
use rand_distr::{Distribution, Normal};

fn criterion_benchmark(c: &mut Criterion) {
    let mut order_book = OrderBook::new(String::from("U.UN"), true, None);

    let mut group = c.benchmark_group("order book");
    group
        .sample_size(10)
        .warm_up_time(std::time::Duration::from_secs(3));

    group.bench_function("add 1 limit orders", |b| {
        b.iter(|| spam_limit_orders(&mut order_book, black_box(1)))
    });

    order_book.flush();
    group.bench_function("add 100k limit orders", |b| {
        b.iter(|| spam_limit_orders(&mut order_book, black_box(100 * 1000)))
    });

    order_book.flush();
    group.bench_function("add 100k limit orders with 10 different prices", |b| {
        b.iter(|| spam_limit_orders_with_variance(&mut order_book, 100 * 1000, 10))
    });

    order_book.flush();
    group.bench_function("add 10mm limit orders", |b| {
        b.iter(|| spam_limit_orders(&mut order_book, 10 * 10 * 100 * 1000))
    });

    order_book.flush();
    group.bench_function("add 1mm limit orders with 100 different prices", |b| {
        b.iter(|| spam_limit_orders_with_variance(&mut order_book, 10 * 100 * 1000, 100))
    });

    order_book.flush();
    group.bench_function("add 1 limit orders with 1 market orders", |b| {
        b.iter(|| spam_limit_and_direct_market_orders(&mut order_book, 1, 1.0, 1.0, 1.0, 1.0))
    });

    order_book.flush();
    group.bench_function("add 100k limit orders with 100k market orders", |b| {
        b.iter(|| {
            spam_limit_and_direct_market_orders(&mut order_book, 100 * 1000, 50.0, 20.0, 50.0, 20.0)
        })
    });

    order_book.flush();
    group.bench_function("add 1mm limit orders with 1mm direct market orders", |b| {
        b.iter(|| {
            spam_limit_and_direct_market_orders(
                &mut order_book,
                10 * 100 * 1000,
                50_000.0,
                10_000.0 * 10_000.0,
                1_000.0,
                500.0 * 500.0,
            )
        })
    });

    order_book.flush();
    group.bench_function("add 100k limit orders with market orders every 10th", |b| {
        b.iter(|| {
            spam_limit_and_occasional_market_orders(
                &mut order_book,
                100 * 1000,
                50_000.0,
                1_000.0,
                1_000.0,
                500.0 * 500.0,
                10,
            )
        })
    });

    order_book.flush();
    group.bench_function("add 1mm limit orders with market orders every 100th", |b| {
        b.iter(|| {
            spam_limit_and_occasional_market_orders(
                &mut order_book,
                10 * 100 * 1000,
                50_000.0,
                10_000.0 * 10_000.0,
                1_000.0,
                500.0 * 500.0,
                100,
            )
        })
    });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

fn spam_limit_orders(book: &mut OrderBook, count: u32) {
    for i in 0..count {
        black_box(book.add_limit_order(Side::Buy, 100, 100, i.into(), i.into(), TimeInForce::GTC));
    }
}

fn spam_limit_orders_with_variance(book: &mut OrderBook, count: u32, variance: usize) {
    for i in 0..count {
        black_box(book.add_limit_order(
            Side::Buy,
            100,
            100 % variance,
            i.into(),
            i.into(),
            TimeInForce::GTC,
        ));
    }
}

fn spam_limit_and_direct_market_orders(
    book: &mut OrderBook,
    count: u32,
    price_mean: f64,
    price_variance: f64,
    quantity_mean: f64,
    quantity_variance: f64,
) {
    let price_distribution = Normal::new(price_mean, price_variance.sqrt()).unwrap();
    let quantity_distribution = Normal::new(quantity_mean, quantity_variance.sqrt()).unwrap();

    for i in 0..count {
        let price_f64 = price_distribution.sample(&mut thread_rng()).floor();
        let quantity_f64 = quantity_distribution.sample(&mut thread_rng()).floor();

        // Ensure the values are at least 1 and do not exceed the maximum value of usize
        let price = price_f64.max(1.0).min(usize::MAX as f64) as usize;
        let quantity = quantity_f64.max(1.0).min(usize::MAX as f64) as usize;

        black_box(book.add_limit_order(
            Side::Buy,
            quantity,
            price,
            i.into(),
            i.into(),
            TimeInForce::GTC,
        ));
        black_box(book.add_market_order(Side::Sell, quantity, i.into(), i.into()));
    }
}

fn spam_limit_and_occasional_market_orders(
    book: &mut OrderBook,
    count: u32,
    price_mean: f64,
    price_variance: f64,
    quantity_mean: f64,
    quantity_variance: f64,
    market_order_frequency: u32,
) {
    let price_distribution = Normal::new(price_mean, price_variance.sqrt()).unwrap();
    let quantity_distribution = Normal::new(quantity_mean, quantity_variance.sqrt()).unwrap();

    for i in 0..count {
        let price_f64 = price_distribution.sample(&mut thread_rng()).floor();
        let quantity_f64 = quantity_distribution.sample(&mut thread_rng()).floor();

        // Ensure the values are at least 1 and do not exceed the maximum value of usize
        let price = price_f64.max(1.0).min(usize::MAX as f64) as usize;
        let quantity = quantity_f64.max(1.0).min(usize::MAX as f64) as usize;

        black_box(book.add_limit_order(
            Side::Buy,
            quantity,
            price,
            i.into(),
            i.into(),
            TimeInForce::GTC,
        ));

        if i % market_order_frequency == 0 {
            black_box(book.add_market_order(Side::Sell, quantity, i.into(), i.into()));
        }
    }
}
