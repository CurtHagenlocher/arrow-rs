// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The metadata key used for storing the JSON encoded [`Schema`]
pub const SCHEMA_METADATA_KEY: &str = "avro.schema";

/// Either a [`PrimitiveType`] or a reference to a previously defined named type
///
/// <https://avro.apache.org/docs/1.11.1/specification/#names>
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
/// A type name in an Avro schema
///
/// This represents the different ways a type can be referenced in an Avro schema.
pub enum TypeName<'a> {
    /// A primitive type like null, boolean, int, etc.
    Primitive(PrimitiveType),
    /// A reference to another named type
    Ref(&'a str),
}

/// A primitive type
///
/// <https://avro.apache.org/docs/1.11.1/specification/#primitive-types>
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PrimitiveType {
    /// null: no value
    Null,
    /// boolean: a binary value
    Boolean,
    /// int: 32-bit signed integer
    Int,
    /// long: 64-bit signed integer
    Long,
    /// float: single precision (32-bit) IEEE 754 floating-point number
    Float,
    /// double: double precision (64-bit) IEEE 754 floating-point number
    Double,
    /// bytes: sequence of 8-bit unsigned bytes
    Bytes,
    /// string: Unicode character sequence
    String,
}

/// Additional attributes within a [`Schema`]
///
/// <https://avro.apache.org/docs/1.11.1/specification/#schema-declaration>
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Attributes<'a> {
    /// A logical type name
    ///
    /// <https://avro.apache.org/docs/1.11.1/specification/#logical-types>
    #[serde(default)]
    pub logical_type: Option<&'a str>,

    /// Additional JSON attributes
    #[serde(flatten)]
    pub additional: HashMap<&'a str, serde_json::Value>,
}

impl Attributes<'_> {
    /// Returns the field metadata for this [`Attributes`]
    pub(crate) fn field_metadata(&self) -> HashMap<String, String> {
        self.additional
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }
}

/// A type definition that is not a variant of [`ComplexType`]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Type<'a> {
    /// The type of this Avro data structure
    #[serde(borrow)]
    pub r#type: TypeName<'a>,
    /// Additional attributes associated with this type
    #[serde(flatten)]
    pub attributes: Attributes<'a>,
}

