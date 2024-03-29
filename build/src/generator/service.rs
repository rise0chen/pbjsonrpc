use super::{write_jsonrpsee_end, write_jsonrpsee_start, Indent};
use crate::message::{FieldModifier, FieldType, Method, Service};
use crate::resolver::Resolver;
use std::io::{Result, Write};

pub fn generate_service<W: Write>(
    resolver: &Resolver<'_>,
    service: &Service,
    writer: &mut W,
    unfold_args: bool,
    server: bool,
    client: bool,
) -> Result<()> {
    let rust_type = resolver.rust_type(&service.path);

    // Generate Serialize
    write_jsonrpsee_start(0, &rust_type, writer, server, client)?;
    for method in &service.methods {
        write_method(resolver, 2, &method, writer, unfold_args)?;
    }
    write_jsonrpsee_end(0, writer)?;
    Ok(())
}

fn write_method<W: Write>(
    resolver: &Resolver<'_>,
    indent: usize,
    method: &Method,
    writer: &mut W,
    unfold_args: bool,
) -> Result<()> {
    let name = method.rust_method_name();
    let mut name = name.as_str();
    let need_unfold_args =
        unfold_args && method.input.path.prefix_match(".google.protobuf").is_none();
    let args: Vec<_> = if !need_unfold_args {
        vec![format!(
            "{}: Option<{}>",
            "args",
            resolver.rust_type(&method.input.path)
        )]
    } else {
        method
            .input
            .all_fields()
            .map(|f| {
                if let FieldModifier::Repeated = f.field_modifier {
                    if let FieldType::Map(..) = f.field_type {
                    } else {
                        return format!(
                            "{}: Option<Vec<{}>>",
                            f.rust_field_name(),
                            resolver.field_type(&f.field_type)
                        );
                    }
                }
                format!(
                    "{}: Option<{}>",
                    f.rust_field_name(),
                    resolver.field_type(&f.field_type)
                )
            })
            .collect()
    };
    let output = resolver.rust_type(&method.output.path);
    if method.is_stream {
        if let Some(new) = name.strip_prefix("sub_") {
            name = new
        }
        writeln!(
            writer,
            r#"{indent}#[subscription(name = "sub_{name}", unsubscribe = "unsub_{name}", item = {output})]"#,
            indent = Indent(indent),
            name = name,
            output = output
        )?;
        writeln!(
            writer,
            r#"{indent}async fn sub_{name}(&self, {args})-> jsonrpsee::core::SubscriptionResult;"#,
            indent = Indent(indent),
            name = name,
            args = args.join(", ")
        )?;
    } else {
        writeln!(
            writer,
            r#"{indent}#[method(name = "{name}")]"#,
            indent = Indent(indent),
            name = name
        )?;
        writeln!(
            writer,
            r#"{indent}async fn {name}(&self, {args}) -> jsonrpsee::core::RpcResult<{output}>;"#,
            indent = Indent(indent),
            name = name,
            args = args.join(", "),
            output = output
        )?;
    }
    Ok(())
}
