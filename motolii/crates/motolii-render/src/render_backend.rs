//! Execution cassette contract for backend-neutral render work.

use crate::render_graph::RenderGraph;

pub trait RenderBackend {
    type Output;
    type Error;

    fn execute(&mut self, graph: &RenderGraph) -> Result<Self::Output, Self::Error>;
}

/// Structural backend used by tests and migration tooling. It proves that the
/// RenderGraph can be consumed without wgpu or Motolii semantic evaluation.
#[derive(Default)]
pub struct CountingBackend;

impl RenderBackend for CountingBackend {
    type Output = usize;
    type Error = std::convert::Infallible;

    fn execute(&mut self, graph: &RenderGraph) -> Result<Self::Output, Self::Error> {
        Ok(graph.work().len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cassette_consumes_render_graph_without_semantic_graph() {
        let mut backend = CountingBackend;
        assert_eq!(backend.execute(&RenderGraph::default()).unwrap(), 0);
    }
}
