# Summary
This repository is where I implement my own version of the Kademlia protocol, creating a Kademlia client that can interact with others to form a peer-to-peer distributed hash table.

### What is a DHT?  (WIP)
Distributed Hash Tables are similar to hash tables in that a node can **store** and **retrieve** information given aribtrary keys and values, but instead of data being written to and read from a single computer, responsibility for storage and retrieval is *distributed* amongst a *network* of computers.  Through this network, a more [robust (in some sense), decentralized and scalable](https://en.wikipedia.org/wiki/Distributed_hash_table) data structure emerges.


*Talk about major components*
- Key space
- Indexing responsibility with hashes
- 

### What does a DHT network look like? (WIP)
A distributed hash table is a bunch of nodes communicating (requesting and responding) to one another about peers, keys (hashed data) and values (data the network stores).

To facilitate communication, *each node* runs some set of rules (a protocol) to provides a set of functionalities required for this ever-evolving data structure to exist.

**General functionalities exposed by a node:**
- WIP
- 


### What does the Kademlia protocol look like? (WIP)
*Explain Kademlia (certain type of rules).  Go into more depth*
The Kademlia protocol is a *style* of DHT that provides simple and efficient routing of nodes.





### How does the Kademlia protocol relate to Data Availability Sampling?  (WIP)
*Briefly mention DAS*
*Our samples need to be stored amongst the Ethereum Network*
*Mention fragility of naive Kademlia*


&nbsp;
&nbsp;
# Repository Roadmap
- Implement basic versions of all RPCs (+ some tests) from the Kademlia protocol.
**Our 4 RPCs:**
    - store()&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;[ ]
        - add_node()&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;[X]
        - add_store()&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;[ ]
    - find_node() &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;[X]
        - search_table() &nbsp;&nbsp;&nbsp;&nbsp; [X]
    - find_value()&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;[ ]
    - ping()&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;[ ]   <--- Could turn into a rabbit hole if I'm not careful!
- Create minimum networking functionality to ping two nodes.

#### Maybe later...
- Create minimum networking functionality to connect to a Kademlia p2p network.
- Encorperate shared parallel lookups
- Implement updatable routing table logic (ie. removal of bad nodes)

&nbsp;
&nbsp;
----
This repo is based off the [original Kademlia paper](https://pdos.csail.mit.edu/~petar/papers/maymounkov-kademlia-lncs.pdf) and [Libp2p's Rust implementation](https://github.com/libp2p/rust-libp2p/tree/6985d7246220388738ce7bec644fef170db0c52a/protocols/kad) of the protocol with some modifications.
Copyright 2018 Parity Technologies (UK) Ltd.

#### Modifications Include:
- 256 bit instead of 160 bit keys
- Fixed, 256 sized buckets array
