#[test]
pub fn test_int_converts() {
    use crate::utils::types::{i32_to_i64, i64_to_i32};
    assert_eq!(0, i32_to_i64(i64_to_i32(0)));
    assert_eq!(
        0x7FFFFFFFFFFFFFFF,
        i32_to_i64(i64_to_i32(0x7FFFFFFFFFFFFFFF))
    );
    assert_eq!(-1, i32_to_i64(i64_to_i32(-1)));
    assert_eq!(
        -0x7FFFFFFFFFFFFFFF,
        i32_to_i64(i64_to_i32(-0x7FFFFFFFFFFFFFFF))
    );
    assert_eq!(564456456465, i32_to_i64(i64_to_i32(564456456465)));
    assert_eq!(-95135745, i32_to_i64(i64_to_i32(-95135745)));
}
