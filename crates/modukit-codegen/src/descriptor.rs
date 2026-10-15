//! The descriptor.proto subset decoder (hand-written, on top of `wire`).
//!
//! Field numbers below are google/protobuf/descriptor.proto's own - stable
//! since 2008 - cited per match arm so the subset stays auditable. Anything
//! the subset does not model is decode-and-skipped (D12 rule 2): the
//! `FieldCursor` already consumed the bytes; the match arms simply have no
//! entry for them.

use crate::error::CodegenError;
use crate::wire::{FieldCursor, Reader};

// The subset types are produced only by this decoder, so this module is
// their public door; emitters import them from `model` directly.
pub use crate::model::{
    EXT_SERVICE_ID, Enum, EnumValue, Field, FieldType, File, Label, Message, Method,
    RUNTIME_PACKAGE, Service,
};

/// Decode one `FileDescriptorSet` buffer (protoc `--descriptor_set_out`)
/// into its file entries.
pub fn decode_file_descriptor_set(buf: &[u8]) -> Result<Vec<File>, CodegenError> {
    let mut r = Reader::new(buf);
    let mut cur = FieldCursor::new(&mut r);
    let mut files = Vec::new();
    while let Some((num, value)) = cur.next_field()? {
        // FileDescriptorSet.file = 1 (repeated FileDescriptorProto); every
        // other number is consumed and dropped - decode-and-skip (D12 rule 2).
        if num == 1 {
            files.push(decode_file(value.as_len_bytes()?)?);
        }
    }
    Ok(files)
}

fn decode_file(buf: &[u8]) -> Result<File, CodegenError> {
    let mut r = Reader::new(buf);
    let mut cur = FieldCursor::new(&mut r);
    let mut file = File {
        name: String::new(),
        package: String::new(),
        messages: Vec::new(),
        enums: Vec::new(),
        services: Vec::new(),
    };
    while let Some((num, value)) = cur.next_field()? {
        match num {
            // FileDescriptorProto: name = 1, package = 2, message_type = 4,
            // enum_type = 5, service = 6. dependency = 3 / options = 8 /
            // extension = 7 / syntax = 12 carry nothing codegen needs.
            1 => file.name = value.as_len_str()?.to_string(),
            2 => file.package = value.as_len_str()?.to_string(),
            4 => file.messages.push(decode_message(value.as_len_bytes()?)?),
            5 => file.enums.push(decode_enum(value.as_len_bytes()?)?),
            6 => file.services.push(decode_service(value.as_len_bytes()?)?),
            _ => {} // decode-and-skip (D12 rule 2)
        }
    }
    Ok(file)
}

fn decode_message(buf: &[u8]) -> Result<Message, CodegenError> {
    let mut r = Reader::new(buf);
    let mut cur = FieldCursor::new(&mut r);
    let mut msg = Message {
        name: String::new(),
        fields: Vec::new(),
    };
    while let Some((num, value)) = cur.next_field()? {
        match num {
            // DescriptorProto: name = 1, field = 2.
            1 => msg.name = value.as_len_str()?.to_string(),
            2 => msg.fields.push(decode_field(value.as_len_bytes()?)?),
            _ => {}
        }
    }
    // Nested types / message-scoped enums (DescriptorProto.nested_type = 3,
    // enum_type = 4) would vanish from a flat subset decode - reject
    // explicitly instead of emitting half a type.
    if let Some(target) = first_present(buf, &[3, 4])? {
        return Err(CodegenError::wire(format!(
            "message {}: descriptor field {target} (nested type / scoped enum) \
             is outside the v0 subset",
            msg.name
        )));
    }
    Ok(msg)
}

