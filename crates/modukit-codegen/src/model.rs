//! The decoded descriptor model: the subset of descriptor.proto the
//! generator reads (names + numbers + types + the modukit option).

use crate::error::CodegenError;

/// `(modukit.service_id)` extension tag: the first slot of the
/// google.protobuf.ServiceOptions extension range (50000-max), declared by
/// hand in contracts/modukit.proto (D12 rule 1: hand-written numbers).
pub const EXT_SERVICE_ID: u32 = 50000;

/// Package of the kernel ledger contract (D27/D51 vocabulary).
pub const RUNTIME_PACKAGE: &str = "org.modukit.runtime";

/// Field label (descriptor.proto `FieldDescriptorProto.Label`: 1 optional,
/// 2 required, 3 repeated; required is refused at decode).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Label {
    Optional,
    Repeated,
}

/// Type (descriptor.proto `FieldDescriptorProto.Type`, values 1..=18).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldType {
    Double,
    Float,
    Int64,
    Uint64,
    Int32,
    Fixed64,
    Fixed32,
    Bool,
    String,
    Group,
    Message,
    Bytes,
    Uint32,
    Enum,
    Sfixed32,
    Sfixed64,
    Sint32,
    Sint64,
}

impl FieldType {
    pub(crate) fn from_type_number(n: u64) -> Result<Self, CodegenError> {
        Ok(match n {
            1 => FieldType::Double,
            2 => FieldType::Float,
            3 => FieldType::Int64,
            4 => FieldType::Uint64,
            5 => FieldType::Int32,
            6 => FieldType::Fixed64,
            7 => FieldType::Fixed32,
            8 => FieldType::Bool,
            9 => FieldType::String,
            10 => FieldType::Group,
            11 => FieldType::Message,
            12 => FieldType::Bytes,
            13 => FieldType::Uint32,
            14 => FieldType::Enum,
            15 => FieldType::Sfixed32,
            16 => FieldType::Sfixed64,
            17 => FieldType::Sint32,
            18 => FieldType::Sint64,
            other => {
                return Err(CodegenError::wire(format!(
                    "unknown FieldDescriptorProto.Type: {other}"
                )));
            }
        })
    }
}

/// One proto field, number included (D12 rule 1).
#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub number: u32,
    pub label: Label,
    pub kind: FieldType,
    /// Fully-qualified `.pkg.Name` for Message/Enum kinds; "" otherwise.
    pub type_name: String,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone)]
pub struct EnumValue {
    pub name: String,
    pub number: i32,
}

#[derive(Debug, Clone)]
pub struct Enum {
    pub name: String,
    pub values: Vec<EnumValue>,
}

impl Enum {
    /// The 0-valued sentinel variant (D12 rule 3); missing => loud error.
    pub fn sentinel(&self) -> Result<&EnumValue, CodegenError> {
        self.values
            .iter()
            .find(|v| v.number == 0)
            .ok_or_else(|| CodegenError::MissingSentinel(self.name.clone()))
    }
}

#[derive(Debug, Clone)]
pub struct Method {
    pub name: String,
    pub input_type: String,
    pub output_type: String,
}

#[derive(Debug, Clone)]
pub struct Service {
    pub name: String,
    pub methods: Vec<Method>,
    /// Value of the `(modukit.service_id)` stamp (D9), validated dotted id.
    pub service_id: String,
}

#[derive(Debug, Clone)]
pub struct File {
    pub name: String,
    pub package: String,
    pub messages: Vec<Message>,
    pub enums: Vec<Enum>,
    pub services: Vec<Service>,
}

impl File {
    pub fn message(&self, bare_name: &str) -> Option<&Message> {
        self.messages.iter().find(|m| m.name == bare_name)
    }

    pub fn module_name(&self) -> String {
        self.package.replace('.', "_")
    }
}
