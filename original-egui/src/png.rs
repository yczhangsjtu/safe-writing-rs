use image::EncodableLayout;

pub fn read_png_metadata(data: &Vec<u8>) -> Option<String> {
    let mut decoder = png::Decoder::new(data.as_bytes());
    decoder.set_transformations(png::Transformations::EXPAND);
    let reader = decoder.read_info();
    if let Ok(reader) = reader {
        reader
            .info()
            .utf8_text
            .iter()
            .map(|text_chunk| text_chunk.get_text())
            .chain(
                reader
                    .info()
                    .compressed_latin1_text
                    .iter()
                    .map(|text_chunk| text_chunk.get_text()),
            )
            .chain(
                reader
                    .info()
                    .uncompressed_latin1_text
                    .iter()
                    .map(|text_chunk| Ok(text_chunk.text.clone())),
            )
            .collect::<Result<Vec<_>, png::DecodingError>>()
            .ok()
            .map(|v| v.join("\n"))
    } else {
        return None;
    }
}
