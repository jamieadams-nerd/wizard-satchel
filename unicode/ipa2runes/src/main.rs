/*!
===============================================================================
ipa2runes — IPA → Runic Transliteration Tool
===============================================================================

Overview
--------
This program converts an input string written in IPA (International Phonetic
Alphabet) into a sequence of runic characters, primarily using letters from
the Anglo-Saxon runic tradition (commonly called the Futhorc).

The tool is designed as an educational and experimental encoding pipeline,
not as a historically strict reconstruction of Old English writing. Its goal
is to provide a deterministic and auditable mapping from phonetic input to
runic output for experimentation, visualization, and fun.

Example:

    Input IPA:  /naɪt/
    Tokens:     n + aɪ + t
    Output:     ᚾᚪᛁᛏ

This allows modern words to be represented using runes through phonetic
approximation.

-------------------------------------------------------------------------------
Historical Background
-------------------------------------------------------------------------------

Runes are letters from early Germanic writing systems used across Northern
Europe roughly between 150 CE and 1200 CE.

Multiple runic alphabets existed:

  • Elder Futhark – earliest common runic system (~24 letters)
  • Younger Futhark – Viking Age Scandinavian system (~16 letters)
  • Anglo-Saxon Futhorc – expanded system used in England

The Anglo-Saxon Futhorc expanded the rune inventory to represent additional
sounds found in Old English, particularly vowels. Because of this expansion,
it is better suited for representing modern phonetics than earlier runic
alphabets.

Unicode places all runic letters into a single RUNIC block, mixing symbols
from multiple traditions. This program explicitly tags runes by tradition
to make mapping decisions transparent.

-------------------------------------------------------------------------------
Purpose of This Tool
-------------------------------------------------------------------------------

This program treats runes as a phonetic encoding target rather than as a
strict historical writing system.

Goals:

  • Convert IPA phonemes to readable rune sequences
  • Show mapping decisions clearly
  • Allow experimentation with phonetic encodings
  • Provide verbose and auditable output
  • Support future extension into encoding pipelines

Non-goals:

  • Perfect historical accuracy
  • Reconstruction of Old English orthography
  • Linguistic scholarship tooling

This is an engineering tool and educational experiment.

-------------------------------------------------------------------------------
How Conversion Works
-------------------------------------------------------------------------------

The processing pipeline:

    IPA input
        ↓
    Normalize input
        ↓
    Tokenize into phonemes (greedy matching)
        ↓
    Map phonemes to runes
        ↓
    Produce runic output string

Tokenization identifies multi-character phonemes first, such as:

    tʃ   (church)
    dʒ   (judge)
    aɪ   (time)
    oʊ   (go)

Each phoneme is then mapped to:

  • a direct rune match,
  • a reasonable approximation,
  • or a sequence of runes.

Some sounds do not exist in historical rune inventories; these are handled
using approximation rules.

-------------------------------------------------------------------------------
Verbosity and Auditability
-------------------------------------------------------------------------------

Verbose mode explains:

  • input normalization
  • phoneme tokenization
  • rune mapping decisions
  • output rune inventory
  • mapping rationale

This makes the transliteration process transparent and easy to debug or
extend.

-------------------------------------------------------------------------------
Rune Metadata
-------------------------------------------------------------------------------

Each rune is stored with:

  • The rune glyph itself
  • Unicode code point
  • Traditional name
  • Historical tradition classification

Example:

    ᚠ (U+16A0 FEOH)

This prevents confusion when working across fonts or terminals.

-------------------------------------------------------------------------------
Limitations
-------------------------------------------------------------------------------

  • Modern English phonetics differ greatly from Old English.
  • Many sounds lack direct rune equivalents.
  • Results are approximate, not historical spellings.
  • Dialect differences may produce alternative outputs.

Despite these limitations, output remains readable and consistent.

-------------------------------------------------------------------------------
Future Directions
-------------------------------------------------------------------------------

Possible future improvements:

  • Text → IPA conversion
  • Mode selection (strict vs readable mappings)
  • JSON mapping definitions
  • JSON trace output
  • Extended rune inventories
  • Encoding visualization tools
  • Interactive transliteration utilities

-------------------------------------------------------------------------------
Summary
-------------------------------------------------------------------------------

ipa2runes demonstrates how historical writing systems can be explored as
phonetic encoding targets using modern tooling. It is intended as a fun,
educational, and extensible experiment rather than a scholarly instrument.

Enjoy experimenting with runes.

===============================================================================
*/

