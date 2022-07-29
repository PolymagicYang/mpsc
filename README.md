# Key-based mpsc.

Key-based mpsc provides a mpsc unbouned channel, with keys collision detection.

**NOTE: key-based mpsc is on the early stage, security issue has a lot and the performence is bad, don't use it in production environment.**

It uses a simple Linked List struture to implement a lock-free multiple producer, single consumer channel. 

### How this channel works
Initially, we have a head which points to a Linked List sentinel node, 



add examples.
more: GC, free on read, timer clean up, loom tests.
Now: free after dropping.