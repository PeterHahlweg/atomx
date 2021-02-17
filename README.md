Atomx provides some data structures, which are intended to be used in multi-threaded applications. For now, this is experimental and not ready for production code.

Signal types are the base building blocks. They wrap atomic types to be practical and hiding some complexity.
The main reason why this crate exists is the state machine type. It provides the ability to create interconnected state machines which can run on different threads.