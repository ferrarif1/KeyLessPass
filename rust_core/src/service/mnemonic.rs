use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};

const DEFAULT_WORD_COUNT: usize = 20;
const MIN_WORD_COUNT: usize = 12;
const MAX_WORD_COUNT: usize = 32;

const ENGLISH_WORDS: &[&str] = &[
    "anchor",
    "apricot",
    "arrow",
    "atlas",
    "bamboo",
    "beacon",
    "birch",
    "breeze",
    "bridge",
    "cabin",
    "canyon",
    "cedar",
    "circle",
    "cloud",
    "cobalt",
    "copper",
    "coral",
    "crystal",
    "delta",
    "drift",
    "ember",
    "fabric",
    "falcon",
    "field",
    "forest",
    "garden",
    "galaxy",
    "harbor",
    "horizon",
    "island",
    "ivory",
    "jasmine",
    "journey",
    "kernel",
    "lantern",
    "laurel",
    "marble",
    "meadow",
    "metal",
    "mist",
    "morning",
    "noble",
    "north",
    "ocean",
    "olive",
    "orbit",
    "pebble",
    "pilot",
    "planet",
    "quartz",
    "quiet",
    "river",
    "rocket",
    "saffron",
    "silver",
    "signal",
    "solar",
    "spruce",
    "stone",
    "summit",
    "timber",
    "valley",
    "velvet",
    "violet",
    "water",
    "winter",
    "wisdom",
    "yellow",
    "zephyr",
    "amber",
    "aurora",
    "bay",
    "blossom",
    "brook",
    "canvas",
    "citadel",
    "cliff",
    "comet",
    "dawn",
    "desert",
    "echo",
    "elm",
    "fable",
    "feather",
    "flame",
    "frost",
    "garnet",
    "glacier",
    "grove",
    "hazel",
    "iris",
    "jade",
    "lagoon",
    "linen",
    "lotus",
    "maple",
    "meridian",
    "mineral",
    "nectar",
    "opal",
    "orchard",
    "pearl",
    "prairie",
    "reef",
    "ridge",
    "sierra",
    "sky",
    "solace",
    "spark",
    "stream",
    "sunrise",
    "terrace",
    "thistle",
    "tide",
    "topaz",
    "trail",
    "vernal",
    "willow",
    "wind",
    "zenith",
    "acorn",
    "basin",
    "bright",
    "cascade",
    "dune",
    "evergreen",
    "ginkgo",
    "harvest",
    "keystone",
    "lumen",
    "magnolia",
    "mesa",
    "pine",
    "ripple",
    "sable",
    "sunset",
];

