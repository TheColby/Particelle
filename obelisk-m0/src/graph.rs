//! Control-thread graph descriptions for the fixed M0 signal chain.

/// Nodes available in the M0 fixed graph. Their order is intentional.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphNode {
    Wavetable,
    ModalBody,
    Filter,
    Delay,
    DiffuseReverb,
    Limiter,
}

const FIXED_CHAIN: [GraphNode; 6] = [
    GraphNode::Wavetable,
    GraphNode::ModalBody,
    GraphNode::Filter,
    GraphNode::Delay,
    GraphNode::DiffuseReverb,
    GraphNode::Limiter,
];

/// A declarative graph submitted on the control thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct M0GraphSpec {
    pub nodes: Vec<GraphNode>,
}

impl Default for M0GraphSpec {
    fn default() -> Self {
        Self {
            nodes: FIXED_CHAIN.to_vec(),
        }
    }
}

impl M0GraphSpec {
    /// Validates a graph once, before audio starts. The audio thread only sees
    /// the resulting immutable descriptor.
    pub fn compile(&self) -> Result<CompiledGraph, GraphCompileError> {
        if self.nodes.as_slice() != FIXED_CHAIN {
            return Err(GraphCompileError::UnsupportedTopology);
        }
        Ok(CompiledGraph { nodes: FIXED_CHAIN })
    }
}

/// Immutable descriptor exposed for host inspection and future graph tooling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledGraph {
    nodes: [GraphNode; 6],
}

impl CompiledGraph {
    pub fn nodes(&self) -> &[GraphNode] {
        &self.nodes
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum GraphCompileError {
    #[error("M0 only supports its fixed, realtime-qualified signal topology")]
    UnsupportedTopology,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factory_graph_compiles() {
        assert_eq!(
            M0GraphSpec::default().compile().unwrap().nodes(),
            FIXED_CHAIN
        );
    }

    #[test]
    fn reordered_graph_is_rejected() {
        let mut spec = M0GraphSpec::default();
        spec.nodes.swap(0, 1);
        assert_eq!(spec.compile(), Err(GraphCompileError::UnsupportedTopology));
    }
}
