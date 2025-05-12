import Head from "next/head";
import { useEffect, useState } from "react";
import {
  Table,
  TableHeader,
  TableColumn,
  TableBody,
  TableRow,
  TableCell,
  getKeyValue,
} from "@nextui-org/react";

import React from "react";
import { 
  fetchData, 
  getMarket, 
  walletTokenAcct4IRMA, 
  wallet, 
  ixAdvanceNonce, 
  sendVersionedTx,
  openOrdersAdminE,
  getAccountInfo,
  tokenIRMA,
  tokenUSDC,
  closeMarketAdminE
} from "../utils/openbook";
import { BN } from "@coral-xyz/anchor";

import { LinkIcon } from "@heroicons/react/24/outline";
import {
  UIMarket,
  Market,
  MarketAccount,
  OpenOrders,
  Side,
  SideUtils,
  PlaceOrderType,
  PlaceOrderTypeUtils,
  SelfTradeBehavior,
  SelfTradeBehaviorUtils,
  nameToString,
  priceLotsToUi,
  OpenOrdersAccount,
  Order,
} from "../openbook";
import { useOpenbookClient } from "../hooks/useOpenbookClient";
import { Keypair, PublicKey, TransferParams, DecodedTransferInstruction } from "@solana/web3.js";
// import { useWallet } from "@solana/wallet-adapter-react";
import { toast } from "react-hot-toast";
import { TOKEN_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";
import { interestBearingMintInitializeInstructionData } from "@solana/spl-token";

function priceData(key) {
  const shiftedValue = key.shrn(64); // Shift right by 64 bits
  return shiftedValue.toNumber(); // Convert BN to a regular number
}

const OURMARKET = "EtYabFxQF7Kew5zegHKpEk9KgCde3bX93thEPXrm9Emj"; // "EU5dWkUXRgHRQBkvMhm8wtQcAF9Lgkpe9KsRuoyp9ke5";

export default function Home() {
  const [asks, setAsks] = useState([]);
  const [bids, setBids] = useState([]);
  const [markets, setMarkets] = useState([
    { market: "", baseMint: "", quoteMint: "", name: "" },
  ]);
  const [market, setMarket] = useState({} as MarketAccount);
  const [marketPubkey, setMarketPubkey] = useState(new PublicKey(OURMARKET));

  const columns = [
    {
      key: "name",
      label: "NAME",
    },
    {
      key: "market",
      label: "Pubkey",
    },
    {
      key: "baseMint",
      label: "Base Mint",
    },
    {
      key: "quoteMint",
      label: "Quote Mint",
    },
  ];

  const columnsBook = [
    {
      key: "owner",
      label: "OWNER",
    },
    {
      key: "quantity",
      label: "SIZE",
    },
    {
      key: "key",
      label: "PRICE",
    },
  ];

  const openbookClient = useOpenbookClient();

  useEffect(() => {
    console.log("================>>> markets len: ", markets.length);
    if (markets.length === 1 && markets[0].market === "") {
      fetchData()
      .then((res: UIMarket[]) => {
        setMarkets(res);
        const ourMarket: UIMarket = {
          market: OURMARKET,
          baseMint: tokenIRMA.toBase58(),
          quoteMint: tokenUSDC.toBase58(),
          name: "IRMA / USDC",
          timestamp: null
        };
        res.push(ourMarket);
        fetchMarket(OURMARKET);
        setMarketPubkey(new PublicKey(OURMARKET));
      })
      .catch((e) => {
        console.log(e);
      });
    }
  }, []);

  function priceDataToUI(key: { shrn: (arg0: number) => any; }) {
    const shiftedValue = key.shrn(64); // Shift right by 64 bits
    const priceLots = shiftedValue.toNumber(); // Convert BN to a regular number

    return priceLotsToUi(market, priceLots);
  }

  function enumerate<T>(items: Generator<T, any, unknown>) {
    let arrayOfT: T[] = [];
    for (const item of items) {
      arrayOfT.push(item);
    }
    return arrayOfT;
  }

  const fetchMarket = async (key: string) => {
    const marketPk = new PublicKey(key,);
    const marketAcct = await Market.load(openbookClient, marketPk);
    const orderBook = await marketAcct.loadOrderBook();

    const booksideAsks = await orderBook.loadAsks();
    const booksideBids = await orderBook.loadBids();

    setMarket(marketAcct.account);
    setMarketPubkey(new PublicKey(key));

    const asks = enumerate(booksideAsks.items());
    setAsks(asks.map(o => o.leafNode));
    const bids = enumerate(booksideBids.items());
    setBids(bids.map(o => o.leafNode));
  };

  const linkedPk = (pk: string) => (
    <div>
      {pk}
      <a
        href={`https://solscan.io/account/${pk}`}
        target="_blank"
        className="pl-2"
      >
        <LinkIcon className="w-4 h-4 inline" />
      </a>
    </div>
  );

  const crankMarket = async () => {
    let accountsToConsume = await openbookClient.getAccountsToConsume(market);
    console.log("accountsToConsume", accountsToConsume);
    console.log("marketPubkey: ", marketPubkey.toBase58());

    try {
      if (accountsToConsume.length > 0) {
        const events = await openbookClient.consumeEventsIx(marketPubkey, market, new BN(5), accountsToConsume);
        console.log("consume events tx", events.keys);
        // toast("Consume events tx: " + tx.toString());
      }
    }
    catch (error) {
      console.error(error);
    };
  };

  // place uneditable order
  const placeOrder = async () => {
      console.log("market pub key picked up from app:");
      console.log(marketPubkey.toBase58());

    try {
      const openOrdersDelegate = null;
      const args: {
        side: Side,
        priceLots: BN,
        maxBaseLots: BN,
        maxQuoteLotsIncludingFees: BN,
        clientOrderId: BN,
        orderType: PlaceOrderType,
        expiryTimestamp: undefined,
        selfTradeBehavior: SelfTradeBehavior,
        limit: number
      } = {
        side: SideUtils.Ask,
        priceLots: new BN(1_000_000),
        maxBaseLots: new BN(9_223_300_000_000n),
        maxQuoteLotsIncludingFees: new BN(92_233_000_000n),
        clientOrderId: new BN(1000),
        orderType: PlaceOrderTypeUtils.Limit,
        expiryTimestamp: undefined,
        selfTradeBehavior: SelfTradeBehaviorUtils.AbortTransaction,
        limit: 200 // count of orders to fill - must be unlimited
      };
      const marketVault = /* args.side === Side.Bid ? market.marketQuoteVault : */ market.marketBaseVault;

      const [openOrdersIxs, openOrdersPubkey] = await openbookClient.createOpenOrdersIx(
        marketPubkey,
        "RockStable Open Orders",
        wallet.publicKey,
        openOrdersDelegate,
        // indexerAccount.publicKey
      );

      const copyBytes = (from: Buffer, to: Uint8Array, j: number) => {
        const b = 32*j + 8;
        let i = b;
        for (; i < b + 32; i++) {
          to[i - b] = from[i];
        }
      }

      const openOrdersAcct = openOrdersPubkey.toBase58();
      console.log("--> open orders: ", openOrdersAcct);
      if (openOrdersAcct.length === 44) {
        console.log("--> examine openOrdersAccount to make sure it doesn't belong to another market");
        const acctInfo = await getAccountInfo(openOrdersAcct);
        console.log("--> account info: ", acctInfo);
        if (!!acctInfo.value) {
          const bufPtr = new Uint8Array(32);
          copyBytes(acctInfo.value.data as Buffer, bufPtr, 0);
          const owner = new PublicKey(bufPtr);
          copyBytes(acctInfo.value.data as Buffer, bufPtr, 1);
          const market = new PublicKey(bufPtr);
          const ownerStr = owner.toBase58();
          const marketStr = market.toBase58();
          console.log("--> owner: ", ownerStr);
          console.log("--> market: ", marketStr);
          if (marketStr !== marketPubkey.toBase58()) {
            // we got an open orders acct that belong to a previously defunct market (that we also created):
            // delete this openOrdersAcct and call createOpenOrdersInstruction again
            console.log("error: suggested open orders acct belongs to defunct market: ", marketStr);
            throw new Error("suggested open orders account belongs to defunct market"); // just exit for now
          }
        }
      }

      const ixPlaceOrder = await openbookClient.program.methods
      .placeOrder(args)
      .accounts({
        signer: openOrdersDelegate != null
              ? openOrdersDelegate.publicKey
              : wallet.publicKey,
        market: marketPubkey,
        asks: market.asks,
        bids: market.bids,
        marketVault,
        eventHeap: market.eventHeap,
        openOrdersAccount: openOrdersPubkey,
        oracleA: market.oracleB.key, // nothing
        oracleB: market.oracleB.key, // nothing
        tokenProgram: TOKEN_PROGRAM_ID,
        // if below set to market.openOrdersAdmin.key: Error: failed to send transaction: Transaction signature verification failure;
        // wallet.publicKey, Error: failed to send transaction: Transaction simulation failed: Error processing Instruction 3: custom program error: 0x7d6 (2006)
        // [2006 is ConstraintSeeds error]
        openOrdersAdmin: openOrdersAdminE.publicKey,
        userTokenAccount: walletTokenAcct4IRMA
      })
      // .remainingAccounts(accountsMeta)
      .instruction();

      const signers = [];
      if (openOrdersDelegate != null) {
          signers.push(openOrdersDelegate);
      }
      signers.push(wallet);
      signers.push(openOrdersAdminE);

      // const ixMoveNonce2 = await ixAdvanceNonce(100000);

      const tx = await sendVersionedTx([/*...indexerTxs,*/ ...openOrdersIxs, ixPlaceOrder], signers);
      console.log("placeOrder events tx", tx);
    } catch (error) {
      console.error(error);
    };
  };

  const closeOpenOrderAccount = async () => {
    const openOrdersAcct = new PublicKey("BpYUcqeQ3cjQDCciWJWhTSrKxDqk4o2xEyqYB23ZCiYh");
    const indexer = Keypair.fromSeed(openOrdersAcct.toBytes()).publicKey;
    const ixOpenOrdersIndexer = await openbookClient.createOpenOrdersIndexerIx(indexer);

    const marketData = market; // await openbookClient.getMarketAccount(marketPubkey);
    const [ixCloseOpenOrdersAcct, signer] = await openbookClient.closeOpenOrdersAccountIx(
      wallet, 
      openOrdersAdminE.publicKey,
      wallet.publicKey,
      indexer
    );
    const ixMoveNonce = await ixAdvanceNonce(20000);
    const tx = await sendVersionedTx([...ixMoveNonce, ixOpenOrdersIndexer, ixCloseOpenOrdersAcct], signer);
    console.log("got something from closing of openOrdersAccount: ", tx);
  }

  const closeMarket = async () => {
  const [ixCloseMarket, signers] = await openbookClient.closeMarketIx(marketPubkey, market, wallet.publicKey, closeMarketAdminE);
    // const result = await Market.load(openbookClient, marketPubkey).closeMarket(marketPubkey, marketData, wallet.publicKey, closeMarketAdminE);
    const ixMoveNonce = await ixAdvanceNonce(20000);
    const tx = await sendVersionedTx([...ixMoveNonce, ixCloseMarket], signers);
    console.log("close market result: ", tx);
  }

  return (
    <div>
      <Head>
        <title>Openbook</title>
        <link rel="icon" href="/favicon.ico" />
      </Head>

      <div className="w-full h-full relative ">
        <div className="flex flex-col gap-3 pb-2.5">
          <Table
            isStriped
            selectionMode="single"
            aria-label="Markets"
            onRowAction={async (key) => fetchMarket(key.toString())}
          >
            <TableHeader columns={columns}>
              {(column) => (
                <TableColumn key={column.key}>{column.label}</TableColumn>
              )}
            </TableHeader>
            <TableBody items={markets}>
              {(item) => (
                <TableRow key={item.market}>
                  {(columnKey) => (
                    <TableCell>
                      {columnKey == "name"
                        ? getKeyValue(item, columnKey)
                        : linkedPk(getKeyValue(item, columnKey))}
                    </TableCell>
                  )}
                </TableRow>
              )}
            </TableBody>
          </Table>
        </div>

        {market.asks ? (
          <div>
            <div className="grid grid-cols-2 gap-2 text-center border-r-4 border-b-4 border-l-4">
              <div className="">
                <p className="font-bold">Name </p>
                {market.asks ? nameToString(market.name) : ""}
                <p className="font-bold">Base Mint </p>
                {market.asks ? market.baseMint.toString() : ""}
                <p className="font-bold">Quote Mint </p>
                {market.asks ? market.quoteMint.toString() : ""}
                <p className="font-bold">Bids </p>
                {market.asks ? market.bids.toString() : ""}
                <p className="font-bold">Asks </p>
                {market.asks ? market.asks.toString() : ""}
                <p className="font-bold">Event Heap </p>
                {market.asks ? market.eventHeap.toString() : ""}
              </div>

              <div className="">
                <p className="font-bold">Base Deposits </p>
                {market.asks ? market.baseDepositTotal.toString() : ""}
                <p className="font-bold">Quote Deposits </p>
                {market.asks ? market.quoteDepositTotal.toString() : ""}
                <p className="font-bold">Taker Fees </p>
                {market.asks ? market.takerFee.toString() : ""}
                <p className="font-bold">Maker Fees </p>
                {market.asks ? market.makerFee.toString() : ""}
                <p className="font-bold">Base Lot Size </p>
                {market.asks ? market.baseLotSize.toString() : ""}
                <p className="font-bold">Quote Lot Size </p>
                {market.asks ? market.quoteLotSize.toString() : ""}
                <p className="font-bold">Base Decimals </p>
                {market.asks ? market.baseDecimals : ""}
                <p className="font-bold">Quote Decimals </p>
                {market.asks ? market.quoteDecimals : ""}
              </div>
            </div>

            <button
              className="items-center text-center bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded"
              onClick={(e: any) => crankMarket()}
            >
              CRANK
            </button>

            <button
              className="items-center text-center bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded"
              onClick={(e: any) => placeOrder()}
            >
              PLACE ORDER
            </button>

            <button
              className="items-center text-center bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded"
              onClick={(e: any) => closeOpenOrderAccount()}
            >
              CLOSE OPEN_ORDER_ACCOUNT
            </button>

            <button
              className="items-center text-center bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded"
              onClick={(e: any) => closeMarket()}
            >
              CLOSE MARKET
            </button>

            <div>
              <h3 className="text-center mt-8 mb-5 text-xl">
                ASKS -------- The Book -------- BIDS
              </h3>
            </div>

            <div className="grid grid-cols-2 gap-2 border-2">
              <Table isStriped selectionMode="single" aria-label="OrderBook">
                <TableHeader className="text-left" columns={columnsBook}>
                  {(column) => (
                    <TableColumn key={column.key}>{column.label}</TableColumn>
                  )}
                </TableHeader>
                <TableBody items={asks}>
                  {(item) => (
                    <TableRow key={priceData(item.key)}>
                      {(columnKey) => (
                        <TableCell>
                          {columnKey == "owner"
                            ? getKeyValue(item, columnKey)
                                .toString()
                                .substring(0, 4) +
                              ".." +
                              getKeyValue(item, columnKey).toString().slice(-4)
                            : columnKey == "quantity"
                            ? getKeyValue(item, columnKey).toString()
                            : priceDataToUI(item.key)}
                        </TableCell>
                      )}
                    </TableRow>
                  )}
                </TableBody>
              </Table>

              <Table isStriped selectionMode="single" aria-label="OrderBook">
                <TableHeader columns={columnsBook}>
                  {(column) => (
                    <TableColumn key={column.key}>{column.label}</TableColumn>
                  )}
                </TableHeader>
                <TableBody items={bids}>
                  {(item) => (
                    <TableRow key={priceData(item.key)}>
                      {(columnKey) => (
                        <TableCell>
                          {columnKey == "owner"
                            ? getKeyValue(item, columnKey)
                                .toString()
                                .substring(0, 4) +
                              ".." +
                              getKeyValue(item, columnKey).toString().slice(-4)
                            : columnKey == "quantity"
                            ? getKeyValue(item, columnKey).toString()
                            : priceDataToUI(item.key)}
                        </TableCell>
                      )}
                    </TableRow>
                  )}
                </TableBody>
              </Table>
            </div>
          </div>
        ) : (
          <h1 className="text-center">This market has been closed!</h1>
        )}

        <footer className="bg-white rounded-lg shadow m-4 dark:bg-gray-800">
          <div className="w-full mx-auto max-w-screen-xl p-4 md:flex md:items-center md:justify-between">
            <span className="text-sm text-gray-500 sm:text-center dark:text-gray-400">
              © 2023{" "}
              <a
                href="https://twitter.com/openbookdex"
                className="hover:underline"
              >
                Openbook Team
              </a>
              . All Rights Reserved.
            </span>
            <ul className="flex flex-wrap items-center mt-3 text-sm font-medium text-gray-500 dark:text-gray-400 sm:mt-0">
              <li>
                <a
                  href="https://twitter.com/openbookdex"
                  className="mr-4 hover:underline md:mr-6"
                >
                  Twitter
                </a>
              </li>
              <li>
                <a
                  href="https://github.com/openbook-dex"
                  className="mr-4 hover:underline md:mr-6"
                >
                  GitHub
                </a>
              </li>
              <li>
                <a
                  href="gofuckyourselfifyouwanttocontactus@weloveyou.shit"
                  className="hover:underline"
                >
                  Contact
                </a>
              </li>
            </ul>
          </div>
        </footer>
      </div>
    </div>
  );
}
