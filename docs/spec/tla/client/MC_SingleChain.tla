--------------------------- MODULE MC_SingleChain ---------------------------

\* @type: () => Int;
MaxHeight == 4
\* @type: () => Int;
NrClientsChainA == 2
\* @type: () => Set(Str);
ClientIDsChainA == {"B1", "B2"}

VARIABLES 
    \* @type: CHAINSTORE;
    chainAstore, \* store of ChainA
    \* @type: Set(DATAGRAM);
    datagramsChainA, \* set of datagrams incoming to ChainA
    \* @type: Str -> [created: Bool, updated: Bool];
    history \* history variable

INSTANCE ICS02SingleChainEnvironment

=============================================================================
