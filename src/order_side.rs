// order_side.rs
use crate::side::Side;

use crate::{order::Order, order_queue::OrderQueue};
use std::{collections::BTreeMap, sync::mpsc::Sender};

#[derive(Debug, Clone)]
pub struct OrderSide {
    prices: BTreeMap<usize, OrderQueue>,
    pub volume: usize,
    pub total: usize,
    pub num_orders: usize,
    pub side: Side,
}

impl OrderSide {
    pub fn new(side: Side) -> Self {
        Self {
            prices: BTreeMap::new(),
            volume: 0,
            total: 0,
            num_orders: 0,
            side,
        }
    }

    pub fn flush(&mut self) {
        self.prices.clear();
        self.volume = 0;
        self.total = 0;
        self.num_orders = 0;
    }

    fn log(tx: &Option<Sender<String>>, msg: String) {
        if let Some(tx) = &tx {
            tx.send(msg).unwrap();
        } else {
            println!("{}", msg);
        }
    }

    pub fn add_order(&mut self, order: Order) -> Order {
        let price = order.price;
        let quantity = order.quantity;
        let order_queue = self.prices.entry(price).or_insert(OrderQueue::new(price));
        order_queue.append(order.clone());
        self.volume += quantity;
        self.total += price * quantity;
        self.num_orders += 1;
        return order;
    }

    pub fn remove_order(&mut self, order: &Order, tx: &Option<Sender<String>>) -> Option<Order> {
        let price = order.price;
        let quantity = order.quantity;
        let order_queue = self.prices.get_mut(&price)?;
        let removed_order = order_queue.remove(order)?;
        OrderSide::log(
            tx,
            format!("A, {:?}, {:?}", removed_order.user_id, removed_order.id),
        );
        self.volume -= quantity;
        self.total -= price * quantity;
        self.num_orders -= 1;
        if order_queue.len() <= 0 {
            self.prices.remove(&price);
        }
        if self.side == Side::Buy {
            let p = self.max_price();
            if price >= p {
                if self.volume == 0 {
                    OrderSide::log(tx, format!("B, B, -, -"));
                } else {
                    OrderSide::log(tx, format!("B, B, {:?}, {:?}", p, quantity));
                }
            }
        } else {
            let p = self.min_price();
            if price <= p {
                if self.volume == 0 {
                    OrderSide::log(tx, format!("B, S, -, -"));
                } else {
                    OrderSide::log(tx, format!("B, S, {:?}, {:?}", p, quantity));
                }
            }
        };
        return Some(removed_order);
    }

    pub fn remove_order_internal(
        &mut self,
        order: &Order,
        tx: &Option<Sender<String>>,
    ) -> Option<Order> {
        let price = order.price;
        let quantity = order.quantity;
        let order_queue = self.prices.get_mut(&price)?;
        let removed_order = order_queue.remove(order)?;
        self.volume -= quantity;
        self.total -= price * quantity;
        self.num_orders -= 1;
        if order_queue.len() <= 0 {
            self.prices.remove(&price);
        }
        if self.side == Side::Buy {
            let p = self.max_price();
            if price >= p {
                if self.volume == 0 {
                    OrderSide::log(tx, format!("B, B, -, -"));
                } else {
                    OrderSide::log(tx, format!("B, B, {:?}, {:?}", p, quantity));
                }
            }
        } else {
            let p = self.min_price();
            if price <= p {
                if self.volume == 0 {
                    OrderSide::log(tx, format!("B, S, -, -"));
                } else {
                    OrderSide::log(tx, format!("B, S, {:?}, {:?}", p, quantity));
                }
            }
        };
        return Some(removed_order);
    }

    // use when a trade is executed and order is partially filled
    pub fn decrease_volume_and_total(&mut self, order: &Order) {
        let price = order.price;
        let quantity = order.quantity;
        self.volume -= quantity;
        self.total -= price * quantity;
    }

    pub fn max_price_queue(&self) -> Option<&OrderQueue> {
        if self.prices.len() > 0 {
            let (_k, v) = self.prices.iter().next_back().unwrap();
            return Some(v);
        }
        None
    }
    pub fn max_price_queue_mut(&mut self) -> Option<&mut OrderQueue> {
        if self.prices.len() > 0 {
            let (_k, v) = self.prices.iter_mut().next_back().unwrap();
            return Some(v);
        }
        None
    }

