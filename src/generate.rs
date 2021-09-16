use std::convert::TryInto;
use std::ops::RangeInclusive;

use crate::crc::crc_update;
use crate::decode::password_char_to_cum;
use crate::encode::bytes_to_password;
use crate::error::Dq1PasswordResult;
use crate::normalize::normalize_pattern;

/// 多次元 Vec を作る。
macro_rules! ndvec {
    ($elem:expr; $n:expr) => {{
        ::std::vec![$elem; $n]
    }};
    ($elem:expr; $n:expr, $($ns:expr),+ $(,)?) => {{
        ::std::vec![ndvec![$elem; $($ns),+]; $n]
    }};
}

/// 指定されたパターンに合致するデコード可能な復活の呪文たちを生成する。
///
/// `pattern` 内の '?' (半角/全角どちらも可)は任意の文字にマッチする。
///
/// `n_max` は生成上限数。
///
/// `pattern` が無効な場合、`Err(Dq1PasswordError::InvalidPattern)` を返す。
pub fn generate(pattern: impl AsRef<str>, n_max: usize) -> Dq1PasswordResult<Vec<String>> {
    // パターンを累積値の配列に変換する。'?' の部分は None になる。
    let cums: Vec<_> = normalize_pattern(pattern)?
        .chars()
        .map(password_char_to_cum)
        .collect();

    let (cums_head, cums_tail) = cums.split_at(2);
    let cums_tail: [_; 18] = cums_tail.try_into().unwrap();

    let mut bytess = Vec::with_capacity(n_max);
    let mut n_remain = n_max;
    for (cum0, cum1) in itertools::iproduct!(cum_range(cums_head[0]), cum_range(cums_head[1])) {
        if n_remain == 0 {
            break;
        }

        let sixs_head = [
            cum0.wrapping_sub(4) & 0x3F,
            cum1.wrapping_sub(cum0 + 4) & 0x3F,
        ];
        let partial = generate_dp(sixs_head, &cums_tail, n_remain);
        n_remain -= partial.len();
        bytess.extend(partial);
    }

    Ok(bytess.iter().map(bytes_to_password).collect())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DpTrace(u16);

impl DpTrace {
    fn new(j: u8, k: u8, l: u8) -> Self {
        debug_assert!((0..=0x3F).contains(&j));
        debug_assert!((0..=0xFF).contains(&k));
        debug_assert!((0..=1).contains(&l));
        Self(u16::from(k) | (u16::from(j) << 8) | (u16::from(l) << 14))
    }

    fn j(self) -> u8 {
        ((self.0 >> 8) & 0x3F).try_into().unwrap()
    }

    fn k(self) -> u8 {
        (self.0 & 0xFF).try_into().unwrap()
    }

    fn l(self) -> u8 {
        (self.0 >> 14).try_into().unwrap()
    }
}

/// 6bit 値配列の先頭 2 要素を指定し、有効なゲーム状態に対応するバイト列たちを生成する。
///
/// 動的計画法を用いる。
fn generate_dp(sixs_head: [u8; 2], cums_tail: &[Option<u8>; 18], n_max: usize) -> Vec<[u8; 15]> {
    const CRC_TABLE: [[u8; 0x40]; 18] = crc8_table_tail();

    debug_assert_ne!(n_max, 0);

    let cum_ini = (sixs_head[0] + sixs_head[1] + 8) & 0x3F;
    let crc_ini = crc8_table_head()[usize::from(sixs_head[1] >> 2)];

    // dp[i][j][k][l]:
    //   tail 部 i 個目までで cum=j, crc=k であるときの Vec<DpTrace> (最大要素数 n_max)
    //   l: six の上位 2bit が 0b11 であるか (道具IDの validate 用)
    let mut dp = ndvec![Vec::with_capacity(n_max); 19, 0x40, 0x100, 2];
    dp[0][usize::from(cum_ini)][usize::from(crc_ini)][0].push(DpTrace::new(0, 0, 0)); // 値自体に意味はない

    // 配るDP
    for (i, j, k, l) in itertools::iproduct!(0..18, 0..=0x3F, 0..=0xFF, 0..=1) {
        if dp[i][usize::from(j)][usize::from(k)][usize::from(l)].is_empty() {
            continue;
        }

        for cum in cum_range(cums_tail[i]) {
            let six = cum.wrapping_sub(j + 4) & 0x3F;

            // やくそう所持数が無効な場合は弾く。
            if i == 11 && (six >> 2) >= 7 {
                continue;
            }

            // かぎ所持数が無効な場合は弾く。
            if i == 12 && (six & 0xF) >= 7 {
                continue;
            }

            // インベントリに無効な道具IDが含まれる場合は弾く。
            if matches!(i, 2 | 6) && (six & 0xF) == 15 {
                continue;
            }
            if matches!(i, 13 | 17) && (six >> 2) == 15 {
                continue;
            }
            if matches!(i, 3 | 7 | 13 | 17) && l == 1 && (six & 3) == 3 {
                continue;
            }

            let crc = k ^ CRC_TABLE[i][usize::from(six)];
            let l_nxt = (six >> 4) == 3;

            let traces = &mut dp[i + 1][usize::from(cum)][usize::from(crc)][usize::from(l_nxt)];
            if traces.len() < n_max {
                traces.push(DpTrace::new(j, k, l));
            }
        }
    }

    generate_dp_restore(sixs_head, cums_tail, n_max, &dp)
}

fn generate_dp_restore(
    sixs_head: [u8; 2],
    cums_tail: &[Option<u8>; 18],
    n_max: usize,
    dp: &[Vec<Vec<Vec<Vec<DpTrace>>>>],
) -> Vec<[u8; 15]> {
    let crc_expect = sixs_head[0] | (sixs_head[1] << 6);

    struct Dfs<'a> {
        n_max: usize,
        dp: &'a [Vec<Vec<Vec<Vec<DpTrace>>>>],
        bytess: Vec<[u8; 15]>,
    }
    impl Dfs<'_> {
        /// 発見済の解の個数が n_max に達したら true を返す。
        fn dfs(&mut self, i: usize, j: u8, k: u8, l: u8, sixs: &mut [u8; 20]) -> bool {
            debug_assert!(!self.dp[i][usize::from(j)][usize::from(k)][usize::from(l)].is_empty());

            if i == 0 {
                self.bytess.push(sixs_to_bytes(sixs));
                return self.bytess.len() == self.n_max;
            }

            for &trace in &self.dp[i][usize::from(j)][usize::from(k)][usize::from(l)] {
                let six = j.wrapping_sub(trace.j() + 4) & 0x3F;
                sixs[i + 1] = six;
                if self.dfs(i - 1, trace.j(), trace.k(), trace.l(), sixs) {
                    return true;
                }
            }

            false
        }
    }

    let mut dfs = Dfs {
        n_max,
        dp,
        bytess: Vec::with_capacity(n_max),
    };
    let mut sixs = [0; 20];
    sixs[0] = sixs_head[0];
    sixs[1] = sixs_head[1];
    for (cum, l) in itertools::iproduct!(cum_range(cums_tail[17]), 0..=1) {
        if dp[18][usize::from(cum)][usize::from(crc_expect)][usize::from(l)].is_empty() {
            continue;
        }
        if dfs.dfs(18, cum, crc_expect, l, &mut sixs) {
            break;
        }
    }

    dfs.bytess
}

