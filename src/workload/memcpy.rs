use crate::bench::Workload;

pub struct Memcpy(Vec<u8>);

impl Workload for Memcpy {
    type State = Vec<u8>;

    fn new(config: &crate::config::Config) -> Self {
        let data = std::iter::repeat(0..=255)
            .flatten()
            .take(config.workloads.memcpy.bytes)
            .collect();

        Self(data)
    }

    fn new_state(_: &crate::config::Config) -> Self::State {
        Vec::new()
    }

    fn work(&mut self, state: &mut Self::State) {
        state.clear();
        state.extend_from_slice(&self.0);
    }
}
