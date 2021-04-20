----------------------------- MODULE MC_IBCCore -----------------------------

EXTENDS IBCCoreDefinitions

\* @type: () => Int;
MaxHeight == 2
\* @type: () => Int;
MaxVersion == 2
\* @type: () => Int;
MaxPacketSeq == 1
\* @type: () => Bool;
ClientDatagramsRelayer1 == TRUE
\* @type: () => Bool;
ClientDatagramsRelayer2 == FALSE
\* @type: () => Bool;
ConnectionDatagramsRelayer1 == TRUE
\* @type: () => Bool;
ConnectionDatagramsRelayer2 == FALSE
\* @type: () => Bool;
ChannelDatagramsRelayer1 == TRUE
\* @type: () => Bool;
ChannelDatagramsRelayer2 == FALSE
\* @type: () => Bool;
PacketDatagramsRelayer1 == TRUE
\* @type: () => Bool;
PacketDatagramsRelayer2 == FALSE
\* @type: () => Str;
ChannelOrdering == "UNORDERED"

VARIABLES 
    \* @type: CHAINSTORE;
    chainAstore, \* chain store of ChainA
    \* @type: CHAINSTORE;
    chainBstore, \* chain store of ChainB
    \* @type: Set(DATAGRAM);
    incomingDatagramsChainA, \* set of (client, connection, channel) datagrams incoming to ChainA
    \* @type: Set(DATAGRAM);
    incomingDatagramsChainB, \* set of (client, connection, channel) datagrams incoming to ChainB
    \* @type: Seq(DATAGRAM);
    incomingPacketDatagramsChainA, \* sequence of packet datagrams incoming to ChainA
    \* @type: Seq(DATAGRAM);
    incomingPacketDatagramsChainB, \* sequence of packet datagrams incoming to ChainB
    \* @type: Str -> Int;
    relayer1Heights, \* the client heights of Relayer1
    \* @type: Str -> Int;
    relayer2Heights, \* the client heights of Relayer2
    \* @type: Str -> Set(DATAGRAM);
    outgoingDatagrams, \* sets of (client, connection, channel) datagrams outgoing of the relayers
    \* @type: Str -> Seq(DATAGRAM);
    outgoingPacketDatagrams, \* sequences of packet datagrams outgoing of the relayers
    \* @type: Bool;
    closeChannelA, \* flag that triggers closing of the channel end at ChainA
    \* @type: Bool;
    closeChannelB, \* flag that triggers closing of the channel end at ChainB
    \* @type: HISTORY;
    historyChainA, \* history variables for ChainA
    \* @type: HISTORY;
    historyChainB, \* history variables for ChainB 
    \* @type: Seq(LOGENTRY);
    packetLog, \* packet log 
    \* @type: Int;
    appPacketSeqChainA, \* packet sequence number from the application on ChainA
    \* @type: Int;
    appPacketSeqChainB \* packet sequence number from the application on ChainB

INSTANCE IBCCore
=============================================================================