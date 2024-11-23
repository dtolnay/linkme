# WASM Support

To support WASM, two requirements are needed: `std`, and a custom linker. 
The custom linker must do the following:
For each `import` module starting with `@@linkme`, rewrite its functions as follows:
Find all `export`s whose name starts with the `import`'s module. The number of them
should be the result of the `_len` function.
The `_init` function should call all of the exports in any order by calling each
with the output of the previous (if the first, the argument), then returning the output of the last.