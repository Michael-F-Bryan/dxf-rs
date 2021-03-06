// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

extern crate dxf;
use self::dxf::*;
use self::dxf::objects::*;
use self::dxf::enums::*;

mod test_helpers;
use test_helpers::helpers::*;

mod generated;
use generated::all_types;

fn read_object(object_type: &str, body: String) -> Object {
    let drawing = from_section("OBJECTS", vec!["0", object_type, body.as_str()].join("\r\n").as_str());
    assert_eq!(1, drawing.objects.len());
    drawing.objects[0].to_owned()
}

#[test]
fn read_empty_objects_section() {
    let drawing = parse_drawing(vec!["0", "SECTION", "2", "OBJECTS", "0", "ENDSEC", "0", "EOF"].join("\r\n").as_str());
    assert_eq!(0, drawing.objects.len());
}

#[test]
fn read_unsupported_object() {
    let drawing = parse_drawing(vec![
        "0", "SECTION",
            "2", "OBJECTS",
                "0", "UNSUPPORTED_OBJECT",
                    "1", "unsupported string",
        "0", "ENDSEC",
        "0", "EOF"].join("\r\n").as_str());
    assert_eq!(0, drawing.objects.len());
}

#[test]
fn read_unsupported_object_between_supported_objects() {
    let drawing = parse_drawing(vec![
        "0", "SECTION",
            "2", "OBJECTS",
                "0", "DICTIONARYVAR",
                "0", "UNSUPPORTED_OBJECT",
                    "1", "unsupported string",
                "0", "IMAGEDEF",
        "0", "ENDSEC",
        "0", "EOF"].join("\r\n").as_str());
    assert_eq!(2, drawing.objects.len());
    match drawing.objects[0].specific {
        ObjectType::DictionaryVariable(_) => (),
        _ => panic!("expected a dictionary variable"),
    }
    match drawing.objects[1].specific {
        ObjectType::ImageDefinition(_) => (),
        _ => panic!("expected an image definition"),
    }
}

#[test]
fn read_common_object_fields() {
    let obj = read_object("IMAGEDEF", vec!["5", "DEADBEEF"].join("\r\n"));
    assert_eq!(0xDEADBEEF, obj.common.handle);
}

#[test]
fn read_image_def() {
    let obj = read_object("IMAGEDEF", vec![
        "1", "path/to/file", // path
        "10", "11", // image_width
        "20", "22", // image_height
        ].join("\r\n"));
    match obj.specific {
        ObjectType::ImageDefinition(ref img) => {
            assert_eq!(11, img.image_width);
            assert_eq!(22, img.image_height);
        },
        _ => panic!("expected an image definition"),
    }
}

#[test]
fn write_common_object_fields() {
    let mut drawing = Drawing::default();
    drawing.header.version = AcadVersion::R14; // IMAGEDEF is only supported on R14+
    let obj = Object {
        common: Default::default(),
        specific: ObjectType::ImageDefinition(Default::default())
    };
    drawing.objects.push(obj);
    assert_contains(&drawing, vec![
        "  0", "IMAGEDEF",
        "  5", "1",
    ].join("\r\n"));
}

#[test]
fn write_specific_object_fields() {
    let mut drawing = Drawing::default();
    drawing.header.version = AcadVersion::R14; // IMAGEDEF is only supported on R14+
    let img = ImageDefinition {
        file_path: String::from("path/to/file"),
        .. Default::default()
    };
    drawing.objects.push(Object::new(ObjectType::ImageDefinition(img)));
    assert_contains(&drawing, vec![
        "100", "AcDbRasterImageDef",
        " 90", "        0",
        "  1", "path/to/file",
    ].join("\r\n"));
}

#[test]
fn read_multiple_objects() {
    let drawing = from_section("OBJECTS", vec![
        "0", "DICTIONARYVAR",
            "1", "value", // value
        "0", "IMAGEDEF",
            "1", "path/to/file", // file_path
            "10", "11", // image_width
            ].join("\r\n").as_str());
    assert_eq!(2, drawing.objects.len());

    // verify dictionary value
    match drawing.objects[0].specific {
        ObjectType::DictionaryVariable(ref var) => {
            assert_eq!("value", var.value);
        },
        _ => panic!("expected a dictionary variable"),
    }

    // verify image definition
    match drawing.objects[1].specific {
        ObjectType::ImageDefinition(ref img) => {
            assert_eq!("path/to/file", img.file_path);
            assert_eq!(11, img.image_width);
        },
        _ => panic!("expected an image definition"),
    }
}

