---------------------------- MODULE MC_TwoChains ----------------------------

\* @type: () => Int;
MaxHeight == 4
\* @type: () => Int;
NrClientsChainA == 2
\* @type: () => Int;
NrClientsChainB == 2
\* @type: () => Set(Str);
ClientIDsChainA == {"B1", "B2"}
\* @type: () => Set(Str);
ClientIDsChainB == {"A1", "A2"}

VARIABLES 
    \* @type: CHAINSTORE;
    chainAstore, \* store of ChainA
    \* @type: CHAINSTORE;
    chainBstore, \* store of ChainB
    \* @type: Set(DATAGRAM);
    datagramsChainA, \* set of datagrams incoming to ChainA
    \* @type: Set(DATAGRAM);
    datagramsChainB, \* set of datagrams incoming to ChainB
    \* @type: Str -> [created: Bool, updated: Bool];
    history \* history variable

INSTANCE ICS02TwoChainsEnvironment
=============================================================================