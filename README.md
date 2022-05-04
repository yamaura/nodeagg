# nodeagg

Expand nodeset expression

## State of this crate

Currently, only node-set expression expansion is supported.
And also operation expression has not been supported yet.

Iterator structure and function will be changed future.


## How to use this crate

```
use nodeagg::Nodeagg;

let hostnames : Vec<String> =
Nodeagg::try_from("node[01-02],node03").unwrap().iter().collect();
assert_eq!(hostnames, vec!["node01", "node02", "node03"]);
```
