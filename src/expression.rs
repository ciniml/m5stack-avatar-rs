
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Expression {
    Angry,
    Sad,
    Happy,
    Sleepy,
}

pub trait ExpressionContext {
    fn expression(&self) -> Expression;
}