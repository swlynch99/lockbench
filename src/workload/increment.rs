use crate::bench::Workload;
use crate::config::Config;


#[derive(Default)]
pub struct Increment;

impl Workload for Increment {
    type State = u64;

    fn new(_: &Config) -> Self {
        Self::default()
    }

    fn new_state(_: &Config) -> Self::State {
        0
    }

    fn work(&mut self, state: &mut Self::State) {
        *state += 1;
    }
}
