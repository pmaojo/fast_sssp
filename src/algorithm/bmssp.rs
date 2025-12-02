use num_traits::{Float, Zero};
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};
use std::fmt::Debug;
use std::marker::PhantomData;

use crate::data_structures::BlockList;
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

    /// Set of vertices with computed shortest paths (dist < new_bound)
    pub vertices: Vec<usize>,

    /// Set of vertices that were reached but have dist >= new_bound
    /// These must be re-inserted into the BlockList at the upper level
    pub overflow: Vec<(usize, W)>,
}

impl<W, G> BMSSP<W, G>
where
    W: Float + Zero + Debug + Copy + Ord,
    G: Graph<W>,
{
    /// Create a new BMSSP algorithm instance with automatically calculated parameters
    pub fn new(vertex_count: usize) -> Self {
        // Calculate parameters k and t using log2
        let log_n = (vertex_count as f64).log2();

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
        W: Ord,
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
        // Algorithm 3, line 12 caps |U| at k * 2^{l t}
        while result_vertices.len() < self.k * 2usize.pow(level as u32 * self.t as u32)
            && !block_list.is_empty()
        {
            // Pull smallest vertices from D with their bound
            let (si, bi) = block_list.pull(block_size);

            if si.is_empty() {
                // If we pulled nothing valid, but block list was not empty,
                // it means everything remaining was stale or filtered out.
                // We should continue to next iteration which will pull more.
                // But pull() updates internal state, so eventually is_empty() becomes true.
                continue;
            }

            // Recursively call BMSSP
            let result = self.execute(graph, level - 1, bi, &si, distances, predecessors)?;
            let ui = result.vertices;
            let new_bound = result.new_bound;
            let overflow_i = result.overflow;

            // Add vertices to result set
            for &vertex in &ui {
                result_vertices.insert(vertex);
            }

            // Handle overflow vertices from recursive call
            // These are vertices with dist >= new_bound.
            // Some might be < bi (should be prepended), some >= bi (should be inserted)
            let mut overflow_prepend = Vec::new();
            for (v, dist) in overflow_i {
                if dist < bi && dist < bound {
                    overflow_prepend.push((v, dist));
                } else if dist >= bi && dist < bound {
                    block_list.insert(v, dist);
                }
            }
            if !overflow_prepend.is_empty() {
                block_list.batch_prepend(overflow_prepend);
            }

            // Relax edges from ui
            let mut batch_prepend_set = Vec::new();
            for &u in &ui {
                for (v, weight) in graph.outgoing_edges(u) {
                    let potential_dist = distances[u] + weight;

                    // Relax edge if shorter path found OR equal path found (re-discovery)
                    if potential_dist <= distances[v] {
                        if potential_dist < distances[v] {
                            distances[v] = potential_dist;
                            predecessors[v] = Some(u);
                        }

                        // Only add to block list if not already processed in this level
                        // Note: result_vertices contains vertices < new_bound
                        // But potential_dist might be >= new_bound
                        if !result_vertices.contains(&v) {
                            // Add to appropriate set based on distance
                            if potential_dist >= bi && potential_dist < bound {
                                block_list.insert(v, potential_dist);
                            } else if potential_dist >= new_bound && potential_dist < bi {
                                batch_prepend_set.push((v, potential_dist));
                            }
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

        // Drain remaining elements from BlockList as overflow
        // These are vertices that were reached but >= prev_bound (or bound)
        let mut overflow_vec = block_list.drain_all();

        // Filter overflow to ensure they are strictly < bound
        overflow_vec.retain(|&(_, dist)| dist < bound);

        Ok(BMSSPResult {
            // Return the smallest bound encountered.
            new_bound: std::cmp::min(bound, prev_bound),
            vertices: result_vec,
            overflow: overflow_vec,
        })
    }

    /// Base case of the BMSSP algorithm (level = 0)
    fn base_case(
        &self,
        graph: &G,
        bound: W,
        sources: &[usize],
        distances: &mut Vec<W>,
        predecessors: &mut Vec<Option<usize>>,
    ) -> Result<BMSSPResult<W>>
    where
        W: Ord,
    {
        // Early termination for empty sources
        if sources.is_empty() {
            return Ok(BMSSPResult {
                new_bound: bound,
                vertices: Vec::new(),
                overflow: Vec::new(),
            });
        }

        // Pre-allocate with capacity
        let mut heap = BinaryHeap::with_capacity(self.k * 4);

        // Use HashSet for visited tracking to avoid O(N) allocation
        let mut visited = HashSet::with_capacity(self.k * 4);

        // Counter for processed vertices to enforce the k-limit
        let mut processed_count = 0;

        // Add all sources to the heap
        for &source in sources {
            if !visited.contains(&source) {
                heap.push(std::cmp::Reverse((distances[source], source)));
                visited.insert(source);
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

            // Process outgoing edges
            for (v, weight) in graph.outgoing_edges(u) {
                let new_dist = dist_u + weight;

                // Only update if the new distance is better (or equal) and within the bound
                if new_dist <= bound && new_dist <= distances[v] {
                    if new_dist < distances[v] {
                        distances[v] = new_dist;
                        predecessors[v] = Some(u);
                    }

                    heap.push(std::cmp::Reverse((new_dist, v)));

                    if !visited.contains(&v) {
                        visited.insert(v);
                    }
                }
            }
        }

        // Collect results from visited set
        let mut collected_vertices: Vec<usize> = visited.into_iter().collect();

        // Determine new boundary
        let new_bound = self.calculate_new_bound(
            collected_vertices.len(),
            bound,
            &collected_vertices,
            distances,
        );

        // Filter into vertices (< new_bound) and overflow (>= new_bound but < bound)
        let mut result_vec = Vec::new();
        let mut overflow_vec = Vec::new();

        for v in collected_vertices {
            if distances[v] < new_bound {
                result_vec.push(v);
            } else if distances[v] < bound {
                overflow_vec.push((v, distances[v]));
            }
        }

        Ok(BMSSPResult {
            new_bound,
            vertices: result_vec,
            overflow: overflow_vec,
        })
    }

    // Removed optimized mini_dijkstra separate path for simplicity and correctness
    // as it duplicated logic and needs similar fixes.
    // The general base_case handles single source efficiently enough with HashSet.

    /// Helper function to process a batch of edges (removed in favor of simpler loop in base_case)
    // ...

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
        let mut discovered_distances: Vec<W> = vertices.iter().map(|&v| distances[v]).collect();
        discovered_distances.sort();
        // Safe because result_size > self.k
        discovered_distances[self.k]
    }

    /// Find pivots as described in the paper
    fn find_pivots(
        &self,
        graph: &G,
        bound: W,
        sources: &[usize],
        distances: &mut Vec<W>,
        predecessors: &mut Vec<Option<usize>>,
    ) -> Result<(Vec<usize>, Vec<usize>)>
    where
        W: Ord,
    {
        // Initialize work set with sources
        let mut work_set = sources.to_vec();
        let mut frontier = VecDeque::new();

        // Add all sources to the frontier
        for &s in sources {
            frontier.push_back(s);
        }

        // Track visited vertices using HashSet to avoid O(N) allocation
        let mut visited = HashSet::new();
        for &s in sources {
            visited.insert(s);
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
                        if !visited.contains(&v) {
                            visited.insert(v);
                            work_set.push(v);
                            frontier.push_back(v);
                        }
                    }
                }
            }

            steps += 1;
        }

        // Use the current frontier as the set of pivots
        // The frontier contains the vertices where the "k-step" search stopped.
        // Resuming the search from these vertices ensures we cover the rest of the graph.
        let pivots: Vec<usize> = frontier.into_iter().collect();

        Ok((pivots, work_set))
    }
}
