## Basics
All OK.

## Completeness
All cases covered.

## Correctness
Simple test csv's included in the repo under `csvs/`.

Unit tests can be ran with `cargo --test` over them.

> Or are you using the type system to ensure correctness?

It's rust 👍️

## Safety and Robustness
> Are you doing something dangerous?

Not really.
All unwraps are reasonable for the purpouses of an exercise (i.e. the static hashmap should always exist.).

> How are you handling errors?

Very generously, everything bails, but if any record has an error, we continue to the next one.

## Efficiency

> Can you stream values through memory as opposed to loading the entire data set upfront?

I'm using a `BufReader` and checking on the csv library, to the best of my understanding it's using an interator that reads record-by-record, so should be ok.


## Maintainability

I tried to keep it simple as usual 👍️
