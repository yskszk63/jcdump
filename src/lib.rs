mod raw;

use std::{io::{self, Write}, usize};

use base64::Engine as _;
use serde::Serialize;

use crate::raw::ParseError;

#[derive(Debug)]
pub struct ClassFileVersion {
    major_version: u16,
    minor_version: u16,
}

impl Serialize for ClassFileVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{}.{}", self.major_version, self.minor_version))
    }
}

#[derive(Debug)]
pub struct Magic;

impl Serialize for Magic {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str("0xCAFEBABE")
    }
}

#[derive(Debug, Serialize)]
pub enum ReferenceKind {
    RefGetField,
    RefGetStatic,
    RefPutField,
    RefPutStatic,
    RefInvokeVirtual,
    RefInvokeStatic,
    RefInvokeSpecial,
    RefNewInvokeSpecial,
    RefNewInvokeInterface,
}

#[derive(Debug, Serialize)]
pub struct BootstrapMethod<S: AsRef<str>> {
    reference_kind: ReferenceKind,
    class: S,
    name: S,
    descriptor: S,
    bootstrap_arguments: Vec<CpInfo<S>>,
}

#[derive(Debug, Serialize)]
pub enum CpInfo<S: AsRef<str>> {
    Utf8(S),
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    Class {
        name: S,
    },
    String {
        string: S,
    },
    Fieldref {
        class: S,
        name: S,
        descriptor: S,
    },
    Methodref {
        class: S,
        name: S,
        descriptor: S,
    },
    InterfaceMethodref {
        class: S,
        name: S,
        descriptor: S,
    },
    NameAndType {
        name: S,
        descriptor: S,
    },
    MethodHandle {
        reference_kind: ReferenceKind,
        class: S,
        name: S,
        descriptor: S,
    },
    MethodType {
        descriptor: S,
    },
    Dynamic {
        bootstrap_method_attr: (), // TODO
        name: S,
        descriptor: S,
    },
    InvokeDynamic {
        bootstrap_method_attr: (), // TODO
        name: S,
        descriptor: S,
    },
    Module {
        name: S,
    },
    Package {
        name: S,
    },
}

#[derive(Debug, Serialize)]
pub enum ConstantValueAttribute<S: AsRef<str>> {
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    String(S),
}

#[repr(u16)]
#[derive(Debug, Serialize, Clone, Copy)]
pub enum InnerClassAccessFlags {
    AccPublic = 0x0001,
    AccPrivate = 0x0002,
    AccProtected = 0x0004,
    AccStatic = 0x0008,
    AccFinal = 0x0010,
    AccInterface = 0x0200,
    AccAbstract = 0x0400,
    AccSynthetic = 0x1000,
    AccAnnotation = 0x2000,
    AccEnum = 0x4000,
}

impl InnerClassAccessFlags {
    const VALUES: [Self; 10] = [
        Self::AccPublic,
        Self::AccPrivate,
        Self::AccProtected,
        Self::AccStatic,
        Self::AccFinal,
        Self::AccInterface,
        Self::AccAbstract,
        Self::AccSynthetic,
        Self::AccAnnotation,
        Self::AccEnum,
    ];
}

#[derive(Debug, Serialize)]
pub struct InnerClass<S: AsRef<str>> {
    inner_class_info: S,
    outer_class_info: Option<S>,
    inner_name: Option<S>,
    inner_class_access_flags: Vec<InnerClassAccessFlags>,
}