    pub fn min_price_queue(&self) -> Option<&OrderQueue> {
        if self.prices.len() > 0 {
            let min = self.prices.iter().next().unwrap();
            return Some(min.1);
        }
        None
    }
    pub fn min_price_queue_mut(&mut self) -> Option<&mut OrderQueue> {
        if self.prices.len() > 0 {
            let min = self.prices.iter_mut().next().unwrap();
            return Some(min.1);
        }
        None
    }

    pub fn max_price(&self) -> usize {
        if self.prices.len() > 0 {
            let (k, _v) = self.prices.iter().next_back().unwrap();
            return *k;
        }
        0
    }

    pub fn min_price(&self) -> usize {
        if self.prices.len() > 0 {
            let min = self.prices.iter().next().unwrap();
            return *min.0;
        }
        usize::MAX
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        order::{OrderType, TimeInForce},
        side::Side,
    };

    #[test]
    fn test_add_order() {
        let mut order_side = OrderSide::new(Side::Buy);
        let id = 1;
        let order = Order::new(
            id,
            id,
            Side::Buy,
            OrderType::Limit,
            TimeInForce::GTC,
            500,
            100,
        );
        order_side.add_order(order);
        assert_eq!(order_side.num_orders, 1);
        assert_eq!(order_side.volume, 100);
        assert_eq!(order_side.total, 500 * 100);
        assert_eq!(order_side.prices.len(), 1);
        assert_eq!(
            &order_side.prices.get(&500).unwrap().head().unwrap().id,
            &id
        );
    }

    #[test]
    fn test_remove_order() {
        let mut order_side = OrderSide::new(Side::Buy);
        let id = 1;
        let order_draft = Order::new(
            id,
            1,
            Side::Buy,
            OrderType::Limit,
            TimeInForce::GTC,
            500,
            100,
        );
        let order = order_side.add_order(order_draft);
        assert_eq!(order_side.num_orders, 1);
        assert_eq!(order_side.volume, 100);
        assert_eq!(order_side.total, 500 * 100);
        assert_eq!(order_side.prices.len(), 1);
        assert_eq!(
            &order_side.prices.get(&500).unwrap().head().unwrap().id,
            &id
        );
        let order_draft_two = Order::new(
            id,
            1,
            Side::Buy,
            OrderType::Limit,
            TimeInForce::GTC,
            123,
            456,
        );
        let order_two = order_side.add_order(order_draft_two);
        assert_eq!(order_side.num_orders, 2);
        assert_eq!(order_side.volume, 556);
        assert_eq!(order_side.total, (500 * 100) + (123 * 456));
        assert_eq!(order_side.prices.len(), 2);
        assert_eq!(
            &order_side.prices.get(&123).unwrap().head().unwrap().id,
            &id
        );
        order_side.remove_order(&order, &None);
        assert_eq!(order_side.num_orders, 1);
        assert_eq!(order_side.volume, 456);
        assert_eq!(order_side.total, (123 * 456));
        assert_eq!(order_side.prices.len(), 1);
        order_side.remove_order(&order_two, &None);
        assert_eq!(order_side.num_orders, 0);
        assert_eq!(order_side.volume, 0);
        assert_eq!(order_side.total, 0);
        assert_eq!(order_side.prices.len(), 0);
    }

    #[test]
    fn test_decrease_volume_and_total() {
        let mut order_side = OrderSide::new(Side::Buy);
        let id = 1;
        let order_draft = Order::new(
            id,
            1,
            Side::Buy,
            OrderType::Limit,
            TimeInForce::GTC,
            500,
            100,
        );
        let order = order_side.add_order(order_draft);
        assert_eq!(order_side.num_orders, 1);
        assert_eq!(order_side.volume, 100);
        assert_eq!(order_side.total, 500 * 100);
        assert_eq!(order_side.prices.len(), 1);
        assert_eq!(
            &order_side.prices.get(&500).unwrap().head().unwrap().id,
            &id
        );
        order_side.decrease_volume_and_total(&order);
        assert_eq!(order_side.num_orders, 1);
        assert_eq!(order_side.volume, 0);
        assert_eq!(order_side.total, 0);
        assert_eq!(order_side.prices.len(), 1);
        assert_eq!(
            &order_side.prices.get(&500).unwrap().head().unwrap().id,
            &id
        );
    }