const CHINESE_WORDS: &[&str] = &[
    "安宁", "白云", "北辰", "碧海", "晨光", "春风", "大地", "丹桂", "飞鸿", "枫林", "高山", "海岸",
    "寒星", "和风", "红叶", "华灯", "江河", "金石", "锦程", "静水", "兰亭", "蓝桥", "林泉", "流光",
    "明月", "南山", "清泉", "秋实", "瑞雪", "山河", "松涛", "天和", "万象", "微光", "星辰", "星河",
    "雪松", "阳光", "云海", "长风", "竹影", "朝露", "青岚", "远航", "澄湖", "星火", "松月", "海棠",
    "竹林", "晴川", "云帆", "锦年", "明澈", "静岚", "远山", "清辉", "松风", "澜石", "新雨", "初晴",
    "丹青", "碧空", "流云", "星野", "秋水", "春山", "云杉", "晴岚", "竹溪", "青石", "晨星", "月华",
    "风荷", "玉衡", "远洲", "明庭", "松径", "清越", "海月", "锦云", "青松", "晴雪", "兰舟", "云岭",
    "星渚", "映月", "风清", "溪桥", "柳岸", "碧峰", "晨曦", "云起", "清波", "松影", "华月", "星霜",
    "江月", "晴岫", "竹月", "青峰", "流泉", "新晴", "远岫", "明霞", "澄江", "秋岚", "松雪", "云岫",
    "静云", "晴光", "兰溪", "星桥", "风竹", "白石", "清岚", "远帆", "碧落", "春溪", "海风", "云舒",
    "青云", "明川", "松泉", "星洲", "晴波", "锦石", "竹风", "朝晖",
];

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateMnemonicRequest {
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub word_count: Option<usize>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateMnemonicResponse {
    pub mnemonic: String,
    pub language: String,
    pub word_count: usize,
}

pub fn generate_mnemonic(
    request: GenerateMnemonicRequest,
) -> std::result::Result<GenerateMnemonicResponse, String> {
    let language = normalize_language(request.language.as_deref())?;
    let word_count = request.word_count.unwrap_or(DEFAULT_WORD_COUNT);
    if !(MIN_WORD_COUNT..=MAX_WORD_COUNT).contains(&word_count) {
        return Err("mnemonic word count is out of range".to_string());
    }

    let words = match language.as_str() {
        "english" => ENGLISH_WORDS,
        "simplifiedChinese" => CHINESE_WORDS,
        _ => return Err("unsupported mnemonic language".to_string()),
    };

    let mut rng = OsRng;
    let selected = (0..word_count)
        .map(|_| words[random_index(words.len(), &mut rng)])
        .collect::<Vec<_>>();

    Ok(GenerateMnemonicResponse {
        mnemonic: selected.join(" "),
        language,
        word_count,
    })
}

fn normalize_language(language: Option<&str>) -> std::result::Result<String, String> {
    match language
        .unwrap_or("english")
        .trim()
        .replace(['_', '-'], "")
        .to_ascii_lowercase()
        .as_str()
    {
        "" | "en" | "english" => Ok("english".to_string()),
        "zh" | "zhcn" | "cn" | "chinese" | "simplifiedchinese" | "simplified" => {
            Ok("simplifiedChinese".to_string())
        }
        _ => Err("unsupported mnemonic language".to_string()),
    }
}

fn random_index(max: usize, rng: &mut OsRng) -> usize {
    let max = max as u64;
    let zone = u64::MAX - (u64::MAX % max);
    loop {
        let value = rng.next_u64();
        if value < zone {
            return (value % max) as usize;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::kdf::{derive_mnemonic_factor, normalize_mnemonic};
    use uuid::Uuid;

    #[test]
    fn generates_english_and_chinese_mnemonics() {
        let english = generate_mnemonic(GenerateMnemonicRequest {
            language: Some("english".to_string()),
            word_count: Some(20),
        })
        .unwrap();
        assert_eq!(english.language, "english");
        assert_eq!(english.mnemonic.split_whitespace().count(), 20);
        assert!(english.mnemonic.is_ascii());

        let chinese = generate_mnemonic(GenerateMnemonicRequest {
            language: Some("simplifiedChinese".to_string()),
            word_count: Some(20),
        })
        .unwrap();
        assert_eq!(chinese.language, "simplifiedChinese");
        assert_eq!(chinese.mnemonic.split_whitespace().count(), 20);
        assert!(!chinese.mnemonic.is_ascii());
    }

    #[test]
    fn mnemonic_normalization_supports_unicode_spacing() {
        assert_eq!(
            normalize_mnemonic("  Ａｎｃｈｏｒ　Bridge  "),
            "anchor bridge"
        );
        assert_eq!(normalize_mnemonic(" 山河　星辰  "), "山河 星辰");
    }

    #[test]
    fn chinese_mnemonic_derivation_is_stable() {
        let user_id = Uuid::new_v4();
        let salt = [7_u8; 16];
        let a = derive_mnemonic_factor("山河 星辰 清泉", &user_id, &salt).unwrap();
        let b = derive_mnemonic_factor("山河　星辰  清泉", &user_id, &salt).unwrap();
        assert_eq!(a, b);
    }
}
