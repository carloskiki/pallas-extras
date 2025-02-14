//! # BIP39 Mnemonic Codes
//!
//! Library crate implementing [BIP39](https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki)
//!

mod language;

use core::{error, fmt, str};
use sha2::{Digest, Sha256};

pub use language::Language;

use crate::entropy::Entropy;

/// The minimum number of words in a mnemonic.
const MIN_NB_WORDS: usize = 12;

/// The maximum number of words in a mnemonic.
const MAX_NB_WORDS: usize = 24;

/// The index used to indicate the mnemonic ended.
const EOF: u16 = u16::MAX;

/// A structured used in the [Error::AmbiguousLanguages] variant that iterates
/// over the possible languages.
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct AmbiguousLanguages([bool; language::MAX_NB_LANGUAGES]);

impl AmbiguousLanguages {
    /// Presents the possible languages in the form of a slice of booleans
    /// that correspond to the occurrences in [Language::ALL].
    pub fn as_bools(&self) -> &[bool; language::MAX_NB_LANGUAGES] {
        &self.0
    }

    /// An iterator over the possible languages.
    pub fn iter(&self) -> impl Iterator<Item = Language> + '_ {
        Language::ALL
            .iter()
            .enumerate()
            .filter(move |(i, _)| self.0[*i])
            .map(|(_, l)| *l)
    }
}

/// A BIP39 error.
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Error {
    /// Mnemonic has a word count that is not a multiple of 3.
    BadWordCount(usize),
    /// Mnemonic contains an unknown word.
    /// Error contains the index of the word.
    /// Use `mnemonic.split_whitespace().get(i)` to get the word.
    UnknownWord(usize),
    /// The mnemonic has an invalid checksum.
    InvalidChecksum,
    /// The mnemonic can be interpreted as multiple languages.
    /// Use the helper methods of the inner struct to inspect
    /// which languages are possible.
    AmbiguousLanguages(AmbiguousLanguages),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::BadWordCount(c) => {
                write!(
                    f,
                    "mnemonic has an invalid word count: {}. Word count must be 12, 15, 18, 21, \
					or 24",
                    c
                )
            }
            Error::UnknownWord(i) => write!(f, "mnemonic contains an unknown word (word {})", i,),
            Error::InvalidChecksum => write!(f, "the mnemonic has an invalid checksum"),
            Error::AmbiguousLanguages(a) => {
                write!(f, "ambiguous word list: ")?;
                for (i, lang) in a.iter().enumerate() {
                    if i == 0 {
                        write!(f, "{}", lang)?;
                    } else {
                        write!(f, ", {}", lang)?;
                    }
                }
                Ok(())
            }
        }
    }
}

impl error::Error for Error {}

/// A mnemonic code.
///
/// The [core::str::FromStr] implementation will try to determine the language of the
/// mnemonic from all the supported languages. (Languages have to be explicitly enabled using
/// the Cargo features.)
///
/// Supported number of words are 12, 15, 18, 21, and 24.
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Mnemonic {
    /// The language the mnemonic.
    lang: Language,
    /// The indices of the words.
    /// Mnemonics with less than the max nb of words are terminated with EOF.
    words: [u16; MAX_NB_WORDS],
}

impl Mnemonic {
    /// Create a new [Mnemonic] in the specified language from the given entropy.
    /// Entropy must be a multiple of 32 bits (4 bytes) and 128-256 bits in length.
    pub fn from_entropy_in(lang: Language, entropy: Entropy) -> Mnemonic {
        let nb_bytes = entropy.as_ref().len();
        let check = Sha256::digest(entropy.as_ref());
        let checksum = check[0] >> (8 - nb_bytes / 4);

        let mut words = [0; MAX_NB_WORDS];
        let mut cursor = 0;
        let mut offset: u8 = 0;
        entropy.as_ref().iter().for_each(|byte| {
            if offset > 3 {
                words[cursor] <<= 8 - (offset - 3);
                words[cursor] |= *byte as u16 >> (offset - 3);
                cursor += 1;
                words[cursor] = *byte as u16 & ((1 << (offset - 3)) - 1);
                offset -= 3;
            } else {
                words[cursor] <<= 8;
                words[cursor] |= *byte as u16;
                offset += 8;
            }
        });
        words[cursor] <<= 11 - offset;
        words[cursor] |= checksum as u16;
        words[cursor + 1..].fill(EOF);

        Mnemonic { lang, words }
    }