    #[test]
    fn test_max_price_queue() {
        let mut order_side = OrderSide::new(Side::Buy);
        let order_draft = Order::new(
            1,
            1,
            Side::Buy,
            OrderType::Limit,
            TimeInForce::GTC,
            100,
            100,
        );
        let order_draft_two = Order::new(
            2,
            1,
            Side::Buy,
            OrderType::Limit,
            TimeInForce::GTC,
            200,
            100,
        );
        let id = 3;
        let order_draft_highest_price = Order::new(
            id,
            1,
            Side::Buy,
            OrderType::Limit,
            TimeInForce::GTC,
            500,
            100,
        );
        order_side.add_order(order_draft);
        order_side.add_order(order_draft_two);
        order_side.add_order(order_draft_highest_price);
        assert_eq!(order_side.num_orders, 3);
        assert_eq!(order_side.volume, 300);
        assert_eq!(order_side.total, 100 * 100 + 200 * 100 + 500 * 100);
        assert_eq!(order_side.prices.len(), 3);
        let max_price_queue = order_side.max_price_queue();
        assert_eq!(max_price_queue.unwrap().head().unwrap().id, id);
    }

    #[test]
    fn test_max_price_queue_mut() {
        let mut order_side = OrderSide::new(Side::Buy);
        let order_draft = Order::new(
            1,
            1,
            Side::Buy,
            OrderType::Limit,
            TimeInForce::GTC,
            100,
            100,
        );
        let order_draft_two = Order::new(
            2,
            1,
            Side::Buy,
            OrderType::Limit,
            TimeInForce::GTC,
            200,
            100,
        );
        let id = 3;
        let order_draft_highest_price = Order::new(
            id,
            1,
            Side::Buy,
            OrderType::Limit,
            TimeInForce::GTC,
            500,
            100,
        );
        order_side.add_order(order_draft);
        order_side.add_order(order_draft_two);
        order_side.add_order(order_draft_highest_price);
        assert_eq!(order_side.num_orders, 3);
        assert_eq!(order_side.volume, 300);
        assert_eq!(order_side.total, 100 * 100 + 200 * 100 + 500 * 100);
        assert_eq!(order_side.prices.len(), 3);
        let max_price_queue = order_side.max_price_queue_mut();
        assert_eq!(max_price_queue.unwrap().head().unwrap().id, id);
    }

    #[test]
    fn test_min_price_queue() {
        let mut order_side = OrderSide::new(Side::Buy);
        let order_draft = Order::new(
            1,
            1,
            Side::Buy,
            OrderType::Limit,
            TimeInForce::GTC,
            100,
            100,
        );
        let order_draft_two = Order::new(
            2,
            1,
            Side::Buy,
            OrderType::Limit,
            TimeInForce::GTC,
            200,
            100,
        );
        let id = 3;
        let order_draft_lowest_price = Order::new(
            id,
            1,
            Side::Buy,
            OrderType::Limit,
            TimeInForce::GTC,
            50,
            100,
        );
        order_side.add_order(order_draft);
        order_side.add_order(order_draft_two);
        order_side.add_order(order_draft_lowest_price);
        assert_eq!(order_side.num_orders, 3);
        assert_eq!(order_side.volume, 300);
        assert_eq!(order_side.total, 100 * 100 + 200 * 100 + 50 * 100);
        assert_eq!(order_side.prices.len(), 3);
        let min_price_queue = order_side.min_price_queue();
        assert_eq!(min_price_queue.unwrap().head().unwrap().id, id);
    }

    #[test]
    fn test_min_price_queue_mut() {
        let mut order_side = OrderSide::new(Side::Buy);
        let order_draft = Order::new(
            1,
            1,
            Side::Buy,
            OrderType::Limit,
            TimeInForce::GTC,
            100,
            100,
        );
        let order_draft_two = Order::new(
            2,
            1,
            Side::Buy,
            OrderType::Limit,
            TimeInForce::GTC,
            200,
            100,
        );
        let id = 3;
        let order_draft_lowest_price = Order::new(
            id,
            1,
            Side::Buy,
            OrderType::Limit,
            TimeInForce::GTC,
            50,
            100,
        );
        order_side.add_order(order_draft);
        order_side.add_order(order_draft_two);
        order_side.add_order(order_draft_lowest_price);
        assert_eq!(order_side.num_orders, 3);
        assert_eq!(order_side.volume, 300);
        assert_eq!(order_side.total, 100 * 100 + 200 * 100 + 50 * 100);
        assert_eq!(order_side.prices.len(), 3);
        let min_price_queue = order_side.min_price_queue_mut();
        assert_eq!(min_price_queue.unwrap().head().unwrap().id, id);
    }
}
