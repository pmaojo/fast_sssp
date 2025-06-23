# Fast SSSP

A Rust implementation of the O(m log^(2/3) n) Single-Source Shortest Paths algorithm for directed graphs with non-negative edge weights, as described in the paper:

> "Breaking the Sorting Barrier for Directed Single-Source Shortest Paths" by Ran Duan, Jiayi Mao, Xiao Mao, Xinkai Shu, and Longhui Yin (2025)

## Overview

This library implements a deterministic algorithm that computes single-source shortest paths in O(m log^(2/3) n) time, breaking the classic O(m + n log n) bound of Dijkstra's algorithm for sparse graphs.

## Features

- **Fast SSSP Algorithm**: The O(m log^(2/3) n) implementation as described in the paper
- **Classic Dijkstra**: Also includes classic Dijkstra's algorithm for comparison
- **Generic Implementation**: Works with any floating-point type for edge weights
- **SOLID Design**: Clean interfaces and separation of concerns

## Usage

Add the library to your `Cargo.toml`:

```toml
[dependencies]
fast_sssp = "0.1.0"
```

Example usage:

```rust
use fast_sssp::{DirectedGraph, FastSSSP, ShortestPathAlgorithm};

fn main() {
    // Create a graph
    let mut graph = DirectedGraph::new();
    
    // Add vertices (0, 1, 2, 3, 4)
    for _ in 0..5 {
        graph.add_vertex();
    }
    
    // Add edges
    graph.add_edge(0, 1, 1.0);
    graph.add_edge(0, 2, 4.0);
    graph.add_edge(1, 2, 2.0);
    graph.add_edge(1, 3, 7.0);
    graph.add_edge(2, 3, 3.0);
    graph.add_edge(2, 4, 5.0);
    graph.add_edge(3, 4, 2.0);
    
    // Create algorithm instance
    let fast_sssp = FastSSSP::new();
    
    // Compute shortest paths from source vertex 0
    let result = fast_sssp.compute_shortest_paths(&graph, 0).unwrap();
    
    // Print distances
    for (vertex, dist) in result.distances.iter().enumerate() {
        match dist {
            Some(d) => println!("Distance to {}: {}", vertex, d),
            None => println!("Vertex {} is unreachable", vertex),
        }
    }
}
```

## Implementation Notes

The algorithm is based on the bounded multi-source shortest path (BMSSP) procedure described in the paper. It recursively partitions the problem and efficiently finds pivots to reduce the "frontier" size.

Key parameters in the algorithm:
- k = log^(1/3)(n)
- t = log^(2/3)(n)
- level = ceil(ln(n) / t)
## Benchmarking with Road Networks

Use the `road_network_benchmark` example to evaluate the algorithms on a real road network in DIMACS `.gr` format:

```bash
cargo run --release --example road_network_benchmark path/to/USA-road-d.CAL.gr
```

Add a source vertex ID as a second argument if needed. The example loads the graph, runs FastSSSP and Dijkstra, and prints their run times.


## License

MIT

## References

- Duan, R., Mao, J., Mao, X., Shu, X., & Yin, L. (2025). Breaking the Sorting Barrier for Directed Single-Source Shortest Paths.
