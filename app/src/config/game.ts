import { PublicKey } from '@solana/web3.js';

export const PROGRAM_ID = new PublicKey('3UYVFTzKfxT842KUoWrqLsZf1PJ1anCVU1rCvZcFwMSx');

// This should be set to the game ID output by the admin client.
// Keep as a string to avoid precision loss for large u64 values; convert to BigInt when needed.
export const GAME_ID = process.env.NEXT_PUBLIC_GAME_ID ? process.env.NEXT_PUBLIC_GAME_ID : '0';

// Constants from the smart contract
export const COIN_REQUIREMENT = 10;
export const REWARD_AMOUNT = 1_000_000_000; // 1 SOL