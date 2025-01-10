# Streamed Cache

You are given an API which returns the current temperature that is measured in different cities.
The API exposes two ways to query it:

1. A slow and expensive `fetch` call, which returns the current temperatures for all cities that are being tracked.
2. A faster and incremental `subscribe` method, which only returns updates when the temperature in a city changes.

Your task is to build a performant and robust cache on top of this API, which allows consumers to `get` the current temperature for any tracked city without an asynchronous network call, by keeping the a map of cities and their current temperature up to date in the background.

The goal of the cache is to always return the most recent value the API has delivered.

To check your solution you can run `cargo test`, which runs a simple (non exhaustive) test on your implementation. Feel free to add more tests if you think important cases are not covered.
Please try to not use any additional crates as dependencies.

To submit a solution open a PR like you would usually do it.