#[derive(Debug, Serialize)]
pub enum AttributeInfo<S: AsRef<str>, B: AsRef<[u8]>> {
    ConstantValue(ConstantValueAttribute<S>),
    Code(#[serde(serialize_with = "as_base64")] B),
    Exceptions(Vec<S>),
    SourceFile(S),
    BootstrapMethods(Vec<BootstrapMethod<S>>),
    InnerClasses(Vec<InnerClass<S>>),
    Unknown(S, #[serde(serialize_with = "as_base64")] B),
}

#[repr(u16)]
#[derive(Debug, Serialize, Clone, Copy)]
pub enum FieldAccessFlags {
    AccPublic = 0x0001,
    AccPrivate = 0x0002,
    AccProcted = 0x0004,
    AccStatic = 0x0008,
    AccFinal = 0x0010,
    AccVolatile = 0x0040,
    AccTransient = 0x0080,
    AccSynthetic = 0x1000,
    AccEnum = 0x4000,
}

impl FieldAccessFlags {
    const VALUES: [Self; 9] = [
        Self::AccPublic,
        Self::AccPrivate,
        Self::AccProcted,
        Self::AccStatic,
        Self::AccFinal,
        Self::AccVolatile,
        Self::AccTransient,
        Self::AccSynthetic,
        Self::AccEnum,
    ];
}

#[derive(Debug, Serialize)]
pub struct FieldInfo<S: AsRef<str>, B: AsRef<[u8]>> {
    access_flags: Vec<FieldAccessFlags>,
    name: S,
    descriptor: S,
    attributes: Vec<AttributeInfo<S, B>>,
}

#[repr(u16)]
#[derive(Debug, Serialize, Clone, Copy)]
pub enum MethodAccessFlags {
    AccPublic = 0x0001,
    AccPrivate = 0x0002,
    AccProcted = 0x0004,
    AccStatic = 0x0008,
    AccFinal = 0x0010,
    AccSynthronized = 0x0020,
    AccBridge = 0x0040,
    AccVarargs = 0x0080,
    AccNative = 0x0100,
    AccAbstract = 0x0400,
    AccStrict = 0x0800,
    AccSynthetic = 0x1000,
}

impl MethodAccessFlags {
    const VALUES: [Self; 12] = [
        Self::AccPublic,
        Self::AccPrivate,
        Self::AccProcted,
        Self::AccStatic,
        Self::AccFinal,
        Self::AccSynthronized,
        Self::AccBridge,
        Self::AccVarargs,
        Self::AccNative,
        Self::AccAbstract,
        Self::AccStrict,
        Self::AccSynthetic,
    ];
}

#[derive(Debug, Serialize)]
pub struct MethodInfo<S: AsRef<str>, B: AsRef<[u8]>> {
    access_flags: Vec<MethodAccessFlags>,
    name: S,
    descriptor: S,
    attributes: Vec<AttributeInfo<S, B>>,
}

#[repr(u16)]
#[derive(Debug, Serialize, Clone, Copy)]
pub enum ClassAccessFlags {
    AccPublic = 0x0001,
    AccFinal = 0x0010,
    AccSuper = 0x0020,
    AccInterface = 0x0200,
    AccAbstract = 0x0400,
    AccSynthetic = 0x1000,
    AccAnnotation = 0x2000,
    AccEnum = 0x4000,
    AccModule = 0x8000,
}

impl ClassAccessFlags {
    const VALUES: [Self; 9] = [
        Self::AccPublic,
        Self::AccFinal,
        Self::AccSuper,
        Self::AccInterface,
        Self::AccAbstract,
        Self::AccSynthetic,
        Self::AccAnnotation,
        Self::AccEnum,
        Self::AccModule,
    ];
}

#[derive(Debug, Serialize)]
pub struct ClassFile<S: AsRef<str>, B: AsRef<[u8]>> {
    magic: Magic,
    version: ClassFileVersion,
    constant_pool: Vec<Option<CpInfo<S>>>,
    access_flags: Vec<ClassAccessFlags>,
    this_class: S,
    super_class: Option<S>,
    interfaces: Vec<S>,
    fields: Vec<FieldInfo<S, B>>,
    methods: Vec<MethodInfo<S, B>>,
    attributes: Vec<AttributeInfo<S, B>>,
}

fn as_base64<T: AsRef<[u8]>, S: serde::Serializer>(
    val: &T,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&base64::engine::general_purpose::STANDARD.encode(val.as_ref()))
}

fn parse_cp_info<'a>(
    pool: &'a [Option<raw::CpInfo>],
    item: &'a Option<raw::CpInfo>,
) -> Result<Option<CpInfo<&'a str>>, ParseError> {
    let Some(item) = item else { return Ok(None) };

    Ok(Some(match item {
        raw::CpInfo::Utf8(val) => CpInfo::Utf8(&val),

        raw::CpInfo::Integer(val) => CpInfo::Integer(*val as i32),

        raw::CpInfo::Float(val) => CpInfo::Float(f32::from_bits(*val)),

        raw::CpInfo::Long(hi, lo) => CpInfo::Long(((*hi as u64) << 32u64 | *lo as u64) as i64),

        raw::CpInfo::Double(hi, lo) => {
            CpInfo::Double(f64::from_bits((*hi as u64) << 32u64 | *lo as u64))
        }

        raw::CpInfo::Class { name_index } => {
            let Some(Some(raw::CpInfo::Utf8(name))) = pool.get(*name_index as usize) else {
                todo!()
            };
            CpInfo::Class { name }
        }

        raw::CpInfo::String { string_index } => {
            let Some(Some(raw::CpInfo::Utf8(string))) = pool.get(*string_index as usize) else {
                todo!()
            };
            CpInfo::String { string }
        }

        raw::CpInfo::Fieldref {
            class_index,
            name_and_type_index,
        } => {
            let Some(class) = pool.get(*class_index as usize) else {
                todo!()
            };
            let Some(CpInfo::Class { name: class }) = parse_cp_info(pool, class)? else {
                todo!()
            };

            let Some(name_and_type) = pool.get(*name_and_type_index as usize) else {
                todo!()
            };
            let Some(CpInfo::NameAndType { name, descriptor }) =
                parse_cp_info(pool, name_and_type)?
            else {
                todo!()
            };

            CpInfo::Fieldref {
                class,
                name,
                descriptor,
            }
        }

        raw::CpInfo::Methodref {
            class_index,
            name_and_type_index,
        } => {
            let Some(class) = pool.get(*class_index as usize) else {
                todo!()
            };
            let Some(CpInfo::Class { name: class }) = parse_cp_info(pool, class)? else {
                todo!()
            };

            let Some(name_and_type) = pool.get(*name_and_type_index as usize) else {
                todo!()
            };
            let Some(CpInfo::NameAndType { name, descriptor }) =
                parse_cp_info(pool, name_and_type)?
            else {
                todo!()
            };

            CpInfo::Methodref {
                class,
                name,
                descriptor,
            }
        }

        raw::CpInfo::InterfaceMethodref {
            class_index,
            name_and_type_index,
        } => {
            let Some(class) = pool.get(*class_index as usize) else {
                todo!()
            };
            let Some(CpInfo::Class { name: class }) = parse_cp_info(pool, class)? else {
                todo!()
            };

            let Some(name_and_type) = pool.get(*name_and_type_index as usize) else {
                todo!()
            };
            let Some(CpInfo::NameAndType { name, descriptor }) =
                parse_cp_info(pool, name_and_type)?
            else {
                todo!()
            };

            CpInfo::InterfaceMethodref {
                class,
                name,
                descriptor,
            }
        }

        raw::CpInfo::NameAndType {
            name_index,
            descriptor_index,
        } => {
            let Some(Some(raw::CpInfo::Utf8(name))) = pool.get(*name_index as usize) else {
                todo!()
            };
            let Some(Some(raw::CpInfo::Utf8(descriptor))) = pool.get(*descriptor_index as usize)
            else {
                todo!()
            };
            CpInfo::NameAndType { name, descriptor }
        }

        raw::CpInfo::MethodHandle {
            reference_kind,
            reference_index,
        } => {
            let reference_kind = match reference_kind {
                1 => ReferenceKind::RefGetField,
                2 => ReferenceKind::RefGetStatic,
                3 => ReferenceKind::RefPutField,
                4 => ReferenceKind::RefPutStatic,
                5 => ReferenceKind::RefInvokeVirtual,
                6 => ReferenceKind::RefInvokeStatic,
                7 => ReferenceKind::RefInvokeSpecial,
                8 => ReferenceKind::RefNewInvokeSpecial,
                9 => ReferenceKind::RefNewInvokeInterface,
                _ => todo!(),
            };

            let Some(reference) = pool.get(*reference_index as usize) else {
                todo!()
            };
            let Some(
                CpInfo::Fieldref {
                    class,
                    name,
                    descriptor,
                }
                | CpInfo::Methodref {
                    class,
                    name,
                    descriptor,
                }
                | CpInfo::InterfaceMethodref {
                    class,
                    name,
                    descriptor,
                },
            ) = parse_cp_info(pool, reference)?
            else {
                todo!()
            };
            CpInfo::MethodHandle {
                reference_kind,
                class,
                name,
                descriptor,
            }
        }

        raw::CpInfo::MethodType { descriptor_index } => {
            let Some(Some(raw::CpInfo::Utf8(descriptor))) = pool.get(*descriptor_index as usize)
            else {
                todo!()
            };
            CpInfo::MethodType { descriptor }
        }

        raw::CpInfo::Dynamic {
            name_and_type_index,
            ..
        } => {
            let Some(name_and_type) = pool.get(*name_and_type_index as usize) else {
                todo!()
            };
            let Some(CpInfo::NameAndType { name, descriptor }) =
                parse_cp_info(pool, name_and_type)?
            else {
                todo!()
            };

            CpInfo::Dynamic {
                bootstrap_method_attr: (), // TODO
                name,
                descriptor,
            }
        }

        raw::CpInfo::InvokeDynamic {
            name_and_type_index,
            ..
        } => {
            let Some(name_and_type) = pool.get(*name_and_type_index as usize) else {
                todo!()
            };
            let Some(CpInfo::NameAndType { name, descriptor }) =
                parse_cp_info(pool, name_and_type)?
            else {
                todo!()
            };

            CpInfo::InvokeDynamic {
                bootstrap_method_attr: (), // TODO
                name,
                descriptor,
            }
        }

        raw::CpInfo::Module { name_index } => {
            let Some(name) = pool.get(*name_index as usize) else {
                todo!()
            };
            let Some(CpInfo::Utf8(name)) = parse_cp_info(pool, name)? else {
                todo!()
            };

            CpInfo::Module { name }
        }

        raw::CpInfo::Package { name_index } => {
            let Some(name) = pool.get(*name_index as usize) else {
                todo!()
            };
            let Some(CpInfo::Utf8(name)) = parse_cp_info(pool, name)? else {
                todo!()
            };

            CpInfo::Package { name }
        }
    }))
}

