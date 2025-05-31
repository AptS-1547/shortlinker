pub mod colors;

pub fn generate_random_code(length: usize) -> String {
    use std::iter;

    // 随机选择字母和数字
    let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

    // 生成指定长度的随机字符串
    iter::repeat_with(|| chars[rand::random_range(0..chars.len())] as char)
        .take(length)
        .collect()
}
