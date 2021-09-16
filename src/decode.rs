use std::convert::TryFrom;

use crate::crc::crc_update;
use crate::error::{Dq1PasswordError, Dq1PasswordResult};
use crate::game_state::GameState;
use crate::normalize::normalize_password;
use crate::validate::{validate_herb_count, validate_inventory, validate_key_count};

/// 復活の呪文を正規化されたゲーム状態にデコードして返す。
///
/// `password` の形式が無効な場合、`Err(Dq1PasswordError::InvalidPassword)` を返す。
///
/// デコード結果が無効なゲーム状態となる場合、`Err(Dq1PasswordError::InvalidGameState)` を返す。
///
/// デコード結果の CRC が一致しない場合、`Err(Dq1PasswordError::CrcMismatch)` を返す。
pub fn decode(password: impl AsRef<str>) -> Dq1PasswordResult<GameState> {
    let password = normalize_password(password)?;

    let bytes = password_to_bytes(password);
    check_bytes_crc(&bytes)?;

    let state = bytes_to_state(&bytes);
    validate_herb_count(state.herb_count)?;
    validate_key_count(state.key_count)?;
    validate_inventory(&state.inventory)?;

    Ok(state)
}

/// 復活の呪文をゲーム状態を表すバイト列に変換する。
///
/// `password` は正規化済みでなければならない。
fn password_to_bytes(password: impl AsRef<str>) -> [u8; 15] {
    let cs: Vec<_> = password.as_ref().chars().collect();

    let mut bytes = [0; 15];

    let mut pre = 0;
    let mut get_bits = move |c: char| -> u8 {
        let cur = password_char_to_cum(c).unwrap();
        let bits = (cur.wrapping_sub(pre + 4)) & 0x3F;
        pre = cur;
        bits
    };

    for (chunk, cs) in itertools::izip!(bytes.chunks_mut(3), cs.chunks(4)) {
        let bits = get_bits(cs[0]);
        chunk[0] = bits;
        let bits = get_bits(cs[1]);
        chunk[0] |= bits << 6;
        chunk[1] = bits >> 2;
        let bits = get_bits(cs[2]);
        chunk[1] |= bits << 4;
        chunk[2] = bits >> 4;
        let bits = get_bits(cs[3]);
        chunk[2] |= bits << 2;
    }

    bytes
}

pub(crate) fn password_char_to_cum(c: char) -> Option<u8> {
    const MAP: phf::Map<char, u8> = phf::phf_map! {
        'あ' => 0x00, 'い' => 0x01, 'う' => 0x02, 'え' => 0x03, 'お' => 0x04,
        'か' => 0x05, 'き' => 0x06, 'く' => 0x07, 'け' => 0x08, 'こ' => 0x09,
        'さ' => 0x0A, 'し' => 0x0B, 'す' => 0x0C, 'せ' => 0x0D, 'そ' => 0x0E,
        'た' => 0x0F, 'ち' => 0x10, 'つ' => 0x11, 'て' => 0x12, 'と' => 0x13,
        'な' => 0x14, 'に' => 0x15, 'ぬ' => 0x16, 'ね' => 0x17, 'の' => 0x18,
        'は' => 0x19, 'ひ' => 0x1A, 'ふ' => 0x1B, 'へ' => 0x1C, 'ほ' => 0x1D,
        'ま' => 0x1E, 'み' => 0x1F, 'む' => 0x20, 'め' => 0x21, 'も' => 0x22,
        'や' => 0x23, 'ゆ' => 0x24, 'よ' => 0x25,
        'ら' => 0x26, 'り' => 0x27, 'る' => 0x28, 'れ' => 0x29, 'ろ' => 0x2A,
        'わ' => 0x2B,
        'が' => 0x2C, 'ぎ' => 0x2D, 'ぐ' => 0x2E, 'げ' => 0x2F, 'ご' => 0x30,
        'ざ' => 0x31, 'じ' => 0x32, 'ず' => 0x33, 'ぜ' => 0x34, 'ぞ' => 0x35,
        'だ' => 0x36, 'ぢ' => 0x37, 'づ' => 0x38, 'で' => 0x39, 'ど' => 0x3A,
        'ば' => 0x3B, 'び' => 0x3C, 'ぶ' => 0x3D, 'べ' => 0x3E, 'ぼ' => 0x3F,
    };

    MAP.get(&c).copied()
}

fn check_bytes_crc(bytes: &[u8; 15]) -> Dq1PasswordResult<()> {
    let crc_actual = bytes[1..].iter().fold(0, |crc, &b| crc_update(crc, b, 8));

    if u8::try_from(crc_actual & 0xFF).unwrap() != bytes[0] {
        return Err(Dq1PasswordError::crc_mismatch(bytes[0], crc_actual));
    }

    Ok(())
}

