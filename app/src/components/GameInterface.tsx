// "use client";

// import dynamic from 'next/dynamic';
// import { useConnection, useWallet } from '@solana/wallet-adapter-react';
// // WalletMultiButton uses DOM APIs and can cause server/client HTML mismatch. Load it only on the client.
// const WalletMultiButton = dynamic(
//     () => import('@solana/wallet-adapter-react-ui').then((mod) => mod.WalletMultiButton),
//     { ssr: false }
// );
// import { LAMPORTS_PER_SOL, Transaction } from '@solana/web3.js';
// import { FC, useCallback, useEffect, useState } from 'react';
// import { PROGRAM_ID, GAME_ID, COIN_REQUIREMENT } from '../config/game';
// import { getGamePDAs, getPlayerPDA, createClaimRewardInstruction, createInitializePlayerInstruction } from '../utils/program';

// interface PlayerStats {
//     coins: number;
//     canClaimReward: boolean;
//     accountExists: boolean;
// }

// const GameInterface: FC = () => {
//     const { connection } = useConnection();
//     const { publicKey, sendTransaction } = useWallet();
//     const [playerStats, setPlayerStats] = useState<PlayerStats | null>(null);
//     const [loading, setLoading] = useState(false);
//     const [error, setError] = useState<string | null>(null);

//     const fetchPlayerStats = useCallback(async () => {
//         if (!publicKey || !connection) return;

//         try {
//             setLoading(true);
//             setError(null);
            
//             const { gameStatePda } = getGamePDAs(PROGRAM_ID);
//             const playerStatePda = getPlayerPDA(PROGRAM_ID, publicKey);
//             // const playerStatePda = getPlayerPDA(PROGRAM_ID, gameStatePda, publicKey);

//             // Fetch player account data
//             const playerAccount = await connection.getAccountInfo(playerStatePda);
            
//             if (playerAccount) {
//                 // Parse player data according to the Anchor `PlayerState` layout:
//                 // 8 bytes discriminator | 32 bytes player pubkey | 32 bytes game_state pubkey | 8 bytes coin_count | ...
//                 try {
//                     const data = playerAccount.data;
//                     // normalize to Uint8Array for DataView
//                     const u8 = data instanceof Uint8Array ? data : new Uint8Array(data);
//                     const offset = 8 + 32 + 32; // discriminator + player pubkey + game_state pubkey

//                     let coinsBigInt: bigint;
//                     if (typeof (u8 as any).readBigUInt64LE === 'function') {
//                         // Buffer-like, use built-in
//                         coinsBigInt = (u8 as any).readBigUInt64LE(offset);
//                     } else {
//                         const view = new DataView(u8.buffer, u8.byteOffset, u8.byteLength);
//                         coinsBigInt = view.getBigUint64(offset, true);
//                     }

//                     const coins = Number(coinsBigInt);
//                     setPlayerStats({
//                         coins,
//                         canClaimReward: coins >= COIN_REQUIREMENT,
//                         accountExists: true,
//                     });
//                 } catch (e) {
//                     console.error('Failed to parse player account data', e);
//                     throw e;
//                 }
//             } else {
//                 setPlayerStats({
//                     coins: 0,
//                     canClaimReward: false,
//                     accountExists: false
//                 });
//             }
//         } catch (error) {
//             console.error('Error fetching player stats:', error);
//             setError('Failed to fetch player stats. Please try again.');
//             setPlayerStats(null);
//         } finally {
//             setLoading(false);
//         }
//     }, [publicKey, connection]);

//     // Fetch player stats when wallet connects
//     useEffect(() => {
//         fetchPlayerStats();
//     }, [fetchPlayerStats]);

//     const initializePlayer = async () => {
//         if (!publicKey || !connection) return;

//         try {
//             setLoading(true);
//             setError(null);

//             const { gameStatePda } = getGamePDAs(PROGRAM_ID);
//             const playerStatePda = getPlayerPDA(PROGRAM_ID, publicKey);
//             // const playerStatePda = getPlayerPDA(PROGRAM_ID, gameStatePda, publicKey);

//             const instruction = createInitializePlayerInstruction(
//                 publicKey,
//                 playerStatePda,
//                 gameStatePda
//             );

//             const transaction = new Transaction().add(instruction);
//             const { blockhash } = await connection.getLatestBlockhash();
//             transaction.recentBlockhash = blockhash;
//             transaction.feePayer = publicKey;

//             await sendTransaction(transaction, connection);
            
//             // Wait a bit for the transaction to be confirmed
//             await new Promise(resolve => setTimeout(resolve, 2000));
//             await fetchPlayerStats();
//         } catch (error) {
//             console.error('Error initializing player:', error);
//             setError('Failed to initialize player. Please try again.');
//         } finally {
//             setLoading(false);
//         }
//     };

//     const handleClaimReward = async () => {
//         if (!publicKey || !playerStats?.canClaimReward || !connection) return;

//         try {
//             setLoading(true);
//             setError(null);

