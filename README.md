# rex

* [Extism](https://extism.org/docs/overview)
* [Extism Rust PDK](https://github.com/extism/rust-pdk)

```sh
ls -la ~/.rex*
target/debug/rex plugin add unpack file:///Users/gurv/Downloads/unpack_extension.wasm --log verbose
target/debug/rex plugin add download https://github.com/moonrepo/plugins/releases/download/download_extension-v1.0.2/download_extension.wasm --log verbose
target/debug/rex run download --log verbose
ls -laR /Users/gurv/.rex
rm -rf /Users/gurv/.rex
```

* run
* generate
  * --template
* mcp
* plugin
  * list
* template
  * add
  * list
    * --filter
* upgrade
