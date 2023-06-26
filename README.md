# Summary
Ethereum plans to implement Danksharding to scale.  Danksharding leverages a technique called [Data Availability Sampling](https://hackmd.io/@EchoAlice/HJc9CwU-2), requiring Ethereum's *Consensus Layer P2P network* to be redesigned.

Design direction seems to be leaning towards using a [distributed hash table](https://en.wikipedia.org/wiki/Distributed_hash_table) to support storage and retrieval of *samples* of blob data (part of Danksharding).

Within this repository, I'm building out an MVP Kademlia client to gain a deeper understanding of DHTs.