//             const { gameStatePda, treasuryPda } = getGamePDAs(PROGRAM_ID);
//             const playerStatePda = getPlayerPDA(PROGRAM_ID, publicKey);
//             // const playerStatePda = getPlayerPDA(PROGRAM_ID, gameStatePda, publicKey);

//             const instruction = createClaimRewardInstruction(
//                 publicKey,
//                 playerStatePda,
//                 gameStatePda,
//                 treasuryPda
//             );

//             const transaction = new Transaction().add(instruction);
//             const { blockhash } = await connection.getLatestBlockhash();
//             transaction.recentBlockhash = blockhash;
//             transaction.feePayer = publicKey;

//             await sendTransaction(transaction, connection);
            
//             // Wait a bit for the transaction to be confirmed
//             await new Promise(resolve => setTimeout(resolve, 2000));
//             await fetchPlayerStats();
//         } catch (error) {
//             console.error('Error claiming reward:', error);
//             setError('Failed to claim reward. Please try again.');
//         } finally {
//             setLoading(false);
//         }
//     };

//     if (!GAME_ID) {
//         return (
//             <div className="container">
//                 <p>Error: Game ID not configured. Please set NEXT_PUBLIC_GAME_ID in your environment.</p>
//             </div>
//         );
//     }

//     return (
//         <div className="container">
//             <div className="wallet-button">
//                 <WalletMultiButton />
//             </div>

//             {error && (
//                 <div className="error-message">
//                     {error}
//                 </div>
//             )}

//             {publicKey ? (
//                 <div className="game-interface">
//                     <h2>Player Stats</h2>
//                     {loading ? (
//                         <p>Loading...</p>
//                     ) : playerStats ? (
//                         <>
//                             <div className="stats">
//                                 <p>Coins: {playerStats.coins}</p>
//                                 {playerStats.canClaimReward && (
//                                     <p className="reward-available">Reward Available! 🎉</p>
//                                 )}
//                             </div>
                            
//                             {!playerStats.accountExists ? (
//                                 <button 
//                                     onClick={initializePlayer}
//                                     disabled={loading}
//                                 >
//                                     Initialize Player Account
//                                 </button>
//                             ) : (
//                                 <button 
//                                     onClick={handleClaimReward}
//                                     disabled={!playerStats.canClaimReward || loading}
//                                 >
//                                     {loading ? 'Processing...' : 'Claim 1 SOL Reward'}
//                                 </button>
//                             )}
//                         </>
//                     ) : (
//                         <p>Failed to load player data. Please try again.</p>
//                     )}
//                 </div>
//             ) : (
//                 <p>Connect your wallet to play</p>
//             )}

//             <style jsx>{`
//                 .container {
//                     max-width: 600px;
//                     margin: 0 auto;
//                     padding: 20px;
//                 }
//                 .wallet-button {
//                     margin-bottom: 20px;
//                 }
//                 .game-interface {
//                     padding: 20px;
//                     border: 1px solid #ccc;
//                     border-radius: 8px;
//                 }
//                 .stats {
//                     margin-bottom: 20px;
//                 }
//                 .reward-available {
//                     color: #4caf50;
//                     font-weight: bold;
//                 }
//                 .error-message {
//                     background-color: #ffebee;
//                     color: #c62828;
//                     padding: 10px;
//                     border-radius: 4px;
//                     margin-bottom: 20px;
//                 }
//                 button {
//                     background-color: #512da8;
//                     color: white;
//                     border: none;
//                     padding: 10px 20px;
//                     border-radius: 4px;
//                     cursor: pointer;
//                     margin-top: 10px;
//                     width: 100%;
//                     font-size: 16px;
//                 }
//                 button:disabled {
//                     background-color: #cccccc;
//                     cursor: not-allowed;
//                 }
//                 button:not(:disabled):hover {
//                     background-color: #4527a0;
//                 }
//             `}</style>
//         </div>
//     );
// };

// export default GameInterface;

// components/GameInterface.tsx (refactor using Anchor)
"use client";

import dynamic from "next/dynamic";
import { FC, useCallback, useEffect, useState } from "react";
import { useConnection, useWallet } from "@solana/wallet-adapter-react";
import { PublicKey, Transaction } from "@solana/web3.js";
import { AnchorProvider, Program, Idl } from "@coral-xyz/anchor";
import { PROGRAM_ID, GAME_ID, COIN_REQUIREMENT } from "../config/game";
import { getGamePDAs, getPlayerPDA } from "../utils/program";
import { SolanaGame } from '../../../target/types/solana_game'

const IDL = require('../../../target/idl/votingdapp.json')

const WalletMultiButton = dynamic(
  () =>
    import("@solana/wallet-adapter-react-ui").then((mod) => mod.WalletMultiButton),
  { ssr: false }
);

interface PlayerStats {
  coins: number;
  canClaimReward: boolean;
  accountExists: boolean;
}

const programId = new PublicKey(PROGRAM_ID);
// const solanaGameProgram = new Program<SolanaGame>(IDL)
// type SolanaGameProgram = Program<Idl>; // generic Program; better typed if you generate TS types

