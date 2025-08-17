use num_traits::{Float, Zero};
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::fmt::Debug;
use std::marker::PhantomData;

use crate::data_structures::{BinaryHeapWrapper, BlockList};
use crate::graph::Graph;
use crate::{Error, Result};

/// Implementation of the Bounded Multi-Source Shortest Path (BMSSP) algorithm
/// as described in the paper "Breaking the Sorting Barrier for Directed Single-Source Shortest Paths"
#[derive(Debug)]
pub struct BMSSP<W, G>
where
    W: Float + Zero + Debug + Copy,
    G: Graph<W>,
{
    /// Parameter k = log^(1/3)(n)
    k: usize,

    /// Parameter t = log^(2/3)(n)
    t: usize,

    /// Graph type marker
    _graph_marker: PhantomData<G>,

    /// Weight type marker
    _weight_marker: PhantomData<W>,
}

/// Result from a BMSSP execution
#[derive(Debug)]
pub struct BMSSPResult<W>
where
    W: Float + Zero + Debug + Copy,
{
    /// New boundary value
    pub new_bound: W,

    /// Set of vertices with computed shortest paths
    pub vertices: Vec<usize>,
}

impl<W, G> BMSSP<W, G>
where
    W: Float + Zero + Debug + Copy + Ord,
    G: Graph<W>,
{
    /// Create a new BMSSP algorithm instance with automatically calculated parameters
    pub fn new(vertex_count: usize) -> Self {
        // Calculate parameters k and t
        let log_n = (vertex_count as f64).ln();

        let k = (log_n.powf(1.0 / 3.0)).ceil() as usize;
        let t = (log_n.powf(2.0 / 3.0)).ceil() as usize;

        // Ensure k and t are at least 2
        let k = k.max(2);
        let t = t.max(2);

        BMSSP {
            k,
            t,
            _graph_marker: PhantomData,
            _weight_marker: PhantomData,
        }
    }

    /// Create a new BMSSP algorithm instance with explicit parameters
    pub fn new_with_params(_vertex_count: usize, k: usize, t: usize) -> Self {
        // Ensure k and t are at least 2
        let k = k.max(2);
        let t = t.max(2);

        println!("Creating BMSSP with parameters: k={}, t={}", k, t);

        BMSSP {
            k,
            t,
            _graph_marker: PhantomData,
            _weight_marker: PhantomData,
        }
    }

    /// Execute the BMSSP algorithm as described in the paper
    pub fn execute(
        &self,
        graph: &G,
        level: usize,
        bound: W,
        sources: &[usize],
        distances: &mut Vec<W>,
        predecessors: &mut Vec<Option<usize>>,
    ) -> Result<BMSSPResult<W>>
    where
        W: Ord, // Add explicit Ord trait bound here
    {
        if sources.is_empty() {
            return Err(Error::AlgorithmError("Empty sources set".to_string()));
        }

        // Base case (level = 0)
        if level == 0 {
            return self.base_case(graph, bound, sources, distances, predecessors);
        }

        // Find pivots
        let (pivots, work_set) =
            self.find_pivots(graph, bound, sources, distances, predecessors)?;

        // Initialize data structure D from Lemma 3.3
        let block_size = 2usize.pow((level - 1) as u32 * self.t as u32);
        let mut block_list = BlockList::new(block_size, bound);

        // Add pivots to D
        for &pivot in &pivots {
            block_list.insert(pivot, distances[pivot]);
        }

        // Initialize result set and previous boundary
        let mut result_vertices = HashSet::new();
        let mut prev_bound = if !pivots.is_empty() {
            pivots
                .iter()
                .map(|&p| distances[p])
                .fold(W::max_value(), |a, b| if a < b { a } else { b })
        } else {
            bound
        };

        // Add all sources to result vertices
        for &source in sources {
            result_vertices.insert(source);
        }

        // Main iteration loop
        let mut _iteration = 0;
        // Algorithm 3, line 12 caps |U| at k * 2^{l t}
        while result_vertices.len() < self.k * 2usize.pow(level as u32 * self.t as u32)
            && !block_list.is_empty()
        {
            _iteration += 1;

            // Pull smallest vertices from D with their bound
            let (si, bi) = block_list.pull(block_size);

            // Recursively call BMSSP
            let result = self.execute(graph, level - 1, bi, &si, distances, predecessors)?;
            let ui = result.vertices;
            let new_bound = result.new_bound;

            // Add vertices to result set
            for &vertex in &ui {
                result_vertices.insert(vertex);
            }

            // Relax edges from ui
            let mut batch_prepend_set = Vec::new();
            for &u in &ui {
                for (v, weight) in graph.outgoing_edges(u) {
                    let potential_dist = distances[u] + weight;

                    if potential_dist < distances[v] {
                        distances[v] = potential_dist;
                        predecessors[v] = Some(u);

                        // Add to appropriate set based on distance
                        if potential_dist >= bi && potential_dist < bound {
                            block_list.insert(v, potential_dist);
                        } else if potential_dist >= new_bound && potential_dist < bi {
                            batch_prepend_set.push((v, potential_dist));
                        }
                    }
                }
            }

            // Batch prepend vertices with distances in [new_bound, bi)
            block_list.batch_prepend(batch_prepend_set);

            // Also batch prepend vertices from Si with distances in [new_bound, bi)
            let si_reinsert = si
                .iter()
                .filter(|&&v| distances[v] >= new_bound && distances[v] < bi)
                .map(|&v| (v, distances[v]))
                .collect::<Vec<_>>();

            if !si_reinsert.is_empty() {
                block_list.batch_prepend(si_reinsert);
            }

            // Update previous bound
            prev_bound = new_bound;

            // Check for early termination condition
            if result_vertices.len() >= self.k * 2usize.pow(level as u32 * self.t as u32) {
                break;
            }
        }

        // Add vertices from work_set with distance < prev_bound
        for &v in &work_set {
            if distances[v] < prev_bound {
                result_vertices.insert(v);
            }
        }

        // Convert result set to vector
        let result_vec = result_vertices.into_iter().collect::<Vec<_>>();

        Ok(BMSSPResult {
            // Return the smallest bound encountered. If no vertices were
            // processed (`block_list` empty), `prev_bound` remains equal to
            // `bound`, so the minimum is the original bound.
            new_bound: std::cmp::min(bound, prev_bound),
            vertices: result_vec,
        })
    }

    /// Base case of the BMSSP algorithm (level = 0)
    /// This is an optimized implementation of the mini-Dijkstra algorithm
    /// that limits the number of vertices processed based on the k parameter
    fn base_case(
        &self,
        graph: &G,
        bound: W,
        sources: &[usize],
        distances: &mut Vec<W>,
        predecessors: &mut Vec<Option<usize>>,
    ) -> Result<BMSSPResult<W>>
    where
        W: Ord, // Add explicit Ord trait bound here
    {
        println!(
            "BMSSP base_case called with {} sources and bound {:?}",
            sources.len(),
            bound
        );

        // Early termination for empty sources
        if sources.is_empty() {
            return Ok(BMSSPResult {
                new_bound: bound,
                vertices: Vec::new(),
            });
        }

        // For single source with small k, use optimized mini-Dijkstra
        if sources.len() == 1 {
            return self.mini_dijkstra(graph, sources[0], bound, distances, predecessors);
        }

        // Pre-allocate with capacity to avoid reallocations
        let mut heap = BinaryHeap::with_capacity(self.k * 4);
        let mut result_vertices = Vec::with_capacity(self.k * 2);

        // Use a bitmap for visited tracking (more efficient than a full boolean vector)
        let vertex_count = graph.vertex_count();
        let mut visited = vec![false; vertex_count];

        // Counter for processed vertices to enforce the k-limit
        let mut processed_count = 0;

        // Add all sources to the heap
        for &source in sources {
            if !visited[source] {
                heap.push(std::cmp::Reverse((distances[source], source)));
                result_vertices.push(source);
                visited[source] = true;
            }
        }

        // Run bounded Dijkstra's algorithm
        while let Some(std::cmp::Reverse((dist_u, u))) = heap.pop() {
            // Skip if we've already found a better path or reached the bound
            if dist_u > distances[u] || dist_u > bound {
                continue;
            }

            // Increment processed count and check limit
            processed_count += 1;
            if processed_count > self.k * 2 {
                // We've processed enough vertices, stop early
                break;
            }

            // Process outgoing edges using block processing for better cache efficiency
            let mut edge_buffer = Vec::with_capacity(8); // Small buffer for edge batching

            for (v, weight) in graph.outgoing_edges(u) {
                edge_buffer.push((v, weight));

                // Process edges in small batches for better cache locality
                if edge_buffer.len() >= 8 {
                    self.process_edge_batch(
                        &edge_buffer,
                        u,
                        dist_u,
                        bound,
                        distances,
                        predecessors,
                        &mut heap,
                        &mut visited,
                        &mut result_vertices,
                    );
                    edge_buffer.clear();
                }
            }

            // Process remaining edges
            if !edge_buffer.is_empty() {
                self.process_edge_batch(
                    &edge_buffer,
                    u,
                    dist_u,
                    bound,
                    distances,
                    predecessors,
                    &mut heap,
                    &mut visited,
                    &mut result_vertices,
                );
            }
        }

        // Determine new boundary using a more efficient approach
        let new_bound =
            self.calculate_new_bound(result_vertices.len(), bound, &result_vertices, distances);

        // Filter out vertices with distances >= new_bound
        // Use drain_filter when it becomes stable for better performance
        let result_vec = result_vertices
            .into_iter()
            .filter(|&v| distances[v] < new_bound)
            .collect::<Vec<_>>();

        Ok(BMSSPResult {
            new_bound,
            vertices: result_vec,
        })
    }

    /// Helper function to process a batch of edges for better cache efficiency
    #[inline]
    fn process_edge_batch(
        &self,
        edge_batch: &[(usize, W)],
        u: usize,
        dist_u: W,
        bound: W,
        distances: &mut Vec<W>,
        predecessors: &mut Vec<Option<usize>>,
        heap: &mut BinaryHeap<std::cmp::Reverse<(W, usize)>>,
        visited: &mut Vec<bool>,
        result_vertices: &mut Vec<usize>,
    ) {
        for &(v, weight) in edge_batch {
            let new_dist = dist_u + weight;

            // Only update if the new distance is better and within the bound
            if new_dist <= bound && new_dist < distances[v] {
                distances[v] = new_dist;
                predecessors[v] = Some(u);
                heap.push(std::cmp::Reverse((new_dist, v)));

                // Add to result vertices if not already visited
                if !visited[v] {
                    result_vertices.push(v);
                    visited[v] = true;
                }
            }
        }
    }

    /// Calculate the new boundary value based on the result set size
    #[inline]
    fn calculate_new_bound(
        &self,
        result_size: usize,
        bound: W,
        vertices: &[usize],
        distances: &[W],
    ) -> W {
        // If we have not discovered more than k vertices, keep the current bound
        if result_size <= self.k {
            return bound;
        }

        // Compute the (k+1)-th smallest distance among discovered vertices
        // This ensures vertices with distance strictly less than this threshold are kept
        let mut discovered_distances: Vec<W> = vertices.iter().map(|&v| distances[v]).collect();
        discovered_distances.sort();
        // Safe because result_size > self.k
        discovered_distances[self.k]
    }

    /// Optimized mini-Dijkstra for the single-source case
    fn mini_dijkstra(
        &self,
        graph: &G,
        source: usize,
        bound: W,
        distances: &mut Vec<W>,
        predecessors: &mut Vec<Option<usize>>,
    ) -> Result<BMSSPResult<W>> {
        let mut heap = BinaryHeap::with_capacity(self.k * 2);
        let mut result_vertices = Vec::with_capacity(self.k * 2);
        let mut visited = vec![false; graph.vertex_count()];

        // Add source to heap and result
        heap.push(std::cmp::Reverse((distances[source], source)));
        result_vertices.push(source);
        visited[source] = true;

        // Process at most k vertices
        let mut processed_count = 0;

        while let Some(std::cmp::Reverse((dist_u, u))) = heap.pop() {
            if dist_u > distances[u] || dist_u > bound {
                continue;
            }

            processed_count += 1;
            if processed_count > self.k {
                break;
            }

            // Use direct iteration for better performance in the single-source case
            for (v, weight) in graph.outgoing_edges(u) {
                let new_dist = dist_u + weight;

                if new_dist <= bound && new_dist < distances[v] {
                    distances[v] = new_dist;
                    predecessors[v] = Some(u);
                    heap.push(std::cmp::Reverse((new_dist, v)));

                    if !visited[v] {
                        result_vertices.push(v);
                        visited[v] = true;
                    }
                }
            }
        }

        let new_bound =
            self.calculate_new_bound(result_vertices.len(), bound, &result_vertices, distances);

        let result_vec = result_vertices
            .into_iter()
            .filter(|&v| distances[v] < new_bound)
            .collect::<Vec<_>>();

        Ok(BMSSPResult {
            new_bound,
            vertices: result_vec,
        })
    }

    /// Find pivots as described in the paper using a more efficient algorithm
    /// This implementation follows the exact procedure from the paper to identify pivots
    fn find_pivots(
        &self,
        graph: &G,
        bound: W,
        sources: &[usize],
        distances: &mut Vec<W>,
        predecessors: &mut Vec<Option<usize>>,
    ) -> Result<(Vec<usize>, Vec<usize>)>
    where
        W: Ord, // Add explicit Ord trait bound here
    {
        use std::collections::VecDeque;

        println!(
            "Finding pivots from {} sources with bound {:?}",
            sources.len(),
            bound
        );

        // Initialize work set with sources
        let mut work_set = sources.to_vec();
        let mut frontier = VecDeque::new();

        // Add all sources to the frontier
        for &s in sources {
            frontier.push_back(s);
        }

        // Track visited vertices to avoid duplicates in work_set
        let mut visited = vec![false; graph.vertex_count()];
        for &s in sources {
            visited[s] = true;
        }

        // Perform k steps of relaxation (Bellman-Ford-like)
        let mut steps = 0;
        while !frontier.is_empty() && steps < self.k {
            let level_size = frontier.len();

            // Process all vertices at the current level
            for _ in 0..level_size {
                let u = frontier.pop_front().unwrap();

                // Relax all outgoing edges
                for (v, weight) in graph.outgoing_edges(u) {
                    let potential_dist = distances[u] + weight;

                    if potential_dist < distances[v] && potential_dist < bound {
                        distances[v] = potential_dist;
                        predecessors[v] = Some(u);

                        // Add to work_set and frontier if not visited
                        if !visited[v] {
                            visited[v] = true;
                            work_set.push(v);
                            frontier.push_back(v);
                        }
                    }
                }
            }

            steps += 1;
        }

        println!("Work set size after {} steps: {}", steps, work_set.len());

        // If work_set is small, return all sources as pivots
        if work_set.len() <= self.k * sources.len() {
            println!("Work set is small, using all sources as pivots");
            return Ok((sources.to_vec(), work_set));
        }

        // Build shortest path forest
        let mut forest = HashMap::new();
        let mut tree_sizes = HashMap::new();

        // Initialize tree sizes for sources
        for &s in sources {
            tree_sizes.insert(s, 1); // Start with size 1 (just the root)
        }

        // Convert sources to a HashSet for constant-time lookups when
        // checking if a vertex is one of the sources during root finding.
        let source_set: HashSet<usize> = sources.iter().copied().collect();

        // Build the forest structure
        for &v in &work_set {
            if let Some(pred) = predecessors[v] {
                if pred != v {
                    // Skip self-loops
                    forest.entry(pred).or_insert_with(Vec::new).push(v);

                    // Increment tree size for the root of this tree
                    let mut current = pred;
                    let mut root = current;

                    // Find the root of this tree using constant-time source lookup
                    while let Some(parent) = predecessors[current] {
                        if parent == current || source_set.contains(&current) {
                            root = current;
                            break;
                        }
                        current = parent;
                    }

                    // Increment tree size for the root
                    *tree_sizes.entry(root).or_insert(1) += 1;
                }
            }
        }

        // Find pivots (sources with large trees)
        let mut pivots = Vec::new();
        for &s in sources {
            if let Some(&size) = tree_sizes.get(&s) {
                if size >= self.k {
                    pivots.push(s);
                }
            }
        }

        // If no pivots found, use the source with the largest tree
        if pivots.is_empty() && !sources.is_empty() {
            let best_source = sources
                .iter()
                .max_by_key(|&&s| tree_sizes.get(&s).unwrap_or(&0))
                .copied()
                .unwrap();

            pivots.push(best_source);
            println!(
                "No large trees found, using source {} with tree size {}",
                best_source,
                tree_sizes.get(&best_source).unwrap_or(&0)
            );
        }

        println!(
            "Found {} pivots from {} sources",
            pivots.len(),
            sources.len()
        );
        Ok((pivots, work_set))
    }
}