    /// Create a new English [`Mnemonic`] from the given [`Entropy`].
    pub fn from_entropy(entropy: Entropy) -> Mnemonic {
        Mnemonic::from_entropy_in(Language::English, entropy)
    }

    /// Get the language of the [Mnemonic].
    pub fn language(&self) -> Language {
        self.lang
    }

    /// Returns an iterator over the words of the [Mnemonic].
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use ponk::bip39::Mnemonic;
    /// use ponk::entropy::Entropy;
    ///
    /// let mnemonic = Mnemonic::from_entropy(Entropy::from([0; 32]));
    /// for (i, word) in mnemonic.words().enumerate() {
    ///     println!("{}. {}", i, word);
    /// }
    /// ```
    pub fn words(&self) -> impl Iterator<Item = &'static str> + Clone + '_ {
        let list = self.lang.word_list();
        self.word_indices().map(move |i| list[i])
    }

    /// Returns an iterator over [Mnemonic] word indices.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use ponk::bip39::{Language, Mnemonic};
    /// use ponk::entropy::Entropy;
    ///
    /// let list = Language::English.word_list();
    /// let mnemonic = Mnemonic::from_entropy(Entropy::from([0; 32]));
    /// for (i, word) in mnemonic.word_indices().zip(mnemonic.words()) {
    ///     assert_eq!(list[i], word);
    /// }
    /// ```
    pub fn word_indices(&self) -> impl Iterator<Item = usize> + Clone + '_ {
        self.words
            .iter()
            .take_while(|&&w| w != EOF)
            .map(|w| *w as usize)
    }

    /// Determine the language of the mnemonic.
    ///
    /// NOTE: This method only guarantees that the returned language is the
    /// correct language on the assumption that the mnemonic is valid.
    /// It does not itself validate the mnemonic.
    ///
    /// Some word lists don't guarantee that their words don't occur in other
    /// word lists. In the extremely unlikely case that a word list can be
    /// interpreted in multiple languages, an [Error::AmbiguousLanguages] is
    /// returned, containing the possible languages.
    pub fn language_of<S: AsRef<str>>(mnemonic: S) -> Result<Language, Error> {
        let mut words = mnemonic.as_ref().split_whitespace().peekable();
        let langs = Language::ALL;
        {
            // Start scope to drop first_word so that words can be reborrowed later.
            let first_word = words.peek().ok_or(Error::BadWordCount(0))?;
            if first_word.is_empty() {
                return Err(Error::BadWordCount(0));
            }

            // We first try find the first word in wordlists that
            // have guaranteed unique words.
            for language in langs.iter().filter(|l| l.unique_words()) {
                if language.find_word(first_word).is_some() {
                    return Ok(*language);
                }
            }
        }

        // If that didn't work, we start with all possible languages
        // (those without unique words), and eliminate until there is
        // just one left.
        let mut possible = [false; language::MAX_NB_LANGUAGES];
        for (i, lang) in langs.iter().enumerate() {
            // To start, only consider lists that don't have unique words.
            // Those were considered above.
            possible[i] = !lang.unique_words();
        }
        for (idx, word) in words.enumerate() {
            // Scrap languages that don't have this word.
            for (i, lang) in langs.iter().enumerate() {
                possible[i] &= lang.find_word(word).is_some();
            }

            // Get an iterator over remaining possible languages.
            let mut iter = possible
                .iter()
                .zip(langs.iter())
                .filter(|(p, _)| **p)
                .map(|(_, l)| l);

            match iter.next() {
                // If all languages were eliminated, it's an invalid word.
                None => return Err(Error::UnknownWord(idx)),
                // If not, see if there is a second one remaining.
                Some(remaining) => {
                    if iter.next().is_none() {
                        // No second remaining, we found our language.
                        return Ok(*remaining);
                    }
                }
            }
        }

        Err(Error::AmbiguousLanguages(AmbiguousLanguages(possible)))
    }

    /// Parse a mnemonic in normalized UTF8 in the given language.
    pub fn parse_in_normalized(lang: Language, s: &str) -> Result<Mnemonic, Error> {
        let mnemonic = Mnemonic::parse_in_normalized_without_checksum_check(lang, s)?;
        if mnemonic.checksum()
            != Sha256::digest(mnemonic.to_entropy())[0] >> (8 - mnemonic.word_count() / 3)
        {
            return Err(Error::InvalidChecksum);
        }
        Ok(mnemonic)
    }

    /// Parse a mnemonic in normalized UTF8 in the given language without checksum check.
    ///
    /// It is advised to use this method together with the utility methods
    /// - [Mnemonic::normalize_utf8_cow]
    /// - [Mnemonic::language_of]
    pub fn parse_in_normalized_without_checksum_check(
        language: Language,
        s: &str,
    ) -> Result<Mnemonic, Error> {
        let nb_words = s.split_whitespace().count();
        if is_invalid_word_count(nb_words) {
            return Err(Error::BadWordCount(nb_words));
        }

        // Here we will store the eventual words.
        let mut words = [EOF; MAX_NB_WORDS];

        for (i, word) in s.split_whitespace().enumerate() {
            let idx = language.find_word(word).ok_or(Error::UnknownWord(i))?;

            words[i] = idx;
        }

        Ok(Mnemonic {
            lang: language,
            words,
        })
    }

    /// Parse a mnemonic in normalized UTF8.
    pub fn parse_normalized(s: &str) -> Result<Mnemonic, Error> {
        let lang = Mnemonic::language_of(s)?;
        Mnemonic::parse_in_normalized(lang, s)
    }

    /// Get the number of words in the mnemonic.
    pub fn word_count(&self) -> usize {
        self.word_indices().count()
    }

    /// Convert the mnemonic back to the entropy used to generate it.
    /// The return value is a byte array and the size.
    pub fn to_entropy(&self) -> Entropy {
        // Preallocate enough space for the longest possible word list
        let mut entropy = [0; 32];
        let mut cursor = 0;
        let mut offset = 0;
        let mut remainder = 0;

        for word in self.words() {
            let idx = self.lang.find_word(word).expect("invalid mnemonic");

            remainder |= ((idx as u32) << (32 - 11)) >> offset;
            offset += 11;

            while offset >= 8 && cursor < 32 {
                entropy[cursor] = (remainder >> 24) as u8;
                cursor += 1;
                remainder <<= 8;
                offset -= 8;
            }
        }

        Entropy(entropy, cursor as u8)
    }

    /// Return checksum value for the Mnemonic.
    ///
    /// The checksum value is the numerical value of the first `self.word_count() / 3` bits of the
    /// [SHA256](https://en.wikipedia.org/wiki/SHA-2) digest of the Mnemonic's entropy, and is
    /// encoded by the last word of the mnemonic sentence.
    pub fn checksum(&self) -> u8 {
        let word_count = self.word_count();
        let last_word = self.words[word_count - 1];
        let mask = 0xFF >> (8 - word_count / 3);
        last_word as u8 & mask
    }
}

