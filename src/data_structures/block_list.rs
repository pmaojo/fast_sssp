use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;
use num_traits::{Float, Zero};

/// Implementation of the block-based linked list data structure from Lemma 3.3 in the paper
/// 
/// This data structure efficiently supports:
/// - Insert: Insert a key/value pair in amortized O(max{1, log(N/M)}) time
/// - BatchPrepend: Insert L key/value pairs with values smaller than any in the structure in amortized O(L·max{1, log(L/M)}) time
/// - Pull: Return a subset of keys with smallest values and an upper bound in amortized O(|S'|) time
#[derive(Debug)]
pub struct BlockList<K, V>
where 
    K: Eq + std::hash::Hash + Copy + Debug,
    V: Float + Zero + Debug + Copy + PartialOrd
{
    /// Maximum block size parameter
    block_size: usize,
    
    /// Upper bound value
    upper_bound: V,
    
    /// Key-value mapping
    key_values: HashMap<K, V>,
    
    /// D0 block sequence (for batch prepended elements)
    d0_blocks: Vec<Block<K, V>>,
    
    /// D1 block sequence (for individually inserted elements)
    d1_blocks: Vec<Block<K, V>>,
    
    /// Binary search tree mapping upper bounds to block indices for D1
    upper_bounds: BTreeMap<V, usize>,
}

/// A block in the block list
#[derive(Debug, Clone)]
struct Block<K, V>
where
    K: Eq + std::hash::Hash + Copy + Debug,
    V: Float + Zero + Debug + Copy + PartialOrd
{
    /// Pairs stored in this block
    pairs: Vec<(K, V)>,
    
    /// Upper bound of values in this block
    upper_bound: V,
}