fn parse_class_access_flags(flags: u16) -> Result<Vec<ClassAccessFlags>, ParseError> {
    let mut ret = vec![];

    let mut wants = 0;
    for value in ClassAccessFlags::VALUES {
        if flags & value as u16 != 0 {
            ret.push(value);
            wants |= value as u16;
        }
    }

    if flags != wants {
        todo!() // TODO containts Unknown flag
    }

    Ok(ret)
}

fn parse_field_access_flags(flags: u16) -> Result<Vec<FieldAccessFlags>, ParseError> {
    let mut ret = vec![];

    let mut wants = 0;
    for value in FieldAccessFlags::VALUES {
        if flags & value as u16 != 0 {
            ret.push(value);
            wants |= value as u16;
        }
    }

    if flags != wants {
        todo!() // TODO containts Unknown flag
    }

    Ok(ret)
}

fn parse_method_access_flags(flags: u16) -> Result<Vec<MethodAccessFlags>, ParseError> {
    let mut ret = vec![];

    let mut wants = 0;
    for value in MethodAccessFlags::VALUES {
        if flags & value as u16 != 0 {
            ret.push(value);
            wants |= value as u16;
        }
    }

    if flags != wants {
        todo!() // TODO containts Unknown flag
    }

    Ok(ret)
}

