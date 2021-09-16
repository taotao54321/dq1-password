use std::convert::TryInto;

use crate::crc::crc_update;
use crate::error::Dq1PasswordResult;
use crate::game_state::GameState;

/// ゲーム状態を復活の呪文にエンコードして返す。
///
/// `state` が無効な場合、`Err(Dq1PasswordError::InvalidGameState)` を返す。
pub fn encode(state: &GameState) -> Dq1PasswordResult<String> {
    let state = state.normalize()?;

    let bytes = state_to_bytes(&state);

    Ok(bytes_to_password(&bytes))
}

/// ゲーム状態をバイト列に変換する。
///
/// `state` は正規化済みでなければならない。
fn state_to_bytes(state: &GameState) -> [u8; 15] {
    fn u16_lo(x: u16) -> u8 {
        (x & 0xFF).try_into().unwrap()
    }
    fn u16_hi(x: u16) -> u8 {
        (x >> 8).try_into().unwrap()
    }
    fn bit_test(x: u8, idx: u8) -> u8 {
        u8::from((x & (1 << idx)) != 0)
    }

    let hero_name_packed = pack_hero_name(&state.hero_name);

    let mut bytes = [0; 15];

    bytes[1] = u16_lo(state.hero_xp);
    bytes[2] = hero_name_packed[2]
        | (u8::from(state.flag_got_death_necklace) << 6)
        | (bit_test(state.salt, 1) << 7);
    bytes[3] = state.inventory[2] | (state.inventory[3] << 4);
    bytes[4] = u16_lo(state.purse);
    bytes[5] = bit_test(state.salt, 0)
        | (u8::from(state.flag_beated_golem) << 1)
        | (hero_name_packed[0] << 2);
    bytes[6] = state.inventory[6] | (state.inventory[7] << 4);
    bytes[7] = hero_name_packed[3]
        | (u8::from(state.flag_beated_dragon) << 6)
        | (bit_test(state.salt, 2) << 7);
    bytes[8] = state.hero_shield | (state.hero_armor << 2) | (state.hero_weapon << 5);
    bytes[9] = u16_hi(state.purse);
    bytes[10] = state.herb_count | (state.key_count << 4);
    bytes[11] = state.inventory[4] | (state.inventory[5] << 4);
    bytes[12] = u16_hi(state.hero_xp);
    bytes[13] = u8::from(state.flag_equip_warrior_ring)
        | (hero_name_packed[1] << 1)
        | (u8::from(state.flag_equip_dragon_scale) << 7);
    bytes[14] = state.inventory[0] | (state.inventory[1] << 4);

    bytes[0] = u16_lo(bytes[1..].iter().fold(0, |crc, &b| crc_update(crc, b, 8)));

    bytes
}

/// 主人公の名前の各文字を 6bit に pack した値の配列を返す。
///
/// `hero_name` は正規化済みでなければならない。
fn pack_hero_name(hero_name: impl AsRef<str>) -> [u8; 4] {
    let mut packed = [0; 4];

    for (e, c) in itertools::zip(&mut packed, hero_name.as_ref().chars()) {
        *e = pack_hero_name_char(c).unwrap();
    }

    packed
}

