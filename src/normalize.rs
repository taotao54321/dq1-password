use itertools::Itertools as _;

use crate::decode::password_char_to_cum;
use crate::encode::pack_hero_name_char;
use crate::error::{Dq1PasswordError, Dq1PasswordResult};

/// 主人公の名前を正規化する。
///
/// 濁点/半濁点の分離などを行い、4 文字に満たない場合 ASCII space でパディングする。
///
/// `hero_name` が無効な場合、`Err(Dq1PasswordError::InvalidGameState)` を返す。
pub fn normalize_hero_name(hero_name: impl AsRef<str>) -> Dq1PasswordResult<String> {
    // validation もこちらで行う。

    let cs: Vec<_> = hero_name
        .as_ref()
        .chars()
        .flat_map(normalize_hero_name_char)
        .take(4 + 1)
        .collect();

    if cs.len() > 4 {
        return Err(Dq1PasswordError::invalid_game_state(
            "主人公の名前は 4 文字以内でなければならない(濁点、半濁点は 1 文字と数える)",
        ));
    }

    let cs_invalid: Vec<_> = cs
        .iter()
        .filter(|&&c| pack_hero_name_char(c).is_none())
        .collect();

    if !cs_invalid.is_empty() {
        return Err(Dq1PasswordError::invalid_game_state(format!(
            "主人公の名前に無効な文字が含まれている: {}",
            cs_invalid
                .into_iter()
                .map(|c| format!("'{}'", c))
                .join(", ")
        )));
    }

    Ok(cs
        .into_iter()
        .chain(std::iter::repeat(' '))
        .take(4)
        .collect())
}

fn normalize_hero_name_char(c: char) -> impl Iterator<Item = char> {
    const MAP: phf::Map<char, &[char]> = phf::phf_map! {
        // 数字
        '０' => &['0'],
        '１' => &['1'],
        '２' => &['2'],
        '３' => &['3'],
        '４' => &['4'],
        '５' => &['5'],
        '６' => &['6'],
        '７' => &['7'],
        '８' => &['8'],
        '９' => &['9'],
        // 濁点/半濁点
        'が' => &['か', '゛'],
        'ぎ' => &['き', '゛'],
        'ぐ' => &['く', '゛'],
        'げ' => &['け', '゛'],
        'ご' => &['こ', '゛'],
        'ざ' => &['さ', '゛'],
        'じ' => &['し', '゛'],
        'ず' => &['す', '゛'],
        'ぜ' => &['せ', '゛'],
        'ぞ' => &['そ', '゛'],
        'だ' => &['た', '゛'],
        'ぢ' => &['ち', '゛'],
        'づ' => &['つ', '゛'],
        'で' => &['て', '゛'],
        'ど' => &['と', '゛'],
        'ば' => &['は', '゛'],
        'び' => &['ひ', '゛'],
        'ぶ' => &['ふ', '゛'],
        'べ' => &['へ', '゛'],
        'ぼ' => &['ほ', '゛'],
        'ぱ' => &['は', '゜'],
        'ぴ' => &['ひ', '゜'],
        'ぷ' => &['ふ', '゜'],
        'ぺ' => &['へ', '゜'],
        'ぽ' => &['ほ', '゜'],
        '\u{3094}' => &['う', '゛'], // 「う」に濁点
        '\u{3099}' => &['゛'], // 結合文字用濁点
        '\u{309A}' => &['゜'], // 結合文字用半濁点
        // ハイフン
        '\u{2010}' => &['-'], // hyphen
        '\u{2011}' => &['-'], // non-breaking hyphen
        '\u{2012}' => &['-'], // figure dash
        '\u{2013}' => &['-'], // en dash
        '\u{2014}' => &['-'], // em dash
        '\u{2015}' => &['-'], // horizontal bar
        '\u{2212}' => &['-'], // minus sign
        '\u{30FC}' => &['-'], // 全角長音
        '\u{FF70}' => &['-'], // 半角長音
        // 空白
        '\u{3000}' => &[' '], // 全角空白
    };

    MAP.get(&c).map_or_else(
        || itertools::Either::Left(std::iter::once(c)),
        |cs| itertools::Either::Right(cs.iter().copied()),
    )
}

