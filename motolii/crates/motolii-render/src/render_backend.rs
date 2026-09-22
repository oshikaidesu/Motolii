//! Execution cassette contract for backend-neutral render work.

use std::collections::HashMap;

use crate::render_graph::{RenderGraph, RenderWork, ResourceId, ResourceSource};

pub trait RenderBackend {
    type Output;
    type Error;
    fn execute(&mut self, graph: &RenderGraph) -> Result<Self::Output, Self::Error>;
}

#[derive(Default)]
pub struct CountingBackend;
impl RenderBackend for CountingBackend {
    type Output = usize;
    type Error = std::convert::Infallible;
    fn execute(&mut self, graph: &RenderGraph) -> Result<Self::Output, Self::Error> { Ok(graph.work().len()) }
}

/// Minimal production-resource cassette used while GpuScene remains the visual
/// oracle. The host supplies concrete resource operations; this executor only
/// understands RenderWork and ResourceId.
pub struct ResourceBackend<'a, R, E> {
    pub raster: &'a mut dyn FnMut(&ResourceSource) -> Result<R, E>,
    pub transfer: &'a mut dyn FnMut(&ResourceSource) -> Result<R, E>,
}

impl<R, E> ResourceBackend<'_, R, E> {
    pub fn execute_resources(&mut self, graph: &RenderGraph) -> Result<HashMap<ResourceId, R>, E> {
        let mut resources = HashMap::new();
        for work in graph.work() {
            match work {
                RenderWork::Raster { source, output } => {
                    resources.insert(*output, (self.raster)(source)?);
                }
                RenderWork::Transfer { source, output } => {
                    resources.insert(*output, (self.transfer)(source)?);
                }
                RenderWork::Compute { .. } | RenderWork::Filter { .. } | RenderWork::Composite { .. } => {}
            }
        }
        Ok(resources)
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

    #[test]
    fn resource_executor_dispatches_transfer_without_knowing_media_semantics() {
        let graph = RenderGraph::new(vec![RenderWork::Transfer {
            source: ResourceSource::Media { path: "still.png".into() },
            output: ResourceId(4),
        }]);
        let mut raster = |_| -> Result<String, ()> { unreachable!() };
        let mut transfer = |source: &ResourceSource| -> Result<String, ()> {
            match source { ResourceSource::Media { path } => Ok(path.clone()), _ => Err(()) }
        };
        let mut backend = ResourceBackend { raster: &mut raster, transfer: &mut transfer };
        assert_eq!(backend.execute_resources(&graph).unwrap().remove(&ResourceId(4)).as_deref(), Some("still.png"));
    }
}