/// An Avro schema
///
/// This represents the different shapes of Avro schemas as defined in the specification.
/// See <https://avro.apache.org/docs/1.11.1/specification/#schemas> for more details.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Schema<'a> {
    /// A direct type name (primitive or reference)
    #[serde(borrow)]
    TypeName(TypeName<'a>),
    /// A union of multiple schemas (e.g., ["null", "string"])
    #[serde(borrow)]
    Union(Vec<Schema<'a>>),
    /// A complex type such as record, array, map, etc.
    #[serde(borrow)]
    Complex(ComplexType<'a>),
    /// A type with attributes
    #[serde(borrow)]
    Type(Type<'a>),
}

/// A complex type
///
/// <https://avro.apache.org/docs/1.11.1/specification/#complex-types>
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ComplexType<'a> {
    /// Record type: a sequence of fields with names and types
    #[serde(borrow)]
    Record(Record<'a>),
    /// Enum type: a set of named values
    #[serde(borrow)]
    Enum(Enum<'a>),
    /// Array type: a sequence of values of the same type
    #[serde(borrow)]
    Array(Array<'a>),
    /// Map type: a mapping from strings to values of the same type
    #[serde(borrow)]
    Map(Map<'a>),
    /// Fixed type: a fixed-size byte array
    #[serde(borrow)]
    Fixed(Fixed<'a>),
}

/// A record
///
/// <https://avro.apache.org/docs/1.11.1/specification/#schema-record>
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Record<'a> {
    /// Name of the record
    #[serde(borrow)]
    pub name: &'a str,
    /// Optional namespace for the record, provides a way to organize names
    #[serde(borrow, default)]
    pub namespace: Option<&'a str>,
    /// Optional documentation string for the record
    #[serde(borrow, default)]
    pub doc: Option<&'a str>,
    /// Alternative names for this record
    #[serde(borrow, default)]
    pub aliases: Vec<&'a str>,
    /// The fields contained in this record
    #[serde(borrow)]
    pub fields: Vec<Field<'a>>,
    /// Additional attributes for this record
    #[serde(flatten)]
    pub attributes: Attributes<'a>,
}

/// A field within a [`Record`]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Field<'a> {
    /// Name of the field within the record
    #[serde(borrow)]
    pub name: &'a str,
    /// Optional documentation for this field
    #[serde(borrow, default)]
    pub doc: Option<&'a str>,
    /// The field's type definition
    #[serde(borrow)]
    pub r#type: Schema<'a>,
    /// Optional default value for this field
    #[serde(borrow, default)]
    pub default: Option<&'a str>,
}

/// An enumeration
///
/// <https://avro.apache.org/docs/1.11.1/specification/#enums>
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Enum<'a> {
    /// Name of the enum
    #[serde(borrow)]
    pub name: &'a str,
    /// Optional namespace for the enum, provides organizational structure
    #[serde(borrow, default)]
    pub namespace: Option<&'a str>,
    /// Optional documentation string describing the enum
    #[serde(borrow, default)]
    pub doc: Option<&'a str>,
    /// Alternative names for this enum
    #[serde(borrow, default)]
    pub aliases: Vec<&'a str>,
    /// The symbols (values) that this enum can have
    #[serde(borrow)]
    pub symbols: Vec<&'a str>,
    /// Optional default value for this enum
    #[serde(borrow, default)]
    pub default: Option<&'a str>,
    /// Additional attributes for this enum
    #[serde(flatten)]
    pub attributes: Attributes<'a>,
}

/// An array
///
/// <https://avro.apache.org/docs/1.11.1/specification/#arrays>
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Array<'a> {
    /// The schema for items in this array
    #[serde(borrow)]
    pub items: Box<Schema<'a>>,
    /// Additional attributes for this array
    #[serde(flatten)]
    pub attributes: Attributes<'a>,
}

/// A map
///
/// <https://avro.apache.org/docs/1.11.1/specification/#maps>
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Map<'a> {
    /// The schema for values in this map
    #[serde(borrow)]
    pub values: Box<Schema<'a>>,
    /// Additional attributes for this map
    #[serde(flatten)]
    pub attributes: Attributes<'a>,
}

/// A fixed length binary array
///
/// <https://avro.apache.org/docs/1.11.1/specification/#fixed>
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fixed<'a> {
    /// Name of the fixed type
    #[serde(borrow)]
    pub name: &'a str,
    /// Optional namespace for the fixed type
    #[serde(borrow, default)]
    pub namespace: Option<&'a str>,
    /// Alternative names for this fixed type
    #[serde(borrow, default)]
    pub aliases: Vec<&'a str>,
    /// The number of bytes in this fixed type
    pub size: usize,
    /// Additional attributes for this fixed type
    #[serde(flatten)]
    pub attributes: Attributes<'a>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::{AvroDataType, AvroField};
    use arrow_schema::{DataType, Fields, TimeUnit};
    use serde_json::json;

    #[test]
    fn test_deserialize() {
        let t: Schema = serde_json::from_str("\"string\"").unwrap();
        assert_eq!(
            t,
            Schema::TypeName(TypeName::Primitive(PrimitiveType::String))
        );

        let t: Schema = serde_json::from_str("[\"int\", \"null\"]").unwrap();
        assert_eq!(
            t,
            Schema::Union(vec![
                Schema::TypeName(TypeName::Primitive(PrimitiveType::Int)),
                Schema::TypeName(TypeName::Primitive(PrimitiveType::Null)),
            ])
        );

        let t: Type = serde_json::from_str(
            r#"{
                   "type":"long",
                   "logicalType":"timestamp-micros"
                }"#,
        )
        .unwrap();

        let timestamp = Type {
            r#type: TypeName::Primitive(PrimitiveType::Long),
            attributes: Attributes {
                logical_type: Some("timestamp-micros"),
                additional: Default::default(),
            },
        };

        assert_eq!(t, timestamp);

        let t: ComplexType = serde_json::from_str(
            r#"{
                   "type":"fixed",
                   "name":"fixed",
                   "namespace":"topLevelRecord.value",
                   "size":11,
                   "logicalType":"decimal",
                   "precision":25,
                   "scale":2
                }"#,
        )
        .unwrap();

        let decimal = ComplexType::Fixed(Fixed {
            name: "fixed",
            namespace: Some("topLevelRecord.value"),
            aliases: vec![],
            size: 11,
            attributes: Attributes {
                logical_type: Some("decimal"),
                additional: vec![("precision", json!(25)), ("scale", json!(2))]
                    .into_iter()
                    .collect(),
            },
        });

        assert_eq!(t, decimal);

        let schema: Schema = serde_json::from_str(
            r#"{
               "type":"record",
               "name":"topLevelRecord",
               "fields":[
                  {
                     "name":"value",
                     "type":[
                        {
                           "type":"fixed",
                           "name":"fixed",
                           "namespace":"topLevelRecord.value",
                           "size":11,
                           "logicalType":"decimal",
                           "precision":25,
                           "scale":2
                        },
                        "null"
                     ]
                  }
               ]
            }"#,
        )
        .unwrap();

        assert_eq!(
            schema,
            Schema::Complex(ComplexType::Record(Record {
                name: "topLevelRecord",
                namespace: None,
                doc: None,
                aliases: vec![],
                fields: vec![Field {
                    name: "value",
                    doc: None,
                    r#type: Schema::Union(vec![
                        Schema::Complex(decimal),
                        Schema::TypeName(TypeName::Primitive(PrimitiveType::Null)),
                    ]),
                    default: None,
                },],
                attributes: Default::default(),
            }))
        );

        let schema: Schema = serde_json::from_str(
            r#"{
                  "type": "record",
                  "name": "LongList",
                  "aliases": ["LinkedLongs"],
                  "fields" : [
                    {"name": "value", "type": "long"},
                    {"name": "next", "type": ["null", "LongList"]}
                  ]
                }"#,
        )
        .unwrap();

        assert_eq!(
            schema,
            Schema::Complex(ComplexType::Record(Record {
                name: "LongList",
                namespace: None,
                doc: None,
                aliases: vec!["LinkedLongs"],
                fields: vec![
                    Field {
                        name: "value",
                        doc: None,
                        r#type: Schema::TypeName(TypeName::Primitive(PrimitiveType::Long)),
                        default: None,
                    },
                    Field {
                        name: "next",
                        doc: None,
                        r#type: Schema::Union(vec![
                            Schema::TypeName(TypeName::Primitive(PrimitiveType::Null)),
                            Schema::TypeName(TypeName::Ref("LongList")),
                        ]),
                        default: None,
                    }
                ],
                attributes: Attributes::default(),
            }))
        );

        // Recursive schema are not supported
        let err = AvroField::try_from(&schema).unwrap_err().to_string();
        assert_eq!(err, "Parser error: Failed to resolve .LongList");

        let schema: Schema = serde_json::from_str(
            r#"{
               "type":"record",
               "name":"topLevelRecord",
               "fields":[
                  {
                     "name":"id",
                     "type":[
                        "int",
                        "null"
                     ]
                  },
                  {
                     "name":"timestamp_col",
                     "type":[
                        {
                           "type":"long",
                           "logicalType":"timestamp-micros"
                        },
                        "null"
                     ]
                  }
               ]
            }"#,
        )
        .unwrap();

        assert_eq!(
            schema,
            Schema::Complex(ComplexType::Record(Record {
                name: "topLevelRecord",
                namespace: None,
                doc: None,
                aliases: vec![],
                fields: vec![
                    Field {
                        name: "id",
                        doc: None,
                        r#type: Schema::Union(vec![
                            Schema::TypeName(TypeName::Primitive(PrimitiveType::Int)),
                            Schema::TypeName(TypeName::Primitive(PrimitiveType::Null)),
                        ]),
                        default: None,
                    },
                    Field {
                        name: "timestamp_col",
                        doc: None,
                        r#type: Schema::Union(vec![
                            Schema::Type(timestamp),
                            Schema::TypeName(TypeName::Primitive(PrimitiveType::Null)),
                        ]),
                        default: None,
                    }
                ],
                attributes: Default::default(),
            }))
        );
        let codec = AvroField::try_from(&schema).unwrap();
        assert_eq!(
            codec.field(),
            arrow_schema::Field::new(
                "topLevelRecord",
                DataType::Struct(Fields::from(vec![
                    arrow_schema::Field::new("id", DataType::Int32, true),
                    arrow_schema::Field::new(
                        "timestamp_col",
                        DataType::Timestamp(TimeUnit::Microsecond, Some("+00:00".into())),
                        true
                    ),
                ])),
                false
            )
        );

        let schema: Schema = serde_json::from_str(
            r#"{
                  "type": "record",
                  "name": "HandshakeRequest", "namespace":"org.apache.avro.ipc",
                  "fields": [
                    {"name": "clientHash", "type": {"type": "fixed", "name": "MD5", "size": 16}},
                    {"name": "clientProtocol", "type": ["null", "string"]},
                    {"name": "serverHash", "type": "MD5"},
                    {"name": "meta", "type": ["null", {"type": "map", "values": "bytes"}]}
                  ]
            }"#,
        )
        .unwrap();

        assert_eq!(
            schema,
            Schema::Complex(ComplexType::Record(Record {
                name: "HandshakeRequest",
                namespace: Some("org.apache.avro.ipc"),
                doc: None,
                aliases: vec![],
                fields: vec![
                    Field {
                        name: "clientHash",
                        doc: None,
                        r#type: Schema::Complex(ComplexType::Fixed(Fixed {
                            name: "MD5",
                            namespace: None,
                            aliases: vec![],
                            size: 16,
                            attributes: Default::default(),
                        })),
                        default: None,
                    },
                    Field {
                        name: "clientProtocol",
                        doc: None,
                        r#type: Schema::Union(vec![
                            Schema::TypeName(TypeName::Primitive(PrimitiveType::Null)),
                            Schema::TypeName(TypeName::Primitive(PrimitiveType::String)),
                        ]),
                        default: None,
                    },
                    Field {
                        name: "serverHash",
                        doc: None,
                        r#type: Schema::TypeName(TypeName::Ref("MD5")),
                        default: None,
                    },
                    Field {
                        name: "meta",
                        doc: None,
                        r#type: Schema::Union(vec![
                            Schema::TypeName(TypeName::Primitive(PrimitiveType::Null)),
                            Schema::Complex(ComplexType::Map(Map {
                                values: Box::new(Schema::TypeName(TypeName::Primitive(
                                    PrimitiveType::Bytes
                                ))),
                                attributes: Default::default(),
                            })),
                        ]),
                        default: None,
                    }
                ],
                attributes: Default::default(),
            }))
        );
    }
}
