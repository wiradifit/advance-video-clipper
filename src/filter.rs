use regex::Regex;
use std::sync::OnceLock;

/// Social Media Content Moderation Filter for Bahasa Indonesia & English.
///
/// Sources:
///   - LDNOOBW/List-of-Dirty-Naughty-Obscene-and-Otherwise-Bad-Words (GitHub)
///   - coffee-and-fun/google-profanity-words (GitHub)
///   - drizki/indonesian-badwords (GitHub, 200+ terms)
///   - asruldev/kasar (GitHub)
///   - SideeID/id-profanity-filter (GitHub)
///   - TikTok Community Guidelines & YouTube Advertiser-Friendly Content Guidelines

// Comprehensive dictionary of words that trigger shadowbans, demonetization,
// or content suppression on TikTok, YouTube Shorts, and Instagram Reels.
// Each entry: (raw_word_lowercase)
// The censor function auto-generates the masked form: first letter + asterisks + last letter.
pub static FILTERED_WORDS: &[&str] = &[
    // ═══════════════════════════════════════════════════════════════════
    // BAHASA INDONESIA — Violence & Death Triggers
    // ═══════════════════════════════════════════════════════════════════
    "mati",
    "kematian",
    "bunuh",
    "membunuh",
    "pembunuh",
    "pembunuhan",
    "dibunuh",
    "terbunuh",
    "darah",
    "berdarah",
    "pendarahan",
    "mayat",
    "jenazah",
    "gantung",
    "menggantung",
    "racun",
    "meracuni",
    "tumbal",
    "korban",
    "mutilasi",
    "kubur",
    "mengubur",
    "kuburan",
    "neraka",
    "siksa",
    "penyiksaan",

    // ═══════════════════════════════════════════════════════════════════
    // BAHASA INDONESIA — Occult / Superstition Triggers
    // ═══════════════════════════════════════════════════════════════════
    "santet",
    "setan",
    "iblis",
    "dajjal",
    "kerasukan",
    "kesurupan",
    "guna-guna",
    "pesugihan",
    "pocong",
    "kuntilanak",
    "dukun",
    "jimat",
    "susuk",

    // ═══════════════════════════════════════════════════════════════════
    // BAHASA INDONESIA — Profanity & Vulgar (drizki/indonesian-badwords)
    // ═══════════════════════════════════════════════════════════════════
    "anjing",
    "anjir",
    "asu",
    "babi",
    "bacot",
    "bagong",
    "bajingan",
    "banci",
    "bangke",
    "bangsat",
    "bedebah",
    "bego",
    "bejat",
    "bencong",
    "biadab",
    "blegug",
    "bodoh",
    "bokep",
    "brengsek",
    "budeg",
    "burik",
    "cangkemu",
    "celeng",
    "cocot",
    "cuk",
    "cuki",
    "cukimai",
    "dancuk",
    "diamputt",
    "dongo",
    "dungu",
    "edan",
    "geblek",
    "goblok",
    "idiot",
    "itil",
    "jablay",
    "jancok",
    "jancuk",
    "jangkrik",
    "jembut",
    "kampang",
    "kampret",
    "kampungan",
    "keparat",
    "kimak",
    "kontol",
    "kunyuk",
    "lonte",
    "mampus",
    "memek",
    "monyet",
    "ngentot",
    "ngewe",
    "nyepong",
    "onta",
    "pantek",
    "pecun",
    "pelakor",
    "pelacur",
    "peler",
    "perek",
    "puki",
    "pukimak",
    "sange",
    "sempak",
    "sinting",
    "sialan",
    "sompret",
    "sontoloyo",
    "sundal",
    "tai",
    "taik",
    "tahi",
    "tetek",
    "titit",
    "tolol",

    // ═══════════════════════════════════════════════════════════════════
    // BAHASA INDONESIA — Self-harm / Harm Triggers
    // ═══════════════════════════════════════════════════════════════════
    "perkosa",
    "memperkosa",
    "pemerkosaan",
    "narkoba",
    "narkotika",
    "ganja",
    "sabu",
    "overdosis",

    // ═══════════════════════════════════════════════════════════════════
    // ENGLISH — Violence & Death (LDNOOBW + Google Profanity + TikTok)
    // ═══════════════════════════════════════════════════════════════════
    "dead",
    "death",
    "die",
    "died",
    "dying",
    "kill",
    "killed",
    "killer",
    "killing",
    "murder",
    "murdered",
    "murderer",
    "suicide",
    "suicidal",
    "blood",
    "bloody",
    "bloodbath",
    "corpse",
    "massacre",
    "slaughter",
    "strangle",
    "stabbed",
    "decapitate",
    "dismember",
    "mutilate",
    "torture",
    "tortured",
    "genocide",
    "homicide",
    "manslaughter",

    // ═══════════════════════════════════════════════════════════════════
    // ENGLISH — Sexual / Adult (LDNOOBW + Google Profanity)
    // ═══════════════════════════════════════════════════════════════════
    "anal",
    "anus",
    "ass",
    "asshole",
    "bastard",
    "bitch",
    "bitches",
    "blowjob",
    "boob",
    "boobs",
    "boner",
    "brothel",
    "bukkake",
    "bullshit",
    "butt",
    "butthole",
    "clitoris",
    "clit",
    "cock",
    "cocks",
    "cum",
    "cumming",
    "cumshot",
    "cunt",
    "dick",
    "dicks",
    "dildo",
    "ejaculation",
    "erection",
    "erotic",
    "fellatio",
    "foreskin",
    "fuck",
    "fucked",
    "fucker",
    "fucking",
    "gangbang",
    "genitals",
    "handjob",
    "hentai",
    "hooker",
    "horny",
    "incest",
    "jerk off",
    "masturbate",
    "masturbation",
    "milf",
    "nipple",
    "nipples",
    "nude",
    "nudity",
    "orgasm",
    "orgy",
    "penis",
    "phallus",
    "porn",
    "porno",
    "pornography",
    "prostitute",
    "prostitution",
    "pussy",
    "rape",
    "raped",
    "rapist",
    "rectum",
    "scrotum",
    "semen",
    "sex",
    "sexual",
    "shit",
    "shitty",
    "slut",
    "sluts",
    "smut",
    "sodomy",
    "sperm",
    "stripper",
    "testicle",
    "testicles",
    "tits",
    "titties",
    "twat",
    "vagina",
    "vibrator",
    "vulva",
    "whore",
    "whorehouse",
    "xxx",

    // ═══════════════════════════════════════════════════════════════════
    // ENGLISH — Hate Speech / Slurs (LDNOOBW + Content Moderation)
    // ═══════════════════════════════════════════════════════════════════
    "chink",
    "coon",
    "coons",
    "darkie",
    "dyke",
    "fag",
    "faggot",
    "gringo",
    "honkey",
    "kike",
    "negro",
    "nigga",
    "nigger",
    "niggers",
    "retard",
    "retarded",
    "spic",
    "tranny",
    "wetback",

    // ═══════════════════════════════════════════════════════════════════
    // ENGLISH — Drugs / Self-Harm (TikTok + YouTube Policy)
    // ═══════════════════════════════════════════════════════════════════
    "cocaine",
    "heroin",
    "meth",
    "methamphetamine",
    "overdose",
    "opioid",
    "fentanyl",
    "crack",
    "ecstasy",
    "ketamine",
    "marijuana",
    "weed",
    "self-harm",
    "cutting",
    "anorexia",
    "bulimia",
    "eating disorder",

    // ═══════════════════════════════════════════════════════════════════
    // ENGLISH — Weapons / Extremism (YouTube Policy)
    // ═══════════════════════════════════════════════════════════════════
    "bomb",
    "bombing",
    "terrorist",
    "terrorism",
    "extremist",
    "jihad",
    "shootout",
    "shooting",
    "gunshot",
];

