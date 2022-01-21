- why in the world is traversal for m&c sometimes faster and sometimes slower than s&c?
  - s&c traverses less and swaps more (_always_ swaps everything!)
  - m&c traverses more and swaps less

- so when we don't have that much garbage in one iteration
  - the list of marked nodes is a lot
  - we traverse over marked nodes 3 times
  - therefore, mark compact takes much longer to compact
  - while stop and copy only traverses over the marked nodes once
  - assume 85% objects are live
  - 3(.85n) vs (.85n) + copy(.85n)
- when we have a lot of garbage
  - 20% objects are live
  - 3(.25n) vs (.25n) + copy(.25n)

- [ ] add metrics: create a "metrics" branch, where we record the number of nodes moved.