fn decode_field(buf: &[u8]) -> Result<Field, CodegenError> {
    let mut r = Reader::new(buf);
    let mut cur = FieldCursor::new(&mut r);
    let mut f = Field {
        name: String::new(),
        number: 0,
        label: Label::Optional,
        kind: FieldType::String,
        type_name: String::new(),
    };
    let mut seen_number = false;
    let mut seen_label = false;
    while let Some((num, value)) = cur.next_field()? {
        match num {
            // FieldDescriptorProto: name = 1, extendee = 2, number = 3,
            // label = 4, type = 5, type_name = 6, default_value = 7,
            // options = 8, oneof_index = 9, json_name = 10.
            1 => f.name = value.as_len_str()?.to_string(),
            3 => {
                f.number = u32::try_from(value.as_varint()?)
                    .map_err(|_| CodegenError::wire("field number overflows u32"))?;
                seen_number = true;
            }
            4 => {
                f.label = match value.as_varint()? {
                    1 => Label::Optional,
                    // required (label 2): this vocabulary is optional /
                    // repeated only - a required tag is an authoring bug.
                    2 => {
                        return Err(CodegenError::FieldRule(format!(
                            "field {:?}: required is refused (use optional)",
                            f.name
                        )));
                    }
                    3 => Label::Repeated,
                    other => {
                        return Err(CodegenError::wire(format!(
                            "unknown FieldDescriptorProto.Label: {other}"
                        )));
                    }
                };
                seen_label = true;
            }
            5 => f.kind = FieldType::from_type_number(value.as_varint()?)?,
            6 => f.type_name = value.as_len_str()?.to_string(),
            9 => {
                return Err(CodegenError::wire(format!(
                    "field {:?}: oneof is D14/M5 territory, refused at v0",
                    f.name
                )));
            }
            _ => {} // options / json_name / defaults: skip
        }
    }
    // protoc always emits number + label; their absence means a hand-built
    // or corrupt descriptor - fail loudly, never default (D12 rule 1).
    if !seen_number || f.number == 0 {
        return Err(CodegenError::FieldRule(format!(
            "field {:?}: no hand-written number",
            f.name
        )));
    }
    if !seen_label {
        return Err(CodegenError::FieldRule(format!(
            "field {:?}: no label",
            f.name
        )));
    }
    Ok(f)
}

fn decode_enum(buf: &[u8]) -> Result<Enum, CodegenError> {
    let mut r = Reader::new(buf);
    let mut cur = FieldCursor::new(&mut r);
    let mut e = Enum {
        name: String::new(),
        values: Vec::new(),
    };
    while let Some((num, value)) = cur.next_field()? {
        match num {
            // EnumDescriptorProto: name = 1, value = 2, options = 3.
            1 => e.name = value.as_len_str()?.to_string(),
            2 => e.values.push(decode_enum_value(value.as_len_bytes()?)?),
            _ => {}
        }
    }
    e.sentinel()?; // D12 rule 3 enforced at decode time
    Ok(e)
}

fn decode_enum_value(buf: &[u8]) -> Result<EnumValue, CodegenError> {
    let mut r = Reader::new(buf);
    let mut cur = FieldCursor::new(&mut r);
    let mut v = EnumValue {
        name: String::new(),
        number: 0,
    };
    while let Some((num, value)) = cur.next_field()? {
        match num {
            // EnumValueDescriptorProto: name = 1, number = 2, options = 3.
            1 => v.name = value.as_len_str()?.to_string(),
            2 => {
                v.number = i32::try_from(value.as_varint()? as i64)
                    .map_err(|_| CodegenError::wire("enum number overflows i32"))?;
            }
            _ => {}
        }
    }
    Ok(v)
}

