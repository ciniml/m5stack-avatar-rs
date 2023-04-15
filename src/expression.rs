
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Expression {
    Angry,
    Sad,
    Doubt,
    Happy,
    Sleepy,
    Neutral,
}

pub trait ExpressionContext {
    fn expression(&self) -> Expression;
}