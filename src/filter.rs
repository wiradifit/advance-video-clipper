use regex::Regex;
use std::sync::OnceLock;

// Comprehensive Social Media Trigger & Profanity Filter List (Bahasa Indonesia + English)
pub static SENSITIVE_DICTIONARY: &[(&str, &str)] = &[
    // Bahasa Indonesia Trigger & Moderation Words
    ("mati", "M*TI"),
    ("kematian", "KEM*TIAN"),
    ("bunuh", "B*NUH"),
    ("pembunuh", "PEMB*NUH"),
    ("pembunuhan", "PEMB*NUHAN"),
    ("darah", "D*RAH"),
    ("mayat", "M*YAT"),
    ("jenazah", "JEN*ZAH"),
    ("gantung", "GANT*NG"),
    ("perkosa", "PERK*SA"),
    ("pemerkosaan", "PEMERK*SAAN"),
    ("tumbal", "TUMB*L"),
    ("racun", "R*CUN"),
    ("santet", "S*NTET"),
    ("setan", "S*TAN"),
    ("iblis", "IBL*S"),
    ("dajjal", "DAJJ*L"),
    ("babi", "B*BI"),
    ("anjing", "ANJ*NG"),
    ("bangsat", "BANGS*T"),
    ("kontol", "KONT*L"),
    ("memek", "MEM*K"),
    ("ngentot", "NGENT*T"),
    ("peler", "PEL*R"),
    ("itil", "IT*L"),
    ("jembut", "JEMB*T"),
    ("taik", "TA*K"),
    ("tahi", "TA*I"),

    // English Sensitive & Profanity Words
    ("dead", "D*AD"),
    ("death", "D*ATH"),
    ("die", "D*E"),
    ("died", "D*ED"),
    ("kill", "K*LL"),
    ("killed", "K*LLED"),
    ("killer", "K*LLER"),
    ("killing", "K*LLING"),
    ("murder", "M*RDER"),
    ("murderer", "M*RDERER"),
    ("suicide", "S*ICIDE"),
    ("blood", "BL*OD"),
    ("bloody", "BL*ODY"),
    ("corpse", "C*RPSE"),
    ("rape", "R*PE"),
    ("poison", "P*ISON"),
    ("fuck", "F*CK"),
    ("fucking", "F*CKING"),
    ("fucked", "F*CKED"),
    ("shit", "SH*T"),
    ("bitch", "B*TCH"),
    ("asshole", "ASS*OLE"),
    ("bastard", "BAST*RD"),
    ("cunt", "C*NT"),
    ("dick", "D*CK"),
    ("pussy", "P*SSY"),
    ("cock", "C*CK"),
];

static REGEX_CACHE: OnceLock<Vec<(Regex, &'static str)>> = OnceLock::new();

fn get_compiled_regexes() -> &'static Vec<(Regex, &'static str)> {
    REGEX_CACHE.get_or_init(|| {
        SENSITIVE_DICTIONARY
            .iter()
            .map(|(word, replacement)| {
                let pat = format!(r"(?i)\b{}\b", regex::escape(word));
                (Regex::new(&pat).unwrap(), *replacement)
            })
            .collect()
    })
}

/// Applies Social Media Content Moderation censorship to text.
/// Replaces sensitive triggering words with asterisks (e.g., MATI -> M*TI, KILL -> K*LL).
pub fn filter_sensitive_text(input: &str) -> String {
    let mut result = input.to_string();
    let compiled = get_compiled_regexes();

    for (re, replacement) in compiled {
        if re.is_match(&result) {
            result = re
                .replace_all(&result, |caps: &regex::Captures| {
                    let matched = &caps[0];
                    if matched.chars().all(|c| c.is_uppercase()) {
                        replacement.to_uppercase()
                    } else if matched.chars().next().map_or(false, |c| c.is_uppercase()) {
                        let mut chars = replacement.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                        }
                    } else {
                        replacement.to_lowercase()
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
    fn test_indonesian_filtering() {
        assert_eq!(filter_sensitive_text("Orang itu mati tadi malam"), "Orang itu m*ti tadi malam");
        assert_eq!(filter_sensitive_text("MATI DI TEMPAT"), "M*TI DI TEMPAT");
        assert_eq!(filter_sensitive_text("Ada tumbal darah proyek"), "Ada tumb*l d*rah proyek");
    }

    #[test]
    fn test_english_filtering() {
        assert_eq!(filter_sensitive_text("He was killed instantly"), "He was k*lled instantly");
        assert_eq!(filter_sensitive_text("DEAD ON ARRIVAL"), "D*AD ON ARRIVAL");
    }
}
