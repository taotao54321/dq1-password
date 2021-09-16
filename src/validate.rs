use crate::error::{Dq1PasswordError, Dq1PasswordResult};
use crate::normalize::{normalize_hero_name, normalize_password, normalize_pattern};

/// 主人公の名前を validate する。正規化していないものも許す。
pub fn validate_hero_name(hero_name: impl AsRef<str>) -> Dq1PasswordResult<()> {
    normalize_hero_name(hero_name).map(|_| ())
}

/// 主人公の装備している武器IDを validate する。
pub fn validate_hero_weapon(weapon: u8) -> Dq1PasswordResult<()> {
    const WEAPON_MAX: u8 = 7;

    if weapon > WEAPON_MAX {
        return Err(Dq1PasswordError::invalid_game_state(format!(
            "主人公の装備している武器IDは {} 以下でなければならない: {}",
            WEAPON_MAX, weapon
        )));
    }

    Ok(())
}

/// 主人公の装備している鎧IDを validate する。
pub fn validate_hero_armor(armor: u8) -> Dq1PasswordResult<()> {
    const ARMOR_MAX: u8 = 7;

    if armor > ARMOR_MAX {
        return Err(Dq1PasswordError::invalid_game_state(format!(
            "主人公の装備している鎧IDは {} 以下でなければならない: {}",
            ARMOR_MAX, armor
        )));
    }

    Ok(())
}

/// 主人公の装備している盾IDを validate する。
pub fn validate_hero_shield(shield: u8) -> Dq1PasswordResult<()> {
    const SHIELD_MAX: u8 = 3;

    if shield > SHIELD_MAX {
        return Err(Dq1PasswordError::invalid_game_state(format!(
            "主人公の装備している盾IDは {} 以下でなければならない: {}",
            SHIELD_MAX, shield
        )));
    }

    Ok(())
}

/// やくそう所持数を validate する。
pub fn validate_herb_count(herb: u8) -> Dq1PasswordResult<()> {
    const HERB_MAX: u8 = 6;

    if herb > HERB_MAX {
        return Err(Dq1PasswordError::invalid_game_state(format!(
            "やくそう所持数は {} 以下でなければならない: {}",
            HERB_MAX, herb
        )));
    }

    Ok(())
}

/// かぎ所持数を validate する。
pub fn validate_key_count(key: u8) -> Dq1PasswordResult<()> {
    const KEY_MAX: u8 = 6;

    if key > KEY_MAX {
        return Err(Dq1PasswordError::invalid_game_state(format!(
            "かぎ所持数は {} 以下でなければならない: {}",
            KEY_MAX, key
        )));
    }

    Ok(())
}

/// インベントリを validate する。
///
/// `inventory.len() == 8` でない場合、panic する。
pub fn validate_inventory(inventory: impl AsRef<[u8]>) -> Dq1PasswordResult<()> {
    let inventory = inventory.as_ref();

    assert_eq!(inventory.len(), 8);

    for (i, &tool) in inventory.iter().enumerate() {
        if let Err(e) = validate_tool(tool) {
            return Err(Dq1PasswordError::invalid_game_state(format!(
                "インベントリ[{}]: {}",
                i, e
            )));
        }
    }

    Ok(())
}

/// 道具IDを validate する。
pub fn validate_tool(tool: u8) -> Dq1PasswordResult<()> {
    const TOOL_MAX: u8 = 14;

    if tool > TOOL_MAX {
        return Err(Dq1PasswordError::invalid_game_state(format!(
            "道具IDは {} 以下でなければならない: {}",
            TOOL_MAX, tool
        )));
    }

    Ok(())
}

/// 復活の呪文エンコード用 salt を validate する。
pub fn validate_salt(salt: u8) -> Dq1PasswordResult<()> {
    const SALT_MAX: u8 = 7;

    if salt > SALT_MAX {
        return Err(Dq1PasswordError::invalid_game_state(format!(
            "salt は {} 以下でなければならない: {}",
            SALT_MAX, salt
        )));
    }

    Ok(())
}

/// 復活の呪文の形式を validate する(デコード可能かどうかは関知しない)。正規化していないものも許す。
pub fn validate_password(password: impl AsRef<str>) -> Dq1PasswordResult<()> {
    normalize_password(password).map(|_| ())
}

/// 復活の呪文パターンを validate する。正規化していないものも許す。
pub fn validate_pattern(pattern: impl AsRef<str>) -> Dq1PasswordResult<()> {
    normalize_pattern(pattern).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_hero_name() {
        assert!(validate_hero_name("").is_ok());
        assert!(validate_hero_name("0123").is_ok());
        assert!(validate_hero_name("ああああ").is_ok());
        assert!(validate_hero_name("がぱ").is_ok());
        assert!(validate_hero_name("　あーす").is_ok());

        assert!(validate_hero_name("     ").is_err());
        assert!(validate_hero_name("あああが").is_err());
        assert!(validate_hero_name("A").is_err());
        assert!(validate_hero_name("漢字").is_err());
    }

    #[test]
    fn test_validate_hero_weapon() {
        assert!(validate_hero_weapon(0).is_ok());
        assert!(validate_hero_weapon(7).is_ok());

        assert!(validate_hero_weapon(8).is_err());
    }

    #[test]
    fn test_validate_hero_armor() {
        assert!(validate_hero_armor(0).is_ok());
        assert!(validate_hero_armor(7).is_ok());

        assert!(validate_hero_armor(8).is_err());
    }

    #[test]
    fn test_validate_hero_shield() {
        assert!(validate_hero_shield(0).is_ok());
        assert!(validate_hero_shield(3).is_ok());

        assert!(validate_hero_shield(4).is_err());
    }

    #[test]
    fn test_validate_herb_count() {
        assert!(validate_herb_count(0).is_ok());
        assert!(validate_herb_count(6).is_ok());

        assert!(validate_herb_count(7).is_err());
    }

    #[test]
    fn test_validate_key_count() {
        assert!(validate_key_count(0).is_ok());
        assert!(validate_key_count(6).is_ok());

        assert!(validate_key_count(7).is_err());
    }

    #[test]
    fn test_validate_inventory() {
        assert!(validate_inventory([0; 8]).is_ok());
        assert!(validate_inventory([14; 8]).is_ok());

        assert!(validate_inventory([0, 0, 0, 0, 0, 0, 0, 15]).is_err());
    }

    #[test]
    fn test_validate_tool() {
        assert!(validate_tool(0).is_ok());
        assert!(validate_tool(14).is_ok());

        assert!(validate_tool(15).is_err());
    }

    #[test]
    fn test_validate_salt() {
        assert!(validate_salt(0).is_ok());
        assert!(validate_salt(7).is_ok());

        assert!(validate_salt(8).is_err());
    }

    #[test]
    fn test_validate_password() {
        assert!(validate_password("ああああああああああああああああああああ").is_ok());
        assert!(validate_password("あああああ あああああああ　あああああ あああ").is_ok());

        assert!(validate_password("あああああああああああああああああああ").is_err());
        assert!(validate_password("あああああああああああああああああああああ").is_err());
        assert!(validate_password("ああああああああああああああああああ漢字").is_err());
    }

    #[test]
    fn test_validate_pattern() {
        assert!(validate_pattern("あああああああ?あ?ああああああああああ").is_ok());
        assert!(validate_pattern("あああああ ああ?あ？ああ　あああああ あああ").is_ok());

        assert!(validate_password("ああああああああああああああああああああ?").is_err());
        assert!(validate_password("ああああああああああああああああああ?").is_err());
        assert!(validate_password("あああああああああああああああああ漢字?").is_err());
    }
}
