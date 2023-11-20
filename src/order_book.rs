// order_book.rs
use crate::{
    order::{Order, OrderType, TimeInForce},
    order_side::OrderSide,
    side::Side,
};
use std::{collections::BTreeMap, sync::mpsc::Sender};

#[derive(Debug)]
pub struct InProcessOrder {
    done: Vec<Order>,
    partial: Option<Order>,
    partial_quantity_processed: usize,
    quantity_left: usize,
    err: Option<String>,
}

#[derive(Debug)]
pub struct OrderBook {
    orders: BTreeMap<u32, Order>,
    bids: OrderSide,
    asks: OrderSide,
    symbol: String,
    trading_enabled: bool,
    tx: Option<Sender<String>>,
}

impl OrderBook {
    pub fn new(symbol: String, trading_enabled: bool, tx: Option<Sender<String>>) -> OrderBook {
        Self {
            orders: BTreeMap::new(),
            bids: OrderSide::new(Side::Buy),
            asks: OrderSide::new(Side::Sell),
            symbol,
            trading_enabled,
            tx,
        }
    }
    pub fn flush(&mut self) {
        self.bids.flush();
        self.asks.flush();
        self.orders.clear();
    }

    fn log(&self, msg: String) {
        if let Some(tx) = &self.tx {
            let _ = tx.send(msg);
        } else {
            println!("{}", msg);
        }
    }

    pub fn add_limit_order(
        &mut self,
        side: Side,
        size: usize,
        price: usize,
        user_id: u32,
        id: u32,
        time_in_force: TimeInForce,
    ) -> InProcessOrder {
        let mut order = InProcessOrder {
            done: Vec::new(),
            partial: None,
            partial_quantity_processed: 0,
            quantity_left: size,
            err: None,
        };
        if self.orders.contains_key(&id) {
            order.err = Some("Order Already Exists".to_string());
            return order;
        }

        let mut quantity_to_trade = size;

        let comparator = match side {
            Side::Buy => Self::greater_than_or_equal,
            Side::Sell => Self::lower_than_or_equal,
        };

        while quantity_to_trade > 0 {
            {
                let side_to_process = match side {
                    Side::Buy => &self.asks,
                    Side::Sell => &self.bids,
                };
                if side_to_process.num_orders <= 0 {
                    break;
                }
                let best_price = match side {
                    Side::Buy => side_to_process.min_price_queue(),
                    Side::Sell => side_to_process.max_price_queue(),
                };
                if best_price.is_none() {
                    break;
                }
                if !comparator(&price, &best_price.as_ref().unwrap().price) {
                    break;
                } else if !self.trading_enabled {
                    order.err = Some("Trading is not enabled".to_string());
                    self.log(format!("R, {:?}, {:?}", user_id, id));
                    return order;
                }
                // TODO: implement FOK order logic
                // If FOK order was not matched completely don't process it
                // if time_in_force == TimeInForce::FOK && size > available_at_limit_price {
                //   break;
                // }
            }
            self.log(format!("A, {:?}, {:?}", user_id, id));
            let process_queue = self.process_queue(side, quantity_to_trade, user_id, id);
            order.done.extend(process_queue.done);
            order.partial = process_queue.partial;
            order.partial_quantity_processed = process_queue.partial_quantity_processed;
            quantity_to_trade = process_queue.quantity_left;
            order.quantity_left = quantity_to_trade;
        }

        if quantity_to_trade > 0 {
            let new_order = Order::new(
                id,
                user_id,
                side,
                OrderType::Limit,
                time_in_force,
                price,
                quantity_to_trade,
            );
            if order.done.len() > 0 {
                order.partial_quantity_processed = size - quantity_to_trade;
                order.partial = Some(new_order.clone());
            }

            self.log(format!("A, {:?}, {:?}", user_id, id));
            let qt = new_order.quantity.clone();
            let p = new_order.price.clone();
            if side == Side::Buy {
                let m_price = self.bids.max_price();
                let value = self.bids.add_order(new_order);
                self.orders.insert(id, value);
                if p > m_price {
                    self.log(format!("B, B, {:?}, {:?}", p, qt));
                } else if p == m_price {
                    let queue = self.bids.max_price_queue();
                    self.log(format!("B, B, {:?}, {:?}", p, &queue.unwrap().volume));
                }
            } else {
                let m_price = self.asks.min_price();
                let value = self.asks.add_order(new_order);
                self.orders.insert(id, value);
                if p < m_price {
                    self.log(format!("B, S, {:?}, {:?}", p, qt));
                } else if p == m_price {
                    let queue = self.asks.min_price_queue();
                    self.log(format!("B, S, {:?}, {:?}", p, &queue.unwrap().volume));
                }
            }
        } else {
            let mut total_quantity: usize = 0;
            let mut total_price: usize = 0;
            order.done.iter().for_each(|order| {
                total_quantity += order.quantity;
                total_price += order.price * order.quantity;
            });

            if order.partial_quantity_processed > 0 && order.partial.is_some() {
                total_quantity += order.partial_quantity_processed;
                total_price +=
                    order.partial.as_ref().unwrap().price * order.partial_quantity_processed;
            }

            order.done.push(Order::new(
                id,
                user_id,
                side,
                OrderType::Limit,
                time_in_force,
                total_price / total_quantity,
                total_quantity,
            ));
        }

        // TODO: implement IOC order logic
        // If IOC order was not matched completely remove from the order book
        // if time_in_force == TimeInForce::IOC && order.quantity_left > 0 {
        //   self.cancel_order(order.id);
        // }

        return order;
    }