impl fmt::Display for Mnemonic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let space = if self.language() == Language::Japanese {
            "\u{3000}"
        } else {
            " "
        };
        let mut words = self.words();
        f.write_str(words.next().unwrap())?;
        words.try_for_each(|word| write!(f, "{}{}", space, word))
    }
}

impl str::FromStr for Mnemonic {
    type Err = Error;

    fn from_str(s: &str) -> Result<Mnemonic, Error> {
        Mnemonic::parse_normalized(s)
    }
}

fn is_invalid_word_count(word_count: usize) -> bool {
    word_count < MIN_NB_WORDS || word_count % 3 != 0 || word_count > MAX_NB_WORDS
}

#[cfg(test)]
mod tests {
    use const_hex::FromHex;
    use rand::random;

    use super::*;

    #[test]
    fn test_language_of() {
        for lang in Language::ALL {
            let entropy = Entropy::from(&random::<[_; 24]>());
            let m = Mnemonic::from_entropy_in(*lang, entropy);
            assert_eq!(*lang, Mnemonic::language_of(m.to_string()).unwrap());
        }
    }

    #[test]
    fn test_ambiguous_languages() {
        let mut present = [false; language::MAX_NB_LANGUAGES];
        let mut present_vec = Vec::new();
        let mut alternate = true;
        (0..Language::ALL.len()).for_each(|i| {
            present[i] = alternate;
            if alternate {
                present_vec.push(Language::ALL[i]);
            }
            alternate = !alternate;
        });
        let amb = AmbiguousLanguages(present);
        assert_eq!(amb.iter().collect::<Vec<_>>(), present_vec);
    }

