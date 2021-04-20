------------------------- MODULE MC_IBCTokenTransfer ------------------------

\* @type: () => Int;
MaxHeight == 5
\* @type: () => Int;
MaxPacketSeq == 5
\* @type: () => Int;
MaxBalance == 5
\* @type: () => Str;
NativeDenominationChainA == "atom"
\* @type: () => Str;
NativeDenominationChainB == "eth"

VARIABLES 
    \* @type: CHAINSTORE;
    chainAstore, \* store of ChainA
    \* @type: CHAINSTORE;
    chainBstore, \* store of ChainB
    \* @type: Seq(DATAGRAM);
    packetDatagramsChainA, \* sequence of packet datagrams incoming to ChainA
    \* @type: Seq(DATAGRAM);
    packetDatagramsChainB, \* sequence of packet datagrams incoming to ChainB
    \* @type: Seq(LOGENTRY);
    packetLog, \* packet log
    \* @type: Int;
    appPacketSeqChainA, \* packet sequence number from the application on ChainA
    \* @type: Int;
    appPacketSeqChainB, \* packet sequence number from the application on ChainB
    \* @type: ACCOUNT -> Int;
    accounts, \* a map from chainIDs and denominations to account balances
    \* @type: ACCOUNT -> Int;
    escrowAccounts \* a map from channelIDs and denominations to escrow account balances

INSTANCE IBCTokenTransfer

=============================================================================