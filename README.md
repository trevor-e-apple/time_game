# Profiling
I use the [flamegraph](https://github.com/flamegraph-rs/flamegraph) crate for profiling. On Linux, you usually need to reduce the strictness
of the securit settings with a command like

`echo "-1" | sudo tee -a /proc/sys/kernel/perf_event_paranoid`.

On my system, restarting the machine resets this setting (which is good). So you'll need to do this
once per boot cycle. 

To run flamegraph on your binary, first build the target that you want to profile. For example, if you want to profile a release build,
you can use the following commands.

```
cargo build -r
DATA_DIR=./data SHADER_SOURCE_DIR=./src/graphics flamegraph -- ./target/release/time_game
```