const GameInterface: FC = () => {
  const { connection } = useConnection();
  const { publicKey, signTransaction, sendTransaction } = useWallet();
  const [playerStats, setPlayerStats] = useState<PlayerStats | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Create Anchor provider + program when wallet & connection available
  const getProgram = useCallback((): Program<SolanaGame> | null => {
    if (!connection || !publicKey || !signTransaction) return null;

    const walletAdapter = {
      publicKey,
      signTransaction,
      signAllTransactions: (txs: Transaction[]) =>
        Promise.reject(new Error("signAll not implemented")),
    } as any; // Anchor expects an "AnchorWallet" type; this is minimal

    const provider = new AnchorProvider(connection, walletAdapter, {
      preflightCommitment: "confirmed",
    });

    return new Program<SolanaGame>(IDL, provider);
  }, [connection, publicKey, signTransaction]);

  const fetchPlayerStats = useCallback(async () => {
    if (!connection || !publicKey) return;

    setLoading(true);
    setError(null);

    try {
      const program = getProgram();
      if (!program) throw new Error("Wallet not connected or provider not ready");

      // derive PDAs
      const { gameStatePda } = getGamePDAs(programId);
      const playerStatePda = getPlayerPDA(programId, publicKey);

      // Try to fetch player account using IDL (returns typed object or null)
      const playerAccount = await program.account.playerState.fetchNullable(playerStatePda);

      if (!playerAccount) {
        setPlayerStats({ coins: 0, canClaimReward: false, accountExists: false });
        return;
      }

      // Anchor deserializes and gives you JS types:
      // playerAccount.coinCount (note: anchor lowercases / camelCases fields)
      // adjust depending on your IDL field names
      const coins = (playerAccount as any).coinCount ?? (playerAccount as any).coin_count ?? 0;
      setPlayerStats({
        coins: Number(coins),
        canClaimReward: Number(coins) >= COIN_REQUIREMENT,
        accountExists: true,
      });
    } catch (err) {
      console.error("fetchPlayerStats error:", err);
      setError("Failed to fetch player stats");
    } finally {
      setLoading(false);
    }
  }, [connection, publicKey, getProgram]);

  useEffect(() => {
    fetchPlayerStats();
  }, [fetchPlayerStats]);

  const initializePlayer = async () => {
    if (!publicKey || !connection) return;

    setLoading(true);
    setError(null);

    try {
      const program = getProgram();
      if (!program) throw new Error("Provider not ready");

      const { gameStatePda } = getGamePDAs(programId);
      const playerStatePda = getPlayerPDA(programId, publicKey);

      // Use Anchor method call (IDL-aware)
      // Note: method name & args must match your program IDL (e.g., "initializePlayer")
      await program.methods
        .initializePlayer(publicKey) // pass player pubkey if your instruction signature requires it
        .accounts({
          player_state: playerStatePda,
          game_state: gameStatePda,
          authority: publicKey,
          systemProgram: PublicKey.default, // Anchor will resolve system program if omitted; or use SystemProgram.programId
        })
        .rpc({ commitment: "confirmed" });

      // After tx
      await fetchPlayerStats();
    } catch (err) {
      console.error("initializePlayer error:", err);
      setError("Failed to initialize player");
    } finally {
      setLoading(false);
    }
  };

  const claimReward = async () => {
    if (!publicKey || !connection) return;
    if (!playerStats?.canClaimReward) return;

    setLoading(true);
    setError(null);

    try {
      const program = getProgram();
      if (!program) throw new Error("Provider not ready");

      const { gameStatePda, treasuryPda } = getGamePDAs(programId);
      const playerStatePda = getPlayerPDA(programId, publicKey);

      await program.methods
        .claimReward() // if your instruction needs args, pass them here
        .accounts({
          playerState: playerStatePda,
          gameState: gameStatePda,
          treasury: treasuryPda,
          authority: publicKey,
          systemProgram: PublicKey.default,
        })
        .rpc({ commitment: "confirmed" });

      await fetchPlayerStats();
    } catch (err) {
      console.error("claimReward error:", err);
      setError("Failed to claim reward");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="container">
      <div className="wallet-button">
        <WalletMultiButton />
      </div>

      {error && <div className="error-message">{error}</div>}

      {publicKey ? (
        <div className="game-interface">
          <h2>Player Stats</h2>
          {loading ? (
            <p>Loading...</p>
          ) : playerStats ? (
            <>
              <div className="stats">
                <p>Coins: {playerStats.coins}</p>
                {playerStats.canClaimReward && <p className="reward-available">Reward Available! 🎉</p>}
              </div>

              {!playerStats.accountExists ? (
                <button onClick={initializePlayer} disabled={loading}>
                  Initialize Player Account
                </button>
              ) : (
                <button onClick={claimReward} disabled={!playerStats.canClaimReward || loading}>
                  {loading ? "Processing..." : "Claim 1 SOL Reward"}
                </button>
              )}
            </>
          ) : (
            <p>Failed to load player data. Please try again.</p>
          )}
        </div>
      ) : (
        <p>Connect your wallet to play</p>
      )}
    </div>
  );
};

export default GameInterface;