fn parse_inner_class_access_flags(flags: u16) -> Result<Vec<InnerClassAccessFlags>, ParseError> {
    let mut ret = vec![];

    let mut wants = 0;
    for value in InnerClassAccessFlags::VALUES {
        if flags & value as u16 != 0 {
            ret.push(value);
            wants |= value as u16;
        }
    }

    if flags != wants {
        todo!() // TODO containts Unknown flag
    }

    Ok(ret)
}

fn parse_attribute_info<'a>(
    pool: &'a [Option<raw::CpInfo>],
    attribute: &'a raw::AttributeInfo,
) -> Result<AttributeInfo<&'a str, &'a [u8]>, ParseError> {
    let Some(attribute_name) = pool.get(attribute.attribute_name_index as usize) else {
        todo!()
    };
    let Some(CpInfo::Utf8(attribute_name)) = parse_cp_info(pool, attribute_name)? else {
        todo!()
    };

    Ok(match attribute_name {
        "ConstantValue" => {
            let (chunks, []) = attribute.info.as_chunks() else {
                todo!()
            };
            let Some(chunk) = chunks.get(0) else { todo!() };
            let index = u16::from_be_bytes(*chunk);

            let Some(item) = pool.get(index as usize) else {
                todo!()
            };
            match parse_cp_info(pool, item)? {
                Some(CpInfo::Integer(val)) => {
                    AttributeInfo::ConstantValue(ConstantValueAttribute::Integer(val))
                }
                Some(CpInfo::Float(val)) => {
                    AttributeInfo::ConstantValue(ConstantValueAttribute::Float(val))
                }
                Some(CpInfo::Long(val)) => {
                    AttributeInfo::ConstantValue(ConstantValueAttribute::Long(val))
                }
                Some(CpInfo::Double(val)) => {
                    AttributeInfo::ConstantValue(ConstantValueAttribute::Double(val))
                }
                Some(CpInfo::String { string }) => {
                    AttributeInfo::ConstantValue(ConstantValueAttribute::String(string))
                }
                _ => todo!(),
            }
        }

        "Code" => AttributeInfo::Code(&attribute.info),

        "Exceptions" => {
            let (chunks, []) = attribute.info.as_chunks() else {
                todo!()
            };
            let Some(first) = chunks.get(0) else { todo!() };
            let n = u16::from_be_bytes(*first) as usize;
            let exception_index_table = &chunks[1..];
            if exception_index_table.len() != n {
                todo!()
            };
            let exceptions = exception_index_table
                .iter()
                .map(|i| u16::from_be_bytes(*i))
                .map(|i| {
                    let Some(item) = pool.get(i as usize) else {
                        todo!()
                    };
                    let Some(CpInfo::Class { name }) = parse_cp_info(pool, item)? else {
                        todo!()
                    };
                    Ok::<_, ParseError>(name)
                })
                .collect::<Result<_, _>>()?;
            AttributeInfo::Exceptions(exceptions)
        }

        "SourceFile" => {
            let (chunks, []) = attribute.info.as_chunks() else {
                todo!()
            };
            let Some(chunk) = chunks.get(0) else { todo!() };
            let index = u16::from_be_bytes(*chunk);

            let Some(item) = pool.get(index as usize) else {
                todo!()
            };
            let Some(CpInfo::Utf8(val)) = parse_cp_info(pool, item)? else {
                todo!()
            };
            AttributeInfo::SourceFile(val)
        }

        "BootstrapMethods" => {
            let (chunks, []) = attribute.info.as_chunks() else {
                todo!()
            };
            let mut chunks = chunks.iter().map(|v| u16::from_be_bytes(*v));
            let Some(num_bootstrap_methods) = chunks.next() else {
                todo!()
            };

            let mut items = Vec::with_capacity(num_bootstrap_methods as usize);
            for _ in 0..num_bootstrap_methods {
                let Some(bootstrap_method_ref) = chunks.next() else {
                    todo!()
                };
                let Some(item) = pool.get(bootstrap_method_ref as usize) else {
                    todo!()
                };
                let Some(CpInfo::MethodHandle {
                    reference_kind,
                    class,
                    name,
                    descriptor,
                }) = parse_cp_info(pool, item)?
                else {
                    todo!()
                };

                let Some(num_bootstrap_arguments) = chunks.next() else {
                    todo!()
                };
                let bootstrap_arguments = chunks
                    .by_ref()
                    .take(num_bootstrap_arguments as usize)
                    .map(|v| {
                        let Some(item) = pool.get(v as usize) else {
                            todo!()
                        };
                        let Some(item) = parse_cp_info(pool, item)? else {
                            todo!()
                        };
                        Ok::<_, ParseError>(item)
                    })
                    .collect::<Result<_, _>>()?;
                items.push(BootstrapMethod {
                    reference_kind,
                    class,
                    name,
                    descriptor,
                    bootstrap_arguments,
                });
            }
            if chunks.next() != None {
                todo!()
            }

            AttributeInfo::BootstrapMethods(items)
        }

        "InnerClasses" => {
            let (chunks, []) = attribute.info.as_chunks() else {
                todo!()
            };
            let mut chunks = chunks.iter().map(|v| u16::from_be_bytes(*v));
            let Some(numer_of_classes) = chunks.next() else {
                todo!()
            };

            let mut items = Vec::with_capacity(numer_of_classes as usize);
            for _ in 0..numer_of_classes {
                let Some(inner_class_info) = chunks.next() else {
                    todo!()
                };
                let Some(item) = pool.get(inner_class_info as usize) else {
                    todo!()
                };
                let Some(CpInfo::Class {
                    name: inner_class_info,
                }) = parse_cp_info(pool, item)?
                else {
                    todo!()
                };

                let Some(outer_class_info) = chunks.next() else {
                    todo!()
                };
                let outer_class_info = if outer_class_info == 0 {
                    None
                } else {
                    let Some(item) = pool.get(outer_class_info as usize) else {
                        todo!()
                    };
                    let Some(CpInfo::Class {
                        name: outer_class_info,
                    }) = parse_cp_info(pool, item)?
                    else {
                        todo!()
                    };
                    Some(outer_class_info)
                };

                let Some(inner_name) = chunks.next() else {
                    todo!()
                };
                let inner_name = if inner_name == 0 {
                    None
                } else {
                    let Some(item) = pool.get(inner_name as usize) else {
                        todo!()
                    };
                    let Some(CpInfo::Utf8(inner_name)) = parse_cp_info(pool, item)? else {
                        todo!()
                    };
                    Some(inner_name)
                };

                let Some(inner_class_access_flags) = chunks.next() else {
                    todo!()
                };
                let inner_class_access_flags =
                    parse_inner_class_access_flags(inner_class_access_flags)?;

                items.push(InnerClass {
                    inner_class_info,
                    outer_class_info,
                    inner_name,
                    inner_class_access_flags,
                });
            }
            if chunks.next() != None {
                todo!()
            }

            AttributeInfo::InnerClasses(items)
        }

        // TODO
        "Module" => AttributeInfo::Unknown(attribute_name, &attribute.info),

        _ => AttributeInfo::Unknown(attribute_name, &attribute.info),
        //name => todo!("{name}"),
    })
}

