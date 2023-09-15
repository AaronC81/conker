# Conker

Conker is a **very experimental, natively-concurrent programming language**, vaguely inspired by the
computer architecture featured in
[Zachtronics' puzzle game TIS-100](https://store.steampowered.com/app/370360/TIS100/).

## Semantics

A Conker program is comprised of _tasks_, all of which run concurrently as separate threads. Tasks
can communicate with each other by sending values over _channels_.

Channels have no buffer - sends and receives block until the other side is satisfied.

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

## Advanced Example - Counter

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