use std::env;

/// A rough “tradition” label for the rune we choose.
/// This is not Unicode’s classification—it's our explicit 
/// metadata for auditing the mapping.
///
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tradition {
    /// Common to the older runic tradition and 
    /// present in Anglo-Saxon Futhorc usage.
    FuthorcCore,

    /// Letters strongly associated with the 
    /// expanded Anglo-Saxon Futhorc inventory
    /// (Old English additions).
    AngloSaxonExtension,

    /// Not used in this minimal tool, but 
    /// included for completeness.
    ScandinavianVariant,

    /// Our mapping hack: no historically clean 
    /// rune exists; we approximate with
    /// something readable.
    Approximation,
}

/// Rune metadata: store BOTH the glyph and the 
/// Unicode code point to avoid ambiguity.
#[derive(Debug, Clone, Copy)]
struct RuneChar {
    ch: char,
    codepoint: u32,
    name: &'static str,
    tradition: Tradition,
}

impl RuneChar {
    const fn new(ch: char, codepoint: u32, name: &'static str, tradition: Tradition) -> Self {
        Self {
            ch,
            codepoint,
            name,
            tradition,
        }
    }
}

/// Helper: format a rune as "ᚠ(U+16A0 FEHU/FEOH)" etc.
fn fmt_rune(r: &RuneChar) -> String {
    format!(
        "{}\t(U+{:04X}  {:20} / {:?})",
        r.ch, r.codepoint, r.name, r.tradition
    )
}

/// A mapping result can be one rune or multiple 
/// runes (e.g., diphthongs, affricates).
#[derive(Debug, Clone)]
struct Mapping {
    runes: Vec<RuneChar>,
    rationale: &'static str,
}

/// Greedy token match for IPA parsing.
#[derive(Debug, Clone)]
struct TokenRule {
    ipa: &'static str,
    desc: &'static str,
}

///
/// Normalize IPA input:
///
/// - strip surrounding slashes /.../
/// - remove common stress markers ˈ ˌ
/// - remove spaces
/// - keep IPA symbols; keep ː (length) but we’ll treat it as ignorable
///
fn normalize_ipa(input: &str) -> String {
    let mut s = input.trim().to_string();
    if s.starts_with('/') && s.ends_with('/') && s.len() >= 2 {
        s = s[1..s.len() - 1].to_string();
    }
    s.chars()
        .filter(|&c| {
            c != 'ˈ'
                && c != 'ˌ'
                && !c.is_ascii_punctuation()
                && c != ' '
                && c != '\t'
                && c != '\n'
                && c != '\r'
        })
        .collect()
}

///
/// Tokenize IPA with greedy longest-match against a rule list.
/// Any unknown char becomes its own token so we can explain failures.
///
fn tokenize_ipa(ipa: &str, verbose: bool) -> Vec<String> {
    // Order matters: longer sequences must come before shorter ones.
    // This is a pragmatic starter set: expand as your project needs.
    let rules = [
        // Affricates
        TokenRule {
            ipa: "tʃ",
            desc: "voiceless postalveolar affricate (ch)",
        },
        TokenRule {
            ipa: "dʒ",
            desc: "voiced postalveolar affricate (j)",
        },
        TokenRule {
            ipa: "ʤ",
            desc: "voiced postalveolar affricate ligature (j)",
        },
        // Common English diphthongs
        TokenRule {
            ipa: "aɪ",
            desc: "diphthong (as in 'time')",
        },
        TokenRule {
            ipa: "eɪ",
            desc: "diphthong (as in 'day')",
        },
        TokenRule {
            ipa: "oʊ",
            desc: "diphthong (as in 'go')",
        },
        TokenRule {
            ipa: "aʊ",
            desc: "diphthong (as in 'now')",
        },
        TokenRule {
            ipa: "ɔɪ",
            desc: "diphthong (as in 'boy')",
        },
        // Common multichar consonant symbols
        TokenRule {
            ipa: "θ",
            desc: "voiceless dental fricative (th in 'thin')",
        },
        TokenRule {
            ipa: "ð",
            desc: "voiced dental fricative (th in 'this')",
        },
        TokenRule {
            ipa: "ŋ",
            desc: "velar nasal (ng in 'sing')",
        },
        TokenRule {
            ipa: "ʃ",
            desc: "voiceless postalveolar fricative (sh)",
        },
        TokenRule {
            ipa: "ʒ",
            desc: "voiced postalveolar fricative (zh)",
        },
        // Length marker (we treat it as ignorable token)
        TokenRule {
            ipa: "ː",
            desc: "length marker",
        },
    ];

    let mut out = Vec::new();
    let mut i = 0;
    while i < ipa.len() {
        let rest = &ipa[i..];

        // Try match any rule
        let mut matched: Option<&TokenRule> = None;
        for rule in &rules {
            if rest.starts_with(rule.ipa) {
                matched = Some(rule);
                break;
            }
        }

        if let Some(rule) = matched {
            out.push(rule.ipa.to_string());
            if verbose {
                eprintln!("[tokenize] matched {:?}: {}", rule.ipa, rule.desc);
            }
            i += rule.ipa.len();
        } else {
            // fallback: one Unicode scalar
            let ch = rest.chars().next().unwrap();
            out.push(ch.to_string());
            if verbose {
                eprintln!("[tokenize] fallback single char token: {:?}", ch);
            }
            i += ch.len_utf8();
        }
    }

    out
}

