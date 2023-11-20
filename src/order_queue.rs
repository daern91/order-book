//order_queue.rs
use crate::order::Order;

#[derive(Debug, Clone)]
pub struct OrderQueue {
    pub price: usize,
    pub volume: usize,
    orders: Vec<Order>,
}

impl OrderQueue {
    pub fn new(price: usize) -> Self {
        Self {
            price,
            volume: 0,
            orders: Vec::new(),
        }
    }

    // adds order to tail of the queue and returns the order
    pub fn append(&mut self, order: Order) -> &Order {
        self.volume += order.quantity;
        self.orders.push(order);
        self.orders.last().unwrap()
    }

    // removes order from the queue and returns the order
    pub fn remove(&mut self, order: &Order) -> Option<Order> {
        if let Some(index) = self.orders.iter().position(|x| x.id == order.id) {
            let order = self.orders.remove(index);
            self.volume -= order.quantity;
            Some(order)
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.orders.len()
    }

    pub fn head(&self) -> Option<&Order> {
        self.orders.first()
    }

    pub fn update_head(&mut self, old_order: Order, new_order: Order) {
        self.volume -= old_order.quantity;
        self.volume += new_order.quantity;
        self.orders[0] = new_order;
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        order::{OrderType, TimeInForce},
        side::Side,
    };

    use super::*;

    #[test]
    fn test_append() {
        let mut queue = OrderQueue::new(100);
        let order = Order::new(
            1,
            1,
            Side::Buy,
            OrderType::Limit,
            TimeInForce::GTC,
            100,
            100,
        );
        let order = queue.append(order);
        assert_eq!(order.id, 1);
        assert_eq!(order.price, 100);
        assert_eq!(order.quantity, 100);
        assert_eq!(queue.volume, 100);
        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn test_remove() {
        let mut queue = OrderQueue::new(100);
        let order = Order::new(
            1,
            1,
            Side::Buy,
            OrderType::Limit,
            TimeInForce::GTC,
            100,
            100,
        );
        queue.append(order);
        let order = Order::new(
            2,
            1,
            Side::Buy,
            OrderType::Limit,
            TimeInForce::GTC,
            100,
            100,
        );
        queue.append(order);
        let order = Order::new(
            3,
            1,
            Side::Buy,
            OrderType::Limit,
            TimeInForce::GTC,
            100,
            100,
        );
        queue.append(order);
        let order = Order::new(
            4,
            1,
            Side::Buy,
            OrderType::Limit,
            TimeInForce::GTC,
            100,
            100,
        );
        queue.append(order);
        let order = Order::new(
            5,
            1,
            Side::Buy,
            OrderType::Limit,
            TimeInForce::GTC,
            100,
            100,
        );
        queue.append(order);
        let order = Order::new(
            6,
            1,
            Side::Buy,
            OrderType::Limit,
            TimeInForce::GTC,
            100,
            100,
        );
        queue.append(order);
        let order = Order::new(
            7,
            1,
            Side::Buy,
            OrderType::Limit,
            TimeInForce::GTC,
            100,
            100,
        );
        queue.append(order);
        let order = Order::new(
            8,
            1,
            Side::Buy,
            OrderType::Limit,
            TimeInForce::GTC,
            100,
            100,
        );
        queue.append(order);
        let order = Order::new(
            9,
            1,
            Side::Buy,
            OrderType::Limit,
            TimeInForce::GTC,
            100,
            100,
        );
        queue.append(order);
        let order = queue.remove(&Order::new(
            5,
            1,
            Side::Buy,
            OrderType::Limit,
            TimeInForce::GTC,
            100,
            100,
        ));
        assert_eq!(order.unwrap().id, 5);
        assert_eq!(queue.len(), 8);
        assert_eq!(queue.volume, 800);
    }

    #[test]
    fn test_len() {
        let mut queue = OrderQueue::new(100);
        let order = Order::new(
            1,
            1,
            Side::Buy,
            OrderType::Limit,
            TimeInForce::GTC,
            100,
            100,
        );
        queue.append(order);
        let order = Order::new(
            2,
            1,
            Side::Buy,
            OrderType::Limit,
            TimeInForce::GTC,
            100,
            100,
        );
        queue.append(order);
        let order = Order::new(
            3,
            1,
            Side::Buy,
            OrderType::Limit,
            TimeInForce::GTC,
            100,
            100,
        );
        queue.append(order);
        assert_eq!(queue.len(), 3);
    }

    #[test]
    fn test_head() {
        let mut queue = OrderQueue::new(100);
        let order = Order::new(
            1,
            1,
            Side::Buy,
            OrderType::Limit,
            TimeInForce::GTC,
            100,
            100,
        );
        queue.append(order);
        let order = Order::new(
            2,
            1,
            Side::Buy,
            OrderType::Limit,
            TimeInForce::GTC,
            100,
            100,
        );
        queue.append(order);
        let order = Order::new(
            3,
            1,
            Side::Buy,
            OrderType::Limit,
            TimeInForce::GTC,
            100,
            100,
        );
        queue.append(order);
        assert_eq!(queue.head().unwrap().id, 1);
    }

    #[test]
    fn test_update_head() {
        let mut queue = OrderQueue::new(100);
        let order = Order::new(
            1,
            1,
            Side::Buy,
            OrderType::Limit,
            TimeInForce::GTC,
            100,
            100,
        );
        queue.append(order);
        let order = Order::new(
            2,
            1,
            Side::Buy,
            OrderType::Limit,
            TimeInForce::GTC,
            100,
            100,
        );
        queue.append(order);
        let old_order = Order::new(
            3,
            1,
            Side::Buy,
            OrderType::Limit,
            TimeInForce::GTC,
            100,
            100,
        );
        let new_order = Order::new(
            3,
            1,
            Side::Buy,
            OrderType::Limit,
            TimeInForce::GTC,
            100,
            200,
        );
        assert_eq!(queue.len(), 2);
        assert_eq!(queue.head().unwrap().quantity, 100);
        queue.update_head(old_order, new_order);
        assert_eq!(queue.len(), 2);
        assert_eq!(queue.head().unwrap().quantity, 200);
    }
}
