@echo off
rem `cargo bench -p ray-tracer` pinned to one core at high priority (much less noise on Windows).
rem Usage: scripts\bench.cmd [-- criterion args], e.g. -- --save-baseline before / -- --baseline before
start "" /B /WAIT /HIGH /AFFINITY 0x10 cargo bench -p ray-tracer %*
