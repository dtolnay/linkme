// In the build script, we try building this file using a rustc invocation with
// `-Clink-arg=-Wl,-z,nostart-stop-gc`. If it succeeds, that means the current
// target's linker supports `-z nostart-stop-gc` so we instruct Cargo to pass
// that flag to the subsequent link.
//
// https://lld.llvm.org/ELF/start-stop-gc
//
// This flag may no longer be needed when #[used(linker)] is stable.

fn main() {}
