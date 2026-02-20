use lopdf::{Document, Object};

fn decode_pdf_string(bytes: &[u8]) -> String {
    if bytes.starts_with(&[0xFE, 0xFF]) {
        let utf16: Vec<u16> = bytes[2..]
            .chunks(2)
            .filter_map(|c| (c.len() == 2).then(|| (c[0] as u16) << 8 | c[1] as u16))
            .collect();
        String::from_utf16_lossy(&utf16)
    } else {
        String::from_utf8_lossy(bytes).into_owned()
    }
}

pub fn get_acroform_field(doc: &Document, field_name: &str) -> Option<String> {
    let catalog = doc.catalog().ok()?;
    let acro_ref = catalog.get(b"AcroForm").ok()?;
    let (_, acro_obj) = doc.dereference(acro_ref).ok()?;
    let acro_dict = acro_obj.as_dict().ok()?;

    let fields_ref = acro_dict.get(b"Fields").ok()?;
    let (_, fields_obj) = doc.dereference(fields_ref).ok()?;
    let fields = fields_obj.as_array().ok()?;

    fields.iter().find_map(|entry| {
        let (_, field_obj) = doc.dereference(entry).ok()?;
        let dict = field_obj.as_dict().ok()?;
        let name = match dict.get(b"T").ok()? {
            Object::String(bytes, _) => decode_pdf_string(bytes),
            _ => return None,
        };
        if name != field_name {
            return None;
        }
        match dict.get(b"V").ok()? {
            Object::String(bytes, _) => Some(decode_pdf_string(bytes)),
            _ => None,
        }
    })
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use lopdf::Document;

    use super::get_acroform_field;

    #[test]
    fn test_med_name_field_value() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/res/PathologyResult-Example.pdf");
        let doc = Document::load(path).expect("failed to load PDF");
        assert_eq!(
            get_acroform_field(&doc, "MedName").as_deref(),
            Some("Estradiol Enanthate")
        );
    }
}