///
/// Rune constants (glyph + Unicode + friendly name).
///
/// Notes:
/// - Names here are Unicode-ish / commonly used runic names; treat them as
///   labels, not strict philology.
/// - Tradition tags are for our *decision audit*.
///
#[allow(dead_code)]
mod runes {
    use super::{RuneChar, Tradition};

    // Core-ish / widely used in Futhorc contexts
    pub const FEOH:  RuneChar = RuneChar::new('ᚠ', 0x16A0, "feoh (fehu)", Tradition::FuthorcCore);
    pub const UR:    RuneChar = RuneChar::new('ᚢ', 0x16A2, "ur", Tradition::FuthorcCore);
    pub const THORN: RuneChar = RuneChar::new('ᚦ', 0x16A6, "thorn", Tradition::FuthorcCore);

    // Old English distinct “os” form is often represented with ᚩ in Unicode (RUNI LETTER OS)
    pub const OS:   RuneChar = RuneChar::new('ᚩ', 0x16A9, "os", Tradition::AngloSaxonExtension);

    pub const RAD:  RuneChar = RuneChar::new('ᚱ', 0x16B1, "rad", Tradition::FuthorcCore);

    // Unicode distinguishes KAUNA/KAUN/CEN; for Futhorc transliteration we use CEN (ᚳ).
    pub const CEN:   RuneChar = RuneChar::new('ᚳ', 0x16B3, "cen", Tradition::AngloSaxonExtension);

    pub const GYFU:  RuneChar = RuneChar::new('ᚷ', 0x16B7, "gyfu", Tradition::FuthorcCore);
    pub const WYNN:  RuneChar = RuneChar::new('ᚹ', 0x16B9, "wynn", Tradition::AngloSaxonExtension);

    pub const HAEGL: RuneChar = RuneChar::new('ᚻ', 0x16BB, "haegl", Tradition::AngloSaxonExtension);
    pub const NYD:   RuneChar = RuneChar::new('ᚾ', 0x16BE, "nyd", Tradition::FuthorcCore);
    pub const IS:    RuneChar = RuneChar::new('ᛁ', 0x16C1, "is", Tradition::FuthorcCore);

    // Unicode has JERAN (ᛃ) and GER (ᛄ). Old English “ger” is often used;
    // we pick ᛄ for /j/ or /g/ palatal-ish.
    pub const GER: RuneChar = RuneChar::new('ᛄ', 0x16C4, "ger", Tradition::AngloSaxonExtension);

    pub const EOH: RuneChar = RuneChar::new('ᛇ', 0x16C7, "eoh (iwaz)", Tradition::FuthorcCore);
    pub const PEORTH: RuneChar = RuneChar::new('ᛈ', 0x16C8, "peorth", Tradition::FuthorcCore);
    pub const EOLHX: RuneChar = RuneChar::new('ᛉ', 0x16C9, "eolhx (algiz)", Tradition::FuthorcCore);

    // S: Unicode has SOWILO (ᛊ) and SIGEL (ᛋ). Old English rune name “sigel” corresponds to ᛋ.
    pub const SIGEL: RuneChar = RuneChar::new('ᛋ', 0x16CB, "sigel", Tradition::AngloSaxonExtension);

