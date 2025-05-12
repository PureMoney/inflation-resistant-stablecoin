import {
  findAllMarkets,
  MarketAccount,
  OPENBOOK_PROGRAM_ID,
  OpenBookV2Client,
  UIMarket,
} from "../openbook";
import {
  ComputeBudgetProgram,
  SystemProgram,
  PublicKey,
  TransactionInstruction,
  Connection,
  AddressLookupTableAccount,
  VersionedTransaction,
  Signer,
} from "@solana/web3.js";
import { useHookConnection, useFakeProvider } from "../hooks/useOpenbookClient";
import { Keypair, TransactionMessage } from "@solana/web3.js";

// MAINNET
// export const RPC = "https://misty-wcb8ol-fast-mainnet.helius-rpc.com/";

export const RPC = process.env.NEXT_PUBLIC_RPC;

// "https://misty-wcb8ol-fast-mainnet.helius-rpc.com/";

// DEVNET
// export const RPC = "https://aimil-f4d13p-fast-devnet.helius-rpc.com/";

// export const connection = new Connection("https://aimil-f4d13p-fast-devnet.helius-rpc.com/");

const connection = new Connection(RPC);

const getPrivateKey = (val: string) => {
  const uArray = Uint8Array.from(
    val
      .split(",")
      .map((x) => parseInt(x))
      .filter((x) => !isNaN(x) && x >= 0 && x <= 255)
  );
  if (uArray.length !== 64) {
    throw new Error("Invalid private key length: " + uArray.length);
  }
  return Keypair.fromSecretKey(uArray);
};

const walletPk = process.env.NEXT_PUBLIC_WALLET_PK;
export const wallet = (walletPk && walletPk.length > 0) ? getPrivateKey(walletPk) : null;

if (wallet) console.log("wallet address: ", wallet.publicKey.toBase58());

const openOrderAdminPk = process.env.NEXT_PUBLIC_OPEN_ORDER_ADMIN_E_PK;
// console.log("openOrderAdminPk: ", openOrderAdminPk);

export const openOrdersAdminE = (openOrderAdminPk && openOrderAdminPk.length > 0) ?
 getPrivateKey(openOrderAdminPk) : null;

if (openOrdersAdminE) console.log("openOrdersAdminE address: ", openOrdersAdminE.publicKey.toBase58());

const closeMarketAdminPk = process.env.NEXT_PUBLIC_CLOSE_MARKET_ADMIN_E_PK;
// console.log("closeMarketAdminPk: ", closeMarketAdminPk);

export const closeMarketAdminE = (closeMarketAdminPk && closeMarketAdminPk.length > 0) ?
  getPrivateKey(closeMarketAdminPk) : null;

if (closeMarketAdminE) console.log("closeMarketAdminE address: ", closeMarketAdminE.publicKey.toBase58());

const consumeEventsAdminPk = process.env.NEXT_PUBLIC_CONSUME_EVENTS_ADMIN_E_PK;
// console.log("consumeEventsAdminPk: ", consumeEventsAdminPk);

export const consumeEventsAdminE = (consumeEventsAdminPk && consumeEventsAdminPk.length > 0) ?
  getPrivateKey(consumeEventsAdminPk) : null;

if (consumeEventsAdminE) console.log("consumeEventsAdminE address: ", consumeEventsAdminE.publicKey.toBase58());

export const tokenUSDC = new PublicKey(
  "E6oEFEuYUYEt8U5mmKQvzvRUhRmgHCsVpmcyyrPEbCNy"
); // fake usdc

export const tokenIRMA = new PublicKey(
  "irmacFBRx7148dQ6qq1zpzUPq57Jr8V4vi5eXDxsDe1"
);

export const walletTokenAcct4IRMA = new PublicKey(
  "FdbTkQi4KMAyLT48FJxqYXM9UjXrtAbhQ63updyn5Zcp" // "5VNRZzJfk6Cfzxd1giXPitvf5yrdixLTwgKPdKTgLerJ"
); // IRMA container in principal's wallet

