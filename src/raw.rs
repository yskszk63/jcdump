/// https://docs.oracle.com/javase/specs/jvms/se25/html/jvms-4.html#jvms-4.4.4
use std::io;

use base64::Engine as _;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("io error. {0}")]
    Io(#[from] io::Error),

    #[error("bad magic number")]
    BadMagicNumber,

    #[error("from utf8 error. {0}")]
    FromUtf8(#[from] std::string::FromUtf8Error),

    #[error("serialize error. {0}")]
    Serialize(#[from] serde_json::Error),

    #[error("incorrect attribute_name_index")]
    IncorrectAttributeNameIndex,
}

#[derive(Debug, Serialize)]
pub enum CpInfo {
    Utf8(String),
    Integer(u32),
    Float(u32),
    Long(u32, u32),
    Double(u32, u32),
    Class {
        name_index: u16,
    },
    String {
        string_index: u16,
    },
    Fieldref {
        class_index: u16,
        name_and_type_index: u16,
    },
    Methodref {
        class_index: u16,
        name_and_type_index: u16,
    },
    InterfaceMethodref {
        class_index: u16,
        name_and_type_index: u16,
    },
    NameAndType {
        name_index: u16,
        descriptor_index: u16,
    },
    MethodHandle {
        reference_kind: u8,
        reference_index: u16,
    },
    MethodType {
        descriptor_index: u16,
    },
    Dynamic {
        bootstrap_method_attr_index: u16,
        name_and_type_index: u16,
    },
    InvokeDynamic {
        bootstrap_method_attr_index: u16,
        name_and_type_index: u16,
    },
    Module {
        name_index: u16,
    },
    Package {
        name_index: u16,
    },
}

#[derive(Debug, Serialize)]
pub struct AttributeInfo {
    pub attribute_name_index: u16,
    #[serde(serialize_with = "as_base64")]
    pub info: Vec<u8>,
}

#[derive(Debug, Serialize)]
pub struct FieldInfo {
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<AttributeInfo>,
}

#[derive(Debug, Serialize)]
pub struct MethodInfo {
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<AttributeInfo>,
}

#[derive(Debug, Serialize)]
pub struct ClassFile {
    pub magic: u32,
    pub minor_version: u16,
    pub major_version: u16,
    pub constant_pool: Vec<Option<CpInfo>>,
    pub access_flags: u16,
    pub this_class: u16,
    pub super_class: u16,
    pub interfaces: Vec<u16>,
    pub fields: Vec<FieldInfo>,
    pub methods: Vec<MethodInfo>,
    pub attributes: Vec<AttributeInfo>,
}

fn as_base64<T: AsRef<[u8]>, S: serde::Serializer>(
    val: &T,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&base64::engine::general_purpose::STANDARD.encode(val.as_ref()))
}

fn read_u1<I: io::Read>(input: &mut I) -> io::Result<u8> {
    let mut buf = [0u8; 1];
    input.read_exact(&mut buf)?;
    Ok(u8::from_be_bytes(buf))
}

fn read_u2<I: io::Read>(input: &mut I) -> io::Result<u16> {
    let mut buf = [0u8; 2];
    input.read_exact(&mut buf)?;
    Ok(u16::from_be_bytes(buf))
}

fn read_u4<I: io::Read>(input: &mut I) -> io::Result<u32> {
    let mut buf = [0u8; 4];
    input.read_exact(&mut buf)?;
    Ok(u32::from_be_bytes(buf))
}

fn read_utf8<I: io::Read>(input: &mut I) -> Result<String, ParseError> {
    let len = read_u2(input)?;
    let mut data = vec![0u8; len as usize];
    input.read_exact(&mut data)?;
    Ok(String::from_utf8(data)?)
}

fn read_cp_info<I: io::Read>(input: &mut I) -> Result<CpInfo, ParseError> {
    let tag = read_u1(input)?;
    match tag {
        // CONSTANT_Utf8
        1 => Ok(CpInfo::Utf8(read_utf8(input)?)),

        // CONSTANT_Integer
        3 => Ok(CpInfo::Integer(read_u4(input)?)),

        // CONSTANT_Float
        4 => Ok(CpInfo::Float(read_u4(input)?)),

        // CONSTANT_Long
        5 => Ok(CpInfo::Long(read_u4(input)?, read_u4(input)?)),

        // CONSTANT_Double
        6 => Ok(CpInfo::Double(read_u4(input)?, read_u4(input)?)),

        // CONSTANT_Class
        7 => Ok(CpInfo::Class {
            name_index: read_u2(input)?,
        }),

        // CONSTANT_String
        8 => Ok(CpInfo::String {
            string_index: read_u2(input)?,
        }),

        // CONSTANT_Fieldref
        9 => Ok(CpInfo::Fieldref {
            class_index: read_u2(input)?,
            name_and_type_index: read_u2(input)?,
        }),

        // CONSTANT_Methodref
        10 => Ok(CpInfo::Methodref {
            class_index: read_u2(input)?,
            name_and_type_index: read_u2(input)?,
        }),

        // CONSTANT_InterfaceMethodref
        11 => Ok(CpInfo::InterfaceMethodref {
            class_index: read_u2(input)?,
            name_and_type_index: read_u2(input)?,
        }),

        // CONSTANT_NameAndType
        12 => Ok(CpInfo::NameAndType {
            name_index: read_u2(input)?,
            descriptor_index: read_u2(input)?,
        }),

        // CONSTANT_MethodHandle
        15 => Ok(CpInfo::MethodHandle {
            reference_kind: read_u1(input)?,
            reference_index: read_u2(input)?,
        }),

        // CONSTANT_MethodType
        16 => Ok(CpInfo::MethodType {
            descriptor_index: read_u2(input)?,
        }),

        // CONSTANT_Dynamic
        // TODO Not tested.
        17 => Ok(CpInfo::Dynamic {
            bootstrap_method_attr_index: read_u2(input)?,
            name_and_type_index: read_u2(input)?,
        }),

        // CONSTANT_InvokeDynamic
        18 => Ok(CpInfo::InvokeDynamic {
            bootstrap_method_attr_index: read_u2(input)?,
            name_and_type_index: read_u2(input)?,
        }),

        // CONSTANT_Module
        19 => Ok(CpInfo::Module {
            name_index: read_u2(input)?,
        }),

        // CONSTANT_Package
        20 => Ok(CpInfo::Package {
            name_index: read_u2(input)?,
        }),

        _ => todo!("Unknown tag {tag}"),
    }
}

fn read_attribute_info<I: io::Read>(input: &mut I) -> Result<AttributeInfo, ParseError> {
    let attribute_name_index = read_u2(input)?;
    let attribute_length = read_u4(input)? as usize;
    let mut info = vec![0u8; attribute_length];
    input.read_exact(&mut info)?;

    return Ok(AttributeInfo {
        attribute_name_index,
        info,
    });
}

fn read_field_info<I: io::Read>(input: &mut I) -> Result<FieldInfo, ParseError> {
    let access_flags = read_u2(input)?;
    let name_index = read_u2(input)?;
    let descriptor_index = read_u2(input)?;
    let attributes_count = read_u2(input)? as usize;
    let mut attributes = Vec::with_capacity(attributes_count);
    for _ in 0..attributes_count {
        attributes.push(read_attribute_info(input)?);
    }

    Ok(FieldInfo {
        access_flags,
        name_index,
        descriptor_index,
        attributes,
    })
}

fn read_method_info<I: io::Read>(input: &mut I) -> Result<MethodInfo, ParseError> {
    let access_flags = read_u2(input)?;
    let name_index = read_u2(input)?;
    let descriptor_index = read_u2(input)?;
    let attributes_count = read_u2(input)? as usize;
    let mut attributes = Vec::with_capacity(attributes_count);
    for _ in 0..attributes_count {
        attributes.push(read_attribute_info(input)?);
    }

    Ok(MethodInfo {
        access_flags,
        name_index,
        descriptor_index,
        attributes,
    })
}

pub fn parse<I: io::Read>(input: &mut I) -> Result<ClassFile, ParseError> {
    let magic = read_u4(input)?;
    if magic != 0xcafebabe {
        return Err(ParseError::BadMagicNumber);
    }

    let minor_version = read_u2(input)?;
    let major_version = read_u2(input)?;

    let constant_pool_count = read_u2(input)? as usize;
    let mut constant_pool = Vec::with_capacity(constant_pool_count);
    constant_pool.push(None);
    while constant_pool.len() < constant_pool_count {
        let entry = read_cp_info(input)?;
        match &entry {
            CpInfo::Long(..) | CpInfo::Double(..) => {
                constant_pool.push(Some(entry));
                constant_pool.push(None);
            }
            _ => {
                constant_pool.push(Some(entry));
            }
        };
    }

    let access_flags = read_u2(input)?;
    let this_class = read_u2(input)?;
    let super_class = read_u2(input)?;
    let interfaces_count = read_u2(input)? as usize;
    let mut interfaces = Vec::with_capacity(interfaces_count);
    for _ in 0..interfaces_count {
        interfaces.push(read_u2(input)?);
    }

    let fields_count = read_u2(input)? as usize;
    let mut fields = Vec::with_capacity(fields_count);
    for _ in 0..fields_count {
        fields.push(read_field_info(input)?);
    }

    let method_count = read_u2(input)? as usize;
    let mut methods = Vec::with_capacity(method_count);
    for _ in 0..method_count {
        methods.push(read_method_info(input)?);
    }

    let attributes_count = read_u2(input)? as usize;
    let mut attributes = Vec::with_capacity(attributes_count);
    for _ in 0..attributes_count {
        attributes.push(read_attribute_info(input)?);
    }

    // check EOF
    let n = input.read(&mut [0])?;
    if n != 0 {
        todo!();
    }

    let classfile = ClassFile {
        magic,
        minor_version,
        major_version,
        constant_pool,
        access_flags,
        this_class,
        super_class,
        interfaces,
        fields,
        methods,
        attributes,
    };

    Ok(classfile)
}
