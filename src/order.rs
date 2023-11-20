use crate::side::Side;
use chrono::{DateTime, Utc};
use std::usize;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OrderType {
    Limit,
    Market,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeInForce {
    GTC, // Good till cancelled
    IOC, // Immediate or cancel
    GTD, // Good till date
    FOK, // Fill or kill
}

#[derive(Debug, Clone)]
pub struct Order {
    pub id: u32,
    pub user_id: u32,
    pub side: Side,
    pub order_type: OrderType,
    pub time_in_force: TimeInForce,
    pub price: usize,
    pub quantity: usize,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Order {
    pub fn new(
        id: u32,
        user_id: u32,
        side: Side,
        order_type: OrderType,
        time_in_force: TimeInForce,
        price: usize,
        quantity: usize,
    ) -> Self {
        Self {
            id,
            user_id,
            side,
            order_type,
            time_in_force,
            price,
            quantity,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_new() {
        let order = Order::new(
            1,
            1,
            Side::Buy,
            OrderType::Limit,
            TimeInForce::GTC,
            100,
            100,
        );
        assert_eq!(order.id, 1);
        assert_eq!(order.side, Side::Buy);
        assert_eq!(order.order_type, OrderType::Limit);
        assert_eq!(order.time_in_force, TimeInForce::GTC);
        assert_eq!(order.price, 100);
        assert_eq!(order.quantity, 100);
    }

    #[test]
    fn test_all_order_types() {
        let order_types = vec![OrderType::Limit, OrderType::Market];
        for order_type in order_types {
            assert!(order_type == OrderType::Limit || order_type == OrderType::Market);
        }
    }

    #[test]
    fn test_all_time_in_force() {
        let time_in_forces = vec![
            TimeInForce::GTC,
            TimeInForce::IOC,
            TimeInForce::GTD,
            TimeInForce::FOK,
        ];
        for time_in_force in time_in_forces {
            assert!(
                (time_in_force == TimeInForce::GTC
                    || time_in_force == TimeInForce::IOC
                    || time_in_force == TimeInForce::GTD
                    || time_in_force == TimeInForce::FOK)
            );
        }
    }
}