#[test]
fn read_field_with_multiples_specific() {
    let obj = read_object("LAYER_FILTER", vec!["8", "one", "8", "two", "8", "three"].join("\r\n"));
    match obj.specific {
        ObjectType::LayerFilter(ref layer_filter) => {
            assert_eq!(vec!["one", "two", "three"], layer_filter.layer_names);
        },
        _ => panic!("expected a layer filter"),
    }
}

#[test]
fn write_field_with_multiples_specific() {
    let mut drawing = Drawing::default();
    drawing.header.version = AcadVersion::R2004; // LAYER_FILTER is only supported up to 2004
    drawing.objects.push(Object {
        common: Default::default(),
        specific: ObjectType::LayerFilter(LayerFilter {
            layer_names: vec![String::from("one"), String::from("two"), String::from("three")],
            .. Default::default()
        }),
    });
    assert_contains(&drawing, vec!["  8", "one", "  8", "two", "  8", "three"].join("\r\n"));
}

#[test]
fn read_object_with_post_parse() {
    let obj = read_object("VBA_PROJECT", vec![
        "310", "deadbeef", // data
        "310", "01234567",
    ].join("\r\n"));
    match obj.specific {
        ObjectType::VbaProject(ref vba) => {
            assert_eq!(8, vba.data.len());
            assert_eq!(vec![0xde, 0xad, 0xbe, 0xef, 0x01, 0x23, 0x45, 0x67], vba.data);
        },
        _ => panic!("expected a VBA_PROJECT"),
    }
}

#[test]
fn write_object_with_write_order() {
    let mut drawing = Drawing::default();
    drawing.header.version = AcadVersion::R2004; // LAYER_FILTER is only supported up to 2004
    drawing.objects.push(Object {
        common: Default::default(),
        specific: ObjectType::LayerFilter(LayerFilter {
            layer_names: vec![String::from("one"), String::from("two"), String::from("three")],
            .. Default::default()
        }),
    });
    assert_contains(&drawing, vec![
        "100", "AcDbFilter",
        "100", "AcDbLayerFilter",
        "  8", "one",
        "  8", "two",
        "  8", "three",
    ].join("\r\n"));
}

#[test]
fn read_object_with_flags() {
    let obj = read_object("LAYOUT", vec!["100", "AcDbLayout", "70", "3"].join("\r\n"));
    match obj.specific {
        ObjectType::Layout(ref layout) => {
            assert!(layout.get_is_ps_lt_scale());
            assert!(layout.get_is_lim_check());
        },
        _ => panic!("expected a LAYOUT"),
    }
}

#[test]
fn write_object_with_flags() {
    let mut drawing = Drawing::default();
    drawing.header.version = AcadVersion::R2000; // LAYOUT is only supported up to R2000
    let mut layout = Layout::default();
    assert_eq!(0, layout.layout_flags);
    layout.set_is_ps_lt_scale(true);
    layout.set_is_lim_check(true);
    layout.tab_order = -54;
    drawing.objects.push(Object {
        common: Default::default(),
        specific: ObjectType::Layout(layout),
    });
    assert_contains(&drawing, vec![
        " 70", "     3", // flags
        " 71", "   -54", // sentinel to make sure we're not reading a header value
    ].join("\r\n"));
}

#[test]
fn read_object_with_handles() {
    let obj = read_object("LIGHTLIST", vec![
        "5", "A1", // handle
        "330", "A2", // owner handle
    ].join("\r\n"));
    assert_eq!(0xa1, obj.common.handle);
    assert_eq!(0xa2, obj.common.__owner_handle);
    match obj.specific {
        ObjectType::LightList(_) => (),
        _ => panic!("expected a light list"),
    }
}