    pub const TIR: RuneChar = RuneChar::new('ᛏ', 0x16CF, "tir", Tradition::FuthorcCore);
    pub const BEORC: RuneChar = RuneChar::new('ᛒ', 0x16D2, "beorc", Tradition::AngloSaxonExtension);
    pub const EH: RuneChar = RuneChar::new('ᛖ', 0x16D6, "eh", Tradition::FuthorcCore);
    pub const MANN: RuneChar = RuneChar::new('ᛗ', 0x16D7, "mann", Tradition::FuthorcCore);
    pub const LAGU: RuneChar = RuneChar::new('ᛚ', 0x16DA, "lagu", Tradition::FuthorcCore);

    // /ŋ/ often maps to ING (ᛝ).
    pub const ING: RuneChar = RuneChar::new('ᛝ', 0x16DD, "ing", Tradition::FuthorcCore);

    pub const DAEG: RuneChar = RuneChar::new('ᛞ', 0x16DE, "daeg", Tradition::FuthorcCore);
    pub const ETHEL: RuneChar = RuneChar::new(
        'ᛟ',
        0x16DF,
        "ethel (othalan)",
        Tradition::AngloSaxonExtension,
    );

    // Vowels expanded in Old English
    pub const AC: RuneChar = RuneChar::new('ᚪ', 0x16AA, "ac", Tradition::AngloSaxonExtension);
    pub const AESC: RuneChar = RuneChar::new('ᚫ', 0x16AB, "aesc", Tradition::AngloSaxonExtension);

    // Additional Old English runes
    pub const YR: RuneChar = RuneChar::new('ᚣ', 0x16A3, "yr", Tradition::AngloSaxonExtension);
    pub const EAR: RuneChar = RuneChar::new('ᛠ', 0x16E0, "ear", Tradition::AngloSaxonExtension);
    pub const IOR: RuneChar = RuneChar::new('ᛡ', 0x16E1, "ior", Tradition::AngloSaxonExtension);

    // This is a *hack* rune for “sh”: Unicode has RUNIC LETTER SH (ᛲ) but it's a
    // later scholarly addition; we tag it as Approximation to keep the audit honest.
    pub const SH: RuneChar = RuneChar::new(
        'ᛲ',
        0x16F2,
        "sh (modern addition)",
        Tradition::Approximation,
    );

    // D: Unicode has RUNIC LETTER D (ᛑ) and DAGAZ/DAEG (ᛞ). We'll use ᛞ for /d/ for simplicity.
    // V is tricky; Unicode has RUNIC LETTER V (ᚡ), but it’s not a standard Futhorc “core” letter.
    pub const V: RuneChar = RuneChar::new('ᚡ', 0x16A1, "v (unicode)", Tradition::Approximation);
}

