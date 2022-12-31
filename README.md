[![Crates.io](https://img.shields.io/crates/v/atomx)](https://crates.io/crates/atomx)
[![docs.rs](https://docs.rs/atomx/badge.svg)](https://docs.rs/atomx/)

# Atomx

__This crate is experimental! Do not use in production.__

## Signal
It provides an inter thread communication channel called Signal.
A `Signal` always has a single `Source` and one or more `Sink`'s (single producer multiple consumer,
SPMC).
The behavior of signal differs from known channels such that there is no fifo like behavior.
Values are not taken out of the channel by a sink. They stay the same regardless how often they
will be read. The only way the Signal data can change is through the Source.
This decouples the frequency domains from the source and sinks.
In case a synchronous behavior of the Signal is required, where all sinks need to acknowledge the changed signal value, there is the signal::sync module. It provides the same interface as the default  implementation and adds synchronization functionality. As it is assumed to be an exceptional requirement, it comes with a little more overhead.

This kind of behavior is probably useful in systems where a subsystem is processing data by sampling it's sources in a given frequency and the source signals may also run in different frequency's. Or if it is known that the source data is up to data at the time of sampling. The synced signal can be used to provide a trigger signal for multiple subsystems.

The underlying mechanism is inspired by page flipping, where one display buffer is displayed while
the other can be modified.
Hazard pointers are used to protect the atomic pointer which references the sink/read buffer.

Kudos to [jonhoo](https://github.com/jonhoo). The Signal module was inspired by his streams and is
powered by his hazard pointer implementation ([jonhoo/haphazard](https://github.com/jonhoo/haphazard)).

## License
Atomx is distributed under the terms of both the MIT license and the Apache License (Version 2.0).
See [LICENSE-APACHE](LICENSE-APACHE-2.0) and [LICENSE-MIT](LICENSE-MIT) for details.

## Contribution
Opening a pull request is assumed to signal agreement with these licensing terms.