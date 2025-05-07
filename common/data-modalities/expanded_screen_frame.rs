#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
pub mod screen {
    pub mod data {
        use lifelog_core::*;
        use lifelog_macros::lifelog_type;
        use lifelog_proto;
        use rand::distr::{Alphanumeric, Distribution, StandardUniform};
        use rand::{thread_rng, Rng};
        use serde::{Deserialize, Serialize};
        pub struct ScreenFrame {
            pub uuid: ::lifelog_core::uuid::Uuid,
            pub timestamp: ::lifelog_core::chrono::DateTime<::lifelog_core::chrono::Utc>,
            pub width: u32,
            pub height: u32,
            pub image: Vec<u8>,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for ScreenFrame {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field5_finish(
                    f,
                    "ScreenFrame",
                    "uuid",
                    &self.uuid,
                    "timestamp",
                    &self.timestamp,
                    "width",
                    &self.width,
                    "height",
                    &self.height,
                    "image",
                    &&self.image,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for ScreenFrame {
            #[inline]
            fn clone(&self) -> ScreenFrame {
                ScreenFrame {
                    uuid: ::core::clone::Clone::clone(&self.uuid),
                    timestamp: ::core::clone::Clone::clone(&self.timestamp),
                    width: ::core::clone::Clone::clone(&self.width),
                    height: ::core::clone::Clone::clone(&self.height),
                    image: ::core::clone::Clone::clone(&self.image),
                }
            }
        }
        #[doc(hidden)]
        #[allow(
            non_upper_case_globals,
            unused_attributes,
            unused_qualifications,
            clippy::absolute_paths,
        )]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl _serde::Serialize for ScreenFrame {
                fn serialize<__S>(
                    &self,
                    __serializer: __S,
                ) -> _serde::__private::Result<__S::Ok, __S::Error>
                where
                    __S: _serde::Serializer,
                {
                    let mut __serde_state = _serde::Serializer::serialize_struct(
                        __serializer,
                        "ScreenFrame",
                        false as usize + 1 + 1 + 1 + 1 + 1,
                    )?;
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "uuid",
                        &self.uuid,
                    )?;
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "timestamp",
                        &self.timestamp,
                    )?;
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "width",
                        &self.width,
                    )?;
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "height",
                        &self.height,
                    )?;
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "image",
                        &self.image,
                    )?;
                    _serde::ser::SerializeStruct::end(__serde_state)
                }
            }
        };
        #[doc(hidden)]
        #[allow(
            non_upper_case_globals,
            unused_attributes,
            unused_qualifications,
            clippy::absolute_paths,
        )]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for ScreenFrame {
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    #[allow(non_camel_case_types)]
                    #[doc(hidden)]
                    enum __Field {
                        __field0,
                        __field1,
                        __field2,
                        __field3,
                        __field4,
                        __ignore,
                    }
                    #[doc(hidden)]
                    struct __FieldVisitor;
                    #[automatically_derived]
                    impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                        type Value = __Field;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private::Formatter,
                        ) -> _serde::__private::fmt::Result {
                            _serde::__private::Formatter::write_str(
                                __formatter,
                                "field identifier",
                            )
                        }
                        fn visit_u64<__E>(
                            self,
                            __value: u64,
                        ) -> _serde::__private::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                0u64 => _serde::__private::Ok(__Field::__field0),
                                1u64 => _serde::__private::Ok(__Field::__field1),
                                2u64 => _serde::__private::Ok(__Field::__field2),
                                3u64 => _serde::__private::Ok(__Field::__field3),
                                4u64 => _serde::__private::Ok(__Field::__field4),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_str<__E>(
                            self,
                            __value: &str,
                        ) -> _serde::__private::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                "uuid" => _serde::__private::Ok(__Field::__field0),
                                "timestamp" => _serde::__private::Ok(__Field::__field1),
                                "width" => _serde::__private::Ok(__Field::__field2),
                                "height" => _serde::__private::Ok(__Field::__field3),
                                "image" => _serde::__private::Ok(__Field::__field4),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_bytes<__E>(
                            self,
                            __value: &[u8],
                        ) -> _serde::__private::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                b"uuid" => _serde::__private::Ok(__Field::__field0),
                                b"timestamp" => _serde::__private::Ok(__Field::__field1),
                                b"width" => _serde::__private::Ok(__Field::__field2),
                                b"height" => _serde::__private::Ok(__Field::__field3),
                                b"image" => _serde::__private::Ok(__Field::__field4),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                    }
                    #[automatically_derived]
                    impl<'de> _serde::Deserialize<'de> for __Field {
                        #[inline]
                        fn deserialize<__D>(
                            __deserializer: __D,
                        ) -> _serde::__private::Result<Self, __D::Error>
                        where
                            __D: _serde::Deserializer<'de>,
                        {
                            _serde::Deserializer::deserialize_identifier(
                                __deserializer,
                                __FieldVisitor,
                            )
                        }
                    }
                    #[doc(hidden)]
                    struct __Visitor<'de> {
                        marker: _serde::__private::PhantomData<ScreenFrame>,
                        lifetime: _serde::__private::PhantomData<&'de ()>,
                    }
                    #[automatically_derived]
                    impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                        type Value = ScreenFrame;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private::Formatter,
                        ) -> _serde::__private::fmt::Result {
                            _serde::__private::Formatter::write_str(
                                __formatter,
                                "struct ScreenFrame",
                            )
                        }
                        #[inline]
                        fn visit_seq<__A>(
                            self,
                            mut __seq: __A,
                        ) -> _serde::__private::Result<Self::Value, __A::Error>
                        where
                            __A: _serde::de::SeqAccess<'de>,
                        {
                            let __field0 = match _serde::de::SeqAccess::next_element::<
                                ::lifelog_core::uuid::Uuid,
                            >(&mut __seq)? {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            0usize,
                                            &"struct ScreenFrame with 5 elements",
                                        ),
                                    );
                                }
                            };
                            let __field1 = match _serde::de::SeqAccess::next_element::<
                                ::lifelog_core::chrono::DateTime<
                                    ::lifelog_core::chrono::Utc,
                                >,
                            >(&mut __seq)? {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            1usize,
                                            &"struct ScreenFrame with 5 elements",
                                        ),
                                    );
                                }
                            };
                            let __field2 = match _serde::de::SeqAccess::next_element::<
                                u32,
                            >(&mut __seq)? {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            2usize,
                                            &"struct ScreenFrame with 5 elements",
                                        ),
                                    );
                                }
                            };
                            let __field3 = match _serde::de::SeqAccess::next_element::<
                                u32,
                            >(&mut __seq)? {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            3usize,
                                            &"struct ScreenFrame with 5 elements",
                                        ),
                                    );
                                }
                            };
                            let __field4 = match _serde::de::SeqAccess::next_element::<
                                Vec<u8>,
                            >(&mut __seq)? {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            4usize,
                                            &"struct ScreenFrame with 5 elements",
                                        ),
                                    );
                                }
                            };
                            _serde::__private::Ok(ScreenFrame {
                                uuid: __field0,
                                timestamp: __field1,
                                width: __field2,
                                height: __field3,
                                image: __field4,
                            })
                        }
                        #[inline]
                        fn visit_map<__A>(
                            self,
                            mut __map: __A,
                        ) -> _serde::__private::Result<Self::Value, __A::Error>
                        where
                            __A: _serde::de::MapAccess<'de>,
                        {
                            let mut __field0: _serde::__private::Option<
                                ::lifelog_core::uuid::Uuid,
                            > = _serde::__private::None;
                            let mut __field1: _serde::__private::Option<
                                ::lifelog_core::chrono::DateTime<
                                    ::lifelog_core::chrono::Utc,
                                >,
                            > = _serde::__private::None;
                            let mut __field2: _serde::__private::Option<u32> = _serde::__private::None;
                            let mut __field3: _serde::__private::Option<u32> = _serde::__private::None;
                            let mut __field4: _serde::__private::Option<Vec<u8>> = _serde::__private::None;
                            while let _serde::__private::Some(__key) = _serde::de::MapAccess::next_key::<
                                __Field,
                            >(&mut __map)? {
                                match __key {
                                    __Field::__field0 => {
                                        if _serde::__private::Option::is_some(&__field0) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field("uuid"),
                                            );
                                        }
                                        __field0 = _serde::__private::Some(
                                            _serde::de::MapAccess::next_value::<
                                                ::lifelog_core::uuid::Uuid,
                                            >(&mut __map)?,
                                        );
                                    }
                                    __Field::__field1 => {
                                        if _serde::__private::Option::is_some(&__field1) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "timestamp",
                                                ),
                                            );
                                        }
                                        __field1 = _serde::__private::Some(
                                            _serde::de::MapAccess::next_value::<
                                                ::lifelog_core::chrono::DateTime<
                                                    ::lifelog_core::chrono::Utc,
                                                >,
                                            >(&mut __map)?,
                                        );
                                    }
                                    __Field::__field2 => {
                                        if _serde::__private::Option::is_some(&__field2) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field("width"),
                                            );
                                        }
                                        __field2 = _serde::__private::Some(
                                            _serde::de::MapAccess::next_value::<u32>(&mut __map)?,
                                        );
                                    }
                                    __Field::__field3 => {
                                        if _serde::__private::Option::is_some(&__field3) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field("height"),
                                            );
                                        }
                                        __field3 = _serde::__private::Some(
                                            _serde::de::MapAccess::next_value::<u32>(&mut __map)?,
                                        );
                                    }
                                    __Field::__field4 => {
                                        if _serde::__private::Option::is_some(&__field4) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field("image"),
                                            );
                                        }
                                        __field4 = _serde::__private::Some(
                                            _serde::de::MapAccess::next_value::<Vec<u8>>(&mut __map)?,
                                        );
                                    }
                                    _ => {
                                        let _ = _serde::de::MapAccess::next_value::<
                                            _serde::de::IgnoredAny,
                                        >(&mut __map)?;
                                    }
                                }
                            }
                            let __field0 = match __field0 {
                                _serde::__private::Some(__field0) => __field0,
                                _serde::__private::None => {
                                    _serde::__private::de::missing_field("uuid")?
                                }
                            };
                            let __field1 = match __field1 {
                                _serde::__private::Some(__field1) => __field1,
                                _serde::__private::None => {
                                    _serde::__private::de::missing_field("timestamp")?
                                }
                            };
                            let __field2 = match __field2 {
                                _serde::__private::Some(__field2) => __field2,
                                _serde::__private::None => {
                                    _serde::__private::de::missing_field("width")?
                                }
                            };
                            let __field3 = match __field3 {
                                _serde::__private::Some(__field3) => __field3,
                                _serde::__private::None => {
                                    _serde::__private::de::missing_field("height")?
                                }
                            };
                            let __field4 = match __field4 {
                                _serde::__private::Some(__field4) => __field4,
                                _serde::__private::None => {
                                    _serde::__private::de::missing_field("image")?
                                }
                            };
                            _serde::__private::Ok(ScreenFrame {
                                uuid: __field0,
                                timestamp: __field1,
                                width: __field2,
                                height: __field3,
                                image: __field4,
                            })
                        }
                    }
                    #[doc(hidden)]
                    const FIELDS: &'static [&'static str] = &[
                        "uuid",
                        "timestamp",
                        "width",
                        "height",
                        "image",
                    ];
                    _serde::Deserializer::deserialize_struct(
                        __deserializer,
                        "ScreenFrame",
                        FIELDS,
                        __Visitor {
                            marker: _serde::__private::PhantomData::<ScreenFrame>,
                            lifetime: _serde::__private::PhantomData,
                        },
                    )
                }
            }
        };
        impl ::lifelog_core::DataType for ScreenFrame {
            fn uuid(&self) -> ::lifelog_core::uuid::Uuid {
                self.uuid
            }
            fn timestamp(
                &self,
            ) -> ::lifelog_core::chrono::DateTime<::lifelog_core::chrono::Utc> {
                self.timestamp
            }
        }
        impl From<lifelog_proto::ScreenFrame> for ScreenFrame {
            fn from(p: lifelog_proto::ScreenFrame) -> Self {
                ScreenFrame {
                    uuid: ::lifelog_core::uuid::Uuid::parse_str(&p.uuid)
                        .expect("invalid uuid"),
                    timestamp: {
                        let ts = p.timestamp.unwrap_or_default();
                        ::lifelog_core::chrono::DateTime::<
                            ::lifelog_core::chrono::Utc,
                        >::from_utc(
                            ::lifelog_core::chrono::NaiveDateTime::from_timestamp(
                                ts.seconds,
                                ts.nanos as u32,
                            ),
                            ::lifelog_core::chrono::Utc,
                        )
                    },
                    width: p.width.into(),
                    height: p.height.into(),
                    image: p.image.to_vec(),
                }
            }
        }
        impl From<ScreenFrame> for lifelog_proto::ScreenFrame {
            fn from(s: ScreenFrame) -> Self {
                lifelog_proto::ScreenFrame {
                    uuid: s.uuid.to_string(),
                    timestamp: Some(::prost_types::Timestamp {
                        seconds: s.timestamp.timestamp(),
                        nanos: s.timestamp.timestamp_subsec_nanos() as i32,
                    }),
                    width: s.width.into(),
                    height: s.height.into(),
                    image: s.image.iter().map(|e| e.to_string()).collect(),
                }
            }
        }
        impl Modality for ScreenFrame {
            const TABLE: &'static str = "screen";
            fn into_payload(self) -> lifelog_proto::lifelog_data::Payload {
                lifelog_proto::lifelog_data::Payload::Screenframe(self.into())
            }
            fn id(&self) -> String {
                self.uuid.to_string()
            }
        }
        impl Distribution<ScreenFrame> for StandardUniform {
            fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ScreenFrame {
                let image_path: String = rng
                    .sample_iter(&Alphanumeric)
                    .take(10)
                    .map(char::from)
                    .collect();
                let width = rng.random_range(640..1920);
                let height = rng.random_range(480..1080);
                let uuid = Uuid::new_v4();
                let timestamp = Utc::now();
                let image = ::alloc::vec::from_elem(0, (width * height) as usize);
                ScreenFrame {
                    uuid,
                    timestamp,
                    width,
                    height,
                    image,
                }
            }
        }
    }
    pub use data::*;
}
