// geometry.rs

pub mod geometry {
    /// Calculates the area of a circle given the radius.
    pub fn area_circle(radius: f64) -> f64 {
        std::f64::consts::PI * radius * radius
    }

    /// Calculates the circumference of a circle given the radius.
    pub fn circumference_circle(radius: f64) -> f64 {
        2.0 * std::f64::consts::PI * radius
    }

    /// Calculates the area of a rectangle given the width and height.
    pub fn area_rectangle(width: f64, height: f64) -> f64 {
        width * height
    }

    /// Calculates the perimeter of a rectangle given the width and height.
    pub fn perimeter_rectangle(width: f64, height: f64) -> f64 {
        2.0 * (width + height)
    }
}