// side.rs
// create and export enum Side with variants Buy and Sell
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Side {
    Buy,
    Sell,
}