export const walletTokenAcct4USDC = new PublicKey(
  "Fr4Nt7yNe9dh6cnx6ZVNivjNbawZoyNQm5mXTMNNXr9R"
);

// export const chainlink = new PublicKey("HEvSKofvBgfaexv23kMabbYqxasxU3mQ4ibBMEmJWHny"); // chainlink

const nonceAcct = new PublicKey("HEbFxdDuoXwSPEND6bQ8tRzdLhjTmn7GeDWtqVnqFw9f"); // "GjbJUK45q4AHRa3GHtwgQbcYpGScBPCGkfXLMmZes3rk");

const lookupTableAddress = new PublicKey(
  "2NLsHEjVJdUWrZzungZd3KekTyf1mtYvrfR86cQRszGP" // "3hA6zazrv9cWJ8hMm4oHVgJcPvdSpmAtzZ2bfsRe1LrN"
);

let lookupTableAccount: AddressLookupTableAccount;
connection.getAddressLookupTable(lookupTableAddress).then((account) => {
  lookupTableAccount = account.value;
});

export const ixAdvanceNonce = async (compUnits: number) => {
  const advanceNonce = SystemProgram.nonceAdvance({
    noncePubkey: nonceAcct,
    authorizedPubkey: wallet.publicKey,
  });
  const modifyComputeUnits = ComputeBudgetProgram.setComputeUnitLimit({
    units: compUnits,
  });
  const addPriorityFee = ComputeBudgetProgram.setComputeUnitPrice({
    microLamports: 150000,
  });
  return [advanceNonce, modifyComputeUnits, addPriorityFee];
};

export const sendVersionedTx = async (
  instructions: TransactionInstruction[],
  additionalSigners: Signer[] = []
) => {
  const blockhash = (await connection.getLatestBlockhash()).blockhash;
  
  // construct a v0 compatible transaction `Message`
  if (!lookupTableAccount) {
    console.error("lookuptable null");
    throw new Error("lookup table null");
  }
  console.log("lookupTableAccount: ", lookupTableAccount.key.toBase58());
  const messageV0 = new TransactionMessage({
    payerKey: wallet.publicKey,
    recentBlockhash: blockhash,
    instructions,
  }).compileToV0Message([lookupTableAccount]);

  console.log("--> lookupTableAccount: ", lookupTableAccount.key.toBase58());

  // create a v0 transaction from the v0 message
  const transactionV0 = new VersionedTransaction(messageV0);

  console.log("+++++ transactionV0 done, now sending"); // , transactionV0);

  transactionV0.sign([wallet, ...additionalSigners]);

  return await connection.sendTransaction(transactionV0, {
    skipPreflight: false,
  });
};

export const getAccountInfo = async (anchorAccount: string) => {
  const connection = useHookConnection();
  return await connection.getParsedAccountInfo(
    new PublicKey(anchorAccount)
  );
}

export const fetchData = async () => {
  const connection = useHookConnection();
  const provider = useFakeProvider();
  console.log("---> openbook v2 public key: ", OPENBOOK_PROGRAM_ID.toBase58());
  const mkts: string[] = [];
  const uniqueMarkets: UIMarket[] = [];
  const markets = await findAllMarkets(
    connection,
    OPENBOOK_PROGRAM_ID,
    provider
  );
  console.log("==> markets length: ", markets.length);
  markets.forEach((mkt) => {
    if (!mkts.includes(mkt.market)) {
      mkts.push(mkt.market);
      uniqueMarkets.push(mkt);
    }
  });
  console.log("==> uniqueMarkets len: ", uniqueMarkets.length);
  return uniqueMarkets;
};

export const getMarket = async (
  client: OpenBookV2Client,
  publicKey: string
): Promise<MarketAccount> => {
  console.log("--> market selected: ", publicKey);
  let market = null;
  if (!!findAllMarkets) {
    const allMarkets = await findAllMarkets(client.connection);
    market = allMarkets.find((mkt) => mkt.market === publicKey);
  }
  return market ? market : ({} as MarketAccount);
};
