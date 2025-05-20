# SandClock

## Purpose
SandClock is a Rust module that can be use to track if an entity is still active.

## How it works
The tracked entity signals (updates) the  ```SandClock``` with its given key.
If signaling stops, a timeout triggers a callback that can inform the caller that the entity is no longer showing activity.
The timout removes the enty from ```SandClock```.
