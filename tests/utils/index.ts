import { BN } from '@coral-xyz/anchor'
import {
  getOrCreateAssociatedTokenAccount,
  getAccount,
  mintToChecked,
  getAssociatedTokenAddressSync,
  createAssociatedTokenAccountInstruction
} from '@solana/spl-token'
import {
  Connection,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  Transaction,
  TransactionInstruction,
  TransactionMessage,
  TransactionSignature,
  VersionedTransaction
} from '@solana/web3.js'

import * as nacl from 'tweetnacl'

export const airDropSol = async (
  connection: Connection,
  publicKey: PublicKey,
  amount = 10
) => {
  try {
    while ((await getSolBalance(connection, publicKey)) < 1) {
      const airdropSignature = await connection.requestAirdrop(
        publicKey,
        amount * LAMPORTS_PER_SOL
      )
      const latestBlockHash = await connection.getLatestBlockhash()
      await connection.confirmTransaction(
        {
          blockhash: latestBlockHash.blockhash,
          lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
          signature: airdropSignature
        },
        connection.commitment
      )
    }
  } catch (error) {
    console.error(error)
    throw error
  }
}

export const airDropSolIfBalanceNotEnough = async (
  connection: Connection,
  publicKey: PublicKey,
  balance = 1
) => {
  const walletBalance = await connection.getBalance(publicKey)
  if (walletBalance < balance * LAMPORTS_PER_SOL) {
    await airDropSol(connection, publicKey)
  }
}

export const getOrCreateATA = async (
  connection: Connection,
  mint: PublicKey,
  owner: PublicKey,
  payer: Keypair
) => {
  const ata = await getOrCreateAssociatedTokenAccount(
    connection,
    payer,
    mint,
    owner
  )

  return ata.address
}

export const toTokenAmount = (uiAmount: number, decimals: number): BN => {
  return new BN(uiAmount * 10 ** decimals)
}

export const toUiAmount = (token_amount: number, decimals: number): number => {
  return token_amount / 10 ** decimals
}

// return in lamports
export const getSolBalance = async (
  connection: Connection,
  pubkey: PublicKey
) => {
  return connection
    .getBalance(pubkey)
    .then(balance => balance)
    .catch(() => 0)
}

export const getBalance = async (connection: Connection, pubkey: PublicKey) => {
  return getAccount(connection, pubkey)
    .then(account => Number(account.amount))
    .catch(() => 0)
}

export const mintTokens = async (
  connection: Connection,
  payer: Keypair,
  uiAmount: number,
  decimals: number,
  mint: PublicKey,
  destiantionWallet: PublicKey
) => {
  await mintToChecked(
    connection,
    payer,
    mint,
    destiantionWallet,
    payer.publicKey,
    toTokenAmount(uiAmount, decimals).toNumber(),
    decimals
  )
}

export const checkAccountValidity = async (
  connection: Connection,
  publicKey: PublicKey
) => {
  const accountInfo = await connection.getAccountInfo(publicKey)
  return accountInfo != null && accountInfo != undefined
}

export const sendTransaction = async (
  connection: Connection,
  tx: TransactionInstruction,
  signer: Keypair
) => {
  const blockhashResponse = await connection.getLatestBlockhash()
  const lastValidBlockHeight = blockhashResponse.lastValidBlockHeight - 150
  const transaction = new Transaction({
    feePayer: signer.publicKey,
    blockhash: blockhashResponse.blockhash,
    lastValidBlockHeight: lastValidBlockHeight
  }).add(tx)

  const message = transaction.serializeMessage()
  const signature = nacl.sign.detached(message, signer.secretKey)
  transaction.addSignature(signer.publicKey, Buffer.from(signature))
  const rawTransaction = transaction.serialize()
  let blockheight = await connection.getBlockHeight()

  let txId: TransactionSignature
  while (blockheight < lastValidBlockHeight) {
    txId = await connection.sendRawTransaction(rawTransaction, {
      skipPreflight: true
    })

    await connection.confirmTransaction(
      {
        blockhash: blockhashResponse.blockhash,
        lastValidBlockHeight: lastValidBlockHeight,
        signature: txId
      },
      connection.commitment
    )

    await sleep(500)
    blockheight = await connection.getBlockHeight()
  }

  return txId
}

