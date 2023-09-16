use crate::util::wincounter::wins::Wins;

pub trait Hands<T> {
    fn wins(&self) -> Wins;
}
