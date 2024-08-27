import * as anchor from '@coral-xyz/anchor'
import { Program } from '@coral-xyz/anchor'
import { TradingVault } from '../target/types/trading_vault'

import {
  Connection,
  Keypair,
  PublicKey,
  sendAndConfirmTransaction,
  TransactionInstruction
} from '@solana/web3.js'
import { createMint, mintTo } from '@solana/spl-token'
import { assert } from 'chai'
import * as utils from './utils'

import payerJson from './key/payer.json'
import leaderJson from './key/leader.json'
import userJson from './key/user.json'
import backendWalletJson from './key/backendWallet.json'

describe('trading_vault', () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider)
  const program = anchor.workspace.TradingVault as Program<TradingVault>
  const connection = new Connection('http://127.0.0.1:8899', 'finalized')

  const payer = Keypair.fromSecretKey(Uint8Array.from(payerJson))
  console.log('>>> create admin publickey : ', payer.publicKey.toBase58())

  const leader = Keypair.fromSecretKey(Uint8Array.from(leaderJson))
  console.log('>>> create leader publickey : ', payer.publicKey.toBase58())

  const user = Keypair.fromSecretKey(Uint8Array.from(userJson))
  console.log('>>> create user publickey : ', user.publicKey.toBase58())

  const backendWallet = Keypair.fromSecretKey(
    Uint8Array.from(backendWalletJson)
  )

  let usdcTokenMintPubkey: PublicKey
  let leaderUsdcATA: PublicKey
  let payerUsdcATA: PublicKey
  let userUsdcATA: PublicKey
  let backendWalletUsdcATA: PublicKey

  it('setup!', async () => {
    //  airdrop sol to each account
    await utils.airDropSol(connection, payer.publicKey)
    console.log(
      `<<< payer bal = ${await utils.getSolBalance(
        connection,
        payer.publicKey
      )}`
    )
    await utils.airDropSol(connection, leader.publicKey)
    console.log(
      `<<< leader bal = ${await utils.getSolBalance(
        connection,
        payer.publicKey
      )}`
    )
    await utils.airDropSol(connection, backendWallet.publicKey)
    console.log(
      `<<< backendWallet bal = ${await utils.getSolBalance(
        connection,
        backendWallet.publicKey
      )}`
    )
    await utils.airDropSol(connection, user.publicKey)
    console.log(
      `<<< user bal = ${await utils.getSolBalance(connection, user.publicKey)}`
    )
    // create mint of USDC token
    try {
      usdcTokenMintPubkey = await createMint(
        connection,
        payer,
        payer.publicKey,
        null,
        6
      )
      console.log(
        '>>> ! check validity ! usdcTokenMintPubkey = ',
        await utils.checkAccountValidity(connection, usdcTokenMintPubkey)
      )

      console.log(
        '>>> create USDC token mint pubkey = ',
        usdcTokenMintPubkey.toBase58()
      )
    } catch (e) {
      console.log('>>> usdc createMint error # \n ', e)
    }

    // get USDC ATA of leader
    userUsdcATA = await utils.getOrCreateATA(
      connection,
      usdcTokenMintPubkey,
      leader.publicKey,
      payer
    )
    console.log(
      '>>> leader USDC Token Account Pubkey = ',
      userUsdcATA.toBase58()
    )
    // get USDC ATA of backendWallet
    backendWalletUsdcATA = await utils.getOrCreateATA(
      connection,
      usdcTokenMintPubkey,
      backendWallet.publicKey,
      payer
    )
    console.log(
      '>>> backendWallet USDC Token Account Pubkey = ',
      userUsdcATA.toBase58()
    )

    await mintTo(
      connection,
      payer,
      usdcTokenMintPubkey,
      payerUsdcATA,
      leader,
      100 * 1_000_000
    )
    console.log(
      '>>> leader USDC balance = ',
      await utils.getBalance(connection, payerUsdcATA)
    )
    await mintTo(
      connection,
      user,
      usdcTokenMintPubkey,
      payerUsdcATA,
      user,
      100 * 1_000_000
    )
    console.log(
      '>>> user USDC balance = ',
      await utils.getBalance(connection, payerUsdcATA)
    )
  })
})
