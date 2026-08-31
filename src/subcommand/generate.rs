use super::*;

#[derive(Parser)]
pub(crate) struct Generate {
    #[arg(
        long,
        value_name = "N",
        help = "Diceware-style words from the BIP39 list, 11 bits each (default 6, ~66 bits)."
    )]
    words: Option<usize>,
    #[arg(
        long,
        value_name = "N",
        conflicts_with = "words",
        help = "Random characters instead of words (~6.5 bits each, ~5.95 without symbols)."
    )]
    length: Option<usize>,
    #[arg(long, help = "Exclude symbols (with --length).")]
    no_symbols: bool,
}

impl Generate {
    pub(crate) fn run(self) -> Result {
        println!("{}", *password(self.words, self.length, self.no_symbols)?);
        Ok(())
    }
}

/// Uniform sampling in both modes: word indices mask to 11 bits (2048 divides
/// 65536 exactly); characters use rejection sampling to kill modulo bias.
pub(crate) fn password(
    words: Option<usize>,
    length: Option<usize>,
    no_symbols: bool,
) -> Result<Zeroizing<String>> {
    if let Some(length) = length {
        ensure!(length > 0, "--length must be positive");
        let mut charset: Vec<char> = ('a'..='z').chain('A'..='Z').chain('0'..='9').collect();
        if !no_symbols {
            charset.extend("!#$%&()*+-./:;<=>?@[]^_{|}~".chars());
        }

        let mut password = Zeroizing::new(String::with_capacity(length));
        let limit = u8::MAX - u8::MAX % charset.len() as u8;
        while password.len() < length {
            let [byte] = random::<1>()?;
            if byte < limit {
                password.push(charset[byte as usize % charset.len()]);
            }
        }
        eprintln!("~{:.0} bits", length as f64 * (charset.len() as f64).log2());
        Ok(password)
    } else {
        let words = words.unwrap_or(6);
        ensure!(words > 0, "--words must be positive");
        let list = gilgamesh::bip39::Language::English.word_list();

        let mut picked = Vec::with_capacity(words);
        for _ in 0..words {
            let index = u16::from_le_bytes(random::<2>()?) & 0x7ff;
            picked.push(list[usize::from(index)]);
        }
        eprintln!("~{} bits", words * 11);
        Ok(Zeroizing::new(picked.join("-")))
    }
}
