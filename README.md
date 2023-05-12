# Summary
This repository is where I implement my own version of the Kademlia protocol.  It turns out DHTs are really important for Data Availability Sampling.

*Explain how Kademlia ties into Ethereum and DAS*

&nbsp;
#### To Do:
- Implement all functionality for Kbucket management
- Create tests these functions

&nbsp;
&nbsp;
----
This repo is based off the [original Kademlia paper](https://pdos.csail.mit.edu/~petar/papers/maymounkov-kademlia-lncs.pdf) and [Libp2p's Rust implementation](https://github.com/libp2p/rust-libp2p/tree/6985d7246220388738ce7bec644fef170db0c52a/protocols/kad) of the protocol.

#### Modifications Include:
- 256 bit instead of 160 bit keys
- Fixed, 256 sized buckets array (same as libp2p and discv5)
