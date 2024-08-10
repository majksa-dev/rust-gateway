pub mod time;

pub trait Also<T> {
    fn also(self, f: impl FnOnce(&T)) -> T;
}

impl<T> Also<T> for T {
    fn also(self, f: impl FnOnce(&T)) -> T {
        f(&self);
        self
    }
}
