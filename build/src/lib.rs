#![doc = include_str!("../README.md")]

use prost_types::FileDescriptorProto;
use std::io::{BufWriter, Error, ErrorKind, Result, Write};
use std::path::PathBuf;

use crate::descriptor::{Descriptor, Package};
use crate::generator::generate_service;
use crate::message::resolve_service;
use crate::resolver::Resolver;

mod descriptor;
mod escape;
mod generator;
mod message;
mod resolver;

#[derive(Debug, Default)]
pub struct Builder {
    descriptors: descriptor::DescriptorSet,
    exclude: Vec<String>,
    out_dir: Option<PathBuf>,
    extern_paths: Vec<(String, String)>,
    retain_enum_prefix: bool,
    unfold_args: bool,
}

impl Builder {
    /// Create a new `Builder`
    pub fn new() -> Self {
        Self::default()
    }

    /// Configures the output directory where generated Rust files will be written.
    ///
    /// If unset, defaults to the `OUT_DIR` environment variable. `OUT_DIR` is set by Cargo when
    /// executing build scripts, so `out_dir` typically does not need to be configured.
    pub fn out_dir<P>(&mut self, path: P) -> &mut Self
    where
        P: Into<PathBuf>,
    {
        self.out_dir = Some(path.into());
        self
    }

    /// Register an encoded `FileDescriptorSet` with this `Builder`
    pub fn register_descriptors(&mut self, descriptors: &[u8]) -> Result<&mut Self> {
        self.descriptors.register_encoded(descriptors)?;
        Ok(self)
    }

    /// Register a decoded `FileDescriptor` with this `Builder`
    pub fn register_file_descriptor(&mut self, file: FileDescriptorProto) -> &mut Self {
        self.descriptors.register_file_descriptor(file);
        self
    }

    /// Don't generate code for the following type prefixes
    pub fn exclude<S: Into<String>, I: IntoIterator<Item = S>>(
        &mut self,
        prefixes: I,
    ) -> &mut Self {
        self.exclude.extend(prefixes.into_iter().map(Into::into));
        self
    }

    /// Configures the code generator to not strip the enum name from variant names.
    pub fn retain_enum_prefix(&mut self) -> &mut Self {
        self.retain_enum_prefix = true;
        self
    }

    /// Declare an externally provided Protobuf package or type
    pub fn extern_path(
        &mut self,
        proto_path: impl Into<String>,
        rust_path: impl Into<String>,
    ) -> &mut Self {
        self.extern_paths
            .push((proto_path.into(), rust_path.into()));
        self
    }

    /// unfold all args in method
    pub fn unfold_args(&mut self) -> &mut Self {
        self.unfold_args = true;
        self
    }

    /// Generates code for all registered types where `prefixes` contains a prefix of
    /// the fully-qualified path of the type
    pub fn build<S: AsRef<str>>(&mut self, prefixes: &[S]) -> Result<()> {
        let mut output: PathBuf = self.out_dir.clone().map(Ok).unwrap_or_else(|| {
            std::env::var_os("OUT_DIR")
                .ok_or_else(|| {
                    Error::new(ErrorKind::Other, "OUT_DIR environment variable is not set")
                })
                .map(Into::into)
        })?;
        output.push("FILENAME");

        let write_factory = move |package: &Package| {
            output.set_file_name(format!("{}.jsonrpc.rs", package));

            let file = std::fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(&output)?;

            Ok(BufWriter::new(file))
        };

        let writers = self.generate(prefixes, write_factory)?;
        for (_, mut writer) in writers {
            writer.flush()?;
        }

        Ok(())
    }

    /// Generates code into instances of write as provided by the `write_factory`
    ///
    /// This function is intended for use when writing output of code generation
    /// directly to output files is not desired. For most use cases inside a
    /// `build.rs` file, the [`build()`][Self::build] method should be preferred.
    pub fn generate<S: AsRef<str>, W: Write, F: FnMut(&Package) -> Result<W>>(
        &self,
        prefixes: &[S],
        mut write_factory: F,
    ) -> Result<Vec<(Package, W)>> {
        let iter = self.descriptors.iter().filter(move |(t, _)| {
            let exclude = self
                .exclude
                .iter()
                .any(|prefix| t.prefix_match(prefix.as_ref()).is_some());
            let include = prefixes
                .iter()
                .any(|prefix| t.prefix_match(prefix.as_ref()).is_some());
            include && !exclude
        });

        // Exploit the fact descriptors is ordered to group together types from the same package
        let mut ret: Vec<(Package, W)> = Vec::new();
        for (type_path, descriptor) in iter {
            let writer = match ret.last_mut() {
                Some((package, writer)) if package == type_path.package() => writer,
                _ => {
                    let package = type_path.package();
                    ret.push((package.clone(), write_factory(package)?));
                    &mut ret.last_mut().unwrap().1
                }
            };

            let resolver = Resolver::new(
                &self.extern_paths,
                type_path.package(),
                self.retain_enum_prefix,
            );

            match descriptor {
                Descriptor::Service(descriptor) => {
                    if let Some(service) = resolve_service(&self.descriptors, descriptor) {
                        generate_service(&resolver, &service, writer, self.unfold_args)?
                    }
                }
                _ => {}
            }
        }

        Ok(ret)
    }
}
