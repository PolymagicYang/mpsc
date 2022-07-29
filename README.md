# Key-based mpsc.

channel holds a list.

1. async:   
use tail to maintain the insertion status, the sender just insert
the message to the and then fire and forget.
[] -> [] -> [] -> []

2. sync  
put a additional wake into the node, after the consumer consumes the msg, then wake the sender up.

3. msg drop:
before inserting the message into the linked-list, install a collision-filter to filter the msg (filter fn/trait is defined by the user), after the msg is dropped, notify the filter to change the filter.

4. Order: Filter > FIFO