fn cum_range(opt_cum: Option<u8>) -> RangeInclusive<u8> {
    opt_cum.map_or(0..=0x3F, |x| x..=x)
}

/// ゲーム状態バイト列の前半から CRC 部を除いた 4bit についての CRC 下位バイトテーブルを返す。
const fn crc8_table_head() -> [u8; 0x10] {
    const CRC16_TABLE: [[u16; 0x40]; 18] = crc16_table_tail();

    let mut table = [0; 0x10];

    let mut j = 0;
    while j < 0x10 {
        table[j] = (crc_update(
            crc_update(crc_update(CRC16_TABLE[3][j << 2], 0, 8), 0, 8),
            0,
            8,
        ) & 0xFF) as u8;
        j += 1;
    }

    table
}

/// ゲーム状態バイト列の後半 108bit についての 6bit 単位の CRC 下位バイトテーブルを返す。
const fn crc8_table_tail() -> [[u8; 0x40]; 18] {
    const CRC16_TABLE: [[u16; 0x40]; 18] = crc16_table_tail();

    let mut table = [[0; 0x40]; 18];

    let mut i = 0;
    while i < 18 {
        let mut j = 0;
        while j < 0x40 {
            table[i][j] = (CRC16_TABLE[i][j] & 0xFF) as u8;
            j += 1;
        }
        i += 1;
    }

    table
}

