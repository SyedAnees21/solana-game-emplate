import { Connection, PublicKey, SystemProgram, Transaction, TransactionInstruction } from '@solana/web3.js';
import { PROGRAM_ID, GAME_ID } from '../config/game';

export const getGamePDAs = (programId: PublicKey) => {
    // Write GAME_ID as little-endian u64 explicitly to avoid endianness issues
    const gameIdBuf = Buffer.alloc(8);
    // Node Buffer supports writeBigUInt64LE; browsers will use polyfills but this is explicit
    gameIdBuf.writeBigUInt64LE(BigInt(GAME_ID));

    const gameStateSeeds = [Buffer.from('game'), gameIdBuf];
    const [gameStatePda] = PublicKey.findProgramAddressSync(gameStateSeeds, programId);

    const [treasuryPda] = PublicKey.findProgramAddressSync(
        [Buffer.from('treasury'), gameIdBuf],
        programId
    );

    return { gameStatePda, treasuryPda };
};

export const getPlayerPDA = (programId: PublicKey, playerPubkey: PublicKey) => {
    const gameIdBuf = Buffer.alloc(8);
    gameIdBuf.writeBigUInt64LE(BigInt(GAME_ID));
    const gameStateSeeds = [Buffer.from('game'), gameIdBuf];
    const [gameStatePda] = PublicKey.findProgramAddressSync(gameStateSeeds, programId);

    const [playerStatePda] = PublicKey.findProgramAddressSync(
        [Buffer.from('player'), gameIdBuf, playerPubkey.toBuffer()],
        programId
    );
    return playerStatePda;
};

export const createClaimRewardInstruction = (
    playerPubkey: PublicKey,
    playerStatePda: PublicKey,
    gameStatePda: PublicKey,
    treasuryPda: PublicKey,
): TransactionInstruction => {
    const keys = [
        { pubkey: playerStatePda, isSigner: false, isWritable: true },
        { pubkey: gameStatePda, isSigner: false, isWritable: true },
        { pubkey: treasuryPda, isSigner: false, isWritable: true },
        { pubkey: playerPubkey, isSigner: true, isWritable: true },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ];

    return new TransactionInstruction({
        keys,
        programId: PROGRAM_ID,
        data: Buffer.from([3]), // 3 is the instruction index for ClaimReward
    });
};

export const createInitializePlayerInstruction = (
    playerPubkey: PublicKey,
    playerStatePda: PublicKey,
    gameStatePda: PublicKey,
): TransactionInstruction => {
    const keys = [
        { pubkey: playerStatePda, isSigner: false, isWritable: true },
        { pubkey: gameStatePda, isSigner: false, isWritable: true },
        { pubkey: playerPubkey, isSigner: true, isWritable: true },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ];

    return new TransactionInstruction({
        keys,
        programId: PROGRAM_ID,
        data: Buffer.from([2]), // 2 is the instruction index for InitializePlayer
    });
};