/// 復活の呪文の形式を正規化する(デコード可能かどうかは関知しない)。
/// 戻り値は有効な形式であることが保証される。
///
/// 空白文字を除去する。
///
/// `password` の形式が無効な場合、`Err(Dq1PasswordError::InvalidPassword)` を返す。
pub fn normalize_password(password: impl AsRef<str>) -> Dq1PasswordResult<String> {
    // validation もこちらで行う。

    let cs: Vec<_> = password
        .as_ref()
        .chars()
        .flat_map(normalize_password_char)
        .take(20 + 1)
        .collect();

    if cs.len() != 20 {
        return Err(Dq1PasswordError::invalid_password(
            "復活の呪文はちょうど 20 文字でなければならない(ただし空白文字は無視される)",
        ));
    }

    let cs_invalid: Vec<_> = cs
        .iter()
        .filter(|&&c| password_char_to_cum(c).is_none())
        .collect();

    if !cs_invalid.is_empty() {
        return Err(Dq1PasswordError::invalid_password(format!(
            "復活の呪文に無効な文字が含まれている: {}",
            cs_invalid
                .into_iter()
                .map(|c| format!("'{}'", c))
                .join(", ")
        )));
    }

    Ok(cs.into_iter().collect())
}

fn normalize_password_char(c: char) -> impl Iterator<Item = char> {
    let cs = match c {
        _ if c.is_whitespace() => Some(&[]),
        _ => None,
    };

    cs.map_or_else(
        || itertools::Either::Left(std::iter::once(c)),
        |cs| itertools::Either::Right(cs.iter().copied()),
    )
}

/// 復活の呪文パターンを正規化する。戻り値は有効であることが保証される。
///
/// 空白文字を除去し、全角の '？' を '?' に置換する。
///
/// `pattern` が無効な場合、`Err(Dq1PasswordError::InvalidPattern)` を返す。
pub fn normalize_pattern(pattern: impl AsRef<str>) -> Dq1PasswordResult<String> {
    // validation もこちらで行う。

    let cs: Vec<_> = pattern
        .as_ref()
        .chars()
        .flat_map(normalize_pattern_char)
        .take(20 + 1)
        .collect();

    if cs.len() != 20 {
        return Err(Dq1PasswordError::invalid_pattern(
            "パターンはちょうど 20 文字でなければならない(ただし空白文字は無視される)",
        ));
    }

    let cs_invalid: Vec<_> = cs
        .iter()
        .filter(|&&c| password_char_to_cum(c).is_none() && c != '?')
        .collect();

    if !cs_invalid.is_empty() {
        return Err(Dq1PasswordError::invalid_password(format!(
            "パターンに無効な文字が含まれている: {}",
            cs_invalid
                .into_iter()
                .map(|c| format!("'{}'", c))
                .join(", ")
        )));
    }

    Ok(cs.into_iter().collect())
}

fn normalize_pattern_char(c: char) -> impl Iterator<Item = char> {
    let cs: Option<&[char]> = match c {
        '？' => Some(&['?']),
        _ if c.is_whitespace() => Some(&[]),
        _ => None,
    };

    cs.map_or_else(
        || itertools::Either::Left(std::iter::once(c)),
        |cs| itertools::Either::Right(cs.iter().copied()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(non_snake_case)]
    fn S(s: &'static str) -> String {
        s.to_owned()
    }

    #[test]
    fn test_normalize_hero_name() {
        assert_eq!(normalize_hero_name(""), Ok(S("    ")));
        assert_eq!(normalize_hero_name("0123"), Ok(S("0123")));
        assert_eq!(normalize_hero_name("ああああ"), Ok(S("ああああ")));
        assert_eq!(normalize_hero_name("がぱ"), Ok(S("か゛は゜")));
        assert_eq!(normalize_hero_name("　あーす"), Ok(S(" あ-す")));

        assert!(normalize_hero_name("     ").is_err());
        assert!(normalize_hero_name("あああが").is_err());
        assert!(normalize_hero_name("A").is_err());
        assert!(normalize_hero_name("漢字").is_err());
    }

    #[test]
    fn test_normalize_password() {
        assert_eq!(
            normalize_password("ああああああああああああああああああああ"),
            Ok(S("ああああああああああああああああああああ"))
        );
        assert_eq!(
            normalize_password("あああああ あああああああ　あああああ あああ"),
            Ok(S("ああああああああああああああああああああ"))
        );

        assert!(normalize_password("あああああああああああああああああああ").is_err());
        assert!(normalize_password("あああああああああああああああああああああ").is_err());
        assert!(normalize_password("ああああああああああああああああああ漢字").is_err());
    }

    #[test]
    fn test_normalize_pattern() {
        assert_eq!(
            normalize_pattern("あああああああ?あ?ああああああああああ"),
            Ok(S("あああああああ?あ?ああああああああああ"))
        );
        assert_eq!(
            normalize_pattern("あああああ ああ?あ？ああ　あああああ あああ"),
            Ok(S("あああああああ?あ?ああああああああああ"))
        );

        assert!(normalize_password("ああああああああああああああああああああ?").is_err());
        assert!(normalize_password("ああああああああああああああああああ?").is_err());
        assert!(normalize_password("あああああああああああああああああ漢字?").is_err());
    }
}
