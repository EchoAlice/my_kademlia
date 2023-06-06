use crate::helper::{Identifier, U256};
use crate::node::{Search, TableRecord};

const BUCKET_SIZE: usize = 20;
const MAX_BUCKETS: usize = 256;

type Bucket = [Option<TableRecord>; BUCKET_SIZE];

// Bucket 0: Closest peers from node in network.
// Bucket 255: Farthest peers from node in network
#[derive(Debug, PartialEq)]
pub struct KbucketTable {
    pub local_node_id: Identifier,
    pub buckets: [Bucket; MAX_BUCKETS],
}

impl KbucketTable {
    pub fn new(local_node_id: Identifier) -> Self {
        Self {
            local_node_id,
            buckets: [Default::default(); MAX_BUCKETS],
        }
    }

    pub fn add_node(&mut self, record: &TableRecord) -> bool {
        match self.search_table(record.node_id) {
            Search::Success(bucket_index, column_index) => false,
            Search::Failure(bucket_index, column_index) => {
                self.buckets[bucket_index][column_index] = Some(*record);
                true
            }
        }
    }

    pub fn search_table(&self, id: Identifier) -> Search {
        let mut last_empty_index = 0;
        let bucket_index = self.xor_bucket_index(id);
        let mut bucket = self.buckets[bucket_index];

        for (i, node) in bucket.iter().enumerate() {
            match node {
                Some(bucket_node) => {
                    if bucket_node.node_id == id {
                        return Search::Success(bucket_index, i);
                    } else {
                        continue;
                    };
                }
                _ => {
                    last_empty_index = i;
                }
            }
        }
        Search::Failure(bucket_index, last_empty_index)
    }

    pub fn xor_bucket_index(&self, identifier: Identifier) -> usize {
        let x = U256::from(self.local_node_id);
        let y = U256::from(identifier);
        let xor_distance = x ^ y;

        MAX_BUCKETS - (xor_distance.leading_zeros() as usize)
    }
}