#[test]
fn write_object_with_handles() {
    let mut drawing = Drawing::default();
    drawing.header.version = AcadVersion::R2007; // LIGHTLIST only supported up to 2007
    drawing.objects.push(Object {
        common: ObjectCommon {
            handle: 0xa1,
            __owner_handle: 0xa2,
            .. Default::default()
        },
        specific: ObjectType::LightList(Default::default()),
    });
    assert_contains(&drawing, vec![
        "  0", "LIGHTLIST",
        "  5", "A1",
        "330", "A2",
    ].join("\r\n"));
}

#[test]
fn read_dictionary() {
    let dict = read_object("DICTIONARY", vec![
        "  3", "key1",
        "350", "AAAA",
        "  3", "key2",
        "350", "BBBB",
    ].join("\r\n"));
    match dict.specific {
        ObjectType::Dictionary(ref dict) => {
            assert_eq!(2, dict.value_handles.len());
            assert_eq!(Some(&0xAAAA), dict.value_handles.get("key1"));
            assert_eq!(Some(&0xBBBB), dict.value_handles.get("key2"));
        },
        _ => panic!("expected a dictionary"),
    }
}

#[test]
fn write_dictionary() {
    let mut dict = Dictionary::default();
    dict.value_handles.insert(String::from("key1"), 0xAAAA);
    dict.value_handles.insert(String::from("key2"), 0xBBBB);
    let mut drawing = Drawing::default();
    drawing.objects.push(Object {
        common: Default::default(),
        specific: ObjectType::Dictionary(dict),
    });
    assert_contains(&drawing, vec![
        "  3", "key1",
        "350", "AAAA",
        "  3", "key2",
        "350", "BBBB",
    ].join("\r\n"));
}

#[test]
fn read_sunstudy() {
    // validates that code 290 values (ideally boolean) can be read as integers, too
    let ss = read_object("SUNSTUDY", vec![
        "290", "1", // use_subset
        "290", "3", // hours
        "290", "4",
        "290", "5",
    ].join("\r\n"));
    match ss.specific {
        ObjectType::SunStudy(ref ss) => {
            assert!(ss.use_subset);
            assert_eq!(vec![3, 4, 5], ss.hours);
        },
        _ => panic!("expected a sunstudy"),
    }
}

#[test]
fn write_version_specific_object() {
    let mut drawing = Drawing::default();
    drawing.objects.push(Object {
        common: Default::default(),
        specific: ObjectType::AcadProxyObject(Default::default()),
    });

    // ACAD_PROXY_OBJECT not supported in R14 and below
    drawing.header.version = AcadVersion::R14;
    assert_contains(&drawing, vec![
        "  0", "SECTION",
        "  2", "OBJECTS",
        "  0", "ENDSEC",
    ].join("\r\n"));

    // but it is in R2000 and above
    drawing.header.version = AcadVersion::R2000;
    assert_contains(&drawing, vec![
        "  0", "SECTION",
        "  2", "OBJECTS",
        "  0", "ACAD_PROXY_OBJECT",
    ].join("\r\n"));
}

#[test]
fn read_extension_data() {
    let obj = read_object("IDBUFFER", vec![
        "102", "{IXMILIA",
        "  1", "some string",
        "102", "}",
    ].join("\r\n"));
    assert_eq!(1, obj.common.extension_data_groups.len());
    let group = &obj.common.extension_data_groups[0];
    assert_eq!("IXMILIA", group.application_name);
    match group.items[0] {
        ExtensionGroupItem::CodePair(ref p) => assert_eq!(&CodePair::new_str(1, "some string"), p),
        _ => panic!("expected a code pair"),
    }
}

#[test]
fn write_extension_data() {
    let drawing = Drawing {
        header: Header { version: AcadVersion::R14, .. Default::default() },
        objects: vec![
            Object {
                common: ObjectCommon {
                    extension_data_groups: vec![
                        ExtensionGroup {
                            application_name: String::from("IXMILIA"),
                            items: vec![
                                ExtensionGroupItem::CodePair(CodePair::new_str(1, "some string")),
                            ],
                        }
                    ],
                    .. Default::default()
                },
                specific: ObjectType::IdBuffer(IdBuffer::default()),
            }
        ],
        .. Default::default()
    };
    assert_contains(&drawing, vec![
        "102", "{IXMILIA",
        "  1", "some string",
        "102", "}",
    ].join("\r\n"));
}