/// バイト列をゲーム状態に変換する。
fn bytes_to_state(bytes: &[u8; 15]) -> GameState {
    fn bit_test(x: u8, idx: u8) -> bool {
        (x & (1 << idx)) != 0
    }

    let hero_name_packed = [
        bytes[5] >> 2,
        (bytes[13] >> 1) & 0x3F,
        bytes[2] & 0x3F,
        bytes[7] & 0x3F,
    ];
    let hero_name = unpack_hero_name(hero_name_packed);

    let hero_xp = u16::from(bytes[1]) | (u16::from(bytes[12]) << 8);
    let purse = u16::from(bytes[4]) | (u16::from(bytes[9]) << 8);
    let hero_weapon = bytes[8] >> 5;
    let hero_armor = (bytes[8] >> 2) & 0x7;
    let hero_shield = bytes[8] & 0x3;
    let herb_count = bytes[10] & 0xF;
    let key_count = bytes[10] >> 4;
    let inventory = [
        bytes[14] & 0xF,
        bytes[14] >> 4,
        bytes[3] & 0xF,
        bytes[3] >> 4,
        bytes[11] & 0xF,
        bytes[11] >> 4,
        bytes[6] & 0xF,
        bytes[6] >> 4,
    ];
    let flag_equip_dragon_scale = bit_test(bytes[13], 7);
    let flag_equip_warrior_ring = bit_test(bytes[13], 0);
    let flag_got_death_necklace = bit_test(bytes[2], 6);
    let flag_beated_golem = bit_test(bytes[5], 1);
    let flag_beated_dragon = bit_test(bytes[7], 6);
    let salt = u8::from(bit_test(bytes[5], 0))
        | (u8::from(bit_test(bytes[2], 7)) << 1)
        | (u8::from(bit_test(bytes[7], 7)) << 2);

    GameState {
        hero_name,
        hero_xp,
        purse,
        hero_weapon,
        hero_armor,
        hero_shield,
        herb_count,
        key_count,
        inventory,
        flag_equip_dragon_scale,
        flag_equip_warrior_ring,
        flag_got_death_necklace,
        flag_beated_golem,
        flag_beated_dragon,
        salt,
    }
}

/// 6bit 値の配列を主人公の名前に unpack する。
fn unpack_hero_name(packed: [u8; 4]) -> String {
    IntoIterator::into_iter(packed)
        .map(unpack_hero_name_char)
        .collect()
}

/// 6bit 値を主人公の名前の文字に unpack する。
fn unpack_hero_name_char(b: u8) -> char {
    #[rustfmt::skip]
    const CHARS: [char; 0x40] = [
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
        'あ', 'い', 'う', 'え', 'お',
        'か', 'き', 'く', 'け', 'こ',
        'さ', 'し', 'す', 'せ', 'そ',
        'た', 'ち', 'つ', 'て', 'と',
        'な', 'に', 'ぬ', 'ね', 'の',
        'は', 'ひ', 'ふ', 'へ', 'ほ',
        'ま', 'み', 'む', 'め', 'も',
        'や', 'ゆ', 'よ',
        'ら', 'り', 'る', 'れ', 'ろ',
        'わ', 'を', 'ん',
        'っ', 'ゃ', 'ゅ', 'ょ',
        '゛', '゜', '-', ' ',
    ];

    CHARS[usize::from(b)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(non_snake_case)]
    fn S(s: &'static str) -> String {
        s.to_owned()
    }

    #[test]
    fn test_decode() {
        assert_eq!(
            decode("つにこへむゆるわげげだどべうきさそさには"),
            Ok(GameState::default().normalize().unwrap())
        );

        // 復活の呪文 A
        assert_eq!(
            decode("ざぼちずどぢぎきつたうずせれえむるのぢえ"),
            Ok(GameState {
                hero_name: S("しと゛-"),
                hero_xp: 1234,
                purse: 5678,
                hero_weapon: 5,
                hero_armor: 5,
                hero_shield: 2,
                herb_count: 6,
                key_count: 6,
                inventory: [1, 2, 3, 4, 5, 6, 7, 8],
                flag_equip_dragon_scale: true,
                flag_equip_warrior_ring: true,
                flag_got_death_necklace: true,
                flag_beated_golem: true,
                flag_beated_dragon: true,
                salt: 5,
            }),
        );

        // 復活の呪文 A に対応する GameState の inventory[7] を 15 にしたもの
        assert!(matches!(
            decode("どくのばうぼぞそこけばがきもびはめつごび"),
            Err(Dq1PasswordError::InvalidGameState(_)),
        ));

        // 復活の呪文 A の最後の文字を変えたもの
        assert!(matches!(
            decode("ざぼちずどぢぎきつたうずせれえむるのぢお"),
            Err(Dq1PasswordError::CrcMismatch { .. })
        ));
    }
}
