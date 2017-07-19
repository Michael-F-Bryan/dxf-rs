// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

extern crate dxf;
extern crate glob;
extern crate tempdir;
use self::dxf::*;
use self::dxf::entities::*;
use self::dxf::enums::*;
use self::glob::glob;
use self::tempdir::TempDir;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::Command;

fn test_teigha_read_ixmilia_generated_file<F>(generator: F)
    where F: Fn() -> Drawing {

    let mut drawing = generator();
    let all_versions = vec![
        AcadVersion::R9,
        AcadVersion::R10,
        AcadVersion::R11,
        AcadVersion::R12,
        AcadVersion::R13,
        AcadVersion::R14,
        AcadVersion::R2000,
        AcadVersion::R2004,
        AcadVersion::R2007,
        AcadVersion::R2010,
        AcadVersion::R2013,
    ];
    let input_dir = TempDir::new("dxf-rs-input").ok().unwrap();
    let output_dir = TempDir::new("dxf-rs-output").ok().unwrap();

    // create files for Teigha to read
    for version in all_versions {
        drawing.header.version = version;
        let output_path = input_dir.path().join(format!("file.{:?}.dxf", version));
        drawing.save_file(output_path.to_str().unwrap()).ok().unwrap();
    }

    // invoke the Teigha converter
    assert_teigha_convert(&input_dir.path(), &output_dir.path(), AcadVersion::R2010);
}

fn assert_teigha_convert(input_dir: &Path, output_dir: &Path, version: AcadVersion) {
    wait_for_process("C:\\Program Files (x86)\\ODA\\Teigha File Converter 4.02.2\\TeighaFileConverter.exe", generate_teigha_arguments(&input_dir, &output_dir, version));
    let mut error_text = String::new();
    for err_file in glob(output_dir.join("*.err").to_str().unwrap()).ok().unwrap() {
        let err_file = err_file.ok().unwrap();
        let mut file = File::open(err_file.to_str().unwrap()).ok().unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).ok().unwrap();
        error_text.push_str(&*format!("{}:\n{}", err_file.to_str().unwrap(), contents));
    }

    if !error_text.is_empty() {
        panic!("error using Teigha to convert files: {}", error_text);
    }
}

fn generate_teigha_arguments(input_dir: &Path, output_dir: &Path, version: AcadVersion) -> Vec<String> {
    let teigha_version = match version {
        AcadVersion::R2010 => "ACAD2010",
        _ => panic!("unsupported version for Teigha convert: {:?}", version),
    };

    vec![
        String::from(input_dir.to_str().unwrap()),
        String::from(output_dir.to_str().unwrap()),
        String::from(teigha_version),
        String::from("DXF"),
        String::from("0"), // recurse
        String::from("1"), // audit
    ]
}

fn wait_for_process(command: &str, args: Vec<String>) {
    let status = Command::new(command).args(args).status().ok().unwrap();
    assert!(status.success());
}

#[test]
fn read_line_with_teigha() {
    test_teigha_read_ixmilia_generated_file(|| {
        Drawing {
            entities: vec![
                Entity::new(EntityType::Line(Line {
                        p1: Point::new(0.0, 0.0, 0.0),
                        p2: Point::new(10.0, 10.0, 0.0),
                        .. Default::default()
                    }),
                )
            ],
            .. Default::default()
        }
    });
}
