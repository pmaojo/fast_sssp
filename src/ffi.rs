use crate::algorithm::fast_sssp::FastSSSP;
use crate::graph::directed::DirectedGraph;
use ordered_float::OrderedFloat;

#[repr(C)]
pub struct FfiGraph {
    graph: DirectedGraph<OrderedFloat<f64>>,
}

#[no_mangle]
pub extern "C" fn fsssp_graph_new() -> *mut FfiGraph {
    Box::into_raw(Box::new(FfiGraph {
        graph: DirectedGraph::new(),
    }))
}

#[no_mangle]
pub extern "C" fn fsssp_graph_add_vertex(g: *mut FfiGraph) -> usize {
    unsafe { &mut *g }.graph.add_vertex()
}

#[no_mangle]
pub extern "C" fn fsssp_graph_add_edge(
    g: *mut FfiGraph,
    from: usize,
    to: usize,
    weight: f64,
) -> bool {
    unsafe { &mut *g }
        .graph
        .add_edge(from, to, OrderedFloat(weight))
}

#[no_mangle]
pub extern "C" fn fsssp_graph_free(g: *mut FfiGraph) {
    if !g.is_null() {
        unsafe {
            drop(Box::from_raw(g));
        }
    }
}

#[repr(C)]
pub struct FfiResult {
    distances: *mut f64,
    predecessors: *mut usize,
    len: usize,
}

#[no_mangle]
pub extern "C" fn fsssp_result_free(res: *mut FfiResult) {
    if !res.is_null() {
        unsafe {
            if !(*res).distances.is_null() {
                drop(Vec::from_raw_parts(
                    (*res).distances,
                    (*res).len,
                    (*res).len,
                ));
            }
            if !(*res).predecessors.is_null() {
                drop(Vec::from_raw_parts(
                    (*res).predecessors,
                    (*res).len,
                    (*res).len,
                ));
            }
            drop(Box::from_raw(res));
        }
    }
}

#[no_mangle]
pub extern "C" fn fsssp_compute_shortest_paths(
    g: *const FfiGraph,
    source: usize,
) -> *mut FfiResult {
    let graph = unsafe { &(*g).graph };
    let alg = FastSSSP::new();
    match alg.compute_shortest_paths(graph, source) {
        Ok(result) => {
            let len = result.distances.len();
            let mut dist_vec: Vec<f64> = result
                .distances
                .into_iter()
                .map(|o| o.map(|d| d.0).unwrap_or(f64::INFINITY))
                .collect();
            let mut pred_vec: Vec<usize> = result
                .predecessors
                .into_iter()
                .map(|p| p.unwrap_or(usize::MAX))
                .collect();
            let dist_ptr = dist_vec.as_mut_ptr();
            let pred_ptr = pred_vec.as_mut_ptr();
            std::mem::forget(dist_vec);
            std::mem::forget(pred_vec);
            Box::into_raw(Box::new(FfiResult {
                distances: dist_ptr,
                predecessors: pred_ptr,
                len,
            }))
        }
        Err(_) => std::ptr::null_mut(),
    }
}
