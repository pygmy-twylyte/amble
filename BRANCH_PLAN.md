# Eliminate the Middleman

In looking at simplifying / refactoring the compiler code, I was hit with a revelation:

**Why am I still loading data from TOML?**

I chose TOML when first starting Amble because I thought content would all be written directly, and TOML was the simplest / most human-friendly syntax to use. However, we've now come to the point where the TOML is never edited by hand, and is only written by the compiler and read by the engine...so it's become a "middleman" without much of a point. The ASTs in the current compiler mirror the engine structs fairly closely, so the conversion to TOML and then back to get it into the engine adds an unnecessary step, two (.amble and TOML data) sources of "truth".

## The Plan

The plan is to create a new intermediary world data crate. The amble_script parser will load all of the entity defs into the main data struct there, which will have a method to create a runtime AmbleWorld object from that.

Using that structure, we should be able to either then just serialize to .ron for a snapshot of the starting world and/or build directly from .amble source data on startup, since that pass typically takes only a fraction of a second.