export const getAssociatedTokenAccountInstruction = (
  payer: Keypair,
  mint: PublicKey,
  owner: PublicKey
): {
  associatedTokenAccount: PublicKey
  tx: TransactionInstruction
} => {
  let associatedTokenAccount = getAssociatedTokenAddressSync(mint, owner)
  let tx = createAssociatedTokenAccountInstruction(
    payer.publicKey,
    associatedTokenAccount,
    owner,
    mint
  )
  return {
    associatedTokenAccount,
    tx
  }
}

const sleep = (ms: number) => {
  return new Promise(resolve => setTimeout(resolve, ms))
}

async function isBlockhashExpired (
  connection: Connection,
  lastValidBlockHeight: number
) {
  let currentBlockHeight = await connection.getBlockHeight('finalized')
  console.log('                           ')
  console.log('Current Block height:             ', currentBlockHeight)
  console.log('Last Valid Block height - 150:     ', lastValidBlockHeight - 150)
  console.log('--------------------------------------------')
  console.log(
    'Difference:                      ',
    currentBlockHeight - (lastValidBlockHeight - 150)
  ) // If Difference is positive, blockhash has expired.
  console.log('                           ')

  return currentBlockHeight > lastValidBlockHeight - 150
}

export const sendTransactionAndConfirm = async (
  connection: Connection,
  ix: TransactionInstruction,
  signer: Keypair
) => {
  const START_TIME = new Date()
  let txId: TransactionSignature
  let lastValidHeight: number

  // Step 1 - Get Latest Blockhash
  const blockhashResponse = await connection.getLatestBlockhashAndContext(
    'finalized'
  )
  lastValidHeight = blockhashResponse.value.lastValidBlockHeight

  // Step 2 - Create a SOL Transfer Transaction
  const messageV0 = new TransactionMessage({
    payerKey: signer.publicKey,
    recentBlockhash: blockhashResponse.value.blockhash,
    instructions: [ix]
  }).compileToV0Message()
  const transaction = new VersionedTransaction(messageV0)
  transaction.sign([signer])
  console.log(`<<< ${transaction}`)
  // Step 3 - Send Transaction to the Network
  txId = await connection.sendTransaction(transaction)

  // Step 4 - Check transaction status and blockhash status until the transaction succeeds or blockhash expires
  let hashExpired = false
  let txSuccess = false
  while (!hashExpired && !txSuccess) {
    console.log(`<<< ${txId}`)
    const { value: status } = await connection.getSignatureStatus(txId)

    // Break loop if transaction has succeeded
    if (
      status &&
      (status.confirmationStatus === 'confirmed' ||
        status.confirmationStatus === 'finalized')
    ) {
      txSuccess = true
      const endTime = new Date()
      const elapsed = (endTime.getTime() - START_TIME.getTime()) / 1000
      console.log(`Transaction Success. Elapsed time: ${elapsed} seconds.`)
      // console.log(`https://explorer.solana.com/tx/${txId}?cluster=devnet`)
      break
    }

    hashExpired = await isBlockhashExpired(connection, lastValidHeight)

    // Break loop if blockhash has expired
    if (hashExpired) {
      const endTime = new Date()
      const elapsed = (endTime.getTime() - START_TIME.getTime()) / 1000
      console.log(`Blockhash has expired. Elapsed time: ${elapsed} seconds.`)
      // (add your own logic to Fetch a new blockhash and resend the transaction or throw an error)
      break
    }

    // Check again after 2.5 sec
    await sleep(2500)
  }
}
