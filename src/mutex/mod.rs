use crate::bench::Mutex;

impl<M> Mutex<M> for std::sync::Mutex<M> {
    fn new(value: M) -> Self {
        Self::new(value)
    }

    fn with<F: FnOnce(&mut M)>(&self, f: F) {
        f(&mut self.lock().unwrap())
    }
}

impl<M> Mutex<M> for parking_lot::Mutex<M> {
    fn new(value: M) -> Self {
        Self::new(value)
    }

    fn with<F: FnOnce(&mut M)>(&self, f: F) {
        f(&mut self.lock())
    }
}
