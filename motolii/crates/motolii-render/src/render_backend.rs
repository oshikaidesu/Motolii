//! Execution cassette contract for backend-neutral render work.

use crate::render_graph::RenderGraph;

pub trait RenderBackend {
    type Output;
    type Error;
    fn execute(&mut self, graph: &RenderGraph) -> Result<Self::Output, Self::Error>;
}

/// Counts contributions and composed outputs; proves a backend needs nothing
/// but the graph.
#[derive(Default)]
pub struct CountingBackend;
impl RenderBackend for CountingBackend {
    type Output = (usize, usize);
    type Error = std::convert::Infallible;
    fn execute(&mut self, graph: &RenderGraph) -> Result<Self::Output, Self::Error> {
        Ok((graph.layers.len(), graph.output.len()))
    }
}