fn decode_service(buf: &[u8]) -> Result<Service, CodegenError> {
    let mut r = Reader::new(buf);
    let mut cur = FieldCursor::new(&mut r);
    let mut s = Service {
        name: String::new(),
        methods: Vec::new(),
        service_id: String::new(),
    };
    let mut ext_service_id: Option<String> = None;
    let mut uninterpreted = false;
    while let Some((num, value)) = cur.next_field()? {
        match num {
            // ServiceDescriptorProto: name = 1, method = 2, options = 3.
            1 => s.name = value.as_len_str()?.to_string(),
            2 => s.methods.push(decode_method(value.as_len_bytes()?)?),
            3 => {
                let (sid, has_uninterpreted) = decode_service_options(value.as_len_bytes()?)?;
                ext_service_id = sid;
                uninterpreted = has_uninterpreted;
            }
            _ => {}
        }
    }
    s.service_id = validate_service_id(&s.name, ext_service_id, uninterpreted)?;
    Ok(s)
}

/// The first of `targets` present as a field number in `buf` (else `None`).
/// Used to reject unmodeled shapes instead of half-reading them.
fn first_present(buf: &[u8], targets: &[u32]) -> Result<Option<u32>, CodegenError> {
    let mut r = Reader::new(buf);
    let mut cur = FieldCursor::new(&mut r);
    while let Some((num, _value)) = cur.next_field()? {
        if targets.contains(&num) {
            return Ok(Some(num));
        }
    }
    Ok(None)
}

fn decode_service_options(buf: &[u8]) -> Result<(Option<String>, bool), CodegenError> {
    let mut r = Reader::new(buf);
    let mut cur = FieldCursor::new(&mut r);
    let mut ext: Option<String> = None;
    let mut uninterpreted = false;
    while let Some((num, value)) = cur.next_field()? {
        match num {
            // Custom option: protoc resolves `(modukit.service_id)` against
            // the imported extension and writes the string at tag 50000.
            EXT_SERVICE_ID => ext = Some(value.as_len_str()?.to_string()),
            // google.protobuf.ServiceOptions.uninterpreted_option = 999 -
            // set when an option failed to resolve at compile time.
            999 => uninterpreted = true,
            // ServiceOptions other fields (deprecated = 33, features, ...):
            // irrelevant here, decode-and-skip carries them past.
            _ => {}
        }
    }
    Ok((ext, uninterpreted))
}

fn validate_service_id(
    service: &str,
    ext: Option<String>,
    uninterpreted: bool,
) -> Result<String, CodegenError> {
    let raw = match ext {
        Some(v) => v,
        None => {
            return Err(CodegenError::ServiceIdOption(if uninterpreted {
                format!(
                    "service {service}: (.modukit.service_id) stayed uninterpreted - \
                     is contracts/modukit.proto imported?"
                )
            } else {
                format!("service {service} carries no (.modukit.service_id) stamp")
            }));
        }
    };
    let dotted_reverse_dns = raw.contains('.')
        && !raw.starts_with('.')
        && !raw.ends_with('.')
        && !raw.contains("..")
        && raw
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'_' || b == b'-');
    if !dotted_reverse_dns {
        return Err(CodegenError::ServiceIdOption(format!(
            "service {service}: {raw:?} is not a dotted canonical id (D9)"
        )));
    }
    Ok(raw)
}

fn decode_method(buf: &[u8]) -> Result<Method, CodegenError> {
    let mut r = Reader::new(buf);
    let mut cur = FieldCursor::new(&mut r);
    let mut m = Method {
        name: String::new(),
        input_type: String::new(),
        output_type: String::new(),
    };
    while let Some((num, value)) = cur.next_field()? {
        match num {
            // MethodDescriptorProto: name = 1, input_type = 2,
            // output_type = 3, options = 4, client_streaming = 5,
            // server_streaming = 6.
            1 => m.name = value.as_len_str()?.to_string(),
            2 => m.input_type = value.as_len_str()?.to_string(),
            3 => m.output_type = value.as_len_str()?.to_string(),
            5 | 6 if value.as_varint()? != 0 => {
                return Err(CodegenError::wire(format!(
                    "method {}: streaming crosses the D6 sync boundary - refused",
                    m.name
                )));
            }
            _ => {}
        }
    }
    Ok(m)
}
