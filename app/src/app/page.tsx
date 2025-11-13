'use client';

import { WalletContextProvider } from '@/components/WalletContextProvider';
import GameInterface from '@/components/GameInterface';

export default function Home() {
  return (
    <WalletContextProvider>
      <main className="min-h-screen p-8">
        <h1 className="text-4xl font-bold text-center mb-8">Solana Game</h1>
        <GameInterface />
      </main>
    </WalletContextProvider>
  );
}