#[test]
fn read_x_data() {
    let obj = read_object("IDBUFFER", vec![
        "1001", "IXMILIA",
        "1000", "some string",
    ].join("\r\n"));
    assert_eq!(1, obj.common.x_data.len());
    let x = &obj.common.x_data[0];
    assert_eq!("IXMILIA", x.application_name);
    match x.items[0] {
        XDataItem::Str(ref s) => assert_eq!("some string", s),
        _ => panic!("expected a string"),
    }
}

#[test]
fn write_x_data() {
    let drawing = Drawing {
        header: Header { version: AcadVersion::R2000, .. Default::default() },
        objects: vec![
            Object {
                common: ObjectCommon {
                    x_data: vec![
                        XData {
                            application_name: String::from("IXMILIA"),
                            items: vec![
                                XDataItem::Real(1.1),
                            ],
                        }
                    ],
                    .. Default::default()
                },
                specific: ObjectType::IdBuffer(IdBuffer::default()),
            }
        ],
        .. Default::default()
    };
    assert_contains(&drawing, vec![
        "1001", "IXMILIA",
        "1040", "1.1",
        "  0", "ENDSEC", // xdata is written after all the object's other code pairs
    ].join("\r\n"));
}

#[test]
fn read_all_types() {
    for (type_string, expected_type, _) in all_types::get_all_object_types() {
        println!("parsing {}", type_string);
        let obj = read_object(type_string, vec![
            "102", "{IXMILIA", // read extension data
            "  1", "some string",
            "102", "}",
            "1001", "IXMILIA", // read x data
            "1040", "1.1",
        ].join("\r\n"));

        // validate specific
        match (&expected_type, &obj.specific) {
            (&ObjectType::LayerIndex(ref a), &ObjectType::LayerIndex(ref b)) => {
                // LayerIndex has a timestamp that will obviously differ; the remaining fields must be checked manually
                assert_eq!(a.layer_names, b.layer_names);
                assert_eq!(a.__id_buffers_handle, b.__id_buffers_handle);
                assert_eq!(a.id_buffer_counts, b.id_buffer_counts);
            },
            (&ObjectType::SpatialIndex(_), &ObjectType::SpatialIndex(_)) => {
                // SpatialIndex has a timestamp that will obviously differ; there are no other fields
            }
            _ => assert_eq!(expected_type, obj.specific),
        }

        // validate extension data
        assert_eq!(1, obj.common.extension_data_groups.len());
        assert_eq!("IXMILIA", obj.common.extension_data_groups[0].application_name);
        assert_eq!(1, obj.common.extension_data_groups[0].items.len());
        assert_eq!(ExtensionGroupItem::CodePair(CodePair::new_str(1, "some string")), obj.common.extension_data_groups[0].items[0]);

        // validate x data
        assert_eq!(1, obj.common.x_data.len());
        assert_eq!("IXMILIA", obj.common.x_data[0].application_name);
        assert_eq!(1, obj.common.x_data[0].items.len());
        assert_eq!(XDataItem::Real(1.1), obj.common.x_data[0].items[0]);
    }
}

#[test]
fn write_all_types() {
    for (type_string, expected_type, max_version) in all_types::get_all_object_types() {
        println!("writing {}", type_string);
        let mut common = ObjectCommon::default();
        common.extension_data_groups.push(ExtensionGroup {
            application_name: String::from("IXMILIA"),
            items: vec![ExtensionGroupItem::CodePair(CodePair::new_str(1, "some string"))]
        });
        common.x_data.push(XData {
            application_name: String::from("IXMILIA"),
            items: vec![XDataItem::Real(1.1)],
        });
        let drawing = Drawing {
            objects: vec![Object { common: common, specific: expected_type }],
            header: Header { version: max_version, .. Default::default() },
            .. Default::default()
        };
        assert_contains(&drawing, vec![
            "  0", type_string,
        ].join("\r\n"));
        if max_version >= AcadVersion::R14 {
            // only written on R14+
            assert_contains(&drawing, vec![
                "102", "{IXMILIA",
                "  1", "some string",
                "102", "}",
            ].join("\r\n"));
        }
        if max_version >= AcadVersion::R2000 {
            // only written on R2000+
            assert_contains(&drawing, vec![
                "1001", "IXMILIA",
                "1040", "1.1",
            ].join("\r\n"));
        }
    }
}