impl<K, V> BlockList<K, V>
where
    K: Eq + std::hash::Hash + Copy + Debug,
    V: Float + Zero + Debug + Copy + PartialOrd + Ord
{
    /// Creates a new BlockList with the specified block size and upper bound
    pub fn new(block_size: usize, upper_bound: V) -> Self {
        BlockList {
            block_size,
            upper_bound,
            key_values: HashMap::new(),
            d0_blocks: Vec::new(),
            d1_blocks: vec![Block { pairs: Vec::new(), upper_bound }],
            upper_bounds: BTreeMap::new(),
        }
    }
    
    /// Checks if the data structure is empty
    pub fn is_empty(&self) -> bool {
        self.key_values.is_empty()
    }
    
    /// Returns the number of keys in the data structure
    pub fn len(&self) -> usize {
        self.key_values.len()
    }
    
    /// Returns the value for a key if it exists
    pub fn get(&self, key: &K) -> Option<V> {
        self.key_values.get(key).copied()
    }
    
    /// Inserts a key-value pair into the data structure
    /// If the key already exists, updates its value if the new value is smaller
    pub fn insert(&mut self, key: K, value: V) {
        // Check if key already exists and only update if new value is smaller
        if let Some(old_value) = self.key_values.get(&key) {
            if value >= *old_value {
                return;
            }
            // Remove old value from blocks
            self.remove_key_from_blocks(&key);
        }
        
        // Update key-value mapping
        self.key_values.insert(key, value);
        
        // Insert into D1 blocks
        let block_idx = self.find_block_for_value(value);
        self.d1_blocks[block_idx].pairs.push((key, value));
        
        // If block exceeds size limit, split it
        if self.d1_blocks[block_idx].pairs.len() > self.block_size {
            self.split_block(block_idx);
        }
    }
    
    /// Batch prepends key-value pairs with values smaller than any in the data structure
    pub fn batch_prepend(&mut self, pairs: Vec<(K, V)>) {
        if pairs.is_empty() {
            return;
        }
        
        // Filter pairs to keep only the smallest value for each key
        let mut unique_pairs: HashMap<K, V> = HashMap::new();
        for (key, value) in pairs {
            // Only keep if it's the smallest value seen for this key
            if let Some(existing_value) = unique_pairs.get(&key) {
                if value < *existing_value {
                    unique_pairs.insert(key, value);
                }
            } else {
                unique_pairs.insert(key, value);
            }
        }
        
        // Update key-value mapping and remove keys from existing blocks if needed
        for (&key, &value) in unique_pairs.iter() {
            if let Some(old_value) = self.key_values.get(&key) {
                if value >= *old_value {
                    continue;
                }
                // Remove old value from blocks
                self.remove_key_from_blocks(&key);
            }
            self.key_values.insert(key, value);
        }
        
        let pairs_vec: Vec<(K, V)> = unique_pairs.into_iter().collect();
        
        // Create blocks for the batch
        if pairs_vec.len() <= self.block_size {
            // Single block case
            if !pairs_vec.is_empty() {
                let max_value = pairs_vec.iter().map(|(_, v)| *v).fold(V::zero(), |a, b| if a > b { a } else { b });
                self.d0_blocks.push(Block {
                    pairs: pairs_vec,
                    upper_bound: max_value,
                });
            }
        } else {
            // Multiple blocks case - divide using median finding
            self.create_multiple_blocks(pairs_vec);
        }
    }
    
    /// Pulls a subset of keys with the smallest values and returns an upper bound
    pub fn pull(&mut self, max_count: usize) -> (Vec<K>, V) {
        let mut result = Vec::new();
        let mut next_bound = self.upper_bound;
        
        // Helper function to collect elements from blocks
        let collect_from_blocks = |blocks: &mut Vec<Block<K, V>>, count: usize| -> Vec<(K, V)> {
            let mut collected = Vec::new();
            let mut remaining = count;
            
            while !blocks.is_empty() && remaining > 0 {
                let block = &blocks[0];
                if block.pairs.len() <= remaining {
                    // Take the whole block
                    collected.extend(block.pairs.iter().cloned());
                    remaining -= block.pairs.len();
                    blocks.remove(0);
                } else {
                    // Take only what we need
                    collected.extend(block.pairs.iter().take(remaining).cloned());
                    
                    // Keep the rest in the block
                    let mut new_block = Block {
                        pairs: block.pairs.iter().skip(remaining).cloned().collect(),
                        upper_bound: block.upper_bound,
                    };
                    std::mem::swap(&mut blocks[0], &mut new_block);
                    break;
                }
            }
            
            collected
        };
        
        // Collect elements from D0 first
        let mut d0_elements = collect_from_blocks(&mut self.d0_blocks, max_count);
        
        // Then from D1 if needed
        let remaining = max_count - d0_elements.len();
        let mut d1_elements = if remaining > 0 {
            collect_from_blocks(&mut self.d1_blocks, remaining)
        } else {
            Vec::new()
        };
        
        // Combine and sort by value
        let mut all_elements = Vec::with_capacity(d0_elements.len() + d1_elements.len());
        all_elements.append(&mut d0_elements);
        all_elements.append(&mut d1_elements);
        
        // Sort by value to ensure we get the smallest ones
        all_elements.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        
        // Take up to max_count elements
        let elements_to_take = all_elements.len().min(max_count);
        let taken_elements = all_elements.iter().take(elements_to_take).cloned().collect::<Vec<_>>();
        
        // Find the next bound
        if taken_elements.len() < all_elements.len() {
            // Next bound is the value of the first element we didn't take
            next_bound = all_elements[taken_elements.len()].1;
        } else if !self.d0_blocks.is_empty() {
            // Next bound is the upper bound of the first remaining D0 block
            next_bound = self.d0_blocks[0].upper_bound;
        } else if !self.d1_blocks.is_empty() {
            // Next bound is the upper bound of the first remaining D1 block
            next_bound = self.d1_blocks[0].upper_bound;
        }
        
        // Remove taken elements from key_values map and extract keys for result
        for (key, _) in &taken_elements {
            self.key_values.remove(key);
            result.push(*key);
        }
        
        // Rebuild upper_bounds for D1
        self.rebuild_upper_bounds();
        
        (result, next_bound)
    }
    
    /// Finds the appropriate block index in D1 for a value
    fn find_block_for_value(&self, value: V) -> usize {
        // Using binary search tree (upper_bounds) to find the right block
        match self.upper_bounds.range(value..).next() {
            Some((_, &idx)) => idx,
            None => self.d1_blocks.len() - 1, // Default to last block
        }
    }
    
    /// Splits a block in D1 that has exceeded the size limit
    fn split_block(&mut self, block_idx: usize) {
        // Find median element
        let block = &mut self.d1_blocks[block_idx];
        block.pairs.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        
        let median_idx = block.pairs.len() / 2;
        let median_value = block.pairs[median_idx].1;
        
        // Create new block with elements greater than or equal to median
        let new_block = Block {
            pairs: block.pairs.drain(median_idx..).collect(),
            upper_bound: block.upper_bound,
        };
        
        // Update upper bound of original block
        block.upper_bound = median_value;
        
        // Insert new block after the original
        self.d1_blocks.insert(block_idx + 1, new_block);
        
        // Rebuild upper bounds mapping
        self.rebuild_upper_bounds();
    }
    
    /// Creates multiple blocks for batch prepend with efficient median finding
    fn create_multiple_blocks(&mut self, mut pairs: Vec<(K, V)>) {
        // Sort by value
        pairs.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        
        // Create blocks of approximately half the maximum size
        let target_size = self.block_size / 2;
        let mut blocks = Vec::new();
        
        for chunk in pairs.chunks(target_size) {
            if !chunk.is_empty() {
                let max_value = chunk.iter().map(|(_, v)| *v).fold(V::zero(), |a, b| if a > b { a } else { b });
                blocks.push(Block {
                    pairs: chunk.to_vec(),
                    upper_bound: max_value,
                });
            }
        }
        
        // Prepend these blocks to D0
        self.d0_blocks = blocks.into_iter().chain(self.d0_blocks.drain(..)).collect();
    }
    
    /// Removes a key from all blocks (used when updating a key's value)
    fn remove_key_from_blocks(&mut self, key: &K) {
        // Remove from D0 blocks
        for block in &mut self.d0_blocks {
            block.pairs.retain(|(k, _)| k != key);
        }
        
        // Remove from D1 blocks
        for block in &mut self.d1_blocks {
            block.pairs.retain(|(k, _)| k != key);
        }
        
        // Clean up empty blocks (except keep at least one in D1)
        self.d0_blocks.retain(|block| !block.pairs.is_empty());
        
        if self.d1_blocks.len() > 1 {
            self.d1_blocks.retain(|block| !block.pairs.is_empty());
            if self.d1_blocks.is_empty() {
                self.d1_blocks.push(Block {
                    pairs: Vec::new(),
                    upper_bound: self.upper_bound,
                });
            }
        }
        
        // Rebuild upper bounds for D1
        self.rebuild_upper_bounds();
    }
    
    /// Rebuilds the upper bounds mapping for D1 blocks
    fn rebuild_upper_bounds(&mut self) {
        self.upper_bounds.clear();
        for (idx, block) in self.d1_blocks.iter().enumerate() {
            self.upper_bounds.insert(block.upper_bound, idx);
        }
    }
}
