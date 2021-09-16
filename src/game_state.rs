use serde::{Deserialize, Serialize};

use crate::error::Dq1PasswordResult;
use crate::normalize::normalize_hero_name;
use crate::validate::*;

/// 復活の呪文に保存されるゲーム状態。
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct GameState {
    /// 主人公の名前。最大 4 文字(濁点、半濁点は 1 文字と数える)。
    /// 4 文字未満の場合、ASCII space でパディングされたものとして扱われる。
    pub hero_name: String,

    /// 主人公の経験値。
    pub hero_xp: u16,

    /// 所持金。
    pub purse: u16,

    /// 主人公の装備している武器ID。
    ///
    /// | 武器ID | 内容           |
    /// | --     | --             |
    /// | 0      | (なし)         |
    /// | 1      | たけざお       |
    /// | 2      | こんぼう       |
    /// | 3      | どうのつるぎ   |
    /// | 4      | てつのおの     |
    /// | 5      | はがねのつるぎ |
    /// | 6      | ほのおのつるぎ |
    /// | 7      | ロトのつるぎ   |
    pub hero_weapon: u8,

    /// 主人公の装備している鎧ID。
    ///
    /// | 鎧ID | 内容           |
    /// | --   | --             |
    /// | 0    | (なし)         |
    /// | 1    | ぬののふく     |
    /// | 2    | かわのふく     |
    /// | 3    | くさりかたびら |
    /// | 4    | てつのよろい   |
    /// | 5    | はがねのよろい |
    /// | 6    | まほうのよろい |
    /// | 7    | ロトのよろい   |
    pub hero_armor: u8,

    /// 主人公の装備している盾ID。
    ///
    /// | 盾ID | 内容           |
    /// | --   | --             |
    /// | 0    | (なし)         |
    /// | 1    | かわのたて     |
    /// | 2    | てつのたて     |
    /// | 3    | みかがみのたて |
    pub hero_shield: u8,

    /// やくそう所持数 (`0..=6`)。
    pub herb_count: u8,

    /// かぎ所持数 (`0..=6`)。
    pub key_count: u8,

    /// インベントリ(所持道具IDの配列)。
    ///
    /// | 道具ID | 内容           |
    /// | --     | --             |
    /// | 0      | (なし)         |
    /// | 1      | たいまつ       |
    /// | 2      | せいすい       |
    /// | 3      | キメラのつばさ |
    /// | 4      | りゅうのうろこ |
    /// | 5      | ようせいのふえ |
    /// | 6      | せんしのゆびわ |
    /// | 7      | ロトのしるし   |
    /// | 8      | おうじょのあい |
    /// | 9      | のろいのベルト |
    /// | 10     | ぎんのたてごと |
    /// | 11     | しのくびかざり |
    /// | 12     | たいようのいし |
    /// | 13     | あまぐものつえ |
    /// | 14     | にじのしずく   |
    pub inventory: [u8; 8],

    /// りゅうのうろこ装備フラグ。
    pub flag_equip_dragon_scale: bool,

    /// せんしのゆびわ装備フラグ。
    pub flag_equip_warrior_ring: bool,

    /// しのくびかざり取得済フラグ。
    pub flag_got_death_necklace: bool,

    /// メルキド入口のゴーレム撃破済フラグ。
    pub flag_beated_golem: bool,

    /// 沼地の洞窟のドラゴン撃破済フラグ。
    pub flag_beated_dragon: bool,

    /// 復活の呪文エンコード用 salt (`0..=7`)。
    pub salt: u8,
}

impl GameState {
    /// ゲーム状態を validate する。
    pub fn validate(&self) -> Dq1PasswordResult<()> {
        validate_hero_name(&self.hero_name)?;
        validate_hero_weapon(self.hero_weapon)?;
        validate_hero_armor(self.hero_armor)?;
        validate_hero_shield(self.hero_shield)?;
        validate_herb_count(self.herb_count)?;
        validate_key_count(self.key_count)?;
        validate_inventory(&self.inventory)?;
        validate_salt(self.salt)?;

        Ok(())
    }

    /// ゲーム状態を正規化したものを返す。戻り値は有効であることが保証される。
    ///
    /// 主人公の名前の正規化のみを行う。
    ///
    /// `self` が無効な場合、`Err(Dq1PasswordError::InvalidGameState)` を返す。
    pub fn normalize(&self) -> Dq1PasswordResult<Self> {
        let hero_name = normalize_hero_name(&self.hero_name)?;

        Ok(Self { hero_name, ..*self })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(non_snake_case)]
    fn S(s: &'static str) -> String {
        s.to_owned()
    }

    #[test]
    fn test_validate() {
        #[rustfmt::skip]
        {
            assert!(GameState::default().validate().is_ok());
            assert!(GameState { hero_name: S("あああああ"), ..Default::default() }.validate().is_err());
            assert!(GameState { hero_weapon: 8, ..Default::default() }.validate().is_err());
            assert!(GameState { hero_armor: 8, ..Default::default() }.validate().is_err());
            assert!(GameState { hero_shield: 4, ..Default::default() }.validate().is_err());
            assert!(GameState { herb_count: 7, ..Default::default() }.validate().is_err());
            assert!(GameState { key_count: 7, ..Default::default() }.validate().is_err());
            assert!(GameState { inventory: [0, 0, 0, 0, 0, 0, 0, 15], ..Default::default() }.validate().is_err());
            assert!(GameState { salt: 8, ..Default::default() }.validate().is_err());
        };
    }

    #[test]
    fn test_normalize() {
        assert_eq!(
            GameState::default().normalize(),
            Ok(GameState {
                hero_name: S("    "),
                ..Default::default()
            })
        );

        assert_eq!(
            GameState {
                hero_name: S("がー　"),
                ..Default::default()
            }
            .normalize(),
            Ok(GameState {
                hero_name: S("か゛- "),
                ..Default::default()
            })
        );

        assert!(GameState {
            hero_name: S("A"),
            ..Default::default()
        }
        .normalize()
        .is_err());
    }
}
