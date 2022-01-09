# 01.22.2021

- problem: need to pass in half the heap size into the test functions for `stop-and-copy` functions vs `mark-compact` functions
- problem: passing in constant values for heap access _does not always work_ for `stop-and-copy` due to the `to_space` and `from_space` flipping.
  - way 1: provide another required public api for `stop-and-copy` that adds `to-space` to a function
    - this however, gives performance benefits for the mark-compact garbage collector for no reason
  - way 2: pass in a boolean flag into the testing functions, this way both `stop-and-copy` functions and `to-space` suffer the same performance deficit
  - way 3: or we could have the api specified in way 1 contained in test modules