/// Map one IPA token to runes with rationale.
/// Deterministic defaults, auditable reasoning.
fn map_token(token: &str) -> Option<Mapping> {
    use runes::*;

    // Ignore length marker
    if token == "ː" {
        return Some(Mapping {
            runes: vec![],
            rationale:
                "IPA length marker ignored for rune output (no direct \
                rune length diacritic here).",
        });
    }

    // Multi-symbol tokens
    match token {
        "tʃ" => {
            // Choice: use CEN + SH (approx) for readability
            return Some(Mapping {
                runes: vec![CEN, SH],
                rationale: "Affricate /tʃ/ approximated as /t/~/k/ \
                           + /ʃ/: CEN + (approx) SH.",
            });
        }
        "dʒ" | "ʤ" => {
            return Some(Mapping {
                runes: vec![DAEG, SH],
                rationale: "Affricate /dʒ/ approximated as /d/ \
                           + /ʒ/~/ʃ/: DAEG + (approx) SH.",
            });
        }
        "aɪ" => {
            return Some(Mapping {
                runes: vec![AC, IS],
                rationale: "Diphthong /aɪ/ approximated as AC + IS (a + i).",
            });
        }
        "eɪ" => {
            return Some(Mapping {
                runes: vec![EH, IS],
                rationale: "Diphthong /eɪ/ approximated as EH + IS (e + i).",
            });
        }
        "oʊ" => {
            return Some(Mapping {
                runes: vec![OS, UR],
                rationale: "Diphthong /oʊ/ approximated as OS + UR (o + u).",
            });
        }
        "aʊ" => {
            return Some(Mapping {
                runes: vec![AC, UR],
                rationale: "Diphthong /aʊ/ approximated as AC + UR (a + u).",
            });
        }
        "ɔɪ" => {
            return Some(Mapping {
                runes: vec![OS, IS],
                rationale: "Diphthong /ɔɪ/ approximated as OS + IS (o + i).",
            });
        }
        _ => {}
    }

    // Single-symbol tokens (including IPA chars 
    // that are one Unicode scalar)
    match token {
        // Consonants
        "p" => Some(Mapping {
            runes: vec![PEORTH],
            rationale: "/p/ -> PEORTH (p).",
        }),
        "b" => Some(Mapping {
            runes: vec![BEORC],
            rationale: "/b/ -> BEORC (b).",
        }),
        "t" => Some(Mapping {
            runes: vec![TIR],
            rationale: "/t/ -> TIR (t).",
        }),
        "d" => Some(Mapping {
            runes: vec![DAEG],
            rationale: "/d/ -> DAEG (d) (using daeg form for simplicity).",
        }),
        "k" => Some(Mapping {
            runes: vec![CEN],
            rationale: "/k/ -> CEN (c/k) chosen as Futhorc-friendly form.",
        }),
        "g" => Some(Mapping {
            runes: vec![GYFU],
            rationale: "/g/ -> GYFU (g).",
        }),
        "f" => Some(Mapping {
            runes: vec![FEOH],
            rationale: "/f/ -> FEOH (f).",
        }),
        "v" => Some(Mapping {
            runes: vec![V],
            rationale: "/v/ -> Unicode V rune (approximation; not cleanly historical).",
        }),
        "θ" => Some(Mapping {
            runes: vec![THORN],
            rationale: "/θ/ -> THORN.",
        }),
        "ð" => Some(Mapping {
            runes: vec![THORN],
            rationale: "/ð/ -> THORN (same rune used for th-sounds here).",
        }),
        "s" => Some(Mapping {
            runes: vec![SIGEL],
            rationale: "/s/ -> SIGEL.",
        }),
        "z" => Some(Mapping {
            runes: vec![SIGEL],
            rationale: "/z/ -> SIGEL (approx; no dedicated z rune in this scheme).",
        }),
        "h" => Some(Mapping {
            runes: vec![HAEGL],
            rationale: "/h/ -> HAEGL.",
        }),
        "m" => Some(Mapping {
            runes: vec![MANN],
            rationale: "/m/ -> MANN.",
        }),
        "n" => Some(Mapping {
            runes: vec![NYD],
            rationale: "/n/ -> NYD.",
        }),
        "ŋ" => Some(Mapping {
            runes: vec![ING],
            rationale: "/ŋ/ -> ING.",
        }),
        "r" => Some(Mapping {
            runes: vec![RAD],
            rationale: "/r/ -> RAD.",
        }),
        "l" => Some(Mapping {
            runes: vec![LAGU],
            rationale: "/l/ -> LAGU.",
        }),
        "j" => Some(Mapping {
            runes: vec![GER],
            rationale: "/j/ -> GER (approx for y-sound; pragmatic mapping).",
        }),
        "w" => Some(Mapping {
            runes: vec![WYNN],
            rationale: "/w/ -> WYNN.",
        }),
        "ʃ" => Some(Mapping {
            runes: vec![SH],
            rationale: "/ʃ/ -> SH rune (approximation; later addition).",
        }),
        "ʒ" => Some(Mapping {
            runes: vec![SH],
            rationale: "/ʒ/ -> SH rune (approximation; closest readable option).",
        }),

        // Vowels (pragmatic defaults)
        "a" => Some(Mapping {
            runes: vec![AC],
            rationale: "/a/ -> AC (a).",
        }),
        "æ" => Some(Mapping {
            runes: vec![AESC],
            rationale: "/æ/ -> AESC (ash).",
        }),
        "e" => Some(Mapping {
            runes: vec![EH],
            rationale: "/e/ -> EH (e).",
        }),
        "ɛ" => Some(Mapping {
            runes: vec![EH],
            rationale: "/ɛ/ -> EH (approx; open-mid front vowel mapped to e-rune).",
        }),
        "i" => Some(Mapping {
            runes: vec![IS],
            rationale: "/i/ -> IS (i).",
        }),
        "ɪ" => Some(Mapping {
            runes: vec![IS],
            rationale: "/ɪ/ -> IS (approx; i-like vowel).",
        }),
        "o" => Some(Mapping {
            runes: vec![OS],
            rationale: "/o/ -> OS (o).",
        }),
        "ɔ" => Some(Mapping {
            runes: vec![OS],
            rationale: "/ɔ/ -> OS (approx; o-like vowel).",
        }),
        "u" => Some(Mapping {
            runes: vec![UR],
            rationale: "/u/ -> UR (u).",
        }),
        "ʊ" => Some(Mapping {
            runes: vec![UR],
            rationale: "/ʊ/ -> UR (approx; u-like vowel).",
        }),
        "ə" => Some(Mapping {
            runes: vec![EH],
            rationale: "/ə/ -> EH (approx schwa -> e as neutral vowel).",
        }),
        "ʌ" => Some(Mapping {
            runes: vec![AC],
            rationale: "/ʌ/ -> AC (approx; a-like central vowel).",
        }),

        // A couple extra OE-ish vowel runes you may want to play with
        "y" => Some(Mapping {
            runes: vec![YR],
            rationale: "/y/ -> YR (approx front rounded vowel; if present in your IPA).",
        }),

        _ => None,
    }
}

