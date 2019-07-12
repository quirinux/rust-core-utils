# rust-core-util
## gnu-coreutil rewritten in rust

This is a personal project meant to help me learning rust and it has no intention to become a production ready tool.
[uutil/coreutils](https://github.com/uutils/coreutils) offers a much better solution and this project was heavily inspired on it.

### Goals
1. Have fun!
1. learn rust
2. be compatible to gnu-coreutil as much as possible

### No goals
1. be production ready
2. rewrite all gnu-coreutil tools

### Compile & Run
[rake](https://github.com/ruby/rake) is optional and used to facilitate the compile, test and run processes, so:

1. rake build:debug
1. rake build:release
1. rake build:all

Will do the trick, run _rake -T_ to show a complete list os tasks available
A part form that all cargo workflow should work fine.


## Lessons learnt
### 1. echo
- cargo usage
- rust compilation process
- initial rust concepts
- string monipulation
- CLI arguments handling
- Struct
- use of src/bin to create multiple binary targets
- how to not create a main.rs target binary

### 2. cat
- macro usage
- error handling
- Box allocation
- Read from and Write to a Buffer
- Fn multiple returning values
- Impl Fn to Struct
- ready binary data from a file

### 3. head
- split code in modules
- share a module among different binaries
- Sctruct Fn and prperties accessibility policies
- how to pass ownership through
- log strategy
- Improved a lot how to handle error

### 4. tail
- enum
- thread operation and strategy
- crossbeam channel
- implement fmt traits for better display and debug
- increased error handling knowledge
- 
