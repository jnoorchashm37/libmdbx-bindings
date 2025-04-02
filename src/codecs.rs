#[cfg(not(feature = "derive"))]
#[macro_export]
macro_rules! table_value_codecs_with_zc {
    ($table_value:ident) => {
        impl alloy_rlp::Encodable for $table_value {
            fn encode(&self, out: &mut dyn alloy_primitives::bytes::BufMut) {
                let encoded = rkyv::to_bytes::<_, 256>(self).unwrap();

                out.put_slice(&encoded)
            }
        }

        impl alloy_rlp::Decodable for $table_value {
            fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
                let archived: &paste::paste!([<Archived $table_value>]) =
                unsafe { rkyv::archived_root::<Self>(&buf[..]) };


                let this = rkyv::Deserialize::deserialize(archived, &mut rkyv::Infallible).unwrap();

                Ok(this)
            }
        }

        impl reth_db_api::table::Compress for $table_value {
            type Compressed = Vec<u8>;

            fn compress_to_buf<B: alloy_primitives::bytes::BufMut + AsMut<[u8]>>(&self, buf: &mut B) {
                let mut encoded = Vec::new();
                alloy_rlp::Encodable::encode(&self, &mut encoded);
                let encoded_compressed = zstd::encode_all(&*encoded, 0).unwrap();

                buf.put_slice(&encoded_compressed);
            }
        }

        impl reth_db_api::table::Decompress for $table_value {
            fn decompress(value: &[u8]) -> Result<Self, reth_storage_errors::db::DatabaseError> {
                let binding = value.to_vec();

                let encoded_decompressed = zstd::decode_all(&*binding).unwrap();
                let buf = &mut encoded_decompressed.as_slice();

                alloy_rlp::Decodable::decode(buf).map_err(|_| reth_storage_errors::db::DatabaseError::Decode)
            }
        }
    };


    ($table_value:ident, $wrapper_table_value:ident) => {
        table_value_codecs_with_zc!($wrapper_table_value);

        impl alloy_rlp::Encodable for $table_value {
            fn encode(&self, out: &mut dyn alloy_primitives::bytes::BufMut) {
                let this: $wrapper_table_value = self.clone().into();
                alloy_rlp::Encodable::encode(&this, out)
            }
        }

        impl alloy_rlp::Decodable for $table_value {
            fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
                alloy_rlp::Decodable::decode(buf).map(|v: $wrapper_table_value| v.into())
            }
        }

        impl reth_db_api::table::Compress for $table_value {
            type Compressed = Vec<u8>;

            fn compress_to_buf<B: alloy_primitives::bytes::BufMut + AsMut<[u8]>>(self, buf: &mut B) {
                let this: $wrapper_table_value = self.into();
                rkyv::Compress::compress_to_buf(this, buf)
            }
        }

        impl reth_db_api::table::Decompress for $table_value {
            fn decompress(value: &[u8]) -> Result<Self, reth_storage_errors::db::DatabaseError> {
                rkyv::Decompress::decompress(value).map(|v: $wrapper_table_value| v.into())
            }
        }
    };

}

#[cfg(not(feature = "derive"))]
#[macro_export]
macro_rules! table_key_codecs_with_zc {
    ($table_value:ident) => {
        crate::table_value_codecs_with_zc!($table_value);

        impl reth_db_api::table::Encode for $table_value {
            type Encoded = Vec<u8>;

            fn encode(self) -> Self::Encoded {
                let mut buf = bytes::BytesMut::new();
                alloy_rlp::Encodable::encode(&self, &mut buf);
                buf.to_vec()
            }
        }

        impl reth_db_api::table::Decode for $table_value {
            fn decode(mut value: &[u8]) -> Result<Self, reth_storage_errors::db::DatabaseError> {
                alloy_rlp::Decodable::decode(&mut value)
                    .map_err(|_| reth_storage_errors::db::DatabaseError::Decode)
            }
        }
    };
}

