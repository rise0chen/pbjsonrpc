pub mod test {
    pub mod common {
        include!(concat!(env!("OUT_DIR"), "/test.common.rs"));
        include!(concat!(env!("OUT_DIR"), "/test.common.serde.rs"));
        include!(concat!(env!("OUT_DIR"), "/test.common.jsonrpc.rs"));
    }
}
pub mod google {
    pub mod protobuf {
        pub use pbjson_types::*;
    }
}
