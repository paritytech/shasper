# WebAssembly-compiled Beacon Chain State Transition Function

This library compiles the `beacon` crate into WebAssembly, exposes it with a
Javascript interface, and thus allow it to run in a browser.

## Quickstart

Add `eth2` to your dependency. After that, run:

```
const beacon = import('eth2');
beacon.then(m => m.execute_minimal(block, state));
```