    pub fn add_market_order(
        &mut self,
        side: Side,
        size: usize,
        user_id: u32,
        id: u32,
    ) -> InProcessOrder {
        let mut in_process_order = InProcessOrder {
            done: Vec::new(),
            partial: None,
            partial_quantity_processed: 0,
            quantity_left: size,
            err: None,
        };

        let mut quantity_to_trade = size;

        while quantity_to_trade > 0 {
            {
                let side_to_process = match side {
                    Side::Buy => &self.asks,
                    Side::Sell => &self.bids,
                };
                if side_to_process.num_orders <= 0 {
                    break;
                }
                let best_price = match side {
                    Side::Buy => side_to_process.min_price_queue(),
                    Side::Sell => side_to_process.max_price_queue(),
                };
                if best_price.is_none() {
                    break;
                }
            }
            self.log(format!("A, {:?}, {:?}", user_id, id));
            let process_queue = self.process_queue(side, quantity_to_trade, user_id, id);
            in_process_order.done.extend(process_queue.done);
            in_process_order.partial = process_queue.partial;
            in_process_order.partial_quantity_processed = process_queue.partial_quantity_processed;
            quantity_to_trade = process_queue.quantity_left;
        }
        in_process_order.quantity_left = quantity_to_trade;
        return in_process_order;
    }

    // pub fn edit_order(&mut self, user_id: u32, id: u32, size: usize) {
    // TODO: Implement the edit order logic here
    // }

    // TODO: redo delete implementation so we use user_id too
    pub fn cancel_order_user(&mut self, _user_id: u32, id: u32) -> Option<Order> {
        self.orders.remove(&id).and_then(|order| match order.side {
            Side::Buy => self.bids.remove_order(&order, &self.tx),
            Side::Sell => self.asks.remove_order(&order, &self.tx),
        })
    }

    fn cancel_order(&mut self, id: u32) -> Option<Order> {
        self.orders.remove(&id).and_then(|order| match order.side {
            Side::Buy => self.bids.remove_order_internal(&order, &self.tx),
            Side::Sell => self.asks.remove_order_internal(&order, &self.tx),
        })
    }

    fn greater_than_or_equal(a: &usize, b: &usize) -> bool {
        return a >= b;
    }
    fn lower_than_or_equal(a: &usize, b: &usize) -> bool {
        return a <= b;
    }

    fn process_queue(
        &mut self,
        side: Side,
        quantity_to_trade: usize,
        user_id: u32,
        id: u32,
    ) -> InProcessOrder {
        let mut in_process_order = InProcessOrder {
            done: Vec::new(),
            partial: None,
            partial_quantity_processed: 0,
            quantity_left: quantity_to_trade,
            err: None,
        };

        while in_process_order.quantity_left > 0 {
            if let Some(order_queue) = match side {
                Side::Buy => self.asks.min_price_queue_mut(),
                Side::Sell => self.bids.max_price_queue_mut(),
            } {
                if order_queue.len() <= 0 {
                    break;
                }
                if let Some(head_order) = order_queue.head() {
                    let head_size = head_order.quantity;
                    if in_process_order.quantity_left < head_size {
                        let mut new_order = Order::new(
                            head_order.id,
                            head_order.user_id,
                            head_order.side,
                            head_order.order_type,
                            head_order.time_in_force,
                            head_order.price,
                            head_size - in_process_order.quantity_left,
                        );

                        let msg = format!(
                            "T, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}",
                            user_id,
                            id,
                            head_order.user_id,
                            head_order.id,
                            head_order.price,
                            in_process_order.quantity_left
                        );

                        in_process_order.partial = Some(new_order.clone());
                        self.orders.insert(new_order.id, new_order.clone());
                        in_process_order.partial_quantity_processed =
                            in_process_order.quantity_left;
                        order_queue.update_head(head_order.clone(), new_order.clone());
                        new_order.quantity = in_process_order.quantity_left;
                        self.log(msg);
                        if side == Side::Buy {
                            let m_price = self.asks.min_price();
                            self.log(format!(
                                "B, S, {:?}, {:?}",
                                m_price,
                                head_size - in_process_order.quantity_left
                            ));
                        } else {
                            let m_price = self.bids.max_price();
                            self.log(format!(
                                "B, B, {:?}, {:?}",
                                m_price,
                                head_size - in_process_order.quantity_left
                            ));
                        }

                        in_process_order.quantity_left = 0;

                        match new_order.side {
                            Side::Buy => self.bids.decrease_volume_and_total(&new_order),
                            Side::Sell => self.asks.decrease_volume_and_total(&new_order),
                        };
                    } else {
                        in_process_order.quantity_left = in_process_order.quantity_left - head_size;
                        let order_id = head_order.id.clone();
                        let msg = format!(
                            "T, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}",
                            user_id,
                            id,
                            head_order.user_id,
                            head_order.id,
                            head_order.price,
                            head_order.quantity
                        );
                        self.log(msg);
                        if let Some(canceled_order) = self.cancel_order(order_id) {
                            in_process_order.done.push(canceled_order);
                        };
                    }
                }
            }
        }
        return in_process_order;
    }
}

// TODO add unit tests
