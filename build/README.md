# Pbjsonrpc-build

Automatically generate [jsonrpsee](https://lib.rs/crates/jsonrpsee) Trait for auto-generated prost types.

## Usage

```toml
[dependencies]
pbjson = "0.3"
pbjson-types = "0.3"
prost = "0.10"
prost-types = "0.10"

[build-dependencies]
pbjson-build = "0.3"
pbjsonrpc-build = "0"
prost-build = "0.10"
```

Next create a `build.rs` containing the following

```rust,ignore
let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("protos");
let proto_files = vec![root.join("myproto.proto")];

// Tell cargo to recompile if any of these proto files are changed
for proto_file in &proto_files {
    println!("cargo:rerun-if-changed={}", proto_file.display());
}

let descriptor_path = PathBuf::from(env::var("OUT_DIR").unwrap())
    .join("proto_descriptor.bin");

prost_build::Config::new()
    // Save descriptors to file
    .file_descriptor_set_path(&descriptor_path)
    // Override prost-types with pbjson-types
    .compile_well_known_types()
    .extern_path(".google.protobuf", "::pbjson_types")
    // Generate prost structs
    .compile_protos(&proto_files, &[root])?;

let descriptor_set = std::fs::read(descriptor_path)?;
pbjson_build::Builder::new()
    .register_descriptors(&descriptor_set)?
    .build(&[".mypackage"])?;
pbjsonrpc_build::Builder::new()
    .register_descriptors(&descriptor_set)?
    .build(&[".mypackage"])?;
```

Finally within `lib.rs`

```rust,ignore
/// Generated by [`prost-build`]
include!(concat!(env!("OUT_DIR"), "/mypackage.rs"));
/// Generated by [`pbjson-build`]
include!(concat!(env!("OUT_DIR"), "/mypackage.serde.rs"));
/// Generated by [`pbjsonrpc-build`]
include!(concat!(env!("OUT_DIR"), "/mypackage.jsonrpc.rs"));
```