# Custom TaskSys

This example shows how to create a (simple) custom task system and use it instead of
ispc-rs's default task system. The example shown just creates a serial execution task
system and calls ISPC code which launches a tree of tasks and does two alloc's in
a single context.