#[cfg(feature = "derive")]
#[macro_export]
macro_rules! table_value_codecs_with_zc {
    ($table_value:ident) => {
        impl libmdbx_bindings::Encodable for $table_value {
            fn encode(&self, out: &mut dyn libmdbx_bindings::BufMut) {
                let encoded = libmdbx_bindings::to_bytes::<_, 256>(self).unwrap();

                out.put_slice(&encoded)
            }
        }

        impl libmdbx_bindings::Decodable for $table_value {
            fn decode(buf: &mut &[u8]) -> libmdbx_bindings::RlpResult<Self> {
                let archived: &$crate::paste!([<Archived $table_value>]) =
                unsafe { libmdbx_bindings::archived_root::<Self>(&buf[..]) };


                let this = libmdbx_bindings::re_export_rkyv::Deserialize::deserialize(archived, &mut libmdbx_bindings::Infallible).unwrap();

                Ok(this)
            }
        }

        impl libmdbx_bindings::Compress for $table_value {
            type Compressed = Vec<u8>;

            fn compress_to_buf<B: libmdbx_bindings::AlloyBytesMut + AsMut<[u8]>>(&self, buf: &mut B) {
                let mut encoded = Vec::new();
                libmdbx_bindings::Encodable::encode(&self, &mut encoded);
                let encoded_compressed = libmdbx_bindings::encode_all(&*encoded, 0).unwrap();

                buf.put_slice(&encoded_compressed);
            }
        }

        impl libmdbx_bindings::Decompress for $table_value {
            fn decompress(value: &[u8]) -> Result<Self, libmdbx_bindings::DatabaseError> {
                let binding = value.to_vec();

                let encoded_decompressed = libmdbx_bindings::decode_all(&*binding).unwrap();
                let buf = &mut encoded_decompressed.as_slice();

                libmdbx_bindings::Decodable::decode(buf).map_err(|_| libmdbx_bindings::DatabaseError::Decode)
            }
        }
    };


    ($table_value:ident, $wrapper_table_value:ident) => {
        table_value_codecs_with_zc!($wrapper_table_value);

        impl libmdbx_bindings::Encodable for $table_value {
            fn encode(&self, out: &mut dyn libmdbx_bindings::BufMut) {
                let this: $wrapper_table_value = self.clone().into();
                libmdbx_bindings::Encodable::encode(&this, out)
            }
        }

        impl libmdbx_bindings::Decodable for $table_value {
            fn decode(buf: &mut &[u8]) -> libmdbx_bindings::RlpResult<Self> {
                libmdbx_bindings::Decodable::decode(buf).map(|v: $wrapper_table_value| v.into())
            }
        }

        impl libmdbx_bindings::Compress for $table_value {
            type Compressed = Vec<u8>;

            fn compress_to_buf<B: libmdbx_bindings::AlloyBytesMut + AsMut<[u8]>>(self, buf: &mut B) {
                let this: $wrapper_table_value = self.into();
                libmdbx_bindings::Compress::compress_to_buf(this, buf)
            }
        }

        impl libmdbx_bindings::Decompress for $table_value {
            fn decompress(value: &[u8]) -> Result<Self, libmdbx_bindings::DatabaseError> {
                libmdbx_bindings::Decompress::decompress(value).map(|v: $wrapper_table_value| v.into())
            }
        }
    };

}

#[cfg(feature = "derive")]
#[macro_export]
macro_rules! table_key_codecs_with_zc {
    ($table_value:ident) => {
        crate::table_value_codecs_with_zc!($table_value);

        impl libmdbx_bindings::WrapEncode for $table_value {}
        impl libmdbx_bindings::WrapDecode for $table_value {}

        impl libmdbx_bindings::WrapEncodable for $table_value {}
        impl libmdbx_bindings::WrapDecodable for $table_value {}

        impl libmdbx_bindings::Encode for $table_value {
            type Encoded = Vec<u8>;

            fn encode(self) -> Self::Encoded {
                let mut buf = bytes::BytesMut::new();
                libmdbx_bindings::Encodable::encode(&self, &mut buf);
                buf.to_vec()
            }
        }

        impl libmdbx_bindings::Decode for $table_value {
            fn decode(mut value: &[u8]) -> Result<Self, libmdbx_bindings::DatabaseError> {
                libmdbx_bindings::Decodable::decode(&mut value)
                    .map_err(|_| libmdbx_bindings::DatabaseError::Decode)
            }
        }
    };
}
