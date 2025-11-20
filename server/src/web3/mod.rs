use anchor_lang::declare_program;
use proto_interface::web3::InlineString;

use crate::web3::solana_game::types;

pub mod provider;
pub mod service;
pub mod instructions;

declare_program!(solana_game);

impl From<InlineString> for types::InlineString {
    fn from(value: InlineString) -> Self {
        Self {
            len: value.get_len() as u8,
            buffer: value.as_ref().clone(),
        }
    }
}