fn main() {
    let mut args: Vec<String> = env::args().collect();

    let prog = args.remove(0);
    if args.is_empty() {
        eprintln!("Usage:");
        eprintln!("  {prog} [--verbose] \"<IPA>\"");
        eprintln!("Examples:");
        eprintln!("  {prog} --verbose \"/naɪt/\"");
        eprintln!("  {prog} --verbose \"tʃɜːtʃ\"   # 'church' (approx)");
        std::process::exit(2);
    }

    let mut verbose = false;
    let mut ipa_input: Option<String> = None;

    for a in args {
        if a == "--verbose" || a == "-v" {
            verbose = true;
        } else if ipa_input.is_none() {
            ipa_input = Some(a);
        } else {
            // If user provided multiple non-flag args, 
            // join with space (rare, but safe).
            ipa_input = Some(format!("{} {}", ipa_input.unwrap(), a));
        }
    }

    let ipa_input = ipa_input.unwrap();
    let normalized = normalize_ipa(&ipa_input);

    if verbose {
        eprintln!("[input] raw:        {:?}", ipa_input);
        eprintln!("[input] normalized: {:?}", normalized);
        eprintln!("[note] mapping goal: deterministic IPA -> \
                   rune string (audited), not strict historical orthography.");
        eprintln!();
    }

    let tokens = tokenize_ipa(&normalized, verbose);

    if verbose {
        eprintln!();
        eprintln!("[tokens] {:?}", tokens);
        eprintln!();
    }

    let mut out_runes: Vec<crate::RuneChar> = Vec::new();
    let mut unknown: Vec<String> = Vec::new();

    for (idx, tok) in tokens.iter().enumerate() {
        if verbose {
            eprintln!("[map] token[{idx}] = {:?}", tok);
        }

        match map_token(tok) {
            Some(mapping) => {
                if verbose {
                    if mapping.runes.is_empty() {
                        eprintln!("      -> (no output) {}", mapping.rationale);
                    } else {
                        let rendered: Vec<String> = mapping.runes.iter().map(fmt_rune).collect();
                        eprintln!("      -> {}", rendered.join(" + "));
                        eprintln!("         rationale: {}", mapping.rationale);
                    }
                }
                out_runes.extend(mapping.runes);
            }
            None => {
                unknown.push(tok.clone());
                if verbose {
                    eprintln!("      -> [unmapped] no rule for token {:?}.", tok);
                }
            }
        }

        if verbose {
            eprintln!();
        }
    }

    // Build final rune string
    let rune_string: String = out_runes
        .iter()
        .map(|r| r.ch.to_string())
        .collect::<Vec<_>>()
        .join(" ");

    // Summary / audit
    if verbose {
        eprintln!("[audit] output rune count: {}", out_runes.len());
        if !unknown.is_empty() {
            eprintln!("[audit] unmapped tokens: {:?}", unknown);
            eprintln!("[audit] suggestion: extend the token rules \
                       map_token() table for these IPA symbols.");
        }

        // Show per-rune details
        eprintln!();
        eprintln!("[audit] per-rune output details:");
        for (i, r) in out_runes.iter().enumerate() {
            eprintln!("  [ {i:03} ] {} ", fmt_rune(r));
        }
        eprintln!();
        eprintln!("[output] rune string: ");
    }

    println!("Here it is represented in Runes:\n");
    println!(" {} \n", rune_string);

    if !unknown.is_empty() {

        // Non-verbose still signals it with an exit 
        // code and message (useful in pipelines).
        eprintln!("Warning: unmapped IPA tokens: {:?}", unknown);

        // Exit code 1 indicates partial success 
        // (output produced, but incomplete mapping).
        std::process::exit(1);
    }
}
