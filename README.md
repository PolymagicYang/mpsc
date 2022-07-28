## just a initial test.

channel holds a list.

1. async:   
There is a global index to reprents the current index of the list, the sender just insert
the message to the and then fire and forget.
[] -> [] -> [] -> []

2. sync  
put a additional wake into the node, after the consumer consumes the msg, then wake the sender up.