/// 主人公の名前の文字を 6bit に pack した値を返す。
pub(crate) fn pack_hero_name_char(c: char) -> Option<u8> {
    const MAP: phf::Map<char, u8> = phf::phf_map! {
        '0'  => 0x00,
        '1'  => 0x01,
        '2'  => 0x02,
        '3'  => 0x03,
        '4'  => 0x04,
        '5'  => 0x05,
        '6'  => 0x06,
        '7'  => 0x07,
        '8'  => 0x08,
        '9'  => 0x09,
        'あ' => 0x0A,
        'い' => 0x0B,
        'う' => 0x0C,
        'え' => 0x0D,
        'お' => 0x0E,
        'か' => 0x0F,
        'き' => 0x10,
        'く' => 0x11,
        'け' => 0x12,
        'こ' => 0x13,
        'さ' => 0x14,
        'し' => 0x15,
        'す' => 0x16,
        'せ' => 0x17,
        'そ' => 0x18,
        'た' => 0x19,
        'ち' => 0x1A,
        'つ' => 0x1B,
        'て' => 0x1C,
        'と' => 0x1D,
        'な' => 0x1E,
        'に' => 0x1F,
        'ぬ' => 0x20,
        'ね' => 0x21,
        'の' => 0x22,
        'は' => 0x23,
        'ひ' => 0x24,
        'ふ' => 0x25,
        'へ' => 0x26,
        'ほ' => 0x27,
        'ま' => 0x28,
        'み' => 0x29,
        'む' => 0x2A,
        'め' => 0x2B,
        'も' => 0x2C,
        'や' => 0x2D,
        'ゆ' => 0x2E,
        'よ' => 0x2F,
        'ら' => 0x30,
        'り' => 0x31,
        'る' => 0x32,
        'れ' => 0x33,
        'ろ' => 0x34,
        'わ' => 0x35,
        'を' => 0x36,
        'ん' => 0x37,
        'っ' => 0x38,
        'ゃ' => 0x39,
        'ゅ' => 0x3A,
        'ょ' => 0x3B,
        '゛' => 0x3C,
        '゜' => 0x3D,
        '-'  => 0x3E,
        ' '  => 0x3F,
    };

    MAP.get(&c).copied()
}

/// ゲーム状態を表すバイト列を復活の呪文に変換する。
pub(crate) fn bytes_to_password(bytes: &[u8; 15]) -> String {
    // utf-8 のひらがな 20 文字分の容量を確保。
    let mut password = String::with_capacity(3 * 20);

    let mut cum = 0;
    for chunk in bytes.chunks(3) {
        cum = (cum + (chunk[0] & 0x3F) + 4) & 0x3F;
        password.push(cum_to_password_char(cum));
        cum = (cum + ((chunk[0] >> 6) | ((chunk[1] & 0xF) << 2)) + 4) & 0x3F;
        password.push(cum_to_password_char(cum));
        cum = (cum + ((chunk[1] >> 4) | ((chunk[2] & 0x3) << 4)) + 4) & 0x3F;
        password.push(cum_to_password_char(cum));
        cum = (cum + (chunk[2] >> 2) + 4) & 0x3F;
        password.push(cum_to_password_char(cum));
    }

    password
}

fn cum_to_password_char(cum: u8) -> char {
    #[rustfmt::skip]
    const CHARS: [char; 0x40] = [
        'あ', 'い', 'う', 'え', 'お',
        'か', 'き', 'く', 'け', 'こ',
        'さ', 'し', 'す', 'せ', 'そ',
        'た', 'ち', 'つ', 'て', 'と',
        'な', 'に', 'ぬ', 'ね', 'の',
        'は', 'ひ', 'ふ', 'へ', 'ほ',
        'ま', 'み', 'む', 'め', 'も',
        'や', 'ゆ', 'よ',
        'ら', 'り', 'る', 'れ', 'ろ',
        'わ',
        'が', 'ぎ', 'ぐ', 'げ', 'ご',
        'ざ', 'じ', 'ず', 'ぜ', 'ぞ',
        'だ', 'ぢ', 'づ', 'で', 'ど',
        'ば', 'び', 'ぶ', 'べ', 'ぼ',
    ];

    CHARS[usize::from(cum)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(non_snake_case)]
    fn S(s: &'static str) -> String {
        s.to_owned()
    }

    #[test]
    fn test_encode() {
        assert_eq!(
            encode(&GameState::default()),
            Ok(S("つにこへむゆるわげげだどべうきさそさには")),
        );

        assert_eq!(
            encode(&GameState {
                hero_name: S("しどー"),
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
            Ok(S("ざぼちずどぢぎきつたうずせれえむるのぢえ"))
        );
    }
}