/// ゲーム状態バイト列の後半 108bit についての 6bit 単位の CRC テーブルを返す。
///
/// エンコード時は 8bit 単位で `crc_update()` が行われるため、bit 順が変わることに注意。
/// 24bit 単位で考えると以下のように並び替わる:
///
/// ```text
/// 8bit: | abcdefgh | ijklmnop | qrstuvwx |
/// 6bit: | cdefgh | mnopab | wxijkl | qrstuv |
/// ```
const fn crc16_table_tail() -> [[u16; 0x40]; 18] {
    let mut table = [[0; 0x40]; 18];

    let mut j = 0;
    while j < 0x40 {
        table[17][j as usize] = crc_update(0, j << 2, 8);
        table[16][j as usize] =
            crc_update(0, j >> 4, 2) ^ crc_update(crc_update(0, j << 4, 8), 0, 8);
        table[15][j as usize] = crc_update(crc_update(0, j >> 2, 4), 0, 8)
            ^ crc_update(crc_update(crc_update(0, j << 6, 8), 0, 8), 0, 8);
        table[14][j as usize] = crc_update(crc_update(crc_update(0, j, 6), 0, 8), 0, 8);
        j += 1;
    }

    let mut i = 13;
    loop {
        let mut j = 0;
        while j < 0x40 {
            table[i][j] = crc_update(crc_update(crc_update(table[i + 4][j], 0, 8), 0, 8), 0, 8);
            j += 1;
        }
        if i == 0 {
            break;
        }
        i -= 1;
    }

    table
}

fn sixs_to_bytes(sixs: &[u8; 20]) -> [u8; 15] {
    let mut bytes = [0; 15];

    for (bs, ss) in itertools::izip!(bytes.chunks_mut(3), sixs.chunks(4)) {
        bs[0] = ss[0] | (ss[1] << 6);
        bs[1] = (ss[1] >> 2) | (ss[2] << 4);
        bs[2] = (ss[2] >> 4) | (ss[3] << 2);
    }

    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ndvec() {
        assert_eq!(ndvec![0; 2, 3, 4], vec![vec![vec![0; 4]; 3]; 2]);
    }

    #[test]
    fn test_crc16_table_tail() {
        const TABLE: [[u16; 0x40]; 18] = crc16_table_tail();
        const BYTES: [u8; 3] = [0b01000101, 0b01100111, 0b10001001];

        let sixs = [
            BYTES[0] & 0x3F,
            (BYTES[0] >> 6) | ((BYTES[1] & 0xF) << 2),
            (BYTES[1] >> 4) | ((BYTES[2] & 0x3) << 4),
            BYTES[2] >> 2,
        ];

        let crc_actual = sixs
            .iter()
            .enumerate()
            .fold(0, |crc, (i, &s)| crc ^ TABLE[14 + i][usize::from(s)]);
        let crc_expect = BYTES.iter().fold(0, |crc, &b| crc_update(crc, b, 8));

        assert_eq!(crc_actual, crc_expect);
    }
}
