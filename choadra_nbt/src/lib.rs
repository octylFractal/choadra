#![deny(warnings)]

pub mod modified_utf8;
pub mod tag;

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::tag::tag::NbtTag;
    use flate2::bufread::GzDecoder;
    use std::io::Read;

    #[test]
    fn test_hello_world() {
        let hello_world = include_bytes!("resources/hello_world.nbt");
        let deserialized = NbtTag::deserialize_from_root(&mut Cursor::new(hello_world))
            .unwrap_or_else(|e| panic!("Failed to deserialize hello_world: {:#?}", e));
        assert_eq!(
            NbtTag::new_compound_ref(vec![(
                "hello world",
                NbtTag::new_compound_ref(vec![("name", NbtTag::String("Bananrama".to_string()))])
            )]),
            deserialized
        )
    }

    #[test]
    fn test_big_test() {
        let bigtest = include_bytes!("resources/bigtest.nbt");

        let mut gzip_reader = GzDecoder::new(&bigtest[..]);
        let mut decompressed = Vec::new();
        gzip_reader.read_to_end(&mut decompressed).unwrap();

        NbtTag::deserialize_from_root(&mut Cursor::new(decompressed))
            .unwrap_or_else(|e| panic!("Failed to deserialize bigtest: {:#?}", e));
    }
}
