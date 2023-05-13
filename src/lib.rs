#![allow(unused)]

// Step 1: Create kbucket management system
// Step 2: Implement the rest

/*
    Expose these functionalities for our end user:

    ping()
    store()
    find_node()
    find_value()
*/

pub mod kbucket;
pub mod helper;

const BUCKET_SIZE: usize = 20;
const MAX_BUCKETS: usize = 256;