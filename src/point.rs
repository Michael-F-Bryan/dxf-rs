// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

use ::{
    CodePair,
    DxfError,
    DxfResult,
};

/// Represents a simple point in Cartesian space.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Point {
    /// The X value of the point.
    pub x: f64,
    /// The Y value of the point.
    pub y: f64,
    /// The Z value of the point.
    pub z: f64,
}

impl Point {
    /// Creates a new `Point` with the specified values.
    pub fn new(x: f64, y: f64, z: f64) -> Point {
        Point{
            x: x,
            y: y,
            z: z,
        }
    }
    /// Returns a point representing the origin of (0, 0, 0).
    pub fn origin() -> Point {
        Point::new(0.0, 0.0, 0.0)
    }
    pub(crate) fn set(&mut self, pair: &CodePair) -> DxfResult<()> {
        match pair.code {
            10 => self.x = pair.value.assert_f64()?,
            20 => self.y = pair.value.assert_f64()?,
            30 => self.z = pair.value.assert_f64()?,
            _ => return Err(DxfError::UnexpectedCodePair(pair.clone(), String::from("expected code [10, 20, 30] for point"))),
        }

        Ok(())
    }
}
