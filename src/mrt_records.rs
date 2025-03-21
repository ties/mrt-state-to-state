use bgpkit_parser::models::{AsPath, AsPathSegment, AttrRaw, AttrType};


pub fn handle_attributes(attr_val: Option<Vec<AttrRaw>>) -> Option<Vec<u8>> {
    match attr_val {
        Some(attrs) => {
            let series = attrs.iter().map(|attr| {
                match attr.attr_type {
                    AttrType::Unknown(id) => id,
                    _ => std::convert::Into::<u8>::into(attr.attr_type),
                }
            }).collect();

            Some(series)
        },
        None => None,
    }
}


pub fn format_as_path(path: AsPath) -> Vec<String> {
    let mut result = Vec::with_capacity(path.len());

    for segment in path.iter_segments() {
        match segment {
            AsPathSegment::AsSequence(as_seq) | AsPathSegment::ConfedSequence(as_seq)=> {
                result.extend(as_seq.iter().map(|asn| asn.to_string()));
            },
            AsPathSegment::AsSet(as_set) | AsPathSegment::ConfedSet(as_set) => {
                let set_segments = as_set.iter().map(|asn| asn.to_string()).collect::<Vec<String>>().join(" ");
                result.push(format!("{{{}}}", set_segments));
            },
        }
    }

    result
}
