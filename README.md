# Summary
Where I implement my own version of the Kademlia protocol.

*Explain how Kademlia ties into Ethereum and DAS*


Based off the [original Kademlia paper](https://pdos.csail.mit.edu/~petar/papers/maymounkov-kademlia-lncs.pdf).

#### Modifications Include:
- 256 bit instead of 160 bit keys
- Fixed, 256 sized buckets array (bc same as libp2p and discv5)