fn parse_field<'a>(
    pool: &'a [Option<raw::CpInfo>],
    field: &'a raw::FieldInfo,
) -> Result<FieldInfo<&'a str, &'a [u8]>, ParseError> {
    let access_flags = parse_field_access_flags(field.access_flags)?;

    let Some(name) = pool.get(field.name_index as usize) else {
        todo!()
    };
    let Some(CpInfo::Utf8(name)) = parse_cp_info(pool, name)? else {
        todo!()
    };

    let Some(descriptor) = pool.get(field.descriptor_index as usize) else {
        todo!()
    };
    let Some(CpInfo::Utf8(descriptor)) = parse_cp_info(pool, descriptor)? else {
        todo!()
    };

    let attributes = field
        .attributes
        .iter()
        .map(|item| parse_attribute_info(pool, item))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(FieldInfo {
        access_flags,
        name,
        descriptor,
        attributes,
    })
}

fn parse_method<'a>(
    pool: &'a [Option<raw::CpInfo>],
    field: &'a raw::MethodInfo,
) -> Result<MethodInfo<&'a str, &'a [u8]>, ParseError> {
    let access_flags = parse_method_access_flags(field.access_flags)?;

    let Some(name) = pool.get(field.name_index as usize) else {
        todo!()
    };
    let Some(CpInfo::Utf8(name)) = parse_cp_info(pool, name)? else {
        todo!()
    };

    let Some(descriptor) = pool.get(field.descriptor_index as usize) else {
        todo!()
    };
    let Some(CpInfo::Utf8(descriptor)) = parse_cp_info(pool, descriptor)? else {
        todo!()
    };

    let attributes = field
        .attributes
        .iter()
        .map(|item| parse_attribute_info(pool, item))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(MethodInfo {
        access_flags,
        name,
        descriptor,
        attributes,
    })
}

