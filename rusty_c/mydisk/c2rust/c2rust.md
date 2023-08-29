# run c2rust
Assuming we don't use bindgen and place all the headers into one "super" file,
```
intercept-build sh -c "cc file/super_file.c" &&
c2rust transpile --emit-build-files --reorganize-definitions file/compile_commands.json
```