    #[test]
    fn test_vectors_english() {
        // These vectors are tuples of
        // (entropy, mnemonic)
        let test_vectors = [
			(
				"00000000000000000000000000000000",
				"abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
			),
			(
				"7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f",
				"legal winner thank year wave sausage worth useful legal winner thank yellow",
			),
			(
				"80808080808080808080808080808080",
				"letter advice cage absurd amount doctor acoustic avoid letter advice cage above",
			),
			(
				"ffffffffffffffffffffffffffffffff",
				"zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong",
			),
			(
				"000000000000000000000000000000000000000000000000",
				"abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon agent",
			),
			(
				"7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f",
				"legal winner thank year wave sausage worth useful legal winner thank year wave sausage worth useful legal will",
			),
			(
				"808080808080808080808080808080808080808080808080",
				"letter advice cage absurd amount doctor acoustic avoid letter advice cage absurd amount doctor acoustic avoid letter always",
			),
			(
				"ffffffffffffffffffffffffffffffffffffffffffffffff",
				"zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo when",
			),
			(
				"0000000000000000000000000000000000000000000000000000000000000000",
				"abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art",
			),
			(
				"7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f",
				"legal winner thank year wave sausage worth useful legal winner thank year wave sausage worth useful legal winner thank year wave sausage worth title",
			),
			(
				"8080808080808080808080808080808080808080808080808080808080808080",
				"letter advice cage absurd amount doctor acoustic avoid letter advice cage absurd amount doctor acoustic avoid letter advice cage absurd amount doctor acoustic bless",
			),
			(
				"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
				"zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo vote",
			),
			(
				"9e885d952ad362caeb4efe34a8e91bd2",
				"ozone drill grab fiber curtain grace pudding thank cruise elder eight picnic",
			),
			(
				"6610b25967cdcca9d59875f5cb50b0ea75433311869e930b",
				"gravity machine north sort system female filter attitude volume fold club stay feature office ecology stable narrow fog",
			),
			(
				"68a79eaca2324873eacc50cb9c6eca8cc68ea5d936f98787c60c7ebc74e6ce7c",
				"hamster diagram private dutch cause delay private meat slide toddler razor book happy fancy gospel tennis maple dilemma loan word shrug inflict delay length",
			),
			(
				"c0ba5a8e914111210f2bd131f3d5e08d",
				"scheme spot photo card baby mountain device kick cradle pact join borrow",
			),
			(
				"6d9be1ee6ebd27a258115aad99b7317b9c8d28b6d76431c3",
				"horn tenant knee talent sponsor spell gate clip pulse soap slush warm silver nephew swap uncle crack brave",
			),
			(
				"9f6a2878b2520799a44ef18bc7df394e7061a224d2c33cd015b157d746869863",
				"panda eyebrow bullet gorilla call smoke muffin taste mesh discover soft ostrich alcohol speed nation flash devote level hobby quick inner drive ghost inside",
			),
			(
				"23db8160a31d3e0dca3688ed941adbf3",
				"cat swing flag economy stadium alone churn speed unique patch report train",
			),
			(
				"8197a4a47f0425faeaa69deebc05ca29c0a5b5cc76ceacc0",
				"light rule cinnamon wrap drastic word pride squirrel upgrade then income fatal apart sustain crack supply proud access",
			),
			(
				"066dca1a2bb7e8a1db2832148ce9933eea0f3ac9548d793112d9a95c9407efad",
				"all hour make first leader extend hole alien behind guard gospel lava path output census museum junior mass reopen famous sing advance salt reform",
			),
			(
				"f30f8c1da665478f49b001d94c5fc452",
				"vessel ladder alter error federal sibling chat ability sun glass valve picture",
			),
			(
				"c10ec20dc3cd9f652c7fac2f1230f7a3c828389a14392f05",
				"scissors invite lock maple supreme raw rapid void congress muscle digital elegant little brisk hair mango congress clump",
			),
			(
				"f585c11aec520db57dd353c69554b21a89b20fb0650966fa0a9d6f74fd989d8f",
				"void come effort suffer camp survey warrior heavy shoot primary clutch crush open amazing screen patrol group space point ten exist slush involve unfold",
			)
		];

        for vector in &test_vectors {
            let entropy = Entropy::from_hex(vector.0).unwrap();
            let mnemonic_str = vector.1;
            let mnemonic = Mnemonic::from_entropy(entropy);
            assert_eq!(mnemonic.to_string(), mnemonic_str,);

            let mnemonic = Mnemonic::parse_normalized(mnemonic_str).unwrap();
            assert_eq!(mnemonic.to_entropy(), entropy,);
        }
    }