pub fn parse_raw<I: io::Read>(input: &mut I) -> Result<raw::ClassFile, ParseError> {
    raw::parse(input)
}

pub fn wrap<'a>(raw: &'a raw::ClassFile) -> Result<ClassFile<&'a str, &'a [u8]>, ParseError> {
    if raw.magic != 0xCAFEBABE {
        panic!("magic != 0xCAFEBABE")
    }

    let constant_pool = raw
        .constant_pool
        .iter()
        .map(|item| parse_cp_info(&raw.constant_pool, item))
        .collect::<Result<Vec<_>, _>>()?;

    let access_flags = parse_class_access_flags(raw.access_flags)?;

    let Some(this_class) = raw.constant_pool.get(raw.this_class as usize) else {
        todo!()
    };
    let Some(CpInfo::Class { name: this_class }) = parse_cp_info(&raw.constant_pool, this_class)?
    else {
        todo!()
    };

    let super_class = if raw.super_class == 0 {
        None
    } else {
        let Some(super_class) = raw.constant_pool.get(raw.super_class as usize) else {
            todo!()
        };
        let Some(CpInfo::Class { name }) = parse_cp_info(&raw.constant_pool, super_class)? else {
            todo!()
        };
        Some(name)
    };

    let interfaces = raw
        .interfaces
        .iter()
        .map(|v| {
            let Some(interface) = raw.constant_pool.get(*v as usize) else {
                todo!()
            };
            let Some(CpInfo::Class { name }) = parse_cp_info(&raw.constant_pool, interface)? else {
                todo!()
            };
            Ok::<&'a str, ParseError>(name)
        })
        .collect::<Result<_, _>>()?;

    let fields = raw
        .fields
        .iter()
        .map(|item| parse_field(&raw.constant_pool, item))
        .collect::<Result<Vec<_>, _>>()?;

    let methods = raw
        .methods
        .iter()
        .map(|item| parse_method(&raw.constant_pool, item))
        .collect::<Result<Vec<_>, _>>()?;

    let attributes = raw
        .attributes
        .iter()
        .map(|item| parse_attribute_info(&raw.constant_pool, item))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ClassFile {
        magic: Magic,
        version: ClassFileVersion {
            major_version: raw.major_version,
            minor_version: raw.minor_version,
        },
        constant_pool,
        access_flags,
        this_class,
        super_class,
        interfaces,
        fields,
        methods,
        attributes,
    })
}

//#[cfg(all(target_arch = "wasm32", target_os = "wasi"))]
#[unsafe(no_mangle)]
pub extern "C" fn parse() -> std::ffi::c_int {
    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();

    let raw = match parse_raw(&mut stdin) {
        Ok(raw) => raw,
        Err(err) => {
            eprintln!("{err}");
            return 1;
        },
    };

    let data = match wrap(&raw) {
        Ok(data) => data,
        Err(err) => {
            eprintln!("{err}");
            return 1;
        },
    };

    match serde_json::to_writer(&mut stdout, &data) {
        Ok(..) => {},
        Err(err) => {
            eprintln!("{err}");
            return 1;
        },
    };

    match stdout.flush() {
        Ok(..) => {},
        Err(err) => {
            eprintln!("{err}");
            return 1;
        },
    };

    return 0;
}
