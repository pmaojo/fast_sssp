use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::fmt::Debug;
// No longer need these imports

/// A wrapper around BinaryHeap for priority queue operations in shortest path algorithms
#[derive(Debug)]
pub struct BinaryHeapWrapper<V, P>
where
    V: Copy + Eq + Debug + Ord,
    P: PartialOrd + Copy + Debug + Ord,
{
    /// The underlying binary heap
    heap: BinaryHeap<Reverse<(P, V)>>,
}

impl<V, P> BinaryHeapWrapper<V, P>
where
    V: Copy + Eq + Debug + Ord,
    P: PartialOrd + Copy + Debug + Ord,
{
    /// Creates a new empty priority queue
    pub fn new() -> Self {
        BinaryHeapWrapper {
            heap: BinaryHeap::new(),
        }
    }
    
    /// Returns true if the priority queue is empty
    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }
    
    /// Returns the number of elements in the priority queue
    pub fn len(&self) -> usize {
        self.heap.len()
    }
    
    /// Pushes an element with the given priority into the priority queue
    pub fn push(&mut self, vertex: V, priority: P) {
        self.heap.push(Reverse((priority, vertex)));
    }
    
    /// Removes the element with the highest priority
    pub fn pop(&mut self) -> Option<(V, P)> {
        self.heap.pop().map(|Reverse((priority, vertex))| (vertex, priority))
    }
    
    /// Returns the element with the highest priority without removing it
    pub fn peek(&self) -> Option<(V, P)> {
        self.heap.peek().map(|Reverse((priority, vertex))| (*vertex, *priority))
    }
    
    /// Clears the priority queue
    pub fn clear(&mut self) {
        self.heap.clear();
    }
}
