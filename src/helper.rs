pub fn ar_to_ms(ar: f32) -> i32 {
    let base = if ar >= 5.0 {
        450.0 + (10.0 - ar) * 150.0
    } else {
        1200.0 + (5.0 - ar) * 120.0
    };
    base as i32
}

pub fn cs_to_osupixels(cs: f32) -> f32 {
    54.4 - 4.48 * cs
}

#[test]
fn test_ar_to_ms() {
    assert_eq!(ar_to_ms(11.0), 300);
    assert_eq!(ar_to_ms(10.0), 450);
    assert_eq!(ar_to_ms(9.3), 555);
    assert_eq!(ar_to_ms(9.0), 600);
    assert_eq!(ar_to_ms(5.0), 1200);
    assert_eq!(ar_to_ms(4.0), 1320);
    assert_eq!(ar_to_ms(0.0), 1800);
}

#[test]
fn test_cs_to_osupixels() {
    assert_eq!(cs_to_osupixels(7.0), 23.04);
    assert_eq!(cs_to_osupixels(6.0), 27.52);
    assert_eq!(cs_to_osupixels(4.0), 36.480003); // floating point precision lol
}
