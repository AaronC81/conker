# Conker

Conker is a **very experimental, natively-concurrent programming language**, vaguely inspired by the
computer architecture featured in
[Zachtronics' puzzle game TIS-100](https://store.steampowered.com/app/370360/TIS100/).

## Semantics

A Conker program is comprised of _tasks_, all of which run concurrently as separate threads. All
tasks begin at the start of the program, and the program continues running until all tasks have
finished, or any task runs an `exit` statement.

Tasks can communicate with each other by sending values over _channels_. Channels have no buffer -
sends and receives block until the other side is satisfied.

The closest to "Hello, world" we can get in a language without strings:

```
task Main
    123 -> $out
```

The `->` operator sends a message to a channel. `$out` is a "magic" channel, which receives
instantly and prints the received value to standard output.

For some inter-task communication, a contrived example:

```
task Adder
    a <- ?c
    b <- c
    a + b -> c

task Main
    5 -> Adder
    4 -> Adder
    result <- Adder
    result -> $out
```

The `<-` operator receives a message. Using `?` on the channel side is a _binding receive_ - this
receives a message on _any_ channel, and stores a reference to that channel using the given name.
That reference can then be used to send or receive further messages on the same channel. In the
definition of the `Adder` task, this means that `b` will definitely be received through the same
channel as `a`.

### Example - Counter

```
# When receiving any message, responds with a number, 
# then increments the number for next time
task Counter
    x = 0
    loop
        _ <- ?c
        x = (x + 1)
        x -> c

# Counts forever
task Main
    loop
        null -> Counter
        x <- Counter
        x -> $out
```

## Multi-Tasks

Sometimes, you may want to parallelise an operation by running multiple instances of the same task.
Conker enables this with _multi-tasks_, which can be defined using `[n]` at the end of a task
definition.

When defining a multi-task, that task's name refers to an _array_ of tasks, rather than directly to
a single task. Within a multi-task, the index of the instance which is running can be accessed with 
`$index`.

The following program prints `0`, `1`, `2`, `3`, `4` in a non-deterministic order:

```
task Printer[5]
    $index -> $out
```

To ensure they were printed in order, another task could mediate the values by receiving from
specific tasks within the multi-task:

```
task ConstantSource[5]
    $index -> Main

task Main
    i = 0
    while i < 5
        x <- ConstantSource[i]
        x -> $out
        i = i + 1
```
