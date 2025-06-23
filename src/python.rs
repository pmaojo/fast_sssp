use crate::algorithm::fast_sssp::FastSSSP;
use crate::graph::directed::DirectedGraph;
use ordered_float::OrderedFloat;
use pyo3::prelude::*;

#[pyclass]
pub struct PyGraph {
    graph: DirectedGraph<OrderedFloat<f64>>,
}

#[pymethods]
impl PyGraph {
    #[new]
    fn new() -> Self {
        PyGraph {
            graph: DirectedGraph::new(),
        }
    }

    fn add_vertex(&mut self) -> usize {
        self.graph.add_vertex()
    }

    fn add_edge(&mut self, from: usize, to: usize, weight: f64) -> bool {
        self.graph.add_edge(from, to, OrderedFloat(weight))
    }
}

#[pyclass]
pub struct PyFastSSSP {
    inner: FastSSSP,
}

#[pymethods]
impl PyFastSSSP {
    #[new]
    fn new() -> Self {
        PyFastSSSP {
            inner: FastSSSP::new(),
        }
    }

    fn compute_shortest_paths(
        &self,
        graph: &PyGraph,
        source: usize,
    ) -> PyResult<(Vec<Option<f64>>, Vec<Option<usize>>)> {
        let result = self
            .inner
            .compute_shortest_paths(&graph.graph, source)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        let distances = result
            .distances
            .into_iter()
            .map(|d| d.map(|v| v.0))
            .collect();
        let preds = result.predecessors;
        Ok((distances, preds))
    }
}

#[pymodule]
fn fast_sssp_py(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyGraph>()?;
    m.add_class::<PyFastSSSP>()?;
    Ok(())
}