    #[test]
    fn checksum() {
        let vectors = [
            "00000000000000000000000000000000",
            "7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f",
            "80808080808080808080808080808080",
            "ffffffffffffffffffffffffffffffff",
            "000000000000000000000000000000000000000000000000",
            "7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f",
            "808080808080808080808080808080808080808080808080",
            "ffffffffffffffffffffffffffffffffffffffffffffffff",
            "0000000000000000000000000000000000000000000000000000000000000000",
            "7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f",
            "8080808080808080808080808080808080808080808080808080808080808080",
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            "9e885d952ad362caeb4efe34a8e91bd2",
            "6610b25967cdcca9d59875f5cb50b0ea75433311869e930b",
            "68a79eaca2324873eacc50cb9c6eca8cc68ea5d936f98787c60c7ebc74e6ce7c",
            "c0ba5a8e914111210f2bd131f3d5e08d",
            "6d9be1ee6ebd27a258115aad99b7317b9c8d28b6d76431c3",
            "9f6a2878b2520799a44ef18bc7df394e7061a224d2c33cd015b157d746869863",
            "23db8160a31d3e0dca3688ed941adbf3",
            "8197a4a47f0425faeaa69deebc05ca29c0a5b5cc76ceacc0",
            "066dca1a2bb7e8a1db2832148ce9933eea0f3ac9548d793112d9a95c9407efad",
            "f30f8c1da665478f49b001d94c5fc452",
            "c10ec20dc3cd9f652c7fac2f1230f7a3c828389a14392f05",
            "f585c11aec520db57dd353c69554b21a89b20fb0650966fa0a9d6f74fd989d8f",
            "ed3b83f0d7913a19667a1cfd7298cd57",
            "70639a4e81b151277b345476d169a3743ff3c141",
            "ba2520298b92063a7a0ee1d453ba92513af81d4f86e1d336",
            "9447d2cf44349cd88a58f5b4ff6f83b9a2d54c42f033e12b8e4d00cc",
            "38711e550dc6557df8082b2a87f7860ebbe47ea5867a7068f5f0f5b85db68be8",
        ];

        for entropy_hex in &vectors {
            let ent = Entropy::from_hex(entropy_hex).unwrap();
            let m = Mnemonic::from_entropy(ent);
            let word_count = m.word_count();
            let cs = m.checksum();
            let digest = Sha256::digest(ent);
            assert_eq!(digest[0] >> (8 - word_count / 3), cs);
        }
    }

    #[test]
    fn test_invalid_engish() {
        // correct phrase:
        // "letter advice cage absurd amount doctor acoustic avoid letter advice cage above"

        assert_eq!(
            Mnemonic::parse_normalized(
                "getter advice cage absurd amount doctor acoustic avoid letter advice cage above",
            ),
            Err(Error::UnknownWord(0))
        );

        assert_eq!(
            Mnemonic::parse_normalized(
                "letter advice cagex absurd amount doctor acoustic avoid letter advice cage above",
            ),
            Err(Error::UnknownWord(2))
        );

        assert_eq!(
            Mnemonic::parse_normalized(
                "advice cage absurd amount doctor acoustic avoid letter advice cage above",
            ),
            Err(Error::BadWordCount(11))
        );

        assert_eq!(
            Mnemonic::parse_normalized(
                "primary advice cage absurd amount doctor acoustic avoid letter advice cage above",
            ),
            Err(Error::InvalidChecksum)
        );
    }

    #[test]
    fn test_vectors_japanese() {
        //! Test some Japanese language test vectors.
        //! For these test vectors, we seem to generate different mnemonic phrases than the test
        //! vectors expect us to. However, our generated seeds are correct and tiny-bip39,
        //! an alternative implementation of bip39 also does not fulfill the test vectors.

        // These vectors are tuples of
        // (entropy, mnemonic)
        let vectors = [
			(
				"00000000000000000000000000000000",
				"あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あいこくしん　あおぞら",
			),
			(
				"7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f",
				"そつう　れきだい　ほんやく　わかす　りくつ　ばいか　ろせん　やちん　そつう　れきだい　ほんやく　わかめ",
			),
		];

        for vector in &vectors {
            let entropy = Entropy::from_hex(vector.0).unwrap();
            let mnemonic_str = vector.1;

            let mnemonic = Mnemonic::from_entropy_in(Language::Japanese, entropy);

            assert_eq!(&mnemonic.to_string(), mnemonic_str);
            assert_eq!(mnemonic.to_entropy().as_ref(), entropy.as_ref())
        }
    }
}