static REGEX_CACHE: OnceLock<Vec<(Regex, String)>> = OnceLock::new();

/// Auto-generates the censored form: first letter + asterisks + last letter
/// Examples: "mati" → "m**i", "bunuh" → "b***h", "dead" → "d**d", "fuck" → "f**k"
fn generate_censored(word: &str) -> String {
    let chars: Vec<char> = word.chars().collect();
    if chars.len() <= 2 {
        return format!("{}*", chars[0]);
    }
    let first = chars[0];
    let last = chars[chars.len() - 1];
    let stars = "*".repeat(chars.len() - 2);
    format!("{}{}{}", first, stars, last)
}

fn get_compiled_regexes() -> &'static Vec<(Regex, String)> {
    REGEX_CACHE.get_or_init(|| {
        FILTERED_WORDS
            .iter()
            .map(|word| {
                let pat = format!(r"(?i)\b{}\b", regex::escape(word));
                let censored = generate_censored(word);
                (Regex::new(&pat).unwrap(), censored)
            })
            .collect()
    })
}

/// Applies Social Media Content Moderation censorship to text.
/// Replaces sensitive triggering words with first+last letter masking.
/// e.g. MATI → M**I, DARAH → D***H, BUNUH → B***H, KILL → K**L, DEAD → D**D
pub fn filter_sensitive_text(input: &str) -> String {
    let mut result = input.to_string();
    let compiled = get_compiled_regexes();

    for (re, censored) in compiled {
        if re.is_match(&result) {
            result = re
                .replace_all(&result, |caps: &regex::Captures| {
                    let matched = &caps[0];
                    if matched.chars().all(|c| c.is_uppercase() || !c.is_alphabetic()) {
                        censored.to_uppercase()
                    } else if matched.chars().next().map_or(false, |c| c.is_uppercase()) {
                        let mut chars = censored.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(first) => {
                                first.to_uppercase().collect::<String>() + chars.as_str()
                            }
                        }
                    } else {
                        censored.to_lowercase()
                    }
                })
                .to_string();
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_censored_generation() {
        assert_eq!(generate_censored("mati"), "m**i");
        assert_eq!(generate_censored("bunuh"), "b***h");
        assert_eq!(generate_censored("dead"), "d**d");
        assert_eq!(generate_censored("fuck"), "f**k");
        assert_eq!(generate_censored("pembunuhan"), "p********n");
        assert_eq!(generate_censored("ass"), "a*s");
    }

    #[test]
    fn test_indonesian_filtering() {
        assert_eq!(filter_sensitive_text("Orang itu mati tadi malam"), "Orang itu m**i tadi malam");
        assert_eq!(filter_sensitive_text("MATI DI TEMPAT"), "M**I DI TEMPAT");
        assert_eq!(filter_sensitive_text("Ada tumbal darah proyek"), "Ada t****l d***h proyek");
    }

    #[test]
    fn test_english_filtering() {
        assert_eq!(filter_sensitive_text("He was killed instantly"), "He was k****d instantly");
        assert_eq!(filter_sensitive_text("DEAD ON ARRIVAL"), "D**D ON ARRIVAL");
        assert_eq!(filter_sensitive_text("What the fuck"), "What the f**k");
    }
}
