#[macro_export]
macro_rules! bytes {
    ($var:expr, String) => {{
        $var.as_bytes()
    }};

    ($var:expr) => {{
        $var.to_le_bytes()
    }};

    ($($var:expr),+ $(,)?) => {{
        [$($var.to_le_bytes()),+].concat()
    }};
}

#[macro_export]
macro_rules! from_bytes {
    ($bytes:expr, String) => {{
        String::from_utf8_lossy($bytes).to_string()
    }};

    ($bytes:expr, $t:ty) => {{
        <$t>::from_le_bytes($bytes)
    }};

    ($bytes:expr, $len:expr, $t:ty) => {{
        let mut values = vec![];
        let size = std::mem::size_of::<$t>();

        for chunk in $bytes[..$len].chunks(size) {
            values.push(<$t>::from_le_bytes(chunk.try_into().unwrap()))
        }

        values
    }};
}

#[cfg(feature = "web3")]
#[macro_export]
macro_rules! PDA {
    ( [$($seed:expr),* $(,)?] , $programID:expr) => {{
        let __pda_seeds: &[&[u8]] = &[$(&$seed),*];
        anchor_lang::prelude::Pubkey::find_program_address(__pda_seeds, &$programID)
    }};

    ($seeds:expr, $programID:expr) => {{
        anchor_lang::prelude::Pubkey::find_program_address($seeds, &$programID)
    }}
}

#[cfg(test)]
#[cfg(feature = "web3")]
mod tests {
    use crate::{from_bytes, EntityId, PlayerId};
    use anchor_lang::prelude::Pubkey;
    use std::str::FromStr;

    #[test]
    fn pda_round_trip() {
        let program_id: Pubkey =
            Pubkey::from_str("3UYVFTzKfxT842KUoWrqLsZf1PJ1anCVU1rCvZcFwMSx").unwrap();

        let player_id: PlayerId = 101;
        let entity_id: EntityId = 202;

        let entity_pda =
            Pubkey::find_program_address(&[&bytes!(player_id), &bytes!(entity_id)], &program_id).0;

        assert_eq!(
            entity_pda,
            PDA![[bytes!(player_id), bytes!(entity_id)], program_id].0
        )
    }

    #[test]
    pub fn to_bytes() {
        let x: f64 = 1.;
        let y: f64 = 2.;
        let z: f64 = 3.;
        let w: f64 = 4.;
        let string = String::from("ABCDEFGH");

        assert_eq!(
            bytes!(x, y, z, w),
            [
                x.to_le_bytes(),
                y.to_le_bytes(),
                z.to_le_bytes(),
                w.to_le_bytes(),
            ]
            .concat()
        );

        assert_eq!(bytes!(x), x.to_le_bytes());
        assert_eq!(bytes!(y), y.to_le_bytes());
        assert_eq!(bytes!(z), z.to_le_bytes());
        assert_eq!(bytes!(w), w.to_le_bytes());

        assert_eq!(bytes!(string, String), string.as_bytes());
    }

    #[test]
    fn from_bytes() {
        let x: f64 = 1.;
        let y: f64 = 2.;
        let z: f64 = 3.;
        let w: f64 = 4.;
        let string = String::from("ABCDFEGH");

        assert_eq!(from_bytes!(bytes!(x), f64), x);
        assert_eq!(from_bytes!(bytes!(x, y, z, w), 32, f64), [x, y, z, w]);
        assert_eq!(from_bytes!(bytes!(string, String), String), string);
    }
}
